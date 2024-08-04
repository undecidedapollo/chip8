use crate::{error::Chip8AssemblerError, lexer::Token, parser::Statement};
use chip8_core::OpCode;

pub(crate) fn strip_hex_u8(instruction: &str) -> Option<u8> {
    if instruction.starts_with("0x") {
        u8::from_str_radix(&instruction[2..], 16).ok()
    } else {
        u8::from_str_radix(instruction, 16).ok()
    }
}

pub(crate) fn strip_hex_u16(instruction: &str) -> Option<u16> {
    if instruction.starts_with("0x") {
        u16::from_str_radix(&instruction[2..], 16).ok()
    } else {
        u16::from_str_radix(instruction, 16).ok()
    }
}

fn nn(instruction: &str) -> Option<u8> {
    strip_hex_u8(instruction).map(|val| val & 0xFF)
}

fn nnn(instruction: &str) -> Option<u16> {
    strip_hex_u16(instruction).map(|val| val & 0x0FFF)
}

fn reg(instruction: &str) -> Option<u8> {
    strip_hex_u8(instruction).map(|val| val & 0xF)
}

fn xnn(
    statement: &Statement,
    register: &str,
    value: &str,
) -> Result<(u8, u8), Chip8AssemblerError> {
    match (reg(register), nn(value)) {
        (Some(x), Some(nn)) => Ok((x, nn)),
        (None, None) => Err(Chip8AssemblerError::InvalidStatementError(
            statement.clone(),
            "Invalid register and number".to_owned(),
        )),
        (None, _) => Err(Chip8AssemblerError::InvalidStatementError(
            statement.clone(),
            "Invalid register".to_owned(),
        )),
        (_, None) => Err(Chip8AssemblerError::InvalidStatementError(
            statement.clone(),
            "Invalid number".to_owned(),
        )),
    }
}

fn xy(statement: &Statement, regx: &str, regy: &str) -> Result<(u8, u8), Chip8AssemblerError> {
    match (reg(regx), reg(regy)) {
        (Some(x), Some(y)) => Ok((x, y)),
        (None, None) => Err(Chip8AssemblerError::InvalidStatementError(
            statement.clone(),
            "Invalid register for x and y".to_owned(),
        )),
        (None, _) => Err(Chip8AssemblerError::InvalidStatementError(
            statement.clone(),
            "Invalid register for x".to_owned(),
        )),
        (_, None) => Err(Chip8AssemblerError::InvalidStatementError(
            statement.clone(),
            "Invalid register for y".to_owned(),
        )),
    }
}

fn xyn(
    statement: &Statement,
    regx: &str,
    regy: &str,
    val: &str,
) -> Result<(u8, u8, u8), Chip8AssemblerError> {
    match (reg(regx), reg(regy), reg(val)) {
        (Some(x), Some(y), Some(n)) => Ok((x, y, n)),
        _ => Err(Chip8AssemblerError::InvalidStatementError(
            statement.clone(),
            "Invalid value for x, y, and n".to_owned(),
        )),
    }
}

impl TryFrom<Statement> for OpCode {
    type Error = Chip8AssemblerError;

