use crate::sched;

mod proc;

/* FIXME: Note: These syscall numbers should match syscall.asm.
 * How can we check the consistency automatically? */
const SYS_OPEN: usize = 0;
const SYS_CLOSE: usize = 1;
const SYS_READ: usize = 2;
const SYS_WRITE: usize = 3;

pub fn syscall_handler() {
    let frame = sched::current_frame();
    // a7 is the number of syscall
    let syscall_num = unsafe { (*frame).get_a(7) };

    warning!("SYSCALL {}", syscall_num);

    let result = match syscall_num {
        SYS_OPEN => proc::sys_open(),
        SYS_WRITE => proc::sys_write(),
        _ => panic!("Unknown syscall {}", syscall_num),
    };

    unsafe {
        (*frame).set_a(0, result);
    }
}

pub(self) fn syscall_args(n: usize) -> usize {
    let frame = sched::current_frame();
    unsafe { (*frame).get_a(n) }
}
