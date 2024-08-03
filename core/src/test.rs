use crate::{
    opcodes::{convert_opcodes_into_u8, OpCode},
    Chip8CPU, Chip8Input, Chip8Screen, CPU,
};

pub(crate) struct NoopScreen;

impl Chip8Screen for NoopScreen {
    // Each row is a byte, with each bit representing a pixel, this is the same as the buffer
    fn draw_sprite(&self, _x: u8, _y: u8, _sprite: &[u8]) -> bool {
        return false;
    }

    fn clear(&self) {}
}

pub(crate) fn u16_to_u8(data: &[u16]) -> Vec<u8> {
    data.iter()
        .flat_map(|num| {
            let left = ((num & 0xFF00) >> 8) as u8;
            let right = (num & 0x00FF) as u8;
            [left, right]
        })
        .collect::<Vec<u8>>()
}

pub(crate) fn run_program<TScreen: Chip8Screen, TInput: Chip8Input>(
    cpu: &mut CPU<'_, TScreen, TInput>,
    data: &[u16],
) -> () {
    cpu.load_program(u16_to_u8(data).as_slice()).ok();
    for _ in 0..data.len() {
        cpu.step().ok();
    }
}

pub(crate) fn run_from_program_counter<TScreen: Chip8Screen, TInput: Chip8Input>(
    cpu: &mut CPU<'_, TScreen, TInput>,
    data: &[u16],
) -> () {
    cpu.load_at_program_counter(u16_to_u8(data).as_slice()).ok();
    for _ in 0..data.len() {
        cpu.step().ok();
    }
}

pub fn op_run_program<TScreen: Chip8Screen, TInput: Chip8Input>(
    cpu: &mut CPU<'_, TScreen, TInput>,
    data: &[OpCode],
) -> () {
    cpu.load_program(convert_opcodes_into_u8(data).as_slice())
        .ok();
    for _ in 0..data.len() {
        cpu.step().ok();
    }
}

pub(crate) fn op_run_from_program_counter<TScreen: Chip8Screen, TInput: Chip8Input>(
    cpu: &mut CPU<'_, TScreen, TInput>,
    data: &[OpCode],
) -> () {
    cpu.load_at_program_counter(convert_opcodes_into_u8(data).as_slice())
        .ok();
    for _ in 0..data.len() {
        cpu.step().ok();
    }
}

#[macro_export]
macro_rules! run {
    ($cpu:expr, $($opcode:ident { $($field:ident: $value:expr),* }),* $(,)?) => {{
        $crate::op_run_program(
            &mut $cpu,
            [
                $(
                    $crate::OpCode::$opcode { $($field: $value),* },
                )*
            ].as_slice(),
        )
    }};
}

#[macro_export]
macro_rules! run_from_pc {
    ($cpu:expr, $($opcode:ident { $($field:ident: $value:expr),* }),* $(,)?) => {{
        op_run_from_program_counter(
            &mut $cpu,
            [
                $(
                    OpCodes::$opcode { $($field: $value),* },
                )*
            ].as_slice(),
        )
    }};
}
