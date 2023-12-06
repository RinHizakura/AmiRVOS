
/* Delay with assembly loop, so compiler won't
 * optimize such logic. */
.globl delay
delay:
    addi a0, a0, -1
    bnez a0, delay
    ret
