extern "C" {
    static KERNEL_SIZE: usize;
}

// assuming that we have at least 128M RAM to be used
const DRAM_SIZE: usize = 0x8000000;
// 4KB page
const PAGE_SIZE: usize = 1 << 12;

pub fn init() {
    unsafe {
        println!(
            "num page = ({} - {}) / {}",
            DRAM_SIZE, KERNEL_SIZE, PAGE_SIZE
        );
    }
}
