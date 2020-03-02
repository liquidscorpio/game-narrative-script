use pest::iterators::Pairs;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

use crate::{Rule};
use crate::compiler::error::ParseError;

pub(crate) mod error;

#[derive(Debug)]
enum SymbolType {
    Character,
    Act,
}

impl SymbolType {
    pub(crate) fn from_str(token: &str) -> Result<SymbolType, ParseError> {
        match token {
            ":character" => Ok(SymbolType::Character),
            ":act" => Ok(SymbolType::Act),
            _ => Err(ParseError::UnknownAtom(token))
        }
    }
}

type SymbolAttributes = HashMap<String, String>;

#[derive(Debug)]
struct SymbolInfo<'a> {
    /// Path where the symbol is defined
    source: &'a Path,
    /// The line number is the source path where the symbol is defined
    line: i32,
    /// Type information
    symbol_type: SymbolType,
    /// Optional extra data associated with the symbol
    attributes: Option<SymbolAttributes>,
}

impl<'a> SymbolInfo<'a> {
    pub(crate) fn new(
        path: &'a Path, sym_type: SymbolType, attrs: Option<SymbolAttributes>
    ) -> Self {
        SymbolInfo {
            source: path,
            line: 0,
            symbol_type: sym_type,
            attributes: attrs
        }
    }
}

#[derive(Debug)]
pub(crate) struct Compiler<'a> {
    symbols: BTreeMap<String, SymbolInfo<'a>>,
    errors: Vec<ParseError<'a>>,
    unknown_symbols: HashSet<String>
}

impl<'a> Compiler<'a> {
    pub(crate) fn new() -> Self {
        Compiler {
            symbols: BTreeMap::new(),
            errors: vec![],
            unknown_symbols: HashSet::new(),
        }
    }

    fn has_symbol(&self, symbol: &str) -> bool {
        match self.symbols.get(symbol) {
            Some(_) => true,
            None => false
        }
    }

    fn record_symbol(&mut self, symbol: String, info: SymbolInfo<'a>) {
        self.unknown_symbols.remove(&symbol);
        self.symbols.insert(symbol, info);
    }

    fn record_conflict(&mut self, symbol: String, path: &'a Path) {
        let previous = self.symbols.get(&symbol).unwrap();
        self.errors.push(ParseError::Redeclared {
            symbol,
            original: previous.source,
            conflict: path,
        })
    }

    fn parse_symbol_type(&mut self, atom: &'a str) -> Option<SymbolType> {
        match SymbolType::from_str(atom) {
            Ok(t) => Some(t),
            Err(e) => {
                self.errors.push(e);
                None
            }
        }
    }

    /// This one generates symbol table for the file at the given path.
    pub(crate) fn compile(
        &mut self, pairs: Pairs<'a, Rule>, path: &'a Path
    ) {
        for pair in pairs {
            // We ignore all other rules
            match pair.as_rule() {
                Rule::dec_expr => {
                    self.dec_expr(pair.into_inner(), path);
                },
                Rule::def_expr => {
                    self.def_expr(pair.into_inner(), path);
                },
                _ => (),
            };
        }
    }
    fn dec_expr(&mut self, mut inner: Pairs<'a, Rule>, path: &'a Path) {
        let atom = inner.next().unwrap().as_str();
        let sym_type = match self.parse_symbol_type(atom) {
            Some(s) => s,
            None => return
        };

        let symbol = inner.next().unwrap().as_str().to_string();
        let attrs = match inner.next() {
            Some(a) => Some(self.object_attrs(a.into_inner())),
            None => None
        };

        let info = SymbolInfo::new(path, sym_type, attrs);
        match self.has_symbol(&symbol) {
            true => self.record_conflict(symbol, path),
            false => self.record_symbol(symbol, info),
        };
    }

    fn object_attrs(&mut self, mut pairs: Pairs<Rule>) -> SymbolAttributes {
        let mut attributes: HashMap<String, String> = HashMap::new();
        let attr_pairs = pairs.next().unwrap().into_inner();
        for kv_pair in attr_pairs {
            let mut key_value = kv_pair.into_inner();
            let key = key_value.next().unwrap().as_str().to_string();
            let val = key_value.next().unwrap().as_str().to_string();
            attributes.insert(key, val);
        }

        attributes
    }

    fn def_expr(&mut self, mut inner: Pairs<'a, Rule>, path: &'a Path) {
        let ident = inner.next().unwrap().as_str();
        if !self.has_symbol(ident) {
            self.unknown_symbols.insert(ident.to_string());
        }

        let defn = inner.next().unwrap();
        match defn.as_rule() {
            Rule::dialogue_def => {
                self.dialogue_def(defn.into_inner());
            },
            _ => ()
        };
    }

    fn dialogue_def(&mut self, pairs: Pairs<Rule>) {
        for pair in pairs {
            match pair.as_rule() {
                Rule::dialogue_expr => {
                    self.dialogue_expr(pair.into_inner());
                }
                Rule::choice_expr => {
                    self.choice_expr(pair.into_inner());
                }
                _ => unreachable!()
            }
        }
    }

    fn dialogue_expr(&mut self, pairs: Pairs<Rule>) {
        println!("{:#?}", pairs);
    }

    fn choice_expr(&mut self, pairs: Pairs<Rule>) {

    }
}
