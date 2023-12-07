.section .text.trampoline
.globl trampoline
trampoline:
.align 4
.global uservec
uservec:
    // TODO
    j uservec

.global userret
userret:
    # a0: satp of the user task which we are going to switch
    csrw satp, a0
    sfence.vma zero, zero

    # Restore registers except t6 from trapframe
    # The trapframe of current user task is stored at sscratch
    csrrw   t6, sscratch, t6
    .set     i, 1
    .rept    30
        load_gp   %i, t6
        .set      i, i+1
    .endr
    # Restore t6
    mv      t5, t6
    csrr    t6, sscratch
    save_gp 31, t5

    # Set sscratch back to the trapframe of current task
    csrw    sscratch, t5

    sret
