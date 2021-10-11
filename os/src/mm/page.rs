use crate::config::{HIGH_MEMORY, LOW_MEMORY};
use core::ptr::null_mut;

// Page struct flag init with zero to represent a free page
static mut PAGE_STRUCT: [Page; PAGE_ENTRY] = [Page { flags: 0 }; PAGE_ENTRY];
// 4KB page
pub const PAGE_SIZE: usize = 1 << 12;
// Number of page entry availibled
const PAGE_ENTRY: usize = (HIGH_MEMORY - LOW_MEMORY) / PAGE_SIZE;

#[derive(Copy, Clone)]
pub struct Page {
    flags: u8,
}

impl Page {
    pub fn is_alloc(&self) -> bool {
        return (self.flags & 1) == 1;
    }

    pub fn set_alloc(&mut self) {
        self.flags |= 1;
    }

    pub fn clear(&mut self) {
        self.flags = 0;
    }

    pub fn set_order(&mut self, order: usize) {
        // a too large page order is invalid now
        assert!(order < 0xff);
        // record the order by encoding order + 1 in the bit [7:1] of flag
        self.flags |= ((order + 1) << 1) as u8 & !1;
    }

    pub fn get_order(&mut self) -> usize {
        let tmp = (self.flags & !1) >> 1;
        return if tmp == 0 {
            usize::MAX
        } else {
            tmp as usize - 1
        };
    }
}

pub fn init() {
    unsafe {
        extern "C" {
            static KERNEL_START: usize;
            static KERNEL_END: usize;
        }

        info!("Kernel region: [{:X} {:X}]", KERNEL_START, KERNEL_END);
        assert!(KERNEL_END < LOW_MEMORY);
    }
}

pub fn alloc(order: usize) -> *mut u8 {
    // only 2^n pages allocation is availible
    let pages = 1 << order;

    unsafe {
        for i in 0..PAGE_ENTRY {
            let mut found = false;

            if !PAGE_STRUCT[i].is_alloc() {
                found = true;

                // make sure the following page are also freed for contiguous allocation
                for j in i + 1..i + pages {
                    if PAGE_STRUCT[j].is_alloc() {
                        found = false;
                    }
                }
            }

            if found {
                for j in i..i + pages {
                    PAGE_STRUCT[j].set_alloc();
                }
                PAGE_STRUCT[i].set_order(order);
                return (LOW_MEMORY + i * PAGE_SIZE) as *mut u8;
            }
        }
    }

    return null_mut();
}

pub fn zalloc(order: usize) -> *mut u8 {
    let ret = alloc(order);
    let pages = 1 << order;

    if !ret.is_null() {
        /* FIXME: This could be inefficient because we clear memory byte by byte */
        let size = PAGE_SIZE * (1 << order);
        for i in 0..size {
            unsafe {
                (*ret.add(i)) = 0;
            }
        }
    }
    ret
}

pub fn free(ptr: *mut u8) {
    let addr = ptr as usize;

    // refuse the invalid address
    if ptr.is_null() || (addr < LOW_MEMORY) || (addr >= HIGH_MEMORY) {
        return;
    }

    unsafe {
        let idx = (addr - LOW_MEMORY) / PAGE_SIZE;
        let order = PAGE_STRUCT[idx].get_order();

        // make sure the 'ptr' point to the first allocaed block
        if !PAGE_STRUCT[idx].is_alloc() || order == usize::MAX {
            return;
        }

        let pages = 1 << order;
        for i in idx..idx + pages {
            assert!(PAGE_STRUCT[i].flags & 1 == 1);
            PAGE_STRUCT[i].clear();
        }
    }
}

pub fn test() {
    // test the page allocation behavior
    let a = alloc(0);
    let b = alloc(0);
    let c = alloc(0);

    free(a);
    let d = alloc(1);
    free(b);
    let e = alloc(1);

    assert_eq!(a, e);

    free(c);
    free(d);
    free(e);
}
