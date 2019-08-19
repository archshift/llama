use std::process::exit;
use std::fs::File;
use std::io::Write;

use libllama::dbgcore::{self, ActiveCpu};
use libllama::utils::from_hex;

/// Prints disassembly for the next instruction
/// Command format: "asm [address hex]"
///
/// `args`: Iterator over &str items
fn cmd_asm<'a, It>(active_cpu: ActiveCpu, debugger: &mut dbgcore::DbgCore, mut args: It)
    where It: Iterator<Item=&'a str> {

    use capstone::Capstone;
    use capstone::arch::BuildsCapstone;
    use capstone::arch::arm::ArchMode;
    let _ = args;

    let mut ctx = debugger.ctx(active_cpu);
    let mut hw = ctx.hw();

    let pause_addr = match args.next().map(from_hex) {
        Some(Ok(x)) => x,
        Some(Err(_)) => { error!("Could not parse hex value!"); return }
        None => hw.pause_addr(),
    };

    let cpu_mode = if hw.is_thumb() {
        ArchMode::Thumb
    } else {
        ArchMode::Arm
    };

    let cs = Capstone::new()
        .arm()
        .mode(cpu_mode)
        .build();

    if let Ok(cs) = cs {
        let mut inst_bytes = [0u8; 4];
        if let Err(e) = hw.read_mem(pause_addr, &mut inst_bytes) {
            error!("{}", e);
            return;
        }

        match cs.disasm_count(&inst_bytes, pause_addr as u64, 1) {
            Ok(insts) => {
                let inst = insts.iter().next().unwrap();
                info!("{:X}: {} {}", pause_addr,
                                     inst.mnemonic().unwrap(),
                                     inst.op_str().unwrap())
            }
            Err(_) => error!("Failed to disassemble instruction at 0x{:X}", pause_addr),
        }
    } else {
        error!("Could not initialize capstone!");
    }
}

/// Adds CPU breakpoint at instruction address
/// Command format: "brk <address hex>"
///
/// `args`: Iterator over &str items
fn cmd_brk<'a, It>(active_cpu: ActiveCpu, debugger: &mut dbgcore::DbgCore, mut args: It)
    where It: Iterator<Item=&'a str> {

    let addr_str = match args.next() {
        Some(arg) => from_hex(arg),
        None => { info!("Usage: `brk <addr>"); return }
    };

    // Check for from_hex errors
    let addr = match addr_str {
        Ok(x) => x,
        _ => { error!("Could not parse hex value!"); return }
    };

    info!("Toggling breakpoint at 0x{:X}", addr);

    let mut ctx = debugger.ctx(active_cpu);
    let mut hw = ctx.hw();

    if !hw.has_breakpoint(addr) {
        hw.set_breakpoint(addr);
    } else {
        hw.del_breakpoint(addr);
    }
}

/// Toggles or displays button state
/// Command format: "btn [button name] [up/down]"
///
/// `args`: Iterator over &str items
fn cmd_btn<'a, It>(_active_cpu: ActiveCpu, debugger: &mut dbgcore::DbgCore, mut args: It)
    where It: Iterator<Item=&'a str> {
    use libllama::io::hid;

    let mut ctx = debugger.ctx(ActiveCpu::Arm11);
    let hw = ctx.hw11();
    let io_shared = &hw.io_shared_devices().hid;

    let btn_map = [
        ("a", hid::Button::A),
        ("b", hid::Button::B),
        ("x", hid::Button::X),
        ("y", hid::Button::Y),
        ("l", hid::Button::L),
        ("r", hid::Button::R),
        ("up", hid::Button::Up),
        ("down", hid::Button::Down),
        ("left", hid::Button::Left),
        ("right", hid::Button::Right),
        ("start", hid::Button::Start),
        ("select", hid::Button::Select)
    ];
    let mut btn_map = btn_map.iter();

    if let Some(button) = args.next() {
        let press = match args.next() {
            Some("up") => hid::ButtonState::Released,
            Some("down") => hid::ButtonState::Pressed,
            _ => {
                error!("Specify whether button `{}` should be `up`/`down`", button);
                return
            }
        };

        if let Some((_, btn)) = btn_map.find(|tup| button.eq_ignore_ascii_case(tup.0)) {
            hid::update_pad(&mut io_shared.lock(), press(*btn));
        } else {
            error!("Button `{}` does not exist!", button);
        }
    } else {
        let pad = hid::pad(&mut io_shared.lock());
        let mut pressed = Vec::new();

        for (label, btn) in btn_map {
            if pad & (1 << (*btn as usize)) != 0 {
                pressed.push(label);
            }
        }

        info!("Pressed buttons: {:?}", pressed);
    }
}

