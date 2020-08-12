use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write,
              Error, ErrorKind,
              self};
use std::path::{Path, PathBuf};
use std::boxed::Box;
use std::env;

use ink::diff::Diff;
use ink::InkError;
use ink::log::Log;


fn main() {
}

fn init() -> Result<(), InkError> {
    // create ./.ink dir
    let ink_dir = env::current_dir()?.join(".ink");
    fs::create_dir(&ink_dir)?;

    Log::new(&ink_dir.join("log"))?;

    Ok(())
}

/// Find the location of the .ink directory
/// For now, this returns the current directory + .ink but 
/// will check in higher directories
fn get_ink_dir() -> Result<PathBuf, InkError> {
    let ink_dir = env::current_dir()?.join(".ink");
    if ink_dir.exists() {
        Ok(ink_dir)
    } else {
        Err(InkError::Uninitialized)
    }
}

/// Find all the file paths in a directory
fn find_paths(dir: &Path, v: &mut Vec<PathBuf>) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                find_paths(&path, v)?;
            } else {
                Vec::push(v, path);
            }
        }
    }
    Ok(())
}

/// Creates a new directory at target and copies all subdirectories from source
fn copy_subdirs(source: &Path, target: &Path) -> io::Result<()> {
    if target.is_dir()  { return Err(Error::new(ErrorKind::Other, "The target directory already exists")) }

    let mut paths = Vec::new(); 
    find_paths(source, &mut paths)?;

    for source_path in paths {
        let source_path = source_path.strip_prefix(source).unwrap();
        let path = target.join(source_path);

        fs::create_dir_all(path.parent().unwrap())?;
    }
    Ok(())
}

