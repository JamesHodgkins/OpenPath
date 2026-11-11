//! Calendar arithmetic (§5.1). All date math goes through this module.
//!
//! Rules:
//! - Intervals are half-open `[start, end)`.
//! - `next_working_instant` returns the start of the next interval when `t` is not working.
//! - `prev_working_instant` returns the end of the previous interval when `t` is not working.
//! - `working_minutes_between` clamps `a` and `b` inward to working time and sums overlaps.
//! - `add_working_minutes(t, 0)` returns `next_working_instant(t)`.
//! - Dates are minute-aligned; DST is wall-clock (no UTC conversion).

#![forbid(unsafe_code)]

use chrono::{Datelike, Duration as ChronoDuration, NaiveDate, NaiveDateTime, Timelike};
use openpath_core::{Calendar, TimeInterval};

/// Days to scan when searching for the next/previous working instant. A calendar with
/// no working time at all within this horizon is treated as empty and the input is
/// returned unchanged (a pathological calendar rejected by import validation).
const HORIZON_DAYS: i64 = 3660;

/// Minutes since midnight for a date-time, truncated to the minute.
fn minute_of(t: NaiveDateTime) -> i64 {
    (t.time().num_seconds_from_midnight() as i64) / 60
}

/// Build a date-time from a date and minutes-since-midnight, rolling `minute >= 1440`
/// into the next day so a `[00:00, 24:00)` interval ends at next-day 00:00.
fn with_minute(date: NaiveDate, minute: i64) -> NaiveDateTime {
    let day_shift = minute.div_euclid(1440);
    let m = minute.rem_euclid(1440);
    let date = date + ChronoDuration::days(day_shift);
    date.and_hms_opt((m / 60) as u32, (m % 60) as u32, 0)
        .expect("minute normalized to 0..1440")
}

/// Working intervals for a date, honoring exceptions. `None` = non-working day.
fn day_intervals(date: NaiveDate, cal: &Calendar) -> Option<&[TimeInterval]> {
    for ex in &cal.exceptions {
        if ex.date == date {
            return if ex.working {
                Some(&ex.intervals)
            } else {
                None
            };
        }
    }
    let idx = date.weekday().num_days_from_monday() as usize;
    let day = &cal.week[idx];
    if day.is_empty() {
        None
    } else {
        Some(day)
    }
}

/// The interval containing a working instant.
fn containing_interval(date: NaiveDate, minute: i64, cal: &Calendar) -> Option<&TimeInterval> {
    day_intervals(date, cal).and_then(|intervals| {
        intervals
            .iter()
            .find(|iv| iv.start <= minute && minute < iv.end)
    })
}

/// True when `t` falls inside a working interval.
pub fn is_working(t: NaiveDateTime, cal: &Calendar) -> bool {
    containing_interval(t.date(), minute_of(t), cal).is_some()
}

/// Earliest working instant at or after `t`. Falls back to `t` if the calendar has no
/// working time within the horizon.
pub fn next_working_instant(t: NaiveDateTime, cal: &Calendar) -> NaiveDateTime {
    next_working_instant_opt(t, cal).unwrap_or(t)
}

fn next_working_instant_opt(t: NaiveDateTime, cal: &Calendar) -> Option<NaiveDateTime> {
    if is_working(t, cal) {
        return Some(t);
    }
    let m = minute_of(t);
    let date = t.date();
    if let Some(intervals) = day_intervals(date, cal) {
        if let Some(iv) = intervals.iter().find(|iv| iv.start > m) {
            return Some(with_minute(date, iv.start));
        }
    }
    for d in 1..=HORIZON_DAYS {
        let date2 = date + ChronoDuration::days(d);
        if let Some(intervals) = day_intervals(date2, cal) {
            if let Some(iv) = intervals.first() {
                return Some(with_minute(date2, iv.start));
            }
        }
    }
    None
}

/// Latest working instant at or before `t`. Falls back to `t` if the calendar has no
/// working time within the horizon. When `t` is non-working this is the *end* of the
/// previous interval (an exclusive boundary).
pub fn prev_working_instant(t: NaiveDateTime, cal: &Calendar) -> NaiveDateTime {
    prev_working_instant_opt(t, cal).unwrap_or(t)
}

fn prev_working_instant_opt(t: NaiveDateTime, cal: &Calendar) -> Option<NaiveDateTime> {
    if is_working(t, cal) {
        return Some(t);
    }
    let m = minute_of(t);
    let date = t.date();
    if let Some(intervals) = day_intervals(date, cal) {
        if let Some(iv) = intervals.iter().rev().find(|iv| iv.end <= m) {
            return Some(with_minute(date, iv.end));
        }
    }
    for d in 1..=HORIZON_DAYS {
        let date2 = date - ChronoDuration::days(d);
        if let Some(intervals) = day_intervals(date2, cal) {
            if let Some(iv) = intervals.last() {
                return Some(with_minute(date2, iv.end));
            }
        }
    }
    None
}

