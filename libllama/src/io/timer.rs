// TODO: How can we test this module?

use std::cell::{RefCell, Cell};
use std::fmt;
use std::rc::Rc;

use cpu::irq::{self, IrqClient};
use io::regs::IoReg;

#[derive(Clone, Copy, Debug)]
pub enum Prescaler {
    Div1 = 0,
    Div64 = 1,
    Div256 = 2,
    Div1024 = 3,
}

impl Prescaler {
    fn new(val: u16) -> Prescaler {
        match val {
            0 => Prescaler::Div1,
            1 => Prescaler::Div64,
            2 => Prescaler::Div256,
            3 => Prescaler::Div1024,
            _ => unreachable!()
        }
    }
}

bf!(CntReg[u16] {
    prescaler: 0:1,
    count_up: 2:2,
    irq_enable: 6:6,
    started: 7:7
});

fn get_regs(dev: &mut TimerDevice, index: usize) -> (&mut IoReg<u16>, &mut IoReg<u16>) {
    match index {
        0 => (&mut dev.val0, &mut dev.cnt0),
        1 => (&mut dev.val1, &mut dev.cnt1),
        2 => (&mut dev.val2, &mut dev.cnt2),
        3 => (&mut dev.val3, &mut dev.cnt3),
        _ => unreachable!()
    }
}

fn reg_val_update(dev: &mut TimerDevice, index: usize) {
    let val = {
        let (val, _) = get_regs(dev, index);
        val.get()
    };
    let mut state = dev._internal_state.all.borrow_mut();
    state[index].set_val(val);
    drop(state);

    update_deadlines(&dev._internal_state);
}

fn reg_val_read(dev: &mut TimerDevice, index: usize) {
    let new_val = {
        let mut state = dev._internal_state.all.borrow_mut();
        state[index].val() as u16
    };
    let (val, _) = get_regs(dev, index);
    val.set_unchecked(new_val);
}

fn reg_cnt_update(dev: &mut TimerDevice, index: usize) {
    let cnt = {
        let (_, cnt) = get_regs(dev, index);
        CntReg::new(cnt.get())
    };
    let mut state = dev._internal_state.all.borrow_mut();

    if !state[index].started && cnt.started.get() == 1 {
        // Set baseline deadline counter for this timer so that we can
        // subtract against it later
        let baseline = dev._internal_state.global_counter.get();
        dev._internal_state.start_counters[index].set(baseline);
    }
    state[index].started = cnt.started.get() == 1;
    state[index].prescaler = Prescaler::new(cnt.prescaler.get());
    state[index].val_cycles = if cnt.count_up.get() == 1 {
        Cycles::CountUp(state[index].val())
    } else {
        Cycles::Unscaled(state[index].val())
    };
    trace!("Setting TIMER CNT{}: {:?}", index, state);
    drop(state);

    update_deadlines(&dev._internal_state);
}

/// Convert CPU cycles to timer cycles using the given prescaler
fn scale(cycles: u64, prescaler: Prescaler) -> u64 {
    match prescaler {
        Prescaler::Div1 => cycles,
        Prescaler::Div64 => cycles >> 6,
        Prescaler::Div256 => cycles >> 8,
        Prescaler::Div1024 => cycles >> 10
    }
}

/// Convert timer cycles to CPU cycles using the given prescaler
fn unscale(clock_ticks: u64, prescaler: Prescaler) -> u64 {
    match prescaler {
        Prescaler::Div1 => clock_ticks,
        Prescaler::Div64 => clock_ticks << 6,
        Prescaler::Div256 => clock_ticks << 8,
        Prescaler::Div1024 => clock_ticks << 10
    }
}

fn irq(t_index: usize) -> irq::IrqType9 {
    match t_index {
        0 => irq::IrqType9::Timer0,
        1 => irq::IrqType9::Timer1,
        2 => irq::IrqType9::Timer2,
        3 => irq::IrqType9::Timer3,
        _ => unreachable!()
    }
}





#[derive(Copy, Clone, Debug)]
enum Cycles {
    CountUp(u64),
    Unscaled(u64)
}

#[derive(Debug)]
pub struct TimerState {
    started: bool,
    val_cycles: Cycles,
    prescaler: Prescaler,
}

impl TimerState {
    pub fn new() -> TimerState {
        TimerState {
            started: false,
            val_cycles: Cycles::Unscaled(0),
            prescaler: Prescaler::Div1
        }
    }

    /// Current timer value in timer cycles
    fn val(&self) -> u64 {
        match self.val_cycles {
            Cycles::Unscaled(cyc) => scale(cyc, self.prescaler),
            Cycles::CountUp(cyc) => cyc
        }
    }

    /// Update current timer value with cycles elapsed, return true if overflow
    fn incr_and_check_overflow(&mut self, clock_diff: Cycles) -> bool {
        let till_overflow = self.clocks_till_overflow();

        match self.val_cycles {
            Cycles::Unscaled(ref mut cyc) => {
                if let Cycles::Unscaled(addend) = clock_diff {
                    *cyc += addend;
                    addend >= till_overflow

                } else { unreachable!() }
            }

            Cycles::CountUp(ref mut cyc) => {
                if let Cycles::CountUp(addend) = clock_diff {
                    let out = addend + *cyc % (1 << 16) >= 1 << 16;
                    *cyc += addend;
                    out

                } else { unreachable!() }
            }
        }
    }

