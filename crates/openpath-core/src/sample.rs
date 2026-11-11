//! A small built-in project used by the CLI smoke harness and the GUI's first launch.

use chrono::NaiveDate;

use crate::{Calendar, Dependency, Duration, Lag, Project, RelationType, Task, TaskUid};

pub fn sample_project() -> Project {
    fn task(uid: i32, name: &str, minutes: i64, milestone: bool) -> Task {
        Task {
            uid: Some(TaskUid(uid)),
            id: Some(uid),
            name: Some(name.to_string()),
            duration: Some(Duration::from_minutes(minutes)),
            is_milestone: milestone,
            ..Default::default()
        }
    }
    let tasks = vec![
        task(1, "Design", 960, false),
        task(2, "Implement", 1440, false),
        task(3, "Test", 960, false),
        task(4, "Docs", 480, false),
        task(5, "Ship", 0, true),
    ];
    let fs = |p: i32, s: i32| Dependency {
        predecessor: TaskUid(p),
        successor: TaskUid(s),
        relation_type: RelationType::FinishToStart,
        lag: Lag::from_minutes(0),
    };
    Project {
        name: Some("Sample Plan".into()),
        start_date: Some(
            NaiveDate::from_ymd_opt(2026, 11, 9)
                .unwrap()
                .and_hms_opt(8, 0, 0)
                .unwrap(),
        ),
        minutes_per_day: Some(480),
        tasks,
        calendars: vec![Calendar::standard("Standard")],
        dependencies: vec![fs(1, 2), fs(2, 3), fs(3, 5), fs(4, 5)],
        ..Default::default()
    }
}
