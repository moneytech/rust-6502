use std::sync::mpsc;
use std::time;
use std::thread;


pub enum ControllerMessage {
    ButtonPressed(String),
    UpdatedProcessorAvailable(Processor),
    UpdatedDataAvailable(Vec<u8>),
}

#[derive(Clone, Debug)]
pub struct Processor {
    pub flags: u8,
    pub acc: u8,
    pub rx: u8,
    pub ry: u8,
    pub pc: u16,
    pub sp: u8,
    pub info: String,
    pub clock: u64,
}

#[derive(Clone, Debug)]
pub struct Computer {
    processor: Processor,
    data: Vec<u8>,
    tx: mpsc::Sender<ControllerMessage>,
}

impl Computer {
    pub fn new(tx: mpsc::Sender<ControllerMessage>, data: Vec<u8>) -> Computer {
        let mut computer = Computer {
            data,
            tx,
            processor: Processor {
                flags: 0,
                acc: 0,
                rx: 0,
                ry: 0,
                /// Start at 0x400
                pc: 0x400,
                sp: 0,
                info: "".to_string(),
                clock: 0,
            }
        };
        computer
    }

    pub fn step(&mut self) -> bool {
        let local_tx = self.tx.clone();
        
        self.run_instruction();

        //println!("{:?}", self.processor);

        let local_proc = self.processor.clone();
        let local_data = self.data.clone();
        local_tx.send(
            ControllerMessage::UpdatedProcessorAvailable(local_proc.clone())
        );
        //only send a slice of the data
        let btm :u16 = if local_proc.pc > 256 { (local_proc.pc - 255) }else {0};
        let top :u16 = if (local_proc.pc < 0xffff - 256) { local_proc.pc + 256} else { 0xffff };
        let mem_to_display = local_data[btm as usize ..=top as usize].to_vec();

        local_tx.send(
            ControllerMessage::UpdatedDataAvailable(mem_to_display)
        );

        true
    }

    fn run_instruction(&mut self) {
        let inst = self.data[(self.processor.pc) as usize];

        match inst {
            0x10 => {
                //// println!("Running instruction bpl : {:x?}", inst);
                self.bpl();
            },
            0x18 => {
                //// println!("Running instruction clc : {:x?}", inst);
                self.clc();
            },
            0x49 => {
                //// println!("Running instruction eor : {:x?}", inst);
                self.eor();
            },
            0x4c => {
                //// println!("Running instruction jmp : {:x?}", inst);
                self.jmp();
            },
            0x69 => {
                //// println!("Running instruction adc : {:x?}", inst);
                self.adc();
            },
            0x88 => {
                //// println!("Running instruction dey : {:x?}", inst);
                self.dey();
            },
            0x8d => {
                //// println!("Running instruction sta : {:x?}", inst);
                self.sta();
            },

            0x98 => {
                //// println!("Running instruction tya : {:x?}", inst);
                self.tya();
            },
            0x9a => {
                //// println!("Running instruction txs : {:x?}", inst);
                self.txs();
            },
            0xa0 => {
                //// println!("Running instruction ldy : {:x?}", inst);
                self.ldy();
            },
            0xa2 => {
                //// println!("Running instruction ldx : {:x?}", inst);
                self.ldx();
            },
            0xaa => {
                //// println!("Running instruction tax : {:x?}", inst);
                self.tax();
            },
            0xa9 | 0xad => {
                //// println!("Running instruction lda : {:x?}", inst);
                self.lda();
            },
            0xc9 => {
                //// println!("Running instruction cmp : {:x?}", inst);
                self.cmp();
            },
            0xca => {
                //// println!("Running instruction dex : {:x?}", inst);
                self.dex();
            },
            0xd0 => {
                //// println!("Running instruction bne : {:x?}", inst);
                self.bne();
            },
            0xd8 => {
                //// println!("Running instruction cld : {:x?}", inst);
                self.cld();
            },
            0xf0 => {
                //// println!("Running instruction beq : {:x?}", inst);
                self.beq();
            },
            _ => {
                //// println!("Running instruction nop : {:x?}", inst);
                self.nop();
            },
        };
    }


    fn adc(&mut self) {
        let mut acc = self.processor.acc;
        let val = self.data[(self.processor.pc + 1) as usize];
        acc += val;
        self.processor.flags = Self::set_flags(self.processor.flags.clone(), acc);
        self.processor.info = format!("Running instruction adc: {:#x}", self.data[(self.processor.pc) as usize]);
        self.processor.acc = acc;
        self.processor.clock += 2;
        self.processor.pc += 2;
    }

