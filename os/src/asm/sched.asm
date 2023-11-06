.global switch_to
switch_to:
    # a0 - address of TrapFrame for current task

    # Use TrapFrame of current task when we switch to it
    csrw    mscratch, a0

    # Load register values from TrapFrame
    # 1. program counter
    ld      a1, 264(a0)
    csrw    mepc, a1
    # 2. satp
    ld      a2, 256(a0)
    csrw    satp, a2
    sfence.vma

    # Restore the content from TrapFrame
    csrr    t6, mscratch
    .set    i, 1
    .rept    31
        load_gp     %i, t6
        .set        i, i+1
    .endr

    mret
