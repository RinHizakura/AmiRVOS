/* A simple ringbuf */
use core::marker::Copy;

/* N must be power of two to guarantee the result */
pub struct RingBuf<T, const N: usize> {
    buf: [T;N],
    head: usize,
    tail: usize,
}

impl<T: Default + Copy, const N: usize> RingBuf<T,N> {
    const OK: () = assert!(N & (N - 1) == 0, "N for RingBuf should be power of two");

    pub fn new() -> Self {
        let _ = RingBuf::<T,N>::OK;
        RingBuf {
            buf: [T::default();N],
            head: 0,
            tail: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head == self.tail
    }

    pub fn is_full(&self) -> bool {
        self.tail - self.head > (N - 1)
    }

    pub fn push(&mut self, v: T) {
        if self.is_full() {
            // drop the value if the buffer is full
            return;
        }
        self.buf[self.tail & (N - 1)] = v;
        self.tail += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        let v = self.buf[self.head & (N - 1)];
        self.head += 1;
        Some(v)
    }
}
