use gdnative::*;
use pest::iterators::Pairs;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

use crate::script_parser::Rule;
use crate::compiler::error::ParseError;
use std::fs::File;
use std::io::Write;
use std::error::Error;
use serde::{Deserialize, Serialize};

pub mod error;
pub mod walker;

#[derive(Debug)]
enum SymbolType {
    Character,
    Act,
}

impl SymbolType {
    pub fn from_str(token: &str) -> Result<SymbolType, ParseError> {
        match token {
            ":character" => Ok(SymbolType::Character),
            ":act" => Ok(SymbolType::Act),
            _ => Err(ParseError::UnknownAtom(token))
        }
    }
}

type FileIndex<'a> = HashMap<&'a str, (usize, usize)>;
type OwnedFileIndex = HashMap<String, (usize, usize)>;

#[derive(ToVariant, FromVariant, Debug, Serialize, Deserialize, Clone)]
pub struct SymbolAttributes {
    key: String,
    value: String,
}

impl SymbolAttributes {
    pub fn new(key: String, value: String) -> Self {
        Self {
            key,
            value,
        }
    }
}

#[derive(Debug)]
struct SymbolInfo<'a> {
    /// Path where the symbol is defined
    source: &'a Path,
    /// The start byte is the source path where the symbol is defined
    start_position: usize,
    /// Type information
    symbol_type: SymbolType,
    /// Optional extra data associated with the symbol
    attributes: Vec<SymbolAttributes>,
}

impl<'a> SymbolInfo<'a> {
    pub fn new(
        path: &'a Path, sym_type: SymbolType, start_position: usize,
        attrs: Vec<SymbolAttributes>,
    ) -> Self {
        SymbolInfo {
            source: path,
            start_position,
            symbol_type: sym_type,
            attributes: attrs,
        }
    }
}

#[derive(ToVariant, FromVariant, Debug, Serialize, Deserialize)]
pub enum NarrativeItem {
    Dialogue {
        character: String,
        display_name: String,
        dialogue: String,
        attributes: Vec<SymbolAttributes>,
    },
    ChoiceSet {
        character: String,
        display_name: String,
        choices: Vec<NarrativeChoice>,
        attributes: Vec<SymbolAttributes>,
    },
}

#[derive(ToVariant, FromVariant, Debug, Serialize, Deserialize)]
pub struct NarrativeChoice {
    text: String,
    jump: String,
}


#[derive(Debug)]
pub struct Compiler<'a> {
    symbols: BTreeMap<String, SymbolInfo<'a>>,
    errors: Vec<ParseError<'a>>,
    unknown_symbols: HashSet<String>,
    definition: BTreeMap<String, Vec<NarrativeItem>>,
    checks_passed: bool,
}

impl<'a> Compiler<'a> {
    pub fn new() -> Self {
        Compiler {
            symbols: BTreeMap::new(),
            errors: vec![],
            unknown_symbols: HashSet::new(),
            definition: BTreeMap::new(),
            checks_passed: false,
        }
    }

    fn has_symbol(&self, symbol: &str) -> bool {
        match self.symbols.get(symbol) {
            Some(_) => true,
            None => false
        }
    }

    fn is_defined(&self, symbol: &str) -> bool {
        match self.definition.get(symbol) {
            Some(_) => true,
            None => false
        }
    }

    fn record_symbol(&mut self, symbol: String, info: SymbolInfo<'a>) {
        self.unknown_symbols.remove(&symbol);
        self.symbols.insert(symbol, info);
    }

    fn record_dec_conflict(&mut self, symbol: String, path: &'a Path) {
        let previous = self.symbols.get(&symbol).unwrap();
        self.errors.push(ParseError::Redeclared {
            symbol,
            original: previous.source,
            conflict: path,
        });
    }

