use std::io::Read;

use chip8_core::OpCode;

fn main() {
    let mut file = std::fs::File::open("test.ch8").unwrap();
    let mut buffer = vec![];
    file.read_to_end(&mut buffer).unwrap();
    // hexdump::hexdump(buffer.as_slice());
    buffer
        .chunks(2)
        .filter_map(|chunk| {
            // Convert chunk to tuple if it has 2 elements
            if chunk.len() == 2 {
                if let Ok(opcode) = OpCode::try_from((chunk[0], chunk[1])) {
                    Some(format!("{:?}", opcode))
                } else {
                    Some(format!("0x{:02X}{:02X}", chunk[0], chunk[1]))
                }
            } else {
                None // Ignore incomplete chunks
            }
        })
        .enumerate()
        .for_each(|(i, f)| {
            let i = i + 0x200;
            println!("0x{:04X}: {}", i, f)
        });
}
