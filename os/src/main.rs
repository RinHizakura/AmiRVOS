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
global_asm!(include_str!("asm/syscall.asm"));
global_asm!(include_str!("asm/switch.asm"));
global_asm!(include_str!("asm/utils.asm"));
global_asm!(include_str!("asm/trampoline.asm"));

#[macro_use]
mod console;

#[macro_use]
mod macros;

mod clint;
mod config;
mod cpu;
mod lock;
mod mm;
mod panic;
mod plic;
mod sched;
mod syscall;
mod trap;
mod uart;
mod utils;
mod virtio;

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
    virtio::blk::init();
    sched::init();

    /* Start the timer tick, the scheduler will then start on
     * accordingly */
    clint::set_next_tick();

    sched::scheduler();

    panic!("We don't expect to return from scheduler");
}
