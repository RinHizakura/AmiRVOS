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

// Inode number for root
pub const ROOTINO: u32 = 1;
//
pub const MAGIC: u32 = 0x52696B6F;

#[repr(C)]
#[derive(Clone, Copy)]
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

// Directory type file
pub const T_DIR: u16 = 1;
// Normal File
pub const T_FILE: u16 = 2;
// Device type file
pub const T_DEVICE: u16 = 3;

// NDIRECT blocks in a file are described with direct link
pub const NDIRECT: usize = 12;
/* NINDIRECT blocks in a file are described with indirect link. At
 * most one block is choosed to store the link */
pub const NINDIRECT: usize = BLKSZ / size_of::<u32>();
/* A file's total blocks is not expected to go over the total availible links */
pub const FILE_MAX_LINK: usize = NDIRECT + NINDIRECT;

#[repr(C)]
#[derive(Clone, Copy)]
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
    // Data block addresses for direct access
    pub directs: [u32; NDIRECT],
    // Data block addresses for indirect access
    pub indirect: u32,
}
unsafe impl plain::Plain for Inode {}

pub const DIRSIZ: usize = 14;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Dirent {
    pub inum: u16,
    pub name: [u8; DIRSIZ],
}

// Block containing inode i
pub fn iblock(sb: &SuperBlock, inum: u32) -> u32 {
    // According to the inode number, evaluate the block to place it
    inum / INODES_PER_BLK as u32 + sb.inodestart
}