/// Backward-anchor cursor: `(date, interval index, minutes from the interval end back to
/// `t`)`. When `t` is working, `from_end` is the working distance from `t` to the interval
/// end; when non-working, the anchor is the last interval at/before `t` with `from_end = 0`.
fn anchor_backward(t: NaiveDateTime, cal: &Calendar) -> Option<(NaiveDate, usize, i64)> {
    let date = t.date();
    let m = minute_of(t);
    if is_working(t, cal) {
        let intervals = day_intervals(date, cal)?;
        let idx = intervals
            .iter()
            .position(|iv| iv.start <= m && m < iv.end)?;
        let from_end = intervals[idx].end - m;
        Some((date, idx, from_end))
    } else {
        if let Some(intervals) = day_intervals(date, cal) {
            if let Some(idx) = intervals.iter().rposition(|iv| iv.end <= m) {
                return Some((date, idx, 0));
            }
        }
        for d in 1..=HORIZON_DAYS {
            let date2 = date - ChronoDuration::days(d);
            if let Some(intervals) = day_intervals(date2, cal) {
                return Some((date2, intervals.len() - 1, 0));
            }
        }
        None
    }
}

/// The last interval of the previous working day before `date`, as `(date, index)`.
fn previous_day_last_interval(date: NaiveDate, cal: &Calendar) -> Option<(NaiveDate, usize)> {
    for d in 1..=HORIZON_DAYS {
        let date2 = date - ChronoDuration::days(d);
        if let Some(intervals) = day_intervals(date2, cal) {
            return Some((date2, intervals.len() - 1));
        }
    }
    None
}

/// Working minutes from `a` (inclusive) to `b` (exclusive), clamping both ends inward to
/// working time. It is not simply `b - a`.
pub fn working_minutes_between(a: NaiveDateTime, b: NaiveDateTime, cal: &Calendar) -> i64 {
    if a >= b {
        return 0;
    }
    let mut total = 0i64;
    let mut date = a.date();
    while date <= b.date() {
        if let Some(intervals) = day_intervals(date, cal) {
            for iv in intervals {
                let iv_start = with_minute(date, iv.start);
                let iv_end = with_minute(date, iv.end);
                let lo = iv_start.max(a);
                let hi = iv_end.min(b);
                if lo < hi {
                    total += (hi - lo).num_minutes();
                }
            }
        }
        date += ChronoDuration::days(1);
    }
    total
}

/// Signed working minutes from `a` to `b`: positive when `b` is after `a`, negative when
/// before. Used for slack, which may be negative (§5.6).
pub fn working_minutes_signed(a: NaiveDateTime, b: NaiveDateTime, cal: &Calendar) -> i64 {
    if a <= b {
        working_minutes_between(a, b, cal)
    } else {
        -working_minutes_between(b, a, cal)
    }
}

/// Add `minutes` working minutes to `t`. Positive moves forward, negative moves backward.
/// `add_working_minutes(t, 0)` returns `next_working_instant(t)`.
pub fn add_working_minutes(t: NaiveDateTime, minutes: i64, cal: &Calendar) -> NaiveDateTime {
    if minutes < 0 {
        return subtract_working_minutes(t, -minutes, cal);
    }
    if minutes == 0 {
        return next_working_instant(t, cal);
    }
    let mut cur = next_working_instant(t, cal);
    let mut remaining = minutes;
    loop {
        let date = cur.date();
        let m = minute_of(cur);
        let iv = containing_interval(date, m, cal).expect("cursor is a working instant");
        let avail = iv.end - m;
        if remaining <= avail {
            return with_minute(date, m + remaining);
        }
        remaining -= avail;
        cur = next_working_instant(with_minute(date, iv.end), cal);
    }
}

