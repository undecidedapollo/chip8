use chip8_core::{run, Screen};

fn main() {
    let screen = Screen::new();
    let mut cpu = chip8_core::CPU::new(&screen);
    run!(
        cpu,
        _6XNN { x: 0, nn: 0x55 },
        _FX1E { x: 0 },
        _6XNN { x: 0, nn: 0x2 },
        _DXYN { x: 0, y: 1, n: 5 },
        // _6XNN { x: 0, nn: 0x05 },
        // _FX1E { x: 0 },
        // _6XNN { x: 0, nn: 0x8 },
        // _DXYN { x: 0, y: 1, n: 5 },
    );
    // screen.draw_sprite(0, 0, &[0xF0, 0x10, 0xF0, 0x10, 0xF0]);
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
