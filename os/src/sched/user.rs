/* FIXME: All the content in this should be implement elsewhere and
 * put on disk image. Now we just embed it in kernel image for simply
 * testing. */

use core::ffi::{c_int, c_short};

const O_RDWR: c_int = 0x002;

#[link_section = ".text.user.main"]
pub extern "C" fn userinit() {
    extern "C" {
        fn open(path: *const u8, flag: c_int) -> c_int;
        fn mknod(path: *const u8, mode: c_short, dev: c_short) -> c_int;
        fn write();
    }

    /* FIXME: Intentionally put the c-string on stack for the
     * page table walk can be right */
    let path = [b'c', b'o', b'n', b's', b'o', b'l', b'e', 0];
    unsafe {
        // Open a file for stdin
        if open(path.as_ptr(), O_RDWR) < 0 {
            mknod(path.as_ptr(), 0, 0);
        }
        loop {}
    }
}
