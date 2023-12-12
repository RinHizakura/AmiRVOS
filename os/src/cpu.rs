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

pub fn r_sstatus() -> usize {
    /* TODO: We write this because sstatus::read doesn't support to
     * read raw bits */
    let val;
    unsafe {
        asm!(
            "csrr {x}, sstatus",
            x = out(reg) val,
        );
    }
    val
}

pub fn w_sstatus(val: usize) {
    /* TODO: We write this because sstatus::write is not supported */
    unsafe {
        asm!(
            "csrw sstatus, {x}",
            x = in(reg) val,
        );
    }
}
