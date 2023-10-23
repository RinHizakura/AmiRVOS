macro_rules! align_up {
    ($val: expr, $align:expr) => {
        ($val) + ((!($val) + 1) & (($align) - 1))
    };
}

macro_rules! align_down {
    ($val: expr, $align:expr) => {
        ($val) & !(($align) - 1)
    };
}

#[macro_export]
macro_rules! mmap_reg {
    ($name: ident, $addr: expr, $type: ty) => {
        mod $name {
            #[inline]
            #[allow(dead_code)]
            unsafe fn __write(value: $type) {
                let reg = $addr as *mut $type;
                reg.write_volatile(value);
            }

            #[inline]
            #[allow(dead_code)]
            pub fn write(value: $type) {
                unsafe {
                    __write(value);
                }
            }

            #[inline]
            #[allow(dead_code)]
            unsafe fn __read() -> $type {
                let reg = $addr as *mut $type;
                reg.read_volatile()
            }

            #[inline]
            #[allow(dead_code)]
            pub fn read() -> $type {
                unsafe { __read() }
            }
        }
    };
}

#[macro_export]
macro_rules! order2size {
    ($order: expr) => {
        // FIXME: change (1 << 12) to PAGE_SIZE if possible
        (1 << $order) * (1 << 12)
    };
}
