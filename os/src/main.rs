#![no_std] // Disables all standard library
#![no_main] // Disables emitting the main symbol
#![feature(asm, global_asm, panic_info_message, alloc_error_handler)]
#![feature(const_mut_refs)]

extern crate alloc;

global_asm!(include_str!("asm/entry.asm"));
global_asm!(include_str!("asm/mem.asm"));

#[macro_use]
mod console;

#[macro_use]
mod macros;

mod config;
mod mm;
mod panic;
mod uart;

#[no_mangle] // Disables Rust to change the symbol name
pub extern "C" fn kinit() {
    /* Do something before we switch the MMU on for
     * virtual addressing. It would be a good idea to boot
     * with Rust codes although our required work now can all
     * be done easily by assembly codes */
}

#[no_mangle]
pub extern "C" fn kmain() -> ! {
    uart::init();
    info!("Welcome to AmiRVOS world!");

    mm::init();
    // apply some run time test for memory management
    mm::test();
    loop {}
}
