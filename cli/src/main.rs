use std::{fs::File, io::Read, thread::sleep, time::Duration};

use chip8_cli::cli::CLIEvent;
use chip8_core::Chip8CPU;
use crossterm::{
    event::{KeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, Clear},
};

fn main() {
    enable_raw_mode().unwrap();
    execute!(
        std::io::stdout(),
        crossterm::cursor::Hide,
        crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
    )
    .unwrap();
    let args: Vec<String> = std::env::args().collect();
    let filename = args.get(1).expect("No filename provided");
    let cli_manager = chip8_cli::cli::CLIManager::new();
    let rx = cli_manager.watch_for_key();
    let mut cpu = chip8_core::CPU::new(&cli_manager, &cli_manager);
    let data = File::open(filename).expect(format!("Could not open file {}", filename).as_str());
    let mut data = std::io::BufReader::new(data);
    let mut buffer = vec![];
    data.read_to_end(&mut buffer).unwrap();
    cpu.load_program(&buffer.as_slice()).unwrap();
    let mut last_pressed_key = cli_manager.pressed_key.read().unwrap().clone();
    loop {
        cpu.step().unwrap();
        let did_draw = cli_manager.draw_if_needed();
        if let Ok(CLIEvent::Sigint) = rx.try_recv() {
            break;
        }
        if let Some(key) = cli_manager.pressed_key.read().unwrap().clone() {
            last_pressed_key.replace(key);
        }
        execute!(
            std::io::stdout(),
            crossterm::cursor::MoveToColumn(0),
            Clear(crossterm::terminal::ClearType::CurrentLine),
            Print(format!(
                "{:?} {:?} {:?}",
                cli_manager.pressed_key.read().unwrap(),
                last_pressed_key,
                &cpu
            ),),
        )
        .unwrap();
        // execute!(
        //     std::io::stdout(),
        //     Print(format!("{:?}", cli_manager.pressed_key.read().unwrap()))
        // )
        // .unwrap();
        sleep(Duration::from_micros(500));
    }
    execute!(std::io::stdout(), crossterm::cursor::Show,).unwrap();
    disable_raw_mode().unwrap();
}
