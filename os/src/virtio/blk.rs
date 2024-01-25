use super::*;

use crate::cpu;
use crate::lock::Locked;
use crate::mm::page::zalloc;
use crate::utils::cast::to_struct;

use core::mem::size_of;
use core::ptr::null_mut;
use lazy_static::lazy_static;

// This should be a power of 2 value
const QSIZE: usize = 8;
const SECTOR_SIZE: usize = 512;

#[repr(C)]
struct VirtqDesc {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

#[repr(C)]
struct VirtqAvail {
    flags: u16,
    idx: u16,
    ring: [u16; QSIZE],
}

#[repr(C)]
struct VirtqUsedElem {
    id: u32,
    len: u32,
}

#[repr(C)]
struct VirtqUsed {
    flags: u16,
    idx: u16,
    ring: [VirtqUsedElem; QSIZE],
}

#[repr(C)]
#[derive(Copy, Clone)]
struct VirtioBlkReq {
    typ: u32,
    reserved: u32,
    sector: u64,
}

impl Default for VirtioBlkReq {
    fn default() -> Self {
        VirtioBlkReq {
            typ: 0,
            reserved: 0,
            sector: 0,
        }
    }
}

struct Disk {
    /* The buffer for virtio-blk request descriptor. Note that this is not a
     * ring buffer, we have to check the "free" array member and know which
     * descriptor entry is free to be used for the command. Most command will
     * require a chain of descriptors. */
    desc: *mut VirtqDesc,
    avail: *mut VirtqAvail,
    used: *mut VirtqUsed,

    // The buffer for virtio-blk request entries
    req: [VirtioBlkReq; QSIZE],
    /* The status field for virtio-blk request's result is seperated from
     * the VirtioBlkReq intensionally, because descriptors seem must be either
     * read-only or write-only for the device.
     *
     * See:
     *  - https://brennan.io/2020/03/22/sos-block-device/
     *  - https://wiki.osdev.org/Virtio#Block%20Device%20Packets
     *  for more information */
    status: [u8; QSIZE],

    /* FIXME: The waiting state to synchronize between normal
     * routine and interrupt handler. Check whether we have better approach to
     * avoid naive synchronization like this. */
    wait: [bool; QSIZE],

    // It marks whether a descriptor entry is free to be used
    free_desc: [bool; QSIZE],

    used_idx: u16,
}
/* FIXME: Get avoid to unsafe if possible */
unsafe impl Sync for Disk {}
unsafe impl Send for Disk {}

impl Disk {
    fn new() -> Self {
        Disk {
            desc: null_mut(),
            avail: null_mut(),
            used: null_mut(),

            req: [VirtioBlkReq::default(); QSIZE],
            status: [0; QSIZE],
            wait: [false; QSIZE],
            free_desc: [true; QSIZE],

            used_idx: 0,
        }
    }

    fn allocq(&mut self) {
        assert!(self.desc.is_null());
        assert!(self.avail.is_null());
        assert!(self.used.is_null());

        /* Although we expect to have QSIZE of entries for each buffer, we
         * still allocate a PAGESIZE for these buffer because of the align
         * requirement.
         *
         * TODO: Maybe do some extra check to make sure that the allocated
         * space is enough to contain QSIZE of entries. */
        self.desc = zalloc(0) as *mut VirtqDesc;
        self.avail = zalloc(0) as *mut VirtqAvail;
        self.used = zalloc(0) as *mut VirtqUsed;
    }

    fn alloc_desc(&mut self) -> Option<usize> {
        for idx in 0..QSIZE {
            if self.free_desc[idx] == true {
                self.free_desc[idx] = false;
                return Some(idx);
            }
        }

        None
    }

    fn free_desc(&mut self, idx: usize) {
        if idx >= QSIZE {
            panic!("Invalid descriptor index to free");
        }

        if self.free_desc[idx] {
            panic!("Non allocated descriptor to free");
        }

        self.free_desc[idx] = true;
    }

    fn alloc_n_desc(&mut self, n: usize) -> Option<[usize; 3]> {
        let mut descs = [0; 3];

        for cnt in 0..n {
            if let Some(idx) = self.alloc_desc() {
                descs[cnt] = idx;
            } else {
                for j in 0..cnt {
                    self.free_desc(descs[j]);
                }
                return None;
            }
        }

        Some(descs)
    }

    fn free_desc_chain(&mut self, n: usize) {
        let mut idx = n;
        loop {
            let desc = self.get_desc(idx);
            let flag = unsafe { (*desc).flags };
            let next = unsafe { (*desc).next };
            self.free_desc(idx);
            if flag & VRING_DESC_F_NEXT != 0 {
                idx = next as usize;
            } else {
                break;
            }
        }
    }

