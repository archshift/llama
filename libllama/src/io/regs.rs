use std::ops::BitAnd;
use std::ops::BitAndAssign;
use std::ops::BitOrAssign;
use std::ops::Not;

use utils::bytes;

#[derive(Debug)]
pub struct IoReg<T>
    where T: Copy + BitAnd<Output=T> + BitAndAssign
                  + BitOrAssign + Not<Output=T> {
    val: T,
    write_bits: T,
}
impl<T> IoReg<T>
    where T: Copy + BitAnd<Output=T> + BitAndAssign
                  + BitOrAssign + Not<Output=T> {

    pub fn new(val: T, write_bits: T) -> IoReg<T> {
        IoReg { val: val, write_bits: write_bits }
    }
    pub fn set(&mut self, new_val: T) {
        self.val &= !self.write_bits;
        self.val |= new_val & self.write_bits;
    }
    pub fn _bitadd_unchecked(&mut self, bits: T) {
        self.val |= bits;
    }
    pub fn _bitclr_unchecked(&mut self, bits: T) {
        self.val &= !bits;
    }
    pub fn set_unchecked(&mut self, new_val: T) {
        self.val = new_val;
    }
    pub fn get(&self) -> T {
        self.val
    }
    pub fn ref_mut<'a>(&'a mut self) -> &'a mut T {
        &mut self.val
    }

    pub fn mem_load(&self, buf: &mut [u8]) {
        let data = unsafe { bytes::from_val(&self.val) };
        buf.copy_from_slice(data);
    }

    pub fn mem_save(&mut self, buf: &[u8]) {
        let data = unsafe { bytes::from_mut_val(&mut self.val) };
        data.copy_from_slice(buf);
    }
}

pub trait IoRegAccess {
    fn read_reg(&mut self, offset: usize, buf: &mut [u8]);
    fn write_reg(&mut self, offset: usize, buf: &[u8]);
}


macro_rules! __iodevice__ {
    ($name:ident, {
        $(internal_state: $instate:path;)*
        regs: {$(
            $reg_offs:expr => $reg_name:ident: $reg_ty:ty {
                default = $reg_default:expr;
                write_bits = $reg_wb:expr;
                read_effect = $reg_reff:expr;
                write_effect = $reg_weff:expr;
            }
        )*}
        ranges: {$(
            $range_offs:expr;$range_size:expr => {
                read_effect = $range_reff:expr;
                write_effect = $range_weff:expr;
            }
        )*}
    }) => (
        #[derive(Debug)]
        pub struct $name {
            $( $reg_name: $crate::io::regs::IoReg<$reg_ty>, )*
            $(_internal_state: $instate,)*
        }

        impl $name {
            #[allow(dead_code)]
            pub fn new($(_internal_state: $instate)*) -> $name {
                $name {
                    $( $reg_name: $crate::io::regs::IoReg::new($reg_default, $reg_wb), )*
                    $(_internal_state: { let val: $instate = _internal_state; val }, )*
                }
            }
        }

        impl $crate::io::regs::IoRegAccess for $name {
            fn read_reg(&mut self, offset: usize, buf: &mut [u8]) {
                #![allow(unused_comparisons)]
                trace!("Reading from {} at +0x{:X}", stringify!($name), offset);
                let buf_size = buf.len();
                match offset {
                    $( $reg_offs => {
                        let reg_size = ::std::mem::size_of::<$reg_ty>();
                        assert!(buf_size % reg_size == 0);

                        $reg_reff(&mut *self);
                        self.$reg_name.mem_load(&mut buf[..reg_size]);
                        if buf_size - reg_size > 0 {
                            // Keep going
                            trace!("{} byte read from {}+{:X} greater than reg size {}; including next register.",
                                buf_size, stringify!($name), offset, reg_size);
                            self.read_reg(offset + reg_size, &mut buf[reg_size..]);
                        }
                    })*

                    $( _ if offset >= $range_offs && offset < $range_offs+$range_size => {
                        assert!(offset + buf_size <= $range_offs + $range_size);
                        $range_reff(&mut *self, offset-$range_offs, buf);
                    })*

                    o @ _ => panic!("Unhandled {} register read: {} bytes @ 0x{:X}", stringify!($name), buf_size, o)
                }
            }

            fn write_reg(&mut self, offset: usize, buf: &[u8]) {
                #![allow(unused_comparisons)]
                trace!("Writing to {} at +0x{:X}", stringify!($name), offset);
                let buf_size = buf.len();
                match offset {
                    $( $reg_offs => {
                        let reg_size = ::std::mem::size_of::<$reg_ty>();
                        assert!(buf_size % reg_size == 0);

                        self.$reg_name.mem_save(&buf[..reg_size]);
                        $reg_weff(&mut *self);
                        if buf_size - reg_size > 0 {
                            // Keep going
                            trace!("{} byte write to {}+{:X} greater than reg size {}; including next register.",
                                buf_size, stringify!($name), offset, reg_size);
                            self.write_reg(offset + reg_size, &buf[reg_size..]);
                        }
                    })*

                    $( _ if offset >= $range_offs && offset < $range_offs+$range_size => {
                        assert!(offset + buf_size <= $range_offs + $range_size);
                        $range_weff(&mut *self, offset-$range_offs, buf);
                    })*

                    o @ _ => panic!("Unhandled {} register write: {} bytes @ 0x{:X}", stringify!($name), buf_size, o)
                }
            }
        }
    )
}

