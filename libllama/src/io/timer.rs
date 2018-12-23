// TODO: How can we test this module?

use std::fmt;
use std::sync::Arc;

use parking_lot::{Mutex, MutexGuard};

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
    let states = &dev._internal_state;
    for mut t in TimerIter::new(states).filter(|t| t.has_index(index)) {
        t.set_val_hword(index, val)
    }
}

fn reg_val_read(dev: &mut TimerDevice, index: usize) {
    let new_val = {
        let states = &dev._internal_state;
        TimerIter::new(states).filter_map(|t| t.val_hword(index))
                              .next().unwrap()
    };
    let (val, _) = get_regs(dev, index);
    val.set_unchecked(new_val);
}

fn reg_cnt_update(dev: &mut TimerDevice, index: usize) {
    let (val, cnt) = {
        let (val, cnt) = get_regs(dev, index);
        (val.get(), CntReg::new(cnt.get()))
    };
    let mut timer = {
        let states = &dev._internal_state;
        TimerIter::new(states).filter(|t| t.has_index(index))
                              .next().unwrap()
    };
    {
        let state = match timer {
            Timer::Unit(_, ref mut state) | Timer::Joined { lowstate: ref mut state, .. } => state
        };
        state.started = cnt.started.get() == 1;
        state.count_up = cnt.count_up.get() == 1;
        state.prescaler = Prescaler::new(cnt.prescaler.get());
    }

    timer.set_val_hword(index, val);
    trace!("Setting TIMER CNT{}: {:?}", index, timer);
}


enum Timer<'a> {
    Joined { bitset: u8, lowstate: MutexGuard<'a, TimerState> },
    Unit(u8, MutexGuard<'a, TimerState>)
}

impl<'a> fmt::Debug for Timer<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Timer::Unit(num, ref state) => write!(f, "Timer::Unit({}, {:?})", num, **state),
            Timer::Joined { bitset: bs, lowstate: ref state } => {
                write!(f, "Timer::Joined {{ bitset: 0b{:b}, lowstate: {:?} }}", bs, **state)
            }
        }
    }
}

impl<'a> Timer<'a> {
    fn started(&self) -> bool {
        match *self {
            Timer::Unit(_, ref state) | Timer::Joined { lowstate: ref state, .. } => state.started
        }
    }

    fn val(&self) -> u64 {
        let state = match *self {
            Timer::Unit(_, ref state) | Timer::Joined { lowstate: ref state, .. } => state
        };
        scale(state.val_cycles, state.prescaler)
    }

    fn incr_val(&mut self, clock_diff: u64) {
        let state = match *self {
            Timer::Unit(_, ref mut state) | Timer::Joined { lowstate: ref mut state, .. } => state
        };
        state.val_cycles += clock_diff;
    }

    fn val_diff(&self, clock_diff: u64) -> u64 {
        let state = match *self {
            Timer::Unit(_, ref state) | Timer::Joined { lowstate: ref state, .. } => state
        };

        let new_val = scale(state.val_cycles + clock_diff, state.prescaler);
        new_val - self.val()
    }

    fn will_overflow_words(&self, clock_diff: u64) -> [bool; 4] {
        let diff = self.val_diff(clock_diff);
        let mut overflow_words = [false; 4];
        match *self {
            Timer::Unit(num, _) => {
                let overflows = (self.val() as u16).checked_add(diff as u16).is_none();
                if diff >> 16 != 0 || overflows {
                    overflow_words[num as usize] = true;
                }
            }
            Timer::Joined { bitset: bs, .. } => {
                let lowest_word = bs.trailing_zeros() as usize;
                let val = self.val();
                let newval = val + diff;
                for i in 0..4 {
                    let val_hword = (val >> (16*i)) as u16;
                    let newval_hword = (newval >> (16*i)) as u16;
                    if newval_hword < val_hword {
                        overflow_words[lowest_word+i] = true;
                    }
                }
            }
        }
        overflow_words
    }

