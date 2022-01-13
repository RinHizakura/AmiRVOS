use lazy_static::lazy_static;
use riscv::register::{mcause, scause, sscratch};

lazy_static! {
    static ref KERNEL_TRAP_FRAME: TrapFrame = TrapFrame::new();
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TrapFrame {
    pub regs: [usize; 32], // 0 ~ 255: x1 to x32 registers
    pub satp: usize,       // 256: satp
    pub epc: usize,        // 264: epc
}

impl TrapFrame {
    fn new() -> TrapFrame {
        TrapFrame {
            regs: [0; 32],
            satp: 0,
            epc: 0,
        }
    }
}

#[no_mangle]
pub fn m_irq_handler(mcause: mcause::Mcause, mtval: usize) {
    panic!("M=Interrupted: {:?}, {:X}", mcause.cause(), mtval);
}

#[no_mangle]
pub fn s_irq_handler(sepc: usize, scause: scause::Scause, stval: usize) -> usize {
    let mut return_pc = sepc;
    warning!("S=Interrupted: {:?}, {:X}", scause.cause(), stval);

    assert_eq!(sepc, KERNEL_TRAP_FRAME.epc);
    // TODO: we should return correct PC according to the trap type
    return_pc + 4
}

pub fn init() {
    sscratch::write((&*KERNEL_TRAP_FRAME as *const TrapFrame) as usize);
}
