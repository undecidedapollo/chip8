use std::{fs::File, io::Read, thread::sleep, time::Duration};

use chip8_core::{run, Screen};

fn main() {
    let screen = Screen::new();
    let mut cpu = chip8_core::CPU::new(&screen);
    let data = File::open("./roms/rando.ch8").unwrap();
    let mut data = std::io::BufReader::new(data);
    let mut buffer = vec![];
    data.read_to_end(&mut buffer).unwrap();
    cpu.load_program(&buffer.as_slice()).unwrap();
    for n in 0..100000 {
        cpu.step().unwrap();
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        print(&screen);
        sleep(Duration::from_micros(1000));
    }
    // screen.draw_sprite(0, 0, &[0xF0, 0x10, 0xF0, 0x10, 0xF0]);
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    print(&screen);
    println!("{:?}", cpu);
}

fn print(screen: &Screen) {
    for y in 0_usize..32 {
        for x in 0_usize..64 {
            let val = screen.buffer.borrow()[(y * 64 + x) / 8];
            let bit = usize::from(x) % 8;
            let mask = 1 << 7 - bit;
            let val = val & mask != 0;
            print!("{}", if val { '█' } else { ' ' });
        }
        println!();
    }
}
