use std::path::Path;
use std::fmt::{Formatter, Display};
use std::fmt;
use std::error::Error;
use crate::{Rule};

#[derive(Debug)]
pub enum ParseError<'a> {
    BadSource(&'a Path),
    NoProgram(&'a Path),
    RuleMismatch{expected: Rule, found: Rule},
    UnknownAtom(&'a str),
    Redeclared {symbol: String, original: &'a Path, conflict: &'a Path}
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
        }
    }
}

impl<'a> Error for ParseError<'a> {}
