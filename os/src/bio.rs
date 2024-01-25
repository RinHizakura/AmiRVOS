use crate::virtio::blk::disk_rw;
use fs::BLKSZ;

pub fn bread(block_no: u32) -> [u8; BLKSZ] {
    let buf: [u8; BLKSZ] = [0; BLKSZ];
    disk_rw(&buf, block_no as usize * BLKSZ, false);
    buf
}

pub fn bwrite(block_no: u32, buf: &[u8; BLKSZ]) {
    disk_rw(buf, block_no as usize * BLKSZ, true);
}
