# TODO: Consider to generate these syscall with auto script

.set SYS_open, 0
.set SYS_close, 1
.set SYS_read, 2
.set SYS_write, 3

.section .text.user
.global open
open:
    li a7, SYS_open
    ecall
    ret

.section .text.user
.global write
write:
    li a7, SYS_write
    ecall
    ret
