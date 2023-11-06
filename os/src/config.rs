// 4KB page
pub const PAGE_SIZE: usize = 1 << 12;

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
// CLINT start from 0x2000000
pub const CLINT_BASE: usize = 0x200_0000;
// 64K mapping region for CLINT
pub const CLINT_SIZE: usize = 0x10000;
// PLIC start from 0xc000000
pub const PLIC_BASE: usize = 0xc00_0000;
// mapping region for PLIC
pub const PLIC_SIZE: usize = 0x21_6000;

/* Memory layout for user space */
// All task at user space will start at a fixed virtual address
pub const TASK_START_ADDR: usize = 0x1000;
// All stack at user space will start at a fixed virtual address
pub const STACK_TOP_ADDR: usize = 0xa000_0000;
// The start virtual address for kernel
pub const KERNEL_START_VA: usize = 0xc000_0000;
// Both kernel and user space map trampoline in the same address to handle trap
pub const TRAMPOLINE_VA: usize = MAX_VA - PAGE_SIZE;
// The maximum virtual address for sv39-based memory mapping
pub const MAX_VA: usize = (1 << 39) - 1;
