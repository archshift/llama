iodevice!(ConfigDevice, {
    regs:
    0x000 => sysprot9: u8 { }
    0x001 => sysprot11: u8 { }
    0x002 => reset11: u8 { }
    0x004 => debugctl: u16 { }
    0x00C => cardctl: u16 { }
    0x010 => cardstatus: u8 { }
    0x012 => cardcycles0: u16 { }
    0x014 => cardcycles1: u16 { }
    0x020 => unknown0: u16 { }
    0x100 => unknown1: u16 { }
    0x200 => extmem_cnt: u8 { }
});

iodevice!(ConfigExtDevice, {
    regs:
    0x000 => bootenv: u32 { }
    0x010 => unitinfo: u8 { }
    0x014 => twl_unitinfo: u8 { }
});