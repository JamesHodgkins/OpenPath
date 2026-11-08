//! MPP import adapter — Phase 0 spike (gate G1).
//!
//! Wraps `mpxj-rs` (LGPL-2.1-or-later) and converts its `Rc<RefCell<…>>` graph into
//! owned `domain` values on a single thread, then drops the graph. Nothing above this
//! crate may name an `mpxj` type (§2.2). Parser panics are contained and converted to
//! errors (§2.6).

#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::io::Cursor;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::rc::Rc;

use mpxj::mpp::mpp_reader::MPPReader;
use mpxj::project_file::ProjectFile;
use mpxj::reader::project_reader::ProjectReader;
use mpxj::{Duration as MppDuration, ProjectProperties, TimeUnit};

use domain::{
    Duration, FidelityClass, FidelityReport, FieldEntry, Project, Task, TaskUid, Tick,
};

/// Errors produced while importing an MPP file.
#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("the MPP parser panicked and was contained: {0}")]
    ParserPanic(String),
    #[error("the MPP parser reported an error: {0}")]
    Parser(String),
}

/// The result of a successful import: the owned project plus its fidelity report.
#[derive(Debug, Clone)]
pub struct ImportedProject {
    pub project: Project,
    pub fidelity: FidelityReport,
}

/// The import boundary exposed to the rest of the application (§3.1).
pub trait MppImporter {
    fn read(&self, bytes: &[u8]) -> Result<ImportedProject, ImportError>;
}

/// Imports MPP14 (Project 2010–365) files via `mpxj-rs`.
#[derive(Debug, Default)]
pub struct Mpp14Importer;

impl MppImporter for Mpp14Importer {
    fn read(&self, bytes: &[u8]) -> Result<ImportedProject, ImportError> {
        let parsed = catch_unwind(AssertUnwindSafe(|| {
            let mut reader = MPPReader::new();
            let mut cursor = Cursor::new(bytes);
            reader.read_reader(&mut cursor)
        }))
        .map_err(|panic| ImportError::ParserPanic(panic_message(panic)))?
        .map_err(|e| ImportError::Parser(format!("{e}")))?;

        Ok(convert(&parsed))
    }
}

fn panic_message(panic: Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = panic.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = panic.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic".to_string()
    }
}

fn convert(file: &Rc<RefCell<ProjectFile>>) -> ImportedProject {
    let f = file.borrow();
    let props = f.get_project_properties();

    let tasks: Vec<Task> = f
        .get_tasks()
        .iter()
        .map(|t| convert_task(&t.borrow(), props))
        .collect();

    let mut report = FidelityReport {
        tasks: tasks.len(),
        resources: f.get_resources().len(),
        assignments: f.get_resource_assignments().len(),
        calendars: f.get_calendars().len(),
        relations: f.get_relations().len(),
        entries: Vec::new(),
    };
    report.entries = fidelity_entries();

    let project = Project {
        name: props.get_name(),
        start_date: props.get_start_date(),
        finish_date: props.get_finish_date(),
        status_date: props.get_status_date(),
        minutes_per_day: props.get_minutes_per_day(),
        tasks,
        ..Default::default()
    };

    ImportedProject { project, fidelity: report }
}

fn convert_task(t: &mpxj::Task, props: &ProjectProperties) -> Task {
    Task {
        uid: t.get_unique_id().map(TaskUid),
        id: t.get_id(),
        name: t.get_name(),
        outline_level: t.get_outline_level(),
        is_summary: t.get_summary(),
        is_milestone: t.get_milestone(),
        start: t.get_start(),
        finish: t.get_finish(),
        percent_complete: t.get_percentage_complete(),
        duration: t.get_duration().and_then(|d| duration_ticks(&d, props)),
        ..Default::default()
    }
}

/// Converts an `mpxj` duration to ticks (0.1 min) using the project's unit defaults.
fn duration_ticks(d: &MppDuration, props: &ProjectProperties) -> Option<Duration> {
    let minutes = d.convert_units(TimeUnit::Minutes, props).get_duration();
    Some(Duration(Tick((minutes * 10.0).round() as i64)))
}

/// Phase 0 fidelity entries: the fields we map now, and the groups still to be mapped.
fn fidelity_entries() -> Vec<FieldEntry> {
    let mapped = [
        "project.name",
        "project.start_date",
        "project.finish_date",
        "project.status_date",
        "project.minutes_per_day",
        "task.uid",
        "task.id",
        "task.name",
        "task.outline_level",
        "task.is_summary",
        "task.is_milestone",
        "task.start",
        "task.finish",
        "task.percent_complete",
        "task.duration",
    ];
    let not_yet_mapped = [
        ("resources", "entity + field extraction pending in the spike"),
        ("calendars", "entity + field extraction pending in the spike"),
        ("dependencies/lag", "entity extraction pending in the spike"),
        ("baselines 0–10", "pending in the spike"),
        ("custom fields", "pending in the spike"),
        ("timephased data", "pending in the spike"),
        ("splits", "pending in the spike"),
        ("notes (RTF)", "pending in the spike"),
    ];

    let mut entries: Vec<FieldEntry> = mapped
        .iter()
        .map(|f| FieldEntry {
            field: (*f).to_string(),
            class: FidelityClass::Full,
            note: None,
        })
        .collect();
    entries.extend(not_yet_mapped.iter().map(|(f, note)| FieldEntry {
        field: (*f).to_string(),
        class: FidelityClass::Ignored,
        note: Some((*note).to_string()),
    }));
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Empty input must produce a parser error, never a panic.
    #[test]
    fn empty_input_errors_cleanly() {
        let importer = Mpp14Importer;
        let result = importer.read(&[]);
        assert!(matches!(result, Err(ImportError::Parser(_))));
    }

    /// Arbitrary garbage must not panic (panic containment, §2.6).
    #[test]
    fn garbage_input_does_not_panic() {
        let importer = Mpp14Importer;
        let bytes: Vec<u8> = (0..1024).map(|i| (i * 31 % 256) as u8).collect();
        let _ = importer.read(&bytes); // Ok or Err, but no unwind.
    }
}
