use core::ffi::c_int;
use core::str::from_utf8;

use fs::*;

use crate::fs::*;
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

// Seperate the last path entry from the path string
fn path_to_parent_file(path: &str) -> Option<(&str, &str)> {
    // TODO: The implementation should be fixed to meet the expected result
    if path.len() == 0 {
        return None;
    }

    let path = path.trim();
    if let Some((parent, file)) = path.rsplit_once('/') {
        if parent == "" {
            // Consider the case when parent is root
            Some(("/", file))
        } else {
            Some((parent, file))
        }
    } else {
        Some(("", path))
    }
}

fn create(path: &str, typ: u16, major: u16, minor: u16) -> Option<Inode> {
    let (path, file) = path_to_parent_file(path)?;
    println!("parent = {}, file = {}", path, file);

    let (mut parent_inode, parent_inum) = path_to_inode(path)?;
    if let Some((file_inode, file_inum)) = dirlookup(&parent_inode, file) {
        // The inode for the file already exists
        todo!("create() existed file");
    }

    /* Note that nlink of directory don't cosider itself(".").
     * The purpose is to get rid of cyclic ref count */
    let nlink = 1;
    // Create inode for this new file/directory
    let file_inum = alloc_inode(typ, major, minor, nlink);
    let mut file_inode = find_inode(file_inum);

    // Link this new file/directory to its parent directory
    dirlink(&mut parent_inode, path, file_inum);

    /* Link '.' and '..' to this new directory inode. Since parent("..")
     * is linked by this directory, we should update parent's inode */
    if typ == T_DIR {
        dirlink(&mut file_inode, ".", file_inum);
        dirlink(&mut file_inode, "..", parent_inum);
    }

    todo!("create()")
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

    let path = from_utf8(&path).expect("mknod path");
    let inode = create(path, T_DEVICE, MAJOR(dev), MINOR(dev));

    todo!("mknod()");

    return 0;
}
