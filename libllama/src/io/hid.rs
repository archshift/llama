#[derive(Clone, Copy)]
pub enum Button {
    A = 0,
    B = 1,
    Select = 2,
    Start = 3,
    Right = 4,
    Left = 5,
    Up = 6,
    Down = 7,
    R = 8,
    L = 9,
    X = 10,
    Y = 11
}

#[derive(Clone, Copy)]
pub enum ButtonState {
    Pressed(Button),
    Released(Button)
}

pub fn update_pad(dev: &mut HidDevice, change: ButtonState) {
    let mut current_pad = dev.pad.get();
     match change {
        ButtonState::Pressed(b) => current_pad &= !(1 << b as u32),
        ButtonState::Released(b) => current_pad |= 1 << b as u32
    }
    dev.pad.set_unchecked(current_pad);
}

iodevice!(HidDevice, {
    regs: {
        0x000 => pad: u16 {
            default = !0;
            write_bits = 0;
        }
        0x002 => unk: u16 {
            read_effect = |_| trace!("STUBBED: Read from unknown HID+0x2 register!");
            write_effect = |_| warn!("STUBBED: Write to unknown HID+0x2 register!");
        }
    }
});