use std::sync::{self, Arc, Mutex};
use std::thread;

use clock;
use cpu;
use ldr;
use mem;
use io;
use msgs;
use fs;

use cpu::{v5, v6};
use cpu::caches::Ops;

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
    io11_priv_hnd: mem::AddressBlockHandle,
}

impl MemoryRegions {
    fn map<F>(io_creator: F) -> Self
        where F: FnOnce(mem::MemController) -> (io::IoRegsArm9, io::IoRegsShared, io::IoRegsArm11, io::IoRegsArm11Priv)
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

        let (arm9_io, shared_io, arm11_io, arm11_io_priv) = io_creator(controller_pica);

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
        let io11_priv_hnd   = controller11.map_region(0x17E00000, mem::AddressBlock::IoPriv11(arm11_io_priv));

        Self {
            mem9: controller9,
            mem11: controller11,
            mem_framebuf: controller_fbuf,

            io9_hnd: io9_hnd,
            io9_shared_hnd: io9_shared_hnd,
            io11_shared_hnd: io11_shared_hnd,
            io11_hnd: io11_hnd,
            io11_priv_hnd: io11_priv_hnd,
        }
    }
}

fn _write_fb_pointers(cpu: &mut cpu::Cpu<v5>) {
    // Initialize framebuffer data to be b9s compatible
    cpu.mpu.dmem_write(0xFFF00000, 0x18000000u32);
    cpu.mpu.dmem_write(0xFFF00004, 0x18000000u32);
    cpu.mpu.dmem_write(0x18000000, 0x18000010u32);
    cpu.mpu.dmem_write(0x18000008, 0x1808CA10u32);
    cpu.regs[0] = 2;
    cpu.regs[1] = 0xFFF00000;
}

fn try_bootrom_load(mem: &mut mem::MemController, llama_file: fs::LlamaFile) {
    use std::io::Read;

    loop {
        let mut file = match fs::open_file(llama_file) {
            Ok(x) => x,
            Err(_) => break
        };
        let mut buf = [0u8; 0x10000];
        if !file.read_exact(&mut buf).is_ok() { break };
        mem.write_buf(0xFFFF0000, &buf);
        return
    }
    info!("Did not find {:?}; not loading bootrom.", llama_file);
}

pub struct Hardware9 {
    pub arm9: cpu::Cpu<v5>,
    io_handle: mem::AddressBlockHandle,
    io_shared_handle: mem::AddressBlockHandle,
}

/// Hardware9 will not contain any x-thread references
unsafe impl Send for Hardware9 {}

impl Hardware9 {
    pub fn io9(&self) -> &io::IoRegsArm9 {
        let region = self.arm9.mpu.main_mem().region(&self.io_handle);
        if let mem::AddressBlock::Io9(ref io) = region {
            io
        } else {
            unreachable!()
        }
    }

    pub fn io_shared(&self) -> &io::IoRegsShared {
        let region = self.arm9.mpu.main_mem().region(&self.io_shared_handle);
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
    io_priv_handle: mem::AddressBlockHandle,
}

/// Hardware11 will not contain any x-thread references
unsafe impl Send for Hardware11 {}

impl Hardware11 {
    pub fn io11(&self) -> &io::IoRegsArm11 {
        let region = self.arm11.mpu.main_mem().region(&self.io_handle);
        if let mem::AddressBlock::Io11(ref io) = region {
            io
        } else {
            unreachable!()
        }
    }

    pub fn io_shared(&self) -> &io::IoRegsShared {
        let region = self.arm11.mpu.main_mem().region(&self.io_shared_handle);
        if let mem::AddressBlock::IoShared(ref io) = region {
            io
        } else {
            unreachable!()
        }
    }

    pub fn io_priv(&self) -> &io::IoRegsArm11Priv {
        let region = self.arm11.mpu.main_mem().region(&self.io_priv_handle);
        if let mem::AddressBlock::IoPriv11(ref io) = region {
            io
        } else {
            unreachable!()
        }
    }
}

pub struct HwCore {
    pub hardware9: Arc<Mutex<Hardware9>>,
    pub hardware11: Arc<Mutex<Hardware11>>,

    client_this: msgs::Client<Message>,
    client_user: Option<msgs::Client<Message>>,
    client_gdb: Option<msgs::Client<Message>>,
    _arm9_thread: thread::JoinHandle<()>,
    _arm11_thread: thread::JoinHandle<()>,

