enum IrqType {
    Dmac1_0 = (1 << 0),
    Dmac1_1 = (1 << 1),
    Dmac1_2 = (1 << 2),
    Dmac1_3 = (1 << 3),
    Dmac1_4 = (1 << 4),
    Dmac1_5 = (1 << 5),
    Dmac1_6 = (1 << 6),
    Dmac1_7 = (1 << 7),
    Timer0  = (1 << 8),
    Timer1  = (1 << 9),
    Timer2  = (1 << 10),
    Timer3  = (1 << 11),
    PxiSync     = (1 << 12),
    PxiNotFull  = (1 << 13),
    PxiNotEmpty = (1 << 14),
    Aes        = (1 << 15),
    Sdio1      = (1 << 16),
    Sdio1Async = (1 << 17),
    Sdio3      = (1 << 18),
    Sdio3Async = (1 << 19),
    DebugRecv  = (1 << 20),
    DebugSend  = (1 << 21),
    RSA        = (1 << 22),
    CtrCard1   = (1 << 23),
    CtrCard2   = (1 << 24),
    Cgc        = (1 << 25),
    CgcDet     = (1 << 26),
    DsCard     = (1 << 27),
    Dmac2      = (1 << 28),
    Dmac2Abort = (1 << 29)
}

// struct Requests {
//     requested_irqs: Arc<Cell<u32>>
// }

// impl Requests {
//     pub fn new() -> Requests {
//         Requests { requested_irqs: Arc::new(Cell::new(0)) }
//     }

//     pub fn add(&mut self, t: IrqType) {
//         self.requested_irqs.set(self.requested_irqs.get() | t as u32)
//     }

//     pub fn clr(&mut self, t: IrqType) {
//         self.requested_irqs.set(self.requested_irqs.get() & !(t as u32))
//     }
// }

iodevice!(IrqDevice, {
    regs:
    0x000 => enabled: u32 {
        write_bits = 0b00111111_11111111_11111111_11111111;
    }
    0x004 => pending: u32 {
        write_bits = 0b00111111_11111111_11111111_11111111;
    }
});