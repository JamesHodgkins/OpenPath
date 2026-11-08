//! MSPDI (Microsoft Project XML) writer — Phase 0 spike (gate G5).
//!
//! `mpxj-rs` is reader-only, so the writer is ours (§3.2). This spike writes a valid
//! Project + Calendars + Tasks (+ PredecessorLinks) document in the MSPDI 2007+ schema,
//! with element order following the schema (§3.2 / Appendix C rule 3). Resources,
//! assignments, baselines, timephased data, and custom fields are Phase 2.
//!
//! The writer is deterministic (same model → same bytes) and UTF-8 without BOM. Dates are
//! local wall-clock ISO 8601 at minute precision; durations are ISO 8601 `PT#H#M#S`.
//!
//! The real G5 oracle is opening the output in Microsoft Project (needs Windows + a
//! Project licence); the spike validates content by round-tripping through `mpxj-rs`'s
//! MSPDI reader as a cross-check only (Appendix C rule 18, §3.2).

#![forbid(unsafe_code)]

use std::io::Write;

use chrono::NaiveDateTime;
use domain::{Calendar, ConstraintType, Dependency, Project, RelationType, Task, TimeInterval};
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::writer::Writer;

type IoResult = std::io::Result<()>;

/// Serialize `project` to MSPDI XML bytes. Deterministic for a given model.
pub fn write_project(project: &Project) -> std::io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    {
        let mut w = Writer::new(&mut buf);
        write_project_inner(&mut w, project)?;
    }
    Ok(buf)
}

fn write_project_inner<W: Write>(w: &mut Writer<W>, p: &Project) -> IoResult {
    let mut root = BytesStart::new("Project");
    root.push_attribute(("xmlns", "http://schemas.microsoft.com/project"));
    w.write_event(Event::Start(root))?;

    // ---- Project header (§5.2 options), in schema order ----
    el_int(w, "SaveVersion", 14)?;
    if let Some(name) = &p.name {
        el_text(w, "Name", name)?;
    }
    el_bool(w, "ScheduleFromStart", true)?;
    if let Some(d) = p.start_date {
        el_text(w, "StartDate", &fmt_date(d))?;
    }
    if let Some(d) = p.finish_date {
        el_text(w, "FinishDate", &fmt_date(d))?;
    }
    el_int(w, "CriticalSlackLimit", 0)?;
    el_int(w, "CalendarUID", project_calendar_uid(p))?;
    el_text(w, "DefaultStartTime", "08:00:00")?;
    el_text(w, "DefaultFinishTime", "17:00:00")?;
    el_int(w, "MinutesPerDay", i64::from(p.minutes_per_day.unwrap_or(480)))?;
    el_int(w, "MinutesPerWeek", 2400)?;
    el_int(w, "DaysPerMonth", 20)?;
    el_int(w, "DefaultTaskType", 0)?;
    el_int(w, "DurationFormat", 7)?;
    el_int(w, "WeekStartDay", 1)?;
    if let Some(d) = p.status_date {
        el_text(w, "StatusDate", &fmt_date(d))?;
    }

    // ---- Calendars ----
    if !p.calendars.is_empty() {
        start(w, "Calendars")?;
        for cal in &p.calendars {
            write_calendar(w, cal)?;
        }
        end(w, "Calendars")?;
    }

    // ---- Tasks ----
    if !p.tasks.is_empty() {
        start(w, "Tasks")?;
        for task in &p.tasks {
            write_task(w, task, p)?;
        }
        end(w, "Tasks")?;
    }

    w.write_event(Event::End(BytesEnd::new("Project")))?;
    Ok(())
}

fn project_calendar_uid(p: &Project) -> i64 {
    p.calendars
        .first()
        .map(|c| i64::from(c.uid.0))
        .unwrap_or(-1)
}

fn write_calendar<W: Write>(w: &mut Writer<W>, cal: &Calendar) -> IoResult {
    start(w, "Calendar")?;
    el_int(w, "UID", i64::from(cal.uid.0))?;
    el_text(w, "Name", &cal.name)?;
    el_bool(w, "IsBaseCalendar", true)?;
    el_int(
        w,
        "BaseCalendarUID",
        cal.base_uid.map(|u| i64::from(u.0)).unwrap_or(-1),
    )?;
    start(w, "WeekDays")?;
    for (i, intervals) in cal.week.iter().enumerate() {
        start(w, "WeekDay")?;
        el_int(w, "DayType", day_type_code(i))?;
        el_bool(w, "DayWorking", !intervals.is_empty())?;
        if !intervals.is_empty() {
            start(w, "WorkingTimes")?;
            for iv in intervals {
                write_working_time(w, iv)?;
            }
            end(w, "WorkingTimes")?;
        }
        end(w, "WeekDay")?;
    }
    end(w, "WeekDays")?;
    end(w, "Calendar")
}

