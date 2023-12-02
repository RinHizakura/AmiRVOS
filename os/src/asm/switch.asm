.section .text
.global switch_to
switch_to:
    # a0 - address of TrapFrame on current task

    # Register sscratch should contain the TrapFrame of currnet task.
    # Exchange t6(x31) and sscratch, so now t6 obtain the address of
    # trapframe for backup
    csrrw   t6, sscratch, t6
    # store registers x1 ~ x30 in trapframe
    .set      i, 1
    .rept     30
       save_gp    %i, t6
    .set       i, i+1
    .endr
    # store register x31 in trapframe
    mv      t5, t6
    csrr    t6, sscratch
    save_gp 31, t5

    # Use TrapFrame of current task when we switch to it
    csrw    sscratch, a0
    # Restore the content of current TrapFrame
    csrr    t6, sscratch
    .set    i, 1
    .rept    31
        load_gp     %i, t6
        .set        i, i+1
    .endr
    ret
