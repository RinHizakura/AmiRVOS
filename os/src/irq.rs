use crate::clint;
use lazy_static::lazy_static;
use riscv::register::{
    mcause,
    mscratch, scause, sscratch,
};
use mcause::{Trap as mTrap, Interrupt as mInterrupt};
use scause::{Trap as sTrap, Interrupt as sInterrupt, Exception as sException};

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
        mTrap::Interrupt(mInterrupt::MachineTimer) => clint::set_next_tick(),
        _ => panic!("M=Interrupted: {:?}, {:X}", mcause.cause(), mtval),
    }
    return_pc
}

#[no_mangle]
pub fn s_irq_handler(sepc: usize, scause: scause::Scause, stval: usize) -> usize {
    let mut return_pc = sepc;
    warning!("S=Interrupted: {:?}, {:X} {:X}", scause.cause(), stval, sepc);

    assert_eq!(sepc, S_KERNEL_TRAP_FRAME.epc);
    match scause.cause() {
        sTrap::Interrupt(sInterrupt::SupervisorExternal) => panic!("todo"),
        sTrap::Exception(sException::Breakpoint) => return_pc += 2,
        _ => panic!("S=Interrupted: {:?}, {:X} {:X}", scause.cause(), stval, sepc),
    }

    return_pc
}

pub fn sinit() {
    let sscratch_base = (&*S_KERNEL_TRAP_FRAME as *const TrapFrame) as usize;
    sscratch::write(sscratch_base);
}

pub fn minit() {
    let mscratch_base = (&*M_KERNEL_TRAP_FRAME as *const TrapFrame) as usize;
    mscratch::write(mscratch_base);
}
