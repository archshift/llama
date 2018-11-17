iodevice!(FbufDevice, {
    regs: {
        0x240 => top_backlight: u32 { }
        0xA40 => bot_backlight: u32 { }
    }
});
