use std::io::Write;
use std::{thread::sleep, time::Duration};

use crossterm::execute;
use crossterm::{
    event::{KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};

fn main() {
    enable_raw_mode().unwrap();
    loop {
        let res = crossterm::event::poll(Duration::from_secs(0)).unwrap();
        // sleep(Duration::from_millis(100));
        if res {
            match crossterm::event::read().unwrap() {
                crossterm::event::Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                }) => {
                    break;
                }
                crossterm::event::Event::Key(KeyEvent { code, .. }) => {
                    write!(std::io::stdout(), "{:?}\n", code).unwrap();
                    execute!(
                        std::io::stdout(),
                        crossterm::cursor::MoveToColumn(0),
                        crossterm::cursor::MoveToNextLine(1)
                    )
                    .unwrap();
                }
                _ => {}
            }
        }
    }
    disable_raw_mode().unwrap();
}
