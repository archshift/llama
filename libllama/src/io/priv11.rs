use std::rc::Rc;
use std::cell::RefCell;

use cpu::irq::Aggregator;

iodevice!(Priv11Device, {
    internal_state: Rc<RefCell<Aggregator>>;
    regs: {
        0x000 => scu_ctrl: u32 {
            default = 0x00001FFE;
            write_effect = |_| warn!("STUBBED: Write to ARM11 SCU ctrl register!");
        }
        0x00C => scu_invalidate_all: u32 {}
        0x100 => interrupt_ctrl: u32 {
            write_effect = |_| warn!("STUBBED: Write to ARM11 Interrupt ctrl register!");
        }
        0x104 => interrupt_prio_mask: u32 {
            write_effect = |_| warn!("STUBBED: Write to ARM11 Interrupt priority mask register!");
        }
        0x108 => interrupt_preemptable: u32 {
            write_effect = |_| warn!("STUBBED: Write to ARM11 Interrupt binary point register!");
        }
        0x10C => interrupt_ack: u32 {
            default = 1023;
            write_bits = 0;
            read_effect = |dev: &mut Priv11Device| {
                let mut next_interrupt = dev._internal_state.borrow_mut()
                    .drain_asserts().trailing_zeros();
                if next_interrupt == 128 {
                    next_interrupt = 1023;
                }
                dev.interrupt_ack.set_unchecked(next_interrupt);
                warn!("STUBBED: Read from ARM11 Acknowledge Interrupt register!")
            };
        }
        0x110 => interrupt_end: u32 {
            write_effect = |dev: &mut Priv11Device| {
                let which = dev.interrupt_end.get();
                let which_mask = 1u128 << (which as usize);
                dev._internal_state.borrow_mut().acknowledge(which_mask);
                warn!("STUBBED: Write {} to ARM11 Interrupt end register!", which);
            };
        }
        0x118 => interrupt_highest_pending: u32 {
            default = 1023;
            write_bits = 0;
            read_effect = |dev: &mut Priv11Device| {
                let mut next_interrupt = dev._internal_state.borrow_mut()
                    .drain_asserts().trailing_zeros();
                if next_interrupt == 128 {
                    next_interrupt = 1023;
                }
                dev.interrupt_highest_pending.set_unchecked(next_interrupt);
                warn!("STUBBED: Read {} from ARM11 Highest Pending Interrupt register!", next_interrupt)
            };
        }

        0x600 => timer_load: u32 {
            write_effect = |dev: &mut Priv11Device| {
                warn!("STUBBED: Write {:08X} to ARM11 Timer load register!", dev.timer_load.get());
            };
        }
        0x604 => timer_counter: u32 {
            read_effect = |dev: &mut Priv11Device| {
                warn!("STUBBED: Read {:08X} from ARM11 Timer counter register!", dev.timer_counter.get());
            };
            write_effect = |dev: &mut Priv11Device| {
                warn!("STUBBED: Write {:08X} to ARM11 Timer counter register!", dev.timer_counter.get());
            };
        }
        0x608 => timer_ctrl: u32 {
            write_effect = |dev: &mut Priv11Device| {
                warn!("STUBBED: Write {:08X} to ARM11 Timer control register!", dev.timer_ctrl.get());
            };
        }
        0x60C => timer_interrupt_stat: u32 {
            write_effect = |dev: &mut Priv11Device| {
                warn!("STUBBED: Write {:08X} to ARM11 Timer interrupt status register!", dev.timer_interrupt_stat.get());
            };
        }
        0x628 => wdg_ctrl: u32 {
            write_effect = |dev: &mut Priv11Device| {
                warn!("STUBBED: Write {:08X} to ARM11 Watchdog control register!", dev.wdg_ctrl.get());
            };
        }
        0x62C => wdg_interrupt_stat: u32 {
            write_effect = |dev: &mut Priv11Device| {
                warn!("STUBBED: Write {:08X} to ARM11 Watchdog interrupt status register!", dev.wdg_interrupt_stat.get());
            };
        }
        0x630 => wdg_reset_sent: u32 {
            write_effect = |dev: &mut Priv11Device| {
                warn!("STUBBED: Write {:08X} to ARM11 Watchdog reset sent register!", dev.wdg_reset_sent.get());
            };
        }
        0x634 => wdg_disable: u32 {
            write_effect = |dev: &mut Priv11Device| {
                warn!("STUBBED: Write {:08X} to ARM11 Watchdog disable register!", dev.wdg_disable.get());
            };
        }
    }
});

#[derive(Debug)]
pub struct GidState {
    agg: Rc<RefCell<Aggregator>>,
    enabled: [u32; 4],
}

impl GidState {
    pub fn new(agg: Rc<RefCell<Aggregator>>) -> Self {
        Self {
            agg,
            enabled: [0;4],
        }
    }
}

