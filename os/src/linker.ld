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

        . = ALIGN(0x1000);
        _trampoline_start = .;
        *(.text.trampoline)
        . = ALIGN(0x1000);
        ASSERT(. - _trampoline_start == 0x1000, "error: trampoline larger than one page");

        . = ALIGN(0x1000);
        _user_start = .;
        *(.text.user.main)
        *(.text.user*)
        . = ALIGN(0x1000);
        ASSERT(. - _user_start == 0x1000, "error: user larger than one page");

        *(.text .text.*)
    }

    . = ALIGN(4K);
    _rodata_start = .;
    .rodata : {
        *(.rodata .rodata.*)
    }

    . = ALIGN(4K);
    _data_start = .;
    /* TODO: Only the variable ranged in gp +-2048 can be relaxed, thus put
     * it on the middle of the first 4K. We should consider to put it at
     * the most intensely used memory area. */
    __global_pointer$ = . + 4K / 2;
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
    /* 4KB is reserved for machine timer trap stack, is this enough? */
    _mtrap_stack_start = .;
    _mtrap_stack_end = _mtrap_stack_start + 0x1000;
    /* 64 KB is reserved for kernel stack */
    _stack_start = .;
    _stack_end = _stack_start + 0x10000;

    . = ALIGN(4K);
    _kernel_end = .;
}

