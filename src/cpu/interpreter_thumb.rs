use cpu::{Cpu, ThumbInstruction, Psr};
use ram;

#[inline(always)]
pub fn interpret_thumb(cpu: &mut Cpu, mut ram: &mut ram::Ram, instr: ThumbInstruction) {
    println!("Instruction {:#X}: {:?}", cpu.regs[15] - cpu.get_pc_offset(), instr);

    let bytes_advanced = match instr {
        // ThumbInstruction::MOV_1(data) => instructions_thumb::mov_1(cpu, data),
        _ => {
            // println!("Unimplemented instruction! {:#X}: {:?}", cpu.regs[15] - cpu.get_pc_offset(), instr);
            2
        }
    };
    cpu.regs[15] += bytes_advanced;
}
