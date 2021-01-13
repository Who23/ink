pub mod diff;
pub mod graph;
pub mod filedata;
pub mod commit;
mod utils;

use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::io;
use std::path::PathBuf;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    pub static ref ROOT_DIR: Option<PathBuf> = {
        let ink_dir = env::current_dir().unwrap().join(".ink");

        if ink_dir.exists() && ink_dir.is_dir() {
            Some(ink_dir)
        } else {
            None
        }
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
