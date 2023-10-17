use alloc::vec::Vec;
use crate::sched::task::{Task, TaskId};

pub struct Scheduler {
    /* FIXME: Just for simplicity now, maintaining a
     * 64 bits bitmap for a maximum 64 task in the OS. */
    task_id_map: u64,
    tasks: Vec<Task>,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            task_id_map: u64::MAX,
            tasks: Vec::new(),
        }
    }

    fn get_task_id(&mut self) -> TaskId {
        let id_map = self.task_id_map;
        // FIXME:It means pids are out of use
        assert!(id_map != 0);

        let next = id_map & (1 << id_map.trailing_zeros());
        self.task_id_map &= !(1 << next);
        TaskId(next)
    }

    pub fn spawn(&mut self) -> TaskId {
        let task_id = self.get_task_id();
        let task = Task::new(task_id);
        self.tasks.push(task);
        task_id
    }
}

