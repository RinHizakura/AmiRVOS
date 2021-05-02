#![no_std] // Disables all standard library
#![no_main] // Disables emitting the main symbol
#![feature(asm, global_asm, panic_info_message)]

global_asm!(include_str!("entry.asm"));

#[macro_use]
mod console;

mod panic;
mod uart;

#[no_mangle] // Disables Rust to change the symbol name
pub extern "C" fn rust_main() -> ! {
    uart::uart_init();
    println!("Hello World!");
    loop {}
}