    /// Returns the number of CPU clocks until this timer triggers an overflow
    fn clocks_till_overflow(&self) -> u64 {
        match self.val_cycles {
            Cycles::Unscaled(cyc) => {
                let period = unscale(1 << 16, self.prescaler);
                (period - cyc % period)
            }
            Cycles::CountUp(_) => {
                // It makes no sense to do this calculation for countup timers
                // since if they overflow it's because some lower timer overflowed
                !0
            }
        }
    }

    /// Updates u16 value written to TIMER registers if we match a timer at `t_index`
    fn set_val(&mut self, val: u16) {
        match self.val_cycles {
            Cycles::Unscaled(ref mut cyc)
                => *cyc = unscale(val as u64, self.prescaler),
            Cycles::CountUp(ref mut cyc)
                => *cyc = val as u64,
        }
    }
}






#[derive(Clone)]
pub struct TimerStates {
    deadline: Rc<Cell<Option<u64>>>,
    pub(crate) global_counter: Rc<Cell<u64>>,
    /// global_counter value for when each timer was started
    start_counters: Rc<[Cell<u64>; 4]>,
    all: Rc<RefCell<[TimerState; 4]>>
}

impl TimerStates {
    pub fn new() -> TimerStates {
        TimerStates {
            deadline: Rc::new(Cell::new(None)),
            global_counter: Rc::new(Cell::new(0)),
            start_counters: Rc::new([
                Cell::new(0), Cell::new(0),
                Cell::new(0), Cell::new(0),
            ]),
            all: Rc::new(RefCell::new([
                TimerState::new(), TimerState::new(),
                TimerState::new(), TimerState::new(),
            ]))
        }
    }
}

impl fmt::Debug for TimerStates {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TimerStates {{ }}")
    }
}



pub fn handle_clock_update(timer_states: &TimerStates, clock_diff: u64, irq_tx: &mut irq::IrqSyncClient) {
    // Update global counter
    let ctr = timer_states.global_counter.get();
    let new_ctr = ctr + clock_diff;
    timer_states.global_counter.set(new_ctr);

    // Check if we have any work to do
    if !past_deadline(timer_states, clock_diff) {
        return
    }

    let mut timers = timer_states.all.borrow_mut();
    if let Cycles::CountUp(_) = timers[0].val_cycles {
        panic!("Don't know how to handle TIMER0 as a count-up timer!");
    }

    // Update individual timers
    let mut prev_overflowed = false;
    for (index, timer) in timers.iter_mut().enumerate() {
        if !timer.started {
            continue;
        }

        let cycles = if let Cycles::CountUp(_) = timer.val_cycles {
            Cycles::CountUp(prev_overflowed as u64)
        } else {
            let ctr = timer_states.global_counter.get();
            let baseline = timer_states.start_counters[index].get();
            let clock_diff = ctr - baseline;

            timer_states.start_counters[index].set(ctr);
            Cycles::Unscaled(clock_diff)
        };

        prev_overflowed = timer.incr_and_check_overflow(cycles);
        if prev_overflowed {
            irq_tx.assert(irq(index))
        }
    }
    drop(timers);

    update_deadlines(timer_states);
}

fn update_deadlines(timer_states: &TimerStates) {
    let timers = timer_states.all.borrow();
    let min_deadline = timers.iter()
        .filter(|timer| timer.started)
        .map(|timer| timer.clocks_till_overflow())
        .min();
    let ctr = timer_states.global_counter.get();
    timer_states.deadline.set(min_deadline.map(|m| m + ctr));
}

fn past_deadline(timer_states: &TimerStates, clock_diff: u64) -> bool {
    if let Some(deadline) = timer_states.deadline.get() {
        ctr > deadline
    } else {
        false
    }
}



iodevice!(TimerDevice, {
    internal_state: TimerStates;
    regs: {
        0x000 => val0: u16 {
            read_effect = |dev: &mut TimerDevice| reg_val_read(dev, 0);
            write_effect = |dev: &mut TimerDevice| reg_val_update(dev, 0);
        }
        0x002 => cnt0: u16 { write_effect = |dev: &mut TimerDevice| reg_cnt_update(dev, 0); }

        0x004 => val1: u16 {
            read_effect = |dev: &mut TimerDevice| reg_val_read(dev, 1);
            write_effect = |dev: &mut TimerDevice| reg_val_update(dev, 1);
        }
        0x006 => cnt1: u16 { write_effect = |dev: &mut TimerDevice| reg_cnt_update(dev, 1); }

        0x008 => val2: u16 {
            read_effect = |dev: &mut TimerDevice| reg_val_read(dev, 2);
            write_effect = |dev: &mut TimerDevice| reg_val_update(dev, 2);
        }
        0x00A => cnt2: u16 { write_effect = |dev: &mut TimerDevice| reg_cnt_update(dev, 2); }

        0x00C => val3: u16 {
            read_effect = |dev: &mut TimerDevice| reg_val_read(dev, 3);
            write_effect = |dev: &mut TimerDevice| reg_val_update(dev, 3);
        }
        0x00E => cnt3: u16 { write_effect = |dev: &mut TimerDevice| reg_cnt_update(dev, 3); }
    }
});
