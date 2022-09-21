use super::id_graph::IDGraph;
use crate::commit::Commit;
use crate::{InkError, GRAPH_FILE};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct CommitGraph {
    graph_path: PathBuf,
    graph: IDGraph,
}

impl CommitGraph {
    pub fn init(ink_dir: &Path) -> Result<(), InkError> {
        let graph_path = &ink_dir.join(GRAPH_FILE);

        let graph = IDGraph::new();
        fs::write(&graph_path, bincode::serialize(&graph)?)?;

        Ok(())
    }

    pub fn get(ink_dir: &Path) -> Result<CommitGraph, InkError> {
        let graph_path = ink_dir.join(GRAPH_FILE);
        let graph: IDGraph = bincode::deserialize(&fs::read(&graph_path)?)?;
        Ok(CommitGraph { graph_path, graph })
    }

    pub fn add_commit(&mut self, from: &Commit, to: &Commit) -> Result<(), InkError> {
        self.graph.add_node(from.hash())?;
        self.graph.add_edge(from.hash(), to.hash())?;
        Ok(())
    }

    pub fn write(self) -> Result<(), InkError> {
        fs::write(&self.graph_path, bincode::serialize(&self.graph)?)?;
        Ok(())
    }
}