    mem_framebuf: mem::MemController,
    pub irq_tx: cpu::irq::IrqAsyncClient,
}

/// HwCore will not contain any x-thread references
unsafe impl Send for HwCore {}

#[derive(Clone, Copy, Debug)]
pub enum Arm11State {
    BootSync,
    KernelSync,
    None
}

impl HwCore {
    pub fn new(loader: &ldr::Loader) -> HwCore {
        let mut msg_spec = msgs::MsgGraph::new(&[
            ("gdb", &[], &["quit", "arm9halted"]),
            ("user", &["quit", "hidupdate"], &["framebufstate"]),
            ("arm9", &["arm9halted", "arm11halted"], &["quit", "startemu", "suspendemu"]),
            ("arm11", &["arm9halted", "arm11halted"], &["quit", "startemu", "suspendemu", "hidupdate"]),
            ("pica", &["framebufstate"], &[]),
            ("hwcore", &["startemu", "suspendemu"], &[]),
        ]);

        let client_gdb = msg_spec.client("gdb");
        let client_user = msg_spec.client("user");

        let client_arm9 = msg_spec.client("arm9").unwrap();
        let client_arm11 = msg_spec.client("arm11").unwrap();
        let client_pica = msg_spec.client("pica").unwrap();
        let client_this = msg_spec.client("hwcore").unwrap();

        let irq_subsys = cpu::irq::IrqSubsys::create();
        let irq_line = irq_subsys.line.clone();
        let irq11_subsys = cpu::irq::IrqSubsys::create();
        let irq11_line = irq11_subsys.line.clone();
        let irq_async_tx = irq_subsys.async_tx.clone();

        let clk_tx = clock::make_channel(irq_subsys.sync_tx.clone());
        let clk_rx = clk_tx.clone();
        let clk11_tx = clock::make_channel(irq11_subsys.sync_tx.clone());
        // let clk11_rx = clk11_tx.clone();

        let mut mem_regions = MemoryRegions::map(|pica_controller| {
            let pica_hw = io::gpu::HardwarePica::new(client_pica, pica_controller);
            io::new_devices(irq_subsys, irq11_subsys, clk_rx, pica_hw)
        });

        loader.load9(&mut mem_regions.mem9);
        loader.load11(&mut mem_regions.mem11);

        try_bootrom_load(&mut mem_regions.mem9, fs::LlamaFile::Boot9);
        try_bootrom_load(&mut mem_regions.mem11, fs::LlamaFile::Boot11);

        let mut cpu9 = cpu::Cpu::new(v5, mem_regions.mem9, irq_line, clk_tx);
        cpu9.reset(loader.entrypoint9());
        
        // TODO: put this boot9strap compatibility code behind some configuration
        cpu9.regs[0] = 1;
        cpu9.regs[1] = 0x01FF8000;
        cpu9.regs[2] = 0x3BEEF;
        cpu9.mpu.main_mem_mut().write(0x01FF8000, 0x01FF8008u32);
        cpu9.mpu.main_mem_mut().write(0x01FF8004, 0u32);
        cpu9.mpu.main_mem_mut().write_buf(0x01FF8008, b"sdmc:/boot.firm\0");
        //write_fb_pointers(&mut cpu9);

        let hardware9 = Hardware9 {
            arm9: cpu9,
            io_handle: mem_regions.io9_hnd,
            io_shared_handle: mem_regions.io9_shared_hnd,
        };

        let mut cpu11 = cpu::Cpu::new(v6, mem_regions.mem11, irq11_line, clk11_tx);
        cpu11.reset(loader.entrypoint11());

        let hardware11 = Hardware11 {
            arm11: cpu11,
            io_shared_handle: mem_regions.io11_shared_hnd,
            io_handle: mem_regions.io11_hnd,
            io_priv_handle: mem_regions.io11_priv_hnd,
        };

        let hardware9 = Arc::new(Mutex::new(hardware9));
        let hardware11 = Arc::new(Mutex::new(hardware11));

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
            client_this: client_this,
            client_user: client_user,
            client_gdb: client_gdb,
            _arm9_thread: arm9_thread,
            _arm11_thread: arm11_thread,

            mem_framebuf: mem_regions.mem_framebuf,
            irq_tx: irq_async_tx,
        }
    }

    pub fn start(&mut self) {
        self.client_this.send(Message::StartEmulation);
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
        self.client_this.send(Message::SuspendEmulation);
        { let _ = self.hardware9.lock().unwrap(); }
        { let _ = self.hardware11.lock().unwrap(); }
    }

    pub fn copy_framebuffers(&self, fbs: &mut Framebuffers, fb_state: &io::gpu::FramebufState) {
        let pixel_depth = |color_fmt: io::gpu::ColorFormat| match color_fmt {
            io::gpu::ColorFormat::Rgb8 => 3,
            io::gpu::ColorFormat::Rgba8 => 4,
            io::gpu::ColorFormat::Rgba4 | io::gpu::ColorFormat::Rgb5a1 | io::gpu::ColorFormat::Rgb565 => 2
        };

        fbs.top_screen_size = (240, 400, pixel_depth(fb_state.color_fmt[0]));
        fbs.bot_screen_size = (240, 320, pixel_depth(fb_state.color_fmt[0]));

        fbs.top_screen.resize({ let (w, h, d) = fbs.top_screen_size; w*h*d }, 0);
        fbs.bot_screen.resize({ let (w, h, d) = fbs.bot_screen_size; w*h*d }, 0);

        let _ = self.mem_framebuf.debug_read_buf(fb_state.addr_top_left[0], fbs.top_screen.as_mut_slice());
        let _ = self.mem_framebuf.debug_read_buf(fb_state.addr_bot[0], fbs.bot_screen.as_mut_slice());
    }

    pub fn take_client_user(&mut self) -> Option<msgs::Client<Message>> {
        self.client_user.take()
    }

    pub fn take_client_gdb(&mut self) -> Option<msgs::Client<Message>> {
        self.client_gdb.take()
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
            client.send(Message::Arm11Halted(reason));
            break 't reason
        }
    };

    client.send(Message::Arm9Halted(reason));
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
            client.send(Message::Arm9Halted(reason));
            break 't reason
        }
    };

    client.send(Message::Arm11Halted(reason));

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


#[derive(Default)]
pub struct Framebuffers {
    pub top_screen: Vec<u8>,
    pub bot_screen: Vec<u8>,
    pub top_screen_size: (usize, usize, usize),
    pub bot_screen_size: (usize, usize, usize),
}
