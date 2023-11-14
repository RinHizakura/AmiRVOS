use crate::lock::Locked;
use crate::sched::scheduler::Scheduler;
use lazy_static::lazy_static;

pub mod scheduler;
pub mod task;

extern "C" {
    fn switch_to(frame: usize, satp: usize, mode: usize) -> !;
}

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
#[repr(align(4096))]
pub extern "C" fn user() {
    //println!("Hello");

    loop {}
}

pub fn init() {
    SCHEDULER.lock().kspawn(initd);
    SCHEDULER.lock().uspawn(user);
}

pub fn schedule() {
    let binding = SCHEDULER.try_lock();

    /* FIXME: The scheduler is locked probably because we have a
     * task which is going to exit. In such case, just simply give
     * CPU to that task since it is almost done.
     *
     * This is somehow unfair and we should consider not to do this in
     * the future. */
    let mut args = None;

    if let Some(mut binding) = binding {
        while let Some(pick) = binding.pick_next() {
            args = Some((pick.frame(), pick.satp(), pick.mode()));
            break;
        }
    }

    if let Some(args) = args {
        unsafe {
            switch_to(args.0, args.1, args.2);
        }
    }
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
