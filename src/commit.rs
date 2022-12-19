use std::collections::HashMap;
use std::fs::{self, File};
use std::path::Path;
use std::time::SystemTime;

use crate::filedata::FileData;
use crate::graph::CommitGraph;
use crate::utils;
use crate::{InkError, COMMIT_EXT};

use custom_debug_derive::Debug;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Struct to hold information about a commit
/// to work with in ink. Stores filedata and time
/// of commit
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Commit {
    #[debug(with = "utils::hex_fmt")]
    #[serde(skip)]
    hash: [u8; 32],
    // TODO: store these as a hash set with custom hash trait for ink id hashes
    files: Vec<FileData>,
    time: u64,
}

/// Serialized representation of a commit
#[derive(Serialize, Deserialize, Default)]
struct CommitRepr {
    files: Vec<FileData>,
    time: u64,
}

impl CommitRepr {
    fn to_commit(mut self) -> Commit {
        // TODO: pull the hashing into a trait for all ink objects
        self.files.sort();

        let mut hasher = Sha256::new();

        for file in &self.files {
            hasher.update(file.hash());
        }

        hasher.update(self.time.to_be_bytes());

        let hash = hasher.finalize().into();

        Commit {
            hash,
            files: self.files,
            time: self.time,
        }
    }
}

pub fn commit_hash_from_prefix(ink_root: &Path, prefix: &[u8]) -> Result<[u8; 32], InkError> {
    if prefix.len() > 32 {
        return Err("invalid commit hash prefix: too long".into());
    }

    let graph = CommitGraph::get(&ink_root)?;
    let all_hashes: Vec<&[u8; 32]> = graph.commit_hashes();

    let candidates: Vec<&&[u8; 32]> = all_hashes
        .iter()
        .filter(|h| (**h).starts_with(prefix))
        .collect();

    if candidates.is_empty() {
        return Err("No commits in the graph match the given prefix".into());
    }

    if candidates.len() > 1 {
        return Err("Too many possible commits with the given prefix".into());
    }

    Ok((*candidates[0]).clone())
}

impl Commit {
    /// Creates and writes a new commit from data in the given directory with the
    /// given timestamp
    pub(crate) fn new<P: AsRef<Path>>(
        files: Vec<P>,
        timestamp: SystemTime,
        ink_root: &Path,
    ) -> Result<Commit, InkError> {
        // get FileData objects for each file
        let mut files = files
            .iter()
            .map(|filepath| FileData::new(filepath.as_ref(), ink_root))
            .collect::<Result<Vec<FileData>, InkError>>()?;

        // get SystemTime, convert to seconds.
        let now = timestamp
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|_| "Cannot commit before unix epoch.")?
            .as_secs();

        files.sort();

        let mut hasher = Sha256::new();

        for file in &files {
            hasher.update(file.hash());
        }

        hasher.update(now.to_be_bytes());

        let hash = hasher.finalize().into();

        Ok(Commit {
            hash,
            files,
            time: now,
        })
    }

    pub(crate) fn write(&self, ink_root: &Path) -> Result<(), InkError> {
        for file in &self.files {
            file.write(ink_root)?;
        }

        let commit_file_path = ink_root.join(COMMIT_EXT).join(hex::encode(self.hash));

        fs::write(commit_file_path, bincode::serialize(&self)?)?;

        Ok(())
    }

    /// Deserialize a commit object from its hash.
    /// Throws an error if the given hash does not match the actual hash of the commit
    /// or if the given commit does not exist.
    pub fn from(hash: &[u8; 32], ink_root: &Path) -> Result<Commit, InkError> {
        let commit_file_path = ink_root.join(COMMIT_EXT).join(hex::encode(hash));

        if !(commit_file_path.exists() && commit_file_path.is_file()) {
            return Err("Given commit hash does not exist on disk".into());
        }

        let reader = File::open(commit_file_path)?;
        let commit: CommitRepr = bincode::deserialize_from(reader)?;
        let commit = commit.to_commit();

        if *hash != commit.hash {
            return Err("Actual hash of commit does not match given hash of commit".into());
        }

        Ok(commit)
    }

    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }

    /// Creates the diff to transform self -> other
    pub fn diff(&self, other: &Commit) -> CommitDiff {
        let mut edits = vec![];

        let self_hashes = self
            .files
            .iter()
            .map(|f| (f.path(), f))
            .collect::<HashMap<&Path, &FileData>>();

        let other_hashes = other
            .files
            .iter()
            .map(|f| (f.path(), f))
            .collect::<HashMap<&Path, &FileData>>();

        // TODO: this should not be checking for the combination of
        // index and path, just path -> get the relevant file
        for (path, file) in &other_hashes {
            if !self_hashes.contains_key(path) {
                edits.push(Edit::Insert((*file).clone()));
            } else {
                let original = self_hashes.get(path).unwrap();
                if file.hash() != original.hash() {
                    edits.push(Edit::Modify {
                        original: (*original).clone(),
                        modified: (*file).clone(),
                    });
                }
            }
        }

        for (path, file) in self_hashes {
            if !other_hashes.contains_key(path) {
                edits.push(Edit::Delete(file.clone()));
            }
        }

        CommitDiff { edits }
    }
}

#[derive(Debug)]
pub struct CommitDiff {
    pub edits: Vec<Edit>,
}

