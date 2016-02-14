use cpu;
use cpu::Cpu;
use ram;

#[inline(always)]
fn get_load_store_addr(instr_data: &cpu::InstrDataLoadStore::Type, cpu: &Cpu) -> u32 {
    use cpu::InstrDataLoadStore as InstrData;

    let c_bit = cpu.cpsr.get::<cpu::Psr::c_bit>() == 1;

    let i_bit = instr_data.get::<InstrData::i_bit>();
    let u_bit = instr_data.get::<InstrData::u_bit>();
    let base_addr = cpu.regs[instr_data.get::<InstrData::rn>() as usize];

    let offset = if i_bit == 0 {
        extract_bits!(instr_data.raw(), 0 => 11)
    } else {
        let pre_shift = cpu.regs[extract_bits!(instr_data.raw(), 0 => 3) as usize];
        let b_bit = instr_data.get::<InstrData::b_bit>();

        let offset = if extract_bits!(instr_data.raw(), 4 => 11) == 0 {
            pre_shift
        } else {
            let shift = extract_bits!(instr_data.raw(), 5 => 6);
            let shift_imm = extract_bits!(instr_data.raw(), 7 => 11);

            match shift {
                0b00 => pre_shift << shift_imm,
                0b01 => {
                    let index = if shift_imm == 0 {
                        0
                    } else {
                        pre_shift >> shift_imm
                    }; index
                },
                0b10 => {
                    let index = if shift_imm == 0 {
                        let index = if extract_bits!(pre_shift, 31 => 31) == 1 {
                            0xFFFFFFFF
                        } else {
                            0
                        }; index
                    } else {
                        ((pre_shift as i32) >> shift_imm) as u32
                    }; index
                },
                0b11 => {
                    let index = if shift_imm == 0 {
                        ((c_bit as u32) << 31) | (pre_shift >> 1)
                    } else {
                        pre_shift.rotate_right(shift_imm)
                    }; index
                }
                _ => {
                    panic!("Unhandled shifter operation!");
                }
            }
        }; offset
    };

    if u_bit == 1 {
        return base_addr + offset;
    } else {
        return base_addr - offset;
    }
}

#[inline(always)]
pub fn ldr(cpu: &mut Cpu, mut ram: &mut ram::Ram, data: cpu::InstrDataLoadStore::Type) -> u32 {
    use cpu::InstrDataLoadStore as InstrData;

    if !cpu::cond_passed(data.get::<InstrData::cond>(), &cpu.cpsr) {
        return 4;
    }

    let rd = data.get::<InstrData::rd>();
    let addr = get_load_store_addr(&data, cpu);
    // TODO: determine behavior based on CP15 r1 bit_U (22)
    let val = ram.read::<u32>(addr.rotate_right(8 * extract_bits!(addr, 0 => 1)));

    // TODO: Implement
    assert!(data.get::<InstrData::p_bit>() == 1);
    assert!(data.get::<InstrData::w_bit>() == 0);

    if rd == 15 {
        cpu.cpsr.set::<cpu::Psr::thumb_bit>(extract_bits!(val, 0 => 0));
        cpu.branch(val & 0xFFFFFFFE);
        return 0;
    } else {
        cpu.regs[rd as usize] = val;
    }

    4
}
