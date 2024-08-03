use std::collections::VecDeque;

use crate::constants::mneumonics::{MAX_OPCODE_LEN, SORTED_MNEUMONICS};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Number(String),
    Mneumonic(String),
    Label(String),
    Comment(String),
    Whitespace(char),
    Unknown(char),
}

pub struct Lexer<TIterator>
where
    TIterator: Iterator<Item = char>,
{
    tokens: TIterator,
    pending_lex: VecDeque<char>,
}

impl<TIterator> Lexer<TIterator>
where
    TIterator: Iterator<Item = char>,
{
    pub fn from_iter(tokens: TIterator) -> Self {
        return Lexer {
            tokens,
            pending_lex: VecDeque::new(),
        };
    }

    fn next_char(&mut self) -> Option<char> {
        if self.pending_lex.is_empty() {
            return self.tokens.next();
        }
        return self.pending_lex.pop_front();
    }

    fn lex_number(&mut self, c: char) -> Token {
        let mut buffer = String::from(c);
        loop {
            if let Some(c) = self.next_char() {
                match c {
                    '0'..='9' | 'a'..='f' | 'A'..='F' | 'x' | 'X' => {
                        buffer.push(c);
                    }
                    _ => {
                        self.pending_lex.push_front(c);
                        break;
                    }
                }
            } else {
                break;
            }
        }
        return Token::Number(buffer);
    }

    fn lex_opcode(&mut self, c: char) -> Option<Token> {
        let mut buffer = String::with_capacity(5);
        buffer.push(c);
        for _ in 1..MAX_OPCODE_LEN {
            // Start at 1 because we already read the first char
            if let Some(c) = self.next_char() {
                match c {
                    'a'..='z' | 'A'..='Z' => {
                        buffer.push(c);
                    }
                    _ => {
                        self.pending_lex.push_front(c);
                        break;
                    }
                }
            } else {
                break;
            }
        }

        let upper_case_buffer = buffer.to_ascii_uppercase();

        for mneumonic in SORTED_MNEUMONICS {
            if upper_case_buffer.starts_with(&mneumonic) {
                let leftover_chars = buffer.len() - mneumonic.len();
                for i in 0..leftover_chars {
                    self.pending_lex.push_front(buffer.chars().nth(i).unwrap());
                }
                return Some(Token::Mneumonic(mneumonic.to_string()));
            }
        }

        for i in (1..buffer.len()).rev() {
            // Start at 1 because we already read the first char, it will be present in the caller
            self.pending_lex.push_front(buffer.chars().nth(i).unwrap());
        }

        return None;
    }

    fn lex_label(&mut self, c: char) -> Token {
        let mut buffer = String::new();
        buffer.push(c);
        loop {
            if let Some(c) = self.next_char() {
                match c {
                    'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                        buffer.push(c);
                    }
                    _ => {
                        self.pending_lex.push_front(c);
                        break;
                    }
                }
            } else {
                break;
            }
        }

        return Token::Label(buffer);
    }

    fn lex_comment(&mut self, c: char) -> Token {
        let mut buffer = String::new();
        buffer.push(c);
        loop {
            if let Some(c) = self.next_char() {
                match c {
                    '\n' => {
                        self.pending_lex.push_front(c);
                        break;
                    }
                    _ => {
                        buffer.push(c);
                    }
                }
            } else {
                break;
            }
        }
        Token::Comment(buffer)
    }

    fn lex_char(&mut self, c: char) -> Token {
        match c {
            ' ' | '\t' | '\r' | '\n' => Token::Whitespace(c),
            'a'..='z' | 'A'..='Z' => {
                if let Some(token) = self.lex_opcode(c) {
                    return token;
                }

                match c {
                    'a'..='f' | 'A'..='F' => {
                        return self.lex_number(c);
                    }
                    _ => {
                        return Token::Unknown(c);
                    }
                };
            }
            '0'..='9' => self.lex_number(c),
            ':' => self.lex_label(c),
            ';' => self.lex_comment(c),
            _ => Token::Unknown(c),
        }
    }
}

