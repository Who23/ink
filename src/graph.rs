/// Implementation of a directed, cyclic graph.

use std::collections::HashMap;

type NodeID = usize;

/// The Graph struct holds all the relevant information of the graph.
/// Contains a list of Nodes of type T, as well as an internal ID counter.
/// This should be created with `Graph::new()` or `Default::default()`
pub struct Graph<T> {
    pub nodes: HashMap<NodeID, Node<T>>,
    id_counter: NodeID,
}

/// A graph node of type T
/// Holds T, an ID, the nodes it points to and the nodes that point to it.
/// A node may point to itself.
/// This should never be created by the user, only by the `Graph` struct.
pub struct Node<T> {
    obj: T,
    id: NodeID,
    pointing_to: Vec<NodeID>,
    pointed_to: Vec<NodeID>,
}

impl<T> Graph<T> {
    /// Create a new graph
    pub fn new() -> Self {
        Graph {
            nodes: HashMap::new(),
            id_counter: 0,
        }
    }

    /// Add a node to the graph, given T. Returns the ID of the created node.
    pub fn add_node(&mut self, obj: T) -> NodeID {
        let node = Node {
            obj,
            id: self.id_counter,
            pointing_to: vec![],
            pointed_to: vec![],
        };

        self.nodes.insert(self.id_counter, node);
        self.id_counter += 1;

        self.id_counter - 1
    }

    /// Remove a node by ID. Fails if the node ID is not found.
    pub fn remove_node(&mut self, id: NodeID) -> Result<(), &'static str> {
        // get edge data for this node
        let (pointing_to, pointed_to) = if let Some(node) = self.nodes.get(&id) {
            Ok((node.pointing_to.clone(), node.pointed_to.clone()))
        } else {
            Err("Invalid Node ID")
        }?;

        // remove all edges for this node
        for other_id in pointed_to.iter() {
            if let Some(node) = self.nodes.get_mut(&other_id) {
                (*node).pointing_to.retain(|elem| elem != &id)
            }
        }

        for other_id in pointing_to.iter() {
            if let Some(node) = self.nodes.get_mut(&other_id) {
                (*node).pointed_to.retain(|elem| elem != &id)
            }
        }

        // remove the node
        self.nodes.remove(&id);

        Ok(())
    }

    /// Add an edge between two nodes by ID. Fails if the node IDs are not found.
    /// It does allow a node to create an edge with itself.
    pub fn add_edge(&mut self, from: NodeID, to: NodeID) -> Result<(), &'static str> {
        self.nodes
            .get_mut(&from)
            .ok_or("Invalid Node ID for 'from' node")?
            .pointing_to
            .push(to);

        self.nodes
            .get_mut(&to)
            .ok_or("Invalid Node ID for 'to' node")?
            .pointed_to
            .push(from);

        Ok(())
    }

    /// Remove an edge between two nodes by ID. Fails if the node IDs are not found.
    pub fn remove_edge(&mut self, from: NodeID, to: NodeID) -> Result<(), &'static str> {
        self.nodes
            .get_mut(&from)
            .ok_or("Invalid Node ID for 'from' node")?
            .pointing_to
            .retain(|elem| elem != &to);

        self.nodes
            .get_mut(&to)
            .ok_or("Invalid Node ID for 'to' node")?
            .pointed_to
            .retain(|elem| elem != &to);

        Ok(())
    }
}

impl<T> Default for Graph<T> {
    fn default() -> Self {
        Graph {
            nodes: Default::default(),
            id_counter: Default::default(),
        }
    }
}

impl<T> std::fmt::Debug for Graph<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Graph")
            .field("nodes", &self.nodes)
            .field("id_counter", &self.id_counter)
            .finish()
    }
}

impl<T> std::fmt::Debug for Node<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("obj", &self.obj)
            .field("id", &self.id)
            .field("pointing_to", &self.pointing_to)
            .field("pointed_to", &self.pointed_to)
            .finish()
    }
}
