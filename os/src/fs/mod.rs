use core::ffi::c_int;
use core::mem::{size_of, MaybeUninit};
use core::ptr;
use core::str::from_utf8;

use crate::bio::*;
use crate::lock::Locked;
use crate::utils::cast::*;

use alloc::vec;
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

/* The containter of Inode. It includes not only
 * the inode instance but also other information
 * of the inode itself, which will be useful to
 * operate the filesystem. */
pub struct FsInode {
    inner: Inode,
    inum: u32,
}

pub fn init() {
    // Block 1 is where the SuperBlock located at
    let mut buf = vec![0; BLKSZ];
    bread(1, &mut buf);

    *SB.lock() = *to_struct::<SuperBlock>(&buf);

    println!("nb = {:x}", SB.lock().nblocks);
}

// Seperate the first path entry from the path string
fn parse_first_path<'a>(path: &'a str) -> Option<(&'a str, &'a str)> {
    // TODO: The implementation should be fixed to meet the expected result
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
pub fn find_inode(inum: u32) -> FsInode {
    let mut inodes = vec![0; BLKSZ];
    bread(iblock(&SB.lock(), inum), &inodes);

    /* TODO: Optimize by implementing cache for Inode, so we don't need to
     * traverse for the result every time. */
    FsInode {
        inner: *block_inode(&mut inodes, inum),
        inum,
    }
}

pub fn update_inode(inode: &FsInode) {
    let mut inodes = vec![0; BLKSZ];
    let inum = inode.inum;
    bread(iblock(&SB.lock(), inum), &inodes);

    *block_inode(&mut inodes, inum) = inode.inner;

    bwrite(iblock(&SB.lock(), inum), &inodes);
}

pub fn alloc_inode(typ: u16, major: u16, minor: u16, nlink: u16) -> u32 {
    let mut inodes = vec![0; BLKSZ];

    /* Linear checking every inode in every inode block for the
     * inode that is marked as non-allocated. */
    for iblock_no in 0..INODE_BLKSZ {
        let iblock = SB.lock().inodestart + iblock_no;
        bread(iblock, &mut inodes);

        for i in 0..(INODES_PER_BLK as u32) {
            let inum = 1 + i + iblock_no * INODES_PER_BLK as u32;
            let inode_ptr = block_inode(&mut inodes, inum);

            if inode_ptr.is_free() {
                inode_ptr.init(typ, major, minor, nlink);
                bwrite(iblock, &inodes);
                return inum;
            }
        }
    }

    panic!("alloc_inode() fail: no empty inode");
}

pub fn free_inode(mut fsinode: FsInode) {
    fsinode.inner.set_free();
    update_inode(&fsinode);
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
    let block_no = alloc_block();
    let block_off = off / BLKSZ;

    if block_off < NDIRECT {
        inode.directs[block_off] = block_no;
    } else {
        todo!("find_or_alloc_block() indirect");
    }

    block_no
}

fn alloc_block() -> u32 {
    let mut bitmap = vec![0; BLKSZ];

    /* Linear checking every bit in every bitmap block for the
     * block that is marked as non-allocated. */
    for bmap_no in 0..BITMAP_BLKSZ {
        let bmap_block = SB.lock().bmapstart + bmap_no;
        bread(bmap_block, &mut bitmap);

        for bit in 0..(BIT_PER_BLK as u32) {
            let bytes = bit as usize / 8;
            let mask = 1 << (bit % 8);
            if bitmap[bytes] & mask == 0 {
                bitmap[bytes] |= mask;
                bwrite(bmap_block, &bitmap);
                return bmap_no * BIT_PER_BLK as u32 + bit;
            }
        }
    }

    panic!("alloc_block() fail: no empty block");
}

// Read data from Inode
fn readi<T>(fsinode: &FsInode, mut off: usize, dst: &mut T) -> bool {
    let inode = &fsinode.inner;

    if off > inode.size as usize {
        return false;
    }

    let mut total = 0;
    let size = size_of::<T>();
    let size = size.min(inode.size as usize - off);
    let buf = vec![0; BLKSZ];

    while total < size {
        let block_num = find_block(inode, off);
        assert!(block_num != 0);

        bread(block_num, &buf);
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

// Write data to Inode
fn writei<T>(fsinode: &mut FsInode, mut off: usize, src: &T) -> bool {
    let inode = &mut fsinode.inner;
    let size = size_of::<T>();

    /* The off should only < size to override data in inode,
     * or = size to append data in inode */
    if off > inode.size as usize {
        return false;
    }

    if (off + size) > (FILE_MAX_LINK * BLKSZ) {
        return false;
    }

    let mut total = 0;
    let mut buf = vec![0; BLKSZ];

    while total < size {
        let block_num = find_block(inode, off);
        assert!(block_num != 0);

        bread(block_num, &buf);
        let n = (size - total).min(BLKSZ - off % BLKSZ);

        // FIXME: Is it possible to make this safe?
        unsafe {
            let mut src_ptr = src as *const T as *const u8;
            src_ptr = src_ptr.add(total);
            let mut dst_ptr = buf.as_mut_ptr();
            dst_ptr = dst_ptr.add(off % BLKSZ);
            ptr::copy_nonoverlapping(src_ptr, dst_ptr, n);
            // Write back
            bwrite(block_num, &buf);
        }

        total += n;
        off += n;
    }

    if off > inode.size as usize {
        inode.size = off as u32;
    }

    /* For simplicity, here we force to write back the inode to
     * disk without checking if it get modified. Note that find_block()
     * may also change the inode content. */
    update_inode(fsinode);

    true
}

// Find the directory's Inode and its number under current Inode
pub fn dirlookup(fsinode: &FsInode, name: &str) -> Option<(FsInode, u32)> {
    let inode = &fsinode.inner;
    assert!(inode.typ == T_DIR);

    for off in (0..inode.size as usize).step_by(size_of::<Dirent>()) {
        let mut dirent: Dirent = unsafe { MaybeUninit::zeroed().assume_init() };
        if !readi(fsinode, off, &mut dirent) {
            return None;
        }

        // This dirent contain nothing
        if dirent.inum == 0 {
            continue;
        }

        let s = from_utf8(&dirent.name).expect("from_utf8(dirent.name)");
        if name == s {
            todo!("dirlookup match");
        }
    }

    None
}

pub fn dirlink(fsinode: &mut FsInode, name: &str, inum: u32) -> bool {
    if dirlookup(fsinode, name).is_none() {
        return false;
    }

    let mut off = 0;
    let mut dirent: Dirent = unsafe { MaybeUninit::zeroed().assume_init() };
    while off < fsinode.inner.size as usize {
        if !readi(fsinode, off, &mut dirent) {
            panic!("dirlink() get dirent fail");
        }

        /* If there is an empty dirent, just making link with it.
         * Otherwise we will extend the inode size(by writei)to append
         * a new entry */
        if dirent.inum == 0 {
            break;
        }

        off += size_of::<Dirent>();
    }

    dirent.update(inum, name);

    if !writei(fsinode, off, &dirent) {
        return false;
    }

    true
}

// Find the corresponding inode by the path
pub fn path_to_inode(mut path: &str) -> Option<(FsInode, u32)> {
    /* FIXME: We only support to use the absolute path which
     * starting from root now. Allow relative path in the future. */
    let mut inode;
    let mut inum;
    if path.chars().nth(0) == Some('/') {
        path = &path[1..];
        inum = ROOTINO;
        inode = find_inode(inum);
    } else {
        todo!("path_to_inode() not start from node");
    }

    while let Some((first, path)) = parse_first_path(path) {
        println!("{} : {}", first, path);
        /* This is not a directory, but the path requires an
         * undering file. This is not a valid path. */
        if inode.inner.typ != T_DIR && path != "" {
            return None;
        }

        let next = dirlookup(&inode, first);
        if let Some(next) = next {
            (inode, inum) = next;
        }
        return None;
    }

    Some((inode, inum))
}
