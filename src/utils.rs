use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::InkError;

/// Find all the file paths in a directory
pub fn find_paths(dir: &Path, v: &mut Vec<PathBuf>) -> io::Result<()> {
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
fn _copy_subdirs(source: &Path, target: &Path) -> Result<(), InkError> {
    if target.is_dir() {
        return Err("The target directory already exists".into());
    }

    let mut paths = Vec::new();
    find_paths(source, &mut paths)?;

    for source_path in paths {
        let source_path = source_path.strip_prefix(source).unwrap();
        let path = target.join(source_path);

        fs::create_dir_all(path.parent().unwrap())?;
    }
    Ok(())
}
