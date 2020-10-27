use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Seek, SeekFrom, Write};
use std::path::Path;

use crate::InkError;

/// An abstraction for working with the commit log of a .ink directory
///
/// Contains the current commit ID, a vec of commit IDs, and the handle
/// to the open file.
///
/// To change the log file, the contents of the struct should be changed.
///
/// Both the `current_commit` and `commits` should be the same variation of `Option`
/// - both `Some` or `None`
pub struct Log {
    // current commit
    pub current_commit: Option<usize>,

    // list of commits in order
    pub commits: Option<Vec<usize>>,

    // handle to the log file
    handle: File,
}

impl Log {
    /// Create an empty log file given a path
    /// Throws an error if that path already exists
    pub fn new(path: &Path) -> Result<Log, InkError> {
        if path.exists() {
            return Err(InkError::Err("Path already exists!"));
        }

        let handle = File::create(path)?;

        Ok(Log {
            current_commit: None,
            commits: None,
            handle,
        })
    }

    /// Serialize a `Log` struct given an existing log file
    /// On top of the normal io errors, `Log::serialize()` will throw an
    /// error if the log file is malformed
    pub fn serialize(path: &Path) -> Result<Log, InkError> {
        let mut handle = OpenOptions::new().write(true).read(true).open(path)?;

        let read_handle = BufReader::new(&handle);
        let mut log_lines = read_handle.lines();
        let current_commit = log_lines.next();
        let commit_vec: Vec<io::Result<String>> = log_lines.collect();

        handle.seek(SeekFrom::Start(0))?;

        if let Some(current) = &current_commit {
            if commit_vec.is_empty() {
                return Err(InkError::Malformed("Ink log file is malformed"));
            }

            // parse log file into usizes.
            let parsed_current: usize = current
                .as_ref()
                .unwrap()
                .parse()
                .map_err(|_| InkError::Malformed("Log file has invalid commit IDs!"))?;

            let commit_vec: Vec<usize> = commit_vec
                .iter()
                .map(|n| n.as_ref().unwrap().parse())
                .collect::<Result<Vec<usize>, _>>()
                .map_err(|_| InkError::Malformed("Log file has invalid commit IDs!"))?;

            Ok(Log {
                current_commit: Some(parsed_current),
                commits: Some(commit_vec),
                handle,
            })
        } else {
            if !commit_vec.is_empty() {
                return Err(InkError::Malformed("Ink log file is malformed"));
            }

            // none
            Ok(Log {
                current_commit: None,
                commits: None,
                handle,
            })
        }
    }

    /// Flush the current contents of the struct into the file, overwriting the old
    /// file completely.
    /// On top of the normal io errors, `Log::flush()` throws an error if the `current_commit`
    /// and `commits` field are not the same variation of `Option`
    /// This function is called when `Log` goes out of scope
    pub fn flush(&mut self) -> Result<(), InkError> {
        self.handle.seek(SeekFrom::Start(0))?;

        if let (Some(current), Some(commits)) = (&self.current_commit, &self.commits) {
            let commits: Vec<String> = commits.iter().map(|n| n.to_string()).collect();
            let commits = commits.join("\n");

            self.handle.set_len(0)?;
            writeln!(&mut self.handle, "{}", current)?;
            write!(&mut self.handle, "{}", commits)?;
        } else if let (None, None) = (&self.current_commit, &self.commits) {
            self.handle.set_len(0)?;
        } else {
            return Err(InkError::Malformed(
                "Only current or latest commit present in log struct",
            ));
        }

        Ok(())
    }
}

impl Drop for Log {
    fn drop(&mut self) {
        self.flush().unwrap();
    }
}
