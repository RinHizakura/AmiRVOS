use core::mem::size_of;
use core::ptr;

use crate::config::*;
use crate::mm::mapping::{Mapping, PteFlag, Segment};
use crate::mm::{mapping, page};
use crate::order2size;
use crate::sched::context::*;
use crate::sched::Locked;
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
    Runnable,
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
    func: extern "C" fn(),
    mm: Option<Mapping>,

    kstack: *mut u8,
    ustack: *mut u8,
    context: *mut Context,
}
/* FIXME: Get avoid to unsafe if possible */
unsafe impl Send for Task {}
unsafe impl Sync for Task {}

impl Task {
    fn init_mm(&mut self) {
        extern "C" {
            static TRAMPOLINE_START: usize;
        }

        let frame = self.frame();
        if let Some(mapping) = &mut self.mm {
            assert!(matches!(self.task_type, TaskType::User));

            let func_vaddr = TASK_START_ADDR;
            let func_paddr = self.func as usize;

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

            mapping.map(Segment {
                vaddr: (STACK_TOP_ADDR - PAGE_SIZE) as u64,
                paddr: self.ustack as u64,
                /* TODO: we should decide the correct size to map the function*/
                len: PAGE_SIZE as u64,
                flags: PteFlag::READ | PteFlag::WRITE | PteFlag::USER,
            });
        }
    }

    fn init_context(&mut self) {
        unsafe {
            let frame = self.frame();
            (*frame).kernel_satp = mapping::kernel_satp() as usize;
            (*frame).kernel_trap = kernel_trap_handler as usize;
            // TODO: every task should have their own kernel stack
            (*frame).kernel_sp = 0;
        }

        unsafe {
            let ctx = self.task_context();
            (*ctx).ra = match self.task_type {
                TaskType::Kernel => self.func as usize,
                TaskType::User => TASK_START_ADDR,
            };
            (*ctx).sp = match self.task_type {
                TaskType::Kernel => self.kstack as usize + PAGE_SIZE,
                TaskType::User => STACK_TOP_ADDR,
            };
        }
    }

    pub fn new(func: extern "C" fn(), task_type: TaskType) -> (Self, TaskId) {
        let id = TASK_ID_ALLOCATOR.lock().alloc_task_id();

        let stack_size_order = 0;
        let stack_size = order2size!(stack_size_order);
        assert_eq!(stack_size, PAGE_SIZE);

        let kstack = page::alloc(stack_size_order);

        /* TODO: User space's stack should not be restricted in one page,
         * we can implement page fault handler for demand paging on this. */
        let ustack = match task_type {
            TaskType::Kernel => ptr::null_mut(),
            TaskType::User => page::alloc(stack_size_order),
        };

        let context_size_order = 0;
        let context_size = order2size!(context_size_order);
        let context = page::alloc(context_size_order) as *mut Context;
        assert!(size_of::<Context>() <= context_size);

        let mm = match task_type {
            TaskType::Kernel => None,
            TaskType::User => Some(Mapping::new()),
        };

        let mut task = Task {
            id,
            task_type,
            task_state: TaskState::Runnable,
            func,
            mm,
            kstack,
            ustack,
            context,
        };

        task.init_mm();
        task.init_context();

        (task, id)
    }

    pub fn frame(&self) -> *mut TrapFrame {
        unsafe { &mut (*self.context).trapframe as *mut TrapFrame }
    }

    pub fn task_context(&self) -> *mut TaskContext {
        unsafe { &mut (*self.context).task_ctx as *mut TaskContext }
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
        page::free(self.kstack);
        page::free(self.context as *mut u8);
        if !self.ustack.is_null() {
            assert!(matches!(self.task_type, TaskType::User));
            page::free(self.ustack);
        }
        TASK_ID_ALLOCATOR.lock().free_task_id(self.id);
    }
}
