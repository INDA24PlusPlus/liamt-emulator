extern crate tty_read;

use enum_try_from::impl_enum_try_from;
use std::io::Write;
use tty_read::{ReaderOptions, TermReader};
// Very nice!
impl_enum_try_from!(
    #[repr(u16)]
    #[derive(PartialEq, Eq, Debug, Clone)]
    enum Opcode {
        ADD = 0b0001,
        AND = 0b0101,
        BR = 0b0000,
        JMP = 0b1100,
        JSR = 0b0100,
        LD = 0b0010,
        LDI = 0b1010,
        LDR = 0b0110,
        LEA = 0b1110,
        NOT = 0b1001,
        RTI = 0b1000,
        ST = 0b0011,
        STI = 0b1011,
        STR = 0b0111,
        TRAP = 0b1111,
        INVALID = 0b1101,
    },
    u16,
    (),
    ()
);

// Sign extend
fn sext(value: u16, bit: u16) -> u16 {
    if (value >> (bit - 1)) & 1 == 1 {
        value | (0xFFFF << bit)
    } else {
        value
    }
}

pub struct Emulator {
    memory: [u16; 0x10000],
    registers: [u16; 8],
    pc: u16,
    psr: u16,
}

impl Emulator {
    pub fn new(start_addr: u16, memory: [u16; 0x10000]) -> Emulator {
        Emulator {
            memory,
            registers: [0; 8],
            pc: start_addr,
            psr: 0,
        }
    }

    fn read_mem(&self, addr: usize) -> u16 {
        if addr == 0xFE02 {
            let mut c;
            {
                let options = ReaderOptions::default();
                let reader = TermReader::open_stdin(&options).expect("failed to open stdin reader");
                c = reader.read_byte().map(|byte| byte as u16).unwrap_or(0);
                if c == 0xd {
                    c = 0xa;
                }
            }
            if c == 0x3 {
                std::process::exit(0);
            }
            return c;
        }

        self.memory.get(addr).copied().unwrap_or(0)
    }

    fn set_mem(&mut self, addr: usize, value: u16) {
        if let Some(cell) = self.memory.get_mut(addr) {
            *cell = value;
        }
    }

    fn read_reg(&self, reg: u16) -> u16 {
        self.registers.get(reg as usize).copied().unwrap_or(0)
    }

    fn set_reg(&mut self, reg: u16, value: u16) {
        if let Some(cell) = self.registers.get_mut(reg as usize) {
            *cell = value;
        }
    }

    fn debug(&self, a: u16, b: u16, opcode: Opcode) {
        std::thread::sleep(std::time::Duration::from_millis(10));
        println!("\nRegisters: {:?}", self.registers);
        println!(
            "PSR: {:#x} PC: {:#x} a: {:#x} b: {:#x} Opcode: {:?}",
            self.psr, self.pc, a, b, opcode
        );
    }

