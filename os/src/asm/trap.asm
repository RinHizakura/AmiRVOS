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
    addi sp, sp, -(8 * 33)
    .set      i, 0
    .rept     32
       save_gp    %i, sp
       .set       i, i+1
    .endr

    call     timer_trap_handler

    # Arrange a supervisor software interrupt after this, and we'll
    # context switch then.
    # TODO We should do this in m_irq_handler. The reason that we are
    # not just because sip::write is not support in riscv library :(
    li       t0, 2
    csrw     sip, t0

    .set     i, 0
    .rept    32
        load_gp   %i, sp
        .set      i, i+1
    .endr
    addi sp, sp, (8 * 33)

    mret

.section .text
.global kernelvec
# Since the stvec register uses the last two bits to change the trapping
# mode, we need to align the address of trap vector to ensure available address.
.align 4
kernelvec:
    # Make room on stack and save gp-registers on it
    addi sp, sp, -(8 * 33)
    .set      i, 0
    .rept     32
       save_gp    %i, sp
       .set       i, i+1
    .endr

    # jump to kernel trap handler
    call kernel_trap_handler

    .set     i, 0
    .rept    32
        load_gp   %i, sp
        .set      i, i+1
    .endr
    addi sp, sp, (8 * 33)

    sret
