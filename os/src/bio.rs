use crate::virtio::blk::disk_rw;
use fs::BLKSZ;

pub fn bread(block_no: u32, buf: &[u8]) {
    assert!(buf.len() == BLKSZ);
    disk_rw(buf, block_no as usize * BLKSZ, false);
}

pub fn bwrite(block_no: u32, buf: &[u8]) {
    assert!(buf.len() == BLKSZ);
    disk_rw(buf, block_no as usize * BLKSZ, true);
}
