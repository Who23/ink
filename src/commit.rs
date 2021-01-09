use std::error::Error;
use std::io;
use std::path::Path;
use std::time::SystemTime;

use crate::filedata::FileData;
use crate::utils;


/// Struct to hold information about a commit
/// to work with in ink. Stores filedata and time
/// of commit
pub struct Commit {
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
        let files = files
            .iter()
            .map(|filepath| FileData::new(&filepath))
            .collect::<io::Result<Vec<FileData>>>()?;

        // get SystemTime, convert to seconds.
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Cannot commit before unix epoch.")
            .as_secs();

        Ok(Commit { files, time: now })
    }
}
