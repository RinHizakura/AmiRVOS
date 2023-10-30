use crate::sched::task::{Task, TaskState, TaskId};
use alloc::vec::Vec;

use super::task::TaskType;

pub struct Scheduler {
    tasks: Vec<Task>,
    current: Option<Task>,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            tasks: Vec::new(),
            current: None,
        }
    }

    pub fn spawn(&mut self, task_type: TaskType, func: extern "C" fn()) -> TaskId {
        let (task, task_id) = Task::new(func, task_type);
        self.tasks.push(task);
        task_id
    }

    pub fn kspawn(&mut self, func: extern "C" fn()) -> TaskId {
        self.spawn(TaskType::Kernel, func)
    }

    fn context_switch(&mut self, task: Task) -> Option<&Task> {
        // TODO: set page table for the next task
        self.current = Some(task);
        self.current.as_ref()
    }

    pub fn pick_next(&mut self) -> Option<&Task> {
        /* Put current task back */
        if let Some(prev) = self.current.take() {
            /* If this task is still running, put it back to the queue
             * for the next time slice */
            if matches!(prev.get_state(), TaskState::Running) {
                self.tasks.push(prev);
            }
        }

        // TODO: Add policy to pick the next task
        if let Some(task) = self.tasks.pop() {
            return self.context_switch(task);
        }

        None
    }

    pub fn cur_exit(&mut self) {
        if let Some(cur) = &mut self.current {
            cur.set_state(TaskState::Dead);
        }
    }
}
