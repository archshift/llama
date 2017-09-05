use std::sync::{self, atomic};

use utils::task::{self, TaskMgmt};

use cpu;
use ldr;
use mem;
use io;
use rt_data;

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

pub struct Hardware9 {
    pub arm9: cpu::Cpu
}

pub struct Hardware11 {
    pub dummy11: cpu::dummy11::Dummy11
}

pub struct HwCore {
    hardware9: Option<Hardware9>,
    hardware11: Option<Hardware11>,

    hardware_task: Option<task::Task<Hardware9>>,
    arm11_handshake_task: Option<task::Task<Hardware11>>,

    mem_pica: mem::MemController,
    pub rt_tx: rt_data::Tx,
}

#[derive(Clone, Copy, Debug)]
pub enum Arm11State {
    BootSync,
    KernelSync,
    None
}

impl HwCore {
    pub fn new(loader: &ldr::Loader) -> HwCore {
        let (rt_tx, rt_rx) = rt_data::make_channels();

        let (io9, io11) = io::new_devices(rt_rx);
        let (mut mem9, mem11, mem_pica) = map_memory_regions(io9, io11);
        loader.load(&mut mem9);

        let mut cpu = cpu::Cpu::new(mem9);
        cpu.reset(loader.entrypoint());

        let arm11_state = loader.arm11_state();
        info!("Creating system with ARM11 mode {:?}...", arm11_state);
        let dummy11_mode = match arm11_state {
            Arm11State::BootSync => cpu::dummy11::modes::boot(),
            Arm11State::KernelSync => cpu::dummy11::modes::kernel(),
            Arm11State::None => cpu::dummy11::modes::idle()
        };

        HwCore {
            hardware9: Some(Hardware9 {
                arm9: cpu
            }),
            hardware11: Some(Hardware11 {
                dummy11: cpu::dummy11::Dummy11::new(mem11, dummy11_mode)
            }),
            hardware_task: None,
            arm11_handshake_task: None,
            mem_pica: mem_pica,
            rt_tx: rt_tx,
        }
    }

    // Spin up the hardware thread, take ownership of hardware
    pub fn start(&mut self) {
        let hardware9 = self.hardware9.take().unwrap();
        let hardware11 = self.hardware11.take().unwrap();

        self.hardware_task = Some(task::Task::spawn(move |running| {
            // Nobody else can access the hardware while the thread runs
            let mut hardware = hardware9;

            while running.load(atomic::Ordering::SeqCst) {
                if let cpu::BreakReason::Breakpoint = hardware.arm9.run(1000) {
                    info!("Breakpoint hit @ 0x{:X}!", hardware.arm9.regs[15] - hardware.arm9.get_pc_offset());
                    break;
                }
            }

            hardware
        }));

        // On reset, the ARM9 and ARM11 processors perform a handshake, where
        // the two processors synchronize over AXI WRAM address 0x1FFFFFF0.
        // Until the ARM11 is emulated, manually doing this will allow FIRM to boot.
        self.arm11_handshake_task = Some(task::Task::spawn(move |running| {
            // Nobody else can access the hardware while the thread runs
            let mut hardware = hardware11;

            use std::{thread, time};

            while running.load(atomic::Ordering::SeqCst) {
                if let cpu::BreakReason::Breakpoint = hardware.dummy11.step() {
                    thread::sleep(time::Duration::from_millis(10));
                }
            }

            hardware
        }));
    }

    fn panic_action(&self) -> ! {
        // Join failed, uh oh
        panic!("CPU thread panicked!");
    }

    fn try_restore_hardwares(&mut self) {
        fn try_get_hw<T>(opt_task: &mut Option<task::Task<T>>, out_hw: &mut Option<T>){
            if let Some(task) = opt_task.as_mut() {
                let ret = task.ret.take();
                if let task::ReturnVal::Ready(val) = ret {
                    *out_hw = Some(val);
                } else {
                    task.ret = ret; // Oops, let's restore that
                }
            }
        }

        try_get_hw(&mut self.hardware_task, &mut self.hardware9);
        try_get_hw(&mut self.arm11_handshake_task, &mut self.hardware11);
    }

    pub fn try_wait(&mut self) -> bool {
        let res = {
            let mut tasks = [
                // TODO: This is ugly!
                (self.hardware_task.as_mut().map(|x| x as &mut task::TaskMgmt), task::EndBehavior::StopAll),
                (self.arm11_handshake_task.as_mut().map(|x| x as &mut task::TaskMgmt), task::EndBehavior::Ignore)
            ];
            task::TaskUnion(&mut tasks).try_join()
        };

        match res {
            Ok(x) => {
                self.try_restore_hardwares();
                x == task::JoinStatus::Joined
            },
            Err(_) => self.panic_action()
        }
    }

    pub fn stop(&mut self) {
        let res = {
            let mut tasks = [
                // TODO: This is ugly!
                (self.hardware_task.as_mut().map(|x| x as &mut task::TaskMgmt), task::EndBehavior::StopAll),
                (self.arm11_handshake_task.as_mut().map(|x| x as &mut task::TaskMgmt), task::EndBehavior::Ignore)
            ];

            task::TaskUnion(&mut tasks).stop()
        };

        if res.is_err() {
            self.panic_action()
        }
        self.try_restore_hardwares();
        self.hardware_task = None;
        self.arm11_handshake_task = None;
    }

    pub fn hardware<'a>(&'a self) -> &'a Hardware9 {
        // Will panic if already running
        self.hardware9.as_ref().unwrap()
    }

    pub fn hardware_mut<'a>(&'a mut self) -> &'a mut Hardware9 {
        // Will panic if already running
        self.hardware9.as_mut().unwrap()
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

pub struct Framebuffers {
    pub top_screen: Vec<u8>,
    pub bot_screen: Vec<u8>,
    pub top_screen_size: (usize, usize, usize),
    pub bot_screen_size: (usize, usize, usize),
}