use pest::Parser;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::borrow::BorrowMut;
use crate::compiler::{Compiler};
use std::error::Error;
use crate::compiler::error::ParseError;

#[macro_use]
extern crate log;

extern crate pest;
#[macro_use]
extern crate pest_derive;

extern crate serde_json;

mod compiler;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct ScriptParser;


pub fn read_source(path: &Path) -> Result<String, ParseError> {
    match File::open(path) {
        Ok(mut fp) => {
            let mut buf = String::new();
            match fp.read_to_string(buf.borrow_mut()) {
                Ok(_) => (),
                Err(e) => {
                    error!("{:?}", e);
                    return Err(ParseError::BadSource(path))
                },
            };
            Ok(buf)
        },
        Err(e) => {
            error!("{:?}", e);
            Err(ParseError::BadSource(path))
        }
    }
}

pub fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let path = Path::new("/home/tintin/Studio/game-narrative-script/src/source.gcs");
    let program = read_source(path)?;
    let mut parse_result = ScriptParser::parse(Rule::program, &program)?;
    match parse_result.next() {
        Some(pair) => {
            match pair.as_rule() {
                Rule::program => {
                    let inner = pair.into_inner();
                    // To avoid name conflict
                    let mut compiler_instance = Compiler::new();
                    compiler_instance.compile(inner, path);
                    compiler_instance.run_checks();
                    compiler_instance.generate_data_files();
                }
                _ => return Err(Box::new(ParseError::NoProgram(path))),
            }
        },
        None => ()
    }

    Ok(())
}
