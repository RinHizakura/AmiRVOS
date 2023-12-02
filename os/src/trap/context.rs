/* Note: take care of file 'trap.asm' and 'sched.asm' if we
 * want to change this structure's layout. */
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TrapFrame {
    pub regs: [usize; 32],  // 0 ~ 255: x1 to x32 registers
    pub kernel_satp: usize, // 256: satp
    pub kernel_trap: usize, // 264: trap handler
    pub kernel_sp: usize,   // 272: sp
    pub epc: usize,         // 280: epc
}

impl TrapFrame {
    /* FIXME: This should be only use for lazy_static, and
     * the initialized value is not that important. Any better
     * to achieve the same thing? */
    pub fn new() -> TrapFrame {
        TrapFrame {
            regs: [0; 32],
            kernel_satp: 0,
            kernel_trap: 0,
            kernel_sp: 0,
            epc: 0,
        }
    }
}
