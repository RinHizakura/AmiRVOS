use super::*;

use crate::lock::Locked;
use crate::mm::page::zalloc;

use core::ptr::null_mut;
use lazy_static::lazy_static;

const QSIZE: usize = 8;

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

struct Disk {
    desc: *mut VirtqDesc,
    avail: *mut VirtqAvail,
    used: *mut VirtqUsed,
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
        }
    }

    fn allocq(&mut self) {
        assert!(self.desc.is_null());
        assert!(self.avail.is_null());
        assert!(self.used.is_null());

        self.desc = zalloc(0) as *mut VirtqDesc;
        self.avail = zalloc(0) as *mut VirtqAvail;
        self.used = zalloc(0) as *mut VirtqUsed;
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
