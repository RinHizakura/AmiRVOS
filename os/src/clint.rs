/* The frequency given by QEMU is 10_000_000 Hz, thus
 * the interval 2_500_000 means that we tick every 250ms
 * to switch our task. */
const INTERVAL: usize = 2_500_000;

mmap_reg!(mtimecmp, 0x200_0000 + 0x4000, usize);
mmap_reg!(mtime, 0x200_0000 + 0xbff8, usize);

pub fn set_next_tick() {
    mtimecmp::write(mtime::read() + INTERVAL);
}
