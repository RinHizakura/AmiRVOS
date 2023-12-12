use crate::sched;

mod proc;

/* FIXME: Note: These syscall numbers should match syscall.asm.
 * How can we check the consistency automatically? */
const SYS_WRITE: usize = 0;

pub fn syscall_handler() {
    let frame = sched::current_frame();
    // a7 is the number of syscall
    let syscall_num = unsafe { (*frame).get_a(7) };

    warning!("SYSCALL {}", syscall_num);

    let result = match syscall_num {
        SYS_WRITE => proc::sys_write(),
        _ => panic!(),
    };

    unsafe {
        (*frame).set_a(0, result);
    }
}

pub(self) fn syscall_args(n: usize) -> usize {
    let frame = sched::current_frame();
    unsafe { (*frame).get_a(n) }
}
