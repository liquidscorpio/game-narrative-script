use std::path::{PathBuf};
use crate::compiler::walker::NarrativeWalker;

#[macro_use]
extern crate log;

extern crate pest;
#[macro_use]
extern crate pest_derive;

extern crate serde_json;

pub mod compiler;
pub mod script_parser;

#[no_mangle]
pub fn get_narrative_walker(path_buf: PathBuf) -> NarrativeWalker {
    NarrativeWalker::new(path_buf).unwrap()
}