/// Dumps framebuffer to file
/// Command format: "fbdmp"
///
/// `args`: Unused
fn cmd_fbdmp<'a, It>(active_cpu: ActiveCpu, debugger: &mut dbgcore::DbgCore, _: It)
    where It: Iterator<Item=&'a str> {

    use libllama::io::gpu;

    let mut ctx = debugger.ctx(active_cpu);
    let fb_state = {
        let hw = ctx.hw11();
        let gpu = &hw.io11_devices().gpu;
        let fb_state = gpu::fb_state(&*gpu.borrow());
        fb_state
    };

    let mut fbs = libllama::hwcore::Framebuffers::default();
    ctx.hwcore().copy_framebuffers(&mut fbs, &fb_state);

    info!("Dumping framebuffers to disk in CWD...");

    let mut top = File::create("./fb-top.bin")
        .expect("Could not create fb-top.bin file!");
    top.write_all(fbs.top_screen.as_slice())
        .expect("Could not write top framebuffer!");

    let mut bot = File::create("./fb-bot.bin")
        .expect("Could not create fb-bot.bin file!");
    bot.write_all(fbs.bot_screen.as_slice())
        .expect("Could not write bottom framebuffer!");
}

/// Sets AES key-dumping state
/// Command format: "keydmp"
///
/// `args`: Unused
fn cmd_keydmp<'a, It>(active_cpu: ActiveCpu, debugger: &mut dbgcore::DbgCore, _: It)
    where It: Iterator<Item=&'a str> {

    use libllama::io::aes;

    let mut ctx = debugger.ctx(active_cpu);
    let hw = ctx.hw9();
    let key_slots = {
        let aes = &hw.io9_devices().aes;
        aes::dump_keys(&*aes.borrow())
    };

    info!("Dumping AES keys to disk...");

    use libllama::fs;
    fs::create_file(fs::LlamaFile::AesKeyDb, |file| {
        for k in key_slots.iter() {
            if let Err(x) = file.write_all(&k.data) {
                error!("Failed to write to aeskeydb file; {:?}", x);
                return
            }
        }
    }).unwrap();
}

/// Triggers the specified IRQ
/// Command format: "irq <type>"
///
/// `args`: Iterator over &str items
fn cmd_irq<'a, It>(active_cpu: ActiveCpu, debugger: &mut dbgcore::DbgCore, mut args: It)
    where It: Iterator<Item=&'a str> {

    let irq_ty = match args.next() {
        Some(arg) => arg.to_lowercase(),
        None => { info!("Usage: `irq <type>"); return }
    };

    let irq = match irq_ty.as_str() {
        "timer0" => dbgcore::IrqType9::Timer0,
        "timer1" => dbgcore::IrqType9::Timer1,
        "timer2" => dbgcore::IrqType9::Timer2,
        "timer3" => dbgcore::IrqType9::Timer3,
        _ => { error!("Unimplemented/unknown IRQ type `{}`", irq_ty); return }
    };

    info!("Triggering IRQ {}", irq_ty);

    let mut ctx = debugger.ctx(active_cpu);
    ctx.trigger_irq(irq);
}

