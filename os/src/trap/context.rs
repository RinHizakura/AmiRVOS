/* Note: take care of file 'trap.asm' and 'sched.asm' if we
 * want to change this structure's layout. */
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TrapFrame {
    pub regs: [usize; 32],  // 0 ~ 255: x1 to x32 registers
    pub kernel_satp: usize, // 256: satp
    pub kernel_trap: usize, // 264: trap handler
    pub kernel_sp: usize,   // 272: sp
    pub pc: usize,          // 280: pc
}
