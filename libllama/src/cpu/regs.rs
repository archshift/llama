use cpu;

use std::iter::Iterator;
use std::ops;

// Program status register
bf!(Psr[u32] {
    mode: 0:4,
    thumb_bit: 5:5,
    disable_fiq_bit: 6:6,
    disable_irq_bit: 7:7,
    q_bit: 27:27,
    v_bit: 28:28,
    c_bit: 29:29,
    z_bit: 30:30,
    n_bit: 31:31
});

#[derive(Debug)]
pub struct GpRegs {
    active: [u32; 16],
    mode: cpu::Mode,
    banks: GpBanks
}

impl GpRegs {
    pub fn new(mode: cpu::Mode) -> GpRegs {
        GpRegs {
            active: [0; 16],
            mode: mode,
            banks: GpBanks {
                basic_bank: [0; 13],
                usr_bank: [0; 2],
                svc_bank: [0; 2],
                abt_bank: [0; 2],
                und_bank: [0; 2],
                irq_bank: [0; 2],
                fiq_bank: [0; 8]
            }
        }
    }

    pub fn swap(&mut self, mode: cpu::Mode) {
        trace!("Swapping register banks: mode {:?} to {:?}", self.mode, mode);
        { // Save active regs to bank
            let mut iter = self.banks.build_iter(self.mode);
            for i in 0..15 {
                *iter.next().unwrap() = self.active[i];
            }
        }
        { // Load bank into active regs
            let mut iter = self.banks.build_iter(mode);
            for i in 0..15 {
                self.active[i] = *iter.next().unwrap();
            }
        }
        self.mode = mode;
    }
}

impl ops::Index<usize> for GpRegs {
    type Output = u32;
    fn index(&self, i: usize) -> &Self::Output {
        &self.active[i]
    }
}

impl ops::IndexMut<usize> for GpRegs {
    fn index_mut<'a>(&'a mut self, i: usize) -> &mut Self::Output {
        &mut self.active[i]
    }
}

// Helper struct to contain all the mode-dependent register banks
#[derive(Debug)]
struct GpBanks {
    basic_bank: [u32; 13],
    usr_bank: [u32; 2],
    svc_bank: [u32; 2],
    abt_bank: [u32; 2],
    und_bank: [u32; 2],
    irq_bank: [u32; 2],
    fiq_bank: [u32; 8],
}

impl GpBanks {
    fn build_iter<'a>(&'a mut self, mode: cpu::Mode) -> impl Iterator<Item=&mut u32> + 'a {
        match mode {
            cpu::Mode::Sys | cpu::Mode::Usr
                           => Box::new(self.basic_bank.iter_mut().take(13).chain(self.usr_bank.iter_mut())),
            cpu::Mode::Svc => Box::new(self.basic_bank.iter_mut().take(13).chain(self.svc_bank.iter_mut())),
            cpu::Mode::Abt => Box::new(self.basic_bank.iter_mut().take(13).chain(self.abt_bank.iter_mut())),
            cpu::Mode::Und => Box::new(self.basic_bank.iter_mut().take(13).chain(self.und_bank.iter_mut())),
            cpu::Mode::Irq => Box::new(self.basic_bank.iter_mut().take(13).chain(self.irq_bank.iter_mut())),
            cpu::Mode::Fiq => Box::new(self.basic_bank.iter_mut().take(7).chain(self.fiq_bank.iter_mut()))
        }
    }
}
