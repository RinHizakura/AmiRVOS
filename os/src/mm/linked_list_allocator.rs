use crate::mm::kheap::Locked;
use core::alloc::{GlobalAlloc, Layout};
use core::mem;
use core::ptr;

struct ListNode {
    size: usize,
    next: Option<&'static mut ListNode>,
}

impl ListNode {
    const fn new(size: usize) -> Self {
        ListNode { size, next: None }
    }

    fn start_addr(&self) -> usize {
        self as *const _ as usize
    }

    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}

pub struct LinkedListAllocator {
    head: ListNode,
}

impl LinkedListAllocator {
    pub const fn new() -> Self {
        Self {
            head: ListNode::new(0),
        }
    }

    fn size_by_align(layout: Layout) -> usize {
        let align = layout
            .align_to(mem::align_of::<ListNode>())
            .expect("adjusting alignment failed")
            .pad_to_align()
            .align();
        let size = layout.size().max(mem::size_of::<ListNode>());
        align_up!(size, align)
    }

    fn add_free_region(&mut self, addr: usize, size: usize) {
        // the address should follow the alignment of ListNode
        assert_eq!(align_up!(addr, mem::align_of::<ListNode>()), addr);
        // the free region has to be able to hold a ListNode
        assert!(size >= mem::size_of::<ListNode>());

        let mut current = &mut self.head;

        while let Some(ref next) = current.next {
            /* No address should exist among the range of node in the linked list, or it
            could be a double-free error */
            assert!(addr >= next.start_addr() || addr < next.end_addr());

            if addr < next.start_addr() {
                // merge node if there are continuous address in neighbor node
                if addr == current.end_addr() && addr + size == next.start_addr() {
                    current.size = current.size + size + next.size;

                    let mut_next = current.next.as_mut().unwrap();
                    current.next = mut_next.next.take();
                    return;
                } else if addr == current.end_addr() {
                    current.size = current.size + size;
                    return;
                } else if addr + size == next.start_addr() {
                    let mut_next = current.next.as_mut().unwrap();

                    let node_ptr = addr as *mut ListNode;
                    let mut node = ListNode::new(mut_next.size + size);
                    node.next = mut_next.next.take();
                    unsafe {
                        node_ptr.write(node);
                        current.next = Some(&mut *node_ptr);
                    }
                    return;
                }
                // else insert the node between current and next
                break;
            }

            current = current.next.as_mut().unwrap();
        }

        let node_ptr = addr as *mut ListNode;
        let mut node = ListNode::new(size);

        node.next = current.next.take();
        unsafe {
            node_ptr.write(node);
            current.next = Some(&mut *node_ptr);
        }
    }

    fn find_free_region(&mut self, size: usize) -> Option<usize> {
        let mut current = &mut self.head;

        // find the first fit free region for allocation
        while let Some(ref mut node) = current.next {
            let alloc_start = node.start_addr();
            let alloc_end = alloc_start + size;

            // go to next node if the required size is smaller than the free block
            if alloc_end > node.end_addr() {
                current = current.next.as_mut().unwrap();
                continue;
            }

            let next = node.next.take();

            /* The space with extra size which exceeds the requirement should be split and add
             * back to the allocator. However, if the rest of the region is not able to fit a
             * ListNode, then we don't split the region but allocate the whole space. */
            let excess_size = node.end_addr() - alloc_end;
            if excess_size > 0 && excess_size < mem::size_of::<ListNode>() {
                current.next = next;
                return Some(alloc_start);
            } else {
                /* We don't use add_free_region here because we already know where to add the
                 * node, but add_free_region will linear search the linked list which can waste
                 * time */
                let node_ptr = alloc_end as *mut ListNode;
                let mut new_node = ListNode::new(excess_size);
                new_node.next = next;
                unsafe {
                    node_ptr.write(new_node);
                    current.next = Some(&mut *node_ptr);
                }

                return Some(alloc_start);
            }
        }

        None
    }

    pub fn check_order(&mut self) {
        let mut current = &mut self.head;
        info!("\nDump the linked list allocator:");
        while let Some(ref next) = current.next {
            info!("addr 0x{:X} with size 0x{:X}", next.start_addr(), next.size);
            current = current.next.as_mut().unwrap();
        }
    }

    pub fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.add_free_region(heap_start, heap_size);
    }
}

unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = LinkedListAllocator::size_by_align(layout);
        let mut allocator = self.lock();

        if let Some(alloc_start) = allocator.find_free_region(size) {
            alloc_start as *mut u8
        } else {
            ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = LinkedListAllocator::size_by_align(layout);
        self.lock().add_free_region(ptr as usize, size)
    }
}
