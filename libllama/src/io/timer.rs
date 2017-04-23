use io::regs::IoReg;

enum Prescaler {
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

bfdesc!(CntReg: u16, {
    prescaler: 0 => 1,
    count_up: 2 => 2,
    irq_enable: 6 => 6,
    started: 7 => 7
});

#[derive(Debug, Default)]
pub struct TimerDeviceState {
    cycles: u64,
    started: [bool; 4],
    start_cycles: [u64; 4],
}

fn get_regs(dev: &mut TimerDevice, index: usize) -> (&mut IoReg<u16>, &mut IoReg<u16>) {
    match index {
        0 => (&mut dev.val0, &mut dev.cnt0),
        1 => (&mut dev.val1, &mut dev.cnt1),
        2 => (&mut dev.val2, &mut dev.cnt2),
        3 => (&mut dev.val3, &mut dev.cnt3),
        _ => unreachable!()
    }
}

fn reg_cnt_update(dev: &mut TimerDevice, index: usize) {
    let now_active = {
        let (_, cnt) = get_regs(dev, index);
        bf!((cnt.get()) @ CntReg::started) == 1
    };
    match (now_active, dev._internal_state.started[index]) {
        /* On start */ (true, false) => dev._internal_state.start_cycles[index] = dev._internal_state.cycles,
        /* On stop */  (false, true) => {},
        /* Not changed */          _ => {}
    }
}

fn reg_val_read(dev: &mut TimerDevice, mut index: usize) {
    dev._internal_state.cycles += 0x20000; // TODO: not-stubbed timer incrementing

    {
        let (_, cnt) = get_regs(dev, index);
        if bf!((cnt.get()) @ CntReg::count_up) == 1 {
            index = (index + 3) % 4; // TODO: Verify that TIMER0 uses TIMER3 to count up
        }
    }

    let diff_cycles = dev._internal_state.cycles - dev._internal_state.start_cycles[index];

    let new_val = {
        let (val, cnt) = get_regs(dev, index);
        let prescaler = bf!((cnt.get()) @ CntReg::prescaler);

        let new_val = match Prescaler::new(prescaler) {
            Prescaler::Div1 => diff_cycles,
            Prescaler::Div64 => diff_cycles >> 6,
            Prescaler::Div256 => diff_cycles >> 8,
            Prescaler::Div1024 => diff_cycles >> 10,
            _ => unreachable!(),
        };
        val.set_unchecked(new_val as u16);
        new_val
    };

    {
        // TODO: Verify that TIMER0 uses TIMER3 to count up
        let (val, cnt) = get_regs(dev, (index + 1) % 4);
        if bf!((cnt.get()) @ CntReg::count_up) == 1 {
            val.set_unchecked((new_val >> 16) as u16)
        }
    }
}

iodevice!(TimerDevice, {
    internal_state: TimerDeviceState;
    regs: {
        0x000 => val0: u16 { write_effect = |dev: &mut TimerDevice| reg_cnt_update(dev, 0); }
        0x002 => cnt0: u16 { read_effect = |dev: &mut TimerDevice| reg_val_read(dev, 0); }

        0x004 => val1: u16 { write_effect = |dev: &mut TimerDevice| reg_cnt_update(dev, 1); }
        0x006 => cnt1: u16 { read_effect = |dev: &mut TimerDevice| reg_val_read(dev, 1); }

        0x008 => val2: u16 { write_effect = |dev: &mut TimerDevice| reg_cnt_update(dev, 2); }
        0x00A => cnt2: u16 { read_effect = |dev: &mut TimerDevice| reg_val_read(dev, 2); }

        0x00C => val3: u16 { write_effect = |dev: &mut TimerDevice| reg_cnt_update(dev, 3); }
        0x00E => cnt3: u16 { read_effect = |dev: &mut TimerDevice| reg_val_read(dev, 3); }
    }
});