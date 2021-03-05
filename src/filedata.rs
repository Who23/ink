use std::cmp::{Eq, Ordering};
use std::fs::{self, File};
use std::io::Read;
use std::os::unix::{ffi::OsStrExt, fs::PermissionsExt};
use std::path::{Path, PathBuf};

use custom_debug_derive::Debug;
use hex;
use sha2::{Digest, Sha256};

use crate::utils;
use crate::{InkError, DATA_EXT, ROOT_DIR};
use serde::{Deserialize, Serialize};

/// A struct holding the file data nessecary
/// to commit changes. Includes unix file permissions,
/// as such it only works on unix systems.
#[derive(Debug, Serialize, Deserialize)]
pub struct FileData {
    #[debug(with = "utils::hex_fmt")]
    hash: [u8; 32],
    path: PathBuf,
    // rust sets/gets unix file perms as a u32
    permissions: u32,
    content: Content,
}

impl FileData {
    /// Creates a FileData struct given a filepath.
    /// Can fail on IO errors.
    pub fn new(filepath: &Path) -> Result<FileData, InkError> {
        let content = Content::new(filepath)?;
        let permissions = fs::metadata(filepath)?.permissions().mode();

        // make filepath relative to project directory
        // find the absolute path of the project directory
        let project_dir = ROOT_DIR
            .as_ref()
            .ok_or("Ink Uninitialized")?
            .parent()
            .ok_or("ROOT_DIR is invalid.")?;

        // root the filepath to the project dir.
        let absolute_filepath = filepath.canonicalize()?;
        let rooted_filepath = absolute_filepath
            .strip_prefix(project_dir)
            .map_err(|_| "Could not root filepaths relative to project dir")?;

        let mut hasher = Sha256::new();
        hasher.update(rooted_filepath.as_os_str().as_bytes());
        hasher.update(permissions.to_be_bytes());
        hasher.update(content.hash);
        let hash = hasher.finalize();

        Ok(FileData {
            hash: hash.into(),
            path: rooted_filepath.to_path_buf(),
            permissions,
            content,
        })
    }

    /// Write content to disk.
    pub fn write_content(&self) -> Result<(), InkError> {
        self.content.write(&self.path)?;
        Ok(())
    }

    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }
}

impl Ord for FileData {
    fn cmp(&self, other: &Self) -> Ordering {
        self.hash.cmp(&other.hash)
    }
}

impl PartialOrd for FileData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for FileData {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Eq for FileData {}

#[derive(Debug, Serialize, Deserialize)]
pub struct Content {
    #[debug(with = "utils::hex_fmt")]
    hash: [u8; 32],
}

impl Content {
    /// Create a Content struct from a tracked file,
    /// and add it to the data directory.
    /// Only created by FileData
    fn new(filepath: &Path) -> Result<Content, InkError> {
        let mut file = File::open(filepath)?;
        let mut hasher = Sha256::new();

        // create buffer for holding chunks of file
        const BUF_SIZE: usize = 1024 * 128;
        let mut buffer = [0; BUF_SIZE];

        // read chunks of the file and update the hash.
        loop {
            let bytes_read = file.read(&mut buffer)?;
            hasher.update(&buffer[..bytes_read]);

            if bytes_read < BUF_SIZE {
                break;
            }
        }

        // get the hash of the file
        let hash = hasher.finalize();

        Ok(Content { hash: hash.into() })
    }

    /// Should only be called by FileData
    fn write(&self, filepath: &Path) -> Result<(), InkError> {
        // add it to the data directory.
        let content_file_path = (*ROOT_DIR)
            .as_ref()
            .ok_or("Ink Uninitialized")?
            .join(DATA_EXT)
            .join(hex::encode(self.hash));

        if !content_file_path.exists() {
            fs::copy(filepath, content_file_path)?;
        }

        Ok(())
    }
}