    fn record_def_conflict(&mut self, symbol: String, path: &'a Path) {
        let previous = self.symbols.get(&symbol).unwrap();
        self.errors.push(ParseError::Redefined {
            symbol,
            original: previous.source,
            conflict: path,
        });
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
    pub fn compile(
        &mut self, pairs: Pairs<'a, Rule>, path: &'a Path,
    ) {
        for pair in pairs {
            // We ignore all other rules
            match pair.as_rule() {
                Rule::dec_expr => {
                    self.dec_expr(pair.into_inner(), path);
                }
                Rule::def_expr => {
                    self.def_expr(pair.into_inner(), path);
                }
                _ => (),
            };
        }
    }
    fn dec_expr(&mut self, mut inner: Pairs<'a, Rule>, path: &'a Path) {
        let atom = inner.next().unwrap();
        let sym_type = match self.parse_symbol_type(atom.as_str()) {
            Some(s) => s,
            None => return
        };

        let symbol = inner.next().unwrap().as_str().to_string();
        let attrs = match inner.next() {
            Some(a) => self.object_attrs(a.into_inner()),
            None => vec![],
        };

        let info = SymbolInfo::new(
            path, sym_type, atom.as_span().start(), attrs
        );

        match self.has_symbol(&symbol) {
            true => self.record_dec_conflict(symbol, path),
            false => self.record_symbol(symbol, info),
        };
    }

    fn object_attrs(&mut self, mut pairs: Pairs<Rule>) -> Vec<SymbolAttributes> {
        let mut attributes = vec![];
        let attr_pairs = pairs.next().unwrap().into_inner();
        for kv_pair in attr_pairs {
            let mut key_value = kv_pair.into_inner();
            let key = key_value.next().unwrap().as_str().to_string();
            let val = key_value.next().unwrap().into_inner().next()
                .unwrap().as_str().to_string();
            attributes.push(SymbolAttributes::new(key, val));
        }

        attributes
    }

    fn def_expr(&mut self, mut inner: Pairs<'a, Rule>, path: &'a Path) {
        let ident = inner.next().unwrap().as_str();
        if !self.has_symbol(ident) {
            self.unknown_symbols.insert(ident.to_string());
        }

        if self.is_defined(ident) {
            self.record_def_conflict(ident.to_string(), path);
            return;
        }

        let definition = inner.next().unwrap();
        match definition.as_rule() {
            Rule::dialogue_def => {
                let seq = self.dialogue_def(definition.into_inner());
                self.definition.insert(ident.to_string(), seq);
            }
            _ => ()
        };
    }

    fn dialogue_def(&mut self, pairs: Pairs<Rule>) -> Vec<NarrativeItem> {
        pairs.map(|pair| {
            match pair.as_rule() {
                Rule::dialogue_expr => self.dialogue_expr(pair.into_inner()),
                Rule::choice_expr => self.choice_expr(pair.into_inner()),
                _ => unreachable!()
            }
        }).collect()
    }

    fn dialogue_expr(&mut self, mut pairs: Pairs<Rule>) -> NarrativeItem {
        let character = pairs.next().unwrap().as_str()[1..].to_string();
        if !self.has_symbol(&character) {
            self.unknown_symbols.insert(character.clone());
        }
        let dialogue = pairs.next().unwrap().into_inner().next().unwrap().as_str().to_string();
        // We do not always have attributes when parsing acts as characters
        // may be defined else where. Therefore, population of attributes is
        // done when generating tree files.
        NarrativeItem::Dialogue { character, display_name: "".to_string(), dialogue, attributes: vec![] }
    }

    fn choice_expr(&mut self, mut pairs: Pairs<Rule>) -> NarrativeItem {
        let character = pairs.next().unwrap().as_str()[1..].to_string();
        if !self.has_symbol(&character) {
            self.unknown_symbols.insert(character.clone());
        }

        let choices: Vec<NarrativeChoice> = pairs.map(|pair| {
            let mut tokens = pair.into_inner();
            let text = tokens.next().unwrap().into_inner().next()
                .unwrap().as_str().to_string();
            let jump = tokens.next().unwrap().as_str();

            if !self.has_symbol(jump) {
                self.unknown_symbols.insert(jump.to_string());
            }

            NarrativeChoice {
                text,
                jump: jump.to_string(),
            }
        }).collect();

        NarrativeItem::ChoiceSet { character, display_name: "".to_string(), choices, attributes: vec![] }
    }

    pub fn are_symbols_defined(&self) -> bool {
        match self.unknown_symbols.is_empty() {
            true => true,
            false => {
                for symbol in self.unknown_symbols.iter() {
                    error!("{}", ParseError::UndeclaredSymbol(symbol));
                }
                false
            }
        }
    }

    pub fn is_error_free(&self) -> bool {
        match self.errors.is_empty() {
            true => true,
            false => {
                for error in self.errors.iter() {
                    error!("{}", error);
                }
                false
            }
        }
    }

    pub fn all_acts_defined(&self) -> bool {
        let mut flag = true;
        self.symbols.iter().for_each(|(s, info)| {
            if let SymbolType::Act = info.symbol_type {
                if !self.definition.contains_key(s) {
                    error!("{}", ParseError::UndefinedSymbol(s));
                    flag = false;
                }
            }
        });
        flag
    }

    pub fn run_checks(&mut self) -> bool {
        let success: [bool; 3] = [
            self.are_symbols_defined(),
            self.is_error_free(),
            self.all_acts_defined(),
        ];
        self.checks_passed = success.iter().all(|v| *v);
        self.checks_passed
    }

    // *** WARNING! HACKY STUFF AHEAD ***
    // TODO: Come up with better design for this hacky approach
    fn update_narrative_items(&mut self) {

        #[inline]
        fn patch(
            character: &mut String, display_name: &mut String,
            attributes: &mut Vec<SymbolAttributes>, symbols: &BTreeMap<String, SymbolInfo>
        ) {
            let sym_info = symbols.get(character).unwrap();
            for obj in &sym_info.attributes {
                if obj.key == "name" {
                    *display_name = obj.value.clone();
                }
            }
            *attributes = sym_info.attributes.to_vec();
        }

        for (_, narrative) in &mut self.definition {
            for ntv in narrative {
                match ntv {
                    NarrativeItem::Dialogue { character, display_name, dialogue: _, attributes } => {
                        patch(character, display_name, attributes, &self.symbols);
                    }
                    NarrativeItem::ChoiceSet { character, display_name, choices: _, attributes } => {
                        patch(character, display_name, attributes, &self.symbols);
                    }
                }
            }
        }
    }

    pub fn generate_data_files(&mut self) {
        if !self.checks_passed {
            error!("Please run 'run_checks' before generating files");
            return;
        }

        // We update narrative-items with metadata after ensuring all
        // characters and acts are defined.
        self.update_narrative_items();

        match self.generate_tree_file() {
            Ok(index) => {
                match self.generate_index_file(&index) {
                    Ok(_) => (),
                    Err(e) => {
                        error!("Error generating index file: {:?}", e);
                    }
                }
            },
            Err(e) => {
                error!("Error generating data file: {:?}", e);
            }
        };
    }

    fn generate_tree_file(&self) -> Result<FileIndex, Box<dyn Error>> {
        let mut fp = File::create("source.gcstree")?;
        let mut start_byte = 0;
        let mut end_byte = 0;
        let mut index: FileIndex = HashMap::new();
        for (act, narrative) in &self.definition {
            let data= serde_json::to_string(narrative)?;
            let mut encoder = snap::raw::Encoder::new();
            let compressed = encoder.compress_vec(data.as_bytes())?;
            let bytes_written = fp.write(compressed.as_ref())?;
            end_byte += bytes_written;
            index.insert(act, (start_byte, end_byte));
            start_byte += bytes_written;
        }
        fp.flush()?;
        Ok(index)
    }

    fn generate_index_file(
        &self, index: &FileIndex
    ) -> Result<(), Box<dyn Error>> {
        let mut fp = File::create("source.gcsindex")?;
        let data = serde_json::to_string(index)?;
        fp.write(data.as_bytes())?;
        fp.flush()?;
        Ok(())
    }
}
