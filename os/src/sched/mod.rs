use crate::config::{TRAMPOLINE_VA, TRAPFRAME_VA};
use crate::lock::Locked;
use crate::sched::scheduler::Scheduler;
use crate::trap::kernel_trapframe;
use core::mem;
use lazy_static::lazy_static;

pub mod scheduler;
pub mod task;

extern "C" {
    fn switch_to(frame: usize);
}

lazy_static! {
    static ref SCHEDULER: Locked<Scheduler> = Locked::new(Scheduler::new());
}

pub extern "C" fn initd() {
    /* Since scheduler will loop until it find an executable task, we
     * make the init task alive as long as the OS running. */
    println!("initd started");
    do_yield();
    loop {}
}

pub extern "C" fn exit() {
    /* TODO: Create a task that exit directly, so that we can
     * check that if kernel reclaims it correctly. */
    println!("exit");
    loop {}
}

/* TODO: This should be implement elsewhere and put on disk
 * image. Now we just embed it in kernel image for simply
 * testing. */
#[link_section = ".text.user.main"]
pub extern "C" fn user() {
    println!("Hi user");
    loop {}
}

pub fn init() {
    SCHEDULER.lock().kspawn(initd);
    SCHEDULER.lock().kspawn(exit);
    SCHEDULER.lock().kspawn(user);
}

macro_rules! cast_func {
    ($address:expr, $t:ty) => {
        mem::transmute::<*const (), $t>($address as _)
    };
}

pub fn scheduler() {
    loop {
        let binding = SCHEDULER.try_lock();

        /* FIXME: The scheduler is locked probably because we have a
         * task which is going to exit. In such case, just simply give
         * CPU to that task since it is almost done.
         *
         * This is somehow unfair and we should consider not to do this in
         * the future. */
        let mut cur = None;

        if let Some(mut binding) = binding {
            while let Some(pick) = binding.pick_next() {
                cur = Some(pick.frame());
                break;
            }
        }

        if let Some(cur) = cur {
            unsafe {
                switch_to(cur);
            }
        }
    }
}

pub fn do_sched() {
    /* Switch back to the kernel context, which
     * should be the scheduler */
    unsafe {
        switch_to(kernel_trapframe());
    }
}

pub fn do_yield() {
    SCHEDULER.lock().put_prev();
    do_sched();
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
