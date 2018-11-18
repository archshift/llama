use std::sync::{self, Arc, Mutex};
use std::thread;

use clock;
use cpu;
use ldr;
use mem;
use io;
use msgs;

use cpu::{v5, v6};

#[derive(Clone)]
pub enum Message {
    Quit,
    StartEmulation,
    SuspendEmulation,
    Arm9Halted(cpu::BreakReason),
    Arm11Halted(cpu::BreakReason),
    HidUpdate(io::hid::ButtonState),
    FramebufState(io::gpu::FramebufState),
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
            Message::HidUpdate(_) => "hidupdate",
            Message::FramebufState(_) => "framebufstate",
        }
    }
}




struct MemoryRegions {
    mem9: mem::MemController,
    mem11: mem::MemController,
    mem_framebuf: mem::MemController,

    io9_hnd: mem::AddressBlockHandle,
    io9_shared_hnd: mem::AddressBlockHandle,
    io11_shared_hnd: mem::AddressBlockHandle,
    io11_hnd: mem::AddressBlockHandle,
}

impl MemoryRegions {
    fn map<F>(io_creator: F) -> Self
        where F: FnOnce(mem::MemController) -> (io::IoRegsArm9, io::IoRegsShared, io::IoRegsArm11)
    {
        let arm9_itcm = mem::SharedMemoryBlock::new(0x20);
        let arm9_ram = mem::UniqueMemoryBlock::new(0x400);
        let arm9_dtcm = mem::UniqueMemoryBlock::new(0x10);
        let arm9_bootrom = mem::UniqueMemoryBlock::new(0x40);

        let vram = mem::SharedMemoryBlock::new(0x1800);
        let dsp_ram = mem::SharedMemoryBlock::new(0x200);
        let axi_wram = mem::SharedMemoryBlock::new(0x200);
        let fcram = mem::SharedMemoryBlock::new(0x20000);

        let arm11_bootrom = mem::SharedMemoryBlock::new(0x40);

        let make_pica = || {
            let mut controller_pica = mem::MemController::new();
            controller_pica.map_region(0x18000000, mem::AddressBlock::SharedRam(vram.clone()));
            controller_pica.map_region(0x20000000, mem::AddressBlock::SharedRam(fcram.clone()));
            controller_pica
        };

        let controller_pica = make_pica();
        let controller_fbuf = make_pica();

        let (arm9_io, shared_io, arm11_io) = io_creator(controller_pica);

        let mut controller9 = mem::MemController::new();
        for i in 0..0x1000 {
            controller9.map_region(i * 0x8000, mem::AddressBlock::SharedRam(arm9_itcm.clone()));
        }
        controller9.map_region(0x08000000, mem::AddressBlock::UniqueRam(arm9_ram));
        controller9.map_region(0x18000000, mem::AddressBlock::SharedRam(vram.clone()));
        controller9.map_region(0x1FF00000, mem::AddressBlock::SharedRam(dsp_ram.clone()));
        controller9.map_region(0x1FF80000, mem::AddressBlock::SharedRam(axi_wram.clone()));
        controller9.map_region(0x20000000, mem::AddressBlock::SharedRam(fcram.clone()));
        controller9.map_region(0xFFF00000, mem::AddressBlock::UniqueRam(arm9_dtcm));
        controller9.map_region(0xFFFF0000, mem::AddressBlock::UniqueRam(arm9_bootrom));
        let io9_hnd         = controller9.map_region(0x10000000, mem::AddressBlock::Io9(arm9_io));
        let io9_shared_hnd  = controller9.map_region(0x10100000, mem::AddressBlock::IoShared(shared_io.clone()));

        let mut controller11 = mem::MemController::new();
        controller11.map_region(0x00000000, mem::AddressBlock::SharedRam(arm11_bootrom.clone()));
        controller11.map_region(0x00010000, mem::AddressBlock::SharedRam(arm11_bootrom.clone()));
        controller11.map_region(0x18000000, mem::AddressBlock::SharedRam(vram.clone()));
        controller11.map_region(0x1FF80000, mem::AddressBlock::SharedRam(axi_wram.clone()));
        controller11.map_region(0x20000000, mem::AddressBlock::SharedRam(fcram.clone()));
        controller11.map_region(0xFFFF0000, mem::AddressBlock::SharedRam(arm11_bootrom));
        let io11_shared_hnd = controller11.map_region(0x10100000, mem::AddressBlock::IoShared(shared_io.clone()));
        let io11_hnd        = controller11.map_region(0x10200000, mem::AddressBlock::Io11(arm11_io));

        Self {
            mem9: controller9,
            mem11: controller11,
            mem_framebuf: controller_fbuf,

            io9_hnd: io9_hnd,
            io9_shared_hnd: io9_shared_hnd,
            io11_shared_hnd: io11_shared_hnd,
            io11_hnd: io11_hnd,
        }
    }
}

