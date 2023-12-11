use core::arch::asm;
use riscv::register::{sie, sstatus};

pub fn intr_on() {
    unsafe { sstatus::set_sie() };
}

pub fn intr_off() {
    unsafe { sstatus::clear_sie() };
}

pub fn timer_on() {
    unsafe { sie::set_stimer() };
    intr_on();
}

pub fn w_sip(val: usize) {
    /* TODO: We write this because sip::write is not supported */
    unsafe {
        asm!(
            "csrw sip, {x}",
            x = in(reg) val,
        );
    }
}
