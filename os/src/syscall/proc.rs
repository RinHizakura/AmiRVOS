use core::ffi::c_int;

use fs::*;

use crate::fs::*;
use crate::sched;
use crate::syscall::syscall_args;
use crate::syscall::types::*;
use crate::utils::cstr::*;

use alloc::string::String;
use alloc::vec;

/* The private function is used by syscall handler to access
 * the current process's memory space for nul-terminated string. */
fn fetchstr(addr: usize) -> String {
    let cur = sched::current();
    let mm = unsafe { (*cur).mm() };
    let mut buf = vec![0; MAXPATH];
    let result = mm.copy_from_user(addr, &mut buf);
    assert!(result);

    buf2cstr(buf)
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

fn create(path: &str, typ: u16, major: u16, minor: u16) -> Option<FsInode> {
    let (parent, file) = path_to_parent_file(path)?;
    dbg!("Create file {} under {}", file, parent);

    let mut parent_inode = path_to_inode(parent)?;
    if let Some(file_inode) = dirlookup(&parent_inode, file) {
        // The inode for the file already exists
        return Some(file_inode);
    }

    /* Note that nlink of directory don't cosider itself(".").
     * The purpose is to get rid of cyclic ref count */
    let nlink = 1;
    // Create inode for this new file/directory
    let file_inum = alloc_inode(typ, major, minor, nlink);
    let mut file_inode = find_inode(file_inum);

    /* Link '.' and '..' to this new directory inode. */
    if typ == T_DIR {
        if !dirlink(&mut file_inode, ".", file_inum)
            || !dirlink(&mut file_inode, "..", parent_inode.inum)
        {
            free_inode(file_inode);
            return None;
        }
    }

    /* Link this new file/directory to its parent directory. Do
     * this after dirlink() the file_inode because it can simplify
     * the error handling flow without rolling back the change
     * on parent inode. */
    if !dirlink(&mut parent_inode, file, file_inum) {
        free_inode(file_inode);
        return None;
    }

    /* Since parent("..") is linked by this directory, we should
     * also update parent inode's nlink */
    if typ == T_DIR {
        parent_inode.inner.nlink += 1;
    }

    Some(file_inode)
}

pub fn sys_open() -> c_int {
    let path_addr = syscall_args(0) as usize;
    let flag = syscall_args(1) as c_int;

    let path = fetchstr(path_addr);

    if flag & O_CREATE == O_CREATE {
        todo!("sys_open O_CREATE");
    } else {
        let inode = path_to_inode(&path);
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
    let _mode = syscall_args(1) as mode_t;
    let dev = syscall_args(2) as dev_t;

    let path = fetchstr(path_addr);

    let _ = create(&path, T_DEVICE, MAJOR(dev), MINOR(dev));

    return 0;
}
