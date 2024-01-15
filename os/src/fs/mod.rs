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

fn balloc() -> u32 {
    /* Linear checking every bit in every bitmap block for the
     * block that is marked as non-allocated. */
    for bmap_no in 0..BITMAP_BLKSZ {
        let bmap_block = SB.lock().bmapstart + bmap_no;
        let mut bitmap = bread(bmap_block);

        for bit in 0..(BIT_PER_BLK as u32) {
            let bytes = bit as usize / 8;
            let mask = 1 << (bit % 8);
            if bitmap[bytes] & mask == 0 {
                bitmap[bytes] |= mask;
                bwrite(bmap_block, bitmap);
                return bmap_no * BIT_PER_BLK as u32 + bit;
            }
        }
    }

    return 0;
}

// Get the block number for the request data offset
fn find_block(inode: &Inode, off: usize) -> u32 {
    let block_off = off / BLKSZ;

    // For the first NDIRECT blocks, they are direct linked
    let block_no = if block_off < NDIRECT {
        inode.directs[block_off]
    } else {
        todo!("find_block() indirect");
    };

    block_no
}

// Get the block number for the request data offset
fn find_or_alloc_block(inode: &mut Inode, off: usize) -> u32 {
    let block_no = find_block(inode, off);
    if block_no != 0 {
        return block_no;
    }

    /* If there is no corresponding block on this link, allocating
     * one for it. */
    let block_no = balloc();
    let block_off = off / BLKSZ;

    if block_off < NDIRECT {
        inode.directs[block_off] = block_no;
    } else {
        todo!("find_or_alloc_block() indirect");
    }

    block_no
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
        let block_num = find_block(inode, off);
        assert!(block_num != 0);

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
