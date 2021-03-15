use std::cmp::{Eq, Ordering};
use std::fs::{self, File};
use std::io::{self, Read};
use std::os::unix::{ffi::OsStrExt, fs::PermissionsExt};
use std::path::{Path, PathBuf};

use custom_debug_derive::Debug;
use sha2::{Digest, Sha256};

use crate::utils;
use crate::{InkError, DATA_EXT};
use libflate::deflate::Encoder;
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
    pub fn new(filepath: &Path, ink_root: &Path) -> Result<FileData, InkError> {
        let content = Content::new(filepath, ink_root)?;
        let permissions = fs::metadata(filepath)?.permissions().mode();

        // make filepath relative to project directory
        // find the absolute path of the project directory
        let project_dir = ink_root.parent().ok_or("ink root dir is invalid.")?;

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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Content {
    #[debug(with = "utils::hex_fmt")]
    hash: [u8; 32],
}

impl Content {
    /// Create a Content struct from a tracked file,
    /// and add it to the data directory.
    /// Only created by FileData
    fn new(filepath: &Path, ink_root: &Path) -> Result<Content, InkError> {
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

        drop(file);

        // get the hash of the file
        let hash = hasher.finalize();

        // add it to the data directory.
        let content_file_path = ink_root.join(DATA_EXT).join(hex::encode(hash));

        // yeah, we read the file twice sometimes. But we don't
        // always write the file to content_file_path, and
        // reading the file only once means we'd have to write & compress
        // every time, then throw it away if we don't need it.
        // TODO: Is there a better way of doing this? inc. safety stuff with two reads.
        if !content_file_path.exists() {
            let file_writer = File::create(content_file_path)?;
            let mut writer = Encoder::new(file_writer);
            let mut reader = File::open(filepath)?;
            io::copy(&mut reader, &mut writer)?;
            writer.finish().into_result()?;
        }

        Ok(Content { hash: hash.into() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libflate::deflate::Decoder;
    use std::convert::TryInto;
    use std::io::Write;

    #[test]
    fn new_content_test() {
        let tmpdir = tempfile::tempdir_in("./test_tmp_files").unwrap();
        let tmpdir_path = tmpdir.path();
        let ex_file_path = tmpdir_path.join("example");

        crate::init(tmpdir_path).unwrap();
        File::create(&ex_file_path)
            .unwrap()
            .write_all(b"this is a test!")
            .unwrap();

        let content = Content::new(&ex_file_path, &tmpdir_path.join(".ink")).unwrap();

        assert_eq!(
            content,
            Content {
                hash: hex::decode(
                    "ca7f87917e4f5029f81ec74d6711f1c587dca0fe91ec82b87bb77aeb15e6566d"
                )
                .unwrap()
                .try_into()
                .unwrap()
            }
        );

        let subdir_content_path: PathBuf = [
            ".ink",
            "data",
            "ca7f87917e4f5029f81ec74d6711f1c587dca0fe91ec82b87bb77aeb15e6566d",
        ]
        .iter()
        .collect();

        let content_path = tmpdir_path.join(subdir_content_path);

        assert!(content_path.exists());

        let mut decoder = Decoder::new(File::open(content_path).unwrap());
        let mut decoded_data = Vec::new();
        decoder.read_to_end(&mut decoded_data).unwrap();

        assert_eq!(decoded_data, b"this is a test!");
    }

    #[test]
    fn new_filedata_test() {
        let tmpdir = tempfile::tempdir_in("./test_tmp_files").unwrap();
        let tmpdir_path = tmpdir.path();
        let ex_file_path = tmpdir_path.join("example");

        crate::init(tmpdir_path).unwrap();
        File::create(&ex_file_path)
            .unwrap()
            .write_all(b"this is a test!")
            .unwrap();

        let filedata = FileData::new(&ex_file_path, &tmpdir_path.join(".ink")).unwrap();

        assert_eq!(
            filedata,
            FileData {
                hash: hex::decode(
                    "d2cf54bef59f1921aeae4fab95594a57924bc8b39ba96e4e32a881fefb949fb9"
                )
                .unwrap()
                .try_into()
                .unwrap(),
                path: Path::new(".").join(ex_file_path),
                permissions: 33188,
                content: Content {
                    hash: hex::decode(
                        "ca7f87917e4f5029f81ec74d6711f1c587dca0fe91ec82b87bb77aeb15e6566d"
                    )
                    .unwrap()
                    .try_into()
                    .unwrap()
                }
            }
        );
    }
}
