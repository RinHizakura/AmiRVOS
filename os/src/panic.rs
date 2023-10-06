use core::panic::PanicInfo;

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    if let Some(s) = info.payload().downcast_ref::<&str>() {
        println!("\x1b[1;31mpanic: '{}'\x1b[0m", s);
    } else {
        println!("\x1b[1;31mpanic: '{}'\x1b[0m", info.message().unwrap());
    }
    loop {}
}
