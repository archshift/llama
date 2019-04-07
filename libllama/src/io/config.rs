iodevice!(ConfigDevice, {
    regs: {
        0x000 => sysprot9: u8 {
            write_effect = |dev: &ConfigDevice| {
                if dev.sysprot9.get() != 0 {
                    panic!("Protecting ARM9 bootrom!");
                }
            };
        }
        0x001 => sysprot11: u8 {
            write_effect = |dev: &ConfigDevice| {
                if dev.sysprot11.get() != 0 {
                    panic!("Protecting ARM11 bootrom!");
                }
            };
        }
        0x002 => reset11: u8 { }
        0x004 => debugctl: u16 { }
        0x008 => unknown0: u8 {
            read_effect = |_| warn!("STUBBED: Read from unknown CONFIG+0x8 register!");
            write_effect = |_| warn!("STUBBED: Write to unknown CONFIG+0x8 register!");
        }
        0x00C => cardctl: u16 { }
        0x010 => cardstatus: u8 { }
        0x012 => cardcycles0: u16 { }
        0x014 => cardcycles1: u16 { }
        0x020 => sdmmcctl: u16 { }
        0x022 => unknown1: u16 {
            read_effect = |_| warn!("STUBBED: Read from unknown CONFIG+0x22 register!");
            write_effect = |_| warn!("STUBBED: Write to unknown CONFIG+0x22 register!");
        }
        0x100 => unknown2: u16 {
            read_effect = |_| warn!("STUBBED: Read from unknown CONFIG+0x100 register!");
            write_effect = |_| warn!("STUBBED: Write to unknown CONFIG+0x100 register!");
        }
        0x200 => extmem_cnt: u8 { }
    }
});

iodevice!(ConfigExtDevice, {
    regs: {
        0x000 => bootenv: u32 { }
        0x010 => unitinfo: u8 { }
        0x014 => twl_unitinfo: u8 { }
    }
});