fn write_working_time<W: Write>(w: &mut Writer<W>, iv: &TimeInterval) -> IoResult {
    start(w, "WorkingTime")?;
    el_text(w, "FromTime", &fmt_minutes(iv.start))?;
    el_text(w, "ToTime", &fmt_minutes(iv.end))?;
    end(w, "WorkingTime")
}

fn write_task<W: Write>(w: &mut Writer<W>, t: &Task, p: &Project) -> IoResult {
    start(w, "Task")?;
    if let Some(uid) = t.uid {
        el_int(w, "UID", i64::from(uid.0))?;
    }
    if let Some(id) = t.id {
        el_int(w, "ID", i64::from(id))?;
    }
    if let Some(name) = &t.name {
        el_text(w, "Name", name)?;
    }
    if let Some(level) = t.outline_level {
        el_int(w, "OutlineLevel", i64::from(level))?;
    }
    el_int(w, "Priority", 500)?;
    if let Some(d) = t.start {
        el_text(w, "Start", &fmt_date(d))?;
    }
    if let Some(d) = t.finish {
        el_text(w, "Finish", &fmt_date(d))?;
    }
    if let Some(dur) = t.duration {
        el_text(w, "Duration", &fmt_duration(dur.ticks()))?;
        el_int(w, "DurationFormat", 7)?;
    }
    el_bool(w, "EffortDriven", true)?;
    el_bool(w, "Milestone", t.is_milestone)?;
    el_bool(w, "Summary", t.is_summary)?;
    el_bool(w, "Critical", false)?;
    if let Some(ct) = t.constraint_type {
        el_int(w, "ConstraintType", constraint_code(ct))?;
    }
    el_int(w, "CalendarUID", -1)?;
    if let Some(d) = t.constraint_date {
        el_text(w, "ConstraintDate", &fmt_date(d))?;
    }
    if let Some(d) = t.deadline {
        el_text(w, "Deadline", &fmt_date(d))?;
    }
    if let Some(pc) = t.percent_complete {
        el_int(w, "PercentComplete", pc.round() as i64)?;
    }
    if let Some(uid) = t.uid {
        for dep in p.dependencies.iter().filter(|d| d.successor == uid) {
            write_predecessor(w, dep)?;
        }
    }
    el_bool(w, "Active", true)?;
    el_bool(w, "Manual", false)?;
    end(w, "Task")
}

fn write_predecessor<W: Write>(w: &mut Writer<W>, dep: &Dependency) -> IoResult {
    start(w, "PredecessorLink")?;
    el_int(w, "PredecessorUID", i64::from(dep.predecessor.0))?;
    el_int(w, "Type", relation_code(dep.relation_type))?;
    el_bool(w, "CrossProject", false)?;
    // MSPDI LinkLag is in tenths of minutes; our Lag ticks are already tenths of a minute.
    el_int(w, "LinkLag", dep.lag.ticks())?;
    el_int(w, "LagFormat", 7)?;
    end(w, "PredecessorLink")
}

// ---- Code mappings (Appendix D; enum discriminants match the MSPDI codes) ----

fn constraint_code(ct: ConstraintType) -> i64 {
    ct as i64
}

fn relation_code(rt: RelationType) -> i64 {
    rt as i64
}

/// MSPDI DayType: 1=Sunday .. 7=Saturday, 2=Monday. Our week index is 0=Monday .. 6=Sunday.
fn day_type_code(index: usize) -> i64 {
    if index == 6 {
        1 // Sunday
    } else {
        (index + 2) as i64 // Monday=2 .. Saturday=7
    }
}

// ---- Value formatting ----

fn fmt_date(d: NaiveDateTime) -> String {
    d.format("%Y-%m-%dT%H:%M:%S").to_string()
}

/// ISO 8601 duration `PT#H#M#S` from ticks (1 tick = 0.1 min = 6 s).
fn fmt_duration(ticks: i64) -> String {
    let seconds = ticks * 6;
    let (h, m, s) = (seconds / 3600, (seconds % 3600) / 60, seconds % 60);
    format!("PT{h}H{m}M{s}S")
}

/// Minutes-since-midnight to `HH:MM:SS`. A 24-hour day end (1440) clamps to 23:59:59,
/// a known deviation to validate against the MSPDI `xsd:time` representation (Appendix F).
fn fmt_minutes(minutes: i64) -> String {
    let secs = (minutes * 60).clamp(0, 86399);
    format!("{:02}:{:02}:{:02}", secs / 3600, (secs % 3600) / 60, secs % 60)
}

