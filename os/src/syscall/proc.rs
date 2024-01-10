use core::ffi::c_int;
use core::result;

use crate::fs::{MAXPATH, O_CREATE};
use crate::sched;
use crate::syscall::syscall_args;

/* The private function is used by syscall handler to access
 * the current process's memory space for nul-terminated string. */
fn fetchstr(addr: usize, buf: &mut [u8]) -> Option<usize> {
    let cur = sched::current();
    let mm = unsafe { (*cur).mm() };
    let result = mm.copy_from_user(addr, buf);
    assert!(result);

    println!("{:?}", buf);

    buf.iter().position(|&w| w == 0)
}

pub fn sys_open() -> c_int {
    let path_addr = syscall_args(0) as usize;
    let flag = syscall_args(1) as c_int;

    let mut path = [0; MAXPATH];
    let n = fetchstr(path_addr, &mut path);

    todo!("sys_open");
}

pub fn sys_write() -> isize {
    let fd = syscall_args(0) as c_int;
    let buf = syscall_args(1);
    let count = syscall_args(2);

    // TODO: Support the syscall correctly

    return 0;
}

pub fn sys_mknod() -> c_int {
    todo!("mknod()");

    return 0;
}
