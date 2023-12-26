/* The design of the filesystem can be referenced to
 * https://github.com/mit-pdos/xv6-riscv/blob/riscv/kernel/fs.h */
#![no_std]

use core::mem::size_of;

// block size
pub const BLKSZ: usize = 1024;
// max number of blocks any FS op writes
pub const MAXOPBLOCKS: usize = 10;

pub const INODES_PER_BLK: usize = BLKSZ / size_of::<Inode>();
pub const NINODES: u32 = 200;

// size of file system in blocks
pub const FS_BLKSZ: u32 = 2000;
// size of log in blocks
pub const LOG_BLKSZ: u32 = MAXOPBLOCKS as u32 * 3;
// size of inode in blocks
pub const INODE_BLKSZ: u32 = NINODES.div_ceil(INODES_PER_BLK as u32);
// size of bitmap in blocks
pub const BITMAP_BLKSZ: u32 = FS_BLKSZ.div_ceil(BLKSZ as u32 * 8);

pub const MAGIC: u32 = 0x52696B6F;

#[repr(C)]
pub struct SuperBlock {
    pub magic: u32,
    // Size of file system image (blocks)
    pub fs_blksz: u32,
    // Number of data blocks
    pub nblocks: u32,
    // Number of inodes
    pub ninodes: u32,
    // Number of log blocks
    pub nlog: u32,
    // Block number of first log block
    pub logstart: u32,
    // Block number of first inode block
    pub inodestart: u32,
    // Block number of first free map block
    pub bmapstart: u32,
}
unsafe impl plain::Plain for SuperBlock {}

pub const NDIRECT: usize = 12;
#[repr(C)]
pub struct Inode {
    // File type
    pub typ: u16,
    // Major device number
    pub major: u16,
    // Minor device number
    pub minor: u16,
    // Number of links to inode in file system
    pub nlink: u16,
    // Size of file (bytes)
    pub size: u32,
    // Data block addresses
    pub addrs: [u32; NDIRECT + 1],
}
unsafe impl plain::Plain for Inode {}
