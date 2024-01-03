use core::ffi::c_int;

use crate::fs::MAXPATH;
use crate::sched;
use crate::syscall::syscall_args;

/* The private function is used by syscall handler to access
 * the current process's memory space for nul-terminated string. */
fn fetchstr(addr: usize, buf: &mut [u8]) -> usize {
    let cur = sched::current();
    let mm = unsafe { (*cur).mm() };
    mm.copy_from_user(addr, buf);

    todo!();
    return 0;
}

pub fn sys_open() -> usize {
    let path_addr = syscall_args(0) as usize;
    let flag = syscall_args(1) as c_int;

    let mut path = [0; MAXPATH];
    fetchstr(path_addr, &mut path);

    todo!();
}

pub fn sys_write() -> usize {
    let fd = syscall_args(0) as c_int;
    let buf = syscall_args(1);
    let count = syscall_args(2);

    // TODO: Support the syscall correctly

    return 0;
}
