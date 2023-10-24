use crate::lock::Locked;
use crate::sched::scheduler::Scheduler;
use lazy_static::lazy_static;

pub mod scheduler;
pub mod task;

extern "C" {
    fn switch_to(frame: usize) -> !;
}

lazy_static! {
    static ref SCHEDULER: Locked<Scheduler> = Locked::new(Scheduler::new());
}

pub extern "C" fn initd() {
    /* Since scheduler will loop until it find an executable task, we
     * make the init task alive as long as the OS running. */
    loop {
        println!("initd started");
    }
}

pub fn init() {
    SCHEDULER.lock().kspawn(initd);
}

pub fn schedule() {
    let mut binding = SCHEDULER.lock();

    while let Some(pick) = binding.pick_next() {
        let frame = pick.frame();
        /* TODO: Unlock the lock manually to avoid deadlock. Any
         * prettier way to do this? */
        drop(binding);
        unsafe {
            switch_to(frame);
        }
    }
}
