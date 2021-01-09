pub mod diff;
pub mod graph;
pub mod filedata;
mod utils;

use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

fn _init() -> Result<(), InkError> {
    // create ./.ink dir
    let ink_dir = env::current_dir()?.join(".ink");
    fs::create_dir(&ink_dir)?;

    File::create(&ink_dir.join("graph"))?;

    Ok(())
}

/// Find the location of the .ink directory
/// For now, this returns the current directory + .ink but
/// will check in higher directories
fn _get_ink_dir() -> Result<PathBuf, InkError> {
    let ink_dir = env::current_dir()?.join(".ink");
    if ink_dir.exists() {
        Ok(ink_dir)
    } else {
        Err(InkError::Err("Uninitialized"))
    }
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
