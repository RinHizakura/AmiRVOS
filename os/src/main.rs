#![no_std] // Disables all standard library
#![no_main] // Disables emitting the main symbol
#![feature(asm, global_asm, panic_info_message)]

global_asm!(include_str!("asm/entry.asm"));
global_asm!(include_str!("asm/mem.asm"));

#[macro_use]
mod console;

mod page;
mod panic;
mod uart;

#[no_mangle] // Disables Rust to change the symbol name
pub extern "C" fn rust_main() -> ! {
    uart::init();
    page::init();
    println!("Hello World!");

    // test the page allocation behavior
    let a = page::alloc(0);
    let b = page::alloc(0);
    let c = page::alloc(0);
    println!("a {:X} b {:X} c {:X}", a as usize, b as usize, c as usize);

    page::free(a);
    let d = page::alloc(1);
    page::free(b);
    let e = page::alloc(1);
    println!("d {:X} e {:X} ", d as usize, e as usize);

    loop {}
}
