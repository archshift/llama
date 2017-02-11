use std::sync::{self, atomic};

use utils::task::{self, TaskMgmt};

use cpu;
use ldr;
use mem;
use io;

fn map_memory_regions() -> (mem::MemController, mem::MemController) {
    let arm9_itcm = mem::MemoryBlock::make_ram(0x20);
    let axi_wram = mem::MemoryBlock::make_ram(0x200);

    let mut controller9 = mem::MemController::new();
    for i in 0..0x1000 {
        controller9.map_region(i * 0x8000, arm9_itcm.clone()); // ITCM
    }
    controller9.map_region(0x08000000, mem::MemoryBlock::make_ram(0x400)); // ARM9 RAM
    controller9.map_region(0x10000000, mem::MemoryBlock::make_io(io::IoRegion::Arm9(Default::default()), 0x400)); // ARM9 IO
    controller9.map_region(0x10100000, mem::MemoryBlock::make_io(io::IoRegion::Arm9(Default::default()), 0x400)); // Shared IO
    controller9.map_region(0x18000000, mem::MemoryBlock::make_ram(0x1800)); // VRAM
    controller9.map_region(0x1FF00000, mem::MemoryBlock::make_ram(0x200)); // DSP
    controller9.map_region(0x1FF80000, axi_wram.clone()); // AXI WRAM
    controller9.map_region(0x20000000, mem::MemoryBlock::make_ram(0x20000)); // FCRAM
    controller9.map_region(0xFFF00000, mem::MemoryBlock::make_ram(0x10)); // DTCM
    controller9.map_region(0xFFFF0000, mem::MemoryBlock::make_ram(0x40)); // Bootrom

    let mut controller11 = mem::MemController::new();
    controller11.map_region(0x1FF80000, axi_wram.clone()); // AXI WRAM

    return (controller9, controller11);
}

pub struct Hardware9 {
    pub arm9: cpu::Cpu
}

pub struct Hardware11 {
    handshake_pos: HandshakePos,
    pub mem: mem::MemController
}

#[derive(Debug)]
enum HandshakePos {
    NotStarted,
    Finished1,
    Finished2,
    Finished3
}

pub struct HwCore {
    hardware9: sync::Arc<sync::RwLock<Hardware9>>,
    hardware11: sync::Arc<sync::RwLock<Hardware11>>,

    hardware_task: Option<task::Task>,
    arm11_handshake_task: Option<task::Task>,
}

impl HwCore {
    pub fn new<L: ldr::Loader>(loader: L) -> HwCore {
        let (mut mem9, mem11) = map_memory_regions();
        loader.load(&mut mem9);

        let mut cpu = cpu::Cpu::new(mem9);
        cpu.reset(loader.entrypoint());

        HwCore {
            hardware9: sync::Arc::new(sync::RwLock::new(Hardware9 {
                arm9: cpu
            })),
            hardware11: sync::Arc::new(sync::RwLock::new(Hardware11 {
                handshake_pos: HandshakePos::NotStarted,
                mem: mem11
            })),
            hardware_task: None,
            arm11_handshake_task: None,
        }
    }

    // Spin up the hardware thread, take ownership of hardware
    pub fn start(&mut self) {
        let hardware9 = self.hardware9.clone();
        let hardware11 = self.hardware11.clone();

        self.hardware_task = Some(task::Task::spawn(move |running| {
            // Nobody else can access the hardware while the thread runs
            let mut hardware = hardware9.write().unwrap();

            while running.load(atomic::Ordering::SeqCst) {
                if let cpu::BreakReason::Breakpoint = hardware.arm9.run(1000) {
                    info!("Breakpoint hit @ 0x{:X}!", hardware.arm9.regs[15] - hardware.arm9.get_pc_offset());
                    break;
                }
            }
        }));

        // On reset, the ARM9 and ARM11 processors perform a handshake, where
        // the two processors synchronize over AXI WRAM address 0x1FFFFFF0.
        // Until the ARM11 is emulated, manually doing this will allow FIRM to boot.
        self.arm11_handshake_task = Some(task::Task::spawn(move |running| {
            use std::thread;

            // Nobody else can access the hardware while the thread runs
            let mut hardware = hardware11.write().unwrap();
            let sync_addr = 0x1FFFFFF0u32;

            if let HandshakePos::NotStarted = hardware.handshake_pos {
                hardware.mem.write::<u8>(sync_addr, 1);
                hardware.handshake_pos = HandshakePos::Finished1;
            }

            if let HandshakePos::Finished1 = hardware.handshake_pos {
                while hardware.mem.read::<u8>(sync_addr) != 2 {
                    if !running.load(atomic::Ordering::SeqCst) { return }
                    thread::yield_now();
                }
                hardware.handshake_pos = HandshakePos::Finished2;
            }

            if let HandshakePos::Finished2 = hardware.handshake_pos {
                hardware.mem.write::<u8>(sync_addr, 3);
                hardware.handshake_pos = HandshakePos::Finished3;
            }
        }));
    }

    fn panic_action(&self) -> ! {
        // Join failed, uh oh
        if let Err(poisoned) = self.hardware9.read() {
            let hw = poisoned.into_inner();
            panic!("CPU thread panicked! PC: 0x{:X}, LR: 0x{:X}", hw.arm9.regs[15], hw.arm9.regs[14]);
        }
        panic!("CPU thread panicked!");
    }

    pub fn try_wait(&mut self) -> bool {
        let res = {
            let mut tasks = [
                (&mut self.hardware_task, task::EndBehavior::StopAll),
                (&mut self.arm11_handshake_task, task::EndBehavior::Ignore)
            ];
            task::TaskUnion(&mut tasks).try_join()
        };

        match res {
            Ok(x) => x == task::JoinStatus::Joined,
            Err(_) => self.panic_action()
        }
    }

    pub fn stop(&mut self) {
        let res = {
            let mut tasks = [
                (&mut self.hardware_task, task::EndBehavior::StopAll),
                (&mut self.arm11_handshake_task, task::EndBehavior::Ignore)
            ];

            task::TaskUnion(&mut tasks).stop()
        };

        if res.is_err() {
            self.panic_action()
        }
        self.hardware_task = None;
        self.arm11_handshake_task = None;
    }

    pub fn hardware(&self) -> sync::RwLockReadGuard<Hardware9> {
        // Will panic if already running
        self.hardware9.try_read().unwrap()
    }

    pub fn hardware_mut(&mut self) -> sync::RwLockWriteGuard<Hardware9> {
        // Will panic if already running
        self.hardware9.try_write().unwrap()
    }
}