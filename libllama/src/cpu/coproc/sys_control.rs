use cpu::coproc::{CpEffect, Coprocessor};
use cpu::caches::{Ops, MemMgr};
use cpu::{Version, v5, v6};

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
    r1_auxctrl: u32,
    r2_dcacheability: u32,
    r2_icacheability: u32,
    r3_bufferability: u32,
    r3_domain_access: u32,
    r5_daccessperms: u32,
    r5_iaccessperms: u32,
    r6_memregions: [MpuRegion::Bf; 8],
    r9_dcache_lockdown: u32,
    r9_icache_lockdown: u32,
    r9_dtcm_size: u32,
    r9_itcm_size: u32,
    r15_perfmon_ctrl: u32,
}

fn mknop<V: Version>() -> CpEffect<V> {
    Box::new(|_| {})
}

impl SysControl {
    pub fn new() -> SysControl {
        SysControl {
            r1_control: RegControl::new(0),
            r1_auxctrl: 0,
            r2_dcacheability: 0,
            r2_icacheability: 0,
            r3_bufferability: 0,
            r3_domain_access: 0,
            r5_daccessperms: 0,
            r5_iaccessperms: 0,
            r6_memregions: [MpuRegion::new(0); 8],
            r9_dcache_lockdown: 0,
            r9_icache_lockdown: 0,
            r9_dtcm_size: 0,
            r9_itcm_size: 0,
            r15_perfmon_ctrl: 0,
        }
    }

    fn write_c1<V: Version>(&mut self, op2: usize, val: u32) -> CpEffect<V> {
        match op2 {
            0b000 => {
                warn!("STUBBED: System control register write");
                self.r1_control.val = val;

                let control = self.r1_control;
                Box::new(move |cpu| {
                    cpu.mpu.set_enabled(control.use_mpu.get() == 1);
                    cpu.mpu.icache_set_enabled(control.use_icache.get() == 1);
                    cpu.mpu.dcache_set_enabled(control.use_dcache.get() == 1);
                    if let MemMgr::Mmu(ref mut mmu) = cpu.mpu {
                        mmu.backcompat_walk = control.disable_subpage_ap.get() == 0;
                    }
                })
            }
            0b001 => {
                warn!("STUBBED: Auxiliary control register write");
                self.r1_auxctrl = val;
                mknop()
            }
            0b010 => {
                warn!("STUBBED: Coproc control register write");
                mknop()
            },
            _ => unreachable!()
        }
    }

    fn write_c2_arm9<V: Version>(&mut self, op2: usize, val: u32) -> CpEffect<V> {
        match op2 {
            0 => {
                trace!("DCache cacheability register write");
                self.r2_dcacheability = val;
                let cacheable = val;
                Box::new(move |cpu| {
                    if let MemMgr::Mpu(ref mut mpu) = cpu.mpu {
                        mpu.region_use_dcache = cacheable as u8
                    }
                })
            }
            1 => {
                trace!("ICache cacheability register write");
                self.r2_icacheability = val;
                let cacheable = val;
                Box::new(move |cpu| {
                    if let MemMgr::Mpu(ref mut mpu) = cpu.mpu {
                        mpu.region_use_icache = cacheable as u8
                    }
                })
            }
            _ => unreachable!()
        }
    }

    fn write_c2_arm11<V: Version>(&mut self, op2: usize, val: u32) -> CpEffect<V> {
        match op2 {
            0 => {
                trace!("Trans. table base 0 register write: {:08X}", val);
                Box::new(move |cpu| {
                    if let MemMgr::Mmu(ref mut mmu) = cpu.mpu {
                        mmu.page_tables[0] = val & !0b11111;
                    }
                })
            }
            1 => {
                trace!("Trans. table base 1 register write: {:08X}", val);
                Box::new(move |cpu| {
                    if let MemMgr::Mmu(ref mut mmu) = cpu.mpu {
                        mmu.page_tables[1] = val & !0b11111;
                    }
                })
            }
            2 => {
                trace!("Trans. table base ctrl. register write: {:08X}", val);
                Box::new(move |cpu| {
                    if let MemMgr::Mmu(ref mut mmu) = cpu.mpu {
                        mmu.pagesel = val as usize;
                    }
                })
            }
            _ => unreachable!()
        }
    }

    fn write_c3_arm9(&mut self, val: u32) {
        warn!("STUBBED: Data bufferability register write");
        self.r3_bufferability = val;
    }

    fn write_c3_arm11(&mut self, val: u32) {
        warn!("STUBBED: Domain access control register write");
        self.r3_domain_access = val;
    }

