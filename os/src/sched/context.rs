/* Note: take care of file 'trap.asm' and 'sched.asm' if we
 * want to change this structure's layout. */
#[repr(C)]
pub struct TrapFrame {
    pub regs: [usize; 32],  // 0 ~ 255: x1 to x32 registers
    pub kernel_satp: usize, // 256: satp
    pub kernel_trap: usize, // 264: trap handler
    pub kernel_sp: usize,   // 272: sp
    pub epc: usize,         // 280: epc
}

#[repr(C)]
pub struct TaskContext {
    pub ra: usize,
    pub sp: usize,

    pub s0: usize,
    pub s1: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,
}

impl TaskContext {
    /* This should be only use for lazy_static, and
     * the initialized value is not that important. */
    pub fn new() -> TaskContext {
        TaskContext {
            ra: 0,
            sp: 0,
            s0: 0,
            s1: 0,
            s2: 0,
            s3: 0,
            s4: 0,
            s5: 0,
            s6: 0,
            s7: 0,
            s8: 0,
            s9: 0,
            s10: 0,
            s11: 0,
        }
    }
}

#[repr(C)]
pub struct Context {
    pub task_ctx: TaskContext,
    pub trapframe: TrapFrame,
}