    fn cld(&mut self) {
        self.processor.info = format!("Running instruction cld: {:#x}", self.data[(self.processor.pc) as usize]);
        self.processor.pc += 1;
        self.processor.flags = self.processor.flags & 0x7;
        self.processor.clock += 2;
    }

    fn txs(&mut self) {
        
        self.processor.pc += 1;
        self.processor.clock += 2;
        self.processor.sp = self.processor.rx;
        self.processor.flags = Self::set_flags( self.processor.flags, self.processor.sp);
        self.processor.info = format!("Running instruction txs: {:#x}", self.data[(self.processor.pc) as usize]);
    }

    fn tya(&mut self) {
        self.processor.pc += 1;
        self.processor.clock += 2;
        self.processor.acc = self.processor.ry;
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.acc);
        self.processor.info = format!("Running instruction tya: {:#x}", self.data[(self.processor.pc) as usize]);
    }

    fn clc(&mut self) {
        self.processor.flags =  self.processor.flags & 0xFE;
        self.processor.info = format!("Running instruction clc: {:#x}", self.data[(self.processor.pc) as usize]);
        self.processor.pc += 1;
        self.processor.clock += 1;
    }

    fn tax(&mut self) {
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.acc);
        self.processor.pc += 1;
        self.processor.clock += 1;
        self.processor.rx = self.processor.acc;
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.rx);
        self.processor.info = format!("Running instruction tax: {:#x}", self.data[(self.processor.pc) as usize]);
    }

    fn eor(&mut self) {
        let val = self.data[(self.processor.pc + 1) as usize];
        let mut acc = self.processor.acc;
        //// println!("EOR {:x?} {:x?}", val, acc);
        self.processor.info = format!("Running instruction eor: {:#x}", self.data[(self.processor.pc) as usize]);
        acc ^= val;
        self.processor.pc += 2;
        self.processor.acc = acc;
    }

    fn ldx(&mut self) {
        let x = self.data[(self.processor.pc + 1) as usize];
        self.processor.rx = x;
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.rx);
        self.processor.pc += 2;
        self.processor.info = format!("Running instruction ldx: {:#x}", self.data[(self.processor.pc) as usize]);
    }

    fn ldy(&mut self) {
        let y = self.data[(self.processor.pc + 1) as usize];
        self.processor.ry = y;
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.ry);
        self.processor.pc += 2;
        self.processor.clock += 4;
        self.processor.info = format!("Running instruction ldy: {:#x}", self.data[(self.processor.pc) as usize]);
    }

    fn lda(&mut self) {
        let mut acc = self.data[(self.processor.pc + 1) as usize];
        let inst = self.data[(self.processor.pc) as usize];
        let mut pc = self.processor.pc + 2;
        let mut info = format!("Running instruction lda: {:#x}", inst);
        self.processor.clock += 2;
        if inst == 0xad {
            //Absolute adressing

            let addr = Self::get_word(&self.data, self.processor.pc + 1);
            //// println!("inst is absolute addr {:x?}", addr);
            acc = self.data[addr as usize];
            pc = self.processor.pc + 3;
            self.processor.clock += 2;
            info = format!("Running instruction lda absolute: {:#x}", inst);
        }
        self.processor.pc = pc;
        self.processor.flags = Self::set_flags(self.processor.flags, acc);
    }

    fn dex(&mut self) {
        self.processor.rx = self.processor.rx.wrapping_sub(1);
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.rx);
        self.processor.info = format!("Running instruction dex: {:#x}", self.data[(self.processor.pc) as usize]);
        self.processor.pc += 1;
        self.processor.clock += 2;
    }

    fn dey(&mut self) {
        self.processor.ry = self.processor.ry.wrapping_sub(1);
        self.processor.flags = Self::set_flags(self.processor.flags,  self.processor.ry);
        self.processor.info = format!("Running instruction dey: {:#x}", self.data[(self.processor.pc) as usize]);
        self.processor.pc += 1;
        self.processor.clock += 2;
    }

    fn cmp(&mut self) {
        let acc = self.processor.acc;
        let value = self.data[(self.processor.pc + 1) as usize];
        let result: u8 = acc.wrapping_sub(value);
        let mut flags = self.processor.flags;
        if (acc > value) {
            flags |= 1;
        }
        flags = Self::set_flags(flags, result as u8);
        self.processor.flags = flags;
        self.processor.pc += 2;
        self.processor.clock += 4;
        self.processor.info = format!("Running instruction cmp: {:#x}", self.data[(self.processor.pc) as usize]);
    }

    fn sta(&mut self) {
        let addr = Self::get_word(&self.data, self.processor.pc + 1);
    // // println!("sta addr 0x{:x?}", addr);
        let mut _addr = self.data.to_vec().clone();
        _addr[addr as usize] = self.processor.acc;
        self.data = _addr;

        self.processor.info = format!("Running instruction sta: {:#x}", self.data[(self.processor.pc) as usize]);
        self.processor.pc += 3;
        self.processor.clock += 5;
    }

    fn jmp(&mut self) {
        let addr = Self::get_word(&self.data, self.processor.pc + 1);
        //// println!("Jumping to 0x{:x?}", addr);
        self.processor.pc = addr;
        self.processor.info = format!("Running instruction jmp: {:#x}", self.data[(self.processor.pc) as usize]);
    }

    fn bne(&mut self) {
        let offset = self.data[(self.processor.pc + 1) as usize];

        let should_jump = (self.processor.flags >> 1) & 1 == 0;
        let mut new_addr :u16;
        new_addr = self.processor.pc + 2;
        let mut info = format!("Running instruction bne not jumping: {:#x}", self.data[(self.processor.pc) as usize]);

        if (should_jump) {
            let rel_address = offset as i8;
            // // println!("Jumping offset {:?}", rel_address);
            new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
            info = format!("Running instruction bne {:#x} jumping to: {:#x}", self.data[(self.processor.pc) as usize], new_addr);
        }

        self.processor.clock += 3;
        self.processor.pc = new_addr;
    }


    fn beq(&mut self) {
        let offset = self.data[(self.processor.pc + 1) as usize];
        // // println!("Jumping RAW offset is {:?} or 0x{:x?}", offset, offset);
        let should_jump = (self.processor.flags >> 1) & 1 == 1;
        let mut new_addr :u16;
        let mut info = format!("Running instruction beq not jumping: {:#x}", self.data[(self.processor.pc) as usize]);
        new_addr = self.processor.pc + 2;
        if (should_jump) {
            let rel_address = offset as i8;
            // // println!("Jumping offset {:?}", rel_address);
            new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
            info = format!("Running instruction beq {:#x} jumping to: {:#x}", self.data[(self.processor.pc) as usize], new_addr);
        }
        self.processor.clock += 3;
        self.processor.pc = new_addr;
    }

    fn bpl(&mut self) {
        let offset = self.data[(self.processor.pc + 1) as usize];
        // println!("Jumping RAW offset is {:?} or 0x{:x?}", offset, offset);
        let should_jump = (self.processor.flags >> 7) & 1 == 0;
        let mut new_addr :u16;
        new_addr = self.processor.pc + 2;
        let mut info = format!("Running instruction bpl not jumping: {:#x}", self.data[(self.processor.pc) as usize]);
        if (should_jump) {
            let rel_address = offset as i8;
            // println!("BPL Jumping offset {:?}", rel_address);
            new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
            info = format!("Running instruction bpl {:#x} jumping to: {:#x}", self.data[(self.processor.pc) as usize], new_addr);
        }
        self.processor.pc = new_addr;
        self.processor.clock += 3;
    }

    fn nop(&mut self) {
        self.processor.pc += 1;
        self.processor.clock += 2;
        self.processor.info = format!("Running instruction nop: {:#x}", self.data[(self.processor.pc) as usize]);
    }

    pub fn set_flags(flags:u8, val:u8) -> u8 {
        let mut _flags = flags;
        if val == 0 {
            //Set zero flag
            _flags |= 0b10;
        } else {
            _flags &= 0b11111101;
        }
        if (val >> 7 == 1) {
            _flags |= 0b10000000;
        }
        // // println!("Setting flags to {:#b}", _flags);
        return _flags;
    }

    pub fn get_word(data: &Vec<u8>, address: u16) -> u16 {
        let low_byte :u16 = data[(address) as usize].into();
        let high_byte :u16 = data[(address + 1) as usize].into();
        return low_byte + (high_byte << 8);
    }
}