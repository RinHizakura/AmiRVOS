.global switch_to
switch_to:
    # a0 - address of TrapFrame for current task
    # a1 - satp for current task
    # a2 - mode bit for current task

    # Use TrapFrame of current task when we switch to it
    csrw    mscratch, a0
    csrw    sscratch, a0

    # Load program counter from TrapFrame
    ld      t0, 280(a0)
    csrw    mepc, t0
    # Load satp
    csrw    satp, a1
    sfence.vma
    # Set priviledge mode according to the mode bit
    # TODO: Could we write this more simpler?
    csrr    t0, mstatus
    li      t1, (0b11 << 11)
    not     t1, t1
    and     t0, t0, t1
    slli    t1, a2, 11
    or      t0, t0, t1
    csrw    mstatus, t0

    # Restore the content from TrapFrame
    csrr    t6, mscratch
    .set    i, 1
    .rept    31
        load_gp     %i, t6
        .set        i, i+1
    .endr

    mret
