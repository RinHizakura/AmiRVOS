#![no_std] // Disables all standard library
#![no_main] // Disables emitting the main symbol
#![feature(asm, global_asm, panic_info_message, alloc_error_handler)]
#![feature(const_mut_refs)]

extern crate alloc;

global_asm!(include_str!("asm/entry.asm"));
global_asm!(include_str!("asm/mem.asm"));
global_asm!(include_str!("asm/trap.asm"));

#[macro_use]
mod console;

#[macro_use]
mod macros;

mod clint;
mod config;
mod irq;
mod mm;
mod panic;
mod uart;

#[no_mangle] // Disables Rust to change the symbol name
pub extern "C" fn kinit() {
    /* Do something before we switch the MMU on for
     * virtual addressing. For example, it would be
     * a good idea to initialize the page table here
     * using Rust codes directly */
}

#[no_mangle]
pub extern "C" fn kmain() -> ! {
    uart::init();
    irq::init();
    print!("Welcome to AmiRVOS world!\n");

    mm::init();
    // apply some run time test for memory management
    mm::test();

    // intentionally trigger a trap
    println!("Trigger ebreak");
    unsafe {
        asm!("ebreak");
    };
    println!("We'll back!");

    clint::init();
    loop {}
}