    fn write_c5_arm9(&mut self, op2: usize, val: u32) {
        match op2 {
            0 | 2 => { // TODO: verify
                warn!("STUBBED: Data access perms register write");
                self.r5_daccessperms = val;
            }
            1 | 3 => { // TODO: verify
                warn!("STUBBED: Instr access perms register write");
                self.r5_iaccessperms = val;
            }
            _ => unreachable!()
        }
    }

    fn write_c6_arm9<V: Version>(&mut self, cpreg2: usize, val: u32) -> CpEffect<V> {
        trace!("MPU region {} register write", cpreg2);
        let index = cpreg2;
        self.r6_memregions[index].val = val;

        let region_data = self.r6_memregions[index];
        Box::new(move |cpu| {
            if let MemMgr::Mpu(ref mut mpu) = cpu.mpu {
                let size_exp = region_data.size.get() + 1;
                mpu.region_size_exp[index] = size_exp;
                mpu.region_base_sigbits[index] = region_data.base_shr_12.get() << 12 >> size_exp;
                mpu.region_enabled |= (region_data.enabled.get() << index) as u8;
            }
        })
    }

    fn write_c7<V: Version>(&mut self, op2: usize, cpreg2: usize) -> CpEffect<V> {
        match (cpreg2, op2) {
            (5, 0..=2) => Box::new(move |cpu| cpu.mpu.icache_invalidate()),
            (6, 0..=2) => Box::new(move |cpu| cpu.mpu.dcache_invalidate()),
            (7, 0) => Box::new(move |cpu| {
                cpu.mpu.icache_invalidate();
                cpu.mpu.dcache_invalidate();
            }),
            (7, 1..=2) => unimplemented!(),
            (10, 0..=2) => Box::new(move |cpu| cpu.mpu.dcache_invalidate()),
            (11, 0..=2) => unimplemented!(),
            (14, 0..=2) => Box::new(move |cpu| cpu.mpu.dcache_invalidate()),
            (15, 0..=2) => unimplemented!(),
            _ => { warn!("STUBBED: Cache control register write; reg2={}, op2={}", cpreg2, op2); mknop() },
        }
    }

    fn write_c9_arm9(&mut self, op2: usize, cpreg2: usize, val: u32) {
        match (cpreg2, op2) {
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
        }
    }

    fn write_c15_arm9(&mut self, op1: usize, op2: usize, cpreg2: usize, val: u32) {
        match op1 {
            3 => warn!("STUBBED: Cache debug CP15 write! reg2={}, op2={}, val={:08X}", cpreg2, op2, val),
            _ => unimplemented!(),
        }
    }



    fn read_c0_arm9(&self, op2: usize) -> u32 {
        match op2 {
            1 => 0x0F0D2112, // On the 3DS: 4k, 4-way dcache; 8k, 4-day icache
            _ => unimplemented!(),
        }
    }

    fn read_c0_arm11(&self, op2: usize) -> u32 {
        match op2 {
            0 => 0x41060361, // TODO: Educated guess, hopefully works?
            5 => 0x00000000, // On the 3DS: 2xMPCore
            n => panic!("Unimplemented access to CP15 c0 with op2={}", n),
        }
    }

    fn read_c1(&self, op2: usize) -> u32 {
        match op2 {
            0b000 => {
                warn!("STUBBED: System control register read");
                self.r1_control.val
            }
            0b001 => {
                warn!("STUBBED: Auxiliary control register read");
                self.r1_auxctrl
            }
            0b010 => unimplemented!(),
            _ => unreachable!()
        }
    }

    fn read_c2_arm9(&self, op2: usize) -> u32 {
        match op2 {
            0 => {
                warn!("STUBBED: DCache cacheability register read");
                self.r2_dcacheability
            }
            1 => {
                warn!("STUBBED: ICache cacheability register read");
                self.r2_icacheability
            }
            _ => unreachable!()
        }
    }

    fn read_c3_arm9(&self) -> u32 {
        warn!("STUBBED: Data bufferability register read");
        self.r3_bufferability
    }

    fn read_c5_arm9(&self, op2: usize) -> u32 {
        match op2 {
            0 | 2 => { // TODO: verify
                warn!("STUBBED: Instr access perms register read");
                self.r5_daccessperms
            }
            1 | 3 => { // TODO: verify
                warn!("STUBBED: Data access perms register read");
                self.r5_iaccessperms
            }
            _ => unreachable!()
        }
    }

    fn read_c6_arm9(&self, cpreg2: usize) -> u32 {
        warn!("STUBBED: MPU region {} register read", cpreg2);
        self.r6_memregions[cpreg2].val
    }

