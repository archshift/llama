use std::sync::{self, atomic};
use std::thread;

use cpu;
use mem;

pub struct System {
    running: sync::Arc<atomic::AtomicBool>,
    join_handles: Vec<thread::JoinHandle<()>>,

    arm9: sync::Arc<sync::RwLock<cpu::Cpu>>,
}

impl System {
    pub fn new(entrypoint: u32, mem_controller: mem::MemController) -> System {
        let mut cpu = cpu::Cpu::new(mem_controller);
        cpu.reset(entrypoint);

        System {
            running: sync::Arc::new(atomic::AtomicBool::new(true)),
            join_handles: Vec::new(),
            arm9: sync::Arc::new(sync::RwLock::new(cpu)),
        }
    }

    pub fn start(&mut self) {
        self.running.store(true, atomic::Ordering::Relaxed);

        self.join_handles.push({
            let arm9 = self.arm9.clone();
            let running = self.running.clone();
            
            thread::spawn(move || {
                while running.load(atomic::Ordering::Relaxed) {
                    arm9.write().unwrap().run(1000);
                }
            })
        })
    }

    pub fn stop(&self) {
        self.running.store(false, atomic::Ordering::Relaxed);
    }

    pub fn wait(&mut self) {
        while let Some(handle) = self.join_handles.pop() {
            handle.join();
        }
    }
}