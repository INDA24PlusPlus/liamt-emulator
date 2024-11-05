use enum_try_from::impl_enum_try_from;

// Very nice!
impl_enum_try_from!(
    #[repr(u16)]
    #[derive(PartialEq, Eq, Debug)]
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

pub struct Emulator {
    memory: [u16; 0x6969],
    registers: [u16; 8],
    pc: usize,
}

impl Emulator {
    pub fn new(prog: Vec<u16>) -> Emulator {
        let mut memory = [0; 0x6969];
        for (i, instr) in prog.iter().enumerate() {
            memory[i] = *instr;
        }
        Emulator {
            memory,
            registers: [0; 8],
            pc: 0,
        }
    }

    pub fn run(&mut self) {
        loop {
            let instr = self.memory[self.pc];

            let opcode = instr >> 12;
            println!("Opcode: {:04b}", opcode);
            let op = Opcode::try_from(opcode).expect("Invalid opcode!");

            match op {
                _ => {}
            }
            self.pc += 1;
        }
    }
}
