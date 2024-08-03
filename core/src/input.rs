use std::{thread, time::Duration};

pub trait Chip8Input {
    fn get_key(&self) -> Option<u8>;
}

pub struct NoopInput;

impl Chip8Input for NoopInput {
    fn get_key(&self) -> Option<u8> {
        return None;
    }
}
