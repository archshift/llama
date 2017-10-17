use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use clock;
use cpu;
use ldr;
use mem;
use io;
use rt_data;
use msgs;



#[derive(Clone)]
pub enum Message {
    Quit,
    StartEmulation,
    SuspendEmulation,
    Arm9Halted(cpu::BreakReason),
    Arm11Halted(cpu::BreakReason),
}

impl msgs::Ident for Message {
    type Identifier = &'static str;
    fn ident(&self) -> Self::Identifier {
        match *self {
            Message::Quit => "quit",
            Message::StartEmulation => "startemu",
            Message::SuspendEmulation => "suspendemu",
            Message::Arm9Halted(_) => "arm9halted",
            Message::Arm11Halted(_) => "arm11halted",
        }
    }
}




fn map_memory_regions(arm9_io: io::IoRegsArm9, shared_io: io::IoRegsShared)
        -> (mem::MemController, mem::MemController, mem::MemController) {
    let arm9_itcm = mem::SharedMemoryBlock::new(0x20);
    let arm9_ram = mem::UniqueMemoryBlock::new(0x400);
    let arm9_io = mem::IoMemoryBlock::new(io::IoRegion::Arm9(arm9_io), 0x400);
    let arm9_dtcm = mem::UniqueMemoryBlock::new(0x10);
    let arm9_bootrom = mem::UniqueMemoryBlock::new(0x40);

    let shared_io = mem::IoMemoryBlock::new(io::IoRegion::Shared(shared_io), 0x400);
    let vram = mem::SharedMemoryBlock::new(0x1800);
    let dsp_ram = mem::SharedMemoryBlock::new(0x200);
    let axi_wram = mem::SharedMemoryBlock::new(0x200);
    let fcram = mem::SharedMemoryBlock::new(0x20000);

    let mut controller9 = mem::MemController::new();
    for i in 0..0x1000 {
        controller9.map_region(i * 0x8000, mem::AddressBlock::SharedRam(arm9_itcm.clone()));
    }
    controller9.map_region(0x08000000, mem::AddressBlock::UniqueRam(arm9_ram));
    controller9.map_region(0x10000000, mem::AddressBlock::Io(arm9_io));
    controller9.map_region(0x10100000, mem::AddressBlock::Io(shared_io.clone()));
    controller9.map_region(0x18000000, mem::AddressBlock::SharedRam(vram.clone()));
    controller9.map_region(0x1FF00000, mem::AddressBlock::SharedRam(dsp_ram.clone()));
    controller9.map_region(0x1FF80000, mem::AddressBlock::SharedRam(axi_wram.clone()));
    controller9.map_region(0x20000000, mem::AddressBlock::SharedRam(fcram.clone()));
    controller9.map_region(0xFFF00000, mem::AddressBlock::UniqueRam(arm9_dtcm));
    controller9.map_region(0xFFFF0000, mem::AddressBlock::UniqueRam(arm9_bootrom));

    let mut controller11 = mem::MemController::new();
    controller11.map_region(0x10100000, mem::AddressBlock::Io(shared_io.clone()));
    controller11.map_region(0x1FF80000, mem::AddressBlock::SharedRam(axi_wram.clone()));
    controller11.map_region(0x20000000, mem::AddressBlock::SharedRam(fcram.clone()));

    let mut controller_pica = mem::MemController::new();
    controller_pica.map_region(0x20000000, mem::AddressBlock::SharedRam(fcram.clone()));

    return (controller9, controller11, controller_pica);
}

fn write_fb_pointers(cpu: &mut cpu::Cpu) {
    // Initialize framebuffer data in a way that's compatible with BRAHMA and b9s
    cpu.memory.write(0xFFF00000, 0x23FFFE00u32);
    cpu.memory.write(0xFFF00004, 0x23FFFE00u32);
    cpu.memory.write(0x23FFFE00, 0x20000000u32);
    cpu.memory.write(0x23FFFE08, 0x2008CA00u32);
    cpu.regs[0] = 2;
    cpu.regs[1] = 0xFFF00000;
}

pub struct Hardware9 {
    pub arm9: cpu::Cpu
}

pub struct Hardware11 {
    pub dummy11: cpu::dummy11::Dummy11
}

pub struct HwCore {
    pub hardware9: Arc<Mutex<Hardware9>>,
    pub hardware11: Arc<Mutex<Hardware11>>,

    _pump_thread: msgs::PumpThread,
    client_this: msgs::Client<Message>,
    arm9_thread: thread::JoinHandle<()>,
    arm11_thread: thread::JoinHandle<()>,

    mem_pica: mem::MemController,
    pub rt_tx: rt_data::Tx,
    pub irq_tx: cpu::irq::IrqRequests,
}

#[derive(Clone, Copy, Debug)]
pub enum Arm11State {
    BootSync,
    KernelSync,
    None
}

