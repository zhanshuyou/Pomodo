use crate::model::{Model, Task, TaskId};

impl Model {
    pub fn add_task(&mut self, name: String, estimate: u8) -> TaskId {
        let id = self.next_task_id;
        self.next_task_id = self.next_task_id.saturating_add(1);
        self.tasks.push(Task {
            id,
            name,
            estimate,
            spent: 0,
            done: false,
        });
        id
    }

    /// Flip a task's done flag. Returns the new value, or false if the id is unknown.
    pub fn toggle_task(&mut self, id: TaskId) -> bool {
        match self.tasks.iter_mut().find(|t| t.id == id) {
            Some(task) => {
                task.done = !task.done;
                task.done
            }
            None => false,
        }
    }

    pub fn delete_task(&mut self, id: TaskId) {
        self.tasks.retain(|t| t.id != id);
        if self.timer.active_task == Some(id) {
            self.timer.active_task = None;
        }
    }

    /// Rename; a blank name is ignored rather than leaving an unnamed row.
    pub fn rename_task(&mut self, id: TaskId, name: &str) -> bool {
        let name = name.trim();
        if name.is_empty() {
            return false;
        }
        match self.tasks.iter_mut().find(|t| t.id == id) {
            Some(task) => {
                task.name = name.to_string();
                true
            }
            None => false,
        }
    }

    pub fn set_task_estimate(&mut self, id: TaskId, estimate: u8) -> bool {
        match self.tasks.iter_mut().find(|t| t.id == id) {
            Some(task) => {
                task.estimate = estimate.max(1);
                true
            }
            None => false,
        }
    }

    /// Put the tasks in the order `ids` lists them. Ids that are unknown are
    /// skipped and tasks the list leaves out keep their relative order at the
    /// end, so a stale list from a window that missed an update cannot lose
    /// anything.
    pub fn reorder_tasks(&mut self, ids: &[TaskId]) {
        let mut ordered: Vec<Task> = Vec::with_capacity(self.tasks.len());
        for id in ids {
            if let Some(pos) = self.tasks.iter().position(|t| t.id == *id) {
                ordered.push(self.tasks.remove(pos));
            }
        }
        ordered.append(&mut self.tasks);
        self.tasks = ordered;
    }

    /// Record one completed pomodoro against a task.
    pub fn credit_task(&mut self, id: TaskId) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            task.spent = task.spent.saturating_add(1);
        }
    }

    pub fn done_count(&self) -> usize {
        self.tasks.iter().filter(|t| t.done).count()
    }
}

#[cfg(test)]
mod tests {
    use crate::model::Model;

    #[test]
    fn add_task_assigns_increasing_ids() {
        let mut m = Model::default();
        let a = m.add_task("写产品需求文档".into(), 3);
        let b = m.add_task("回 Sarah 的邮件".into(), 1);
        assert_ne!(a, b);
        assert_eq!(m.tasks.len(), 2);
        assert_eq!(m.tasks[0].name, "写产品需求文档");
        assert_eq!(m.tasks[0].estimate, 3);
        assert_eq!(m.tasks[0].spent, 0);
        assert!(!m.tasks[0].done);
    }

    #[test]
    fn toggle_task_flips_done_and_reports_the_new_value() {
        let mut m = Model::default();
        let id = m.add_task("整理用研笔记".into(), 2);
        assert!(m.toggle_task(id));
        assert!(m.tasks[0].done);
        assert!(!m.toggle_task(id));
        assert!(!m.tasks[0].done);
    }

    #[test]
    fn toggling_a_missing_task_is_a_no_op() {
        let mut m = Model::default();
        assert!(!m.toggle_task(999));
    }

    #[test]
    fn rename_task_trims_and_refuses_blanks() {
        let mut m = Model::default();
        let id = m.add_task("改登录页文案".into(), 1);
        assert!(m.rename_task(id, "  改注册页文案 "));
        assert_eq!(m.tasks[0].name, "改注册页文案");
        assert!(!m.rename_task(id, "   "));
        assert_eq!(m.tasks[0].name, "改注册页文案");
        assert!(!m.rename_task(999, "x"));
    }

    #[test]
    fn set_task_estimate_never_goes_below_one() {
        let mut m = Model::default();
        let id = m.add_task("整理用研笔记".into(), 2);
        assert!(m.set_task_estimate(id, 3));
        assert_eq!(m.tasks[0].estimate, 3);
        assert!(m.set_task_estimate(id, 0));
        assert_eq!(m.tasks[0].estimate, 1);
    }

    #[test]
    fn reorder_tasks_follows_the_list_and_keeps_stragglers() {
        let mut m = Model::default();
        let a = m.add_task("a".into(), 1);
        let b = m.add_task("b".into(), 1);
        let c = m.add_task("c".into(), 1);
        m.reorder_tasks(&[c, 42, a]);
        let names: Vec<&str> = m.tasks.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, ["c", "a", "b"]);
        assert_eq!(m.tasks[0].id, c);
        let _ = b;
    }

    #[test]
    fn delete_task_removes_it() {
        let mut m = Model::default();
        let id = m.add_task("改登录页文案".into(), 1);
        m.delete_task(id);
        assert!(m.tasks.is_empty());
    }

    #[test]
    fn credit_task_increments_spent_and_saturates() {
        let mut m = Model::default();
        let id = m.add_task("周会前更新看板".into(), 1);
        m.credit_task(id);
        m.credit_task(id);
        assert_eq!(m.tasks[0].spent, 2);

        m.tasks[0].spent = u8::MAX;
        m.credit_task(id);
        assert_eq!(m.tasks[0].spent, u8::MAX);
    }

    #[test]
    fn done_count_counts_only_finished_tasks() {
        let mut m = Model::default();
        let a = m.add_task("a".into(), 1);
        m.add_task("b".into(), 1);
        m.toggle_task(a);
        assert_eq!(m.done_count(), 1);
    }
}
