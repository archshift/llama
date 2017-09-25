use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use cpu::irq::IrqRequests;
use io::timer;

#[derive(Clone)]
pub struct SysClock {
    counter: Arc<AtomicUsize>,
    pub timer_states: timer::TimerStates,
    pub irq_tx: IrqRequests,
}

impl SysClock {
    pub fn increment(&mut self, by: usize) {
        self.counter.fetch_add(by, Ordering::Relaxed);
        timer::handle_clock_update(&self.timer_states, by, &mut self.irq_tx);
    }

    pub fn get(&self) -> usize {
        self.counter.load(Ordering::Relaxed)
    }
}

pub fn make_channel(irq_tx: IrqRequests) -> SysClock {
    let counter = Arc::new(AtomicUsize::new(0));
    let timer_states = timer::TimerStates::new();
    SysClock {
        counter: counter.clone(),
        timer_states: timer_states,
        irq_tx: irq_tx
    }
}