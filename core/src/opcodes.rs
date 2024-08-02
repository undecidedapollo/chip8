use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub enum OpCodes {
    _0NNN { nnn: u16 },
    _00E0,
    _00EE,
    _1NNN { nnn: u16 },
    _2NNN { nnn: u16 },
    _3XNN { x: u8, nn: u8 },
    _4XNN { x: u8, nn: u8 },
    _5XY0 { x: u8, y: u8 },
    _6XNN { x: u8, nn: u8 },
    _7XNN { x: u8, nn: u8 },
    _8XY0 { x: u8, y: u8 },
    _8XY1 { x: u8, y: u8 },
    _8XY2 { x: u8, y: u8 },
    _8XY3 { x: u8, y: u8 },
    _8XY4 { x: u8, y: u8 },
    _8XY5 { x: u8, y: u8 },
    _8XY6 { x: u8, y: u8 },
    _8XY7 { x: u8, y: u8 },
    _8XYE { x: u8, y: u8 },
    _9XY0 { x: u8, y: u8 },
    _ANNN { nnn: u16 },
    _BNNN { nnn: u16 },
    _CXNN { x: u8, nn: u8 },
    _DXYN { x: u8, y: u8, n: u8 },
    _EX9E { x: u8 },
    _EXA1 { x: u8 },
    _FX07 { x: u8 },
    _FX0A { x: u8 },
    _FX15 { x: u8 },
    _FX18 { x: u8 },
    _FX1E { x: u8 },
    _FX29 { x: u8 },
    _FX33 { x: u8 },
    _FX55 { x: u8 },
    _FX65 { x: u8 },
}

fn nn(instruction: u16) -> u8 {
    return (instruction & 0xFF) as u8;
}

fn nnn(instruction: u16) -> u16 {
    return (instruction & 0xFFF) as u16;
}

#[derive(Error, Debug)]
pub enum Chip8Error {
    #[error("Invalid opcode: {0}")]
    InvalidOpcodeError(u16),
    #[error("Unknown opcode: {0:?}")]
    UnknownOpcodeError(OpCodes),
    #[error("Unimplemented opcode: {0:?}")]
    UnimplementedOpcodeError(OpCodes),
    #[error("Stack underflow")]
    StackUnderflowError,
}

impl TryFrom<(u8, u8)> for OpCodes {
    type Error = Chip8Error;

    fn try_from((op1, op2): (u8, u8)) -> Result<Self, Self::Error> {
        let char1 = op1 >> 4;
        let char2 = op1 & 0x0F;
        let char3 = op2 >> 4;
        let char4 = op2 & 0x0F;
        let instruction: u16 = op1 as u16 * 256 + op2 as u16;

        match (char1, char2, char3, char4) {
            (0, 0, 0xE, 0) => Ok(Self::_00E0),
            (0, 0, 0xE, 0xE) => Ok(Self::_00EE),
            (0, _, _, _) => Ok(Self::_0NNN {
                nnn: nnn(instruction),
            }),
            (1, _, _, _) => Ok(Self::_1NNN {
                nnn: nnn(instruction),
            }),
            (2, _, _, _) => Ok(Self::_2NNN {
                nnn: nnn(instruction),
            }),
            (3, x, _, _) => Ok(Self::_3XNN {
                x,
                nn: nn(instruction),
            }),
            (4, x, _, _) => Ok(Self::_4XNN {
                x,
                nn: nn(instruction),
            }),
            (5, x, y, 0) => Ok(Self::_5XY0 { x, y }),
            (6, x, _, _) => Ok(Self::_6XNN {
                x,
                nn: nn(instruction),
            }),
            (7, x, _, _) => Ok(Self::_7XNN {
                x,
                nn: nn(instruction),
            }),
            (8, x, y, 0) => Ok(Self::_8XY0 { x, y }),
            (8, x, y, 1) => Ok(Self::_8XY1 { x, y }),
            (8, x, y, 2) => Ok(Self::_8XY2 { x, y }),
            (8, x, y, 3) => Ok(Self::_8XY3 { x, y }),
            (8, x, y, 4) => Ok(Self::_8XY4 { x, y }),
            (8, x, y, 5) => Ok(Self::_8XY5 { x, y }),
            (8, x, y, 6) => Ok(Self::_8XY6 { x, y }),
            (8, x, y, 7) => Ok(Self::_8XY7 { x, y }),
            (8, x, y, 0xE) => Ok(Self::_8XYE { x, y }),
            (9, x, y, 0) => Ok(Self::_9XY0 { x, y }),
            (0xA, _, _, _) => Ok(Self::_ANNN {
                nnn: nnn(instruction),
            }),
            (0xB, _, _, _) => Ok(Self::_BNNN {
                nnn: nnn(instruction),
            }),
            (0xC, x, _, _) => Ok(Self::_CXNN {
                x,
                nn: nn(instruction),
            }),
            (0xD, x, y, n) => Ok(Self::_DXYN { x, y, n }),
            (0xE, x, 0x9, 0xE) => Ok(Self::_EX9E { x }),
            (0xE, x, 0xA, 0x1) => Ok(Self::_EXA1 { x }),
            (0xF, x, 0x0, 0x7) => Ok(Self::_FX07 { x }),
            (0xF, x, 0x0, 0xA) => Ok(Self::_FX0A { x }),
            (0xF, x, 0x1, 0x5) => Ok(Self::_FX15 { x }),
            (0xF, x, 0x1, 0x8) => Ok(Self::_FX18 { x }),
            (0xF, x, 0x1, 0xE) => Ok(Self::_FX1E { x }),
            (0xF, x, 0x2, 0x9) => Ok(Self::_FX29 { x }),
            (0xF, x, 0x3, 0x3) => Ok(Self::_FX33 { x }),
            (0xF, x, 0x5, 0x5) => Ok(Self::_FX55 { x }),
            (0xF, x, 0x6, 0x5) => Ok(Self::_FX65 { x }),
            _ => Err(Chip8Error::InvalidOpcodeError(instruction)),
        }
    }
}

