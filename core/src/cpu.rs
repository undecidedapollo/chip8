use std::{
    f32::consts::E,
    fmt::Debug,
    io::Write,
    mem, thread,
    time::{Duration, Instant},
};

use rand::Rng;

use crate::{
    opcodes::{Chip8Error, OpCode},
    Chip8Input, Chip8Screen,
};

const PGRM_LOAD_START_ADDR: u16 = 0x200;
const FONT_START_ADDR: u16 = 0x50;

trait RegistryUtils {
    fn nth(&self, n: u8) -> u8;
    fn set(&mut self, index: u8, value: u8) -> ();
}

impl RegistryUtils for [u8] {
    fn nth(&self, n: u8) -> u8 {
        return self[n as usize];
    }

    fn set(&mut self, index: u8, value: u8) -> () {
        self[index as usize] = value;
    }
}

#[rustfmt::skip]
const FONT_BUFFER : [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0,
    0x20, 0x60, 0x20, 0x20, 0x70,
    0xF0, 0x10, 0xF0, 0x80, 0xF0,
    0xF0, 0x10, 0xF0, 0x10, 0xF0,
    0x90, 0x90, 0xF0, 0x10, 0x10,
    0xF0, 0x80, 0xF0, 0x10, 0xF0,
    0xF0, 0x80, 0xF0, 0x90, 0xF0,
    0xF0, 0x10, 0x20, 0x40, 0x40,
    0xF0, 0x90, 0xF0, 0x90, 0xF0,
    0xF0, 0x90, 0xF0, 0x10, 0xF0,
    0xF0, 0x90, 0xF0, 0x90, 0x90,
    0xE0, 0x90, 0xE0, 0x90, 0xE0,
    0xF0, 0x80, 0x80, 0x80, 0xF0,
    0xE0, 0x90, 0x90, 0x90, 0xE0,
    0xF0, 0x80, 0xF0, 0x80, 0xF0,
    0xF0, 0x80, 0xF0, 0x80, 0x80,
];

pub trait Chip8CPU {
    fn step(&mut self) -> Result<(), Chip8Error>;
}

pub struct CPU<'a, TScreen, TInput>
where
    TScreen: Chip8Screen,
    TInput: Chip8Input,
{
    // memory: Box<[u8; 65536]>,
    memory: Box<[u8; 4096]>,
    v: [u8; 16],
    i: u16,
    timer: u8,
    sound: u8,
    pc: u16,
    stack_ptr: u16,
    screen: &'a TScreen,
    input: &'a TInput,
    last_decrement: Instant,
}

impl<'a, TScreen, TInput> CPU<'a, TScreen, TInput>
where
    TScreen: Chip8Screen,
    TInput: Chip8Input,
{
    pub fn new(screen: &'a TScreen, input: &'a TInput) -> Self {
        let mut cpu = CPU {
            // memory: Box::new([0; 65536]),
            memory: Box::new([0; 4096]),
            v: [0; 16],
            i: 0,
            timer: 0,
            sound: 0,
            pc: 0x200,
            stack_ptr: 0xFFF,
            screen,
            input,
            last_decrement: Instant::now(),
        };

        cpu.memory[0x50..]
            .as_mut()
            .write_all(&FONT_BUFFER)
            .expect("Failed to write font data into memory");

        return cpu;
    }

    pub fn reset(&mut self) {
        self.pc = 0x200;
        self.stack_ptr = 0xFFF;
        self.memory.fill(0);
        self.v.fill(0);
        self.i = 0;
        self.timer = 0;
        self.sound = 0;
        self.screen.clear();
    }

    pub fn load_into_memory(&mut self, start_addr: u16, data: &[u8]) -> Result<(), std::io::Error> {
        self.memory[start_addr as usize..start_addr as usize + data.len()]
            .as_mut()
            .write_all(data)
    }

    pub(crate) fn load_at_program_counter(&mut self, data: &[u8]) -> Result<(), std::io::Error> {
        self.load_into_memory(self.pc, data)
    }

    pub fn load_program(&mut self, data: &[u8]) -> Result<(), std::io::Error> {
        self.load_into_memory(PGRM_LOAD_START_ADDR, data)
    }
}

