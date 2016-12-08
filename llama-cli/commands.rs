use std::process::exit;

use libllama::dbgcore;

/// Prints memory to the screen based on provided address, number of bytes
/// Command format: "mem <start address hex> [# bytes hex]"
///
/// `args`: Iterator over &str items
fn cmd_mem<'a, It>(debugger: &mut dbgcore::DbgCore, mut args: It)
    where It: Iterator<Item=&'a str> {
    use common::from_hex;

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
    print!("{:02X}", hw.read_mem(start));
    for addr in (start + 1) .. (start + num) {
        print!(" {:02X}", hw.read_mem(addr));
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

/// Controls debugger behavior based on user-provided commands
///
/// `command`: Iterator over &str items
pub fn handle<'a, It>(debugger: &mut dbgcore::DbgCore, mut command: It) -> bool
    where It: Iterator<Item=&'a str> {
    let mut is_paused = true;

    match command.next() {
        Some("run") => { debugger.ctx().resume(); is_paused = false; },
        Some("mem") => cmd_mem(debugger, command),
        Some("reg") => cmd_reg(debugger, command),
        Some("quit") | Some("exit") => {
            debugger.ctx().hwcore_mut().stop();
            // TODO: Cleaner exit?
            exit(0);
        }
        None => println!("Error: No command"),
        Some(unk_cmd @ _) => println!("Error: Unrecognized command `{}`", unk_cmd),
    }

    return is_paused;
}