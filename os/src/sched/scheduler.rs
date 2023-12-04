use crate::sched::task::{Task, TaskId, TaskState};
use alloc::collections::VecDeque;

use super::task::TaskType;

pub struct Scheduler {
    tasks: VecDeque<Task>,
    current: Option<Task>,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            tasks: VecDeque::new(),
            current: None,
        }
    }

    fn spawn(&mut self, task_type: TaskType, func: extern "C" fn()) -> TaskId {
        let (task, task_id) = Task::new(func, task_type);
        self.tasks.push_back(task);
        task_id
    }

    pub fn kspawn(&mut self, func: extern "C" fn()) -> TaskId {
        self.spawn(TaskType::Kernel, func)
    }

    pub fn uspawn(&mut self, func: extern "C" fn()) -> TaskId {
        self.spawn(TaskType::User, func)
    }

    pub fn put_prev(&mut self) -> &Task {
        /* Put current task back if there's any */
        if let Some(prev) = self.current.take() {
            assert!(matches!(prev.get_state(), TaskState::Running));
            self.tasks.push_back(prev);
            return self.tasks.back().expect("put_prev()");
        }

        panic!("put_prev() is not expected to fail");
    }

    pub fn pick_next(&mut self) -> &Task {
        /* We should only pick a new task by explcitly
         * put back the current task first(if any). */
        assert!(self.current.is_none());

        // TODO: Add policy to pick the next task
        if let Some(task) = self.tasks.pop_front() {
            self.current = Some(task);
            return self.current.as_ref().expect("pick_next()");
        }

        panic!("pick_next() is not expected to fail");
    }
}
