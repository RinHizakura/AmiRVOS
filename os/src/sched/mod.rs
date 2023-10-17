use spin::Mutex;
use crate::sched::scheduler::Scheduler;
use lazy_static::lazy_static;

pub mod task;
pub mod scheduler;

lazy_static! {
    static ref SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());
}

pub fn init() {
    let tid = SCHEDULER.lock().spawn();
    info!("Create a new task with id {}", tid.0);
}
