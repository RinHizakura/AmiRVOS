use crate::virtio::blk::disk_rw;
use fs::BLKSZ;

pub fn bread(block_no: usize) -> [u8; BLKSZ] {
    let buf: [u8; BLKSZ] = [0; BLKSZ];
    disk_rw(&buf, block_no * BLKSZ, false);
    buf
}
