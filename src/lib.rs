pub mod commit;
pub mod diff;
pub mod filedata;
pub mod graph;
mod utils;

use std::env;
use std::fs::{self, File};
use std::io;
use std::path::PathBuf;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    pub static ref ROOT_DIR: Option<PathBuf> = {
        let curr_dir = env::current_dir().unwrap().canonicalize().unwrap();

        for path in curr_dir.ancestors() {
            let ink_dir = path.join(".ink");
            if ink_dir.exists() && ink_dir.is_dir() {
                return Some(ink_dir);
            }
        }

        None
    };
}

const DATA_EXT: &str = "data";
const COMMIT_EXT: &str = "commit";

fn _init() -> Result<(), InkError> {
    // create ./.ink dir
    let ink_dir = env::current_dir()?.join(".ink");
    fs::create_dir(&ink_dir)?;

    File::create(&ink_dir.join("graph"))?;

    Ok(())
}

#[derive(Debug)]
pub enum InkError {
    Err(&'static str),
    IO(io::Error),
}

impl From<io::Error> for InkError {
    fn from(err: io::Error) -> InkError {
        InkError::IO(err)
    }
}

impl From<&'static str> for InkError {
    fn from(err: &'static str) -> InkError {
        InkError::Err(err)
    }
}
