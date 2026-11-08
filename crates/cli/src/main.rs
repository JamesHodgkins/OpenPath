//! OpenPath CLI — Phase 0 smoke harness.
//!
//! A small binary that exercises the engine and the file formats end-to-end, so there is a
//! real `cargo run` target before the Tauri GUI exists. The GUI will reuse the same crates.

#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::path::Path;

use chrono::{NaiveDate, NaiveDateTime};
use domain::{Calendar, Dependency, Duration, Lag, Project, RelationType, Task, TaskUid};
use schedule::{ScheduleOptions, ScheduleOutput};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("sample") => cmd_sample(),
        Some("schedule") => cmd_schedule(args.get(1)),
        Some("export") => cmd_export(args.get(1), args.get(2)),
        _ => {
            print_usage();
            Ok(())
        }
    }
}

fn print_usage() {
    println!("OpenPath CLI (Phase 0 spike)");
    println!();
    println!("  cli sample                      generate a sample plan, run CPM, save native + MSPDI");
    println!("  cli schedule <file.rproj>        load a native file and print the CPM schedule");
    println!("  cli export <in.rproj> <out.xml>  convert a native file to MSPDI");
}

fn cmd_sample() -> Result<(), Box<dyn std::error::Error>> {
    let mut project = sample_project();
    let out = run_schedule(&project);
    // Write the scheduled dates back so the saved files carry them.
    for s in &out.scheduled {
        if let Some(t) = project.tasks.iter_mut().find(|t| t.uid == Some(s.uid)) {
            t.start = Some(s.early_start);
            t.finish = Some(s.early_finish);
        }
    }
    project.finish_date = out.scheduled.iter().map(|s| s.early_finish).max();
    print_schedule(&project, &out);

    let dir = std::env::temp_dir();
    let native = dir.join("openpath_sample.rproj");
    let xml = dir.join("openpath_sample.xml");
    store::save_atomic(&native, &project)?;
    std::fs::write(&xml, interop_mspdi::write_project(&project)?)?;
    println!("\nWrote {}", native.display());
    println!("Wrote {}", xml.display());
    Ok(())
}

fn cmd_schedule(file: Option<&String>) -> Result<(), Box<dyn std::error::Error>> {
    let Some(path) = file else {
        eprintln!("usage: cli schedule <file.rproj>");
        return Ok(());
    };
    let project = store::load(Path::new(path))?;
    let out = run_schedule(&project);
    print_schedule(&project, &out);
    Ok(())
}

fn cmd_export(
    inp: Option<&String>,
    outp: Option<&String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (Some(in_path), Some(out_path)) = (inp, outp) else {
        eprintln!("usage: cli export <in.rproj> <out.xml>");
        return Ok(());
    };
    let project = store::load(Path::new(in_path))?;
    std::fs::write(out_path, interop_mspdi::write_project(&project)?)?;
    println!("Wrote {out_path}");
    Ok(())
}

fn run_schedule(project: &Project) -> ScheduleOutput {
    let cal = project.calendars.first().expect("project has a calendar");
    let options = ScheduleOptions {
        project_start: project.start_date.expect("project has a start date"),
        ..Default::default()
    };
    schedule::schedule(&project.tasks, &project.dependencies, cal, &options)
}

fn print_schedule(project: &Project, out: &ScheduleOutput) {
    let names: HashMap<TaskUid, &str> = project
        .tasks
        .iter()
        .filter_map(|t| Some((t.uid?, t.name.as_deref().unwrap_or("(unnamed)"))))
        .collect();

    println!("{:>3}  {:<12}  {:<16}  {:<16}  {:>7}  {:>7}  Crit", "ID", "Name", "Start", "Finish", "Slack", "Free");
    for s in &out.scheduled {
        let name = names.get(&s.uid).copied().unwrap_or("?");
        println!(
            "{:>3}  {:<12}  {:<16}  {:<16}  {:>7}  {:>7}  {}",
            s.uid.0,
            name,
            fmt_dt(s.early_start),
            fmt_dt(s.early_finish),
            fmt_min(s.total_slack_minutes),
            fmt_min(s.free_slack_minutes),
            if s.is_critical { "*" } else { "" },
        );
    }
    for d in &out.diagnostics {
        println!("  [{}] {}", d.code, d.message);
    }
}

fn fmt_dt(d: NaiveDateTime) -> String {
    d.format("%Y-%m-%d %H:%M").to_string()
}

fn fmt_min(minutes: i64) -> String {
    format!("{minutes}m")
}

fn sample_project() -> Project {
    fn task(uid: i32, id: i32, name: &str, minutes: i64, milestone: bool) -> Task {
        Task {
            uid: Some(TaskUid(uid)),
            id: Some(id),
            name: Some(name.to_string()),
            duration: Some(Duration::from_minutes(minutes)),
            is_milestone: milestone,
            ..Default::default()
        }
    }
    let tasks = vec![
        task(1, 1, "Design", 960, false),
        task(2, 2, "Implement", 1440, false),
        task(3, 3, "Test", 960, false),
        task(4, 4, "Docs", 480, false),
        task(5, 5, "Ship", 0, true),
    ];
    let fs = |p: i32, s: i32| Dependency {
        predecessor: TaskUid(p),
        successor: TaskUid(s),
        relation_type: RelationType::FinishToStart,
        lag: Lag::from_minutes(0),
    };
    let deps = vec![fs(1, 2), fs(2, 3), fs(3, 5), fs(4, 5)];
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
        dependencies: deps,
        ..Default::default()
    }
}
