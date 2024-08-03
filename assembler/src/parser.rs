use std::iter::Peekable;

use crate::lexer::Token;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Statement {
    pub label: Option<String>,
    pub opcode: String,
    pub operands: Vec<Token>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseResult {
    Comment(String),
    Label(String),
    Statement(Statement),
    Unknown(Vec<Token>),
}

pub struct Parser<TIterator>
where
    TIterator: Iterator<Item = Token>,
{
    tokens: Peekable<TIterator>,
}

impl<TIterator> Parser<TIterator>
where
    TIterator: Iterator<Item = Token>,
{
    pub fn from_iter(tokens: TIterator) -> Self {
        return Parser {
            tokens: tokens.peekable(),
        };
    }

    fn consume_whitespace(&mut self) -> bool {
        while let Some(token) = self.tokens.peek() {
            match token {
                Token::Whitespace('\n') => {
                    self.tokens.next();
                    return true;
                }
                Token::Whitespace(_) => {
                    self.tokens.next();
                }
                _ => {
                    return false;
                }
            }
        }

        true
    }

    fn parse_statement(&mut self) -> Option<ParseResult> {
        let label = if let Some(Token::Label(label)) = self.tokens.peek().cloned() {
            self.tokens.next();
            Some(label.to_owned())
        } else {
            None
        };
        let end_of_statement = self.consume_whitespace();
        if end_of_statement && label.is_none() {
            return None;
        } else if end_of_statement {
            return Some(ParseResult::Label(label.unwrap()));
        }
        let token = self.tokens.next();
        let Some(Token::Mneumonic(mneumonic)) = token else {
            if label.is_none() {
                return Some(ParseResult::Unknown(vec![token.unwrap()]));
            } else {
                return Some(ParseResult::Unknown(vec![
                    Token::Label(label.unwrap()),
                    token.unwrap(),
                ]));
            }
        };
        let end_of_statement = self.consume_whitespace();
        if end_of_statement {
            return Some(ParseResult::Statement(Statement {
                label,
                opcode: mneumonic.to_string(),
                operands: Vec::new(),
                comment: None,
            }));
        }
        let operands = self.parse_operands();
        let end_of_statement = self.consume_whitespace();
        if end_of_statement {
            return Some(ParseResult::Statement(Statement {
                label,
                opcode: mneumonic.to_string(),
                operands,
                comment: None,
            }));
        }

        let Some(Token::Comment(comment)) = self.tokens.peek().cloned() else {
            let mut base_vec = if label.is_none() {
                vec![Token::Mneumonic(mneumonic)]
            } else {
                vec![Token::Label(label.unwrap()), Token::Mneumonic(mneumonic)]
            };

            for operand in operands {
                base_vec.push(operand);
            }

            return Some(ParseResult::Unknown(base_vec));
        };

        self.tokens.next();
        return Some(ParseResult::Statement(Statement {
            label,
            opcode: mneumonic.to_string(),
            operands,
            comment: Some(comment.to_owned()),
        }));
    }

    fn parse_operands(&mut self) -> Vec<Token> {
        let mut operands = Vec::new();
        loop {
            match self.tokens.peek() {
                Some(Token::Number(_)) => {
                    operands.push(self.tokens.next().unwrap());
                }
                Some(Token::Label(_)) => {
                    operands.push(self.tokens.next().unwrap());
                }
                Some(Token::Whitespace('\n')) => {
                    break;
                }
                Some(Token::Whitespace(_)) => {
                    self.tokens.next();
                }
                _ => {
                    break;
                }
            }
        }

        return operands;
    }
}

impl<TIterator> Iterator for Parser<TIterator>
where
    TIterator: Iterator<Item = Token>,
{
    type Item = ParseResult;
    fn next(&mut self) -> Option<Self::Item> {
        match self.tokens.peek().cloned() {
            Some(Token::Comment(comment)) => {
                self.tokens.next();
                self.consume_whitespace();
                return Some(ParseResult::Comment(comment.to_owned()));
            }
            Some(Token::Whitespace(_)) => {
                self.consume_whitespace();
                self.next()
            }
            Some(Token::Label(_)) => self.parse_statement(),
            Some(Token::Mneumonic(_)) => self.parse_statement(),
            Some(Token::Number(number)) => {
                self.tokens.next();
                Some(ParseResult::Unknown(vec![Token::Number(number.to_owned())]))
            }
            Some(Token::Unknown(unknown)) => {
                self.tokens.next();
                Some(ParseResult::Unknown(vec![Token::Unknown(
                    unknown.to_owned(),
                )]))
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;

    use super::*;

    #[test]
    fn parse_comment() {
        let lexer = Lexer::from_iter("; comment".chars());
        let mut parser = Parser::from_iter(lexer);

        assert_eq!(
            parser.next(),
            Some(ParseResult::Comment("; comment".to_owned()))
        );
    }

    #[test]
    fn parse_label() {
        let lexer = Lexer::from_iter(":label".chars());
        let mut parser = Parser::from_iter(lexer);

        assert_eq!(parser.next(), Some(ParseResult::Label(":label".to_owned())));
    }

    mod statement_permutations {
        use super::*;

        #[test]
        fn parse_statement_label_mneu_op() {
            let lexer = Lexer::from_iter(":label SKE 0x1234".chars());
            let mut parser = Parser::from_iter(lexer);

            assert_eq!(
                parser.next(),
                Some(ParseResult::Statement(Statement {
                    label: Some(":label".to_owned()),
                    opcode: "SKE".to_owned(),
                    operands: vec![Token::Number("0x1234".to_owned())],
                    comment: None,
                }))
            );
        }

        #[test]
        fn parse_statement_mneu_op() {
            let lexer = Lexer::from_iter("SKE 0x1234".chars());
            let mut parser = Parser::from_iter(lexer);

            assert_eq!(
                parser.next(),
                Some(ParseResult::Statement(Statement {
                    label: None,
                    opcode: "SKE".to_owned(),
                    operands: vec![Token::Number("0x1234".to_owned())],
                    comment: None,
                }))
            );
        }

        #[test]
        fn parse_statement_mneu_op_comment() {
            let lexer = Lexer::from_iter("SKE 0x1234 ; comment".chars());
            let mut parser = Parser::from_iter(lexer);

            assert_eq!(
                parser.next(),
                Some(ParseResult::Statement(Statement {
                    label: None,
                    opcode: "SKE".to_owned(),
                    operands: vec![Token::Number("0x1234".to_owned())],
                    comment: Some("; comment".to_owned()),
                }))
            );
        }

        #[test]
        fn parse_statement_label_mneu_op_comment() {
            let lexer = Lexer::from_iter(":label SKE 0x1234 ; comment".chars());
            let mut parser = Parser::from_iter(lexer);

            assert_eq!(
                parser.next(),
                Some(ParseResult::Statement(Statement {
                    label: Some(":label".to_owned()),
                    opcode: "SKE".to_owned(),
                    operands: vec![Token::Number("0x1234".to_owned())],
                    comment: Some("; comment".to_owned()),
                }))
            );
        }

        #[test]
        fn parse_statement_mneu_op_op() {
            let lexer = Lexer::from_iter("SKE 0x1234 0x5678".chars());
            let mut parser = Parser::from_iter(lexer);

            assert_eq!(
                parser.next(),
                Some(ParseResult::Statement(Statement {
                    label: None,
                    opcode: "SKE".to_owned(),
                    operands: vec![
                        Token::Number("0x1234".to_owned()),
                        Token::Number("0x5678".to_owned()),
                    ],
                    comment: None,
                }))
            );
        }

        #[test]
        fn parse_statement_mneu_op_op_comment() {
            let lexer = Lexer::from_iter("SKE 0x1234 0x5678 ; comment".chars());
            let mut parser = Parser::from_iter(lexer);

            assert_eq!(
                parser.next(),
                Some(ParseResult::Statement(Statement {
                    label: None,
                    opcode: "SKE".to_owned(),
                    operands: vec![
                        Token::Number("0x1234".to_owned()),
                        Token::Number("0x5678".to_owned()),
                    ],
                    comment: Some("; comment".to_owned()),
                }))
            );
        }

        #[test]
        fn parse_statement_mneu_comment() {
            let lexer = Lexer::from_iter("SKE ; comment".chars());
            let mut parser = Parser::from_iter(lexer);

            assert_eq!(
                parser.next(),
                Some(ParseResult::Statement(Statement {
                    label: None,
                    opcode: "SKE".to_owned(),
                    operands: Vec::new(),
                    comment: Some("; comment".to_owned()),
                }))
            );
        }

        #[test]
        fn parse_statement_mneu() {
            let lexer = Lexer::from_iter("SKE".chars());
            let mut parser = Parser::from_iter(lexer);

            assert_eq!(
                parser.next(),
                Some(ParseResult::Statement(Statement {
                    label: None,
                    opcode: "SKE".to_owned(),
                    operands: Vec::new(),
                    comment: None,
                }))
            );
        }
    }

    mod invalid_statements {
        use super::*;

        #[test]
        fn parse_number_by_itself() {
            let lexer = Lexer::from_iter("0x1234".chars());
            let mut parser = Parser::from_iter(lexer);

            assert_eq!(
                parser.next(),
                Some(ParseResult::Unknown(vec![Token::Number(
                    "0x1234".to_owned()
                )]))
            );
        }

        #[test]
        fn parse_number_then_comment() {
            let lexer = Lexer::from_iter("0x1234 ; comment".chars());
            let mut parser = Parser::from_iter(lexer);

            assert_eq!(
                parser.next(),
                Some(ParseResult::Unknown(vec![Token::Number(
                    "0x1234".to_owned()
                )]))
            );

            assert_eq!(
                parser.next(),
                Some(ParseResult::Comment("; comment".to_owned()))
            );
        }
    }

    mod example_programs {
        use super::*;

        #[test]
        fn example_program() {
            let lexer = Lexer::from_iter(
                "
                :start SKE 0x1234
                SKE 0x5678
                ; comment
                :label
                SKE 0x1234 ; comment
            "
                .chars(),
            );
            let mut parser = Parser::from_iter(lexer);

            assert_eq!(
                parser.next(),
                Some(ParseResult::Statement(Statement {
                    label: Some(":start".to_owned()),
                    opcode: "SKE".to_owned(),
                    operands: vec![Token::Number("0x1234".to_owned())],
                    comment: None,
                }))
            );

            assert_eq!(
                parser.next(),
                Some(ParseResult::Statement(Statement {
                    label: None,
                    opcode: "SKE".to_owned(),
                    operands: vec![Token::Number("0x5678".to_owned())],
                    comment: None,
                }))
            );

            assert_eq!(
                parser.next(),
                Some(ParseResult::Comment("; comment".to_owned()))
            );

            assert_eq!(parser.next(), Some(ParseResult::Label(":label".to_owned())));

            assert_eq!(
                parser.next(),
                Some(ParseResult::Statement(Statement {
                    label: None,
                    opcode: "SKE".to_owned(),
                    operands: vec![Token::Number("0x1234".to_owned())],
                    comment: Some("; comment".to_owned())
                }))
            );
        }
    }
}
