use crate::clint;
use lazy_static::lazy_static;
use riscv::register::{
    mcause,
    mcause::{Interrupt, Trap},
    mscratch, scause, sscratch,
};

lazy_static! {
    static ref M_KERNEL_TRAP_FRAME: TrapFrame = TrapFrame::new();
    static ref S_KERNEL_TRAP_FRAME: TrapFrame = TrapFrame::new();
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
pub fn m_irq_handler(mepc: usize, mcause: mcause::Mcause, mtval: usize) -> usize {
    let mut return_pc = mepc;
    warning!("M=Interrupted: {:?}, {:X}", mcause.cause(), mtval);

    /* We only aim to handle timer interrupt in machine mode irq handler now, otherwise
     * they are taken as invalid interrupt.  */
    match mcause.cause() {
        Trap::Interrupt(Interrupt::MachineTimer) => clint::set_next_tick(),
        _ => panic!("M=Interrupted: {:?}, {:X}", mcause.cause(), mtval),
    }
    return_pc
}

#[no_mangle]
pub fn s_irq_handler(sepc: usize, scause: scause::Scause, stval: usize) -> usize {
    let mut return_pc = sepc;
    warning!("S=Interrupted: {:?}, {:X}", scause.cause(), stval);

    assert_eq!(sepc, S_KERNEL_TRAP_FRAME.epc);
    // TODO: we should return correct PC according to the trap type
    return_pc + 4
}

pub fn sinit() {
    let sscratch_base = (&*S_KERNEL_TRAP_FRAME as *const TrapFrame) as usize;
    sscratch::write(sscratch_base);
}

pub fn minit() {
    let mscratch_base = (&*M_KERNEL_TRAP_FRAME as *const TrapFrame) as usize;
    mscratch::write(mscratch_base);
}
