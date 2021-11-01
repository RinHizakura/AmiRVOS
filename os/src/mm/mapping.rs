use crate::config;
use crate::mm::page;
use alloc::{vec, vec::Vec};
use bitflags::*;
use core::ops::{Index, IndexMut};

static mut MAPPING: Option<Mapping> = None;

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

pub struct PageTable {
    pub entries: PteArray,
}

pub struct PteArray(*mut Pte);
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

pub struct Pte(u64);
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
        self.0 & PteFlag::VALID.bits as u64 != 0
    }
    #[inline]
    fn get_next_table(&self) -> PageTable {
        let entries = PteArray(((self.0 & !0x3ff) << 2) as *mut Pte);
        PageTable { entries: entries }
    }
    #[inline]
    pub fn has_next_level(&self) -> bool {
        !(((self.0 & PteFlag::READ.bits as u64) != 0)
            || (self.0 & PteFlag::EXECUTE.bits as u64) != 0)
    }
    #[inline]
    pub fn page_num(&self) -> u64 {
        (self.0 & 0x003f_ffff_ffff_fc00) >> 10
    }
}

pub struct Segment {
    pub start: u64,
    pub end: u64,
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

    pub fn activate(&self) {
        /* 8 for sv39 page table */
        let new_satp = self.root_ppn | (8 << 60);
        unsafe {
            asm!("csrw satp, {}", in(reg) new_satp);
            asm!("sfence.vma");
        }
    }

    pub fn map(&mut self, segment: Segment) {
        let start_addr = align_down!(segment.start, page::PAGE_SIZE as u64);
        let end_addr = align_up!(segment.end, page::PAGE_SIZE as u64);

        for addr in (start_addr..end_addr).step_by(page::PAGE_SIZE) {
            self.map_one(
                addr,
                addr,
                PteFlag::EXECUTE | PteFlag::READ | PteFlag::WRITE | PteFlag::VALID,
            );
        }
    }

    pub fn map_one(&mut self, vaddr: u64, paddr: u64, flags: PteFlag) {
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
                    .set_value(((p as i64 >> 2) | (PteFlag::VALID.bits as u16 as i64)) as u64);
            }
            new_table = next_entry.get_next_table();
            next_entry = &mut new_table.entries[vpn[i] as usize];
        }

        next_entry.set_value(
            ((ppn[2] << 28) as i64
                | (ppn[1] << 19) as i64
                | (ppn[0] << 10) as i64
                | (flags.bits) as u16 as i64) as u64,
        );
    }

    pub fn walk(&mut self, vaddr: u64) -> Option<u64> {
        let vaddr = align_down!(vaddr, page::PAGE_SIZE as u64);

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
}

pub fn init() {
    unsafe {
        MAPPING = Some(Mapping::new());

        // map the memory region including kernel code, stack, and heap
        // FIXME: we should consider the attribute for different section

        let mapping = MAPPING.as_mut().unwrap();

        mapping.map(Segment {
            start: config::DRAM_BASE as u64,
            end: config::HIGH_MEMORY as u64,
        });

        mapping.map(Segment {
            start: config::UART_BASE as u64,
            end: (config::UART_BASE + 100) as u64,
        });

        mapping.activate();
    }
}

pub fn test() {
    unsafe {
        /* simply check if we did linear map the address space */
        let mapping = MAPPING.as_mut().unwrap();
        let vaddr = (config::DRAM_BASE + 0x2000) as u64;
        match mapping.walk(vaddr) {
            None => panic!("walking page table of vaddr {:X} failed!\n", vaddr),
            Some(paddr) => {
                assert_eq!(vaddr, paddr);
            }
        }
    }
}
