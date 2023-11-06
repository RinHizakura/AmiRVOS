#![no_std] // Disables all standard library
#![no_main]
// Disables emitting the main symbol

/* TODO: consider to not rely on these features to stay in
 * stable channel */
#![feature(panic_info_message, alloc_error_handler)]
#![feature(const_mut_refs)]
#![feature(fn_align)]

extern crate alloc;

use core::arch::global_asm;

global_asm!(include_str!("asm/entry.asm"));
global_asm!(include_str!("asm/mem.asm"));
global_asm!(include_str!("asm/trap.asm"));
global_asm!(include_str!("asm/sched.asm"));

#[macro_use]
mod console;

#[macro_use]
mod macros;

mod clint;
mod config;
mod lock;
mod mm;
mod panic;
mod plic;
mod sched;
mod trap;
mod uart;
mod utils;
mod cpu;

#[no_mangle] // Disables Rust to change the symbol name
pub extern "C" fn kinit() {
    /* Initialize UART for debugging message as early as possible */
    uart::init();
    /* Prepare memory subsystem before entering supervisor mode */
    mm::init();
    /* Setup trap registers before enabling interrupt/exception */
    trap::init();
}

#[no_mangle]
pub extern "C" fn kmain() -> ! {
    print!("Welcome to AmiRVOS world!\n");

    plic::init();
    sched::init();

    /* Enable the interrupt for timer */
    cpu::timer_on();

    /* Start the timer tick, the scheduler will then start on
     * accordingly */
    clint::set_next_tick();
    loop {}
}
