use cpu;
use ram;

// Program status register
create_bitfield!(Psr: u32, {
    mode: 0 => 4,
    thumb_bit: 5 => 5,
    disable_fiq_bit: 6 => 6,
    disable_irq_bit: 7 => 7,
    q_bit: 27 => 27,
    v_bit: 28 => 28,
    c_bit: 29 => 29,
    z_bit: 30 => 30,
    n_bit: 31 => 31
});

pub struct Cpu {
    pub regs: [u32; 16],
    pub cpsr: Psr::Type,
    pub spsr: [Psr::Type; 5],
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            regs: [0; 16],
            cpsr: Psr::new(0),
            spsr: [
                Psr::new(0), Psr::new(0), Psr::new(0),
                Psr::new(0), Psr::new(0)
            ],
        }
    }

    pub fn reset(&mut self, entry: u32) {
        self.regs[15] = entry + self.get_pc_offset();
    }

    pub fn get_pc_offset(&self) -> u32 {
        if self.cpsr.get::<Psr::thumb_bit>() == 1 {
            4
        } else {
            8
        }
    }

    pub fn select_saved_psr(&mut self) {

    }

    pub fn branch(&mut self, addr: u32) {

    }

    pub fn run(&mut self, mut ram: &mut ram::Ram) {
        loop {
            let addr = self.regs[15] - self.get_pc_offset();
            let encoding = ram.read::<u32>(addr);

            if self.cpsr.get::<Psr::thumb_bit>() == 0 {
                let instr = cpu::decode_arm_instruction(encoding);
                cpu::interpret_arm(self, instr);
            } else {
                panic!("Thumb not supported!");
            }
        }
    }
}
