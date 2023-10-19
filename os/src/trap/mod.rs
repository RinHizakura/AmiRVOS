use crate::{plic, clint};
use crate::trap::context::TrapFrame;
use riscv::register::{
    mcause,
    mscratch, scause, sscratch,
};
use mcause::{Trap as mTrap, Interrupt as mInterrupt};
use scause::{Trap as sTrap, Interrupt as sInterrupt, Exception as sException};
use lazy_static::lazy_static;

pub mod context;

lazy_static! {
    static ref KERNEL_TRAP_FRAME: TrapFrame = TrapFrame::new();
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

    assert_eq!(sepc, KERNEL_TRAP_FRAME.epc);
    match scause.cause() {
        sTrap::Interrupt(sInterrupt::SupervisorExternal) => plic::irq_handler(),
        sTrap::Exception(sException::Breakpoint) => return_pc += 2,
        _ => panic!("S=Interrupted: {:?}, {:X} {:X}", scause.cause(), stval, sepc),
    }

    return_pc
}

pub fn init() {
    let trapframe = (&*KERNEL_TRAP_FRAME as *const TrapFrame) as usize;
    mscratch::write(trapframe);
    sscratch::write(trapframe);
}
