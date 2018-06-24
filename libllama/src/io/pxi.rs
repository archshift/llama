bfdesc!(RegSync: u32, {
    _data_recv: 0 => 7,
    _data_sent: 8 => 15
});

fn reg_sync_write(_dev: &mut PxiDevice) {
    warn!("STUBBED: Write to PXI_SYNC");
}

iodevice!(PxiDevice, {
    regs: {
        0x000 => sync: u32 {
            write_effect = reg_sync_write;
        }
        0x004 => cnt: u16 {}
        0x008 => send: u32 { write_effect = |_| unimplemented!(); }
        0x00C => recv: u32 { read_effect = |_| unimplemented!(); }
    }
});