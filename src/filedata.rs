use std::error::Error;
use std::fs::{self, File};
use std::io::{self, Read};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use hex;

use crate::{DATA_EXT, ROOT_DIR};

/// A struct holding the file data nessecary
/// to commit changes. Includes unix file permissions,
/// as such it only works on unix systems.
pub struct FileData {
    hash: [u8; 32],
    path: PathBuf,
    // rust sets/gets unix file perms as a u32
    permissions: u32,
}

impl FileData {
    /// Creates a FileData struct given a filepath. 
    /// Can fail on IO errors.
    pub fn new(filepath: &Path) -> io::Result<FileData> {
        let mut file = File::open(filepath)?;
        let mut hasher = Sha256::new();

        // create buffer for holding chunks of file
        const BUF_SIZE : usize = 1024 * 128;
        let mut buffer = [0; BUF_SIZE];

        // read chunks of the file and update the hash.
        loop {
            let bytes_read = file.read(&mut buffer)?;
            hasher.update(&buffer[..bytes_read]);

            if bytes_read < BUF_SIZE {
                break;
            }
        }
        
        // get the hash and unix permissions of the file.
        let hash = hasher.finalize();
        let permissions = file.metadata()?.permissions().mode(); 

        Ok(FileData {
            hash: hash.into(),
            path: filepath.to_path_buf(),
            permissions
        })
    }

    /// Copy the contents of the file this FileData struct
    /// refers to into a file in the given directory.
    /// The name of the file will be it's SHA256 hash.
    pub fn write_content(&self) -> Result<(), Box<dyn Error>> {
        let filepath = (*ROOT_DIR)
            .as_ref()
            .ok_or("Ink Uninitialized")?
            .join(DATA_EXT)
            .join(hex::encode(&self.hash));

        fs::copy(&self.path, filepath)?;

        Ok(())
    }
}
