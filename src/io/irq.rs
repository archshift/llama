use io::IoDeviceRegion;

#[derive(Default)]
pub struct IrqDevice;

impl IoDeviceRegion for IrqDevice {
    unsafe fn read_reg(&self, offset: usize, buf: *mut u8, buf_size: usize) {
        match offset {
            x @ _ => error!("Unimplemented IRQ read at +0x{:X}", x),
        }
    }

    unsafe fn write_reg(&self, offset: usize, buf: *const u8, buf_size: usize) {
        match offset {
            x @ _ => error!("Unimplemented IRQ write at +0x{:X}", x),
        }
    }
}