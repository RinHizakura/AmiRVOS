# Avoid RVC instruction to make sure that each vector function starts with 4 bytes alignment
.option norvc
.altmacro

.macro save_gp i, basereg=t6
	sd	x\i, ((\i)*8)(\basereg)
.endm
.macro load_gp i, basereg=t6
	ld	x\i, ((\i)*8)(\basereg)
.endm

.section .text
.global m_trap_vector
# Since the stvec register uses the last two bits to change the trapping
# mode, we need to align the address of trap vector to ensure available address.
.align 4
m_trap_vector:
    csrr    a0, mcause
    csrr    a1, mtval
    jal     m_irq_handler

.section .text
.global s_trap_vector
# Since the stvec register uses the last two bits to change the trapping
# mode, we need to align the address of trap vector to ensure available address.
.align 4
s_trap_vector:
   # exchange t6 and mscratch, so now t6 obtain the address of trapframe for backup
   csrrw   t6, sscratch, t6

   # store registers x1 ~ x30 in trapframe
   .set      i, 1
   .rept     30
       save_gp    %i
       .set       i, i+1
   .endr

    # store register x31 in trapframe
    mv      t5, t6
    csrr    t6, sscratch
    save_gp 31, t5

    # restore trap frame address back to mscratch
    csrw    sscratch, t5

    # store sepc in trapframe
    csrr    t0, sepc
    sd      t0, 264(t5)
    # store satp in trapframe
    csrr    t1, satp
    sd      t1, 256(t5)

    la       sp, _trap_stack_end

    csrr     a0, sepc
    csrr     a1, scause
    csrr     a2, stval
    call     s_irq_handler

    csrw     sepc, a0

    # load the trap frame back into t6
    csrr     t6, sscratch

    # restore registers x1 ~ x31 in trapframe
    .set     i, 1
    .rept    31
        load_gp    %i
	.set	i, i+1
    .endr

    sret