    fn get_desc(&self, idx: usize) -> *mut VirtqDesc {
        self.desc.wrapping_offset(idx as isize)
    }
}

lazy_static! {
    static ref DISK: Locked<Disk> = Locked::new(Disk::new());
}
static DEV: VirtioDev = VirtioDev::new(VIRTIO0);

pub fn init() {
    /* Note: We may need to probe for each virtio device instead of
     * assuming that it is virtio-blk */

    // Reverse of "virt"
    assert!(DEV.read(VIRTIO_MMIO_MAGIC_VALUE) == 0x74726976);
    // Version 2 for non-legacy virtio
    assert!(DEV.read(VIRTIO_MMIO_VERSION) == 2);
    // It means virtio-blk for device ID 2
    assert!(DEV.read(VIRTIO_MMIO_DEVICE_ID) == 2);
    // Reverse of "QEMU"
    assert!(DEV.read(VIRTIO_MMIO_VENDOR_ID) == 0x554d4551);

    // Reference: 3.1.1 Driver Requirements: Device Initialization
    /* 1. Reset the device by writing 0 to the status register. */
    let mut status = 0;
    DEV.write(VIRTIO_MMIO_STATUS, status);

    /* 2. Set the ACKNOWLEDGE status bit. */
    status |= VIRTIO_CONFIG_S_ACKNOWLEDGE;
    DEV.write(VIRTIO_MMIO_STATUS, status);

    /* 3. Set the DRIVER status bit. */
    status |= VIRTIO_CONFIG_S_DRIVER;
    DEV.write(VIRTIO_MMIO_STATUS, status);

    /* 4. Read device feature bits, and write the subset of feature bits understood
     * by the OS and driver to the device. */
    let mut feats = DEV.read(VIRTIO_MMIO_DEVICE_FEATURES);
    /* FIXME: Reference to
     * https://github.com/mit-pdos/xv6-riscv/blob/riscv/kernel/virtio_disk.c
     * How can we make sure which feature to be negotiated ourself? */
    feats &= !(VIRTIO_BLK_F_RO
        | VIRTIO_BLK_F_CONFIG_WCE
        | VIRTIO_BLK_F_MQ
        | VIRTIO_F_ANY_LAYOUT
        | VIRTIO_RING_F_EVENT_IDX
        | VIRTIO_RING_F_INDIRECT_DESC);
    DEV.write(VIRTIO_MMIO_DEVICE_FEATURES, feats);

    /* 5. Set the FEATURES_OK status bit. */
    status |= VIRTIO_CONFIG_S_FEATURES_OK;
    DEV.write(VIRTIO_MMIO_STATUS, status);

    /* 6. Re-read device status to ensure the FEATURES_OK bit is still set: otherwise,
     * the device does not support our subset of features and the device is unusable. */
    status = DEV.read(VIRTIO_MMIO_STATUS);
    assert!(status & VIRTIO_CONFIG_S_FEATURES_OK != 0);

    /* 7. Perform device-specific setup, including discovery of virtqueues for the
     * device, optional per-bus setup, reading and possibly writing the device’s virtio
     * configuration space, and population of virtqueues. */

    /* Select queue 0 */
    DEV.write(VIRTIO_MMIO_QUEUE_SEL, 0);

    /* queue 0 should not be used */
    assert!(DEV.read(VIRTIO_MMIO_QUEUE_READY) == 0);

    /* Ensure the supported maximum queue size meets the requirements */
    let max = DEV.read(VIRTIO_MMIO_QUEUE_NUM_MAX);
    assert!(max > 0 && max > QSIZE as u32);

    /* Set queue size */
    DEV.write(VIRTIO_MMIO_QUEUE_NUM, QSIZE as u32);

    /* Allocate queue for virtio-blk */
    DISK.lock().allocq();

    /* For non-legacy interface, we can have each virtq independently */
    DEV.write(VIRTIO_MMIO_QUEUE_DESC_LOW, DISK.lock().desc as u64 as u32);
    DEV.write(
        VIRTIO_MMIO_QUEUE_DESC_HIGH,
        ((DISK.lock().desc as u64) >> 32) as u32,
    );
    DEV.write(VIRTIO_MMIO_DRIVER_DESC_LOW, DISK.lock().avail as u64 as u32);
    DEV.write(
        VIRTIO_MMIO_DRIVER_DESC_HIGH,
        ((DISK.lock().avail as u64) >> 32) as u32,
    );
    DEV.write(VIRTIO_MMIO_DEVICE_DESC_LOW, DISK.lock().used as u64 as u32);
    DEV.write(
        VIRTIO_MMIO_DEVICE_DESC_HIGH,
        ((DISK.lock().used as u64) >> 32) as u32,
    );
    DEV.write(VIRTIO_MMIO_QUEUE_READY, 1);

    /* 8. Set the DRIVER_OK status bit. At this point the device is “live” */
    status |= VIRTIO_CONFIG_S_DRIVER_OK;
    DEV.write(VIRTIO_MMIO_STATUS, status);
}

pub fn disk_rw(buf: &[u8], offset: usize, is_write: bool) {
    let buf_size = buf.len();
    let mut disk = DISK.acquire();

    let sector = offset / SECTOR_SIZE;
    // Allocate 3 descriptors for this command
    let idxs = disk.alloc_n_desc(3).expect("alloc_n_desc(3)");
    /* Use the first index of the descriptor chain to pick a request entry.
     * This can help us to simply find the corresponding request when getting
     * response from device. */
    let req_idx = idxs[0];

    if is_write {
        disk.req[req_idx].typ = VIRTIO_BLK_T_OUT;
    } else {
        disk.req[req_idx].typ = VIRTIO_BLK_T_IN;
    }
    disk.req[req_idx].sector = sector as u64;

    let req_ptr = &disk.req[req_idx] as *const VirtioBlkReq;
    unsafe {
        let desc0 = disk.get_desc(idxs[0]);
        (*desc0).addr = req_ptr as u64;
        (*desc0).len = size_of::<VirtioBlkReq>() as u32;
        (*desc0).flags = VRING_DESC_F_NEXT;
        (*desc0).next = idxs[1] as u16;
    }

    unsafe {
        let desc1 = disk.get_desc(idxs[1]);
        (*desc1).addr = buf.as_ptr() as u64;
        (*desc1).len = buf_size as u32;
        (*desc1).flags = if is_write { 0 } else { VRING_DESC_F_WRITE };
        (*desc1).flags |= VRING_DESC_F_NEXT;
        (*desc1).next = idxs[2] as u16;
    }

    /* Set the status field to a invalid value, so we can
     * observe the change if the device modifies it. */
    disk.status[req_idx] = 0xff;
    unsafe {
        let status_ptr = &disk.status[req_idx] as *const u8;
        let desc2 = disk.get_desc(idxs[2]);
        (*desc2).addr = status_ptr as u64;
        (*desc2).len = size_of::<u8>() as u32;
        (*desc2).flags = VRING_DESC_F_WRITE;
        (*desc2).next = 0;
    }

    /* Put the first descriptor of the descriptor chain in the avail ring */
    unsafe {
        let avail_idx = (*disk.avail).idx as usize;
        (*disk.avail).ring[avail_idx % QSIZE] = idxs[0] as u16;
        (*disk.avail).idx += 1;
    }

    /* Set the flag and wait for the interrupt handler to reset, which
     * means the request is completed */
    disk.wait[req_idx] = true;

    // Notify queue 0 for the request
    DEV.write(VIRTIO_MMIO_QUEUE_NOTIFY, 0);

    /* FIXME: Release the disk lock and blocking wait for virtio-blk
     * to finish. The lock should be required by virtio::blk::irq_handler()
     * a few times later.
     *
     * Note that this implementation will cause deadlock if other task
     * also want to make request, so we need to improve this in the
     * future. */
    DISK.release(disk);

    /* FIXME: Here we'll race the lock with virtio::bl::irq_handler(), so
     * we have to synchronize for the share state. Looking for better
     * synchronization mechanism to do this in prettier and readable
     * implementation. */
    let mut stop_wait = true;
    while stop_wait {
        let mut disk = DISK.acquire();
        if !disk.wait[req_idx] {
            stop_wait = false;
        }
        DISK.release(disk);
    }

    // Get the lock again and release allocated descriptors
    let mut disk = DISK.acquire();
    disk.free_desc_chain(idxs[0]);
    DISK.release(disk);
}

pub fn irq_handler() {
    /* A bit mask of events that cause the interrupt is showed in this register:
     * - bit 0 - the device has used a buffer in at least one of the active virtual queues
     * - bit 1 - the configuration of the device has changed */
    let status = DEV.read(VIRTIO_MMIO_INTERRUPT_STATUS);
    /* Ack the interrupt with value in InterruptStatus to notify the device
     * that events causing the interrupt have been handled */
    DEV.write(VIRTIO_MMIO_INTERRUPT_ACK, status & 0x3);

    let mut disk = DISK.acquire();

    /* The used ring is used to receive the response. */
    while disk.used_idx != unsafe { (*disk.used).idx } {
        /* Note that the device don't have to handle the request in order,
         * so we use the id of element */
        let id = unsafe { (*disk.used).ring[disk.used_idx as usize % QSIZE].id as usize };
        /* TODO: The status is expected to be VIRTIO_BLK_S_OK for success
         * request. Maybe we can consider to do error handling here */
        assert!(disk.status[id] == VIRTIO_BLK_S_OK);

        // Notify the request is completed
        disk.wait[id] = false;

        disk.used_idx += 1;
    }

    DISK.release(disk);
}
