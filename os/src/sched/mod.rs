use crate::config::{TRAMPOLINE_VA, TRAPFRAME_VA};
use crate::lock::Locked;
use crate::sched::context::TaskContext;
use crate::sched::scheduler::Scheduler;
use core::mem;
use lazy_static::lazy_static;

mod context;
mod scheduler;
mod task;

extern "C" {
    fn switch_to(prev: *mut TaskContext, cur: *mut TaskContext);
}

lazy_static! {
    static ref SCHEDULER: Locked<Scheduler> = Locked::new(Scheduler::new());
    static ref TASK_CONTEXT: TaskContext = TaskContext::new();
}

fn kernel_task_context() -> *mut TaskContext {
    &*TASK_CONTEXT as *const TaskContext as *mut TaskContext
}

pub extern "C" fn initd() {
    /* Since scheduler will loop until it find an executable task, we
     * make the init task alive as long as the OS running. */
    println!("initd started");
    do_sched();
    loop {}
}

pub extern "C" fn exit() {
    /* TODO: Create a task that exit directly, so that we can
     * check that if kernel reclaims it correctly. */
    println!("exit");
    do_sched();
    loop {}
}

/* TODO: This should be implement elsewhere and put on disk
 * image. Now we just embed it in kernel image for simply
 * testing. */
#[link_section = ".text.user.main"]
pub extern "C" fn user() {
    println!("Hi user");
    do_sched();
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
        let mut scheduler_lock = SCHEDULER.try_lock();
        let cur;
        if let Some(ref mut scheduler) = scheduler_lock {
            let task = scheduler.pick_next();
            cur = task.task_context();
        } else {
            panic!("Fail to get scheduler lock for scheduler()");
        }
        /* We need to release the lock manually because switch_to()
         * may not return to scheduler() directly. */
        drop(scheduler_lock);

        unsafe {
            switch_to(kernel_task_context(), cur);
        }
    }
}

pub fn do_sched() {
    let mut scheduler_lock = SCHEDULER.try_lock();

    let prev;
    if let Some(ref mut scheduler) = scheduler_lock {
        prev = scheduler.put_prev().task_context();
    } else {
        panic!("Fail to get scheduler lock for do_sched()");
    }
    /* We need to release the lock manually because switch_to()
     * may not return to do_sched() directly. */
    drop(scheduler_lock);

    /* Switch back to the kernel context, which
     * should be the scheduler */
    unsafe {
        switch_to(prev, kernel_task_context());
    }
}
