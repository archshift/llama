use std::collections::HashMap;
use std::fmt;


fn mcu_power_write(dat: &mut u8) {
    if *dat & 0b111 != 0 {
        panic!("Powering off!");
    }
}




struct PeriphMCU {
    regs: HashMap<u8, McuReg>
}

impl PeriphMCU {
    fn new() -> Self {
        let mut regs: HashMap<u8, McuReg> = HashMap::new();

        regs.insert(0x00, McuReg::new(0x00, NOP, NOP)); // Stubbed: Version high
        regs.insert(0x01, McuReg::new(0x00, NOP, NOP)); // Stubbed: Version low
        regs.insert(0x09, McuReg::new(0x00, NOP, NOP)); // Stubbed: Volume slider
        regs.insert(0x0B, McuReg::new(0xFF, NOP, NOP)); // Stubbed: Battery charge
        regs.insert(0x0F, McuReg::new(0x00, NOP, NOP)); // Stubbed: Port statuses
        regs.insert(0x10, McuReg::new(0x00, NOP, NOP)); // Stubbed: IRQs b0
        regs.insert(0x11, McuReg::new(0x00, NOP, NOP)); // Stubbed: IRQs b1
        regs.insert(0x12, McuReg::new(0x00, NOP, NOP)); // Stubbed: IRQs b2
        regs.insert(0x13, McuReg::new(0x00, NOP, NOP)); // Stubbed: IRQs b3
        regs.insert(0x18, McuReg::new(0x00, NOP, NOP)); // Stubbed: IRQ mask b0
        regs.insert(0x19, McuReg::new(0x00, NOP, NOP)); // Stubbed: IRQ mask b1
        regs.insert(0x1A, McuReg::new(0x00, NOP, NOP)); // Stubbed: IRQ mask b2
        regs.insert(0x1B, McuReg::new(0x00, NOP, NOP)); // Stubbed: IRQ mask b3
        regs.insert(0x20, McuReg::new(0x00, NOP, mcu_power_write));
        regs.insert(0x22, McuReg::new(0x00, NOP, NOP)); // Stubbed: LCD backlights
        regs.insert(0x30, McuReg::new(0x00, NOP, NOP)); // Stubbed: Realtime clock seconds
        regs.insert(0x31, McuReg::new(0x00, NOP, NOP)); // Stubbed: Realtime clock minutes
        regs.insert(0x32, McuReg::new(0x00, NOP, NOP)); // Stubbed: Realtime clock hours
        regs.insert(0x33, McuReg::new(0x00, NOP, NOP)); // Stubbed: Realtime clock week
        regs.insert(0x34, McuReg::new(0x01, NOP, NOP)); // Stubbed: Realtime clock days
        regs.insert(0x35, McuReg::new(0x01, NOP, NOP)); // Stubbed: Realtime clock months
        regs.insert(0x36, McuReg::new(0x20, NOP, NOP)); // Stubbed: Realtime clock years
        regs.insert(0x37, McuReg::new(0x00, NOP, NOP)); // Stubbed: Realtime clock leapyear counter

        Self {
            regs
        }
    }
}

impl Peripheral for PeriphMCU {
    fn read(&mut self, register: u8) -> Option<u8> {
        if let Some(reg) = self.regs.get_mut(&register) {
            (reg.read_effect)(&mut reg.dat);
            return Some(reg.dat);
        }
        panic!("Unimplemented MCU read at register {:02X}", register);
        //None
    }
    fn write(&mut self, register: u8, dat: u8) -> bool {
        if let Some(reg) = self.regs.get_mut(&register) {
            reg.dat = dat;
            (reg.write_effect)(&mut reg.dat);
            return true;
        }
        panic!("Unimplemented MCU write at register {:02X}", register);
        //false
    }
}


struct PeriphLCD;
impl PeriphLCD {
    fn new() -> Self { Self }
}

impl Peripheral for PeriphLCD {
    fn read(&mut self, register: u8) -> Option<u8> {
        warn!("STUBBED: I2C LCD read at register {:02X}", register);
        Some(0)
    }
    fn write(&mut self, register: u8, dat: u8) -> bool {
        warn!("STUBBED: I2C LCD write {:02X} at register {:02X}", dat, register);
        true
    }
}


struct McuReg {
    dat: u8, 
    read_effect: fn(&mut u8),
    write_effect: fn(&mut u8)
}

const NOP: fn(&mut u8) = |_: &mut u8| {};

impl McuReg {
    fn new(dat: u8, read_effect: fn(&mut u8), write_effect: fn(&mut u8)) -> Self {
        Self {
            dat,
            read_effect,
            write_effect
        }
    }
}



trait Peripheral {
    fn read(&mut self, register: u8) -> Option<u8>;
    fn write(&mut self, register: u8, dat: u8) -> bool;
}

