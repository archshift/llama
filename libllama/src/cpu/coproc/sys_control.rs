use cpu::coproc::{CpEffect, Coprocessor};

bf!(RegControl[u32] {
    use_mpu: 0:0,
    align_fault: 1:1,
    use_dcache: 2:2,
    write_buf: 3:3,
    big_endian: 7:7,
    sys_protect: 8:8,
    rom_protect: 9:9,
    f_bit: 10:10,
    predict_branches: 11:11,
    use_icache: 12:12,
    high_vectors: 13:13,
    predictable_cache: 14:14,
    disable_thumb: 15:15,
    low_latency_frq: 21:21,
    allow_unaligned: 22:22,
    disable_subpage_ap: 23:23,
    vectored_interrupts: 24:24,
    mixed_endian_exceptions: 25:25,
    use_l2cache: 26:26
});

bf!(MpuRegion[u32] {
    enabled: 0:0,
    size: 1:5,
    base_shr_12: 12:31
});

pub struct SysControl {
    r1_control: RegControl::Bf,
    r2_dcacheability: u32,
    r2_icacheability: u32,
    r3_bufferability: u32,
    r5_daccessperms: u32,
    r5_iaccessperms: u32,
    r6_memregions: [MpuRegion::Bf; 8],
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
            r6_memregions: [MpuRegion::new(0); 8],
            r9_dcache_lockdown: 0,
            r9_icache_lockdown: 0,
            r9_dtcm_size: 0,
            r9_itcm_size: 0,
        }
    }
}

impl Coprocessor for SysControl {
    fn move_in(&mut self, cpreg1: usize, cpreg2: usize, op1: usize, op2: usize, val: u32) -> CpEffect {
        let mut effect: CpEffect = Box::new(move |_cpu| {});

        if cpreg1 != 15 {
            assert_eq!(op1, 0);
        }

        match cpreg1 {
            1 => match op2 {
                0b000 => {
                    warn!("STUBBED: System control register write");
                    self.r1_control.val = val;

                    let control = self.r1_control;
                    effect = Box::new(move |cpu| {
                        cpu.mpu.enabled = control.use_mpu.get() == 1;
                        cpu.mpu.icache_enabled = control.use_icache.get() == 1;
                        cpu.mpu.dcache_enabled = control.use_dcache.get() == 1;
                    });
                }
                0b001 | 0b010 => unimplemented!(),
                _ => unreachable!()
            },

            2 => match op2 {
                0 => {
                    trace!("DCache cacheability register write");
                    self.r2_dcacheability = val;
                    let cacheable = val;
                    effect = Box::new(move |cpu| {
                        cpu.mpu.region_use_dcache = cacheable as u8
                    });
                }
                1 => {
                    trace!("ICache cacheability register write");
                    self.r2_icacheability = val;
                    let cacheable = val;
                    effect = Box::new(move |cpu| {
                        cpu.mpu.region_use_icache = cacheable as u8
                    });
                }
                _ => unreachable!()
            },

            3 => {
                warn!("STUBBED: Data bufferability register write");
                self.r3_bufferability = val;
            }

            5 => match op2 {
                0 | 2 => { // TODO: verify
                    warn!("STUBBED: Data access perms register write");
                    self.r5_daccessperms = val;
                }
                1 | 3 => { // TODO: verify
                    warn!("STUBBED: Instr access perms register write");
                    self.r5_iaccessperms = val;
                }
                _ => unreachable!()
            },

            6 => {
                trace!("MPU region {} register write", cpreg2);
                let index = cpreg2;
                self.r6_memregions[index].val = val;

                let region_data = self.r6_memregions[index];
                effect = Box::new(move |cpu| {
                    let size_exp = region_data.size.get() + 1;
                    cpu.mpu.region_size_exp[index] = size_exp;
                    cpu.mpu.region_base_sigbits[index] = region_data.base_shr_12.get() << 12 >> size_exp;
                    cpu.mpu.region_enabled |= (region_data.enabled.get() << index) as u8;
                });
            }

            7 => match (cpreg2, op2) {
                (5, 0...2) => effect = Box::new(move |cpu| cpu.mpu.icache_invalidate()),
                (6, 0...2) => effect = Box::new(move |cpu| cpu.mpu.dcache_invalidate()),
                (7, 0) => effect = Box::new(move |cpu| {
                    cpu.mpu.icache_invalidate();
                    cpu.mpu.dcache_invalidate();
                }),
                (7, 1...2) => unimplemented!(),
                (10, 0...2) => effect = Box::new(move |cpu| cpu.mpu.dcache_invalidate()),
                (11, 0...2) => unimplemented!(),
                (14, 0...2) => effect = Box::new(move |cpu| cpu.mpu.dcache_invalidate()),
                (15, 0...2) => unimplemented!(),
                _ => warn!("STUBBED: Cache control register write; reg2={}, op2={}", cpreg2, op2),
            }

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

            15 => match op1 {
                3 => warn!("STUBBED: Cache debug CP15 write! reg2={}, op2={}, val={:08X}", cpreg2, op2, val),
                _ => unimplemented!(),
            }

            _ => panic!("Unimplemented CP15 write to coproc reg {}", cpreg1)
        };

        info!("Write 0x{:08X} to CP15 reg {}; reg2={}, op2={}", val, cpreg1, cpreg2, op2);
        effect
    }

    fn move_out(&mut self, cpreg1: usize, cpreg2: usize, op1: usize, op2: usize) -> u32 {
        assert_eq!(op1, 0);

        let res = match cpreg1 {
            0 => match op2 {
                1 => 0x0F0D2112, // On the 3DS: 4k, 4-way dcache; 8k, 4-day icache
                _ => unimplemented!(),
            },

            1 => match op2 {
                0b000 => {
                    warn!("STUBBED: System control register read");
                    self.r1_control.val
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
                0 | 2 => { // TODO: verify
                    warn!("STUBBED: Instr access perms register read");
                    self.r5_daccessperms
                }
                1 | 3 => { // TODO: verify
                    warn!("STUBBED: Data access perms register read");
                    self.r5_iaccessperms
                }
                _ => unreachable!()
            },

            6 => {
                warn!("STUBBED: MPU region {} register read", cpreg2);
                self.r6_memregions[cpreg2].val
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
