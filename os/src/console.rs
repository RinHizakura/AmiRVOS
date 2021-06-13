use crate::uart::uart_put;
use core::fmt::{self, Error, Write};

struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, out: &str) -> Result<(), Error> {
        for c in out.bytes() {
            uart_put(c);
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

macro_rules! info {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!("\x1b[1;94m", $fmt, "\x1b[0m\n") $(, $($arg)+)?));
    }
}
