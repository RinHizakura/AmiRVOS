// Reference: http://byterunner.com/16550.html
use crate::config::UART_BASE;
use core::convert::TryInto;

// Receive holding register (read mode)
const UART_RHR: usize = UART_BASE + 0;
// Transmit holding register (write mode)
const UART_THR: usize = UART_BASE + 0;
// LSB of Divisor Latch when Enabled
const UART_DIVISOR_LSB: usize = UART_BASE + 0;
// Interrupt enable register
const UART_IER: usize = UART_BASE + 1;
// MSB of Divisor Latch when Enabled
const UART_DIVISOR_MSB: usize = UART_BASE + 1;
// FIFO control register (write mode)
const UART_FCR: usize = UART_BASE + 2;
// Line control register
const UART_LCR: usize = UART_BASE + 3;
// Line status register
const UART_LSR: usize = UART_BASE + 5;

pub fn init() {
    unsafe {
        let lcr = UART_LCR as *mut u8;
        let fcr = UART_FCR as *mut u8;
        let ier = UART_IER as *mut u8;
        let divisor_lsb = UART_DIVISOR_LSB as *mut u8;
        let divisor_msb = UART_DIVISOR_MSB as *mut u8;

        // set word length to 8 bits
        lcr.write_volatile(0x3);
        // enable the transmit and receive FIFO
        fcr.write_volatile(0x1);
        // enable the receiver ready interrupt
        ier.write_volatile(0x1);

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
        let lcr_value = lcr.read_volatile();
        lcr.write_volatile(lcr_value | 1 << 7);

        // set the Divisor Latch when Enabled
        divisor_lsb.write_volatile(divisor_least);
        divisor_msb.write_volatile(divisor_most);

        // restore LCR to access original register
        lcr.write_volatile(lcr_value);
    }
}

pub fn uart_put(c: u8) {
    unsafe {
        let lsr = UART_LSR as *mut u8;
        let thr = UART_THR as *mut u8;

        // wait for transmit holding register is not full
        loop {
            match lsr.read_volatile() & 0x20 == 0 {
                true => {
                    continue;
                }
                false => {
                    break;
                }
            }
        }
        thr.write_volatile(c);
    }
}

pub fn uart_get() -> u8 {
    unsafe {
        let lsr = UART_LSR as *mut u8;
        let rhr = UART_RHR as *mut u8;

        // wait for receive holding register is not empty
        loop {
            match lsr.read_volatile() & 1 == 0 {
                true => {
                    continue;
                }
                false => {
                    break;
                }
            }
        }
        rhr.read_volatile()
    }
}
