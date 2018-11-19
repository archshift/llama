iodevice!(Priv11Device, {
    regs: {
        0x000 => scu_ctrl: u32 {
            default = 0x00001FFE;
            write_effect = |_| warn!("STUBBED: Write to ARM11 SCU ctrl register!");
        }
        0x100 => interrupt_ctrl: u32 {
            write_effect = |_| warn!("STUBBED: Write to ARM11 Interrupt ctrl register!");
        }
    }
});

iodevice!(GidDevice, {
    regs: {
        0x000 => dist_ctrl: u32 {
            write_effect = |_| warn!("STUBBED: Write to ARM11 Interrupt Distributor ctrl register!");
        }
    }
});
