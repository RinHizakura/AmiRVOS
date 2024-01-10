use crate::sched;

mod proc;

// https://elixir.bootlin.com/linux/latest/source/include/uapi/asm-generic/unistd.h
/* FIXME: Note: These syscall numbers should match syscall.asm.
 * How can we check the consistency automatically? */
const SYS_OPEN: usize = 56; // FIXME: 56 is for openat in fact
const SYS_CLOSE: usize = 57;
const SYS_READ: usize = 63;
const SYS_WRITE: usize = 64;
const SYS_MKNOD: usize = 33; // FIXME: 33 is for mknodat in fact

pub fn syscall_handler() {
    let frame = sched::current_frame();
    // a7 is the number of syscall
    let syscall_num = unsafe { (*frame).get_a(7) };

    warning!("SYSCALL {}", syscall_num);

    let result = match syscall_num {
        SYS_OPEN => proc::sys_open() as usize,
        SYS_WRITE => proc::sys_write() as usize,
        SYS_MKNOD => proc::sys_mknod() as usize,
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