pub struct I2cPeripherals {
    periphs: HashMap<u8, Box<dyn Peripheral>>
}

impl fmt::Debug for I2cPeripherals {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "I2cPeripherals {{ }}")
    }
}

impl I2cPeripherals {
    fn read(&mut self, device: u8, register: u8) -> Option<u8> {
        if let Some(p) = self.periphs.get_mut(&device) {
            p.read(register)
        } else {
            panic!("Unimplemented I2C read for peripheral at 0x{:02X}", device);
        }
    }
    
    fn write(&mut self, device: u8, register: u8, dat: u8) -> bool {
        if let Some(p) = self.periphs.get_mut(&device) {
            p.write(register, dat)
        } else {
            panic!("Unimplemented I2C write for peripheral at 0x{:02X}", device);
        }
    }
}


pub fn make_peripherals() -> I2cPeripherals {
    let mut periphs: HashMap<u8, Box<dyn Peripheral>> = HashMap::new();

    periphs.insert(0x2c, Box::new(PeriphLCD::new()));
    periphs.insert(0x2e, Box::new(PeriphLCD::new()));
    periphs.insert(0x4a, Box::new(PeriphMCU::new()));

    I2cPeripherals {
        periphs
    }
}




bf!(RegCnt[u8] {
    stop: 0:0,
    start: 1:1,
    pause: 2:2,
    ack: 4:4,
    is_read: 5:5,
    enable_interrupt: 6:6,
    busy: 7:7
});

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum I2cByteExpected {
    DeviceSelect,
    RegisterSelect,
    DataWrite,
    DataRead
}

impl Default for I2cByteExpected {
    fn default() -> Self {
        I2cByteExpected::DeviceSelect
    }
}

#[derive(Debug)]
pub struct I2cDeviceState {
    device: u8,
    register: u8,
    next_input: I2cByteExpected,

    periphs: I2cPeripherals
}

impl I2cDeviceState {
    pub fn new(periphs: I2cPeripherals) -> Self {
        Self {
            device: 0,
            register: 0,
            next_input: I2cByteExpected::DeviceSelect,

            periphs
        }
    }
}


fn advance_state_machine(dev: &mut I2cDevice) -> bool {
    let byte = dev.data.get();
    let state = &mut dev._internal_state;
    let next_input = match state.next_input {
        I2cByteExpected::DeviceSelect => {
            state.device = byte & 0xFE;

            trace!("Selected I2C device 0x{:02X}", byte);
            if byte & 1 == 0 {
                I2cByteExpected::RegisterSelect
            } else {
                I2cByteExpected::DataRead
            }
        }
        I2cByteExpected::RegisterSelect => {
            state.register = byte;
            trace!("Selected I2C register 0x{:02X}", byte);
            I2cByteExpected::DataWrite
        }
        I2cByteExpected::DataWrite => {
            let ok = state.periphs.write(state.device, state.register, byte);
            trace!("Wrote I2C data to dev 0x{:02X} reg 0x{:02X}: 0x{:02X}", state.device, state.register, byte);

            if !ok {
                return false;
            }
            state.register += 1;
            I2cByteExpected::DataWrite
        }
        I2cByteExpected::DataRead => {
            let byte = state.periphs.read(state.device, state.register);
            trace!("Read I2C data from dev 0x{:02X} reg 0x{:02X}", state.device, state.register);
            if let Some(byte) = byte {
                dev.data.set(byte);
            } else {
                return false;
            }
            state.register += 1;
            I2cByteExpected::DataRead
        }
    };
    state.next_input = next_input;
    return true;
}

iodevice!(I2cDevice, {
    internal_state: I2cDeviceState;
    regs: {
        0x000 => data: u8 {
            write_effect = |dev: &I2cDevice| {
                trace!("I2C write to DATA: 0x{:02X}", dev.data.get());
            };
        }
        0x001 => cnt: u8 {
            read_effect = |_| {}; 
            write_effect = |dev: &mut I2cDevice| {
                let mut cnt = RegCnt::new(dev.cnt.get());
                trace!("I2C write to CNT: 0x{:02X}, {:?}", dev.cnt.get(), cnt);
                if cnt.start.get() == 1 {
                    dev._internal_state.next_input = I2cByteExpected::DeviceSelect;
                }
                if cnt.busy.get() == 1 {
                    if cnt.is_read.get() == 1 {
                        assert!(dev._internal_state.next_input == I2cByteExpected::DataRead);
                    }
                    let res = advance_state_machine(dev);
                    cnt.ack.set(res as u8);
                    cnt.busy.set(0);
                }
                dev.cnt.set_unchecked(cnt.val);
            };
        }
        0x002 => cntex: u16 {
            write_effect = |_| warn!("STUBBED: Write to I2C CNTEX register");
        }
        0x004 => scl: u16 {
            write_effect = |_| warn!("STUBBED: Write to I2C SCL register");
        }
    }
});
