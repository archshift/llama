use cpu::irq::IrqRequests;

iodevice!(IrqDevice, {
    internal_state: IrqRequests;
    regs: {
        0x000 => enabled: u32 {
            write_bits = 0b00111111_11111111_11111111_11111111;
            write_effect = |dev: &mut IrqDevice| {
                dev._internal_state.set_enabled(dev.enabled.get());
            };
        }
        0x004 => pending: u32 {
            write_bits = 0b00111111_11111111_11111111_11111111;
            read_effect = |dev: &mut IrqDevice| {
                dev.pending.set_unchecked(dev._internal_state.get_pending());
            };
            write_effect = |dev: &mut IrqDevice| {
                let new_pending = dev._internal_state.acknowledge(dev.pending.get());
                dev.pending.set_unchecked(new_pending);
            };
        }
    }
});