    fn has_index(&self, t_index: usize) -> bool {
        let t_index = t_index as u8;

        match *self {
            Timer::Unit(num, ..) => num == t_index,
            Timer::Joined { bitset: bs, .. } => bs & (1 << t_index) != 0
        }
    }

    fn val_hword(&self, t_index: usize) -> Option<u16> {
        let t_index = t_index as u8;

        match *self {
            Timer::Unit(num, ..) if num == t_index => {
                Some(self.val() as u16)
            }
            Timer::Joined { bitset: bs, .. } if bs & (1 << t_index) != 0 => {
                Some((self.val() >> (16*t_index)) as u16)
            }
            _ => None
        }
    }

    fn set_val_hword(&mut self, t_index: usize, val: u16) {
        let t_index = t_index as u8;
        let new = match *self {
            Timer::Unit(num, ..) if num == t_index => {
                val as u64
            }
            Timer::Joined { bitset: bs, .. } if bs & (1 << t_index) != 0 => {
                self.val() & !(0xFFFF << (16*t_index)) | ((val as u64) << (16*t_index))
            }
            _ => return
        };

        let state = match *self {
            Timer::Unit(_, ref mut state) | Timer::Joined { lowstate: ref mut state, .. } => state
        };
        state.val_cycles = unscale(new, state.prescaler);
    }
}



struct TimerIter<'a> {
    next_timer: u8,
    timer_states: Vec<MutexGuard<'a, TimerState>>,
}

impl<'a> TimerIter<'a> {
    fn new<'b>(states: &'b TimerStates) -> TimerIter<'b> {
        TimerIter {
            next_timer: 3,
            timer_states: states.0.iter().map(|x| x.lock()).collect(),
        }
    }
}

impl<'a> Iterator for TimerIter<'a> {
    type Item = Timer<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.next_timer == 0xFF {
            return None
        }
        let mut item = self.timer_states.pop().unwrap();
        if !item.count_up {
            let ret = Timer::Unit(self.next_timer, item);
            self.next_timer -= 1;
            return Some(ret)
        }

        // assert!(bf!(cnt(0) @ CntReg::count_up) == 0);

        let mut joined_bitset = 0;
        loop {
            joined_bitset |= 1 << self.next_timer;
            self.next_timer = self.next_timer.wrapping_sub(1);
            if !item.count_up || self.next_timer == 0xFF {
                break
            }
            item = self.timer_states.pop().unwrap();
        }
        Some(Timer::Joined { bitset: joined_bitset, lowstate: item })
    }
}

fn scale(cycles: u64, prescaler: Prescaler) -> u64 {
    match prescaler {
        Prescaler::Div1 => cycles,
        Prescaler::Div64 => cycles >> 6,
        Prescaler::Div256 => cycles >> 8,
        Prescaler::Div1024 => cycles >> 10
    }
}

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




#[derive(Clone)]
pub struct TimerStates(Arc<[Mutex<TimerState>; 4]>);
impl TimerStates {
    pub fn new() -> TimerStates {
        TimerStates(Arc::new([
            Mutex::new(TimerState::new()), Mutex::new(TimerState::new()),
            Mutex::new(TimerState::new()), Mutex::new(TimerState::new()),
        ]))
    }
}

impl fmt::Debug for TimerStates {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TimerStates {{ }}")
    }
}

#[derive(Debug)]
pub struct TimerState {
    started: bool,
    count_up: bool,
    val_cycles: u64,
    prescaler: Prescaler,
}

impl TimerState {
    pub fn new() -> TimerState {
        TimerState {
            started: false,
            count_up: false,
            val_cycles: 0,
            prescaler: Prescaler::Div1
        }
    }
}

pub fn handle_clock_update(timer_states: &TimerStates, clock_diff: usize, irq_tx: &mut irq::IrqSyncClient) {
    let iter_started = TimerIter::new(&timer_states).filter(|t| t.started());

    for mut timer in iter_started {
        let overflows = timer.will_overflow_words(clock_diff as u64);
        timer.incr_val(clock_diff as u64);
        for (index, status) in overflows.iter().enumerate() {
            if *status {
                // Overflow happened
                irq_tx.assert(irq(index))
            }
        }
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
