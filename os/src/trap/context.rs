#[repr(C)]
#[derive(Clone, Copy)]
pub struct TrapFrame {
    pub regs: [usize; 32], // 0 ~ 255: x1 to x32 registers
    pub satp: usize,       // 256: satp
    pub epc: usize,        // 264: epc
}

impl TrapFrame {
    pub fn new() -> TrapFrame {
        TrapFrame {
            regs: [0; 32],
            satp: 0,
            epc: 0,
        }
    }
}
