use serde::{Deserialize, Serialize};
/// Implementation of a directed, cyclic graph for ink objects
/// Nodes in graph are IDs (SHA256 hashes)
use std::collections::HashMap;

type InkID = [u8; 32];

/// The Graph struct holds all the relevant information of the graph.
/// Contains a HashMap of IDs and their neighbors
/// This should be created with `Graph::new()` or `Default::default()`
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct IDGraph {
    nodes: HashMap<InkID, Neighbors>,
}

/// The neighbors for a given ID, parents and children.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Neighbors {
    parents: Vec<InkID>,
    children: Vec<InkID>,
}

impl IDGraph {
    /// Create a new graph
    pub fn new() -> Self {
        IDGraph {
            nodes: HashMap::new(),
        }
    }

    /// Add an ID to the graph
    pub fn add_node(&mut self, id: InkID) -> Result<(), &'static str> {
        if self.nodes.contains_key(&id) {
            return Err("ID is already in the graph");
        }

        self.nodes.insert(id, Default::default());
        Ok(())
    }

    /// Remove an ID. Fails if the ID is not found.
    pub fn remove_node(&mut self, id: InkID) -> Result<(), &'static str> {
        // get edge data for this node
        let (children, parents) = if let Some(node) = self.nodes.get_mut(&id) {
            Ok((node.children.clone(), node.parents.clone()))
        } else {
            Err("Invalid Node ID")
        }?;

        // remove all edges for this node
        for other_id in children.iter() {
            if let Some(node) = self.nodes.get_mut(other_id) {
                (*node).parents.retain(|elem| elem != &id)
            }
        }

        for other_id in parents.iter() {
            if let Some(node) = self.nodes.get_mut(other_id) {
                (*node).children.retain(|elem| elem != &id)
            }
        }

        // remove the node
        self.nodes.remove(&id);

        Ok(())
    }

    /// Add an edge between two IDs. Fails if the IDs are not found.
    /// Allows a node to create an edge with itself.
    pub fn add_edge(&mut self, from: InkID, to: InkID) -> Result<(), &'static str> {
        if !self.nodes.contains_key(&from) {
            return Err("Invalid Node ID for 'from' node");
        }
        if !self.nodes.contains_key(&to) {
            return Err("Invalid Node ID for 'to' node");
        }

        let from_children = &mut self.nodes.get_mut(&from).unwrap().children;

        if from_children.contains(&to) {
            return Err("'from' node already contains an edge to 'to' node");
        }

        from_children.push(to.clone());

        let to_parents = &mut self.nodes.get_mut(&to).unwrap().parents;

        if to_parents.contains(&from) {
            return Err("'to' node already contains an edge from 'from' node");
        }

        to_parents.push(from);

        Ok(())
    }

    /// Remove an edge between two IDs. Fails if the node IDs are not found.
    pub fn remove_edge(&mut self, from: InkID, to: InkID) -> Result<(), &'static str> {
        if !self.nodes.contains_key(&from) {
            return Err("Invalid ID for 'from' node");
        }
        if !self.nodes.contains_key(&to) {
            return Err("Invalid ID for 'to' node");
        }

        let from_children = &mut self.nodes.get_mut(&from).unwrap().children;

        if !from_children.contains(&to) {
            return Err("No edge exists between the 'from' node and the 'to' node");
        }

        from_children.retain(|elem| elem != &to);

        let to_parents = &mut self.nodes.get_mut(&to).unwrap().parents;

        if !to_parents.contains(&from) {
            return Err("No edge exists between the 'to' node and the 'from' node");
        }

        to_parents.retain(|elem| elem != &from);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const FIRST_ID: [u8; 32] = [
        47, 62, 4, 48, 8, 219, 114, 34, 76, 225, 158, 178, 171, 44, 21, 206, 85, 135, 95, 218, 80,
        229, 222, 56, 32, 233, 245, 238, 153, 232, 251, 134,
    ];
    const SECOND_ID: [u8; 32] = [
        61, 202, 46, 215, 146, 214, 232, 32, 155, 26, 209, 243, 231, 117, 234, 169, 84, 114, 137,
        175, 103, 40, 22, 203, 70, 67, 56, 244, 230, 213, 180, 182,
    ];
    const THIRD_ID: [u8; 32] = [
        90, 9, 100, 122, 200, 204, 166, 197, 160, 25, 192, 156, 157, 69, 122, 174, 149, 47, 247,
        106, 67, 79, 186, 214, 249, 10, 87, 89, 134, 231, 53, 9,
    ];

    #[test]
    fn adding_node() {
        let mut graph = IDGraph::new();
        graph.add_node(FIRST_ID).unwrap();
        graph.add_node(SECOND_ID).unwrap();

        assert_eq!(
            graph,
            IDGraph {
                nodes: [
                    (
                        FIRST_ID,
                        Neighbors {
                            parents: vec![],
                            children: vec![]
                        }
                    ),
                    (
                        SECOND_ID,
                        Neighbors {
                            parents: vec![],
                            children: vec![]
                        }
                    )
                ]
                .iter()
                .cloned()
                .collect(),
            }
        );
    }

    #[test]
    fn removing_valid_node() {
        let mut graph = IDGraph::new();
        graph.add_node(FIRST_ID).unwrap();
        graph.add_node(SECOND_ID).unwrap();

        graph.remove_node(SECOND_ID).unwrap();

        assert_eq!(
            graph,
            IDGraph {
                nodes: [(
                    FIRST_ID,
                    Neighbors {
                        parents: vec![],
                        children: vec![]
                    }
                )]
                .iter()
                .cloned()
                .collect(),
            }
        );
    }

    #[test]
    fn removing_invalid_node() {
        let mut graph = IDGraph::new();
        graph.add_node(FIRST_ID).unwrap();
        graph.add_node(SECOND_ID).unwrap();

        assert_eq!(graph.remove_node(THIRD_ID), Err("Invalid Node ID"));
    }

    #[test]
    fn adding_valid_edge() {
        let mut graph = IDGraph::new();
        graph.add_node(FIRST_ID).unwrap();
        graph.add_node(SECOND_ID).unwrap();
        graph.add_edge(FIRST_ID, SECOND_ID).unwrap();

        assert_eq!(
            graph,
            IDGraph {
                nodes: [
                    (
                        FIRST_ID,
                        Neighbors {
                            parents: vec![],
                            children: vec![SECOND_ID]
                        }
                    ),
                    (
                        SECOND_ID,
                        Neighbors {
                            parents: vec![FIRST_ID],
                            children: vec![]
                        }
                    )
                ]
                .iter()
                .cloned()
                .collect(),
            }
        );
    }

    #[test]
    fn adding_invalid_edge() {
        let mut graph = IDGraph::new();
        graph.add_node(FIRST_ID).unwrap();
        graph.add_node(SECOND_ID).unwrap();
        graph.add_edge(FIRST_ID, SECOND_ID).unwrap();

        assert_eq!(
            graph.add_edge(THIRD_ID, SECOND_ID),
            Err("Invalid Node ID for 'from' node")
        );

        assert_eq!(
            graph.add_edge(FIRST_ID, THIRD_ID),
            Err("Invalid Node ID for 'to' node")
        );

        assert_eq!(
            graph.add_edge(FIRST_ID, SECOND_ID),
            Err("'from' node already contains an edge to 'to' node")
        )
    }

    #[test]
    fn removing_valid_edge() {
        let mut graph = IDGraph::new();
        graph.add_node(FIRST_ID).unwrap();
        graph.add_node(SECOND_ID).unwrap();
        graph.add_edge(FIRST_ID, SECOND_ID).unwrap();
        graph.add_edge(SECOND_ID, FIRST_ID).unwrap();
        graph.remove_edge(FIRST_ID, SECOND_ID).unwrap();

        assert_eq!(
            graph,
            IDGraph {
                nodes: [
                    (
                        FIRST_ID,
                        Neighbors {
                            parents: vec![SECOND_ID],
                            children: vec![]
                        }
                    ),
                    (
                        SECOND_ID,
                        Neighbors {
                            parents: vec![],
                            children: vec![FIRST_ID]
                        }
                    )
                ]
                .iter()
                .cloned()
                .collect(),
            }
        )
    }

    #[test]
    fn removing_invalid_edge() {
        let mut graph = IDGraph::new();
        graph.add_node(FIRST_ID).unwrap();
        graph.add_node(SECOND_ID).unwrap();
        graph.add_edge(FIRST_ID, SECOND_ID).unwrap();

        assert_eq!(
            graph.remove_edge(THIRD_ID, SECOND_ID),
            Err("Invalid ID for 'from' node")
        );

        assert_eq!(
            graph.remove_edge(FIRST_ID, THIRD_ID),
            Err("Invalid ID for 'to' node")
        );

        graph.remove_edge(FIRST_ID, SECOND_ID).unwrap();

        assert_eq!(
            graph.remove_edge(FIRST_ID, SECOND_ID),
            Err("No edge exists between the 'from' node and the 'to' node")
        )
    }

    #[test]
    fn adding_duplicate_node_id() {
        let mut graph = IDGraph::new();
        graph.add_node(FIRST_ID).unwrap();

        assert_eq!(graph.add_node(FIRST_ID), Err("ID is already in the graph"));
    }
}
