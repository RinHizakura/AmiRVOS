use crate::mm::mapping;
use crate::mm::{mapping::Mapping, page};
use crate::trap::context::TrapFrame;

struct TaskPage {
    addr: usize,
    size: usize,
}
impl TaskPage {
    pub fn new() -> Self {
        /* FIXME: Consider the true usage to allocate appropriate size  */
        let size_order = 1; // order 1 means two pages
        let ptr = page::alloc(size_order) as usize;

        TaskPage {
            addr: ptr,
            size: (1 << size_order) * page::PAGE_SIZE,
        }
    }

    pub fn base(&self) -> usize {
        self.addr
    }

    pub fn top(&self) -> usize {
        self.addr + self.size
    }
}

#[derive(Clone, Copy)]
pub struct TaskId(pub u32);
pub struct Task {
    stack: TaskPage,
    func: extern "C" fn(),
    pc: usize,
    pub id: TaskId,
    frame: TaskPage,
    mm: Mapping,
}

impl Task {
    pub fn new(func: extern "C" fn(), id: TaskId) -> Self {
        let task = Task {
            stack: TaskPage::new(),
            func: func,
            pc: 0,
            id: id,
            frame: TaskPage::new(),
            mm: Mapping::new(),
        };

        let func_paddr = func as usize;
        let func_vaddr = func_paddr;

        unsafe {
            let frame = task.frame.base() as *mut TrapFrame;
            (*frame).epc = func_vaddr;
            (*frame).satp = mapping::global_satp() as usize;
            // stack
            (*frame).regs[2] = task.stack.top();
        }

        task
    }

    pub fn frame(&self) -> usize {
        self.frame.base()
    }
}
