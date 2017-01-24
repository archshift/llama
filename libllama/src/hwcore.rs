use std::sync::{self, atomic};
use std::thread;

use cpu;
use ldr;
use mem;
use io;

fn map_memory_regions() -> mem::MemController {
    let mut controller9 = mem::MemController::new();

    let mem_itcm = mem::MemoryBlock::make_ram(0x20);
    for i in 0..0x1000 {
        controller9.map_region(i * 0x8000, mem_itcm.clone()); // ITCM
    }
    controller9.map_region(0x08000000, mem::MemoryBlock::make_ram(0x400)); // ARM9 RAM
    controller9.map_region(0x10000000, mem::MemoryBlock::make_io(io::IoRegion::Arm9(Default::default()), 0x400)); // ARM9 IO
    controller9.map_region(0x10100000, mem::MemoryBlock::make_io(io::IoRegion::Arm9(Default::default()), 0x400)); // Shared IO
    controller9.map_region(0x18000000, mem::MemoryBlock::make_ram(0x1800)); // VRAM
    controller9.map_region(0x1FF00000, mem::MemoryBlock::make_ram(0x200)); // DSP
    controller9.map_region(0x1FF80000, mem::MemoryBlock::make_ram(0x200)); // AXI WRAM
    controller9.map_region(0x20000000, mem::MemoryBlock::make_ram(0x20000)); // FCRAM
    controller9.map_region(0xFFF00000, mem::MemoryBlock::make_ram(0x10)); // DTCM
    controller9.map_region(0xFFFF0000, mem::MemoryBlock::make_ram(0x40)); // Bootrom

    return controller9;
}

pub struct Hardware {
    pub arm9: cpu::Cpu
}


pub struct HwCore {
    // TODO: Extract thread details into some `Task` abstraction
    running: sync::Arc<atomic::AtomicBool>,
    running_internal: sync::Arc<atomic::AtomicBool>,
    hardware: sync::Arc<sync::RwLock<Hardware>>,
    hardware_thread: Option<thread::JoinHandle<()>>,
}

impl HwCore {
    pub fn new<L: ldr::Loader>(loader: L) -> HwCore {
        let mut mem = map_memory_regions();
        loader.load(&mut mem);

        let mut cpu = cpu::Cpu::new(mem);
        cpu.reset(loader.entrypoint());

        HwCore {
            running: sync::Arc::new(atomic::AtomicBool::new(false)),
            running_internal: sync::Arc::new(atomic::AtomicBool::new(false)),
            hardware: sync::Arc::new(sync::RwLock::new(Hardware {
                arm9: cpu
            })),
            hardware_thread: None,
        }
    }

    // Spin up the hardware thread, take ownership of hardware
    pub fn start(&mut self) {
        // Signals that we're currently running, returns if we were already running before
        if self.running.swap(true, atomic::Ordering::SeqCst) {
            return
        }

        let hardware = self.hardware.clone();
        let running = self.running.clone();
        let running_internal = self.running_internal.clone();
        self.running_internal.store(true, atomic::Ordering::SeqCst);

        self.hardware_thread = Some(thread::spawn(move || {
            // Nobody else can access the hardware while the thread runs
            let mut hardware = hardware.write().unwrap();

            while running.load(atomic::Ordering::SeqCst) {
                if let cpu::BreakReason::Breakpoint = hardware.arm9.run(1000) {
                    info!("Breakpoint hit @ 0x{:X}!", hardware.arm9.regs[15] - hardware.arm9.get_pc_offset());
                    break;
                }
            }

            running_internal.store(false, atomic::Ordering::SeqCst);
        }));
    }

    pub fn try_wait(&mut self) -> Result<(), ()> {
        if self.running_internal.load(atomic::Ordering::SeqCst) {
            Err(())
        } else {
            self.stop();
            Ok(())
        }
    }

    pub fn stop(&mut self) {
        if let Some(handle) = self.hardware_thread.take() {
            self.running.store(false, atomic::Ordering::SeqCst);
            if handle.join().is_ok() {
                return;
            }

            // Join failed, uh oh
            if let Err(poisoned) = self.hardware.read() {
                let hw = poisoned.into_inner();
                panic!("CPU thread panicked! PC: 0x{:X}, LR: 0x{:X}", hw.arm9.regs[15], hw.arm9.regs[14]);
            }
        }
    }

    pub fn hardware(&self) -> sync::RwLockReadGuard<Hardware> {
        // Will panic if already running
        self.hardware.try_read().unwrap()
    }

    pub fn hardware_mut(&mut self) -> sync::RwLockWriteGuard<Hardware> {
        // Will panic if already running
        self.hardware.try_write().unwrap()
    }
}