impl<TScreen, TInput> Chip8CPU for CPU<'_, TScreen, TInput>
where
    TScreen: Chip8Screen,
    TInput: Chip8Input,
{
    fn step(&mut self) -> Result<(), Chip8Error> {
        if self.last_decrement.elapsed().as_millis() >= 16 {
            self.last_decrement = Instant::now();
            if self.timer > 0 {
                self.timer -= 1;
            }

            if self.sound > 0 {
                self.sound -= 1;
            }
        }

        let op1 = self.memory[self.pc as usize];
        let op2 = self.memory[self.pc as usize + 1];
        let opcode = OpCode::try_from((op1, op2))?;
        // println!("PC: {:04X} INSTRUCTION: {:?}", self.pc, opcode);

        let res: Result<bool, _> = match opcode {
            // Execute machine language subroutine at address
            OpCode::_0NNN { .. } => {
                Err(Chip8Error::UnimplementedOpcodeError(opcode))
                // Ok(true)
            }
            // Clear the screen
            OpCode::_00E0 => {
                self.screen.clear();
                Ok(true)
            }
            //Return from subroutine
            OpCode::_00EE => {
                let left = (self.memory[(self.stack_ptr + 1) as usize] as u16) << 8;
                let right = self.memory[(self.stack_ptr + 2) as usize] as u16;
                self.pc = left | right;
                // println!(
                //     "Popping {:04X} onto stack as left: {:02X} and right: {:02X}",
                //     self.pc, left, right
                // );
                if (self.stack_ptr as u16) == 0xFFF {
                    return Err(Chip8Error::StackUnderflowError);
                }
                self.stack_ptr = self.stack_ptr + 2;
                Ok(false)
            }
            // Jump to address NNN
            OpCode::_1NNN { nnn } => {
                self.pc = nnn;
                Ok(false)
            }
            // Execute subroutine at address NNN
            OpCode::_2NNN { nnn } => {
                let pc_to_push = self.pc + 2;
                let left = (pc_to_push >> 8) as u8;
                let right = pc_to_push as u8;
                self.memory[(self.stack_ptr - 1) as usize] = left;
                self.memory[self.stack_ptr as usize] = right;
                // println!(
                //     "Pushing {:04X} onto stack as left: {:02X} and right: {:02X}",
                //     pc_to_push, left, right
                // );
                self.stack_ptr = self.stack_ptr - 2;
                self.pc = nnn;
                Ok(false)
            }
            // Skip the following instruction if the value of register VX equals NN
            OpCode::_3XNN { x, nn } => {
                let vx_val = self.v.nth(x);
                if vx_val == nn {
                    self.pc += 2;
                }
                Ok(true)
            }
            // Skip the following instruction if the value of register VX is not equal to NN
            OpCode::_4XNN { x, nn } => {
                let vx_val = self.v.nth(x);
                if vx_val != nn {
                    self.pc += 2;
                }
                Ok(true)
            }
            // Skip the following instruction if the value of register VX is equal to the value of register VY
            OpCode::_5XY0 { x, y } => {
                let vx_val = self.v.nth(x);
                let vy_val = self.v.nth(y);
                if vx_val == vy_val {
                    self.pc += 2;
                }
                Ok(true)
            }
            // Store number NN in register VX
            OpCode::_6XNN { x, nn } => {
                self.v.set(x, nn);

                Ok(true)
            }
            // Add the value NN to register VX
            OpCode::_7XNN { x, nn } => {
                let xval = self.v.nth(x);
                self.v.set(x, xval.wrapping_add(nn));
                Ok(true)
            }
            // Store the value of register VY in register VX
            OpCode::_8XY0 { x, y } => {
                let val = self.v.nth(y);
                self.v.set(x, val);

                Ok(true)
            }
            // Set VX to VX OR VY
            OpCode::_8XY1 { x, y } => {
                let xval = self.v.nth(x);
                let yval = self.v.nth(y);
                self.v.set(x, xval | yval);
                self.v.set(0xF, 0);
                Ok(true)
            }
            // Set VX to VX AND VY
            OpCode::_8XY2 { x, y } => {
                let xval = self.v.nth(x);
                let yval = self.v.nth(y);
                self.v.set(x, xval & yval);
                self.v.set(0xF, 0);
                Ok(true)
            }
            // Set VX to VX XOR VY
            OpCode::_8XY3 { x, y } => {
                let xval = self.v.nth(x);
                let yval = self.v.nth(y);
                self.v.set(x, xval ^ yval);
                self.v.set(0xF, 0);
                Ok(true)
            }
            // Add the value of register VY to register VX
            // Set VF to 01 if a carry occurs
            // Set VF to 00 if a carry does not occur
            OpCode::_8XY4 { x, y } => {
                let xval = self.v.nth(x) as u16;
                let yval = self.v.nth(y) as u16;
                let result = xval + yval;
                self.v.set(x, result as u8);
                self.v.set(0xF, ((result & 0x0100) >> 8) as u8);
                Ok(true)
            }
            // Subtract the value of register VY from register VX
            // Set VF to 00 if a borrow occurs
            // Set VF to 01 if a borrow does not occur
            OpCode::_8XY5 { x, y } => {
                let xval = self.v.nth(x) as u16;
                let yval = self.v.nth(y) as u16;
                let result = xval.wrapping_sub(yval);

                self.v.set(x, result as u8);
                self.v.set(0xF, if yval > xval { 0 } else { 1 });
                // println!(
                //     "0xF: {:02X} x: {:02X} y: {:02X} result: {:04X} result & 0x0100: {:04X}",
                //     self.v[0xF],
                //     xval,
                //     yval,
                //     result,
                //     (result & 0x0100) >> 8
                // );
                Ok(true)
            }
            // Store the value of register VY shifted right one bit in register VX¹
            // Set register VF to the least significant bit prior to the shift
            // VY is unchanged
            OpCode::_8XY6 { x, y } => {
                let yval = self.v.nth(y);
                self.v.set(x, yval >> 1);
                self.v.set(0xF, (yval & 0x01) as u8);
                Ok(true)
            }
            // Set register VX to the value of VY minus VX
            // Set VF to 00 if a borrow occurs
            // Set VF to 01 if a borrow does not occur
            OpCode::_8XY7 { x, y } => {
                let xval = self.v.nth(x) as u16;
                let yval = self.v.nth(y) as u16;
                let result = yval.wrapping_sub(xval);
                self.v.set(x, result as u8);
                self.v.set(0xF, if xval > yval { 0 } else { 1 });

                Ok(true)
            }
            // Store the value of register VY shifted left one bit in register VX¹
            // Set register VF to the most significant bit prior to the shift
            // VY is unchanged
            OpCode::_8XYE { x, y } => {
                let yval = self.v.nth(y);
                self.v.set(x, yval << 1);
                self.v.set(0xF, (yval >> 7) as u8);
                Ok(true)
            }
            // Skip the following instruction if the value of register VX is not equal to the value of register VY
            OpCode::_9XY0 { x, y } => {
                if self.v.nth(x) != self.v.nth(y) {
                    self.pc += 2;
                }
                Ok(true)
            }
            // Store memory address NNN in register I
            OpCode::_ANNN { nnn } => {
                self.i = nnn;
                Ok(true)
            }
            // Jump to address NNN + V0
            OpCode::_BNNN { nnn } => {
                self.pc = nnn + self.v[0] as u16;
                Ok(true)
            }
            // Set VX to a random number with a mask of NN
            OpCode::_CXNN { x, nn } => {
                let val = rand::thread_rng().gen_range(0x00..=0xFF);
                self.v.set(x, val & nn);

                Ok(true)
            }
            // Draw a sprite at position VX, VY with N bytes of sprite data starting at the address stored in I
            // Set VF to 01 if any set pixels are changed to unset, and 00 otherwise
            OpCode::_DXYN { x, y, n } => {
                let mem_start = self.i as usize;
                let mem_end = mem_start + n as usize;
                let memslice = &self.memory[mem_start..mem_end];
                let was_unset =
                    self.screen
                        .draw_sprite(self.v[x as usize], self.v[y as usize], memslice);
                self.v.set(0xF, was_unset as u8);
                Ok(true)
            }
            //Skip the following instruction if the key corresponding to the hex value currently stored in register VX is pressed
            OpCode::_EX9E { x } => {
                let key = self.input.get_key();
                if key == Some(self.v[x as usize]) {
                    self.pc += 2;
                }
                Ok(true)
            }
            // Skip the following instruction if the key corresponding to the hex value currently stored in register VX is not pressed
            OpCode::_EXA1 { x } => {
                let key = self.input.get_key();
                if key != Some(self.v[x as usize]) {
                    self.pc += 2;
                }
                Ok(true)
            }
            // Store the current value of the delay timer in register VX
            OpCode::_FX07 { x } => {
                self.v.set(x, self.timer);
                Ok(true)
            }
            // Wait for a keypress and store the result in register VX
            OpCode::_FX0A { x } => {
                let key = self.input.get_key();
                if key.is_some() {
                    self.v.set(x, key.unwrap());
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            // Set the delay timer to the value of register VX
            OpCode::_FX15 { x } => {
                self.timer = self.v.nth(x);
                Ok(true)
            }
            // Set the sound timer to the value of register VX
            OpCode::_FX18 { x } => {
                self.sound = self.v.nth(x);
                Ok(true)
            }

            // Add the value stored in register VX to register I
            OpCode::_FX1E { x } => {
                self.i = self.i + self.v[x as usize] as u16;
                Ok(true)
            }

            // Set I to the memory address of the sprite data corresponding to the hexadecimal digit stored in register VX
            OpCode::_FX29 { x } => {
                let vs = self.v[x as usize] % 16;
                self.i = FONT_START_ADDR + ((vs as u16) * 5);
                // self.pc += 2; // TODO: Why is this here?
                Ok(true)
            }

            // Store the binary-coded decimal equivalent of the value stored in register VX at addresses I, I + 1, and I + 2
            OpCode::_FX33 { x } => {
                let val = self.v.nth(x);
                self.memory[self.i as usize] = val / 100;
                self.memory[self.i as usize + 1] = (val / 10) % 10;
                self.memory[self.i as usize + 2] = val % 10;
                Ok(true)
            }

            // Store the values of registers V0 to VX inclusive in memory starting at address I
            // I is set to I + X + 1 after operation
            OpCode::_FX55 { x } => {
                for reg in 0..=x {
                    self.memory[(self.i + reg as u16) as usize] = self.v.nth(reg);
                }
                self.i = self.i + x as u16 + 1;
                Ok(true)
            }
            // Fill registers V0 to VX inclusive with the values stored in memory starting at address I
            // I is set to I + X + 1 after operation
            OpCode::_FX65 { x } => {
                for reg in 0..=x {
                    self.v.set(reg, self.memory[(self.i + reg as u16) as usize]);
                }
                self.i = self.i + x as u16 + 1;
                Ok(true)
            }
            _ => Err(Chip8Error::UnknownOpcodeError(opcode)),
        };
        let Ok(increment_pc) = res else {
            return Err(res.unwrap_err());
        };
        if increment_pc {
            self.pc += 2;
        }
        return Ok(());
    }
}

impl<TScreen, TInput> Debug for CPU<'_, TScreen, TInput>
where
    TScreen: Chip8Screen,
    TInput: Chip8Input,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mapped_registers = self
            .v
            .iter()
            .map(|x| format!("{:02X}", x))
            .collect::<Vec<String>>()
            .join(", ");
        write!(
            f,
            "CPU {{ r_v: [{:#}], r_i: {:04X}, r_timer: {:02X}, r_sound: {:02X}, pc: {:02X} }}",
            mapped_registers, self.i, self.timer, self.sound, self.pc
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{test::NoopScreen, NoopInput};

    #[test]
    fn test_cpu() {
        let cpu = CPU::new(&NoopScreen, &NoopInput);
        hexdump::hexdump(cpu.memory.as_ref());
        let first_font_char = cpu.memory[usize::from(FONT_START_ADDR)];
        assert_eq!(first_font_char, 0xF0);
        let last_font_char = cpu.memory[usize::from(FONT_START_ADDR) + FONT_BUFFER.len() - 1];
        assert_eq!(last_font_char, 0x80);
    }

    mod instructions {
        use super::*;
        use crate::{
            run,
            test::{op_run_program, NoopScreen},
        };

        #[test]
        fn _3xnn() {
            let mut cpu = CPU::new(&NoopScreen, &NoopInput);
            op_run_program(
                &mut cpu,
                [
                    OpCode::_6XNN { x: 0, nn: 0x12 },
                    OpCode::_6XNN { x: 1, nn: 0x12 },
                    OpCode::_3XNN { x: 0, nn: 0x12 },
                    OpCode::_7XNN { x: 0, nn: 0x03 },
                    OpCode::_3XNN { x: 1, nn: 0x13 },
                    OpCode::_7XNN { x: 1, nn: 0x03 },
                ]
                .as_slice(),
            );
            assert_eq!(cpu.v[0], 0x12); // It should skip updating reg 0
            assert_eq!(cpu.v[1], 0x15); // It should update reg 1
        }

        #[test]
        fn _6xnn() {
            let mut cpu = CPU::new(&NoopScreen, &NoopInput);
            run! {
                cpu,
                _6XNN { x: 0, nn: 0x12 },
            };
            assert_eq!(cpu.v[0], 0x12);
        }

        #[test]
        fn _7xnn() {
            let mut cpu = CPU::new(&NoopScreen, &NoopInput);
            run! {
                cpu,
                _6XNN { x: 0, nn: 0x12 },
                _7XNN { x: 0, nn: 0x03 },
            }

            assert_eq!(cpu.v[0], 0x15);
        }

        #[test]
        fn _8xy0() {
            let mut cpu = CPU::new(&NoopScreen, &NoopInput);
            run! {
                cpu,
                _6XNN { x: 0, nn: 0x12 },
                _6XNN { x: 1, nn: 0x13 },
                _8XY0 { x: 0, y: 1 },
            }
            assert_eq!(cpu.v[0], 0x13);
            assert_eq!(cpu.v[1], 0x13);
        }

        #[test]
        fn _8xy1() {
            let mut cpu = CPU::new(&NoopScreen, &NoopInput);
            run! {
                cpu,
                _6XNN { x: 0, nn: 0x12 },
                _6XNN { x: 1, nn: 0x13 },
                _8XY1 { x: 0, y: 1 },
            }
            assert_eq!(cpu.v[0], 0x12 | 0x13);
            assert_eq!(cpu.v[1], 0x13);
        }

        #[test]
        fn _8xy2() {
            let mut cpu = CPU::new(&NoopScreen, &NoopInput);
            run! {
                cpu,
                _6XNN { x: 0, nn: 0x12 },
                _6XNN { x: 1, nn: 0x13 },
                _8XY2 { x: 0, y: 1 },
            }
            assert_eq!(cpu.v[0], 0x12 & 0x13);
            assert_eq!(cpu.v[1], 0x13);
        }

        #[test]
        fn _8xy3() {
            let mut cpu = CPU::new(&NoopScreen, &NoopInput);
            run! {
                cpu,
                _6XNN { x: 0, nn: 0x12 },
                _6XNN { x: 1, nn: 0x13 },
                _8XY3 { x: 0, y: 1 },
            }
            assert_eq!(cpu.v[0], 0x12 ^ 0x13);
            assert_eq!(cpu.v[1], 0x13);
        }

        #[test]
        fn _8xy4() {
            let mut cpu = CPU::new(&NoopScreen, &NoopInput);
            run! {
                cpu,
                _6XNN { x: 0, nn: 0x12 },
                _6XNN { x: 1, nn: 0x13 },
                _8XY4 { x: 0, y: 1 },
            }
            assert_eq!(cpu.v[0], 0x12 + 0x13);
            assert_eq!(cpu.v[1], 0x13);
            assert_eq!(cpu.v[0xF], 0);
            cpu.reset();
            run! {
                cpu,
                _6XNN { x: 0, nn: 0xFF },
                _6XNN { x: 1, nn: 0xFF },
                _8XY4 { x: 0, y: 1 },
            }
            assert_eq!(cpu.v[0], u8::wrapping_add(0xFF, 0xFF));
            assert_eq!(cpu.v[1], 0xFF);
            assert_eq!(cpu.v[0xF], 0x01);
        }

        #[test]
        fn _8xy5() {
            let mut cpu = CPU::new(&NoopScreen, &NoopInput);
            run! {
                cpu,
                _6XNN { x: 0, nn: 0x12 },
                _6XNN { x: 1, nn: 0x13 },
                _8XY5 { x: 1, y: 0 },
            }
            assert_eq!(cpu.v[0], 0x12);
            assert_eq!(cpu.v[1], u8::wrapping_sub(0x13, 0x12));
            assert_eq!(cpu.v[0xF], 1);
            cpu.reset();
            run! {
                cpu,
                _6XNN { x: 0, nn: 0x12 },
                _6XNN { x: 1, nn: 0x13 },
                _8XY5 { x: 0, y: 1 },
            }
            assert_eq!(cpu.v[0], u8::wrapping_sub(0x12, 0x13));
            assert_eq!(cpu.v[1], 0x13);
            assert_eq!(cpu.v[0xF], 0);
        }

        #[test]
        fn _8xy6() {
            let mut cpu = CPU::new(&NoopScreen, &NoopInput);
            run! {
                cpu,
                _6XNN { x: 0, nn: 0x12 },
                _6XNN { x: 1, nn: 0x13 },
                _8XY6 { x: 0, y: 1 },
            }
            assert_eq!(cpu.v[0], 0x13 >> 1);
            assert_eq!(cpu.v[1], 0x13);
            assert_eq!(cpu.v[0xF], 1);

            cpu.reset();

            run! {
                cpu,
                _6XNN { x: 0, nn: 0x12 },
                _6XNN { x: 1, nn: 0x13 },
                _8XY6 { x: 1, y: 0 },
            }
            assert_eq!(cpu.v[0], 0x12);
            assert_eq!(cpu.v[1], 0x12 >> 1);
            assert_eq!(cpu.v[0xF], 0);
        }

        #[test]
        fn _8xy7() {
            let mut cpu = CPU::new(&NoopScreen, &NoopInput);
            run! {
                cpu,
                _6XNN { x: 0, nn: 0x12 },
                _6XNN { x: 1, nn: 0x13 },
                _8XY7 { x: 0, y: 1 },
            }
            assert_eq!(cpu.v[0], u8::wrapping_sub(0x13, 0x12));
            assert_eq!(cpu.v[1], 0x13);
            assert_eq!(cpu.v[0xF], 0x01);

            cpu.reset();

            run! {
                cpu,
                _6XNN { x: 0, nn: 0x12 },
                _6XNN { x: 1, nn: 0x13 },
                _8XY7 { x: 1, y: 0 },
            }
            assert_eq!(cpu.v[0], 0x12);
            assert_eq!(cpu.v[1], u8::wrapping_sub(0x12, 0x13));
            assert_eq!(cpu.v[0xF], 0x00);
        }

        #[test]
        fn _8xye() {
            let mut cpu = CPU::new(&NoopScreen, &NoopInput);
            run! {
                cpu,
                _6XNN { x: 0, nn: 0x12 },
                _6XNN { x: 1, nn: 0xFF },
                _8XYE { x: 0, y: 1 },
            }
            assert_eq!(cpu.v[0], 0xFF << 1);
            assert_eq!(cpu.v[1], 0xFF);
            assert_eq!(cpu.v[0xF], 1);

            cpu.reset();

            run! {
                cpu,
                _6XNN { x: 0, nn: 0xFF },
                _6XNN { x: 1, nn: 0x12 },
                _8XYE { x: 0, y: 1 },
            }
            assert_eq!(cpu.v[0], 0x12 << 1);
            assert_eq!(cpu.v[1], 0x12);
            assert_eq!(cpu.v[0xF], 0);
        }
    }
}