impl<TIterator> Iterator for Lexer<TIterator>
where
    TIterator: Iterator<Item = char>,
{
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        let Some(c) = self.next_char() else {
            return None;
        };

        return Some(self.lex_char(c));
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn hex_number() {
        let mut lexer = Lexer::from_iter("0x1234".chars());
        assert_eq!(lexer.next(), Some(Token::Number("0x1234".to_owned())));

        let mut lexer = Lexer::from_iter("1234".chars());
        assert_eq!(lexer.next(), Some(Token::Number("1234".to_owned())));

        let mut lexer = Lexer::from_iter("AF".chars());
        assert_eq!(lexer.next(), Some(Token::Number("AF".to_owned())));

        let mut lexer = Lexer::from_iter("af".chars());
        assert_eq!(lexer.next(), Some(Token::Number("af".to_owned())));
    }

    #[test]
    fn mneumonic() {
        let mut lexer = Lexer::from_iter("SKE".chars());
        assert_eq!(lexer.next(), Some(Token::Mneumonic("SKE".to_owned())));
        assert_eq!(lexer.pending_lex.len(), 0);

        let mut lexer = Lexer::from_iter("LOADS".chars());
        assert_eq!(lexer.next(), Some(Token::Mneumonic("LOADS".to_owned())));
        assert_eq!(lexer.pending_lex.len(), 0);

        let mut lexer = Lexer::from_iter("LOAD".chars());
        assert_eq!(lexer.next(), Some(Token::Mneumonic("LOAD".to_owned())));
        assert_eq!(lexer.pending_lex.len(), 0);

        let mut lexer = Lexer::from_iter("LOAD ".chars());
        assert_eq!(lexer.next(), Some(Token::Mneumonic("LOAD".to_owned())));
        assert_eq!(lexer.pending_lex.len(), 1);

        let mut lexer = Lexer::from_iter("FAKE".chars());
        assert_eq!(lexer.next(), Some(Token::Number("FA".to_owned())));
        assert_eq!(lexer.next(), Some(Token::Unknown('K')));
        assert_eq!(lexer.next(), Some(Token::Number("E".to_owned())));
    }

    #[test]
    fn lex_label() {
        let mut lexer = Lexer::from_iter(":label".chars());
        assert_eq!(lexer.next(), Some(Token::Label(":label".to_owned())));
    }

    #[test]
    fn lex_comment() {
        let mut lexer = Lexer::from_iter("; comment".chars());
        assert_eq!(lexer.next(), Some(Token::Comment("; comment".to_owned())));

        let mut lexer = Lexer::from_iter(";comment".chars());
        assert_eq!(lexer.next(), Some(Token::Comment(";comment".to_owned())));

        let mut lexer = Lexer::from_iter("; com\nment".chars());
        assert_eq!(lexer.next(), Some(Token::Comment("; com".to_owned())));
        assert_eq!(lexer.next(), Some(Token::Whitespace('\n')));
        assert_eq!(lexer.pending_lex.len(), 0);
    }

    #[test]
    fn lex_op_statement() {
        let mut lexer = Lexer::from_iter("SKE 0x1234 ; comment".chars());
        assert_eq!(lexer.next(), Some(Token::Mneumonic("SKE".to_owned())));
        assert_eq!(lexer.next(), Some(Token::Whitespace(' ')));
        assert_eq!(lexer.next(), Some(Token::Number("0x1234".to_owned())));
        assert_eq!(lexer.next(), Some(Token::Whitespace(' ')));
        assert_eq!(lexer.next(), Some(Token::Comment("; comment".to_owned())));
    }

    #[test]
    fn lex_full_statement() {
        let mut lexer = Lexer::from_iter(":start SKE 0x1234 ; comment".chars());
        assert_eq!(lexer.next(), Some(Token::Label(":start".to_owned())));
        assert_eq!(lexer.next(), Some(Token::Whitespace(' ')));
        assert_eq!(lexer.next(), Some(Token::Mneumonic("SKE".to_owned())));
        assert_eq!(lexer.next(), Some(Token::Whitespace(' ')));
        assert_eq!(lexer.next(), Some(Token::Number("0x1234".to_owned())));
        assert_eq!(lexer.next(), Some(Token::Whitespace(' ')));
        assert_eq!(lexer.next(), Some(Token::Comment("; comment".to_owned())));
    }

    #[test]
    fn lex_lines_comment_label_statement() {
        let mut lexer = Lexer::from_iter("; comment\n:label\nSKE 0x1234 ; comment".chars());
        assert_eq!(lexer.next(), Some(Token::Comment("; comment".to_owned())));
        assert_eq!(lexer.next(), Some(Token::Whitespace('\n')));
        assert_eq!(lexer.next(), Some(Token::Label(":label".to_owned())));
        assert_eq!(lexer.next(), Some(Token::Whitespace('\n')));
        assert_eq!(lexer.next(), Some(Token::Mneumonic("SKE".to_owned())));
        assert_eq!(lexer.next(), Some(Token::Whitespace(' ')));
        assert_eq!(lexer.next(), Some(Token::Number("0x1234".to_owned())));
        assert_eq!(lexer.next(), Some(Token::Whitespace(' ')));
        assert_eq!(lexer.next(), Some(Token::Comment("; comment".to_owned())));
    }
}
