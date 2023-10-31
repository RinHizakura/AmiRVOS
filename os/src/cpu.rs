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
