use std::fmt;

pub struct OtpDeviceState {
    otp: [u8; 0x100]
}

impl fmt::Debug for OtpDeviceState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "OtpDeviceState {{ }}")
    }
}

impl Default for OtpDeviceState {
    fn default() -> OtpDeviceState {
        OtpDeviceState {
            otp: [0; 0x100]
        }
    }
}

fn reg_otp_write(dev: &mut OtpDevice, buf_pos: usize, source: &[u8]) {
    dev._internal_state.otp[buf_pos .. buf_pos + source.len()].copy_from_slice(source);
}

fn reg_otp_read(dev: &mut OtpDevice, buf_pos: usize, dest: &mut [u8]) {
    let src_slice = &dev._internal_state.otp[buf_pos .. buf_pos + dest.len()];
    dest.clone_from_slice(src_slice);
}

iodevice!(OtpDevice, {
    internal_state: OtpDeviceState;
    regs: {}
    ranges: {
        0x000;0x100 => {
            read_effect = reg_otp_read;
            write_effect = reg_otp_write;
        }
    }
});