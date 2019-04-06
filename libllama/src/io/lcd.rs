iodevice!(LcdDevice, {
    regs: {
        0x000 => parallax_enable: u32 { }
        0x004 => unk0: u32 { }
        0x008 => unk1: u32 { }
        0x00C => unk2: u32 { }
        0x014 => unk3: u32 { }
        0x200 => top_unk0: u32 { }
        0x204 => top_fill: u32 {
            write_effect = |dev: &LcdDevice| {
                let fill = dev.top_fill.get();
                if (fill & (1 << 24) != 0) {
                    warn!("Enabling top screen pixel fill! Color: #{:06X}", fill & 0xFFFFFF);
                }
            };
        }
        0x238 => top_unk1: u32 { }
        0x240 => top_backlight: u32 { }
        0x244 => top_unk2: u32 { }
        0xA00 => bot_unk0: u32 { }
        0xA04 => bot_fill: u32 {
            write_effect = |dev: &LcdDevice| {
                let fill = dev.bot_fill.get();
                if (fill & (1 << 24) != 0) {
                    warn!("Enabling bottom screen pixel fill! Color: #{:06X}", fill & 0xFFFFFF);
                }
            };
        }
        0xA38 => bot_unk1: u32 { }
        0xA40 => bot_backlight: u32 { }
        0xA44 => bot_unk2: u32 { }
    }
    ranges: {
        0x400;0x400 => { } // top unknown region
        0xC00;0x400 => { } // bottom unknown region
    }

});
