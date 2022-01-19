use riscv::register::{sie, sstatus};

const INTERVAL: usize = 100000;

mmap_reg!(mtimecmp, 0x200_0000 + 0x4000, usize);
mmap_reg!(mtime, 0x200_0000 + 0xbff8, usize);

fn set_next_tick() {
    mtimecmp::write(mtime::read() + INTERVAL);
}

pub fn init() {
    unsafe {
        sie::set_stimer();
        sstatus::set_sie();
    }
    set_next_tick();
}
