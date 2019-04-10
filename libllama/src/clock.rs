use std::cell::Cell;
use std::rc::Rc;

use cpu::irq::IrqSyncClient;
use io::timer;

#[derive(Clone)]
pub struct SysClock {
    counter: Rc<Cell<usize>>,
    pub timer_states: timer::TimerStates,
    pub irq_tx: IrqSyncClient,
}



impl SysClock {
    pub fn increment(&mut self, by: usize) {
        self.counter.set(self.counter.get() + by);
        timer::handle_clock_update(&self.timer_states, by, &mut self.irq_tx);
    }

    pub fn get(&self) -> usize {
        self.counter.get()
    }
}

pub fn make_channel(irq_tx: IrqSyncClient) -> SysClock {
    let counter = Rc::new(Cell::new(0));
    let timer_states = timer::TimerStates::new();
    SysClock {
        counter: counter.clone(),
        timer_states: timer_states,
        irq_tx: irq_tx
    }
}
