use core::ffi::c_int;

use crate::bio::*;
use crate::lock::Locked;
use crate::utils::cast::to_struct;

use fs::*;
use lazy_static::lazy_static;

// Maximum length for the name of file
pub const MAXPATH: usize = 128;

pub const O_RDONLY: c_int = 0x000;
pub const O_WRONLY: c_int = 0x001;
pub const O_RDWR: c_int = 0x002;
pub const O_CREATE: c_int = 0x200;
pub const O_TRUNC: c_int = 0x400;

lazy_static! {
    static ref SB: Locked<SuperBlock> = Locked::new(SuperBlock::default());
}
pub fn init() {
    // Block 1 is where the SuperBlock located at
    let buf = bread(1);

    *SB.lock() = *to_struct::<SuperBlock>(&buf);

    println!("nb = {:x}", SB.lock().nblocks);
}

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
    let buf: [u8; 512] = [0; 512];

    /* TODO: Optimize by implementing cache for Inode, so we don't need to
     * traverse for the result every time. */
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