// fn write_fb_pointers(cpu: &mut cpu::Cpu<v5>) {
//     // Initialize framebuffer data to be b9s compatible
//     cpu.mpu.dmem_write(0xFFF00000, 0x18000000u32);
//     cpu.mpu.dmem_write(0xFFF00004, 0x18000000u32);
//     cpu.mpu.dmem_write(0x18000000, 0x18000010u32);
//     cpu.mpu.dmem_write(0x18000008, 0x1808CA10u32);
//     cpu.regs[0] = 2;
//     cpu.regs[1] = 0xFFF00000;
// }

pub struct Hardware9 {
    pub arm9: cpu::Cpu<v5>,
    io_handle: mem::AddressBlockHandle,
    io_shared_handle: mem::AddressBlockHandle,
}

impl Hardware9 {
    pub fn io9(&self) -> &io::IoRegsArm9 {
        let region = self.arm9.mpu.memory.region(&self.io_handle);
        if let mem::AddressBlock::Io9(ref io) = region {
            io
        } else {
            unreachable!()
        }
    }

    pub fn io_shared(&self) -> &io::IoRegsShared {
        let region = self.arm9.mpu.memory.region(&self.io_shared_handle);
        if let mem::AddressBlock::IoShared(ref io) = region {
            io
        } else {
            unreachable!()
        }
    }
}

pub struct Hardware11 {
    pub arm11: cpu::Cpu<v6>,    
    io_shared_handle: mem::AddressBlockHandle,
    io_handle: mem::AddressBlockHandle,
}

impl Hardware11 {
    pub fn io11(&self) -> &io::IoRegsArm11 {
        let region = self.arm11.mpu.memory.region(&self.io_handle);
        if let mem::AddressBlock::Io11(ref io) = region {
            io
        } else {
            unreachable!()
        }
    }

    pub fn io_shared(&self) -> &io::IoRegsShared {
        let region = self.arm11.mpu.memory.region(&self.io_shared_handle);
        if let mem::AddressBlock::IoShared(ref io) = region {
            io
        } else {
            unreachable!()
        }
    }
}

pub struct HwCore {
    pub hardware9: Arc<Mutex<Hardware9>>,
    pub hardware11: Arc<Mutex<Hardware11>>,

    _pump_thread: msgs::PumpThread,
    client_this: msgs::Client<Message>,
    _arm9_thread: thread::JoinHandle<()>,
    _arm11_thread: thread::JoinHandle<()>,

    mem_framebuf: mem::MemController,
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
        let (irq_tx, irq_rx) = cpu::irq::make_channel();
        let clk_tx = clock::make_channel(irq_tx.clone());
        let clk_rx = clk_tx.clone();
        
        let client_pica = msg_pump.add_client(&[]);

        let mut mem_regions = MemoryRegions::map(|pica_controller| {
            let pica_hw = io::gpu::HardwarePica::new(client_pica, pica_controller);
            io::new_devices(irq_tx.clone(), clk_rx, pica_hw)
        });

        loader.load9(&mut mem_regions.mem9);
        loader.load11(&mut mem_regions.mem11);

        let mut cpu9 = cpu::Cpu::new(v5, mem_regions.mem9, irq_rx, clk_tx);
        cpu9.reset(loader.entrypoint9());
        // write_fb_pointers(&mut cpu9);

        let hardware9 = Hardware9 {
            arm9: cpu9,
            io_handle: mem_regions.io9_hnd,
            io_shared_handle: mem_regions.io9_shared_hnd,
        };