/// Prints memory to the screen based on provided address, number of bytes
/// Command format: "mem <start address hex> [# bytes hex]"
///
/// `args`: Iterator over &str items
fn cmd_mem<'a, It>(active_cpu: ActiveCpu, debugger: &mut dbgcore::DbgCore, mut args: It)
    where It: Iterator<Item=&'a str> {

    // Tuple: (u32: start, u32: num)
    let arg_res = match (args.next(), args.next()) {
        (Some(ss), Some(ns)) => from_hex(ss).and_then(|s| Ok((s, from_hex(ns)?))),
        (Some(ss), None) => from_hex(ss).and_then(|s| Ok((s, 1))),
        (None, _) => { info!("Usage: `mem <start> [num] [outfile.bin]"); return }
    };

    // Check for from_hex errors, validate `num` input
    let (start, num) = match arg_res {
        Ok((s, n)) if n > 0 => (s, n),
        Ok((s, _)) => (s, 1),
        _ => { error!("Could not parse hex value!"); return }
    };

    trace!("Printing {} bytes of RAM starting at 0x{:08X}", num, start);

    let mut ctx = debugger.ctx(active_cpu);
    let mut hw = ctx.hw();

    let mut mem_bytes = vec![0u8; num as usize];
    if let Err(e) = hw.read_mem(start, &mut mem_bytes) {
        error!("{}", e);
        return;
    } else {
        let mut strbuf = String::new();
        strbuf.push_str(&format!("{:02X}", mem_bytes[0]));
        for i in 1 .. num as usize {
            strbuf.push_str(&format!(" {:02X}", mem_bytes[i]));
        }
        info!("{}", &strbuf);
    }

    if let Some(filename) = args.next() {
        let file = File::create(filename);
        let mut file = match file {
            Ok(file) => file,
            Err(e) => {
                error!("Unable to open file `{}` for dumping memory: {:?}!", filename, e);
                return;
            }
        };

        if let Err(e) = file.write_all(mem_bytes.as_slice()) {
            error!("Unable to write into file `{}`: {:?}", filename, e);
            return;
        }
        info!("Wrote 0x{:X} bytes to `{}`", num, filename);
    }
}

/// Prints registers to the screen based on provided register name
/// Command format: "reg [register name]"
///
/// `args`: Iterator over &str items
fn cmd_reg<'a, It>(active_cpu: ActiveCpu, debugger: &mut dbgcore::DbgCore, mut args: It)
    where It: Iterator<Item=&'a str> {
    let mut ctx = debugger.ctx(active_cpu);
    let hw = ctx.hw();

    let print_reg = |reg_num| info!("R{} = 0x{:08X}", reg_num, hw.read_reg(reg_num));
    let print_cpsr = || info!("CPSR = 0x{:08X}", hw.read_cpsr());

    let reg_str = match args.next() {
        Some(arg) => arg.to_owned().to_lowercase(),
        None => {
            for i in 0..16 {
                print_reg(i);
            }
            print_cpsr();
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
        "cpsr" => print_cpsr(),
        _ => error!("Unrecognized register!"),
    }
}

/// Runs one instruction on the CPU
/// Command format: "step"
///
/// `args`: Unused
fn cmd_step<'a, It>(active_cpu: ActiveCpu, debugger: &mut dbgcore::DbgCore, args: It)
    where It: Iterator<Item=&'a str> {
    let _ = args;
    let mut ctx = debugger.ctx(active_cpu);
    let mut hw = ctx.hw();

    hw.step();
}

/// Controls debugger behavior based on user-provided commands
///
/// `command`: Iterator over &str items
pub fn handle<'a, It>(active_cpu: &mut ActiveCpu, debugger: &mut dbgcore::DbgCore, mut command: It)
    where It: Iterator<Item=&'a str> {

    match command.next() {
        Some("asm") => cmd_asm(*active_cpu, debugger, command),
        Some("brk") => cmd_brk(*active_cpu, debugger, command),
        Some("btn") => cmd_btn(*active_cpu, debugger, command),
        Some("fbdmp") => cmd_fbdmp(*active_cpu, debugger, command),
        Some("irq") => cmd_irq(*active_cpu, debugger, command),
        Some("keydmp") => cmd_keydmp(*active_cpu, debugger, command),
        Some("mem") => cmd_mem(*active_cpu, debugger, command),
        Some("reg") => cmd_reg(*active_cpu, debugger, command),
        Some("run") => { debugger.ctx(*active_cpu).resume() },
        Some("step") => cmd_step(*active_cpu, debugger, command),

        Some("cpu") => {
            match command.next() {
                Some("arm9") => *active_cpu = ActiveCpu::Arm9,
                Some("arm11") => *active_cpu = ActiveCpu::Arm11,
                _ => error!("Expected `cpu <arm9|arm11>")
            }
        }
        Some("quit") | Some("exit") => {
            debugger.ctx(*active_cpu).hwcore_mut().stop();
            // TODO: Cleaner exit?
            exit(0);
        }
        None => {},
        Some(unk_cmd @ _) => error!("Unrecognized command `{}`", unk_cmd),
    }
}
