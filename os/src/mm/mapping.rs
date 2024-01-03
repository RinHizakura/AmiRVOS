use crate::config::*;
use crate::lock::Locked;
use crate::mm::page;
use alloc::{vec, vec::Vec};
use bitflags::*;
use core::ops::{Index, IndexMut};
use lazy_static::lazy_static;
use riscv::register::satp;

lazy_static! {
    static ref MAPPING: Locked<Mapping> = Locked::new(Mapping::new());
}

bitflags! {
    #[derive(Default)]
    pub struct PteFlag: u8 {
    const VALID = 1 << 0;
    const READ = 1 << 1;
    const WRITE = 1 << 2;
    const EXECUTE = 1 << 3;
    const USER = 1 << 4;
    const GLOBAL = 1 << 5;
    const ACCESS = 1 << 6;
    const DIRTY = 1 << 7;
   }
}

struct PageTable {
    entries: PteArray,
}

struct PteArray(*mut Pte);
impl Index<usize> for PteArray {
    type Output = Pte;
    fn index(&self, idx: usize) -> &Self::Output {
        unsafe { self.0.add(idx).as_ref().unwrap() }
    }
}
impl IndexMut<usize> for PteArray {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        unsafe { self.0.add(idx).as_mut().unwrap() }
    }
}
/* If we implement a type that contains a type that is not Send or Sync, such as raw
 * pointers, and we want to mark that type as Send or Sync, we must use unsafe. see the
 * following explaination:
 *
 * - https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html
 * - reddit.com/r/learnrust/comments/k7rmlr/lazy_static_cannot_be_sent_between_threads_safely */
unsafe impl Send for PteArray {}

struct Pte(u64);
impl Pte {
    #[inline]
    fn set_value(&mut self, value: u64) {
        self.0 = value;
    }
    #[inline]
    fn get_value(&mut self) -> u64 {
        self.0
    }
    #[inline]
    fn is_valid(&self) -> bool {
        self.0 & PteFlag::VALID.bits() as u64 != 0
    }
    #[inline]
    fn get_next_table(&self) -> PageTable {
        let entries = PteArray(((self.0 & !0x3ff) << 2) as *mut Pte);
        PageTable { entries: entries }
    }
    #[inline]
    fn has_next_level(&self) -> bool {
        !(((self.0 & PteFlag::READ.bits() as u64) != 0)
            || (self.0 & PteFlag::EXECUTE.bits() as u64) != 0)
    }
    #[inline]
    fn page_num(&self) -> u64 {
        (self.0 & 0x003f_ffff_ffff_fc00) >> 10
    }
}

pub struct Segment {
    pub vaddr: u64,
    pub paddr: u64,
    pub len: u64,
    pub flags: PteFlag,
}

pub struct Mapping {
    page_tables: Vec<PageTable>,
    root_ppn: u64,
}

impl Mapping {
    pub fn new() -> Mapping {
        // allocate a page to create page table
        let root = PteArray(page::zalloc(0) as *mut Pte);
        let root_table = PageTable { entries: root };
        let root_ppn = (root_table.entries.0 as u64 >> 12) & 0x7ff_ffff_ffff;
        Mapping {
            page_tables: vec![root_table],
            root_ppn,
        }
    }

    pub fn satp(&self) -> u64 {
        /* 8 for sv39 page table */
        self.root_ppn | (8 << 60)
    }

    fn activate(&self) {
        satp::write(self.satp() as usize);
    }

    pub fn map(&mut self, segment: Segment) {
        /* 1. The alignment should be followed
         * 2. No extra check on duplicate vaddr, we should carefully decide it */
        assert_eq!(align_up!(segment.vaddr, PAGE_SIZE as u64), segment.vaddr);
        assert_eq!(align_up!(segment.paddr, PAGE_SIZE as u64), segment.paddr);
        let len = align_up!(segment.len, PAGE_SIZE as u64);
        for offset in (0..len).step_by(PAGE_SIZE) {
            self.map_one(
                segment.vaddr + offset,
                segment.paddr + offset,
                segment.flags.bits() | (PteFlag::VALID).bits(),
            );
        }
    }

    fn map_one(&mut self, vaddr: u64, paddr: u64, flags: u8) {
        let vpn = [
            (vaddr >> 12) & 0x1ff,
            (vaddr >> 21) & 0x1ff,
            (vaddr >> 30) & 0x1ff,
        ];

        let ppn = [
            (paddr >> 12) & 0x1ff,
            (paddr >> 21) & 0x1ff,
            (paddr >> 30) & 0x3ff_ffff,
        ];
        /* declare the next level table here so it won't be drop in the for iteration */
        let mut new_table;
        let root_table = &mut self.page_tables[0];
        let mut next_entry = &mut root_table.entries[vpn[2] as usize];
        /* FIXME: map page table for different level */
        for i in (0..2).rev() {
            if !next_entry.is_valid() {
                let p = page::zalloc(1);
                /* write the information of the next level page table into current entry */
                next_entry
                    .set_value(((p as i64 >> 2) | (PteFlag::VALID.bits() as u16 as i64)) as u64);
            }
            new_table = next_entry.get_next_table();
            next_entry = &mut new_table.entries[vpn[i] as usize];
        }

        next_entry.set_value(
            ((ppn[2] << 28) as i64
                | (ppn[1] << 19) as i64
                | (ppn[0] << 10) as i64
                | flags as u16 as i64) as u64,
        );
    }