        let (irq11_tx, irq11_rx) = cpu::irq::make_channel();
        let clk11_tx = clock::make_channel(irq11_tx.clone());
        // let clk11_rx = clk11_tx.clone();
        let mut cpu11 = cpu::Cpu::new(v6, mem_regions.mem11, irq11_rx, clk11_tx);
        cpu11.reset(loader.entrypoint11());

        let hardware11 = Hardware11 {
            arm11: cpu11,
            io_shared_handle: mem_regions.io11_shared_hnd,
            io_handle: mem_regions.io11_hnd,
        };

        let hardware9 = Arc::new(Mutex::new(hardware9));
        let hardware11 = Arc::new(Mutex::new(hardware11));

        let client_arm9 = msg_pump.add_client(&["quit", "startemu", "suspendemu"]);
        let client_arm11 = msg_pump.add_client(&["quit", "startemu", "suspendemu", "hidupdate"]);
        let client_this = msg_pump.add_client(&[]);
        let pump_thread = msg_pump.start();

        let hardware = hardware9.clone();
        let arm9_thread = thread::Builder::new().name("ARM9".to_owned()).spawn(move || {
            let client = client_arm9;
            loop {
                if !emu_idle(&client) { break }
                {
                    let mut hw_guard = hardware.lock().unwrap();
                    if !arm9_run(&client, &mut hw_guard) { break }
                }
            }
        }).unwrap();

        let hardware = hardware11.clone();
        let arm11_thread = thread::Builder::new().name("ARM11".to_owned()).spawn(move || {
            let client = client_arm11;
            loop {
                if !emu_idle(&client) { break }
                {
                    let mut hw_guard = hardware.lock().unwrap();
                    if !arm11_run(&client, &mut hw_guard) { break }
                }
            }
        }).unwrap();

        HwCore {
            hardware9: hardware9,
            hardware11: hardware11,
            _pump_thread: pump_thread,
            client_this: client_this,
            _arm9_thread: arm9_thread,
            _arm11_thread: arm11_thread,

            mem_framebuf: mem_regions.mem_framebuf,
            irq_tx: irq_tx,
        }
    }

    pub fn start(&mut self) {
        self.client_this.send(Message::StartEmulation).unwrap();
    }

    pub fn running(&mut self) -> bool {
        if let Err(sync::TryLockError::WouldBlock) = self.hardware9.try_lock() {
            return true
        }
        if let Err(sync::TryLockError::WouldBlock) = self.hardware11.try_lock() {
            return true
        }
        return false
    }

    pub fn stop(&mut self) {
        self.client_this.send(Message::SuspendEmulation).unwrap();
        { let _ = self.hardware9.lock().unwrap(); }
        { let _ = self.hardware11.lock().unwrap(); }
    }

    pub fn copy_framebuffers(&mut self, fbs: &mut Framebuffers, fb_state: &io::gpu::FramebufState) {
        fbs.top_screen.resize({ let (w, h, d) = fbs.top_screen_size; w*h*d }, 0);
        fbs.bot_screen.resize({ let (w, h, d) = fbs.bot_screen_size; w*h*d }, 0);

        self.mem_framebuf.read_buf(fb_state.addr_top_left[0], fbs.top_screen.as_mut_slice());
        self.mem_framebuf.read_buf(fb_state.addr_bot[0], fbs.bot_screen.as_mut_slice());
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
    let reason = 't: loop {
        for msg in client.try_iter() {
            match msg {
                Message::Quit => return false,
                Message::SuspendEmulation => {
                    break 't cpu::BreakReason::Trapped
                }
                Message::HidUpdate(btn) => {
                    let io_shared = &hardware.io_shared().hid;
                    io::hid::update_pad(&mut io_shared.lock(), btn);
                }
                _ => {}
            }
        }

        if let reason @ cpu::BreakReason::Breakpoint = hardware.arm11.run(1000) {
            info!("Breakpoint hit @ 0x{:X}!", hardware.arm11.regs[15] - hardware.arm11.get_pc_offset());
            break 't reason
        }
    };

    client.send(Message::Arm11Halted(reason)).unwrap();
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