fn update_enabled(dev: &mut GidDevice) {
    let enabled = {
        let enabled = &dev._internal_state.enabled;
        ((enabled[3] as u128) << 96)
            | ((enabled[2] as u128) << 64)
            | ((enabled[1] as u128) << 32)
            | (enabled[0] as u128)
    };
    dev._internal_state.agg.borrow().set_enabled(enabled);
}

macro_rules! enable_setX {
    ($reg:ident, $i:expr) => (|dev: &mut GidDevice| {
        dev._internal_state.enabled[$i] |= dev.$reg.get();
        dev.$reg.set_unchecked(0);
        update_enabled(dev);
    })
}

macro_rules! enable_clrX {
    ($reg:ident, $i:expr) => (|dev: &mut GidDevice| {
        dev._internal_state.enabled[$i] &= !dev.$reg.get();
        dev.$reg.set_unchecked(0);
        update_enabled(dev);
    })
}

macro_rules! pending_clrX {
    ($reg:ident, $i:expr) => (|dev: &mut GidDevice| {
        let which = (dev.$reg.get() as u128) << ($i * 32);
        dev._internal_state.agg.borrow_mut().acknowledge(which);
    })
}

macro_rules! pending_setX {
    ($reg:ident, $i:expr) => (|dev: &mut GidDevice| {
        let which = (dev.$reg.get() as u128) << ($i * 32);
        warn!("STUBBED: Force set ARM11 interrupts with val {:X}", which);
    })
}

macro_rules! pending_getX {
    ($reg:ident, $i:expr) => (|dev: &mut GidDevice| {
        let asserted = dev._internal_state.agg
            .borrow_mut().drain_asserts();
        dev.$reg.set_unchecked((asserted >> ($i * 32)) as u32);
    })
}


iodevice!(GidDevice, {
    internal_state: GidState;
    regs: {
        0x000 => dist_ctrl: u32 {
            write_effect = |_| warn!("STUBBED: Write to ARM11 Interrupt Distributor ctrl register!");
        }

        0x100 => enable_set0: u32 {
            write_effect = enable_setX!(enable_set0, 0);
        }
        0x104 => enable_set1: u32 {
            write_effect = enable_setX!(enable_set1, 1);
        }
        0x108 => enable_set2: u32 {
            write_effect = enable_setX!(enable_set2, 2);
        }
        0x10C => enable_set3: u32 {
            write_effect = enable_setX!(enable_set3, 3);
        }

        0x180 => enable_clr0: u32 {
            write_effect = enable_clrX!(enable_clr0, 0);
        }
        0x184 => enable_clr1: u32 {
            write_effect = enable_clrX!(enable_clr1, 1);
        }
        0x188 => enable_clr2: u32 {
            write_effect = enable_clrX!(enable_clr2, 2);
        }
        0x18C => enable_clr3: u32 {
            write_effect = enable_clrX!(enable_clr3, 3);
        }

        0x200 => pending_set0: u32 {
            read_effect = pending_getX!(pending_set0, 0);
            write_effect = pending_setX!(pending_set0, 0);
        }
        0x204 => pending_set1: u32 {
            read_effect = pending_getX!(pending_set1, 1);
            write_effect = pending_setX!(pending_set1, 1);
        }
        0x208 => pending_set2: u32 {
            read_effect = pending_getX!(pending_set2, 2);
            write_effect = pending_setX!(pending_set2, 2);
        }
        0x20C => pending_set3: u32 {
            read_effect = pending_getX!(pending_set3, 3);
            write_effect = pending_setX!(pending_set3, 3);
        }

        0x280 => pending_clr0: u32 {
            read_effect = pending_getX!(pending_clr0, 0);
            write_effect = pending_clrX!(pending_clr0, 0);
        }
        0x284 => pending_clr1: u32 {
            read_effect = pending_getX!(pending_clr1, 1);
            write_effect = pending_clrX!(pending_clr1, 1);
        }
        0x288 => pending_clr2: u32 {
            read_effect = pending_getX!(pending_clr2, 2);
            write_effect = pending_clrX!(pending_clr2, 2);
        }
        0x28C => pending_clr3: u32 {
            read_effect = pending_getX!(pending_clr3, 3);
            write_effect = pending_clrX!(pending_clr3, 3);
        }
    }
    ranges: {
        0x400;0x100 => {
            write_effect = |_, _buf_pos, _src| {
                warn!("STUBBED: Set ARM11 Interrupt Priority");
            };
        }
        0x800;0x100 => {
            write_effect = |_, _buf_pos, _src| {
                warn!("STUBBED: Set ARM11 Interrupt Target");
            };
        }
        0xC00;0x100 => {
            write_effect = |_, _buf_pos, _src| {
                warn!("STUBBED: Set ARM11 Interrupt Configuration");
            };
        }
    }
});
