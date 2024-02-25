/* The design of the filesystem can be referenced to
 * https://github.com/mit-pdos/xv6-riscv/blob/riscv/kernel/fs.h */
#![no_std]

use core::mem::size_of;

// block size
pub const BLKSZ: usize = 1024;
// max number of blocks any FS op writes
pub const MAXOPBLOCKS: usize = 10;

pub const INODES_PER_BLK: usize = BLKSZ / size_of::<Inode>();
pub const BIT_PER_BLK: usize = BLKSZ * 8;
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
#[derive(Clone, Copy, Debug)]
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
#[derive(Clone, Copy, Debug)]
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
impl Inode {
    pub fn init(&mut self, typ: u16, major: u16, minor: u16, nlink: u16) {
        *self = Inode {
            typ,
            major,
            minor,
            nlink,
            size: 0,
            directs: [0; NDIRECT],
            indirect: 0,
        }
    }

    pub fn set_free(&mut self) {
        // typ == 0 means this is a free inode
        self.typ = 0;
    }

    pub fn is_free(&self) -> bool {
        self.typ == 0
    }
}

pub const DIRSIZ: usize = 14;
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Dirent {
    pub inum: u16,
    pub name: [u8; DIRSIZ],
}

impl Dirent {
    pub fn update(&mut self, inum: u32, s: &str) {
        let slen = s.len();
        // At least the last '\0' should be retained
        assert!(slen < DIRSIZ - 1);

        // TODO: take care of truncation after casting
        self.inum = inum as u16;
        self.name[0..slen].copy_from_slice(s.as_bytes());
        self.name[slen] = 0;
    }
}

// Block containing inode i
pub fn iblock(sb: &SuperBlock, inum: u32) -> u32 {
    assert!(inum != 0);

    // According to the inode number, evaluate the block to place it
    (inum - 1) / INODES_PER_BLK as u32 + sb.inodestart
}

// Get inode i in the block which contains it
pub fn block_inode(inodes: &mut [u8], inum: u32) -> &mut Inode {
    assert!(inum != 0);

    let start = ((inum - 1) % INODES_PER_BLK as u32) as usize * size_of::<Inode>();
    let end = start + size_of::<Inode>();

    plain::from_mut_bytes::<Inode>(&mut inodes[start..end]).expect("Fail to cast bytes to Inode")
}

// The bitmap block for the given block
pub fn block_bmap(sb: &SuperBlock, block_no: u32) -> u32 {
    block_no / BIT_PER_BLK as u32 + sb.bmapstart
}
