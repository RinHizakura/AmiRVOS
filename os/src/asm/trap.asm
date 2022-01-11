# Avoid RVC instruction to make sure that each vector function starts with 4 bytes alignment
.option norvc

.section .text
.global m_trap_vector
# Since the stvec register uses the last two bits to change the trapping
# mode, we need to align the address of trap vector to ensure available address.
.align 4
m_trap_vector:
    # TODO: we should save the kernel process context to restore from irq handler
    csrr    a0, mcause
    csrr    a1, mtval
    jal  m_irq_handler

.section .text
.global s_trap_vector
# Since the stvec register uses the last two bits to change the trapping
# mode, we need to align the address of trap vector to ensure available address.
.align 4
s_trap_vector:
    # TODO: we should save the kernel process context to restore from irq handler
    csrr    a0, scause
    csrr    a1, stval
    jal  s_irq_handler
