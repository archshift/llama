use cpu::coproc::Coprocessor;

bitfield!(RegControl: u32, {
    use_mpu: 0 => 0,
    align_fault: 1 => 1,
    use_dcache: 2 => 2,
    write_buf: 3 => 3,
    big_endian: 7 => 7,
    sys_protect: 8 => 8,
    rom_protect: 9 => 9,
    f_bit: 10 => 10,
    predict_branches: 11 => 11,
    use_icache: 12 => 12,
    high_vectors: 13 => 13,
    predictable_cache: 14 => 14,
    disable_thumb: 15 => 15,
    low_latency_frq: 21 => 21,
    allow_unaligned: 22 => 22,
    disable_subpage_ap: 23 => 23,
    vectored_interrupts: 24 => 24,
    mixed_endian_exceptions: 25 => 25,
    use_l2cache: 26 => 26
});

pub struct SysControl {
    r1_control: RegControl,
    r2_dcacheability: u32,
    r2_icacheability: u32,
    r3_bufferability: u32,
    r5_daccessperms: u32,
    r5_iaccessperms: u32,
    r6_memregions: [u32; 8],
    r9_dcache_lockdown: u32,
    r9_icache_lockdown: u32,
    r9_dtcm_size: u32,
    r9_itcm_size: u32,
}

impl SysControl {
    pub fn new() -> SysControl {
        SysControl {
            r1_control: RegControl::new(0),
            r2_dcacheability: 0,
            r2_icacheability: 0,
            r3_bufferability: 0,
            r5_daccessperms: 0,
            r5_iaccessperms: 0,
            r6_memregions: [0; 8],
            r9_dcache_lockdown: 0,
            r9_icache_lockdown: 0,
            r9_dtcm_size: 0,
            r9_itcm_size: 0,
        }
    }
}

impl Coprocessor for SysControl {
    fn move_in(&mut self, cpreg1: usize, cpreg2: usize, op1: usize, op2: usize, val: u32) {
        assert_eq!(op1, 0);

        match cpreg1 {
            1 => match op2 {
                0b000 => {
                    warn!("STUBBED: System control register write");
                    self.r1_control.set_raw(val);
                }
                0b001 | 0b010 => unimplemented!(),
                _ => unreachable!()
            },

            2 => match op2 {
                0 => {
                    warn!("STUBBED: DCache cacheability register write");
                    self.r2_dcacheability = val;
                }
                1 => {
                    warn!("STUBBED: ICache cacheability register write");
                    self.r2_icacheability = val;
                }
                _ => unreachable!()
            },

            3 => {
                warn!("STUBBED: Data bufferability register write");
                self.r3_bufferability = val;
            }

            5 => match op2 {
                0 | 1 => unimplemented!(),
                2 => {
                    warn!("STUBBED: Instr access perms register write");
                    self.r5_daccessperms = val;
                }
                3 => {
                    warn!("STUBBED: Data access perms register write");
                    self.r5_iaccessperms = val;
                }
                _ => unreachable!()
            },

            6 => {
                warn!("STUBBED: MPU region {} register write", cpreg2);
                self.r6_memregions[cpreg2] = val
            }

            7 => warn!("STUBBED: Cache control register write; reg2={}, op2={}", cpreg2, op2),

            9 => match (cpreg2, op2) {
                (0, 0) => {
                    warn!("STUBBED: DCache lockdown register write");
                    self.r9_dcache_lockdown = val;
                }
                (0, 1) => {
                    warn!("STUBBED: ICache lockdown register write");
                    self.r9_icache_lockdown = val;
                }
                (1, 0) => {
                    warn!("STUBBED: DTCM size register write");
                    self.r9_dtcm_size = val;
                }
                (1, 1) => {
                    warn!("STUBBED: ITCM size register write");
                    self.r9_itcm_size = val;
                }
                _ => unreachable!()
            },

            _ => panic!("Unimplemented CP15 write to coproc reg {}", cpreg1)
        }

        info!("Write 0x{:08X} to CP15 reg {}; reg2={}, op2={}", val, cpreg1, cpreg2, op2);
    }

    fn move_out(&mut self, cpreg1: usize, cpreg2: usize, op1: usize, op2: usize) -> u32 {
        assert_eq!(op1, 0);

        let res = match cpreg1 {
            1 => match op2 {
                0b000 => {
                    warn!("STUBBED: System control register read");
                    self.r1_control.raw()
                }
                0b001 | 0b010 => unimplemented!(),
                _ => unreachable!()
            },

            2 => match op2 {
                0 => {
                    warn!("STUBBED: DCache cacheability register read");
                    self.r2_dcacheability
                }
                1 => {
                    warn!("STUBBED: ICache cacheability register read");
                    self.r2_icacheability
                }
                _ => unreachable!()
            },

            3 => {
                warn!("STUBBED: Data bufferability register read");
                self.r3_bufferability
            }

            5 => match op2 {
                0 | 1 => unimplemented!(),
                2 => {
                    warn!("STUBBED: Instr access perms register read");
                    self.r5_daccessperms
                }
                3 => {
                    warn!("STUBBED: Data access perms register read");
                    self.r5_iaccessperms
                }
                _ => unreachable!()
            },

            6 => {
                warn!("STUBBED: MPU region {} register read", cpreg2);
                self.r6_memregions[cpreg2]
            }

            7 => panic!("Cannot read from cache control register!"),

            9 => match (cpreg2, op2) {
                (0, 0) => {
                    warn!("STUBBED: DCache lockdown register read");
                    self.r9_dcache_lockdown
                }
                (0, 1) => {
                    warn!("STUBBED: ICache lockdown register read");
                    self.r9_icache_lockdown
                }
                (1, 0) => {
                    warn!("STUBBED: DTCM size register read");
                    self.r9_dtcm_size
                }
                (1, 1) => {
                    warn!("STUBBED: ITCM size register read");
                    self.r9_itcm_size
                }
                _ => unreachable!()
            },

            _ => panic!("Unimplemented CP15 read from coproc reg {}", cpreg1)
        };

        info!("Read from CP15 reg {}; reg2={}, op2={}", cpreg1, cpreg2, op2);
        res
    }
}