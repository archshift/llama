use cpu::irq::IrqSyncClient;
use io::timer;

#[derive(Clone)]
pub struct SysClock {
    pub timer_states: timer::TimerStates,
    pub irq_tx: IrqSyncClient,
}



impl SysClock {
    pub fn increment(&mut self, by: u64) {
        timer::handle_clock_update(&self.timer_states, by, &mut self.irq_tx);
    }

    pub fn get(&self) -> u64 {
        self.timer_states.global_counter.get()
    }
}

pub fn make_channel(irq_tx: IrqSyncClient) -> SysClock {
    let timer_states = timer::TimerStates::new();
    SysClock {
        timer_states: timer_states,
        irq_tx: irq_tx
    }
}
