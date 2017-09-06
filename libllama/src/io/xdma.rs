iodevice!(XdmaDevice, {
    regs: {
        0x000 => dm_status: u32 { }
        0x020 => int_enable: u32 { }
        0x02C => int_clr: u32 { }
        0x100 => csr0: u32 { }
        0xD00 => dbg_status: u32 { }
        0xD04 => dbg_cmd: u32 { }
        0xD08 => dbg_inst0: u32 { }
        0xD0C => dbg_inst1: u32 { }
    }
});