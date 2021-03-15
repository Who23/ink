pub mod commit;
pub mod diff;
pub mod filedata;
pub mod graph;
mod utils;

use crate::commit::Commit;
use crate::graph::IDGraph;

use std::env;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use std::path::PathBuf;

const DATA_EXT: &str = "data";
const COMMIT_EXT: &str = "commit";

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

    let mut graph_file = File::create(&ink_dir.join("graph"))?;

    let graph = IDGraph::new();
    let encoded_graph = bincode::serialize(&graph)?;

    graph_file.write_all(&encoded_graph)?;

    Ok(())
}

pub fn commit() -> Result<(), InkError> {
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
    let commit = Commit::new(paths, &root_dir)?;
    commit.write(&root_dir)?;

    let graph_path = root_dir.join("graph");
    let mut graph: IDGraph = bincode::deserialize(&fs::read(&graph_path)?)?;

    graph.add_node(commit.hash())?;

    // no branching yet so there will only be one head
    let head = graph.heads()[0];
    graph.add_edge(head, commit.hash())?;

    fs::write(&graph_path, bincode::serialize(&graph)?)?;

    Ok(())
}

#[derive(Debug)]
pub enum InkError {
    Err(&'static str),
    IO(io::Error),
    Serialization(bincode::ErrorKind),
}

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
