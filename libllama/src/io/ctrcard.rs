iodevice!(CtrcardDevice, {
    regs: {
        0x000 => cnt: u32 {
            write_effect = |dev: &mut CtrcardDevice| {
                trace!("Wrote 0x{:08X} to CTRCARD CNT register!", dev.cnt.get());
            };
        }
        0x004 => blk_cnt: u32 { }
        0x008 => sec_cnt: u32 { }
        0x010 => sec_seed: u32 { }
        0x020 => cmd0: u32 { }
        0x024 => cmd1: u32 { }
        0x028 => cmd2: u32 { }
        0x02C => cmd3: u32 { }
        0x030 => fifo: u32 { }
    }
});