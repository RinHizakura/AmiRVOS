# TODO: Consider to generate these syscall with auto script

.equ SYS_write, 0

.section .text.user
.global write
write:
    li a7, SYS_write
    ecall
    ret