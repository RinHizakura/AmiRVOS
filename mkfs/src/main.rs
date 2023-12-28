#![feature(const_maybe_uninit_zeroed)]

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
    static NEXT_INUM: Cell<u32> = Cell::new(1);
}

pub fn to_struct<T: plain::Plain>(args: &mut [u8]) -> &mut T {
    let size = size_of::<T>();
    let slice = &mut args[0..size];
    return plain::from_mut_bytes::<T>(slice).expect("Fail to cast bytes to Args");
}

fn as_slice<T: Sized>(p: &T) -> &[u8] {
    unsafe { slice::from_raw_parts((p as *const T) as *const u8, size_of::<T>()) }
}

fn wsect(sec: u64, buf: &[u8]) {
    let off = FSFILE.with(|f| {
        f.borrow_mut()
            .seek(SeekFrom::Start(sec * BLKSZ as u64))
            .expect("seek for wsect fail")
    });
    assert!(off as u64 == sec * BLKSZ as u64);

    let size = FSFILE.with(|f| f.borrow_mut().write(buf).expect("write for wsect fail"));
    assert!(size == BLKSZ);
}

fn rsect(sec: u64, buf: &mut [u8]) {
    let off = FSFILE.with(|f| {
        f.borrow_mut()
            .seek(SeekFrom::Start(sec * BLKSZ as u64))
            .expect("seek for rsect fail")
    });
    assert!(off as u64 == sec * BLKSZ as u64);

    let size = FSFILE.with(|f| f.borrow_mut().read(buf).expect("read for rsect fail"));
    assert!(size == BLKSZ);
}

fn winode(sb: &SuperBlock, inum: u32, inode: Inode) {
    let mut buf = [0; BLKSZ];

    // Read, write, and modify
    let block = iblock(sb, inum);
    rsect(block as u64, &mut buf);

    assert!(inum > 0);
    let start = ((inum - 1) as usize % INODES_PER_BLK) * size_of::<Inode>();
    let end = start + size_of::<Inode>();
    let inode_ptr = to_struct::<Inode>(&mut buf[start..end]);

    *inode_ptr = inode;
    wsect(block as u64, &buf);
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
        addrs: [0; NDIRECT + 1],
    };

    winode(sb, inum, inode);
    inum
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
        wsect(i as u64, &[0; BLKSZ]);
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
}
