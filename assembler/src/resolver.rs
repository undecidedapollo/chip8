use std::vec;

use chip8_core::OpCode;

use crate::{
    error::Chip8AssemblerError,
    lexer::Token,
    parser::{ParseResult, Statement},
};

pub struct Resolver<TIterator>
where
    TIterator: Iterator<Item = ParseResult>,
{
    iterated: bool,
    source: TIterator,
    labels: Vec<(String, u16)>,
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

        let res = self
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

        return Ok(res);
    }
}
