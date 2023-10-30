#[repr(C)]
#[derive(Clone, Copy)]
pub struct TrapFrame {
    pub regs: [usize; 32], // 0 ~ 255: x1 to x32 registers
    pub satp: usize,       // 256: satp
    pub pc: usize,         // 264: pc
}

impl TrapFrame {
    pub fn new() -> TrapFrame {
        TrapFrame {
            regs: [0; 32],
            satp: 0,
            pc: 0,
        }
    }
}
