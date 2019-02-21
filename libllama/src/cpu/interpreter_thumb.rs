use cpu::{Cpu, InstrStatus, Version};
use cpu::caches::Ops;

pub type InstFn<V> = fn(&mut Cpu<V>, u16) -> InstrStatus;
mod interpreter {
    use cpu;
    pub use cpu::instructions_thumb::*;
    pub fn undef<V: cpu::Version>(cpu: &mut cpu::Cpu<V>, instr: u16) -> cpu::InstrStatus {
        panic!("Unimplemented instruction! {:#X}: {:?}", cpu.regs[15] - cpu.get_pc_offset(), instr)
    }
}

include!(concat!(env!("OUT_DIR"), "/thumb.decoder.rs"));

#[inline]
pub fn interpret_next<V: Version>(cpu: &mut Cpu<V>, addr: u32) -> InstrStatus {
    let instr = cpu.mpu.imem_read::<u16>(addr);
    let inst_fn = *cpu.thumb_decode_cache.get_or(instr as u32, &mut ());

    // trace!("THUMB{:?} @ {:08X}: {} ({:04X})", cpu._version, addr, ::cpu::thumb::disasm(instr), instr);
    inst_fn(cpu, instr)
}
