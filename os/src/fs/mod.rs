use core::ffi::c_int;

use fs::{Inode, ROOTINO};

// Maximum length for the name of file
pub const MAXPATH: usize = 128;

pub const O_RDONLY: c_int = 0x000;
pub const O_WRONLY: c_int = 0x001;
pub const O_RDWR: c_int = 0x002;
pub const O_CREATE: c_int = 0x200;
pub const O_TRUNC: c_int = 0x400;

// Get parent directory and file path from a path
fn parse_path<'a>(path: &'a str) -> Option<(&'a str, &'a str)> {
    if path.len() == 0 {
        return None;
    }
    let path = path.trim();
    if let Some(result) = path.split_once('/') {
        Some(result)
    } else {
        Some((path, ""))
    }
}

// Find the corresponding inode by inode number
fn find_inode(inum: u32) -> Inode {
    todo!("find_inode()");
}

// Find the corresponding inode by the path
pub fn path_to_inode(mut path: &str) -> Inode {
    /* FIXME: We only support to use the absolute path which
     * starting from root now. Allow relative path in the future. */
    assert!(path.chars().nth(0) == Some('/'));

    let inode = find_inode(ROOTINO);

    while let Some((parent, file_path)) = parse_path(path) {
        println!("parent {} file {}", parent, file_path);
        path = file_path;
    }

    todo!("namei()");
}
