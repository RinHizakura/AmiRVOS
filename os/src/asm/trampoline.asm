.section .text.trampoline
.globl trampoline
trampoline:
.align 4
.global uservec
uservec:
    # Store trapframe address to t6 and t6 old value to sscratch
    csrrw t6, sscratch, t6

    # Save all general purpose registers except t6 to trapframe
    .set      i, 1
    .rept     30
       save_gp    %i, t6
       .set       i, i+1
    .endr
    # Save t6 to trapframe
    mv      t5, t6
    csrr    t6, sscratch
    save_gp 31, t5

    # Restore trapframe address of current task back to sscratch
    csrw    sscratch, t5

    # (Note: Now the trapframe address is stored at t5)
    # load kernel satp address
    ld    t0, 256(t5)
    # load kernel trap address
    ld    t1, 264(t5)
    # load kernel stack address
    ld    sp, 272(t5)

    # install kernel satp
    csrw satp, t0
    sfence.vma

    # jump to kernel trap handler
    jalr      t1


.global userret
userret:
    # a0: satp of the user task which we are going to switch
    csrw satp, a0
    sfence.vma

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
    load_gp 31, t5

    # Set sscratch back to the trapframe of current task
    csrw    sscratch, t5

    sret
