iodevice!(FbufDevice, {
    regs: {
        0x00C => unk0: u32 { }
        0x014 => unk1: u32 { }
        0x240 => top_backlight: u32 { }
        0x244 => top_unk0: u32 { }
        0xA40 => bot_backlight: u32 { }
        0xA44 => bot_unk0: u32 { }
    }
});
