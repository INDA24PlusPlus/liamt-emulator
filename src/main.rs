mod emulator;
use emulator::*;

use std::fs::File;
use std::io::Read;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file>", args[0]);
        std::process::exit(1);
    }

    let file = &args[1];
    let mut f = File::open(file).expect("ERROR: Unable to open file");

    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)
        .expect("ERROR: Unable to read file");
    if buffer.len() % 2 != 0 {
        eprintln!("ERROR: Invalid file size");
        std::process::exit(1);
    }

    let mut program = Vec::new();
    for i in (0..buffer.len()).step_by(2) {
        let instr = ((buffer[i] as u16) << 8) | buffer[i + 1] as u16;
        program.push(instr);
    }

    println!("Program: {:?}", program);

    let mut emulator = Emulator::new(program);
    emulator.run();
}
