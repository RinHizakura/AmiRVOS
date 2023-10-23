use crate::mm::mapping;
use crate::mm::{mapping::Mapping, page};
use crate::trap::context::TrapFrame;
use crate::order2size;

#[derive(Clone, Copy)]
pub struct TaskId(pub u32);
pub struct Task {
    stack: *mut u8,
    func: extern "C" fn(),
    pc: usize,
    id: TaskId,
    frame: *mut TrapFrame,
    mm: Mapping,
}
/* FIXME: Get avoid to unsafe if possible */
unsafe impl Send for Task {}
unsafe impl Sync for Task {}

impl Task {
    pub fn new(func: extern "C" fn(), id: TaskId) -> Self {
        let stack_size_order = 1;
        let stack_size = order2size!(stack_size_order);

        let task = Task {
            stack:  page::alloc(stack_size_order) as *mut u8,
            func: func,
            pc: 0,
            id: id,
            frame: page::alloc(1) as *mut TrapFrame,
            mm: Mapping::new(),
        };

        let func_paddr = func as usize;
        let func_vaddr = func_paddr;

        unsafe {
            let frame = task.frame;
            (*frame).epc = func_vaddr;
            (*frame).satp = mapping::global_satp() as usize;
            // stack
            (*frame).regs[2] = task.stack as usize + stack_size;
        }

        task
    }

    pub fn frame(&self) -> usize {
        self.frame as usize
    }
}
