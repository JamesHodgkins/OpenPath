//! OpenPath domain model — Phase 0 spike subset.
//!
//! This crate holds the owned, `Send + Sync` project model the rest of the
//! application works on. During Phase 0 it is deliberately minimal: it exists to
//! prove the MPP import pipeline can convert `mpxj-rs`'s `Rc<RefCell<…>>` graph into
//! owned values without leaking parser types. It will be expanded to the full model
//! described in §4 of `specification-v1.1.md` during Phase 1.
//!
//! Invariants:
//! - This crate MUST NOT depend on any parser, I/O, or UI crate (§2.2).
//! - `Tick` is 0.1 minute (§4.2).

#![forbid(unsafe_code)]

use chrono::NaiveDateTime;

/// 0.1 minute. Durations and work are stored as integer ticks (§4.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Tick(pub i64);

/// Working duration, a distinct newtype so it cannot be confused with work or lag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Duration(pub Tick);

/// Entity identities: integer MSP UIDs, preserved exactly from the source (§4.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskUid(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceUid(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssignmentUid(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CalendarUid(pub i32);

/// Fidelity class for a single imported field (Appendix A.0).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FidelityClass {
    /// Full: imported with exact values.
    Full,
    /// Read/import only: preserved in native, not written to MSPDI.
    ReadOnly,
    /// Lossy: imported with a documented approximation.
    Lossy,
    /// Computed: recalculated by the engine.
    Computed,
    /// Ignored: not imported.
    Ignored,
}

impl FidelityClass {
    pub const COUNT: usize = 5;
    pub fn index(self) -> usize {
        match self {
            FidelityClass::Full => 0,
            FidelityClass::ReadOnly => 1,
            FidelityClass::Lossy => 2,
            FidelityClass::Computed => 3,
            FidelityClass::Ignored => 4,
        }
    }
}

/// One field-level fidelity record.
#[derive(Debug, Clone)]
pub struct FieldEntry {
    pub field: String,
    pub class: FidelityClass,
    pub note: Option<String>,
}

/// Aggregate fidelity report produced by every import (§3.3 step 6).
#[derive(Debug, Clone, Default)]
pub struct FidelityReport {
    pub tasks: usize,
    pub resources: usize,
    pub assignments: usize,
    pub calendars: usize,
    pub relations: usize,
    pub entries: Vec<FieldEntry>,
}

impl FidelityReport {
    /// Count of entries in each fidelity class, ordered `[Full, ReadOnly, Lossy, Computed, Ignored]`.
    pub fn counts_by_class(&self) -> [usize; FidelityClass::COUNT] {
        let mut counts = [0usize; FidelityClass::COUNT];
        for entry in &self.entries {
            counts[entry.class.index()] += 1;
        }
        counts
    }

    /// True when anything was ignored or made lossy — the UI surfaces the report in that case.
    pub fn has_degradation(&self) -> bool {
        self.entries
            .iter()
            .any(|e| matches!(e.class, FidelityClass::Ignored | FidelityClass::Lossy))
    }
}

/// A task, in row order (§4.4). Minimal Phase 0 subset.
#[derive(Debug, Clone, Default)]
pub struct Task {
    pub uid: Option<TaskUid>,
    pub id: Option<i32>,
    pub name: Option<String>,
    pub outline_level: Option<i32>,
    pub is_summary: bool,
    pub is_milestone: bool,
    pub start: Option<NaiveDateTime>,
    pub finish: Option<NaiveDateTime>,
    pub percent_complete: Option<f64>,
    pub duration: Option<Duration>,
}

/// The project root. Minimal Phase 0 subset.
#[derive(Debug, Clone, Default)]
pub struct Project {
    pub name: Option<String>,
    pub start_date: Option<NaiveDateTime>,
    pub finish_date: Option<NaiveDateTime>,
    pub status_date: Option<NaiveDateTime>,
    pub minutes_per_day: Option<i32>,
    pub tasks: Vec<Task>,
}
