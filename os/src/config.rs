// DRAM start from 0x80000000
pub const DRAM_BASE: usize = 0x8000_0000;
// Assuming that we have at least 128M RAM to be used
pub const DRAM_SIZE: usize = 0x800_0000;
// The bottom memory address of HEAP to be accessed
// - Warning: we naively assume our kernel size won't exceed 2 MB, this
//   assumption help our code become easier to be understand
pub const LOW_MEMORY: usize = DRAM_BASE + 0x20_0000;
// The top memory address of HEAP to be accessed
pub const HIGH_MEMORY: usize = DRAM_BASE + DRAM_SIZE;
// 1 MB size will be reserved in bss section for kernel heap
pub const KERNEL_HEAP_SIZE: usize = 0x10_0000;
// UART start from 0x10000000
pub const UART_BASE: usize = 0x1000_0000;
