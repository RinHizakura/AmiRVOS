#![no_std] // Disables all standard library
#![no_main] // Disables emitting the main symbol

/* TODO: consider to not rely on these features to stay in
 * stable channel */
#![feature(panic_info_message, alloc_error_handler)]
#![feature(const_mut_refs)]

extern crate alloc;

use core::arch::{asm, global_asm};

global_asm!(include_str!("asm/entry.asm"));
global_asm!(include_str!("asm/mem.asm"));
global_asm!(include_str!("asm/trap.asm"));

#[macro_use]
mod console;

#[macro_use]
mod macros;

mod clint;
mod config;
mod trap;
mod mm;
mod sched;
mod panic;
mod plic;
mod uart;
mod utils;
mod lock;

#[no_mangle] // Disables Rust to change the symbol name
pub extern "C" fn kinit() {
    /* We can do something more before we switch the MMU on for
     * virtual addressing. For example, it would be a good idea to
     * initialize the page table here using Rust codes directly. */
    trap::init();
}

#[no_mangle]
pub extern "C" fn kmain() -> ! {
    uart::init();

    print!("Welcome to AmiRVOS world!\n");

    mm::init();
    clint::init();
    plic::init();
    sched::init();

    loop {}
}
