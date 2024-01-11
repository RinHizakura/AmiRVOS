/* Note: take care of file 'trap.asm' and 'sched.asm' if we
 * want to change this structure's layout. */
#[repr(C)]
pub struct TrapFrame {
    pub regs: [usize; 32],  // 0 ~ 255: x1 to x32 registers
    pub kernel_satp: usize, // 256: satp
    pub kernel_trap: usize, // 264: trap handler
    pub kernel_sp: usize,   // 272: sp
    pub epc: usize,         // 280: epc
}

impl TrapFrame {
    pub fn set_sp(&mut self, val: usize) {
        self.regs[2] = val;
    }

    pub fn get_a(&mut self, n: usize) -> usize {
        match n {
            0 => self.regs[10],
            1 => self.regs[11],
            2 => self.regs[12],
            3 => self.regs[13],
            4 => self.regs[14],
            5 => self.regs[15],
            6 => self.regs[16],
            7 => self.regs[17],
            _ => panic!("Invalid to get argument register {}", n),
        }
    }

    pub fn set_a(&mut self, n: usize, val: usize) {
        match n {
            0 => self.regs[10] = val,
            1 => self.regs[11] = val,
            2 => self.regs[12] = val,
            3 => self.regs[13] = val,
            4 => self.regs[14] = val,
            5 => self.regs[15] = val,
            6 => self.regs[16] = val,
            7 => self.regs[17] = val,
            _ => panic!("Invalid to set argument register {}", n),
        };
    }
}

#[repr(C)]
#[derive(Default)]
pub struct TaskContext {
    pub ra: usize,
    pub sp: usize,

    pub s0: usize,
    pub s1: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,
}

#[repr(C)]
pub struct Context {
    pub trapframe: TrapFrame,
    pub task_ctx: TaskContext,
}
