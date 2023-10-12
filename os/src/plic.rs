/* [PLIC Reference](https://github.com/riscv/riscv-plic-spec/blob/master/riscv-plic.adoc)
 * [PLIC memory map of Qemu on xv6-riscv](https://github.com/mit-pdos/xv6-riscv/blob/riscv/kernel/memlayout.h)
 */
use crate::uart;
const UART0_IRQ: u32 = 10;

// Interrupt source UART0_IRQ(=10) priority
mmap_reg!(plic_pri_uart0, 0xc00_0000 + 4 * 10, u32);
// Enable bits for sources 0-31 on context 1(CPU0 / S mode)
mmap_reg!(plic_senable, 0xc00_0000 + 0x2080, u32);
// Priority threshold for context 1
mmap_reg!(plic_sthreshold, 0xc00_0000 + 0x20_1000, u32);
// Claim/complete for context 1
mmap_reg!(plic_sclaim, 0xc00_0000 + 0x20_1004, u32);

pub fn init() {
    // set the UART IRQ priority to non-zero
    plic_pri_uart0::write(1);
    // enable UART IRQ for hart 0 in S mode
    plic_senable::write(1 << UART0_IRQ);
    // set priority threshold to 0 for hart 0 in S mode
    plic_sthreshold::write(0);
}


pub fn irq_handler() {
    let irq = plic_sclaim::read();
    /* It is expected that the claim value here is nonzero,
     * because we should land here for external interrupt handling. */
    assert_ne!(irq, 0);

    match irq {
        UART0_IRQ => uart::irq_handler(),
        _ => panic!("Unknown external interrupt: {}", irq),
    }

    // Signal interrupt handle complete
    plic_sclaim::write(irq);
}
