//! Critical Path Method: forward/backward pass, constraints, and slack (§5.4–5.6).
//!
//! Phase 0 spike scope:
//! - Topological sort with cycle detection (Kahn's algorithm).
//! - Forward pass (schedule from start) and backward pass for FS/SS/FF/SF with lag.
//! - Constraints: ASAP, ALAP, Must Start/Finish On, Start/Finish No Earlier/Later Than.
//! - Start/finish/total/free slack and the critical flag.
//!
//! Not yet implemented (Phase 1 / G3): schedule-from-finish, manual tasks, summary
//! rollup, per-task calendars, and ALAP late-bound resolution. Slack is measured in
//! working minutes (not clock time); the remaining items are called out where relevant.

#![forbid(unsafe_code)]

use std::collections::{HashMap, VecDeque};

use chrono::NaiveDateTime;
use domain::{Calendar, ConstraintType, Dependency, RelationType, Task, TaskUid};

use crate::calendar::{
    add_working_minutes, next_working_instant, prev_working_instant, working_minutes_signed,
};

/// Scheduling inputs from §5.2.
#[derive(Debug, Clone)]
pub struct ScheduleOptions {
    /// Forward pass primary (`true`) vs schedule-from-finish (not yet implemented).
    pub schedule_from_start: bool,
    /// Earliest bound in forward scheduling.
    pub project_start: NaiveDateTime,
    /// Input in backward scheduling; computed as max EF in forward scheduling.
    pub project_finish: Option<NaiveDateTime>,
    /// Tasks with total slack at or below this are critical (§5.6).
    pub critical_slack_limit: i64,
}

impl Default for ScheduleOptions {
    fn default() -> Self {
        ScheduleOptions {
            schedule_from_start: true,
            project_start: chrono::NaiveDate::from_ymd_opt(2026, 1, 1)
                .unwrap()
                .and_hms_opt(8, 0, 0)
                .unwrap(),
            project_finish: None,
            critical_slack_limit: 0,
        }
    }
}

/// Per-task scheduling result.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScheduledTask {
    pub uid: TaskUid,
    pub early_start: NaiveDateTime,
    pub early_finish: NaiveDateTime,
    pub late_start: NaiveDateTime,
    pub late_finish: NaiveDateTime,
    /// `min(start_slack, finish_slack)`, in working minutes.
    pub total_slack_minutes: i64,
    pub free_slack_minutes: i64,
    pub is_critical: bool,
}

/// A non-blocking scheduling diagnostic (§5.12).
#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub uid: Option<TaskUid>,
    pub message: String,
}

impl Diagnostic {
    fn conflict(uid: Option<TaskUid>) -> Self {
        Diagnostic {
            code: "C001",
            uid,
            message: "constraint conflicts with a dependency; the constraint wins".into(),
        }
    }
    fn cycle(uid: Option<TaskUid>, count: usize) -> Self {
        Diagnostic {
            code: "C005",
            uid,
            message: format!("circular dependency involving {count} task(s); not scheduled"),
        }
    }
    fn alap(uid: Option<TaskUid>) -> Self {
        Diagnostic {
            code: "C003",
            uid,
            message: "ALAP task has no late bound; treated as ASAP".into(),
        }
    }
}

