pub mod commit;
mod cursor;
pub mod diff;
pub mod filedata;
pub mod graph;
mod utils;

use crate::commit::{Commit, Edit};
use crate::graph::CommitGraph;

use std::env;
use std::error::Error;
use std::fmt::Display;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

const DATA_EXT: &str = "data";
const COMMIT_EXT: &str = "commit";
const GRAPH_FILE: &str = "graph";
const CURSOR_FILE: &str = "cursor";

fn root_dir() -> Result<Option<PathBuf>, InkError> {
    let curr_dir = env::current_dir()?.canonicalize()?;

    for path in curr_dir.ancestors() {
        let ink_dir = path.join(".ink");
        if ink_dir.exists() && ink_dir.is_dir() {
            return Ok(Some(ink_dir));
        }
    }

    Ok(None)
}

// functions called by cli
pub fn init(in_dir: &Path) -> Result<(), InkError> {
    // create ./.ink dir
    let ink_dir = in_dir.join(".ink");

    fs::create_dir(&ink_dir)?;
    fs::create_dir(&ink_dir.join(COMMIT_EXT))?;
    fs::create_dir(&ink_dir.join(DATA_EXT))?;
    let empty_commit = Commit::new::<PathBuf>(vec![], SystemTime::now(), &ink_dir)?;
    println!("{:?}", empty_commit);
    empty_commit.write(&ink_dir)?;
    cursor::init(&ink_dir)?;
    cursor::set(&ink_dir, &empty_commit)?;
    CommitGraph::init(&ink_dir, &empty_commit)?;

    Ok(())
}

fn create_commit_from_wd(root_dir: &Path) -> Result<Commit, InkError> {
    let mut paths = Vec::new();
    utils::find_paths(
        root_dir
            .parent()
            .ok_or("Could not find project directory")?,
        &mut paths,
    )?;
    paths = paths
        .into_iter()
        .filter(|p| !p.starts_with(&root_dir))
        .collect();

    Commit::new(paths, SystemTime::now(), &root_dir)
}

pub fn commit() -> Result<Commit, InkError> {
    let root_dir = root_dir()?.ok_or("Ink Uninitialized")?;
    let commit = create_commit_from_wd(&root_dir)?;
    commit.write(&root_dir)?;

    let mut graph = CommitGraph::get(&root_dir)?;

    let current_commit = cursor::get(&root_dir)?;
    graph.add_commit(&current_commit, &commit)?;

    cursor::set(&root_dir, &commit)?;
    graph.write()?;

    Ok(commit)
}

pub fn go(to: Commit) -> Result<(), InkError> {
    let root_dir = root_dir()?.ok_or("Ink Uninitialized")?;
    let from = cursor::get(&root_dir)?;

    // perform check to see if pwd is dirty
    if !(create_commit_from_wd(&root_dir)?.diff(&from).edits).is_empty() {
        return Err(
            "The working directory is dirty, please commit all changes before proceeding".into(),
        );
    }

    // diff current commit and target commit
    let diff = from.diff(&to);
    // apply diff by removing removed files, applying diffs to changed files, and add new files
    for edit in diff.edits {
        match edit {
            Edit::Insert(f) => f.write_to(&root_dir, f.path()),
            Edit::Delete(f) => fs::remove_file(f.path()).map_err(|e| e.into()),
            Edit::Modify { original, modified } => {
                fs::remove_file(original.path())?;
                modified.write_to(&root_dir, modified.path())
            }
        }?;
    }

    // set cursor to new commit
    cursor::set(&root_dir, &to)
}

#[derive(Debug)]
pub enum InkError {
    Err(&'static str),
    IO(io::Error),
    Serialization(bincode::ErrorKind),
}

impl Error for InkError {}

impl From<io::Error> for InkError {
    fn from(err: io::Error) -> InkError {
        InkError::IO(err)
    }
}

impl From<&'static str> for InkError {
    fn from(err: &'static str) -> InkError {
        InkError::Err(err)
    }
}

impl From<Box<bincode::ErrorKind>> for InkError {
    fn from(err: Box<bincode::ErrorKind>) -> InkError {
        InkError::Serialization(*err)
    }
}

impl Display for InkError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self {
            InkError::Err(e) => write!(f, "{}", e),
            InkError::IO(e) => write!(f, "{}", e),
            InkError::Serialization(e) => write!(f, "{}", e),
        }
    }
}
