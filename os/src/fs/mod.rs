use core::ffi::c_int;
use core::mem::{size_of, MaybeUninit};
use core::ptr;

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
    static ref SB: Locked<SuperBlock> = Locked::new(unsafe { MaybeUninit::zeroed().assume_init() });
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
    let buf = bread(iblock(&SB.lock(), ROOTINO));

    /* TODO: Optimize by implementing cache for Inode, so we don't need to
     * traverse for the result every time. */
    *to_struct::<Inode>(&buf)
}

// Get the block number for the request data offset
fn bmap(inode: &Inode, off: usize) -> u32 {
    let block_off = off / BLKSZ;

    todo!("bmap()");
}

// Read data from Inode
fn readi<T>(inode: &Inode, mut off: usize, dst: &mut T) -> bool {
    if off > inode.size as usize {
        return false;
    }

    let mut total = 0;
    let size = size_of::<T>();
    let size = size.min(inode.size as usize - off);

    while total < size {
        let block_num = bmap(inode, off);
        if block_num == 0 {
            break;
        }

        let buf = bread(block_num);
        let n = (size - total).min(BLKSZ - off % BLKSZ);

        // FIXME: Is it possible to make this safe?
        unsafe {
            let src_ptr = buf.as_ptr().add(off % BLKSZ);
            let mut dst_ptr = dst as *mut T as *mut u8;
            dst_ptr = dst_ptr.add(total);
            ptr::copy_nonoverlapping(src_ptr, dst_ptr, n);
        }

        total += n;
        off += n;
    }

    true
}

// Find the directory's Inode under current Inode
fn dirlookup(inode: &Inode, name: &str) -> Option<Inode> {
    assert!(inode.typ == T_DIR);

    let mut off = 0;
    while off < inode.size as usize {
        let mut dirent: Dirent = unsafe { MaybeUninit::zeroed().assume_init() };
        if !readi(inode, off, &mut dirent) {
            return None;
        }

        todo!("dirlookup()");
    }

    todo!("dirlookup()");
}

// Find the corresponding inode by the path
pub fn path_to_inode(mut path: &str) -> Option<Inode> {
    /* FIXME: We only support to use the absolute path which
     * starting from root now. Allow relative path in the future. */
    assert!(path.chars().nth(0) == Some('/'));

    let mut inode = find_inode(ROOTINO);

    println!("inode {}", inode.size);

    while let Some((parent, file_path)) = parse_path(path) {
        println!("parent {} file {}", parent, file_path);
        /* This parent is not a directory, but the path requires an
         * undering file. This is not a valid path. */
        if inode.typ != T_DIR && file_path != "" {
            return None;
        }

        let next = dirlookup(&inode, parent);
        path = file_path;
    }

    Some(inode)
}
