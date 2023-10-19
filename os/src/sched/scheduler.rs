use crate::sched::task::{Task, TaskId};
use alloc::vec::Vec;

pub struct Scheduler {
    /* FIXME: Just for simplicity now, maintaining a
     * 64 bits bitmap for a maximum 64 task in the OS. */
    task_id_map: u64,
    tasks: Vec<Task>,
    current: Option<Task>,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            task_id_map: u64::MAX,
            tasks: Vec::new(),
            current: None,
        }
    }

    fn alloc_task_id(&mut self) -> TaskId {
        let id_map = self.task_id_map;
        // FIXME:It means pids are out of use
        assert!(id_map != 0);

        let next = id_map.trailing_zeros();
        self.task_id_map &= !(1 << next);
        TaskId(next)
    }

    pub fn spawn(&mut self, func: extern "C" fn()) -> TaskId {
        let task_id = self.alloc_task_id();
        let task = Task::new(func, task_id);
        self.tasks.push(task);
        task_id
    }

    fn context_switch(&mut self, task: Task) -> Option<&Task> {
        if let Some(prev) = self.current.take() {
            self.tasks.push(prev);
        }

        // TODO: set page table for the next task
        self.current = Some(task);
        self.current.as_ref()
    }

    pub fn pick_next(&mut self) -> Option<&Task> {
        // TODO: Add policy to pick the next task
        if let Some(task) = self.tasks.pop() {
            return self.context_switch(task);
        }

        None
    }
}
