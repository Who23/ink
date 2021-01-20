use std::error::Error;
use std::fs::{self, File};
use std::io::{self, Read};
use std::os::unix::{fs::PermissionsExt, ffi::OsStrExt};
use std::path::{Path, PathBuf};
use std::cmp::{Ordering, Eq};
use std::fmt;

use hex;
use sha2::{Digest, Sha256};
use custom_debug_derive::Debug;

use crate::{DATA_EXT, ROOT_DIR};
use crate::utils;

/// A struct holding the file data nessecary
/// to commit changes. Includes unix file permissions,
/// as such it only works on unix systems.
#[derive(Debug)]
pub struct FileData {
    #[debug(with = "utils::hex_fmt")]
    pub hash: [u8; 32],
    path: PathBuf,
    // rust sets/gets unix file perms as a u32
    permissions: u32,
    content: Content
}

impl FileData {
    /// Creates a FileData struct given a filepath.
    /// Can fail on IO errors.
    pub fn new(filepath: &Path) -> Result<FileData, Box<dyn Error>> {
        let content = Content::new(filepath)?;
        let permissions = fs::metadata(filepath)?.permissions().mode();

        let mut hasher = Sha256::new();
        hasher.update(filepath.as_os_str().as_bytes());
        hasher.update(permissions.to_be_bytes());
        hasher.update(content.hash);
        let hash = hasher.finalize();

        Ok(FileData {
            hash: hash.into(),
            path: filepath.to_path_buf(),
            permissions,
            content,
        })
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

#[derive(Debug)]
pub struct Content {
    #[debug(with = "utils::hex_fmt")]
    hash: [u8; 32]
}

impl Content {
    /// Create a Content struct from a tracked file,
    /// and add it to the data directory.
    pub fn new(filepath: &Path) -> Result<Content, Box<dyn Error>> {
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

        // add it to the data directory.
        let content_file_path = (*ROOT_DIR)
            .as_ref()
            .ok_or("Ink Uninitialized")?
            .join(DATA_EXT)
            .join(hex::encode(hash));

        if !content_file_path.exists() {
            fs::copy(filepath, content_file_path)?;
        }

        Ok(Content {
            hash: hash.into()
        })
    }
}
