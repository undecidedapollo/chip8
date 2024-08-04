use std::vec;

use chip8_core::OpCode;

use crate::{
    error::Chip8AssemblerError,
    lexer::Token,
    opcodes::strip_hex_u16,
    opcodes::strip_hex_u8,
    parser::{Declaration, ParseResult, Statement},
};

struct MemPin {
    label: String,
    size: u16,
    members: Option<Vec<u8>>,
}

pub struct Resolver<TIterator>
where
    TIterator: Iterator<Item = ParseResult>,
{
    iterated: bool,
    source: TIterator,
    labels: Vec<(String, u16)>,
    mem_pins: Vec<MemPin>,
    data: Vec<Statement>,
}

const PGRM_LOAD_START_ADDR: u16 = 0x200;

impl<TIterator> Resolver<TIterator>
where
    TIterator: Iterator<Item = ParseResult>,
{
    pub fn from_iter(source: TIterator) -> Self {
        return Resolver {
            iterated: false,
            source,
            labels: Vec::new(),
            data: Vec::new(),
            mem_pins: Vec::new(),
        };
    }

    fn current_address(&self) -> u16 {
        return <u16>::try_from(self.data.len() * 2).unwrap() + PGRM_LOAD_START_ADDR;
    }

    fn process_iterator(&mut self) -> bool {
        if self.iterated {
            return false;
        }
        self.iterated = true;

        loop {
            match self.source.next() {
                Some(ParseResult::Label(label)) => {
                    self.labels.push((label, self.current_address()));
                }
                Some(ParseResult::Statement(statement)) => {
                    if let Some(label) = &statement.label {
                        self.labels.push((label.to_owned(), self.current_address()));
                    }
                    self.data.push(statement);
                }
                Some(ParseResult::Declaration(Declaration {
                    label,
                    size: Some(Token::Number(size)),
                    members: None,
                })) => {
                    self.mem_pins.push(MemPin {
                        label,
                        size: strip_hex_u16(&size).unwrap(),
                        members: None,
                    });
                }
                Some(ParseResult::Declaration(Declaration {
                    label,
                    size: None,
                    members: Some(members),
                })) => {
                    self.mem_pins.push(MemPin {
                        label,
                        size: members.len() as u16,
                        members: Some(
                            members
                                .iter()
                                .map(|token| {
                                    let Token::Number(token) = token else {
                                        return 0;
                                    };
                                    strip_hex_u8(token).unwrap()
                                })
                                .collect(),
                        ),
                    });
                }
                Some(ParseResult::Declaration(Declaration {
                    label,
                    size: Some(Token::Number(size)),
                    members: Some(members),
                })) => {
                    assert_eq!(strip_hex_u16(&size).unwrap(), members.len() as u16);
                    self.mem_pins.push(MemPin {
                        label,
                        size: members.len() as u16,
                        members: Some(
                            members
                                .iter()
                                .map(|token| {
                                    let Token::Number(token) = token else {
                                        return 0;
                                    };
                                    strip_hex_u8(token).unwrap()
                                })
                                .collect(),
                        ),
                    });
                }
                Some(ParseResult::Declaration(x)) => {
                    panic!("Invalid declaration: {:?}", x); // TODO: Handle this
                }
                Some(ParseResult::Unknown(_)) => {}
                Some(ParseResult::Comment(_)) => {}
                None => {
                    break;
                }
            }
        }

        return true;
    }

    pub fn resolve(&mut self) -> Result<Vec<u8>, Chip8AssemblerError> {
        self.process_iterator();

        let (new_current_address, mem_pins_with_addresses) =
            self.mem_pins
                .iter()
                .fold((self.current_address(), Vec::new()), |mut acc, mem_pin| {
                    let address = acc.0;
                    acc.0 = acc.0 + mem_pin.size as u16;
                    acc.1.push((address, mem_pin));
                    acc
                });

        let mut code_res = self
            .data
            .iter()
            .map(|statement| {
                let mapped_operands = statement
                    .operands
                    .iter()
                    .map(|operand| match operand {
                        Token::Label(label) => {
                            let label = label.to_owned();
                            if let Some((_, address)) =
                                self.labels.iter().find(|(l, _)| l == &label)
                            {
                                return Ok(Token::Number(format!("{:#x}", address)));
                            } else if let Some((address, _)) = mem_pins_with_addresses
                                .iter()
                                .find(|(_, l)| l.label == label)
                            {
                                return Ok(Token::Number(format!("{:#x}", address)));
                            } else {
                                return Err(Chip8AssemblerError::UnknownLabelError(label));
                            }
                        }
                        operand => Ok(operand.clone()),
                    })
                    .collect::<Result<Vec<Token>, Chip8AssemblerError>>()?;

                let statement = Statement {
                    label: statement.label.clone(),
                    opcode: statement.opcode.clone(),
                    operands: mapped_operands,
                    comment: statement.comment.clone(),
                };

                let opcode = OpCode::try_from(statement.clone())?;

                let res = <(u8, u8)>::from(opcode);

                return Ok([res.0, res.1]);
            })
            .collect::<Result<Vec<[u8; 2]>, Chip8AssemblerError>>()?
            .iter()
            .flat_map(|f| f.clone())
            .collect::<Vec<u8>>();

        let data_res = mem_pins_with_addresses
            .iter()
            .flat_map(|(address, mem_pin)| {
                let mut buffer = vec![0; mem_pin.size as usize];
                for (i, member) in mem_pin.members.as_ref().unwrap().iter().enumerate() {
                    buffer[i] = *member;
                }
                return buffer;
            })
            .collect::<Vec<u8>>();
        code_res.extend(data_res);
        hexdump::hexdump(code_res.as_slice());
        return Ok(code_res);
    }
}
