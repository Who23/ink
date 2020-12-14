/// Implementation of a directed, cyclic graph.

use std::collections::HashMap;

type NodeID = usize;

/// The Graph struct holds all the relevant information of the graph.
/// Contains a HashMap of Nodes, as well as an internal ID counter.
/// This should be created with `Graph::new()` or `Default::default()`
pub struct Graph<N: Node> {
    pub nodes: HashMap<NodeID, N>,
    id_counter: NodeID,
}

/// A graph node
/// Must store/return an ID, a vec of nodes it is pointing to, 
/// and a vec of nodes pointing to it.
pub trait Node {
    type Inner;

    fn new(obj: Self::Inner, id: NodeID) -> Self;
    fn pointing_to(&mut self) -> &mut Vec<NodeID>;
    fn pointed_to(&mut self) -> &mut Vec<NodeID>;
    fn id(&self) -> NodeID;
}

impl<N: Node> Graph<N> {
    /// Create a new graph
    pub fn new() -> Self {
        Graph {
            nodes: HashMap::new(),
            id_counter: 0,
        }
    }

    /// Add a node to the graph, given the inner object.
    /// Returns the ID of the created node.
    pub fn add_node(&mut self, obj: N::Inner) -> NodeID {
        let node = Node::new(obj, self.id_counter);

        self.nodes.insert(self.id_counter, node);
        self.id_counter += 1;

        self.id_counter - 1
    }

    /// Remove a node by ID. Fails if the node ID is not found.
    pub fn remove_node(&mut self, id: NodeID) -> Result<(), &'static str> {
        // get edge data for this node
        let (pointing_to, pointed_to) = if let Some(node) = self.nodes.get(&id) {
            Ok((node.pointing_to().clone(), node.pointed_to().clone()))
        } else {
            Err("Invalid Node ID")
        }?;

        // remove all edges for this node
        for other_id in pointed_to.iter() {
            if let Some(node) = self.nodes.get_mut(&other_id) {
                (*node).pointing_to().retain(|elem| elem != &id)
            }
        }

        for other_id in pointing_to.iter() {
            if let Some(node) = self.nodes.get_mut(&other_id) {
                (*node).pointed_to().retain(|elem| elem != &id)
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
            .pointing_to()
            .push(to);

        self.nodes
            .get_mut(&to)
            .ok_or("Invalid Node ID for 'to' node")?
            .pointed_to()
            .push(from);

        Ok(())
    }

    /// Remove an edge between two nodes by ID. Fails if the node IDs are not found.
    pub fn remove_edge(&mut self, from: NodeID, to: NodeID) -> Result<(), &'static str> {
        self.nodes
            .get_mut(&from)
            .ok_or("Invalid Node ID for 'from' node")?
            .pointing_to()
            .retain(|elem| elem != &to);

        self.nodes
            .get_mut(&to)
            .ok_or("Invalid Node ID for 'to' node")?
            .pointed_to()
            .retain(|elem| elem != &to);

        Ok(())
    }
}

impl<N: Node> Default for Graph<N> {
    fn default() -> Self {
        Graph {
            nodes: Default::default(),
            id_counter: Default::default(),
        }
    }
}

impl<N> std::fmt::Debug for Graph<N>
where
    N: Node + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Graph")
            .field("nodes", &self.nodes)
            .field("id_counter", &self.id_counter)
            .finish()
    }
}