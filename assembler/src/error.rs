use thiserror::Error;

use crate::parser::Statement;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Chip8AssemblerError {
    #[error("Invalid statement: {0:?} {1}")]
    InvalidStatementError(Statement, String),
    #[error("Unable to resolve label: {0}")]
    UnknownLabelError(String),
}