fn subtract_working_minutes(t: NaiveDateTime, minutes: i64, cal: &Calendar) -> NaiveDateTime {
    debug_assert!(minutes >= 0);
    if minutes == 0 {
        return prev_working_instant(t, cal);
    }
    let Some((mut date, mut idx, mut from_end)) = anchor_backward(t, cal) else {
        return t;
    };
    let mut remaining = minutes;
    loop {
        let intervals = day_intervals(date, cal).expect("anchor day has intervals");
        let iv = intervals[idx];
        let avail = (iv.end - iv.start) - from_end;
        if remaining <= avail {
            return with_minute(date, iv.end - from_end - remaining);
        }
        remaining -= avail;
        from_end = 0;
        if idx > 0 {
            idx -= 1;
        } else {
            let Some((d2, i2)) = previous_day_last_interval(date, cal) else {
                return with_minute(date, iv.start);
            };
            date = d2;
            idx = i2;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn dt(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(y, mo, d)
            .unwrap()
            .and_hms_opt(h, mi, 0)
            .unwrap()
    }

    #[test]
    fn zero_add_lands_on_working_time() {
        let cal = Calendar::standard("std");
        // Saturday 09:00 is non-working; next_working_instant -> Monday 08:00.
        let sat = dt(2026, 11, 7, 9, 0); // 2026-11-07 is a Saturday
        assert!(!is_working(sat, &cal));
        let next = next_working_instant(sat, &cal);
        assert_eq!(next, dt(2026, 11, 9, 8, 0));
        assert!(is_working(next, &cal));
        assert_eq!(add_working_minutes(next, 0, &cal), next);
    }

    #[test]
    fn add_working_minutes_idempotent_property() {
        // §5.1 property 1: add_working_minutes(next_working_instant(t), 0) == next_working_instant(t)
        let cal = Calendar::standard("std");
        for day in 2..=8 {
            let t = dt(2026, 11, day, 10, 30);
            let nwi = next_working_instant(t, &cal);
            assert_eq!(add_working_minutes(nwi, 0, &cal), nwi);
        }
    }

    #[test]
    fn working_minutes_roundtrip_property() {
        // §5.1 property 2: working_minutes_between(t, add_working_minutes(t, m)) == m for working t, m >= 0
        let cal = Calendar::standard("std");
        let t = dt(2026, 11, 9, 8, 0); // Monday 08:00
        assert!(is_working(t, &cal));
        for m in [0, 1, 59, 60, 240, 479, 480, 481, 960, 2400] {
            let end = add_working_minutes(t, m, &cal);
            assert_eq!(working_minutes_between(t, end, &cal), m, "m = {m}");
        }
    }

    #[test]
    fn finish_lands_on_interval_boundary() {
        // A full 08:00–17:00 day (540 working minutes) starting at 08:00 finishes at 17:00,
        // not 16:59 (half-open boundary, §5.1).
        let cal = Calendar::standard("std");
        let start = dt(2026, 11, 9, 8, 0);
        assert_eq!(
            add_working_minutes(start, 540, &cal),
            dt(2026, 11, 9, 17, 0)
        );
    }

    #[test]
    fn lunch_break_is_skipped() {
        // 08:00–12:00 and 13:00–17:00: 480 working minutes from 08:00 lands at 17:00.
        let day = vec![TimeInterval::new(480, 720), TimeInterval::new(780, 1020)];
        let mut cal = Calendar::standard("lunch");
        for slot in cal.week.iter_mut().take(5) {
            *slot = day.clone();
        }
        let start = dt(2026, 11, 9, 8, 0);
        assert_eq!(
            add_working_minutes(start, 480, &cal),
            dt(2026, 11, 9, 17, 0)
        );
        assert!(!is_working(dt(2026, 11, 9, 12, 30), &cal));
        assert_eq!(
            working_minutes_between(start, dt(2026, 11, 9, 17, 0), &cal),
            480
        );
    }

    #[test]
    fn negative_minutes_go_backward() {
        let cal = Calendar::standard("std");
        let finish = dt(2026, 11, 9, 17, 0); // Monday, non-working boundary
        let start = add_working_minutes(finish, -540, &cal);
        assert_eq!(start, dt(2026, 11, 9, 8, 0));
        // round-trip
        assert_eq!(add_working_minutes(start, 540, &cal), finish);
    }

    #[test]
    fn twenty_four_hour_calendar_is_elapsed() {
        // §5.1 property 4: a 24-hour calendar behaves like elapsed time.
        let cal = Calendar::twenty_four_seven("24/7");
        let t = dt(2026, 11, 9, 14, 30);
        for m in [0, 1, 60, 1440, 5000] {
            assert_eq!(
                add_working_minutes(t, m, &cal),
                t + ChronoDuration::minutes(m)
            );
        }
        let b = t + ChronoDuration::minutes(5000);
        assert_eq!(working_minutes_between(t, b, &cal), 5000);
    }

    #[test]
    fn exception_overrides_weekday() {
        let mut cal = Calendar::standard("std");
        // 2026-11-09 is a Monday (working). Mark it non-working.
        cal.exceptions.push(openpath_core::CalendarException {
            date: NaiveDate::from_ymd_opt(2026, 11, 9).unwrap(),
            working: false,
            intervals: Vec::new(),
        });
        assert!(!is_working(dt(2026, 11, 9, 10, 0), &cal));
        // next working day is Tuesday.
        assert_eq!(
            next_working_instant(dt(2026, 11, 9, 10, 0), &cal),
            dt(2026, 11, 10, 8, 0)
        );
    }

    #[test]
    fn no_result_falls_in_non_working_interval() {
        // §5.1 property 3: a task start or end is never mid-gap, except the finish may be a
        // half-open boundary. Check a sampling of durations all land on working time or a
        // boundary that immediately follows working time.
        let cal = Calendar::standard("std");
        let start = dt(2026, 11, 9, 8, 0);
        for m in 1..=540 {
            let end = add_working_minutes(start, m, &cal);
            let minute = minute_of(end);
            let date = end.date();
            let idx = date.weekday().num_days_from_monday() as usize;
            let day = &cal.week[idx];
            let working = day.iter().any(|iv| iv.start <= minute && minute < iv.end);
            let boundary = day.iter().any(|iv| iv.end == minute);
            assert!(working || boundary, "m={m} end={end:?}");
        }
    }
}
