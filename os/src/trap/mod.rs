use crate::config::TRAMPOLINE_VA;
use crate::mm::mapping;
use crate::trap::context::TrapFrame;
use crate::{clint, plic, sched};
use lazy_static::lazy_static;
use mcause::{Interrupt as mInterrupt, Trap as mTrap};
use riscv::register::{mcause, mepc, mscratch, mtval, mtvec, satp};
use riscv::register::{scause, sepc, sscratch, stval, stvec};
use scause::{Exception as sException, Interrupt as sInterrupt, Trap as sTrap};

pub mod context;

lazy_static! {
    static ref KERNEL_TRAP_FRAME: TrapFrame = TrapFrame::new();
}

#[no_mangle]
pub fn m_irq_handler() {
    let mepc = mepc::read();
    let mtval = mtval::read();
    let mcause = mcause::read();

    warning!("M=Interrupted: {:?}, {:X}", mcause.cause(), mtval);

    /* We only aim to handle timer interrupt in machine mode irq handler now, otherwise
     * they are taken as invalid interrupt.  */
    match mcause.cause() {
        mTrap::Interrupt(mInterrupt::MachineTimer) => {
            clint::set_next_tick();
            sched::schedule();
        }
        _ => panic!("M=Interrupted: {:?}, {:X}", mcause.cause(), mtval),
    }

    mepc::write(mepc);
}

#[no_mangle]
pub fn s_irq_handler() -> usize {
    let mut sepc = sepc::read();
    let stval = stval::read();
    let scause = scause::read();

    warning!(
        "S=Interrupted: {:?}, {:X} {:X}",
        scause.cause(),
        stval,
        sepc
    );

    match scause.cause() {
        sTrap::Interrupt(sInterrupt::SupervisorExternal) => plic::irq_handler(),
        sTrap::Exception(sException::Breakpoint) => sepc += 2,
        _ => panic!(
            "S=Interrupted: {:?}, {:X} {:X}",
            scause.cause(),
            stval,
            sepc
        ),
    }

    sepc::write(sepc);

    mapping::kernel_satp() as usize
}

pub fn init() {
    extern "C" {
        fn m_trap_vector();
        fn s_trap_vector();
    }
    let trapframe = (&*KERNEL_TRAP_FRAME as *const TrapFrame) as usize;
    mscratch::write(trapframe);
    sscratch::write(trapframe);

    unsafe {
        mtvec::write(m_trap_vector as usize, mtvec::TrapMode::Direct);
        stvec::write(TRAMPOLINE_VA, stvec::TrapMode::Direct);
    }
}
