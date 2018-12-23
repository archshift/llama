iodevice!(Priv11Device, {
    regs: {
        0x000 => scu_ctrl: u32 {
            default = 0x00001FFE;
            write_effect = |_| warn!("STUBBED: Write to ARM11 SCU ctrl register!");
        }
        0x100 => interrupt_ctrl: u32 {
            write_effect = |_| warn!("STUBBED: Write to ARM11 Interrupt ctrl register!");
        }
        0x104 => interrupt_prio_mask: u32 {
            write_effect = |_| warn!("STUBBED: Write to ARM11 Interrupt priority mask register!");
        }
        0x108 => interrupt_preemptable: u32 {
            write_effect = |_| warn!("STUBBED: Write to ARM11 Interrupt binary point register!");
        }
        0x110 => interrupt_end: u32 {
            write_effect = |_| warn!("STUBBED: Write to ARM11 Interrupt end register!");
        }
        0x118 => interrupt_highest_pending: u32 {
            default = 1023;
            write_bits = 0;
            read_effect = |_| warn!("STUBBED: Read from ARM11 Highest Pending Interrupt register!");
        }
    }
});

#[derive(Default, Debug)]
pub struct GidState {
    enabled: [u32; 4],
    pending: [u32; 4],
}

iodevice!(GidDevice, {
    internal_state: GidState;
    regs: {
        0x000 => dist_ctrl: u32 {
            write_effect = |_| warn!("STUBBED: Write to ARM11 Interrupt Distributor ctrl register!");
        }

        0x100 => enable_set0: u32 {
            write_effect = |dev: &mut GidDevice| { dev._internal_state.enabled[0] |= dev.enable_set0.get(); dev.enable_set0.set_unchecked(0) };
        }
        0x104 => enable_set1: u32 {
            write_effect = |dev: &mut GidDevice| { dev._internal_state.enabled[1] |= dev.enable_set1.get(); dev.enable_set1.set_unchecked(0) };
        }
        0x108 => enable_set2: u32 {
            write_effect = |dev: &mut GidDevice| { dev._internal_state.enabled[2] |= dev.enable_set2.get(); dev.enable_set2.set_unchecked(0) };
        }
        0x10C => enable_set3: u32 {
            write_effect = |dev: &mut GidDevice| { dev._internal_state.enabled[3] |= dev.enable_set3.get(); dev.enable_set3.set_unchecked(0) };
        }

        0x180 => enable_clr0: u32 {
            write_effect = |dev: &mut GidDevice| { dev._internal_state.enabled[0] &= !dev.enable_clr0.get(); dev.enable_clr0.set_unchecked(0) };
        }
        0x184 => enable_clr1: u32 {
            write_effect = |dev: &mut GidDevice| { dev._internal_state.enabled[1] &= !dev.enable_clr1.get(); dev.enable_clr1.set_unchecked(0) };
        }
        0x188 => enable_clr2: u32 {
            write_effect = |dev: &mut GidDevice| { dev._internal_state.enabled[2] &= !dev.enable_clr2.get(); dev.enable_clr2.set_unchecked(0) };
        }
        0x18C => enable_clr3: u32 {
            write_effect = |dev: &mut GidDevice| { dev._internal_state.enabled[3] &= !dev.enable_clr3.get(); dev.enable_clr3.set_unchecked(0) };
        }

        0x280 => pending_clr0: u32 {
            write_effect = |_| warn!("STUBBED: Cleared ARM11 Interrupt Pending (0)!");
        }
        0x284 => pending_clr1: u32 {
            write_effect = |_| warn!("STUBBED: Cleared ARM11 Interrupt Pending (1)!");
        }
        0x288 => pending_clr2: u32 {
            write_effect = |_| warn!("STUBBED: Cleared ARM11 Interrupt Pending (2)!");
        }
        0x28C => pending_clr3: u32 {
            write_effect = |_| warn!("STUBBED: Cleared ARM11 Interrupt Pending (3)!");
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
