use cpu::coproc::Coprocessor;

pub struct SysControl {

}

impl SysControl {
    pub fn new() -> SysControl {
        SysControl {}
    }
}

impl Coprocessor for SysControl {
    fn move_in(&mut self, cpreg1: usize, cpreg2: usize, op1: usize, op2: usize, val: u32) {
        panic!("Unimplemented CP15 operation to coproc reg {}", cpreg1);
    }

    fn move_out(&mut self, cpreg1: usize, cpreg2: usize, op1: usize, op2: usize) -> u32 {
        panic!("Unimplemented CP15 operation from coproc reg {}", cpreg1);
    }
}