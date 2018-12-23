use cpu::irq::Aggregator;

iodevice!(IrqDevice, {
    internal_state: Aggregator;
    regs: {
        0x000 => enabled: u32 {
            write_bits = 0b00111111_11111111_11111111_11111111;
            write_effect = |dev: &mut IrqDevice| {
                let state = &mut dev._internal_state;
                state.set_enabled(dev.enabled.get() as u128);
            };
        }
        0x004 => pending: u32 {
            write_bits = 0b00111111_11111111_11111111_11111111;
            read_effect = |dev: &mut IrqDevice| {
                let state = &mut dev._internal_state;
                dev.pending.set_unchecked(state.drain_asserts() as u32);
            };
            write_effect = |dev: &mut IrqDevice| {
                let state = &mut dev._internal_state;
                let new_pending = state.acknowledge(dev.pending.get() as u128);
                dev.pending.set_unchecked(new_pending as u32);
            };
        }
    }
});
