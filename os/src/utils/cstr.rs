use crate::alloc::string::ToString;
use alloc::string::String;
use alloc::vec::Vec;
use core::str::from_utf8;

pub fn buf2cstr(mut buf: Vec<u8>) -> String {
    let n = buf
        .iter()
        .position(|&w| w == 0)
        .expect("Invalid buffer for cstr");

    /* FIXME: Free useless space after knowing the strlen,
     * but this solution is kind of ugly. */
    buf.truncate(n);

    from_utf8(&buf)
        .expect("from_utf8() error for buf2cstr()")
        .to_string()
}