macro_rules! __iodevice_desc_default__ {
    ($val:expr) => ($val);
    () => (0);
}

macro_rules! __iodevice_desc_wb__ {
    ($val:expr) => ($val);
    () => (!0);
}

macro_rules! __iodevice_desc_reg_eff__ {
    ($val:expr) => ($val);
    () => (|_|{});
}

macro_rules! __iodevice_desc_range_eff__ {
    ($val:expr) => ($val);
    () => (|_, _, _|{});
}

#[macro_export]
macro_rules! iodevice {
    ($name:ident, {
        $(internal_state: $instate:path;)*
        regs: {$(
            $reg_offs:expr => $reg_name:ident: $reg_ty:ty {
                $(default = $reg_default:expr;)*
                $(write_bits = $reg_wb:expr;)*
                $(read_effect = $reg_reff:expr;)*
                $(write_effect = $reg_weff:expr;)*
            }
        )*}
        $(ranges: {$(
            $range_offs:expr;$range_size:expr => {
                $(read_effect = $range_reff:expr;)*
                $(write_effect = $range_weff:expr;)*
            }
        )*})*
    }) => (
        __iodevice__!($name, {
            $(internal_state: $instate;)*
            regs: {$(
                $reg_offs => $reg_name: $reg_ty {
                    default = __iodevice_desc_default__!($($reg_default),*);
                    write_bits = __iodevice_desc_wb__!($($reg_wb),*);
                    read_effect = __iodevice_desc_reg_eff__!($($reg_reff),*);
                    write_effect = __iodevice_desc_reg_eff__!($($reg_weff),*);
                }
            )*}
            ranges: {$($(
                $range_offs;$range_size => {
                    read_effect = __iodevice_desc_range_eff__!($($range_reff),*);
                    write_effect = __iodevice_desc_range_eff__!($($range_weff),*);
                }
            )*)*}
        });
    );
}


#[cfg(test)]
mod test {
    use super::*;

    iodevice!(MMCRegs, {
        regs: {
            0x000 => reg0: u16 { }
            0x002 => reg2: u16 {
                write_effect = |_dev| { panic!("while writing") };
            }
            0x004 => reg4: u16 { write_bits = 0; }
        }
    });

    #[test]
    fn read_reg() {
        let mut mmc_regs = MMCRegs::new();
        let mut buf = vec![0xFFu8; 2];
        unsafe { mmc_regs.read_reg(0x000, buf.as_mut_ptr(), buf.len()); }
        assert_eq!(buf, vec![0x00, 0x00]);
    }

    #[test]
    fn write_reg() {
        let mut mmc_regs = MMCRegs::new();
        assert_eq!(mmc_regs.reg0.get(), 0x0000);

        let buf = vec![0xFFu8; 2];
        unsafe { mmc_regs.write_reg(0x000, buf.as_ptr(), buf.len()); }
        assert_eq!(mmc_regs.reg0.get(), 0xFFFF);
    }

    #[test]
    #[should_panic]
    fn write_effect() {
        let mut mmc_regs = MMCRegs::new();
        let buf = vec![0xFFu8; 2];
        unsafe { mmc_regs.write_reg(0x002, buf.as_ptr(), buf.len()); }
    }
}
