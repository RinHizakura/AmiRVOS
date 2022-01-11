use riscv::register::{mcause, scause, sie, stvec};

#[no_mangle]
pub fn m_irq_handler(mcause: mcause::Mcause, mtval: usize) {
    panic!("M=Interrupted: {:?}, {:X}", mcause.cause(), mtval);
}

#[no_mangle]
pub fn s_irq_handler(scause: scause::Scause, stval: usize) {
    panic!("S=Interrupted: {:?}, {:X}", scause.cause(), stval);
}
