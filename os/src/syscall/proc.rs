use core::ffi::c_int;

use crate::syscall::syscall_args;

pub fn sys_write() -> usize {
    let fd = syscall_args(0) as c_int;
    let buf = syscall_args(1);
    let count = syscall_args(2);

    // TODO: Support the syscall correctly

    return 0;
}
