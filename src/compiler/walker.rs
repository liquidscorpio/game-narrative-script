use std::path::PathBuf;
use std::fs::File;
use crate::compiler::{OwnedFileIndex, NarrativeItem};
use std::error::Error;
use std::io::{Read, Seek, SeekFrom};
use std::str;
use crate::compiler::error::{WalkerError};

#[repr(C)]
#[no_mangle]
#[derive(Debug)]
pub struct NarrativeWalker {
    _source_path: PathBuf,
    source_handle: File,
    index: OwnedFileIndex,
}

impl NarrativeWalker {
    pub fn new(path: PathBuf) -> Result<Self, Box<dyn Error>> {
        let source_fp = File::open(&path)?;
        let mut index_path = path.to_path_buf();
        index_path.set_extension("gcsindex");
        let mut buf = String::new();
        let mut index_fp = File::open(index_path)?;
        index_fp.read_to_string(&mut buf)?;
        let index: OwnedFileIndex = serde_json::from_str(buf.as_str())?;
        Ok(NarrativeWalker {
            _source_path: path,
            source_handle: source_fp,
            index,
        })
    }

    pub fn traverse(
        &mut self, start_scene: &str
    ) -> Result<Vec<NarrativeItem>, Box<dyn Error>> {
        match self.index.get(start_scene) {
            Some((start, end)) => {
                let size = end - start;
                let mut buf = vec![0u8; size];
                self.source_handle.seek(SeekFrom::Start(*start as u64))?;
                self.source_handle.read_exact(&mut buf)?;
                let mut decoder = snap::raw::Decoder::new();
                let data = decoder.decompress_vec(buf.as_ref())?;
                let json_str = str::from_utf8(data.as_ref())?;
                let items: Vec<NarrativeItem> = serde_json::from_str(json_str)?;
                Ok(items)
            }
            None => {
                let err = WalkerError::UnknownScene(start_scene.to_string());
                Err(Box::new(err))
            }
        }
    }
}
