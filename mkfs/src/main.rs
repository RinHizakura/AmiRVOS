use std::env;
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::mem::size_of;
use std::slice;

use fs::*;

fn as_slice<T: Sized>(p: &T) -> &[u8] {
    unsafe { slice::from_raw_parts((p as *const T) as *const u8, size_of::<T>()) }
}

fn wsect(f: &mut File, sec: u64, buf: &[u8]) {
    let off = f
        .seek(SeekFrom::Start(sec * BLKSZ as u64))
        .expect("seek for wsect fail");
    assert!(off as u64 == sec * BLKSZ as u64);

    let size = f.write(buf).expect("write for wsect fail");
    assert!(size == BLKSZ);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let img_name = &args[1];

    let mut buf = [0; BLKSZ];
    let mut f = File::create(img_name).expect("create()");

    for i in 0..FS_BLKSZ {
        wsect(&mut f, i as u64, &[0; BLKSZ]);
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
    wsect(&mut f, 1, &mut buf);
}