fn left_bit(hex: u8) -> u8 {
    return hex << 4;
}

impl From<OpCodes> for (u8, u8) {
    fn from(op_code: OpCodes) -> Self {
        match op_code {
            OpCodes::_00E0 => (left_bit(0) & 0, 0xE),
            OpCodes::_00EE => (left_bit(0) & 0, 0xEE),
            OpCodes::_0NNN { nnn } => (left_bit(0) | (nnn >> 8) as u8, nnn as u8),
            OpCodes::_1NNN { nnn } => (left_bit(1) | (nnn >> 8) as u8, nnn as u8),
            OpCodes::_2NNN { nnn } => (left_bit(2) | (nnn >> 8) as u8, nnn as u8),
            OpCodes::_3XNN { x, nn } => (left_bit(3) | x, nn),
            OpCodes::_4XNN { x, nn } => (left_bit(4) | x, nn),
            OpCodes::_5XY0 { x, y } => (left_bit(5) | x, left_bit(y)),
            OpCodes::_6XNN { x, nn } => (left_bit(6) | x, nn),
            OpCodes::_7XNN { x, nn } => (left_bit(7) | x, nn),
            OpCodes::_8XY0 { x, y } => (left_bit(8) | x, left_bit(y)),
            OpCodes::_8XY1 { x, y } => (left_bit(8) | x, left_bit(y) | 0x01),
            OpCodes::_8XY2 { x, y } => (left_bit(8) | x, left_bit(y) | 0x02),
            OpCodes::_8XY3 { x, y } => (left_bit(8) | x, left_bit(y) | 0x03),
            OpCodes::_8XY4 { x, y } => (left_bit(8) | x, left_bit(y) | 0x04),
            OpCodes::_8XY5 { x, y } => (left_bit(8) | x, left_bit(y) | 0x05),
            OpCodes::_8XY6 { x, y } => (left_bit(8) | x, left_bit(y) | 0x06),
            OpCodes::_8XY7 { x, y } => (left_bit(8) | x, left_bit(y) | 0x07),
            OpCodes::_8XYE { x, y } => (left_bit(8) | x, left_bit(y) | 0x0E),
            OpCodes::_9XY0 { x, y } => (left_bit(9) | x, left_bit(y)),
            OpCodes::_ANNN { nnn } => (left_bit(0xA) | (nnn >> 8) as u8, nnn as u8),
            OpCodes::_BNNN { nnn } => (left_bit(0xB) | (nnn >> 8) as u8, nnn as u8),
            OpCodes::_CXNN { x, nn } => (left_bit(0xC) | x, nn),
            OpCodes::_DXYN { x, y, n } => (left_bit(0xD) | x, left_bit(y) | n),
            OpCodes::_EX9E { x } => (left_bit(0xE) | x, 0x9E),
            OpCodes::_EXA1 { x } => (left_bit(0xE) | x, 0xA1),
            OpCodes::_FX07 { x } => (left_bit(0xF) | x, 0x07),
            OpCodes::_FX0A { x } => (left_bit(0xF) | x, 0x0A),
            OpCodes::_FX15 { x } => (left_bit(0xF) | x, 0x15),
            OpCodes::_FX18 { x } => (left_bit(0xF) | x, 0x18),
            OpCodes::_FX1E { x } => (left_bit(0xF) | x, 0x1E),
            OpCodes::_FX29 { x } => (left_bit(0xF) | x, 0x29),
            OpCodes::_FX33 { x } => (left_bit(0xF) | x, 0x33),
            OpCodes::_FX55 { x } => (left_bit(0xF) | x, 0x55),
            OpCodes::_FX65 { x } => (left_bit(0xF) | x, 0x65),
        }
    }
}

pub fn convert_opcodes_into_u8_tuples(slice: &[OpCodes]) -> Vec<(u8, u8)> {
    slice.iter().map(|&b| b.into()).collect()
}

pub fn convert_opcodes_into_u8(slice: &[OpCodes]) -> Vec<u8> {
    slice
        .iter()
        .flat_map(|&b| {
            let (op1, op2) = b.into();
            [op1 as u8, op2 as u8]
        })
        .collect()
}

pub fn convert_opcodes_into_u16(slice: &[OpCodes]) -> Vec<u16> {
    slice
        .iter()
        .map(|&b| {
            let (op1, op2) = b.into();
            (op1 as u16) * 256 + op2 as u16
        })
        .collect()
}

pub fn convert_u8_tuples_into_opcodes(slice: &[(u8, u8)]) -> Result<Vec<OpCodes>, Chip8Error> {
    slice
        .iter()
        .map(|(op1, op2)| OpCodes::try_from((*op1, *op2)))
        .collect()
}

pub fn convert_u8_into_opcodes(slice: &[u8]) -> Result<Vec<OpCodes>, Chip8Error> {
    slice
        .chunks(2)
        .filter_map(|chunk| {
            // Convert chunk to tuple if it has 2 elements
            if chunk.len() == 2 {
                Some((chunk[0], chunk[1]).try_into())
            } else {
                None // Ignore incomplete chunks
            }
        })
        .collect()
}