    fn walk(&mut self, vaddr: u64) -> Option<u64> {
        let vaddr = align_down!(vaddr, PAGE_SIZE as u64);

        let vpn = [
            (vaddr >> 12) & 0x1ff,
            (vaddr >> 21) & 0x1ff,
            (vaddr >> 30) & 0x1ff,
        ];

        let mut new_table;
        let root_table = &mut self.page_tables[0];
        let mut next_entry = &mut root_table.entries[vpn[2] as usize];
        let mut offset_length = 12 + 2 * 9;
        for i in (0..2).rev() {
            if !next_entry.is_valid() {
                return None;
            }

            if next_entry.has_next_level() {
                offset_length -= 9;
                new_table = next_entry.get_next_table();
                next_entry = &mut new_table.entries[vpn[i] as usize];
            } else {
                break;
            }
        }
        let offset = vaddr & ((1 << offset_length) - 1);
        return Some(next_entry.page_num() << 12 + offset as u64);
    }

    pub fn copy_from_user(&self, addr: usize, buf: &mut [u8]) {
        todo!("copy_from_user()")
    }
}

impl Drop for Mapping {
    /* TODO: Reclaim memory for the page allocation */
    fn drop(&mut self) {
        todo!("reclaimation for map")
    }
}

pub fn init() {
    // map the memory region including kernel code, stack, and heap

    extern "C" {
        static KERNEL_END: usize;
        static TEXT_START: usize;
        static RODATA_START: usize;
        static DATA_START: usize;
        static BSS_START: usize;
        fn trampoline();
    }

    unsafe {
        MAPPING.lock().map(Segment {
            vaddr: TEXT_START as u64,
            paddr: TEXT_START as u64,
            len: RODATA_START as u64 - TEXT_START as u64,
            flags: PteFlag::EXECUTE | PteFlag::READ,
        });

        MAPPING.lock().map(Segment {
            vaddr: RODATA_START as u64,
            paddr: RODATA_START as u64,
            len: DATA_START as u64 - RODATA_START as u64,
            flags: PteFlag::READ,
        });

        MAPPING.lock().map(Segment {
            vaddr: DATA_START as u64,
            paddr: DATA_START as u64,
            len: BSS_START as u64 - DATA_START as u64,
            flags: PteFlag::READ | PteFlag::WRITE,
        });

        MAPPING.lock().map(Segment {
            vaddr: BSS_START as u64,
            paddr: BSS_START as u64,
            len: KERNEL_END as u64 - BSS_START as u64,
            flags: PteFlag::READ | PteFlag::WRITE,
        });

        MAPPING.lock().map(Segment {
            vaddr: KERNEL_END as u64,
            paddr: KERNEL_END as u64,
            len: HIGH_MEMORY as u64 - KERNEL_END as u64,
            flags: PteFlag::READ | PteFlag::WRITE,
        });

        MAPPING.lock().map(Segment {
            vaddr: UART_BASE as u64,
            paddr: UART_BASE as u64,
            len: 100,
            flags: PteFlag::READ | PteFlag::WRITE,
        });

        MAPPING.lock().map(Segment {
            vaddr: CLINT_BASE as u64,
            paddr: CLINT_BASE as u64,
            len: CLINT_SIZE as u64,
            flags: PteFlag::READ | PteFlag::WRITE,
        });

        MAPPING.lock().map(Segment {
            vaddr: PLIC_BASE as u64,
            paddr: PLIC_BASE as u64,
            len: PLIC_SIZE as u64,
            flags: PteFlag::READ | PteFlag::WRITE,
        });

        MAPPING.lock().map(Segment {
            vaddr: VIRTIO_BASE as u64,
            paddr: VIRTIO_BASE as u64,
            len: VIRTIO_SIZE as u64,
            flags: PteFlag::READ | PteFlag::WRITE,
        });

        /* The trampoline is a part of text section which we have mapped
         * before, but we map another here to share with user space. */
        MAPPING.lock().map(Segment {
            vaddr: TRAMPOLINE_VA as u64,
            paddr: trampoline as u64,
            len: PAGE_SIZE as u64,
            flags: PteFlag::EXECUTE | PteFlag::READ,
        });
    }
    MAPPING.lock().activate();
}

pub fn test() {
    /* simply check if we did linear map the address space */
    let vaddr = (DRAM_BASE + 0x2000) as u64;
    match MAPPING.lock().walk(vaddr) {
        None => panic!("walking page table of vaddr {:X} failed!\n", vaddr),
        Some(paddr) => {
            assert_eq!(vaddr, paddr);
        }
    }
}
