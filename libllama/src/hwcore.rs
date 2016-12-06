use std::sync::{self, atomic};
use std::thread;

use cpu;
use mem;
use io;

pub fn map_memory_regions() -> mem::MemController {
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
    running: sync::Arc<atomic::AtomicBool>,
    hardware: sync::Arc<sync::RwLock<Hardware>>,
    hardware_thread: Option<thread::JoinHandle<()>>,
}

impl HwCore {
    pub fn new(entrypoint: u32, mem_controller: mem::MemController) -> HwCore {
        let mut cpu = cpu::Cpu::new(mem_controller);
        cpu.reset(entrypoint);

        HwCore {
            running: sync::Arc::new(atomic::AtomicBool::new(false)),
            hardware: sync::Arc::new(sync::RwLock::new(Hardware {
                arm9: cpu
            })),
            hardware_thread: None,
        }
    }

    // Spin up the hardware thread, take ownership of hardware
    pub fn start(&mut self) {
        // Signals that we're currently running, returns if we were already running before
        if self.running.swap(true, atomic::Ordering::Relaxed) {
            return
        }

        let hardware = self.hardware.clone();
        let running = self.running.clone();

        self.hardware_thread = Some(thread::spawn(move || {
            // Nobody else can access the hardware while the thread runs
            let mut hardware = hardware.write().unwrap();

            while running.load(atomic::Ordering::Relaxed) {
                hardware.arm9.run(1000);
            }
        }));
    }

    pub fn stop(&mut self) {
        self.running.store(false, atomic::Ordering::Relaxed);
        if let Some(handle) = self.hardware_thread.take() {
            let _ = handle.join();
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