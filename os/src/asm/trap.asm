# Avoid RVC instruction to make sure that each vector function starts with 4 bytes alignment
.option norvc
.altmacro

.macro save_gp i, basereg
	sd	x\i, ((\i)*8)(\basereg)
.endm
.macro load_gp i, basereg
	ld	x\i, ((\i)*8)(\basereg)
.endm

.section .text
.global timervec
# The irq handler for machine mode will only be used to handle timer interrupt
# currently. It follows the similar approach of s_trap_vector.
.align 4
timervec:
    csrrw   t6, mscratch, t6

    .set      i, 1
    .rept     30
       save_gp    %i, t6
       .set       i, i+1
    .endr

    mv      t5, t6
    csrr    t6, mscratch
    save_gp 31, t5

    csrw    mscratch, t5

    la       sp, _trap_stack_end

    call     m_irq_handler

    # Arrange a supervisor software interrupt after this, and we'll
    # context switch then.
    # TODO We should do this in m_irq_handler. The reason that we are
    # not just because sip::write is not support in riscv library :(
    li       t0, 2
    csrw     sip, t0

    csrr     t6, mscratch

    .set     i, 1
    .rept    31
        load_gp   %i, t6
        .set      i, i+1
    .endr

    mret

.section .text.trampoline
.global s_trap_vector
# Since the stvec register uses the last two bits to change the trapping
# mode, we need to align the address of trap vector to ensure available address.
.align 4
s_trap_vector:
    # exchange t6(x31) and sscratch, so now t6 obtain the address of trapframe for backup
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

    # restore trap frame address back to sscratch
    csrw    sscratch, t5

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

    # kernel trap handler should return the satp for current task
    csrw     satp, a0
    sfence.vma

    # load the trap frame back into t6
    csrr     t6, sscratch

    # restore registers x1 ~ x31 in trapframe
    .set     i, 1
    .rept    31
        load_gp   %i, t6
        .set      i, i+1
    .endr

    sret

.global switch_to
switch_to:
    # a0 - address of TrapFrame for current task
    # a1 - satp for current task
    # a2 - mode bit for current task

    # Load satp first to access the correct trap frame
    csrw    satp, a1
    sfence.vma

    # Use TrapFrame of current task when we switch to it
    csrw    sscratch, a0

    # Load program counter from TrapFrame
    ld      t0, 280(a0)
    csrw    sepc, t0
    # Set priviledge mode according to the mode bit
    # TODO: Could we write this more simpler?
    csrr    t0, sstatus
    li      t1, (1 << 8)
    not     t1, t1
    and     t0, t0, t1
    slli    t1, a2, 8
    or      t0, t0, t1
    csrw    sstatus, t0

    # Restore the content from TrapFrame
    csrr    t6, sscratch
    .set    i, 1
    .rept    31
        load_gp     %i, t6
        .set        i, i+1
    .endr

    sret
