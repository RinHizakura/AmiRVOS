mod proc;

/* FIXME: Note: These syscall numbers should match syscall.asm.
 * How can we check the consistency automatically? */
const SYS_write: usize = 0;

pub fn syscall_handler(syscall_num: usize) -> isize {
    warning!("SYSCALL {}", syscall_num);

    match syscall_num {
        SYS_write => proc::sys_write(),
        _ => panic!(),
    };

    todo!();
}