    fn try_from(statement: Statement) -> Result<Self, Self::Error> {
        match (statement.opcode.as_str(), statement.operands.as_slice()) {
            ("SYS", [Token::Number(number)]) => match nnn(number) {
                Some(nnn) => Ok(OpCode::_0NNN { nnn }),
                None => Err(Chip8AssemblerError::InvalidStatementError(
                    statement.clone(),
                    "Invalid number".to_owned(),
                )),
            },
            ("CLR", []) => Ok(OpCode::_00E0),
            ("RTS", []) => Ok(OpCode::_00EE),
            ("JUMP", [Token::Number(number)]) => nnn(number)
                .map(|nnn| OpCode::_1NNN { nnn })
                .ok_or(Chip8AssemblerError::InvalidStatementError(
                    statement.clone(),
                    "Invalid number".to_owned(),
                )),
            ("CALL", [Token::Number(number)]) => nnn(number)
                .map(|nnn| OpCode::_2NNN { nnn })
                .ok_or(Chip8AssemblerError::InvalidStatementError(
                    statement.clone(),
                    format!(
                        "Invalid number: {}, expected a number between 0x000 and 0xFFF",
                        number
                    ),
                )),
            ("SKE", [Token::Number(register), Token::Number(number)]) => {
                xnn(&statement, register, number).map(|(x, nn)| OpCode::_3XNN { x, nn })
            }
            ("SKNE", [Token::Number(register), Token::Number(number)]) => {
                xnn(&statement, register, number).map(|(x, nn)| OpCode::_4XNN { x, nn })
            }
            ("SKRE", [Token::Number(xreg), Token::Number(yreg)]) => {
                xy(&statement, xreg, yreg).map(|(x, y)| OpCode::_5XY0 { x, y })
            }
            ("LOAD", [Token::Number(xreg), Token::Number(val)]) => {
                xnn(&statement, xreg, val).map(|(x, y)| OpCode::_6XNN { x, nn: y })
            }
            ("ADD", [Token::Number(xreg), Token::Number(val)]) => {
                xnn(&statement, xreg, val).map(|(x, y)| OpCode::_7XNN { x, nn: y })
            }
            ("MOVE", [Token::Number(xreg), Token::Number(val)]) => {
                xy(&statement, xreg, val).map(|(x, y)| OpCode::_8XY0 { x, y })
            }
            ("OR", [Token::Number(xreg), Token::Number(val)]) => {
                xy(&statement, xreg, val).map(|(x, y)| OpCode::_8XY1 { x, y })
            }
            ("AND", [Token::Number(xreg), Token::Number(val)]) => {
                xy(&statement, xreg, val).map(|(x, y)| OpCode::_8XY2 { x, y })
            }
            ("XOR", [Token::Number(xreg), Token::Number(val)]) => {
                xy(&statement, xreg, val).map(|(x, y)| OpCode::_8XY3 { x, y })
            }
            ("ADDR", [Token::Number(xreg), Token::Number(yreg)]) => {
                xy(&statement, xreg, yreg).map(|(x, y)| OpCode::_8XY4 { x, y })
            }
            ("SUB", [Token::Number(xreg), Token::Number(yreg)]) => {
                xy(&statement, xreg, yreg).map(|(x, y)| OpCode::_8XY5 { x, y })
            }
            ("SHR", [Token::Number(xreg), Token::Number(yreg)]) => {
                xy(&statement, xreg, yreg).map(|(x, y)| OpCode::_8XY6 { x, y })
            }
            // TODO: Implement _8XY7
            ("SHL", [Token::Number(xreg), Token::Number(yreg)]) => {
                xy(&statement, xreg, yreg).map(|(x, y)| OpCode::_8XYE { x, y })
            }
            ("SKRNE", [Token::Number(xreg), Token::Number(yreg)]) => {
                xy(&statement, xreg, yreg).map(|(x, y)| OpCode::_9XY0 { x, y })
            }
            ("LOADI", [Token::Number(val)]) => match nnn(val) {
                Some(nnn) => Ok(OpCode::_ANNN { nnn }),
                None => Err(Chip8AssemblerError::InvalidStatementError(
                    statement.clone(),
                    "Invalid address".to_owned(),
                )),
            },
            ("JUMPI", [Token::Number(val)]) => match nnn(val) {
                Some(nnn) => Ok(OpCode::_BNNN { nnn }),
                None => Err(Chip8AssemblerError::InvalidStatementError(
                    statement.clone(),
                    "Invalid address".to_owned(),
                )),
            },
            ("RAND", [Token::Number(xreg), Token::Number(val)]) => {
                xnn(&statement, xreg, val).map(|(x, y)| OpCode::_CXNN { x, nn: y })
            }
            ("DRAW", [Token::Number(xreg), Token::Number(yreg), Token::Number(val)]) => {
                xyn(&statement, xreg, yreg, val).map(|(x, y, n)| OpCode::_DXYN { x, y, n })
            }
            ("SKPR", [Token::Number(xreg)]) => match reg(xreg) {
                Some(reg) => Ok(OpCode::_EX9E { x: reg }),
                None => Err(Chip8AssemblerError::InvalidStatementError(
                    statement.clone(),
                    "Invalid key".to_owned(),
                )),
            },
            ("SKUP", [Token::Number(xreg)]) => match reg(xreg) {
                Some(reg) => Ok(OpCode::_EXA1 { x: reg }),
                None => Err(Chip8AssemblerError::InvalidStatementError(
                    statement.clone(),
                    "Invalid key".to_owned(),
                )),
            },
            ("MOVED", [Token::Number(xreg)]) => match reg(xreg) {
                Some(reg) => Ok(OpCode::_FX07 { x: reg }),
                None => Err(Chip8AssemblerError::InvalidStatementError(
                    statement.clone(),
                    "Invalid key".to_owned(),
                )),
            },
            ("KEYD", [Token::Number(xreg)]) => match reg(xreg) {
                Some(reg) => Ok(OpCode::_FX0A { x: reg }),
                None => Err(Chip8AssemblerError::InvalidStatementError(
                    statement.clone(),
                    "Invalid key".to_owned(),
                )),
            },
            ("LOADD", [Token::Number(xreg)]) => match reg(xreg) {
                Some(reg) => Ok(OpCode::_FX15 { x: reg }),
                None => Err(Chip8AssemblerError::InvalidStatementError(
                    statement.clone(),
                    "Invalid key".to_owned(),
                )),
            },
            ("LOADS", [Token::Number(xreg)]) => match reg(xreg) {
                Some(reg) => Ok(OpCode::_FX18 { x: reg }),
                None => Err(Chip8AssemblerError::InvalidStatementError(
                    statement.clone(),
                    "Invalid key".to_owned(),
                )),
            },
            ("ADDI", [Token::Number(xreg)]) => match reg(xreg) {
                Some(reg) => Ok(OpCode::_FX1E { x: reg }),
                None => Err(Chip8AssemblerError::InvalidStatementError(
                    statement.clone(),
                    "Invalid key".to_owned(),
                )),
            },
            ("LDSPR", [Token::Number(xreg)]) => match reg(xreg) {
                Some(reg) => Ok(OpCode::_FX29 { x: reg }),
                None => Err(Chip8AssemblerError::InvalidStatementError(
                    statement.clone(),
                    "Invalid key".to_owned(),
                )),
            },
            ("BCD", [Token::Number(xreg)]) => match reg(xreg) {
                Some(reg) => Ok(OpCode::_FX33 { x: reg }),
                None => Err(Chip8AssemblerError::InvalidStatementError(
                    statement.clone(),
                    "Invalid key".to_owned(),
                )),
            },
            ("STOR", [Token::Number(xreg)]) => match reg(xreg) {
                Some(reg) => Ok(OpCode::_FX55 { x: reg }),
                None => Err(Chip8AssemblerError::InvalidStatementError(
                    statement.clone(),
                    "Invalid key".to_owned(),
                )),
            },
            ("READ", [Token::Number(xreg)]) => match reg(xreg) {
                Some(reg) => Ok(OpCode::_FX65 { x: reg }),
                None => Err(Chip8AssemblerError::InvalidStatementError(
                    statement.clone(),
                    "Invalid key".to_owned(),
                )),
            },
            _ => Err(Chip8AssemblerError::InvalidStatementError(
                statement,
                "Invalid opcode".to_owned(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opcode_try_from() {
        let statement = Statement {
            label: None,
            opcode: "SKE".to_owned(),
            operands: vec![
                Token::Number("0x1".to_owned()),
                Token::Number("0xF3".to_owned()),
            ],
            comment: None,
        };
        assert_eq!(
            OpCode::try_from(statement),
            Ok(OpCode::_3XNN { x: 0x1, nn: 0xF3 })
        );
    }
}
