use std::path::Path;
use std::fmt::{Formatter, Display};
use std::fmt;
use std::error::Error;
use crate::script_parser::{Rule};

#[derive(Debug)]
pub enum ParseError<'a> {
    BadSource(&'a Path),
    NoProgram(&'a Path),
    RuleMismatch{expected: Rule, found: Rule},
    UnknownAtom(&'a str),
    Redeclared {symbol: String, original: &'a Path, conflict: &'a Path},
    Redefined {symbol: String, original: &'a Path, conflict: &'a Path},
    UndeclaredSymbol(&'a str),
    UndefinedSymbol(&'a str),
}

impl<'a> Display for ParseError<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::BadSource(p) =>
                write!(f, "Error accessing: {:?}", p),
            ParseError::NoProgram(p) =>
                write!(f, "An incorrect progrm in the file: {:?}.", p),
            ParseError::RuleMismatch { expected, found } =>
                write!(f, "Expected: {:?} and found: {:?}", expected, found),
            ParseError::UnknownAtom(s) =>
                write!(f, "Unknown atom: {:?}", s),
            ParseError::Redeclared { symbol, original, conflict } =>
                write!(
                    f,
                    "'{}' has more than one declaration at {:?} and {:?}",
                    symbol, original, conflict
                ),
            ParseError::Redefined { symbol, original, conflict } =>
                write!(
                    f,
                    "'{}' has more than one defined at {:?} and {:?}",
                    symbol, original, conflict
                ),
            ParseError::UndeclaredSymbol(s) =>
                write!(f, "Symbol {} is not declared", s),
            ParseError::UndefinedSymbol(s) =>
                write!(f, "Symbol {} is declared but not defined", s),
        }
    }
}

impl<'a> Error for ParseError<'a> {}

#[derive(Debug)]
pub enum WalkerError {
    UnknownScene(String),
}

impl Display for WalkerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            WalkerError::UnknownScene(s) =>
                write!(f, "An unknown scene: {:?}", s),
        }
    }
}

impl Error for WalkerError {}
