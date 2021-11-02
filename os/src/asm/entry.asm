.section .text.boot
.global _start
_start:
	# For the hart that id != 0, hanging in a infinite for loop
	csrr	t0, mhartid
	bnez	t0, 3f

	csrw	satp, zero

	# .bss section is reset to be zero
	la 	a0, _bss_start
	la	a1, _bss_end
	bgeu	a0, a1, 2f
1:
	sd	zero, (a0)
	addi	a0, a0, 8
	bltu	a0, a1, 1b
2:
	la	sp, _stack_end
        
        # (0b11 << 11): the privilege level will be set to 3(machine mode) after mret
        # for bit 7 of mstatus, the machine mode interrupt-enable bit will be 0 after mret
        li	t0, (0b11 << 11)
	csrw	mstatus, t0

        # pc will be set to address of kinit after mret
	la	t1, kinit
	csrw	mepc, t1

	la	ra, 2f
	mret
2:
        # (1 << 11): the privilege level will be set to 1(supervisor mode) after mret
        # (1 << 7): the machine mode interrupt-enable bit will be 1 after mret
        # (1 << 5): the supervisor mode interrupt-enable bit will be 1 after mret
        li	t0, (1 << 11) | (1 << 7) | (1 << 5)
        csrw	mstatus, t0

        # pc will be set to address of kmain after sret
        # note that MMU will be switched on after sret, so we have to
        # set sepc to the virtual address of kmain
	la	t0, kmain
        csrw	mepc, t0

        # M-mode and S-mode software interrupt, timer interrupt, and external
        # interrupt are enabled
	li	t0, 0xa | (0xa << 4) | (1 << 8)
	csrw	mie, t0

        # delegate software interrupt, timer interrupt, and external interrupt from
        # M-mode to S-mode
        li	t2, (1 << 1) | (1 << 5) | (1 << 9)
	csrw	mideleg, t2

        # set up page table and corresponding mode for virtual addressing
        la	t0, boot_page_table
        srli	t0, t0, 12
        li	t1, (8 << 60)
        or	t0, t0, t1
        csrw	satp, t0
        sfence.vma

        # hanging if return from kmain
	la	ra, 3f
        mret
3:
	wfi
	j	3b

# Initialize a page table to map the kernel in booting stage. We use 1G resolution pages
# to simplify the mapping work with assembly first, and change to 4K resolution pages
# by Rust codes after switching to supervisor mode later.
    .section .data
    .align 12
boot_page_table:
    # mapping address 0x0000_0000 to 0x0000_0000, which should set the entry of index 0
    .quad (0x00000 << 10) | 0xcf
    .quad 0
    # mapping address 0x8000_0000 to 0x8000_0000, which should set the entry of index 2
    .quad (0x80000 << 10) | 0xcf
    .zero 509 * 8
