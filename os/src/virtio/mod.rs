pub mod blk;

pub(self) const VIRTIO_MMIO_MAGIC_VALUE: usize = 0x000;
pub(self) const VIRTIO_MMIO_VERSION: usize = 0x004;
pub(self) const VIRTIO_MMIO_DEVICE_ID: usize = 0x008;
pub(self) const VIRTIO_MMIO_VENDOR_ID: usize = 0x00c;
pub(self) const VIRTIO_MMIO_DEVICE_FEATURES: usize = 0x010;
pub(self) const VIRTIO_MMIO_DRIVER_FEATURES: usize = 0x020;
pub(self) const VIRTIO_MMIO_QUEUE_SEL: usize = 0x030;
pub(self) const VIRTIO_MMIO_QUEUE_NUM_MAX: usize = 0x034;
pub(self) const VIRTIO_MMIO_QUEUE_NUM: usize = 0x038;
pub(self) const VIRTIO_MMIO_QUEUE_READY: usize = 0x044;
pub(self) const VIRTIO_MMIO_QUEUE_NOTIFY: usize = 0x050;
pub(self) const VIRTIO_MMIO_INTERRUPT_STATUS: usize = 0x060;
pub(self) const VIRTIO_MMIO_INTERRUPT_ACK: usize = 0x064;
pub(self) const VIRTIO_MMIO_STATUS: usize = 0x070;
pub(self) const VIRTIO_MMIO_QUEUE_DESC_LOW: usize = 0x080;
pub(self) const VIRTIO_MMIO_QUEUE_DESC_HIGH: usize = 0x084;
pub(self) const VIRTIO_MMIO_DRIVER_DESC_LOW: usize = 0x090;
pub(self) const VIRTIO_MMIO_DRIVER_DESC_HIGH: usize = 0x094;
pub(self) const VIRTIO_MMIO_DEVICE_DESC_LOW: usize = 0x0a0;
pub(self) const VIRTIO_MMIO_DEVICE_DESC_HIGH: usize = 0x0a4;

pub(self) const VIRTIO0: usize = 0x1000_1000;

pub struct VirtioDev {
    base: usize,
}

impl VirtioDev {
    const fn new(base: usize) -> Self {
        VirtioDev { base }
    }

    fn write(&self, offset: usize, val: u32) {
        let reg = (self.base + offset) as *mut u32;
        unsafe {
            reg.write_volatile(val);
        }
    }

    fn read(&self, offset: usize) -> u32 {
        let reg = (self.base + offset) as *mut u32;
        unsafe { reg.read_volatile() }
    }
}

pub(self) const VIRTIO_CONFIG_S_ACKNOWLEDGE: u32 = 1;
pub(self) const VIRTIO_CONFIG_S_DRIVER: u32 = 1 << 1;
pub(self) const VIRTIO_CONFIG_S_DRIVER_OK: u32 = 1 << 2;
pub(self) const VIRTIO_CONFIG_S_FEATURES_OK: u32 = 1 << 3;

/* Device supports request barriers */
#[allow(dead_code)]
pub(self) const VIRTIO_BLK_F_BARRIER: u32 = 1;
/* Maximum size of any single segment is in size_max */
#[allow(dead_code)]
pub(self) const VIRTIO_BLK_F_SIZE_MAX: u32 = 1 << 1;
/* Maximum number of segments in a request is in seg_max. */
#[allow(dead_code)]
pub(self) const VIRTIO_BLK_F_SEG_MAX: u32 = 1 << 2;
/* Disk-style geometry specified in geometry. */
#[allow(dead_code)]
pub(self) const VIRTIO_BLK_F_GEOMETRY: u32 = 1 << 4;
/* Device is read-only */
pub(self) const VIRTIO_BLK_F_RO: u32 = 1 << 5;
/* Block size of disk is in blk_size. */
#[allow(dead_code)]
pub(self) const VIRTIO_BLK_F_BLK_SIZE: u32 = 1 << 6;
/* Device supports scsi packet commands */
#[allow(dead_code)]
pub(self) const VIRTIO_BLK_F_SCSI: u32 = 1 << 7;
/* Cache flush command support */
#[allow(dead_code)]
pub(self) const VIRTIO_BLK_F_FLUSH: u32 = 1 << 9;
/* Device exports information on optimal I/O alignment */
#[allow(dead_code)]
pub(self) const VIRTIO_BLK_F_TOPOLOGY: u32 = 1 << 10;
/* Device can toggle its cache between writeback and writethrough modes */
pub(self) const VIRTIO_BLK_F_CONFIG_WCE: u32 = 1 << 11;
/* Support more than one vq */
pub(self) const VIRTIO_BLK_F_MQ: u32 = 1 << 12;
/* Device can support discard command, maximum discard sectors size in
 * max_discard_sectors and maximum discard segment number in max_discard_seg */
#[allow(dead_code)]
pub(self) const VIRTIO_BLK_F_DISCARD: u32 = 1 << 13;
/* Device can support write zeroes command, maximum write zeroes sectors
 * size in max_write_zeroes_sectors and maximum write zeroes segment
 * number in max_write_zeroes_seg */
#[allow(dead_code)]
pub(self) const VIRTIO_BLK_F_WRITE_ZEROES: u32 = 1 << 14;
/* Secure Erase is supported */
#[allow(dead_code)]
pub(self) const VIRTIO_BLK_F_SECURE_ERASE: u32 = 1 << 16;
/* Zoned block device */
#[allow(dead_code)]
pub(self) const VIRTIO_BLK_F_ZONED: u32 = 1 << 17;

pub(self) const VIRTIO_BLK_S_OK: u8 = 0;

/* If this feature has been negotiated by driver, the device MUST issue a used
 * buffer notification if the device runs out of available descriptors on a virtqueue,
 * even though notifications are suppressed using the VIRTQ_AVAIL_F_NO_INTERRUPT flag
 * or the used_event field. Note: An example of a driver using this feature is the
 * legacy networking driver: it doesn’t need to know every time a packet is transmitted,
 * but it does need to free the transmitted packets a finite time after they are
 * transmitted. It can avoid using a timer if the device notifies it when all the packets
 * are transmitted. */
#[allow(dead_code)]
pub(self) const VIRTIO_F_NOTIFY_ON_EMPTY: u32 = 1 << 24;
/* This feature indicates that the device accepts arbitrary descriptor layouts */
pub(self) const VIRTIO_F_ANY_LAYOUT: u32 = 1 << 27;
/* We support indirect buffer descriptors */
pub(self) const VIRTIO_RING_F_INDIRECT_DESC: u32 = 1 << 28;
/* - The Guest publishes the used index for which it expects an interrupt
 * at the end of the avail ring. Host should ignore the avail->flags field.
 * - The Host publishes the avail index for which it expects a kick
 * at the end of the used ring. Guest should ignore the used->flags field. */
pub(self) const VIRTIO_RING_F_EVENT_IDX: u32 = 1 << 29;

pub(self) const VIRTIO_BLK_T_IN: u32 = 0;
pub(self) const VIRTIO_BLK_T_OUT: u32 = 1;

pub(self) const VRING_DESC_F_NEXT: u16 = 1;
pub(self) const VRING_DESC_F_WRITE: u16 = 2;
