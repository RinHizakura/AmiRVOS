use core::mem::size_of;

use super::exit_task;
use crate::mm::mapping::Mapping;
use crate::mm::{mapping, page};
use crate::order2size;
use crate::trap::context::TrapFrame;
use lazy_static::lazy_static;
use crate::sched::Locked;

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

struct TaskIdAllocator {
    /* FIXME: Just for simplicity now, maintaining a
     * 64 bits bitmap for a maximum 64 task in the OS. */
    task_id_map: u64,
}

impl TaskIdAllocator {
    pub fn new() -> Self {
        TaskIdAllocator {
            task_id_map: u64::MAX,
        }
    }

    pub fn alloc_task_id(&mut self) -> TaskId {
        let id_map = self.task_id_map;
        // FIXME:It means pids are out of use
        assert!(id_map != 0);

        let next = id_map.trailing_zeros();
        self.task_id_map &= !(1 << next);
        TaskId(next)
    }

    pub fn free_task_id(&mut self, task_id: TaskId) {
        /* This should be a allocated id */
        assert!(((self.task_id_map >> task_id.0) & 1) == 0);
        self.task_id_map |= 1 << task_id.0;
    }
}

lazy_static! {
    static ref TASK_ID_ALLOCATOR: Locked<TaskIdAllocator> =
        Locked::new(TaskIdAllocator::new());
}

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
    pub fn new(func: extern "C" fn(), task_type: TaskType) -> (Self, TaskId) {
        let id = TASK_ID_ALLOCATOR.lock().alloc_task_id();

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

        let exit_paddr = exit_task as usize;
        let exit_vaddr = exit_paddr;

        unsafe {
            let frame = task.frame;
            (*frame).pc = func_vaddr;
            /* Use return address for the task reclaim routine, */
            (*frame).regs[1] = exit_vaddr;
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

        (task, id)
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
        TASK_ID_ALLOCATOR.lock().free_task_id(self.id);
    }
}
