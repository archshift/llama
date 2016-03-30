use io::IoDeviceRegion;

#[derive(Default)]
pub struct ConfigDevice;

impl IoDeviceRegion for ConfigDevice {
    unsafe fn read_reg(&self, offset: usize, buf: *mut u8, buf_size: usize) {
        match offset {
            x @ _ => error!("Unimplemented CONFIG read at +0x{:X}", x),
        }
    }

    unsafe fn write_reg(&self, offset: usize, buf: *const u8, buf_size: usize) {
        match offset {
            x @ _ => error!("Unimplemented CONFIG write at +0x{:X}", x),
        }
    }
}