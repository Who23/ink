pub mod commit;
mod cursor;
pub mod diff;
pub mod filedata;
pub mod graph;
mod utils;

use crate::commit::Commit;
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
    cursor::init(&ink_dir)?;
    CommitGraph::init(&ink_dir)?;

    Ok(())
}

pub fn commit() -> Result<Commit, InkError> {
    let mut paths = Vec::new();
    let root_dir = root_dir()?.ok_or("Ink Uninitialized")?;
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

    let commit = Commit::new(paths, SystemTime::now(), &root_dir)?;
    commit.write(&root_dir)?;

    let mut graph = CommitGraph::get(&root_dir)?;

    let current_commit = cursor::get(&root_dir)?;
    graph.add_commit(&current_commit, &commit)?;

    cursor::set(&root_dir, &commit)?;
    graph.write()?;

    Ok(commit)
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
