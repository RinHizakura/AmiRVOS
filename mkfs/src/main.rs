use std::cell::{Cell, RefCell};
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::mem::{size_of, MaybeUninit};
use std::slice;

use fs::*;

thread_local! {
    static FSFILE: RefCell<File> =
                    RefCell::new(unsafe { MaybeUninit::zeroed().assume_init() });
    static ALLOC_BLOCK: Cell<u32> =
                    Cell::new(unsafe { MaybeUninit::zeroed().assume_init() });
    // NOTE: This is a 1-based value
    static NEXT_INUM: Cell<u32> = Cell::new(1);
}

fn as_slice<T: Sized>(p: &T) -> &[u8] {
    unsafe { slice::from_raw_parts((p as *const T) as *const u8, size_of::<T>()) }
}

fn wsect(sec: u32, buf: &[u8]) {
    let off = FSFILE.with(|f| {
        f.borrow_mut()
            .seek(SeekFrom::Start(sec as u64 * BLKSZ as u64))
            .expect("seek for wsect fail")
    });
    assert!(off == sec as u64 * BLKSZ as u64);

    let size = FSFILE.with(|f| f.borrow_mut().write(buf).expect("write for wsect fail"));
    assert!(size == BLKSZ);
}

fn rsect(sec: u32, buf: &mut [u8]) {
    let off = FSFILE.with(|f| {
        f.borrow_mut()
            .seek(SeekFrom::Start(sec as u64 * BLKSZ as u64))
            .expect("seek for rsect fail")
    });
    assert!(off == sec as u64 * BLKSZ as u64);

    let size = FSFILE.with(|f| f.borrow_mut().read(buf).expect("read for rsect fail"));
    assert!(size == BLKSZ);
}

fn rinode(sb: &SuperBlock, inum: u32) -> Inode {
    let mut buf = [0; BLKSZ];

    let block = iblock(sb, inum);
    rsect(block, &mut buf);

    assert!(inum > 0);

    let inode_ptr = block_inode(&mut buf, inum);
    *inode_ptr
}

fn winode(sb: &SuperBlock, inum: u32, inode: Inode) {
    let mut buf = [0; BLKSZ];

    // Read, write, and modify
    let block = iblock(sb, inum);
    rsect(block, &mut buf);

    assert!(inum > 0);

    let inode_ptr = block_inode(&mut buf, inum);
    *inode_ptr = inode;
    wsect(block, &buf);
}

fn alloc_inode(sb: &SuperBlock, typ: u16) -> u32 {
    let inum = NEXT_INUM.get();
    NEXT_INUM.set(inum + 1);

    let inode = Inode {
        typ: typ,
        major: 0,
        minor: 0,
        nlink: 1,
        size: 0,
        directs: [0; NDIRECT],
        indirect: 0,
    };

    winode(sb, inum, inode);
    inum
}

fn iappend(sb: &SuperBlock, inum: u32, data: &[u8]) {
    /* Append new contents to the file described by this inode */
    let mut buf = [0; BLKSZ];
    let len = data.len();
    let mut inode = rinode(sb, inum);

    /* The new data will append from the end of file */
    let end = inode.size as usize;
    let mut off = 0;

    while off < len {
        let nlink = (end + off) / BLKSZ;
        assert!(nlink < FILE_MAX_LINK);

        let mut block_num;
        if nlink < NDIRECT {
            /* The first NDIRECT links are directly linked */
            block_num = inode.directs[nlink];

            if block_num == 0 {
                block_num = ALLOC_BLOCK.get();
                inode.directs[nlink] = block_num;
                ALLOC_BLOCK.set(block_num + 1);
            }
        } else {
            // TODO: Support larger file size which requires indirect linking
            todo!();
        }

        let n = len.min((nlink + 1) * BLKSZ - (end + off));
        rsect(block_num, &mut buf);
        let buf_start = (end + off) - nlink * BLKSZ;
        buf[buf_start..buf_start + n].copy_from_slice(&data[off..off + n]);
        wsect(block_num, &buf);

        off += n;
    }

    inode.size += len as u32;
    winode(sb, inum, inode);
}

fn create_dent(sb: &SuperBlock, inum: u32, name: &str) {
    let mut dirent: Dirent = unsafe { MaybeUninit::zeroed().assume_init() };
    dirent.update(inum, name);
    iappend(&sb, inum, as_slice(&dirent));
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let img_name = &args[1];

    let mut buf = [0; BLKSZ];
    FSFILE.set(
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(img_name)
            .expect("create()"),
    );

    for i in 0..FS_BLKSZ {
        wsect(i as u32, &[0; BLKSZ]);
    }

    /* In this design, we have the following metadata. Sequentially from
     * the first block:
     * - 1 block for boot block
     * - 1 block for suberblock
     * - LOG_BLKSZ count for log block
     * - INODE_BLKSZ count for inode block
     * - BITMAP_BLKSZ count for bitmap block */
    assert!(BLKSZ % size_of::<Inode>() == 0);
    let nmeta = 1 + 1 + LOG_BLKSZ + INODE_BLKSZ + BITMAP_BLKSZ;
    // The remaining blocks are used for data block
    let nblocks = FS_BLKSZ - nmeta;

    // There are nmeta blocks allocated currently
    ALLOC_BLOCK.set(nmeta as u32);

    println!(
        "Total {} = 1 boot + 1 superblock + {} log + {} inode + {} bitmap + {} data",
        FS_BLKSZ, LOG_BLKSZ, INODE_BLKSZ, BITMAP_BLKSZ, nblocks
    );

    let sb = SuperBlock {
        magic: MAGIC,
        fs_blksz: FS_BLKSZ,
        nblocks: nblocks,
        ninodes: NINODES,
        nlog: LOG_BLKSZ,
        logstart: 2,
        inodestart: 2 + LOG_BLKSZ,
        bmapstart: 2 + LOG_BLKSZ + INODE_BLKSZ,
    };
    buf[0..size_of::<SuperBlock>()].copy_from_slice(as_slice(&sb));
    wsect(1, &mut buf);

    /* Create inode for the root directory */
    let rootino = alloc_inode(&sb, T_DIR);
    assert!(rootino == ROOTINO);

    /* Create root directory entry that the root inode refers to */
    create_dent(&sb, rootino, ".");

    /* Create parent directory entry */
    create_dent(&sb, rootino, "..");

    /* TODO: Append file under root directory(if any) */

    /* Update bitmap for the allocated and unallocated blocks */
    let total_used = ALLOC_BLOCK.get();
    let mut buf = [0; BLKSZ];
    /* We simply assume that the bitmap in the first block is enough for
     * all the used data. */
    assert!((total_used as usize) < BLKSZ * 8);
    for i in 0..total_used {
        buf[i as usize / 8] |= 1 << (i % 8);
    }
    wsect(sb.bmapstart, &buf);
}
