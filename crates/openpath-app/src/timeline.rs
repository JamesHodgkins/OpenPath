//! Pure timeline geometry for the Gantt chart: dates ↔ x coordinates and bar rectangles.
//! Kept free of egui painting so it can be unit-tested.

use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime};

/// Maps instants to horizontal pixel offsets. Day granularity is linear in wall-clock time
/// (a bar spans its real start→finish), so weekends appear as gaps in the shading, not in x.
#[derive(Debug, Clone, Copy)]
pub struct Timeline {
    /// Left edge of the chart (midnight of the first visible day).
    pub origin: NaiveDate,
    pub px_per_day: f32,
}

impl Timeline {
    pub fn new(origin: NaiveDate, px_per_day: f32) -> Self {
        Timeline { origin, px_per_day }
    }

    /// Pixel offset of `t` from the chart's left edge.
    pub fn x(&self, t: NaiveDateTime) -> f32 {
        let secs = (t - self.origin.and_hms_opt(0, 0, 0).unwrap()).num_seconds();
        secs as f32 / 86_400.0 * self.px_per_day
    }

    pub fn day_x(&self, day: NaiveDate) -> f32 {
        (day - self.origin).num_days() as f32 * self.px_per_day
    }

    /// Total chart width needed to show through `last` (inclusive) plus padding days.
    pub fn width_for(&self, last: NaiveDate, pad_days: i64) -> f32 {
        (((last - self.origin).num_days() + 1 + pad_days).max(1)) as f32 * self.px_per_day
    }

    /// Days from `origin` for `n` consecutive days.
    pub fn days(&self, n: i64) -> impl Iterator<Item = NaiveDate> + '_ {
        (0..n).map(move |i| self.origin + Duration::days(i))
    }
}

pub fn is_weekend(d: NaiveDate) -> bool {
    d.weekday().number_from_monday() >= 6
}

/// Monday on or before `d`; the chart starts on a week boundary one week before the project.
pub fn week_start(d: NaiveDate) -> NaiveDate {
    d - Duration::days(d.weekday().num_days_from_monday() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn x_is_linear_in_time() {
        let tl = Timeline::new(d(2026, 11, 2), 24.0);
        assert_eq!(tl.x(d(2026, 11, 2).and_hms_opt(0, 0, 0).unwrap()), 0.0);
        assert_eq!(tl.x(d(2026, 11, 3).and_hms_opt(0, 0, 0).unwrap()), 24.0);
        assert_eq!(tl.x(d(2026, 11, 2).and_hms_opt(12, 0, 0).unwrap()), 12.0);
        assert_eq!(tl.day_x(d(2026, 11, 9)), 168.0);
    }

    #[test]
    fn week_start_is_monday() {
        assert_eq!(week_start(d(2026, 11, 11)), d(2026, 11, 9)); // Wed → Mon
        assert_eq!(week_start(d(2026, 11, 9)), d(2026, 11, 9));
        assert_eq!(week_start(d(2026, 11, 15)), d(2026, 11, 9)); // Sun → Mon
    }

    #[test]
    fn weekends() {
        assert!(is_weekend(d(2026, 11, 14)));
        assert!(is_weekend(d(2026, 11, 15)));
        assert!(!is_weekend(d(2026, 11, 13)));
    }

    #[test]
    fn width_includes_padding() {
        let tl = Timeline::new(d(2026, 11, 2), 10.0);
        assert_eq!(tl.width_for(d(2026, 11, 2), 0), 10.0);
        assert_eq!(tl.width_for(d(2026, 11, 4), 7), 100.0);
    }
}
