/* target architecture */
OUTPUT_ARCH(riscv)
/* The first instruction(symbol) to execute in a program */
ENTRY(_start)

SECTIONS
{
    /* '.' means the location counter, starting from 0x80000000 */
    . = 0x80000000;
    _kernel_start = .;

    .text : {
        *(.text.boot)
        *(.text .text.*)
    }

    .rodata : {
        *(.rodata .rodata.*)
    }

    . = ALIGN(16);
    .data : {
        *(.data .data.*)
    }

    _bss_start = .;
    .bss : {
        *(.sbss .bss .bss.*)
    }
    _bss_end = .;
    /* 64KB in .bss section is reserved for stack */
    _stack = _bss_end + 0x10000;
    . = ALIGN(0x00001000);
    _kernel_end = .;
}

_kernel_size = _kernel_end - _kernel_start;