impl HwCore {
    pub fn new(mut msg_pump: msgs::Pump<Message>, loader: &ldr::Loader) -> HwCore {
        let (rt_tx, rt_rx) = rt_data::make_channels();
        let (irq_tx, irq_rx) = cpu::irq::make_channel();
        let clk_tx = clock::make_channel(irq_tx.clone());
        let clk_rx = clk_tx.clone();

        let (io9, io11) = io::new_devices(rt_rx, irq_tx.clone(), clk_rx);
        let (mut mem9, mem11, mem_pica) = map_memory_regions(io9, io11);
        loader.load(&mut mem9);

        let mut cpu = cpu::Cpu::new(mem9, irq_rx, clk_tx);
        cpu.reset(loader.entrypoint());
        write_fb_pointers(&mut cpu);

        let arm11_state = loader.arm11_state();
        info!("Creating system with ARM11 mode {:?}...", arm11_state);
        let dummy11_mode = match arm11_state {
            Arm11State::BootSync => cpu::dummy11::modes::boot(),
            Arm11State::KernelSync => cpu::dummy11::modes::kernel(),
            Arm11State::None => cpu::dummy11::modes::idle()
        };

        let hardware9 = Hardware9 {
            arm9: cpu
        };
        let hardware11 = Hardware11 {
            dummy11: cpu::dummy11::Dummy11::new(mem11, dummy11_mode)
        };

        let hardware9 = Arc::new(Mutex::new(hardware9));
        let hardware11 = Arc::new(Mutex::new(hardware11));

        let client_arm9 = msg_pump.add_client(&["quit", "startemu", "suspendemu"]);
        let client_arm11 = msg_pump.add_client(&["quit", "startemu", "suspendemu"]);
        let client_this = msg_pump.add_client(&[]);
        let pump_thread = msg_pump.start();

        let hardware = hardware9.clone();
        let arm9_thread = thread::Builder::new().name("ARM9".to_owned()).spawn(move || {
            let client = client_arm9;
            loop {
                emu_idle(&client) || break;
                {
                    let mut hw_guard = hardware.lock().unwrap();
                    arm9_run(&client, &mut hw_guard) || break;
                }
            }
        }).unwrap();

        let hardware = hardware11.clone();
        let arm11_thread = thread::Builder::new().name("ARM11".to_owned()).spawn(move || {
            let client = client_arm11;
            loop {
                emu_idle(&client) || break;
                {
                    let mut hw_guard = hardware.lock().unwrap();
                    arm11_run(&client, &mut hw_guard) || break;
                }
            }
        }).unwrap();

        HwCore {
            hardware9: hardware9,
            hardware11: hardware11,
            _pump_thread: pump_thread,
            client_this: client_this,
            arm9_thread: arm9_thread,
            arm11_thread: arm11_thread,

            mem_pica: mem_pica,
            rt_tx: rt_tx,
            irq_tx: irq_tx,
        }
    }

    pub fn start(&mut self) {
        self.client_this.send(Message::StartEmulation).unwrap();
    }

    pub fn running(&mut self) -> bool {
        self.hardware9.try_lock().is_err() || self.hardware11.try_lock().is_err()
    }

    pub fn stop(&mut self) {
        self.client_this.send(Message::SuspendEmulation).unwrap();
        self.hardware9.lock().unwrap();
        self.hardware11.lock().unwrap();
    }

    pub fn copy_framebuffers(&mut self, fbs: &mut Framebuffers) {
        fbs.top_screen.resize({ let (w, h, d) = fbs.top_screen_size; w*h*d }, 0);
        fbs.bot_screen.resize({ let (w, h, d) = fbs.bot_screen_size; w*h*d }, 0);

        self.mem_pica.read_buf(0x20000000u32, fbs.top_screen.as_mut_slice());
        // self.mem_pica.read_buf(0x20046500u32, ..);
        self.mem_pica.read_buf(0x2008CA00u32, fbs.bot_screen.as_mut_slice());
        // self.mem_pica.read_buf(0x200C4E00u32, ..);
    }
}

fn arm9_run(client: &msgs::Client<Message>, hardware: &mut Hardware9) -> bool {
    let reason = 't: loop {
        for msg in client.try_iter() {
            match msg {
                Message::Quit => return false,
                Message::SuspendEmulation => {
                    break 't cpu::BreakReason::Trapped
                }
                _ => {}
            }
        }

        if let reason @ cpu::BreakReason::Breakpoint = hardware.arm9.run(1000) {
            info!("Breakpoint hit @ 0x{:X}!", hardware.arm9.regs[15] - hardware.arm9.get_pc_offset());
            break 't reason
        }
    };

    client.send(Message::Arm9Halted(reason)).unwrap();
    true
}

fn arm11_run(client: &msgs::Client<Message>, hardware: &mut Hardware11) -> bool {
    't: loop {
        for msg in client.try_iter() {
            match msg {
                Message::Quit => return false,
                Message::SuspendEmulation => {
                    break 't
                }
                _ => {}
            }
        }

        if let cpu::BreakReason::Breakpoint = hardware.dummy11.step() {
            thread::park_timeout(Duration::from_millis(200));
        }
    }

    client.send(Message::Arm11Halted(cpu::BreakReason::Trapped)).unwrap();
    true
}

fn emu_idle(client: &msgs::Client<Message>) -> bool {
    for msg in client.iter() {
        match msg {
            Message::StartEmulation => return true,
            Message::Quit => return false,
            _ => {}
        }
    }
    return false
}


pub struct Framebuffers {
    pub top_screen: Vec<u8>,
    pub bot_screen: Vec<u8>,
    pub top_screen_size: (usize, usize, usize),
    pub bot_screen_size: (usize, usize, usize),
}