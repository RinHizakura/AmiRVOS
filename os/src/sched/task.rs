use core::mem::size_of;

use super::exit_task;
use crate::config::*;
use crate::mm::mapping::{Mapping, PteFlag, Segment};
use crate::mm::{mapping, page};
use crate::order2size;
use crate::sched::Locked;
use crate::trap::context::TrapFrame;
use crate::trap::kernel_trap_handler;
use lazy_static::lazy_static;

#[derive(Debug)]
pub enum TaskType {
    Kernel,
    User,
}

const USER_MODE: usize = 0;
const SUPERVISOR_MODE: usize = 1;

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
    static ref TASK_ID_ALLOCATOR: Locked<TaskIdAllocator> = Locked::new(TaskIdAllocator::new());
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
        extern "C" {
            static TRAMPOLINE_START: usize;
        }
        let id = TASK_ID_ALLOCATOR.lock().alloc_task_id();

        let stack_size_order = 1;
        let stack_size = order2size!(stack_size_order);
        let stack = page::alloc(stack_size_order) as *mut u8;

        let frame_size_order = 0;
        let frame_size = order2size!(frame_size_order);
        let frame = page::alloc(frame_size_order) as *mut TrapFrame;
        assert_eq!(PAGE_SIZE, frame_size);

        let func_paddr = func as usize;
        let exit_paddr = exit_task as usize;

        let func_vaddr;
        let exit_vaddr;
        let mm;
        let stack_top;
        match task_type {
            TaskType::Kernel => {
                func_vaddr = func_paddr;
                exit_vaddr = exit_paddr;
                mm = None;
                stack_top = stack as usize + stack_size;
            }
            TaskType::User => {
                func_vaddr = TASK_START_ADDR;
                /* TODO: We should assign correct exit point for
                 * userspace task(possibly doing syscall exit there),
                 * but now we just not ready for that. */
                exit_vaddr = exit_paddr;

                let mut mapping = Mapping::new();
                mapping.map(Segment {
                    vaddr: func_vaddr as u64,
                    paddr: func_paddr as u64,
                    /* TODO: we should decide the correct size to map the function*/
                    len: PAGE_SIZE as u64,
                    flags: PteFlag::READ | PteFlag::EXECUTE | PteFlag::USER,
                });

                mapping.map(Segment {
                    vaddr: TRAMPOLINE_VA as u64,
                    paddr: unsafe { TRAMPOLINE_START as u64 },
                    len: PAGE_SIZE as u64,
                    flags: PteFlag::EXECUTE | PteFlag::READ,
                });

                mapping.map(Segment {
                    vaddr: TRAPFRAME_VA as u64,
                    paddr: frame as u64,
                    len: PAGE_SIZE as u64,
                    flags: PteFlag::READ | PteFlag::WRITE | PteFlag::USER,
                });

                /* TODO: User space's stack should not be restricted too much,
                 * we can implement page fault handler for demand paging on this. */
                mapping.map(Segment {
                    vaddr: (STACK_TOP_ADDR - stack_size) as u64,
                    paddr: stack as u64,
                    /* TODO: we should decide the correct size to map the function*/
                    len: stack_size as u64,
                    flags: PteFlag::READ | PteFlag::WRITE | PteFlag::USER,
                });
                mm = Some(mapping);
                stack_top = STACK_TOP_ADDR;
            }
        };

        let mut task = Task {
            task_type,
            task_state: TaskState::Running,
            stack,
            func,
            pc: 0,
            id,
            frame,
            mm,
        };

        // The allocated size for TrapFrame should be enough
        assert!(frame_size > size_of::<TrapFrame>());

        unsafe {
            let frame = task.frame;
            (*frame).kernel_satp = mapping::kernel_satp() as usize;
            (*frame).kernel_trap = kernel_trap_handler as usize;
            // TODO: every task should have their own kernel stack
            (*frame).kernel_sp = 0;
            /* Set return address as the start of task, so we can ret
             * to there after doing switch_to. */
            (*frame).regs[1] = func_vaddr;
            /* stack */
            (*frame).regs[2] = stack_top;
        }

        (task, id)
    }

    pub fn frame(&self) -> usize {
        match self.task_type {
            TaskType::User => TRAPFRAME_VA,
            TaskType::Kernel => self.frame as usize,
        }
    }

    pub fn satp(&self) -> usize {
        let satp = if let Some(map) = &self.mm {
            // Use the task-owned mapping
            map.satp()
        } else {
            assert!(matches!(self.task_type, TaskType::Kernel));
            mapping::kernel_satp()
        };
        satp as usize
    }

    pub fn mode(&self) -> usize {
        match self.task_type {
            TaskType::User => USER_MODE,
            TaskType::Kernel => SUPERVISOR_MODE,
        }
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
