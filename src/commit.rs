use std::fs::{self, File};
use std::path::Path;
use std::time::SystemTime;

use crate::filedata::FileData;
use crate::utils;
use crate::{InkError, COMMIT_EXT, ROOT_DIR};

use custom_debug_derive::Debug;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Struct to hold information about a commit
/// to work with in ink. Stores filedata and time
/// of commit
#[derive(Debug, Serialize, Deserialize)]
pub struct Commit {
    #[debug(with = "utils::hex_fmt")]
    #[serde(skip)]
    hash: [u8; 32],
    files: Vec<FileData>,
    time: u64,
}

impl Commit {
    /// Creates a new commit from data in the given directory.
    pub fn new<P: AsRef<Path>>(files: Vec<P>) -> Result<Commit, InkError> {
        // get FileData objects for each file
        let mut files = files
            .iter()
            .map(|filepath| FileData::new(filepath.as_ref()))
            .collect::<Result<Vec<FileData>, InkError>>()?;

        files.sort();

        // get SystemTime, convert to seconds.
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|_| "Cannot commit before unix epoch.")?
            .as_secs();

        let mut hasher = Sha256::new();

        for file in &files {
            hasher.update(file.hash());
        }

        hasher.update(now.to_be_bytes());

        let hash = hasher.finalize();

        let commit = Commit {
            hash: hash.into(),
            files,
            time: now,
        };

        Ok(commit)
    }

    /// Write commit data to disk in .ink
    /// This should be called when the state of the repo is in
    /// this commit - otherwise, the wrong files will be written to disk.
    pub fn write(&self) -> Result<(), InkError> {
        let commit_file_path = (*ROOT_DIR)
            .as_ref()
            .ok_or("Ink Uninitialized")?
            .join(COMMIT_EXT)
            .join(hex::encode(self.hash));

        fs::write(commit_file_path, bincode::serialize(&self)?)?;

        Ok(())
    }

    /// Deserialize a commit object from its hash.
    pub fn from(hash: &[u8; 32]) -> Result<Commit, InkError> {
        let commit_file_path = (*ROOT_DIR)
            .as_ref()
            .ok_or("Ink Uninitialized")?
            .join(COMMIT_EXT)
            .join(hex::encode(hash));

        if !(commit_file_path.exists() && commit_file_path.is_file()) {
            return Err("Given commit hash does not exist on disk".into());
        }

        let reader = File::open(commit_file_path)?;
        let mut commit: Commit = bincode::deserialize_from(reader)?;
        commit.hash = *hash;

        Ok(commit)
    }

    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }
}
