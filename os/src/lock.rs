use crate::cpu;

// A wrapper around spin::Mutex to permit trait implementations.
pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

/* FIXME: Please note that if what you are looking for is the correctness of
 * synchronization, you should always use acquire() and release() pairs
 * to meet the purpose, which will better consider the deadlock issue.
 * You will only use lock() or try_lock() explicitly when knowing the fact
 * that there's no race in the using context. For example, some strutures are
 * wrapped with lock just because they are static mutable data. Even though
 * we don't really access them on different thread, we still need to apply
 * synchronization mechanism on them for Rust's safety check. This design may
 * be confusing, and we should consider to improve it in the future. */
impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }

    pub fn try_lock(&self) -> Option<spin::MutexGuard<A>> {
        self.inner.try_lock()
    }

    pub fn acquire(&self) -> spin::MutexGuard<A> {
        /* Disable interrupts to avoid deadlock
         * TODO: Consider the case for multiple acquire/release, we
         * need to match the times of operation. For example, after
         * two acquirement for lock, we should enable the interrupt until
         * two release. */
        cpu::intr_off();

        let mut binding;
        loop {
            binding = self.try_lock();
            if binding.is_some() {
                break;
            }
        }
        binding.unwrap()
    }

    pub fn release(&self, binding: spin::MutexGuard<A>) {
        drop(binding);

        /* TODO: see acquire */
        cpu::intr_on();
    }
}
