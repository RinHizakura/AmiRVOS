// Reference: http://byterunner.com/16550.html
use crate::lock::Locked;
use crate::utils::ringbuf::RingBuf;
use core::convert::TryInto;
use lazy_static::lazy_static;

// Transmit holding register (write mode)
mmap_reg!(uart_thr, 0x1000_0000 + 0, u8);

// Receive holding register (read mode)
mmap_reg!(uart_rhr, 0x1000_0000 + 0, u8);

// LSB of Divisor Latch when Enabled
mmap_reg!(uart_lsb, 0x1000_0000 + 0, u8);

// Interrupt enable register
mmap_reg!(uart_ier, 0x1000_0000 + 1, u8);

// MSB of Divisor Latch when Enabled
mmap_reg!(uart_msb, 0x1000_0000 + 1, u8);

// FIFO control register (write mode)
mmap_reg!(uart_fcr, 0x1000_0000 + 2, u8);

// Line control register
mmap_reg!(uart_lcr, 0x1000_0000 + 3, u8);

// Line status register
mmap_reg!(uart_lsr, 0x1000_0000 + 5, u8);

pub fn init() {
    // set word length to 8 bits
    uart_lcr::write(0x3);
    // enable the transmit and receive FIFO
    uart_fcr::write(0x1);
    // enable the receiver ready interrupt
    uart_ier::write(0x1);

    /* Note: the os dosen't try on a real hardware now, so maybe we should fix this
     *
     * Assume:
     * - the global clock rate is 1.8432 MHz
     * - the output freq of the Baudout is equal to the 16X of transmission baud rate
     * - we want 115200 BAUD rate
     *
     * Then the divisor should be: ceil( (1.8432 * 10 ^6) / (16 * 115200) ) = 1.0
     */
    let divisor: u16 = 1;
    let divisor_least: u8 = (divisor & 0xff).try_into().unwrap();
    let divisor_most: u8 = (divisor >> 8).try_into().unwrap();

    // enable the divisor latch
    let lcr_value = uart_lcr::read();
    uart_lcr::write(lcr_value | (1 << 7));

    // set the Divisor Latch when Enabled
    uart_lsb::write(divisor_least);
    uart_msb::write(divisor_most);

    // restore LCR to access original register
    uart_lcr::write(lcr_value);
}

pub fn uart_put(c: u8) {
    // wait for transmit holding register is not full
    loop {
        match uart_lsr::read() & 0x20 == 0 {
            true => {
                continue;
            }
            false => {
                break;
            }
        }
    }
    uart_thr::write(c);
}

pub fn uart_get() -> u8 {
    // wait for receive holding register is not empty
    loop {
        match uart_lsr::read() & 1 == 0 {
            true => {
                continue;
            }
            false => {
                break;
            }
        }
    }
    uart_rhr::read()
}

lazy_static! {
    static ref READ_BUFFER: Locked<RingBuf<u8, 512>> = Locked::new(RingBuf::new());
}

pub fn irq_handler() {
    let c = uart_get();

    /* TODO: Collect character in the ring buffer without
     * consuming it. We'll take care of this once console is supported
     * in the future. */
    READ_BUFFER.lock().push(c);

    // FIXME: Echo the character for checking now
    if (c as char).is_ascii_alphanumeric() {
        print!("{}", c as char);
    }
}
