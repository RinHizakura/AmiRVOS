use crate::config::{TRAMPOLINE_VA, TRAPFRAME_VA};
use crate::mm::mapping;
use crate::trap::context::TrapFrame;
use crate::{clint, plic, sched};
use core::arch::asm;
use lazy_static::lazy_static;
use mcause::{Interrupt as mInterrupt, Trap as mTrap};
use riscv::register::{mcause, mepc, mscratch, mtval, mtvec, satp, sip};
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

    /* We only aim to handle timer interrupt in machine mode irq handler,
     * otherwise they are taken as invalid interrupt. */
    if !matches!(mcause.cause(), mTrap::Interrupt(mInterrupt::MachineTimer)) {
        panic!("M=Interrupted: {:?}, {:X}", mcause.cause(), mtval);
    }

    /* Arrange next timer interrupt */
    clint::set_next_tick();

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
        sTrap::Interrupt(sInterrupt::SupervisorSoft) => {
            let sip_val = sip::read().bits() & !2;
            /* TODO: We write this because sip::write is not supported */
            unsafe {
                asm!(
                    "csrw sip, {x}",
                    x = in(reg) sip_val,
                );
            }
            /* We should only receive software interrupt from a machine-mode
             * timer interrupt. Doing context switch accordingly. */
            sched::schedule();
        }
        sTrap::Exception(sException::UserEnvCall) => {
            todo!()
        }
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
        fn timervec();
    }
    let trapframe = (&*KERNEL_TRAP_FRAME as *const TrapFrame) as usize;
    mscratch::write(trapframe);
    sscratch::write(trapframe);

    unsafe {
        mtvec::write(timervec as usize, mtvec::TrapMode::Direct);
        stvec::write(TRAMPOLINE_VA, stvec::TrapMode::Direct);
    }
}
