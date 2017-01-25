use std::sync::{self, atomic};

use utils::task::Task;

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
    hardware: sync::Arc<sync::RwLock<Hardware>>,
    hardware_task: Option<Task>,
}

impl HwCore {
    pub fn new<L: ldr::Loader>(loader: L) -> HwCore {
        let mut mem = map_memory_regions();
        loader.load(&mut mem);

        let mut cpu = cpu::Cpu::new(mem);
        cpu.reset(loader.entrypoint());

        HwCore {
            hardware: sync::Arc::new(sync::RwLock::new(Hardware {
                arm9: cpu
            })),
            hardware_task: None,
        }
    }

    // Spin up the hardware thread, take ownership of hardware
    pub fn start(&mut self) {
        let hardware = self.hardware.clone();

        self.hardware_task = Some(Task::spawn(move |running| {
            // Nobody else can access the hardware while the thread runs
            let mut hardware = hardware.write().unwrap();

            while running.load(atomic::Ordering::SeqCst) {
                if let cpu::BreakReason::Breakpoint = hardware.arm9.run(1000) {
                    info!("Breakpoint hit @ 0x{:X}!", hardware.arm9.regs[15] - hardware.arm9.get_pc_offset());
                    break;
                }
            }
        }));
    }

    fn panic_action(&self) -> ! {
        // Join failed, uh oh
        if let Err(poisoned) = self.hardware.read() {
            let hw = poisoned.into_inner();
            panic!("CPU thread panicked! PC: 0x{:X}, LR: 0x{:X}", hw.arm9.regs[15], hw.arm9.regs[14]);
        }
        panic!("CPU thread panicked!");
    }

    pub fn try_wait(&mut self) -> bool {
        let res = if let Some(ref mut task) = self.hardware_task {
            task.try_join()
        } else {
            Ok(true)
        };

        match res {
            Ok(x) => x,
            Err(_) => self.panic_action()
        }
    }

    pub fn stop(&mut self) {
        let is_err = if let Some(ref mut task) = self.hardware_task {
            task.stop().is_err()
        } else {
            false
        };

        if is_err {
            self.panic_action();
        }
        self.hardware_task = None;
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