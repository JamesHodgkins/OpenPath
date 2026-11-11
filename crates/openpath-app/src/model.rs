//! The editable plan: a [`Project`] plus its latest schedule. All edits go through methods
//! here so the schedule is recomputed consistently and the UI stays a thin view.

use std::collections::HashMap;

use openpath_core::{Dependency, Duration, Lag, Project, RelationType, Task, TaskUid};
use openpath_schedule::{ScheduleOptions, ScheduleOutput, ScheduledTask};

pub struct Plan {
    pub project: Project,
    pub output: ScheduleOutput,
    by_uid: HashMap<TaskUid, usize>,
}

impl Plan {
    pub fn new(project: Project) -> Self {
        let mut plan = Plan {
            project,
            output: ScheduleOutput {
                scheduled: Vec::new(),
                diagnostics: Vec::new(),
            },
            by_uid: HashMap::new(),
        };
        plan.reschedule();
        plan
    }

    /// Recompute the schedule and write start/finish back onto the tasks.
    pub fn reschedule(&mut self) {
        let (Some(cal), Some(start)) = (self.project.calendars.first(), self.project.start_date)
        else {
            return;
        };
        let options = ScheduleOptions {
            project_start: start,
            ..Default::default()
        };
        self.output = openpath_schedule::schedule(
            &self.project.tasks,
            &self.project.dependencies,
            cal,
            &options,
        );
        self.by_uid = self
            .output
            .scheduled
            .iter()
            .enumerate()
            .map(|(i, s)| (s.uid, i))
            .collect();
        for s in &self.output.scheduled {
            if let Some(t) = self.project.tasks.iter_mut().find(|t| t.uid == Some(s.uid)) {
                t.start = Some(s.early_start);
                t.finish = Some(s.early_finish);
            }
        }
        self.project.finish_date = self.output.scheduled.iter().map(|s| s.early_finish).max();
    }

    pub fn scheduled(&self, uid: TaskUid) -> Option<&ScheduledTask> {
        self.by_uid.get(&uid).map(|&i| &self.output.scheduled[i])
    }

    pub fn minutes_per_day(&self) -> i64 {
        self.project.minutes_per_day.unwrap_or(480) as i64
    }

    pub fn duration_days(&self, task: &Task) -> f64 {
        task.duration.map_or(0.0, |d| {
            d.as_minutes() as f64 / self.minutes_per_day() as f64
        })
    }

    pub fn set_duration_days(&mut self, index: usize, days: f64) {
        let minutes = (days * self.minutes_per_day() as f64).round() as i64;
        if let Some(t) = self.project.tasks.get_mut(index) {
            t.duration = Some(Duration::from_minutes(minutes.max(0)));
            t.is_milestone = minutes <= 0;
        }
        self.reschedule();
    }

    pub fn rename(&mut self, index: usize, name: String) {
        if let Some(t) = self.project.tasks.get_mut(index) {
            t.name = Some(name);
        }
    }

    /// Append a task (one day long). When `after` is given the new task is linked
    /// finish-to-start from it. Returns the new row index.
    pub fn add_task(&mut self, after: Option<usize>) -> usize {
        let uid = self
            .project
            .tasks
            .iter()
            .filter_map(|t| t.uid)
            .map(|u| u.0)
            .max()
            .unwrap_or(0)
            + 1;
        let row = self.project.tasks.len() as i32 + 1;
        self.project.tasks.push(Task {
            uid: Some(TaskUid(uid)),
            id: Some(row),
            name: Some(format!("New task {uid}")),
            duration: Some(Duration::from_minutes(self.minutes_per_day())),
            ..Default::default()
        });
        if let Some(pred) = after
            .and_then(|i| self.project.tasks.get(i))
            .and_then(|t| t.uid)
        {
            self.project.dependencies.push(Dependency {
                predecessor: pred,
                successor: TaskUid(uid),
                relation_type: RelationType::FinishToStart,
                lag: Lag::from_minutes(0),
            });
        }
        self.reschedule();
        self.project.tasks.len() - 1
    }

    pub fn delete_task(&mut self, index: usize) {
        if index >= self.project.tasks.len() {
            return;
        }
        let uid = self.project.tasks.remove(index).uid;
        self.project
            .dependencies
            .retain(|d| Some(d.predecessor) != uid && Some(d.successor) != uid);
        for (i, t) in self.project.tasks.iter_mut().enumerate() {
            t.id = Some(i as i32 + 1);
        }
        self.reschedule();
    }

    pub fn critical_count(&self) -> usize {
        self.output
            .scheduled
            .iter()
            .filter(|s| s.is_critical)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use openpath_core::sample::sample_project;

    #[test]
    fn sample_schedules_and_has_critical_path() {
        let plan = Plan::new(sample_project());
        assert_eq!(plan.output.scheduled.len(), 5);
        assert!(plan.output.diagnostics.is_empty());
        // Design→Implement→Test→Ship is the longest chain; Docs has slack.
        assert!(plan.scheduled(TaskUid(1)).unwrap().is_critical);
        assert!(!plan.scheduled(TaskUid(4)).unwrap().is_critical);
        assert!(plan.project.finish_date.is_some());
    }

    #[test]
    fn lengthening_a_critical_task_pushes_the_finish() {
        let mut plan = Plan::new(sample_project());
        let before = plan.project.finish_date.unwrap();
        plan.set_duration_days(0, 5.0);
        assert!(plan.project.finish_date.unwrap() > before);
        assert_eq!(plan.duration_days(&plan.project.tasks[0]), 5.0);
    }

    #[test]
    fn zero_duration_becomes_milestone() {
        let mut plan = Plan::new(sample_project());
        plan.set_duration_days(3, 0.0);
        assert!(plan.project.tasks[3].is_milestone);
    }

    #[test]
    fn add_links_after_selection_and_delete_removes_links() {
        let mut plan = Plan::new(sample_project());
        let deps = plan.project.dependencies.len();
        let new = plan.add_task(Some(4)); // after "Ship"
        assert_eq!(plan.project.dependencies.len(), deps + 1);
        let ship = plan.scheduled(TaskUid(5)).unwrap().early_finish;
        let added = plan.project.tasks[new].uid.unwrap();
        assert!(plan.scheduled(added).unwrap().early_start >= ship);

        plan.delete_task(new);
        assert_eq!(plan.project.dependencies.len(), deps);
        assert_eq!(plan.project.tasks.len(), 5);
    }
}
