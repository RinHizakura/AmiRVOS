use crate::mm::mapping::Mapping;
use crate::mm::{mapping, page};
use crate::order2size;
use crate::trap::context::TrapFrame;

pub enum TaskType {
    Kernel,
    User,
}

#[derive(Clone, Copy)]
pub struct TaskId(pub u32);

pub struct Task {
    task_type: TaskType,
    stack: *mut u8,
    func: extern "C" fn(),
    pc: usize,
    id: TaskId,
    frame: *mut TrapFrame,
    mm: Option<Mapping>,
}
/* FIXME: Get avoid to unsafe if possible */
unsafe impl Send for Task {}
unsafe impl Sync for Task {}

impl Task {
    pub fn new(func: extern "C" fn(), task_type: TaskType, id: TaskId) -> Self {
        let stack_size_order = 1;
        let stack_size = order2size!(stack_size_order);

        let task = Task {
            task_type,
            stack: page::alloc(stack_size_order) as *mut u8,
            func,
            pc: 0,
            id,
            frame: page::alloc(1) as *mut TrapFrame,
            mm: None,
        };

        let func_paddr = func as usize;
        let func_vaddr = func_paddr;

        unsafe {
            let frame = task.frame;
            (*frame).epc = func_vaddr;
            (*frame).satp = if let Some(map) = &task.mm {
                // Use the task-owned mapping
                map.satp()
            } else {
                assert!(matches!(task.task_type, TaskType::Kernel));
                mapping::kernel_satp()
            } as usize;
            // stack
            (*frame).regs[2] = task.stack as usize + stack_size;
        }

        task
    }

    pub fn frame(&self) -> usize {
        self.frame as usize
    }
}
