#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use core::ffi::c_int;

pub const MINORBITS: c_int = 20;
pub const MINORMASK: c_int = (1 << MINORBITS) - 1;

pub fn MAJOR(dev: c_int) -> u16 {
    (dev >> MINORBITS) as u16
}

pub fn MINOR(dev: c_int) -> u16 {
    (dev as c_int & MINORMASK) as u16
}

pub type mode_t = c_int;
pub type dev_t = c_int;
