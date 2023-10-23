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

static mut FLAG: u8 = 0;

pub extern "C" fn initd1() {
    println!("initd started");
    unsafe {
        FLAG = 1;
    }
    loop {}
}

pub extern "C" fn initd2() {
    println!("initd2 started");
    unsafe { while FLAG == 0 {} }
    println!("initd2 end");
    loop {}
}

pub fn init() {
    SCHEDULER.lock().kspawn(initd1);
    SCHEDULER.lock().kspawn(initd2);
}

pub fn schedule() {
    let mut binding = SCHEDULER.lock();
    let pick = binding.pick_next();

    if let Some(pick) = pick {
        let frame = pick.frame();
        /* TODO: Unlock the lock manually to avoid deadlock. Any
         * prettier way to do this? */
        drop(binding);
        unsafe {
            switch_to(frame);
        }
    }
}
