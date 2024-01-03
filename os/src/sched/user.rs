/* FIXME: All the content in this should be implement elsewhere and
 * put on disk image. Now we just embed it in kernel image for simply
 * testing. */

const O_RDWR: u32 = 0x002;

#[link_section = ".text.user.main"]
pub extern "C" fn userinit() {
    extern "C" {
        fn open(path: *const u8, flag: u32);
        fn write();
    }

    unsafe {
        // Open a file for stdin
        open("console".as_ptr(), O_RDWR);
        loop {}
    }
}
