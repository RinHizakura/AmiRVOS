# TODO: Consider to generate these syscall with auto script

.set SYS_open, 56
.set SYS_close, 57
.set SYS_read, 63
.set SYS_write, 64
.set SYS_mknod, 33

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

.section .text.user
.global mknod
mknod:
    li a7, SYS_mknod
    ecall
    ret
