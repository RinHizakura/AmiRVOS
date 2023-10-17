use crate::trap::context::TrapFrame;
use crate::mm::{page, mapping::Mapping};

struct TaskStack(usize);
impl TaskStack {
    pub fn new() -> Self {
        /* FIXME: Consider the true usage to allocate appropriate size  */
        let size_order = 1; // order 1 means two pages
        let ptr = page::alloc(size_order) as usize;
        TaskStack(ptr)
    }
}

#[derive(Clone, Copy)]
pub struct TaskId(pub u64);
pub struct Task {
    stack: TaskStack,
    pc: usize,
    id: TaskId,
    frame: TrapFrame,
    mm: Mapping,
}

impl Task {

    pub fn new(id: TaskId) -> Self {
        let task = Task {
            stack: TaskStack::new(),
            pc: 0,
            id: id,
            frame: TrapFrame::new(),
            mm: Mapping::new(),
        };

        task
    }
}

