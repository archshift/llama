iodevice!(ConfigDevice, {
    0x000 => sysprot9: u8 { }
    0x001 => sysprot11: u8 { }
    0x002 => reset11: u8 { }
    0x004 => debugctl: u32 { }
    0x00C => cardctl: u16 { }
    0x010 => cardstatus: u8 { }
    0x012 => cardcycles0: u16 { }
    0x014 => cardcycles1: u16 { }
});