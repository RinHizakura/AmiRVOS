use crate::config;
use crate::mm::page;
use alloc::{vec, vec::Vec};
use bitflags::*;
use core::ops::{Index, IndexMut};

pub struct Mapping {
    page_tables: Vec<PageTable>,
    root_ppn: usize,
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

pub struct PageTable {
    pub entries: Pte,
}

pub struct Pte(*mut u64);

pub struct Segment {
    pub start: usize,
    pub end: usize,
}

impl PageTable {
    pub fn zero_init(&mut self) {
        for idx in 0..511 {
            self.entries[idx] = 0;
        }
    }
}

impl Index<usize> for Pte {
    type Output = u64;
    fn index(&self, idx: usize) -> &Self::Output {
        unsafe { self.0.add(idx).as_ref().unwrap() }
    }
}

impl IndexMut<usize> for Pte {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        unsafe { self.0.add(idx).as_mut().unwrap() }
    }
}

impl Mapping {
    pub fn new() -> Mapping {
        // allocate a page to create page table
        let root = Pte(page::alloc(0) as *mut u64);
        let root_table = PageTable { entries: root };
        let root_ppn = root_table.entries.0 as usize;

        Mapping {
            page_tables: vec![root_table],
            root_ppn,
        }
    }

    pub fn map(&mut self, segment: Segment) {
        let start_addr = align_down!(segment.start, page::PAGE_SIZE);
        let end_addr = align_up!(segment.end, page::PAGE_SIZE);

        for addr in (segment.start..segment.end).step_by(page::PAGE_SIZE) {
            self.map_one(addr, addr, PteFlag::READ | PteFlag::WRITE | PteFlag::VALID);
        }
    }

    pub fn map_one(&mut self, vaddr: usize, paddr: usize, flags: PteFlag) {}
}

pub fn init() {
    let mut mapping = Mapping::new();

    // map the memory region including kernel code, stack, and heap
    // FIXME: we should consider the attribute for different section
    mapping.map(Segment {
        start: config::DRAM_BASE,
        end: config::HIGH_MEMORY,
    });
}