/// The result of a scheduling run.
#[derive(Debug, Clone, Default)]
pub struct ScheduleOutput {
    pub scheduled: Vec<ScheduledTask>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Schedule `tasks` against `cal` using `deps` and `options`.
///
/// Preconditions: every task has a unique `uid`; `deps` reference existing task UIDs.
/// A dependency cycle aborts the run with a `C005` diagnostic (§5.14).
pub fn schedule(
    tasks: &[Task],
    deps: &[Dependency],
    cal: &Calendar,
    options: &ScheduleOptions,
) -> ScheduleOutput {
    let mut diags = Vec::new();
    let n = tasks.len();

    let uid_to_idx: HashMap<TaskUid, usize> = tasks
        .iter()
        .enumerate()
        .map(|(i, t)| (t.uid.expect("scheduling requires every task to have a UID"), i))
        .collect();

    let order = match topological_order(tasks, deps, &uid_to_idx) {
        Ok(o) => o,
        Err(cycle) => {
            let count = cycle.len();
            diags.push(Diagnostic::cycle(cycle.first().copied(), count));
            return ScheduleOutput {
                scheduled: Vec::new(),
                diagnostics: diags,
            };
        }
    };

    let duration = |t: &Task| t.duration.map(|d| d.as_minutes()).unwrap_or(0);

    // ---- Forward pass ----
    let mut es = vec![None; n];
    let mut ef = vec![None; n];
    for &i in &order {
        let task = &tasks[i];
        let dur = duration(task);
        let mut e = options.project_start;

        for d in deps.iter().filter(|d| d.successor == task.uid.unwrap()) {
            let Some(&pi) = uid_to_idx.get(&d.predecessor) else {
                continue;
            };
            let pred_es = es[pi].expect("predecessor scheduled first");
            let pred_ef = ef[pi].expect("predecessor scheduled first");
            let lag = d.lag.as_minutes();
            let (base, offset) = match d.relation_type {
                RelationType::FinishToStart => (pred_ef, lag),
                RelationType::StartToStart => (pred_es, lag),
                RelationType::FinishToFinish => (pred_ef, lag - dur),
                RelationType::StartToFinish => (pred_es, lag - dur),
            };
            let cand = add_working_minutes(base, offset, cal);
            if cand > e {
                e = cand;
            }
        }

        e = apply_early_constraint(task, e, dur, cal, &mut diags);
        e = next_working_instant(e, cal);
        let f = add_working_minutes(e, dur, cal);
        es[i] = Some(e);
        ef[i] = Some(f);
    }

    // ---- Project finish ----
    let project_finish = match options.project_finish {
        Some(f) => f,
        None => ef
            .iter()
            .filter_map(|x| *x)
            .max()
            .unwrap_or(options.project_start),
    };

    // ---- Backward pass (reverse topological order) ----
    let mut ls = vec![None; n];
    let mut lf = vec![None; n];
    for &i in order.iter().rev() {
        let task = &tasks[i];
        let dur = duration(task);
        let mut l = project_finish;

        for d in deps.iter().filter(|d| d.predecessor == task.uid.unwrap()) {
            let Some(&si) = uid_to_idx.get(&d.successor) else {
                continue;
            };
            let succ_ls = ls[si].expect("successor scheduled first");
            let succ_lf = lf[si].expect("successor scheduled first");
            let lag = d.lag.as_minutes();
            let (base, offset) = match d.relation_type {
                RelationType::FinishToStart => (succ_ls, -lag),
                RelationType::StartToStart => (succ_ls, -lag + dur),
                RelationType::FinishToFinish => (succ_lf, -lag),
                RelationType::StartToFinish => (succ_lf, -lag + dur),
            };
            let cand = add_working_minutes(base, offset, cal);
            if cand < l {
                l = cand;
            }
        }

        l = apply_late_constraint(task, l, dur, cal, &mut diags);
        let s = add_working_minutes(l, -dur, cal);
        let l2 = add_working_minutes(s, dur, cal);
        ls[i] = Some(s);
        lf[i] = Some(l2);
    }

    // ---- Slack and criticality ----
    let mut scheduled = Vec::with_capacity(n);
    for i in 0..n {
        let task = &tasks[i];
        let e_s = es[i].unwrap();
        let e_f = ef[i].unwrap();
        let l_s = ls[i].unwrap();
        let l_f = lf[i].unwrap();
        let start_slack = working_minutes_signed(e_s, l_s, cal);
        let finish_slack = working_minutes_signed(e_f, l_f, cal);
        let total = start_slack.min(finish_slack);
        let free = free_slack(i, tasks, deps, &uid_to_idx, &es, &ef, cal).unwrap_or(total);
        let is_critical = total <= options.critical_slack_limit;
        scheduled.push(ScheduledTask {
            uid: task.uid.unwrap(),
            early_start: e_s,
            early_finish: e_f,
            late_start: l_s,
            late_finish: l_f,
            total_slack_minutes: total,
            free_slack_minutes: free,
            is_critical,
        });
    }

    ScheduleOutput {
        scheduled,
        diagnostics: diags,
    }
}

fn topological_order(
    tasks: &[Task],
    deps: &[Dependency],
    uid_to_idx: &HashMap<TaskUid, usize>,
) -> Result<Vec<usize>, Vec<TaskUid>> {
    let n = tasks.len();
    let mut out = vec![Vec::new(); n];
    let mut indeg = vec![0usize; n];
    for d in deps {
        if let (Some(&p), Some(&s)) = (
            uid_to_idx.get(&d.predecessor),
            uid_to_idx.get(&d.successor),
        ) {
            out[p].push(s);
            indeg[s] += 1;
        }
    }
    let mut queue: VecDeque<usize> = (0..n).filter(|&i| indeg[i] == 0).collect();
    let mut order = Vec::with_capacity(n);
    while let Some(i) = queue.pop_front() {
        order.push(i);
        for &j in &out[i] {
            indeg[j] -= 1;
            if indeg[j] == 0 {
                queue.push_back(j);
            }
        }
    }
    if order.len() < n {
        let in_order: std::collections::HashSet<usize> = order.iter().copied().collect();
        let cycle: Vec<TaskUid> = (0..n)
            .filter(|i| !in_order.contains(i))
            .map(|i| tasks[i].uid.unwrap())
            .collect();
        Err(cycle)
    } else {
        Ok(order)
    }
}

fn apply_early_constraint(
    task: &Task,
    es: NaiveDateTime,
    duration: i64,
    cal: &Calendar,
    diags: &mut Vec<Diagnostic>,
) -> NaiveDateTime {
    let Some(ct) = task.constraint_type else {
        return es;
    };
    let uid = task.uid;
    match ct {
        ConstraintType::AsSoonAsPossible | ConstraintType::AsLateAsPossible => es,
        ConstraintType::MustStartOn => {
            let Some(d) = task.constraint_date else {
                return es;
            };
            let d = next_working_instant(d, cal);
            if es > d {
                diags.push(Diagnostic::conflict(uid));
            }
            d
        }
        ConstraintType::MustFinishOn => {
            let Some(d) = task.constraint_date else {
                return es;
            };
            let f = prev_working_instant(d, cal);
            let s = add_working_minutes(f, -duration, cal);
            if es > s {
                diags.push(Diagnostic::conflict(uid));
            }
            s
        }
        ConstraintType::StartNoEarlierThan => {
            let Some(d) = task.constraint_date else {
                return es;
            };
            es.max(next_working_instant(d, cal))
        }
        ConstraintType::FinishNoEarlierThan => {
            let Some(d) = task.constraint_date else {
                return es;
            };
            let f = prev_working_instant(d, cal);
            es.max(add_working_minutes(f, -duration, cal))
        }
        ConstraintType::StartNoLaterThan => {
            let Some(d) = task.constraint_date else {
                return es;
            };
            if es > next_working_instant(d, cal) {
                diags.push(Diagnostic::conflict(uid));
            }
            es
        }
        ConstraintType::FinishNoLaterThan => {
            let Some(d) = task.constraint_date else {
                return es;
            };
            let f = prev_working_instant(d, cal);
            if add_working_minutes(es, duration, cal) > f {
                diags.push(Diagnostic::conflict(uid));
            }
            es
        }
    }
}

fn apply_late_constraint(
    task: &Task,
    lf: NaiveDateTime,
    duration: i64,
    cal: &Calendar,
    diags: &mut Vec<Diagnostic>,
) -> NaiveDateTime {
    let Some(ct) = task.constraint_type else {
        return lf;
    };
    let uid = task.uid;
    match ct {
        ConstraintType::AsSoonAsPossible => lf,
        ConstraintType::AsLateAsPossible => {
            diags.push(Diagnostic::alap(uid));
            lf
        }
        ConstraintType::MustStartOn => {
            let Some(d) = task.constraint_date else {
                return lf;
            };
            add_working_minutes(next_working_instant(d, cal), duration, cal)
        }
        ConstraintType::MustFinishOn => {
            let Some(d) = task.constraint_date else {
                return lf;
            };
            prev_working_instant(d, cal)
        }
        ConstraintType::StartNoEarlierThan | ConstraintType::FinishNoEarlierThan => lf,
        ConstraintType::StartNoLaterThan => {
            let Some(d) = task.constraint_date else {
                return lf;
            };
            lf.min(add_working_minutes(next_working_instant(d, cal), duration, cal))
        }
        ConstraintType::FinishNoLaterThan => {
            let Some(d) = task.constraint_date else {
                return lf;
            };
            lf.min(prev_working_instant(d, cal))
        }
    }
}

fn free_slack(
    i: usize,
    tasks: &[Task],
    deps: &[Dependency],
    uid_to_idx: &HashMap<TaskUid, usize>,
    es: &[Option<NaiveDateTime>],
    ef: &[Option<NaiveDateTime>],
    cal: &Calendar,
) -> Option<i64> {
    let task = &tasks[i];
    let e_s = es[i].unwrap();
    let e_f = ef[i].unwrap();
    let mut best: Option<i64> = None;
    for d in deps.iter().filter(|d| d.predecessor == task.uid.unwrap()) {
        let Some(&si) = uid_to_idx.get(&d.successor) else {
            continue;
        };
        let succ_es = es[si].unwrap();
        let succ_ef = ef[si].unwrap();
        let lag = d.lag.as_minutes();
        // Free-slack term (§5.6), measured in working minutes.
        let term = match d.relation_type {
            RelationType::FinishToStart => {
                working_minutes_signed(add_working_minutes(e_f, lag, cal), succ_es, cal)
            }
            RelationType::StartToStart => {
                working_minutes_signed(add_working_minutes(e_s, lag, cal), succ_es, cal)
            }
            RelationType::FinishToFinish => {
                working_minutes_signed(add_working_minutes(e_f, lag, cal), succ_ef, cal)
            }
            RelationType::StartToFinish => {
                working_minutes_signed(add_working_minutes(e_s, lag, cal), succ_ef, cal)
            }
        };
        best = Some(best.map_or(term, |b| b.min(term)));
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use domain::{Duration, Lag, TaskUid};

    fn dt(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(y, mo, d)
            .unwrap()
            .and_hms_opt(h, mi, 0)
            .unwrap()
    }

    fn task(uid: i32, minutes: i64) -> Task {
        Task {
            uid: Some(TaskUid(uid)),
            duration: Some(Duration::from_minutes(minutes)),
            ..Default::default()
        }
    }

    fn options(start: NaiveDateTime) -> ScheduleOptions {
        ScheduleOptions {
            project_start: start,
            ..Default::default()
        }
    }

    #[test]
    fn fs_chain_on_24h_calendar() {
        let cal = Calendar::twenty_four_seven("24/7");
        let tasks = vec![task(1, 480), task(2, 480)];
        let deps = vec![Dependency {
            predecessor: TaskUid(1),
            successor: TaskUid(2),
            relation_type: RelationType::FinishToStart,
            lag: Lag::from_minutes(0),
        }];
        let start = dt(2026, 11, 9, 0, 0);
        let out = schedule(&tasks, &deps, &cal, &options(start));

        assert!(out.diagnostics.is_empty());
        let a = &out.scheduled[0];
        let b = &out.scheduled[1];
        assert_eq!(a.early_start, dt(2026, 11, 9, 0, 0));
        assert_eq!(a.early_finish, dt(2026, 11, 9, 8, 0));
        assert_eq!(b.early_start, dt(2026, 11, 9, 8, 0));
        assert_eq!(b.early_finish, dt(2026, 11, 9, 16, 0));
        assert_eq!(a.total_slack_minutes, 0);
        assert_eq!(b.total_slack_minutes, 0);
        assert!(a.is_critical && b.is_critical);
        assert_eq!(a.free_slack_minutes, 0);
    }

    #[test]
    fn fnlt_constraint_creates_negative_slack() {
        let cal = Calendar::twenty_four_seven("24/7");
        let mut t = task(1, 480);
        t.constraint_type = Some(ConstraintType::FinishNoLaterThan);
        t.constraint_date = Some(dt(2026, 11, 9, 4, 0));
        let start = dt(2026, 11, 9, 0, 0);
        let out = schedule(&[t], &[], &cal, &options(start));

        assert_eq!(out.diagnostics.len(), 1);
        assert_eq!(out.diagnostics[0].code, "C001");
        let r = &out.scheduled[0];
        assert_eq!(r.early_finish, dt(2026, 11, 9, 8, 0));
        assert_eq!(r.late_finish, dt(2026, 11, 9, 4, 0));
        assert_eq!(r.total_slack_minutes, -240);
        assert!(r.is_critical);
    }

    #[test]
    fn free_slack_reflects_later_successor() {
        // A -> B (FS, tight). X -> B (FS) delays B, so A has free slack of 480 min.
        let cal = Calendar::twenty_four_seven("24/7");
        let tasks = vec![task(1, 480), task(2, 480), task(3, 960)];
        let deps = vec![
            Dependency {
                predecessor: TaskUid(1),
                successor: TaskUid(2),
                relation_type: RelationType::FinishToStart,
                lag: Lag::from_minutes(0),
            },
            Dependency {
                predecessor: TaskUid(3),
                successor: TaskUid(2),
                relation_type: RelationType::FinishToStart,
                lag: Lag::from_minutes(0),
            },
        ];
        let start = dt(2026, 11, 9, 0, 0);
        let out = schedule(&tasks, &deps, &cal, &options(start));

        let a = &out.scheduled[0]; // uid 1
        let x = &out.scheduled[2]; // uid 3
        assert_eq!(x.early_finish, dt(2026, 11, 9, 16, 0));
        assert_eq!(a.free_slack_minutes, 480);
        assert_eq!(a.total_slack_minutes, 480);
        assert!(!a.is_critical);
        assert!(x.is_critical);
    }

    #[test]
    fn cycle_is_detected() {
        let cal = Calendar::twenty_four_seven("24/7");
        let tasks = vec![task(1, 480), task(2, 480)];
        let deps = vec![
            Dependency {
                predecessor: TaskUid(1),
                successor: TaskUid(2),
                relation_type: RelationType::FinishToStart,
                lag: Lag::from_minutes(0),
            },
            Dependency {
                predecessor: TaskUid(2),
                successor: TaskUid(1),
                relation_type: RelationType::FinishToStart,
                lag: Lag::from_minutes(0),
            },
        ];
        let out = schedule(&tasks, &deps, &cal, &options(dt(2026, 11, 9, 0, 0)));
        assert!(out.scheduled.is_empty());
        assert_eq!(out.diagnostics[0].code, "C005");
    }
}
