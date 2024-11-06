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

    let start_addr = u16::from_be_bytes([buffer[0], buffer[1]]);

    let mut memory = [0; 0x10000];
    for i in 0..(buffer.len() - 2) / 2 {
        memory[start_addr as usize + i] =
            u16::from_be_bytes([buffer[i * 2 + 2], buffer[i * 2 + 3]]);
    }

    println!("Start address: {:#x}", start_addr);

    println!(
        "Memory: {:?}",
        &memory[start_addr as usize..(start_addr + 0x20) as usize]
    );

    let mut emulator = Emulator::new(start_addr, memory);
    emulator.run();
}
