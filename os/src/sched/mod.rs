use crate::config::{TRAMPOLINE_VA, TRAPFRAME_VA};
use crate::lock::Locked;
use crate::sched::scheduler::Scheduler;
use core::mem;
use lazy_static::lazy_static;

pub mod scheduler;
pub mod task;

extern "C" {}

lazy_static! {
    static ref SCHEDULER: Locked<Scheduler> = Locked::new(Scheduler::new());
}

pub extern "C" fn initd() {
    /* Since scheduler will loop until it find an executable task, we
     * make the init task alive as long as the OS running. */
    println!("initd started");
    loop {}
}

/* TODO: This should be implement elsewhere and put on disk
 * image. Now we just embed it in kernel image for simply
 * testing. */
#[link_section = ".text.user.main"]
pub extern "C" fn user() {
    println!("Hi");
    loop {}
}

pub fn init() {
    SCHEDULER.lock().kspawn(initd);
    SCHEDULER.lock().kspawn(user);
}

macro_rules! cast_func {
    ($address:expr, $t:ty) => {
        mem::transmute::<*const (), $t>($address as _)
    };
}

pub fn scheduler() {
    println!("Start scheduling!");
    todo!();
}

/* TODO: Every task should end up here to make scheduler know
 * to drop the task out. Another strategy could be just leaved
 * every exit task as zombies and reap it by a certain process? */
pub extern "C" fn exit_task() {
    SCHEDULER.lock().cur_exit();
    /* FIXME: Insert a loop here to wait until got reclaimed. It means a task
     * could exhaust its time slice just for looping until the next schedule tick.
     * Any good idea to get reclaim directly? */
    loop {}
}