    fn read_c7_arm9(&self) -> u32 {
        panic!("Cannot read from cache control register!")
    }

    fn write_c8_arm11(&mut self, op2: usize, cpreg2: usize, val: u32) {
        warn!("STUBBED: ARM11 TLB Control write! reg2={}, op2={}, val={:08X}", cpreg2, op2, val);
    }

    fn read_c9_arm9(&self, op2: usize, cpreg2: usize) -> u32 {
        match (cpreg2, op2) {
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
        }
    }

    fn write_c15_arm11(&mut self, op2: usize, cpreg2: usize, val: u32) {
        match (cpreg2, op2) {
            (12, 0) => {
                warn!("STUBBED: ARM11 Performance Monitor Control CP15 write! reg2={}, op2={}, val={:08X}", cpreg2, op2, val);
                self.r15_perfmon_ctrl = val;
            }
            _ => unimplemented!()
        }
    }

    fn read_c15_arm11(&self, op2: usize, cpreg2: usize) -> u32 {
        match (cpreg2, op2) {
            (12, 0) => {
                warn!("STUBBED: ARM11 Performance Monitor Control read");
                self.r15_perfmon_ctrl
            }
            (12, 1) => {
                warn!("STUBBED: ARM11 Performance Monitor Cycle counter read");
                0
            }
            (12, 2) => {
                warn!("STUBBED: ARM11 Performance Monitor Misc. counter 0 read");
                0
            }
            (12, 3) => {
                warn!("STUBBED: ARM11 Performance Monitor Misc. counter 0 read");
                0
            }
            _ => panic!("Unimplemented ARM11 C15 read cpreg2={} op2={}", cpreg2, op2)
        }
    }
}




impl<V: Version> Coprocessor<V> for SysControl {
    fn move_in(&mut self, cpreg1: usize, cpreg2: usize, op1: usize, op2: usize, val: u32) -> CpEffect<V> {
        let mut effect: CpEffect<V> = Box::new(move |_cpu| {});

        if cpreg1 != 15 {
            assert_eq!(op1, 0);
        }

        if V::is::<v5>() {
            match cpreg1 {
                1 => effect = self.write_c1(op2, val),
                2 => effect = self.write_c2_arm9(op2, val),
                3 => self.write_c3_arm9(val),
                5 => self.write_c5_arm9(op2, val),
                6 => effect = self.write_c6_arm9(cpreg2, val),
                7 => effect = self.write_c7(op2, cpreg2),
                9 => self.write_c9_arm9(op2, cpreg2, val),
                15 => self.write_c15_arm9(op1, op2, cpreg2, val),
                _ => panic!("Unimplemented CP15 write to coproc reg {}", cpreg1)
            };
        } else if V::is::<v6>() {
            match cpreg1 {
                1 => effect = self.write_c1(op2, val),
                3 => self.write_c3_arm11(val),
                2 => effect = self.write_c2_arm11(op2, val),
                7 => effect = self.write_c7(op2, cpreg2),
                8 => self.write_c8_arm11(op2, cpreg2, val),
                15 => self.write_c15_arm11(op2, cpreg2, val),
                _ => panic!("Unimplemented CP15 write to coproc reg {}", cpreg1)
            }
        } else {
            unreachable!()
        };

        info!("Write 0x{:08X} to CP15 reg {}; reg2={}, op2={}", val, cpreg1, cpreg2, op2);
        effect
    }

    fn move_out(&mut self, cpreg1: usize, cpreg2: usize, op1: usize, op2: usize) -> u32 {
        assert_eq!(op1, 0);

        let res = if V::is::<v5>() {
            match cpreg1 {
                0 => self.read_c0_arm9(op2),
                1 => self.read_c1(op2),
                2 => self.read_c2_arm9(op2),
                3 => self.read_c3_arm9(),
                5 => self.read_c5_arm9(op2),
                6 => self.read_c6_arm9(cpreg2),
                7 => self.read_c7_arm9(),
                9 => self.read_c9_arm9(op2, cpreg2),
                _ => panic!("Unimplemented CP15 read from coproc reg {}", cpreg1)
            }
        } else if V::is::<v6>() {
            match cpreg1 {
                0 => self.read_c0_arm11(op2),
                1 => self.read_c1(op2),
                15 => self.read_c15_arm11(op2, cpreg2),
                _ => unimplemented!()
            }
        } else {
            unreachable!()
        };

        info!("Read from CP15 reg {}; reg2={}, op2={}", cpreg1, cpreg2, op2);
        res
    }
}
