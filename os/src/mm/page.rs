use crate::config::{HIGH_MEMORY, LOW_MEMORY};
use core::ptr::null_mut;

// Page struct flag init with zero to represent a free page
static mut PAGE_STRUCT: [Page; PAGE_ENTRY] = [Page { flags: 0 }; PAGE_ENTRY];
// 4KB page
const PAGE_SIZE: usize = 1 << 12;
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

    pub fn set(&mut self) {
        self.flags = 1;
    }

    pub fn clear(&mut self) {
        self.flags = 0;
    }
}

pub fn init() {
    unsafe {
        extern "C" {
            static KERNEL_START: usize;
            static KERNEL_END: usize;
        }

        assert!(KERNEL_END < LOW_MEMORY);
        info!("Kernel region: [{:X} {:X}]", KERNEL_START, KERNEL_END);
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
                    PAGE_STRUCT[j].set();
                }
                // trickly record the order+1 of page in flag
                PAGE_STRUCT[i].flags |= ((order + 1) << 1) as u8 & !1;
                return (LOW_MEMORY + i * PAGE_SIZE) as *mut u8;
            }
        }
    }

    return null_mut();
}

pub fn free(ptr: *mut u8) {
    let addr = ptr as usize;

    // refuse the invalid address
    if ptr.is_null() || (addr < LOW_MEMORY) || (addr >= HIGH_MEMORY) {
        return;
    }

    unsafe {
        let idx = (addr - LOW_MEMORY) / PAGE_SIZE;
        let order = (PAGE_STRUCT[idx].flags & !1) >> 1;

        // makr sure the 'ptr' point to the first allocaed block
        if !PAGE_STRUCT[idx].is_alloc() || order == 0 {
            return;
        }
        // note that the record value is 'order + 1'
        let pages = 1 << (order - 1);

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

    runtest!(a == e);

    free(c);
    free(d);
    free(e);
}
