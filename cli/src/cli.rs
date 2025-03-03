use std::{
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};

use chip8_core::{Chip8Input, Chip8Screen, Screen};
use crossterm::{
    cursor::{MoveTo, MoveToColumn, MoveToNextLine},
    event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    style::Print,
};

pub struct CLIManager {
    pub pressed_key: Arc<RwLock<Option<u8>>>,
    screen: Screen,
}

pub enum CLIEvent {
    Sigint,
}

impl CLIManager {
    pub fn new() -> CLIManager {
        return CLIManager {
            pressed_key: Arc::new(RwLock::new(None)),
            screen: Screen::new(),
        };
    }

    pub fn watch_for_key(&self) -> std::sync::mpsc::Receiver<CLIEvent> {
        let (tx, rx) = std::sync::mpsc::channel();
        let pressed_key = self.pressed_key.clone();
        thread::spawn(move || loop {
            let hex = match crossterm::event::read().unwrap() {
                crossterm::event::Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                }) => {
                    tx.send(CLIEvent::Sigint).unwrap();
                    None
                }
                crossterm::event::Event::Key(KeyEvent { code, .. }) => match code {
                    KeyCode::Char('0'..='9')
                    | KeyCode::Char('a'..='f')
                    | KeyCode::Char('A'..='F') => u8::from_str_radix(&code.to_string(), 16).ok(),
                    _ => None,
                },
                _ => None, // Ignore other events
            };

            if let Some(key) = hex {
                pressed_key.write().unwrap().replace(key);
                thread::sleep(Duration::from_millis(50));
                pressed_key.write().unwrap().take();
            }
        });

        return rx;
    }

    pub fn draw_if_needed(&self) -> bool {
        if !self.screen.is_pending_draw() {
            return false;
        }
        execute!(std::io::stdout(), MoveTo(0, 0)).unwrap();

        self.screen.draw_as_string().split("\n").for_each(|line| {
            execute!(
                std::io::stdout(),
                Print(line),
                MoveToNextLine(1),
                MoveToColumn(0)
            )
            .unwrap();
        });
        self.screen.mark_drawn();
        return true;
    }
}

impl Chip8Input for CLIManager {
    fn get_key(&self) -> Option<u8> {
        self.pressed_key.read().unwrap().clone()
    }
}

impl Chip8Screen for CLIManager {
    fn draw_sprite(&self, x: u8, y: u8, sprite: &[u8]) -> bool {
        self.screen.draw_sprite(x, y, sprite)
    }

    fn clear(&self) {
        self.screen.clear();
    }
}
