use std::sync::mpsc;

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

pub enum ButtonState {
    Pressed(Button),
    Released(Button)
}

fn reg_pad_read(dev: &mut HidDevice) {
    let mut current_pad = dev.pad.get();
    for change in dev._internal_state.try_iter() {
        match change {
            ButtonState::Pressed(b) => current_pad &= !(1 << b as u32),
            ButtonState::Released(b) => current_pad |= 1 << b as u32
        }
    }
    dev.pad.set_unchecked(current_pad);
}

iodevice!(HidDevice, {
    internal_state: mpsc::Receiver<ButtonState>;
    regs: {
        0x000 => pad: u16 {
            default = !0;
            write_bits = 0;
            read_effect = reg_pad_read;
        }
        0x002 => unk: u16 {
            read_effect = |_| warn!("STUBBED: Read from unknown HID+0x2 register!");
            write_effect = |_| warn!("STUBBED: Write to unknown HID+0x2 register!");
        }
    }
});