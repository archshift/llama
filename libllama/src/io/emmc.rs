use io;

bitfield!(RegCmd: u16, {
    command_index: 0 => 5,
    command_type: 6 => 7,
    response_type: 8 => 10,
    has_data: 11 => 11,
    is_reading: 12 => 12,
    has_multi_block: 13 => 13
});

#[derive(Debug, Default)]
pub struct EmmcDevice {
    reg_cmd: RegCmd,
    reg_port_select: u16,
    reg_cmd_param0: u16,
    reg_cmd_param1: u16,
    reg_stop_internal: u16,
    reg_data16_blk_cnt: u16,
    reg_response0: u16,
    reg_response1: u16,
    reg_response2: u16,
    reg_response3: u16,
    reg_response4: u16,
    reg_response5: u16,
    reg_response6: u16,
    reg_response7: u16,
    reg_irq_status0: u16,
    reg_irq_status1: u16,
    reg_irq_mask0: u16,
    reg_irq_mask1: u16,
    reg_clk_ctl: u16,
    reg_data16_blk_len: u16,
    reg_card_option: u16,
    reg_err_status0: u16,
    reg_err_status1: u16,
    reg_data16_fifo: u16,
    reg_data_ctl: u16,
    reg_software_reset: u16,
}

impl io::IoDeviceRegion for EmmcDevice {
    unsafe fn read_reg(&self, offset: usize, buf: *mut u8, buf_size: usize) {
        match offset {
            0x000 => io::copy_from_reg(&self.reg_cmd, buf, buf_size),
            0x002 => io::copy_from_reg(&self.reg_port_select, buf, buf_size),
            0x004 => io::copy_from_reg(&self.reg_cmd_param0, buf, buf_size),
            0x006 => io::copy_from_reg(&self.reg_cmd_param1, buf, buf_size),
            0x008 => io::copy_from_reg(&self.reg_stop_internal, buf, buf_size),
            0x00A => io::copy_from_reg(&self.reg_data16_blk_cnt, buf, buf_size),
            0x00C => io::copy_from_reg(&self.reg_response0, buf, buf_size),
            0x00E => io::copy_from_reg(&self.reg_response1, buf, buf_size),
            0x010 => io::copy_from_reg(&self.reg_response2, buf, buf_size),
            0x012 => io::copy_from_reg(&self.reg_response3, buf, buf_size),
            0x014 => io::copy_from_reg(&self.reg_response4, buf, buf_size),
            0x016 => io::copy_from_reg(&self.reg_response5, buf, buf_size),
            0x018 => io::copy_from_reg(&self.reg_response6, buf, buf_size),
            0x01A => io::copy_from_reg(&self.reg_response7, buf, buf_size),
            0x01C => io::copy_from_reg(&self.reg_irq_status0, buf, buf_size),
            0x01E => io::copy_from_reg(&self.reg_irq_status1, buf, buf_size),
            0x020 => io::copy_from_reg(&self.reg_irq_mask0, buf, buf_size),
            0x022 => io::copy_from_reg(&self.reg_irq_mask1, buf, buf_size),
            0x024 => io::copy_from_reg(&self.reg_clk_ctl, buf, buf_size),
            0x026 => io::copy_from_reg(&self.reg_data16_blk_len, buf, buf_size),
            0x028 => io::copy_from_reg(&self.reg_card_option, buf, buf_size),
            0x02C => io::copy_from_reg(&self.reg_err_status0, buf, buf_size),
            0x02E => io::copy_from_reg(&self.reg_err_status1, buf, buf_size),
            0x030 => io::copy_from_reg(&self.reg_data16_fifo, buf, buf_size),
            0x0D8 => io::copy_from_reg(&self.reg_data_ctl, buf, buf_size),
            0x0E0 => io::copy_from_reg(&self.reg_software_reset, buf, buf_size),
            // 0x => io::copy_from_reg(&self., buf, buf_size),
            // 0x => io::copy_from_reg(&self., buf, buf_size),
            x @ _ => error!("Unimplemented EMMC read at +0x{:X}", x),
        }
    }

    unsafe fn write_reg(&mut self, offset: usize, buf: *const u8, buf_size: usize) {
        trace!("Writing to EMMC at +0x{:X}", offset);

        match offset {
            0x000 => {
                io::copy_into_reg(&mut self.reg_cmd, buf, buf_size);
                trace!("{:?}", *self);
            },
            0x002 => io::copy_into_reg(&mut self.reg_port_select, buf, buf_size),
            0x004 => io::copy_into_reg(&mut self.reg_cmd_param0, buf, buf_size),
            0x006 => io::copy_into_reg(&mut self.reg_cmd_param1, buf, buf_size),
            0x008 => io::copy_into_reg(&mut self.reg_stop_internal, buf, buf_size),
            0x00A => io::copy_into_reg(&mut self.reg_data16_blk_cnt, buf, buf_size),
            0x00C => io::copy_into_reg(&mut self.reg_response0, buf, buf_size),
            0x00E => io::copy_into_reg(&mut self.reg_response1, buf, buf_size),
            0x010 => io::copy_into_reg(&mut self.reg_response2, buf, buf_size),
            0x012 => io::copy_into_reg(&mut self.reg_response3, buf, buf_size),
            0x014 => io::copy_into_reg(&mut self.reg_response4, buf, buf_size),
            0x016 => io::copy_into_reg(&mut self.reg_response5, buf, buf_size),
            0x018 => io::copy_into_reg(&mut self.reg_response6, buf, buf_size),
            0x01A => io::copy_into_reg(&mut self.reg_response7, buf, buf_size),
            0x01C => io::copy_into_reg(&mut self.reg_irq_status0, buf, buf_size),
            0x01E => io::copy_into_reg(&mut self.reg_irq_status1, buf, buf_size),
            0x020 => io::copy_into_reg(&mut self.reg_irq_mask0, buf, buf_size),
            0x022 => io::copy_into_reg(&mut self.reg_irq_mask1, buf, buf_size),
            0x024 => io::copy_into_reg(&mut self.reg_clk_ctl, buf, buf_size),
            0x026 => io::copy_into_reg(&mut self.reg_data16_blk_len, buf, buf_size),
            0x028 => io::copy_into_reg(&mut self.reg_card_option, buf, buf_size),
            0x02C => io::copy_into_reg(&mut self.reg_err_status0, buf, buf_size),
            0x02E => io::copy_into_reg(&mut self.reg_err_status1, buf, buf_size),
            0x030 => io::copy_into_reg(&mut self.reg_data16_fifo, buf, buf_size),
            0x0D8 => io::copy_into_reg(&mut self.reg_data_ctl, buf, buf_size),
            0x0E0 => self.write_reg_software_reset(offset, buf, buf_size),
            x @ _ => error!("Unimplemented EMMC write at +0x{:X}", x),
        }
    }
}

impl EmmcDevice {
    unsafe fn write_reg_software_reset(&mut self, offset: usize, buf: *const u8, buf_size: usize) {
        let mut tmp: u16 = 0;
        io::copy_into_reg(&mut tmp, buf, buf_size);
        let reset_status = tmp & 0x1;
        self.reg_software_reset &= !0x1; self.reg_software_reset |= reset_status;
    }
}