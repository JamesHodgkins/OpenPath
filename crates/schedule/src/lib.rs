//! OpenPath scheduling engine — Phase 0 spike (gate G3).
//!
//! Depends only on `domain` (§2.2). All date arithmetic goes through [`calendar`];
//! the critical-path forward/backward pass, constraints, and slack live in [`cpm`].

#![forbid(unsafe_code)]

pub mod calendar;
pub mod cpm;

pub use calendar::{
    add_working_minutes, is_working, next_working_instant, prev_working_instant,
    working_minutes_between, working_minutes_signed,
};
pub use cpm::{schedule, Diagnostic, ScheduleOptions, ScheduleOutput, ScheduledTask};
