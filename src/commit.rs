use std::error::Error;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

use crate::filedata::FileData;
use crate::utils;
use crate::{COMMIT_EXT, ROOT_DIR};

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
    pub fn new<P: AsRef<Path>>(files: Vec<P>) -> Result<Commit, Box<dyn Error>> {
        // get FileData objects for each file
        let mut files = files
            .iter()
            .map(|filepath| FileData::new(filepath.as_ref()))
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

        let commit = Commit {
            hash: hash.into(),
            files,
            time: now,
        };

        let commit_file_path = (*ROOT_DIR)
            .as_ref()
            .ok_or("Ink Uninitialized")?
            .join(COMMIT_EXT)
            .join(hex::encode(hash));

        fs::write(commit_file_path, commit.to_string())?;

        Ok(commit)
    }

    pub fn to_string(&self) -> String {
        format!(
            "{}\n{}\n{}",
            hex::encode(self.hash),
            self.files
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<String>>()
                .join("\n"),
            self.time
        )
    }
}
