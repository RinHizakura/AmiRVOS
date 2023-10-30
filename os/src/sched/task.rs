use core::mem::size_of;

use crate::mm::mapping::Mapping;
use crate::mm::{mapping, page};
use crate::order2size;
use crate::trap::context::TrapFrame;

#[derive(Debug)]
pub enum TaskType {
    Kernel,
    User,
}

#[derive(Debug)]
pub enum TaskState {
    Running,
    Sleeping,
    Dead,
}

#[derive(Clone, Copy)]
pub struct TaskId(pub u32);

pub struct Task {
    pub id: TaskId,
    task_type: TaskType,
    task_state: TaskState,
    stack: *mut u8,
    func: extern "C" fn(),
    pc: usize,
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

        let frame_size_order = 0;
        let frame_size = order2size!(frame_size_order);

        let task = Task {
            task_type,
            task_state: TaskState::Running,
            stack: page::alloc(stack_size_order) as *mut u8,
            func,
            pc: 0,
            id,
            frame: page::alloc(frame_size_order) as *mut TrapFrame,
            mm: None,
        };

        // The allocated size for TrapFrame should be enough
        assert!(frame_size > size_of::<TrapFrame>());

        let func_paddr = func as usize;
        let func_vaddr = func_paddr;

        unsafe {
            let frame = task.frame;
            (*frame).pc = func_vaddr;
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

    pub fn get_state(&self) -> &TaskState {
        &self.task_state
    }

    pub fn set_state(&mut self, state: TaskState) {
        self.task_state = state;
    }
}

impl Drop for Task {
    fn drop(&mut self) {
        page::free(self.stack as *mut u8);
        page::free(self.frame as *mut u8);
    }
}
