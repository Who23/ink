pub mod diff;
pub mod graph;

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

/// Find all the file paths in a directory
fn _find_paths(dir: &Path, v: &mut Vec<PathBuf>) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                _find_paths(&path, v)?;
            } else {
                Vec::push(v, path);
            }
        }
    }
    Ok(())
}

/// Creates a new directory at target and copies all subdirectories from source
fn _copy_subdirs(source: &Path, target: &Path) -> Result<(), InkError> {
    if target.is_dir() {
        return Err("The target directory already exists".into());
    }

    let mut paths = Vec::new();
    _find_paths(source, &mut paths)?;

    for source_path in paths {
        let source_path = source_path.strip_prefix(source).unwrap();
        let path = target.join(source_path);

        fs::create_dir_all(path.parent().unwrap())?;
    }
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
