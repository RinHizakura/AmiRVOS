.section .rodata
.global KERNEL_START
KERNEL_START: .dword _kernel_start

.global KERNEL_END
KERNEL_END: .dword _kernel_end

.global TEXT_START
TEXT_START: .dword _text_start

.global RODATA_START
RODATA_START: .dword _rodata_start

.global DATA_START
DATA_START: .dword _data_start

.global BSS_START
BSS_START: .dword _bss_start

.global TRAMPOLINE_START
TRAMPOLINE_START: .dword _trampoline_start

.global TRAP_STACK_END
TRAP_STACK_END: .dword _trap_stack_end
