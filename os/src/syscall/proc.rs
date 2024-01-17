use core::ffi::c_int;
use core::str::from_utf8;

use fs::{Inode, T_DEVICE};

use crate::fs::{parse_last_path, path_to_inode, MAXPATH, O_CREATE};
use crate::sched;
use crate::syscall::syscall_args;
use crate::syscall::types::*;

/* The private function is used by syscall handler to access
 * the current process's memory space for nul-terminated string. */
fn fetchstr(addr: usize, buf: &mut [u8]) -> Option<usize> {
    let cur = sched::current();
    let mm = unsafe { (*cur).mm() };
    let result = mm.copy_from_user(addr, buf);
    assert!(result);

    buf.iter().position(|&w| w == 0)
}

fn create(path: &str, typ: u16, major: u16, minor: u16) -> Option<Inode> {
    let result = parse_last_path(path);
    if result.is_none() {
        return None;
    }

    let (mut path, file) = result.unwrap();
    // TODO: Consider the case when parent is root, can we do this simpler?
    if path == "" {
        path = "/";
    }
    println!("parent = {}, file = {}", path, file);

    let parent_inode = path_to_inode(path);
    if parent_inode.is_none() {
        return None;
    }

    todo!("create")
}

pub fn sys_open() -> c_int {
    let path_addr = syscall_args(0) as usize;
    let flag = syscall_args(1) as c_int;

    let mut path = [0; MAXPATH];
    let _n = fetchstr(path_addr, &mut path);

    if flag & O_CREATE == O_CREATE {
        todo!("sys_open O_CREATE");
    } else {
        let inode = path_to_inode(from_utf8(&path).expect("open path"));
        // The file is not existing
        if inode.is_none() {
            return -1;
        }
    }

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
    let path_addr = syscall_args(0) as usize;
    let mode = syscall_args(1) as mode_t;
    let dev = syscall_args(2) as dev_t;

    let mut path = [0; MAXPATH];
    let _n = fetchstr(path_addr, &mut path);

    let inode = create(
        from_utf8(&path).expect("mknod path"),
        T_DEVICE,
        MAJOR(dev),
        MINOR(dev),
    );

    todo!("mknod()");

    return 0;
}
