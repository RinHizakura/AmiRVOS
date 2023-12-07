use crate::config::{TRAMPOLINE_VA, TRAPFRAME_VA};
use crate::cpu;
use crate::sched::do_sched;
use crate::{clint, plic, sched};

use core::arch::asm;
use lazy_static::lazy_static;
use mcause::{Interrupt as mInterrupt, Trap as mTrap};
use riscv::register::{mcause, mepc, mscratch, mtval, mtvec, satp, sip, sstatus};
use riscv::register::{scause, sepc, sscratch, stval, stvec};
use scause::{Exception as sException, Interrupt as sInterrupt, Trap as sTrap};

#[no_mangle]
pub fn timer_trap_handler() {
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
pub fn kernel_trap_handler() {
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

            do_sched();
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
}

/* This is the intermediate function which helps touse crate::cpu;
 * switch from kernel to user space with the
 * correct context */
pub fn user_trap_ret() -> ! {
    extern "C" {
        fn trampoline();
        fn uservec();
        fn userret();
    }

    let current = sched::current();
    assert!(!current.is_null());

    /* Disable interrupts until we're goinh to back
     * in the userspace, or we'll have trouble when
     * timer interrupt triggers preemption here. */
    cpu::intr_off();

    /* When getting trap at userspace, it should arrive
     * the trampoline before entering the true trap handler
     * directly. This is because kernel and user task are under
     * different context and different virtual address space. */
    unsafe {
        let uservec_va = TRAMPOLINE_VA + (uservec as usize - trampoline as usize);
        stvec::write(uservec_va, stvec::TrapMode::Direct);
    }

    unsafe {
        let frame = (*current).frame();
        assert!(!frame.is_null());

        (*frame).kernel_satp = satp::read().bits();
        (*frame).kernel_trap = kernel_trap_handler as usize;
        (*frame).kernel_sp = (*current).kstack_top() as usize;

        /* Return to epc after next sret, which is the expected
         * user space address. */
        sepc::write((*frame).epc);
        sscratch::write(TRAPFRAME_VA);
    }

    unsafe {
        // Enter user mode after next sret
        sstatus::set_spp(sstatus::SPP::User);
        // Enable interrupt after next sret
        sstatus::set_spie();
    }

    unsafe {
        let userret_va = TRAMPOLINE_VA + (userret as usize - trampoline as usize);
        let userret_f = cast_func!(userret_va, extern "C" fn(satp: usize));
        userret_f((*current).satp());
    }

    panic!("user_trap_ret()");
}

pub fn init() {
    extern "C" {
        fn timervec();
        fn kernelvec();
        static MTRAP_STACK_END: usize;
    }

    unsafe {
        mscratch::write(MTRAP_STACK_END);
        mtvec::write(timervec as usize, mtvec::TrapMode::Direct);
        stvec::write(kernelvec as usize, stvec::TrapMode::Direct);
    }
}
