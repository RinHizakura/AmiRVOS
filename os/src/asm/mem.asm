.section .rodata
.global KERNEL_START
KERNEL_START: .dword _kernel_start

.section .rodata
.global KERNEL_END
KERNEL_END: .dword _kernel_end

.section .rodata
.global TEXT_START
TEXT_START: .dword _text_start

.section .rodata
.global RODATA_START
RODATA_START: .dword _rodata_start

.section .rodata
.global DATA_START
DATA_START: .dword _data_start

.section .rodata
.global BSS_START
BSS_START: .dword _bss_start

.section .rodata
.global TRAMPOLINE_START
TRAMPOLINE_START: .dword _trampoline_start
