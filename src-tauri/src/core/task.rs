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

    /// Record one completed pomodoro against a task.
    pub fn credit_task(&mut self, id: TaskId) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            task.spent = task.spent.saturating_add(1);
        }
    }

    pub fn done_count(&self) -> usize {
        self.tasks.iter().filter(|t| t.done).count()
    }

    /// First-run content, copied from the design's task list.
    pub fn seed_demo_tasks(&mut self) {
        if !self.tasks.is_empty() {
            return;
        }
        let seeds: [(&str, u8, u8, bool); 5] = [
            ("写产品需求文档", 3, 3, false),
            ("回 Sarah 的邮件", 1, 0, false),
            ("整理用研笔记", 2, 0, false),
            ("改登录页文案", 1, 1, true),
            ("周会前更新看板", 1, 1, true),
        ];
        for (name, estimate, spent, done) in seeds {
            let id = self.add_task(name.to_string(), estimate);
            if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
                task.spent = spent;
                task.done = done;
            }
        }
        self.timer.active_task = self.tasks.first().map(|t| t.id);
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

    #[test]
    fn seed_demo_tasks_matches_the_design() {
        let mut m = Model::default();
        m.seed_demo_tasks();
        assert_eq!(m.tasks.len(), 5);
        assert_eq!(m.tasks[0].name, "写产品需求文档");
        assert_eq!(m.tasks[0].spent, 3);
        assert_eq!(m.done_count(), 2);
        assert!(m.tasks[3].done);
        assert!(m.tasks[4].done);
    }

    #[test]
    fn seeding_twice_does_not_duplicate() {
        let mut m = Model::default();
        m.seed_demo_tasks();
        m.seed_demo_tasks();
        assert_eq!(m.tasks.len(), 5);
    }
}
