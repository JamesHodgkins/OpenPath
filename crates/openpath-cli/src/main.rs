//! OpenPath CLI — Phase 0 smoke harness.
//!
//! A small binary that exercises the engine and the file formats end-to-end, so there is a
//! real `cargo run` target before the egui app exists. The GUI will reuse the same crates.

#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::path::Path;

use chrono::NaiveDateTime;
use openpath_core::sample::sample_project;
use openpath_core::{Project, TaskUid};
use openpath_schedule::{ScheduleOptions, ScheduleOutput};

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
    println!("  openpath-cli sample                        generate a sample plan, run CPM, save native + MSPDI");
    println!("  openpath-cli schedule <file.opath>         load a native file and print the CPM schedule");
    println!("  openpath-cli export <in.opath> <out.xml>   convert a native file to MSPDI");
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
    let native = dir.join("openpath_sample.opath");
    let xml = dir.join("openpath_sample.xml");
    openpath_files::save_atomic(&native, &project)?;
    std::fs::write(&xml, openpath_mspdi::write_project(&project)?)?;
    println!("\nWrote {}", native.display());
    println!("Wrote {}", xml.display());
    Ok(())
}

fn cmd_schedule(file: Option<&String>) -> Result<(), Box<dyn std::error::Error>> {
    let Some(path) = file else {
        eprintln!("usage: openpath-cli schedule <file.opath>");
        return Ok(());
    };
    let project = openpath_files::load(Path::new(path))?;
    let out = run_schedule(&project);
    print_schedule(&project, &out);
    Ok(())
}

fn cmd_export(
    inp: Option<&String>,
    outp: Option<&String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (Some(in_path), Some(out_path)) = (inp, outp) else {
        eprintln!("usage: openpath-cli export <in.opath> <out.xml>");
        return Ok(());
    };
    let project = openpath_files::load(Path::new(in_path))?;
    std::fs::write(out_path, openpath_mspdi::write_project(&project)?)?;
    println!("Wrote {out_path}");
    Ok(())
}

fn run_schedule(project: &Project) -> ScheduleOutput {
    let cal = project.calendars.first().expect("project has a calendar");
    let options = ScheduleOptions {
        project_start: project.start_date.expect("project has a start date"),
        ..Default::default()
    };
    openpath_schedule::schedule(&project.tasks, &project.dependencies, cal, &options)
}

fn print_schedule(project: &Project, out: &ScheduleOutput) {
    let names: HashMap<TaskUid, &str> = project
        .tasks
        .iter()
        .filter_map(|t| Some((t.uid?, t.name.as_deref().unwrap_or("(unnamed)"))))
        .collect();

    println!(
        "{:>3}  {:<12}  {:<16}  {:<16}  {:>7}  {:>7}  Crit",
        "ID", "Name", "Start", "Finish", "Slack", "Free"
    );
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