#[derive(Debug)]
pub enum Edit {
    Insert(FileData),
    Delete(FileData),
    Modify {
        original: FileData,
        modified: FileData,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filedata::tests::get_filedata;
    use std::convert::TryInto;
    use std::fmt::Debug;
    use std::io::Write;
    use std::path::PathBuf;
    use std::time::Duration;

    #[derive(Debug)]
    struct CommitInfo {
        tmpdir: tempfile::TempDir,
        paths: Vec<PathBuf>,
        time: SystemTime,
    }

    // timestamp: seconds after unix epoch
    fn env_setup(timestamp: u64) -> CommitInfo {
        let tmpdir = tempfile::tempdir_in("./test_tmp_files").unwrap();
        let tmpdir_path = tmpdir.path();

        crate::init(tmpdir_path).unwrap();

        let ex_file_path = tmpdir_path.join("example");
        File::create(&ex_file_path)
            .unwrap()
            .write_all(b"this is a test!")
            .unwrap();

        let ex_file_path_2 = tmpdir_path.join("example2");
        File::create(&ex_file_path_2)
            .unwrap()
            .write_all(b"this is a test! again")
            .unwrap();

        let time = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(timestamp))
            .unwrap();

        CommitInfo {
            tmpdir,
            paths: vec![ex_file_path, ex_file_path_2],
            time,
        }
    }

    #[test]
    fn new_commit() {
        let info = env_setup(1379995200);

        let commit = Commit::new(info.paths, info.time, &info.tmpdir.path().join(".ink"));
        let commit = commit.unwrap();
        assert_eq!(
            commit,
            Commit {
                hash: hex::decode(
                    "b27b7b5bdd38f0d8c35734bd54f941e41674e1f516c9e0ec5092800565686626"
                )
                .unwrap()
                .try_into()
                .unwrap(),
                files: vec![
                    get_filedata(
                        "778e3e48cbd97193fce773a4be3a1adf528c38340ed90d71993135db104c06dd",
                        "example2",
                        33188,
                        "cbdcf3dccd3ba4012e846ab734b3c5e28b3064314e58db85e2765ee3eb082396"
                    ),
                    get_filedata(
                        "d2cf54bef59f1921aeae4fab95594a57924bc8b39ba96e4e32a881fefb949fb9",
                        "example",
                        33188,
                        "ca7f87917e4f5029f81ec74d6711f1c587dca0fe91ec82b87bb77aeb15e6566d"
                    )
                ],
                time: 1379995200
            }
        );
    }

    #[test]
    fn write_commit() {
        let info = env_setup(1379995200);
        let ink_dir = info.tmpdir.path().join(".ink");

        let commit = Commit::new(info.paths, info.time, &ink_dir).unwrap();
        commit.write(&ink_dir).unwrap();

        let commit_path = ink_dir
            .join("commit")
            .join("b27b7b5bdd38f0d8c35734bd54f941e41674e1f516c9e0ec5092800565686626");

        assert!(commit_path.exists());

        let commit_repr: CommitRepr =
            bincode::deserialize_from(File::open(commit_path).unwrap()).unwrap();

        assert_eq!(commit, commit_repr.to_commit());
    }

    #[test]
    fn commit_from_hash() {
        let info = env_setup(1379995200);
        let ink_dir = info.tmpdir.path().join(".ink");

        let commit = Commit::new(info.paths, info.time, &ink_dir).unwrap();
        commit.write(&ink_dir).unwrap();
        let read_commit = Commit::from(
            &hex::decode("b27b7b5bdd38f0d8c35734bd54f941e41674e1f516c9e0ec5092800565686626")
                .unwrap()
                .try_into()
                .unwrap(),
            &ink_dir,
        )
        .unwrap();

        assert_eq!(commit, read_commit);
    }

    #[test]
    fn commit_from_nonexistant_hash() {
        let info = env_setup(1379995200);
        let ink_dir = info.tmpdir.path().join(".ink");

        let _commit = Commit::new(info.paths, info.time, &ink_dir).unwrap();
        let read_commit = Commit::from(
            &hex::decode("a27b7b5bdd38f0d8c35734bd54f941e41674e1f516c9e0ec5092800565686626")
                .unwrap()
                .try_into()
                .unwrap(),
            &ink_dir,
        );

        match read_commit.unwrap_err() {
            InkError::Err(s) => assert_eq!(s, "Given commit hash does not exist on disk"),
            _ => panic!("wrong kind of error"),
        };
    }

    #[test]
    fn commit_from_incorrect_hash() {
        let info = env_setup(1379995200);
        let ink_dir = info.tmpdir.path().join(".ink");

        let commit = Commit::new(info.paths, info.time, &ink_dir);
        let mut commit = commit.unwrap();

        let commit_file_path = ink_dir.join(COMMIT_EXT).join(hex::encode(commit.hash));
        commit.time = 1379995210;
        fs::write(commit_file_path, bincode::serialize(&commit).unwrap()).unwrap();

        let read_commit = Commit::from(
            &hex::decode("b27b7b5bdd38f0d8c35734bd54f941e41674e1f516c9e0ec5092800565686626")
                .unwrap()
                .try_into()
                .unwrap(),
            &ink_dir,
        );

        match read_commit.unwrap_err() {
            InkError::Err(s) => assert_eq!(
                s,
                "Actual hash of commit does not match given hash of commit"
            ),
            _ => panic!("wrong kind of error"),
        };
    }
}
