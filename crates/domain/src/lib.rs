//! OpenPath domain model — Phase 0 spike subset.
//!
//! This crate holds the owned, `Send + Sync` project model the rest of the
//! application works on. During Phase 0 it is deliberately minimal: it exists to
//! prove the MPP import pipeline can convert `mpxj-rs`'s `Rc<RefCell<…>>` graph into
//! owned values without leaking parser types, and to give the scheduling engine and the
//! native store a model to operate on. It will be expanded to the full model described in
//! §4 of `specification-v1.1.md` during Phase 1.
//!
//! Invariants:
//! - This crate MUST NOT depend on any parser, I/O, or UI crate (§2.2). `serde` derives are
//!   data-format-agnostic serialization, not I/O.
//! - `Tick` is 0.1 minute (§4.2).

#![forbid(unsafe_code)]

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

/// 0.1 minute. Durations and work are stored as integer ticks (§4.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct Tick(pub i64);

/// Working duration, a distinct newtype so it cannot be confused with work or lag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct Duration(pub Tick);

impl Duration {
    pub fn from_minutes(min: i64) -> Self {
        Duration(Tick(min * 10))
    }
    pub fn as_minutes(self) -> i64 {
        self.0.0 / 10
    }
    pub fn ticks(self) -> i64 {
        self.0.0
    }
}

/// Dependency lag (lead when negative), a distinct newtype from duration and work.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct Lag(pub Tick);

impl Lag {
    pub fn from_minutes(min: i64) -> Self {
        Lag(Tick(min * 10))
    }
    pub fn as_minutes(self) -> i64 {
        self.0.0 / 10
    }
    pub fn ticks(self) -> i64 {
        self.0.0
    }
}

/// Entity identities: integer MSP UIDs, preserved exactly from the source (§4.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TaskUid(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ResourceUid(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AssignmentUid(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CalendarUid(pub i32);

/// Fidelity class for a single imported field (Appendix A.0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldEntry {
    pub field: String,
    pub class: FidelityClass,
    pub note: Option<String>,
}

/// Aggregate fidelity report produced by every import (§3.3 step 6).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
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

/// A working interval within a day, half-open `[start, end)` in minutes since midnight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeInterval {
    pub start: i64,
    pub end: i64,
}

impl TimeInterval {
    pub fn new(start: i64, end: i64) -> Self {
        debug_assert!(start < end && start >= 0 && end <= 1440);
        TimeInterval { start, end }
    }
}

/// A calendar exception: a single date that overrides the weekly pattern (§4.7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalendarException {
    pub date: NaiveDate,
    pub working: bool,
    /// Replacement working intervals when `working` is true.
    pub intervals: Vec<TimeInterval>,
}

/// A project/resource/task calendar (§4.7).
///
/// `week` is indexed by `Weekday::num_days_from_monday()`, i.e. 0 = Monday .. 6 = Sunday.
/// Base-calendar inheritance is a Phase 1 concern; the spike resolves a single calendar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Calendar {
    pub uid: CalendarUid,
    pub name: String,
    pub base_uid: Option<CalendarUid>,
    pub week: [Vec<TimeInterval>; 7],
    pub exceptions: Vec<CalendarException>,
}

impl Calendar {
    /// Monday–Friday, 08:00–17:00.
    pub fn standard(name: impl Into<String>) -> Self {
        let day = vec![TimeInterval::new(480, 1020)];
        let mut week: [Vec<TimeInterval>; 7] = Default::default();
        for slot in week.iter_mut().take(5) {
            *slot = day.clone();
        }
        Calendar {
            uid: CalendarUid(0),
            name: name.into(),
            base_uid: None,
            week,
            exceptions: Vec::new(),
        }
    }

    /// Every day, 00:00–24:00.
    pub fn twenty_four_seven(name: impl Into<String>) -> Self {
        let day = vec![TimeInterval::new(0, 1440)];
        let mut week: [Vec<TimeInterval>; 7] = Default::default();
        for slot in week.iter_mut() {
            *slot = day.clone();
        }
        Calendar {
            uid: CalendarUid(0),
            name: name.into(),
            base_uid: None,
            week,
            exceptions: Vec::new(),
        }
    }
}

/// The eight Microsoft Project constraint types (§5.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintType {
    AsSoonAsPossible,
    AsLateAsPossible,
    MustStartOn,
    MustFinishOn,
    StartNoEarlierThan,
    StartNoLaterThan,
    FinishNoEarlierThan,
    FinishNoLaterThan,
}

/// The four dependency relation types (§4.8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationType {
    FinishToFinish,
    FinishToStart,
    StartToFinish,
    StartToStart,
}

/// A predecessor/successor dependency with a relation type and lag (§4.8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dependency {
    pub predecessor: TaskUid,
    pub successor: TaskUid,
    pub relation_type: RelationType,
    pub lag: Lag,
}

/// A task, in row order (§4.4). Minimal Phase 0 subset.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
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
    pub constraint_type: Option<ConstraintType>,
    pub constraint_date: Option<NaiveDateTime>,
    pub deadline: Option<NaiveDateTime>,
}

/// The project root. Minimal Phase 0 subset.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub name: Option<String>,
    pub start_date: Option<NaiveDateTime>,
    pub finish_date: Option<NaiveDateTime>,
    pub status_date: Option<NaiveDateTime>,
    pub minutes_per_day: Option<i32>,
    pub tasks: Vec<Task>,
    pub calendars: Vec<Calendar>,
    pub dependencies: Vec<Dependency>,
}