    pub fn run(&mut self, verbose: bool) {
        self.memory[0xFE00] = 0b1000000000000000;

        loop {
            let instr = self.memory[self.pc as usize];

            let bits = format!("{:016b}", instr);
            let mut bits = bits
                .as_bytes()
                .iter()
                .map(|&b| b - b'0')
                .collect::<Vec<_>>();
            bits.reverse();

            let opcode = instr >> 12;
            let op = Opcode::try_from(opcode).expect("Invalid opcode!");

            let a = (instr >> 9) & 0b111;
            let b = (instr >> 6) & 0b111;

            if verbose {
                self.debug(a, b, op.clone());
            }

            self.pc = self.pc.wrapping_add(1);

            match op {
                Opcode::ADD => {
                    let value = if bits[5] == 0 {
                        let sr2 = instr & 0b111;
                        self.read_reg(b).wrapping_add(self.read_reg(sr2))
                    } else {
                        let imm5 = sext(instr & 0b11111, 5);
                        self.read_reg(b).wrapping_add(imm5)
                    };
                    self.set_reg(a, value);
                    self.setcc(value);
                }
                Opcode::AND => {
                    let value = if bits[5] == 0 {
                        let sr2 = instr & 0b111;
                        self.read_reg(b) & self.read_reg(sr2)
                    } else {
                        let imm5 = sext(instr & 0b11111, 5);
                        self.read_reg(b) & imm5
                    };
                    self.set_reg(a, value);
                    self.setcc(value);
                }
                Opcode::BR => {
                    let n = bits[11] == 1;
                    let z = bits[10] == 1;
                    let p = bits[9] == 1;

                    let nn = (self.psr >> 2) & 1 == 1;
                    let zz = (self.psr >> 1) & 1 == 1;
                    let pp = self.psr & 1 == 1;

                    let pc_offset = sext(instr & 0b111111111, 9);
                    if (n && nn) || (z && zz) || (p && pp) {
                        self.pc = self.pc.wrapping_add(pc_offset);
                    }
                }
                // RET also
                Opcode::JMP => {
                    self.pc = self.read_reg(b);
                }
                // JSRR also
                Opcode::JSR => {
                    self.set_reg(7, self.pc as u16);
                    if bits[11] == 0 {
                        self.pc = self.read_reg(b);
                    } else {
                        let pc_offset = sext(instr & 0b11111111111, 11);
                        self.pc = self.pc.wrapping_add(pc_offset);
                    }
                }
                Opcode::LD => {
                    let pc_offset = sext(instr & 0b111111111, 9);
                    let value = self.read_mem(self.pc.wrapping_add(pc_offset) as usize);
                    self.set_reg(a, value);
                    self.setcc(value);
                }
                Opcode::LDI => {
                    let pc_offset = sext(instr & 0b111111111, 9);
                    let value = self
                        .read_mem(self.read_mem(self.pc.wrapping_add(pc_offset) as usize) as usize);
                    self.set_reg(a, value);
                    self.setcc(value);
                }
                Opcode::LDR => {
                    let offset = sext(instr & 0b111111, 6);
                    let value = self.read_mem(self.read_reg(b).wrapping_add(offset) as usize);
                    self.set_reg(a, value);
                    self.setcc(value);
                }
                Opcode::LEA => {
                    let pc_offset = sext(instr & 0b111111111, 9);
                    let value = self.pc.wrapping_add(pc_offset);
                    self.set_reg(a, value);
                    self.setcc(value);
                }
                Opcode::NOT => {
                    let value = !self.read_reg(b);
                    self.set_reg(a, value);
                    self.setcc(value);
                }
                Opcode::RTI => {
                    panic!("RTI not implemented 😂");
                }
                Opcode::ST => {
                    let pc_offset = sext(instr & 0b111111111, 9);
                    self.set_mem(self.pc.wrapping_add(pc_offset) as usize, self.read_reg(a));
                }
                Opcode::STI => {
                    let pc_offset = sext(instr & 0b111111111, 9);
                    self.set_mem(
                        self.read_mem(self.pc.wrapping_add(pc_offset) as usize) as usize,
                        self.read_reg(a),
                    );
                }
                Opcode::STR => {
                    let offset = sext(instr & 0b111111, 6);
                    self.set_mem(
                        self.read_reg(b).wrapping_add(offset) as usize,
                        self.read_reg(a),
                    );
                }
                Opcode::TRAP => {
                    self.trap(instr & 0b11111111);
                }
                _ => {
                    println!("Invalid opcode: {:?}", op);
                    std::process::exit(1);
                }
            }
        }
    }

    fn trap(&mut self, vector: u16) {
        self.set_reg(7, self.pc as u16);

        match vector {
            // GETC
            0x20 => {
                let mut c;
                {
                    let options = ReaderOptions::default();
                    let reader =
                        TermReader::open_stdin(&options).expect("failed to open stdin reader");
                    c = reader.read_byte().map(|byte| byte as u16).unwrap_or(0);
                    if c == 0xd {
                        c = 0xa;
                    }
                    self.set_reg(0, c);
                    self.setcc(c);
                }
                if c == 0x3 {
                    std::process::exit(0);
                }
            }
            // OUT
            0x21 => {
                let c = self.read_reg(0) as u8 as char;
                print!("{}", c);
            }
            // PUTS
            0x22 => {
                let mut addr = self.read_reg(0) as usize;
                loop {
                    let c = self.read_mem(addr);
                    if c == 0 {
                        break;
                    }
                    print!("{}", c as u8 as char);
                    addr += 1;

                    std::io::stdout().flush().unwrap();
                }
            }
            // IN
            0x23 => {
                print!("\n> ");
                std::io::stdout().flush().unwrap();

                let mut c;
                {
                    let options = ReaderOptions::default();
                    let reader =
                        TermReader::open_stdin(&options).expect("failed to open stdin reader");
                    c = reader.read_byte().map(|byte| byte as u16).unwrap_or(0);
                    if c == 0xd {
                        c = 0xa;
                    }
                    self.set_reg(0, c);
                    self.setcc(c);
                }
                if c == 0x3 {
                    std::process::exit(0);
                }
                print!("{}", c as u8 as char);
            }
            // PUTSP
            0x24 => {
                let mut addr = self.read_reg(0) as usize;
                loop {
                    let c = self.read_mem(addr);
                    let c1 = c & 0xFF;
                    let c2 = c >> 8;

                    if c1 != 0 {
                        print!("{}", c1 as u8 as char);
                    }
                    if c2 != 0 {
                        print!("{}", c2 as u8 as char);
                    }
                    if c1 == 0 && c2 == 0 {
                        break;
                    }

                    std::io::stdout().flush().unwrap();

                    addr += 1;
                }
            }
            // HALT
            0x25 => {
                println!("\nHALT");
                std::process::exit(0);
            }
            _ => {}
        }
    }

    fn setcc(&mut self, value: u16) {
        let n = (value >> 15) == 1 && value != 0;
        let z = value == 0;
        let p = (value >> 15) == 0 && value != 0;

        let mut psr = self.psr;
        psr &= 0b1111111111111000; // Clear the condition codes
        psr |= (n as u16) << 2; // Bit 2 is N
        psr |= (z as u16) << 1; // Bit 1 is Z
        psr |= p as u16; // Bit 0 is P

        self.psr = psr;
    }
}
