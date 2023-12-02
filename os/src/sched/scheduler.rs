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

    pub fn put_prev(&mut self) {
        if let Some(prev) = self.current.take() {
            /* If this task is still running, put it back to the queue
             * for the next time slice */
            if matches!(prev.get_state(), TaskState::Running) {
                self.tasks.push_back(prev);
            } else {
                // Rust will drop it automatically?
            }
        }
    }

    pub fn pick_next(&mut self) -> Option<&Task> {
        /* Put current task back if there's any */
        self.put_prev();

        // TODO: Add policy to pick the next task
        if let Some(task) = self.tasks.pop_front() {
            self.current = Some(task);
            return self.current.as_ref();
        }

        None
    }

    pub fn cur_exit(&mut self) {
        if let Some(cur) = &mut self.current {
            cur.set_state(TaskState::Dead);
        }
    }
}
