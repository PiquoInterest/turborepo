use serde_json::{Map, Number, Value};
use thiserror::Error;

const MAX_NESTING_DEPTH: usize = 128;
pub(crate) const MAX_JSON5_BYTES: usize = 4 * 1_024 * 1_024;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum Json5Error {
    #[error("JSON5 input exceeds the {MAX_JSON5_BYTES}-byte limit")]
    InputTooLarge,
    #[error("unexpected end of input at byte {0}")]
    UnexpectedEnd(usize),
    #[error("expected an object key at byte {0}")]
    ExpectedKey(usize),
    #[error("expected ':' after object key at byte {0}")]
    ExpectedColon(usize),
    #[error("expected ',' or a closing delimiter at byte {0}")]
    ExpectedCommaOrEnd(usize),
    #[error("unexpected value at byte {0}")]
    UnexpectedValue(usize),
    #[error("invalid escape sequence at byte {0}")]
    InvalidEscape(usize),
    #[error("invalid unicode escape at byte {0}")]
    InvalidUnicodeEscape(usize),
    #[error("unterminated string at byte {0}")]
    UnterminatedString(usize),
    #[error("unterminated block comment at byte {0}")]
    UnterminatedComment(usize),
    #[error("invalid number at byte {0}")]
    InvalidNumber(usize),
    #[error("non-finite numbers are not valid Turborepo configuration at byte {0}")]
    NonFiniteNumber(usize),
    #[error("JSON5 nesting exceeds {MAX_NESTING_DEPTH} levels")]
    NestingTooDeep,
    #[error("unexpected trailing content at byte {0}")]
    TrailingContent(usize),
}

struct Parser<'a> {
    input: &'a str,
    bytes: &'a [u8],
    index: usize,
}

