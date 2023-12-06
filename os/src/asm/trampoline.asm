.section .text.trampoline
.globl trampoline
trampoline:
.align 4
.global uservec
uservec:
    // TODO
    j uservec
