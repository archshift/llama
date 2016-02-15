use cpu;
use cpu::Cpu;
use ram;

pub fn ldr_1(cpu: &mut Cpu, ram: &ram::Ram, data: cpu::ThumbInstrLDR_1) -> u32 {
	use cpu::ThumbInstrLDR_1 as ThumbInstr;

	let base_val = cpu.regs[data.get(ThumbInstr::rn()) as usize];
	let immed_5 = data.get(ThumbInstr::immed_5()) as u32;

	let addr = base_val + immed_5 * 4;
	// TODO: determine behavior based on CP15 r1 bit_U (22)
	cpu.regs[data.get(ThumbInstr::rd()) as usize] = ram.read::<u32>(addr);

	2
}

pub fn ldr_3(cpu: &mut Cpu, ram: &ram::Ram, data: cpu::ThumbInstrLDR_3) -> u32 {
	use cpu::ThumbInstrLDR_3 as ThumbInstr;

	let immed_8 = data.get(ThumbInstr::immed_8()) as u32;
	let addr = (cpu.regs[15] & 0xFFFFFFFC) + immed_8 * 4;
	cpu.regs[data.get(ThumbInstr::rd()) as usize] = ram.read::<u32>(addr);

	2
}
