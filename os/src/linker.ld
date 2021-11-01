/* target architecture */
OUTPUT_ARCH(riscv)
/* The first instruction(symbol) to execute in a program */
ENTRY(_start)

SECTIONS
{
    /* '.' means the location counter */
    . = 0x80000000;
    _kernel_start = .;

    _text_start = .;
    .text : {
        *(.text.boot)
        *(.text .text.*)
    }
 
    . = ALIGN(4K);
    _rodata_start = .;
    .rodata : {
        *(.rodata .rodata.*)
    }

    . = ALIGN(4K);
    _data_start = .;
    .data : {
        *(.data .data.*)
    }

    . = ALIGN(4K);
    _bss_start = .;
    .bss : {
        *(.sbss .bss .bss.*)
    }
    _bss_end = .;

    . = ALIGN(4K);
    /* 64KB is reserved for stack */
    _stack_start = .;
    _stack_end = _stack_start + 0x10000;

    . = ALIGN(4K);
    _kernel_end = .;
}
