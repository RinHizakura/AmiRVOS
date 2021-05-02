/* target architecture */
OUTPUT_ARCH(riscv)
/* The first instruction(symbol) to execute in a program */
ENTRY(_start)

SECTIONS
{
    /* '.' means the location counter, starting from 0x80000000 */
    . = 0x80000000;
    .text : {
        *(.text.boot)
        *(.text .text.*)
    }

    .rodata : {
        *(.rodata .rodata.*)
    }

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
}
