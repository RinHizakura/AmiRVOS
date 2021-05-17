.section .rodata
.global KERNEL_SIZE
KERNEL_SIZE: .dword _kernel_size

.section .rodata
.global HEAP_START
HEAP_START: .dword _heap_start
