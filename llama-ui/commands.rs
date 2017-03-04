use std::process::exit;

use libllama::dbgcore;

/// Prints disassembly for the next instruction
/// Command format: "asm"
///
/// `args`: Unused
fn cmd_asm<'a, It>(debugger: &mut dbgcore::DbgCore, mut args: It)
    where It: Iterator<Item=&'a str> {

    use capstone::{Capstone, CsArch, CsMode};
    let _ = args;

    let mut ctx = debugger.ctx();
    let hw = ctx.hw();

    let pause_addr = hw.pause_addr();
    let cpu_mode = if hw.is_thumb() {
        CsMode::MODE_THUMB
    } else {
        CsMode::MODE_LITTLE_ENDIAN
    };

    if let Some(cs) = Capstone::new(CsArch::ARCH_ARM, cpu_mode) {
        let mut inst_bytes = [0u8; 4];
        hw.read_mem(pause_addr, &mut inst_bytes);

        match cs.disasm(&inst_bytes, pause_addr as u64, 1) {
            Some(insts) => {
                let inst = insts.iter().next().unwrap();
                println!("{:X}: {} {}", pause_addr,
                                        inst.mnemonic().unwrap(),
                                        inst.op_str().unwrap())
            }
            None => println!("Error: failed to disassemble instruction at 0x{:X}", pause_addr),
        }
    } else {
        println!("Error: could not initialize capstone!");
    }
}

/// Adds CPU breakpoint at instruction address
/// Command format: "brk <address hex>"
///
/// `args`: Iterator over &str items
fn cmd_brk<'a, It>(debugger: &mut dbgcore::DbgCore, mut args: It)
    where It: Iterator<Item=&'a str> {
    use libllama::utils::from_hex;

    let addr_str = match args.next() {
        Some(arg) => from_hex(arg),
        None => { println!("Usage: `brk <addr>"); return }
    };

    // Check for from_hex errors
    let addr = match addr_str {
        Ok(x) => x,
        _ => { println!("Error: could not parse hex value!"); return }
    };

    info!("Toggling breakpoint at 0x{:X}", addr);

    let mut ctx = debugger.ctx();
    let mut hw = ctx.hw();

    if !hw.has_breakpoint(addr) {
        hw.set_breakpoint(addr);
    } else {
        hw.del_breakpoint(addr);
    }
}

/// Prints memory to the screen based on provided address, number of bytes
/// Command format: "mem <start address hex> [# bytes hex]"
///
/// `args`: Iterator over &str items
fn cmd_mem<'a, It>(debugger: &mut dbgcore::DbgCore, mut args: It)
    where It: Iterator<Item=&'a str> {
    use libllama::utils::from_hex;

    // Tuple: (u32: start, u32: num)
    let arg_res = match (args.next(), args.next()) {
        (Some(ss), Some(ns)) => from_hex(ss).and_then(|s| Ok((s, from_hex(ns)?))),
        (Some(ss), None) => from_hex(ss).and_then(|s| Ok((s, 1))),
        (None, _) => { println!("Usage: `mem <start> [num]"); return }
    };

    // Check for from_hex errors, validate `num` input
    let (start, num) = match arg_res {
        Ok((s, n)) if n > 0 => (s, n),
        Ok((s, _)) => (s, 1),
        _ => { println!("Error: could not parse hex value!"); return }
    };

    trace!("Printing {} bytes of RAM starting at 0x{:08X}", num, start);

    let mut ctx = debugger.ctx();
    let hw = ctx.hw();

    let mut mem_bytes = vec![0u8; num as usize];
    hw.read_mem(start, &mut mem_bytes);

    print!("{:02X}", mem_bytes[0]);
    for i in 1 .. num as usize {
        print!(" {:02X}", mem_bytes[i]);
    }
    println!("");
}

/// Prints registers to the screen based on provided register name
/// Command format: "reg [register name]"
///
/// `args`: Iterator over &str items
fn cmd_reg<'a, It>(debugger: &mut dbgcore::DbgCore, mut args: It)
    where It: Iterator<Item=&'a str> {
    let mut ctx = debugger.ctx();
    let hw = ctx.hw();

    let print_reg = |reg_num| println!("R{} = 0x{:08X}", reg_num, hw.read_reg(reg_num));

    let reg_str = match args.next() {
        Some(arg) => arg.to_owned().to_lowercase(),
        None => {
            for i in 0..16 {
                print_reg(i);
            }
            return;
        }
    };

    match reg_str.as_str() {
        "r0" => print_reg(0),
        "r1" => print_reg(1),
        "r2" => print_reg(2),
        "r3" => print_reg(3),
        "r4" => print_reg(4),
        "r5" => print_reg(5),
        "r6" => print_reg(6),
        "r7" => print_reg(7),
        "r8" => print_reg(8),
        "r9" => print_reg(9),
        "r10" => print_reg(10),
        "r11" => print_reg(11),
        "r12" => print_reg(12),
        "sp" | "r13" => print_reg(13),
        "lr" | "r14" => print_reg(14),
        "pc" | "r15" => print_reg(15),
        _ => println!("Error: Unrecognized register!"),
    }
}

/// Runs one instruction on the CPU
/// Command format: "step"
///
/// `args`: Unused
fn cmd_step<'a, It>(debugger: &mut dbgcore::DbgCore, mut args: It)
    where It: Iterator<Item=&'a str> {
    let _ = args;
    let mut ctx = debugger.ctx();
    let mut hw = ctx.hw();

    hw.step();
}

/// Controls debugger behavior based on user-provided commands
///
/// `command`: Iterator over &str items
pub fn handle<'a, It>(debugger: &mut dbgcore::DbgCore, mut command: It) -> bool
    where It: Iterator<Item=&'a str> {
    let mut is_paused = true;

    match command.next() {
        Some("run") => { debugger.ctx().resume(); is_paused = false; },
        Some("brk") => cmd_brk(debugger, command),
        Some("asm") => cmd_asm(debugger, command),
        Some("mem") => cmd_mem(debugger, command),
        Some("reg") => cmd_reg(debugger, command),
        Some("step") => cmd_step(debugger, command),
        Some("quit") | Some("exit") => {
            debugger.ctx().hwcore_mut().stop();
            // TODO: Cleaner exit?
            exit(0);
        }
        None => {},
        Some(unk_cmd @ _) => println!("Error: Unrecognized command `{}`", unk_cmd),
    }

    return is_paused;
}