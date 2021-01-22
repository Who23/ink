use std::error::Error;
use std::fmt;
use std::io;
use std::path::Path;
use std::time::SystemTime;

use crate::filedata::FileData;
use crate::utils;

use custom_debug_derive::Debug;
use sha2::{Digest, Sha256};

/// Struct to hold information about a commit
/// to work with in ink. Stores filedata and time
/// of commit
#[derive(Debug)]
pub struct Commit {
    #[debug(with = "utils::hex_fmt")]
    hash: [u8; 32],
    files: Vec<FileData>,
    time: u64,
}

impl Commit {
    /// Creates a new commit from data in the given directory.
    pub fn new(dirpath: &Path) -> Result<Commit, Box<dyn Error>> {
        // list files
        let mut files = Vec::new();
        utils::find_paths(dirpath, &mut files)?;

        // get FileData objects for each file
        let mut files = files
            .iter()
            .map(|filepath| FileData::new(&filepath))
            .collect::<Result<Vec<FileData>, Box<dyn Error>>>()?;

        files.sort();

        // get SystemTime, convert to seconds.
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Cannot commit before unix epoch.")
            .as_secs();

        let mut hasher = Sha256::new();

        for file in &files {
            hasher.update(file.hash);
        }

        hasher.update(now.to_be_bytes());

        let hash = hasher.finalize();

        Ok(Commit {
            hash: hash.into(),
            files,
            time: now,
        })
    }
}
