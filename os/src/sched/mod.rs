use crate::cpu;
use crate::lock::Locked;
use crate::sched::context::TaskContext;
use crate::sched::scheduler::Scheduler;
use crate::sched::task::Task;
use lazy_static::lazy_static;

use self::context::TrapFrame;

mod context;
mod scheduler;
mod task;

extern "C" {
    fn switch_to(prev: *mut TaskContext, cur: *mut TaskContext);
    fn delay(count: usize);
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
    loop {
        println!("initd started");
        unsafe {
            delay(300000000);
        }
    }
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
    extern "C" {
        fn write();
    }
    loop {
        unsafe {
            //write();
        }
    }
}

pub fn init() {
    SCHEDULER.lock().kspawn(initd);
    SCHEDULER.lock().kspawn(exit);
    SCHEDULER.lock().uspawn(user);
}

pub fn current() -> *mut Task {
    let cur = SCHEDULER.lock().current();
    assert!(!cur.is_null());
    cur
}

pub fn current_frame() -> *mut TrapFrame {
    let cur = current();
    let frame;
    unsafe {
        frame = (*cur).frame();
    }
    assert!(!frame.is_null());
    frame
}

pub fn scheduler() {
    loop {
        /* Since scheduler could be executed after timer interrupt, we
         * need to avoid deadlock by enabling the interrupt again */
        cpu::intr_on();

        let mut scheduler_lock = SCHEDULER.try_lock();
        let cur;
        if let Some(ref mut scheduler) = scheduler_lock {
            if let Some(task) = scheduler.pick_next() {
                cur = task.task_context();
            } else {
                panic!("We don't expect failing to pick the task");
            }
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
        if let Some(task) = scheduler.put_prev() {
            prev = task.task_context();
        } else {
            // Don't sched if we are not at the task context
            return;
        }
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
