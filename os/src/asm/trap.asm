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
.global m_trap_vector
# The irq handler for machine mode will only be used to handle timer interrupt
# currently. It follows the similar approach of s_trap_vector.
.align 4
m_trap_vector:
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

    csrr    t0, mepc
    sd      t0, 264(t5)
    csrr    t1, satp
    sd      t1, 256(t5)

    la       sp, _trap_stack_end

    call     m_irq_handler

    csrr     t6, mscratch

    ld       t1, 256(t6)
    csrw     satp, t1

    .set     i, 1
    .rept    31
        load_gp   %i, t6
        .set      i, i+1
    .endr

    mret

.section .text.trap
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

    # store sepc in trapframe
    csrr    t0, sepc
    sd      t0, 264(t5) # sizeof(u64) * 33

    # store satp in trapframe
    csrr    t1, satp
    sd      t1, 256(t5) # sizeof(u64) * 32

    la       sp, _trap_stack_end

    call     s_irq_handler

    # load the trap frame back into t6
    csrr     t6, sscratch

    ld       t1, 256(t6)
    csrw     satp, t1

    # restore registers x1 ~ x31 in trapframe
    .set     i, 1
    .rept    31
        load_gp   %i, t6
        .set      i, i+1
    .endr

    sret
