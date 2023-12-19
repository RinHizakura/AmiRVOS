.section .text.boot
.global _start
_start:
    # For the hart that id != 0, hanging in a infinite for loop
    csrr    t0, mhartid
    bnez    t0, 3f

    # Initialize global pointer. It is mainly used for linker relaxation.
    # We need to disable linker relaxation(.norelax) here to prevent initialization
    # instruction relaxed into a NOP-like instruction (e.g. mv gp, gp).
    .option push
    .option norelax
    la gp, __global_pointer$
    .option pop

    csrw    satp, zero

    # .bss section is reset to be zero
    la      a0, _bss_start
    la      a1, _bss_end
    bgeu    a0, a1, 2f
1:
    sd      zero, (a0)
    addi    a0, a0, 8
    bltu    a0, a1, 1b
2:
    la      sp, _stack_end

    # (0b11 << 11): the privilege level will be set to 3(machine mode) after mret
    # for bit 7 of mstatus, the machine mode interrupt-enable bit will be 0 after mret
    li      t0, (0b11 << 11)
    csrw    mstatus, t0

    # pc will be set to address of kinit after mret
    la      t1, kinit
    csrw    mepc, t1

    la      ra, 2f
    mret
2:
    # (1 << 11): the privilege level will be set to 1(supervisor mode) after mret
    # (1 << 7): the machine mode interrupt-enable bit will be 1 after mret
    li      t0, (1 << 11) | (1 << 7)
    csrw    mstatus, t0

    # M-mode and S-mode software interrupt, timer interrupt, and external
    # interrupt are enabled
    li      t0, 0xa | (0xa << 4) | (0xa << 8)
    csrw    mie, t0

    # delegate software interrupt, timer interrupt, and external interrupt from
    # M-mode to S-mode
    li      t2, 0xffff
    csrw    mideleg, t2

    # delegate all kind of exceptions from M-mode to S-mode
    li      t2, 0xffff
    csrw    medeleg, t2

    # pc will be set to address of kmain after sret
    # note that MMU will be switched on after sret, so we have to
    # set sepc to the virtual address of kmain
    la      t0, kmain
    csrw    mepc, t0

    # We need to set the PMP entry to start kmain correctly. To be simple at
    # first, we just use single pmp with unrestricted privilege to cover whole
    # memory region. See https://stackoverflow.com/questions/69133848/risc-v-illegal-instruction-exception-when-switching-to-supervisor-mode
    li      t0, 0xffffffffffffffff
    csrw    pmpaddr0, t0
    # (0b11 << 3): NAPOT
    # 0x7: read, write, execute are valid
    li      t0, (0b11 << 3) | 0x7
    csrw    pmpcfg0, t0

    # hanging if return from kmain
    la      ra, 3f
    mret
3:
    wfi
    j       3b
