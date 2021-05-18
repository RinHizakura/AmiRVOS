.section .data
.section .text.boot
.global _start
_start:
	# For the hart that id != 0, hanging in a infinite for loop
	csrr	t0, mhartid
	bnez	t0, 3f

	# .bss section is reset to be zero
	la 		a0, _bss_start
	la		a1, _bss_end
	bgeu	a0, a1, 2f
1:
	sd		zero, (a0)
	addi	a0, a0, 8
	bltu	a0, a1, 1b
2:
	la		sp, _stack_end
        
        # (0b11 << 11): the privilege level will be set to 3(machine mode) after mret
        # (1 << 7): the machine mode interrupt-enable bit will be 1 after mret
	li		t0, (0b11 << 11) | (1 << 7)
	csrw	mstatus, t0

        # pc will be set to address of rust_main after mret
	la		t1, rust_main
	csrw	mepc, t1

        # M-mode software interrupt, timer interrupt, and  external interrupt are enabled
	li		t3, (1 << 3) | (1 << 7) | (1 << 11)
	csrw	mie, t3

        # hanging if return from rust_main
	la		ra, 3f
	mret
3:
	wfi
	j		3b