// ---- Low-level event helpers ----

fn start<W: Write>(w: &mut Writer<W>, name: &str) -> IoResult {
    w.write_event(Event::Start(BytesStart::new(name)))
}

fn end<W: Write>(w: &mut Writer<W>, name: &str) -> IoResult {
    w.write_event(Event::End(BytesEnd::new(name)))
}

fn el_text<W: Write>(w: &mut Writer<W>, name: &str, value: &str) -> IoResult {
    start(w, name)?;
    w.write_event(Event::Text(BytesText::new(value)))?;
    end(w, name)
}

fn el_int<W: Write>(w: &mut Writer<W>, name: &str, value: i64) -> IoResult {
    let s = value.to_string();
    el_text(w, name, &s)
}

fn el_bool<W: Write>(w: &mut Writer<W>, name: &str, value: bool) -> IoResult {
    el_int(w, name, if value { 1 } else { 0 })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use domain::{Calendar, Dependency, Duration, Lag, RelationType, Task, TaskUid};
    use mpxj::mspdi::mspdi_reader::MSPDIReader;
    use mpxj::reader::project_reader::ProjectReader;

    fn dt(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(y, mo, d)
            .unwrap()
            .and_hms_opt(h, mi, 0)
            .unwrap()
    }

    fn sample_project() -> Project {
        let cal = Calendar::standard("Standard");
        let t1 = Task {
            uid: Some(TaskUid(1)),
            id: Some(1),
            name: Some("Task A".into()),
            duration: Some(Duration::from_minutes(480)),
            start: Some(dt(2026, 11, 9, 8, 0)),
            finish: Some(dt(2026, 11, 9, 17, 0)),
            ..Default::default()
        };
        let t2 = Task {
            uid: Some(TaskUid(2)),
            id: Some(2),
            name: Some("Task B".into()),
            duration: Some(Duration::from_minutes(480)),
            start: Some(dt(2026, 11, 10, 8, 0)),
            finish: Some(dt(2026, 11, 10, 17, 0)),
            ..Default::default()
        };
        Project {
            name: Some("Spike".into()),
            start_date: Some(dt(2026, 11, 9, 8, 0)),
            finish_date: Some(dt(2026, 11, 10, 17, 0)),
            minutes_per_day: Some(480),
            tasks: vec![t1, t2],
            calendars: vec![cal],
            dependencies: vec![Dependency {
                predecessor: TaskUid(1),
                successor: TaskUid(2),
                relation_type: RelationType::FinishToStart,
                lag: Lag::from_minutes(0),
            }],
            ..Default::default()
        }
    }

    #[test]
    fn write_is_deterministic() {
        let p = sample_project();
        let a = write_project(&p).unwrap();
        let b = write_project(&p).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn output_contains_expected_elements() {
        let bytes = write_project(&sample_project()).unwrap();
        let xml = String::from_utf8(bytes).unwrap();
        assert!(xml.contains(r#"<Project xmlns="http://schemas.microsoft.com/project">"#));
        assert!(xml.contains("<Name>Spike</Name>"));
        assert!(xml.contains("<StartDate>2026-11-09T08:00:00</StartDate>"));
        assert!(xml.contains("<Duration>PT8H0M0S</Duration>"));
        assert!(xml.contains("<PredecessorLink>"));
        assert!(xml.contains("<PredecessorUID>1</PredecessorUID>"));
        assert!(xml.contains("<Type>1</Type>")); // FS relation
        assert!(xml.contains("<DayType>2</DayType>")); // Monday
    }

    #[test]
    fn output_roundtrips_through_mpxj_reader() {
        let bytes = write_project(&sample_project()).unwrap();
        let mut reader = MSPDIReader::new();
        let mut cursor = std::io::Cursor::new(bytes);
        let file = reader
            .read_reader(&mut cursor)
            .expect("mpxj-rs should parse our output");
        let f = file.borrow();

        assert_eq!(f.get_tasks().len(), 2);
        let names: Vec<String> = f
            .get_tasks()
            .iter()
            .map(|t| t.borrow().get_name().unwrap())
            .collect();
        assert_eq!(names, vec!["Task A".to_string(), "Task B".to_string()]);

        let first = f.get_tasks().iter().next().unwrap();
        let t = first.borrow();
        assert_eq!(
            t.get_start().unwrap().format("%Y-%m-%dT%H:%M:%S").to_string(),
            "2026-11-09T08:00:00"
        );

        // The calendar should round-trip too.
        assert!(!f.get_calendars().is_empty());
    }
}
