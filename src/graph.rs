/// Implementation of a directed, cyclic graph.
use std::collections::HashMap;

type NodeID = usize;

/// The Graph struct holds all the relevant information of the graph.
/// Contains a HashMap of Nodes, as well as an internal ID counter.
/// This should be created with `Graph::new()` or `Default::default()`
#[derive(Eq, PartialEq)]
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
        let (pointing_to, pointed_to) = if let Some(node) = self.nodes.get_mut(&id) {
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
        if !self.nodes.contains_key(&from) {
            return Err("Invalid Node ID for 'from' node");
        }
        if !self.nodes.contains_key(&to) {
            return Err("Invalid Node ID for 'to' node");
        }

        let from_pointing_to = self.nodes.get_mut(&from).unwrap().pointing_to();

        if from_pointing_to.contains(&to) {
            return Err("'from' node already contains an edge to 'to' node");
        }

        from_pointing_to.push(to);

        let to_pointed_to = self.nodes.get_mut(&to).unwrap().pointed_to();

        if to_pointed_to.contains(&from) {
            return Err("'to' node already contains an edge from 'from' node");
        }

        to_pointed_to.push(from);

        Ok(())
    }

    /// Remove an edge between two nodes by ID. Fails if the node IDs are not found.
    pub fn remove_edge(&mut self, from: NodeID, to: NodeID) -> Result<(), &'static str> {
        if !self.nodes.contains_key(&from) {
            return Err("Invalid Node ID for 'from' node");
        }
        if !self.nodes.contains_key(&to) {
            return Err("Invalid Node ID for 'to' node");
        }

        let from_pointing_to = self.nodes.get_mut(&from).unwrap().pointing_to();

        if !from_pointing_to.contains(&to) {
            return Err("No edge exists between the 'from' node and the 'to' node");
        }

        from_pointing_to.retain(|elem| elem != &to);

        let to_pointed_to = self.nodes.get_mut(&to).unwrap().pointed_to();

        if !to_pointed_to.contains(&from) {
            return Err("No edge exists between the 'to' node and the 'from' node");
        }

        to_pointed_to.retain(|elem| elem != &from);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct TestNode {
        id: NodeID,
        obj: usize,
        pointing_to: Vec<NodeID>,
        pointed_to: Vec<NodeID>,
    }

    impl Node for TestNode {
        type Inner = usize;

        fn new(obj: Self::Inner, id: NodeID) -> Self {
            TestNode {
                id,
                obj,
                pointing_to: vec![],
                pointed_to: vec![],
            }
        }

        fn pointing_to(&mut self) -> &mut Vec<NodeID> {
            &mut self.pointing_to
        }

        fn pointed_to(&mut self) -> &mut Vec<NodeID> {
            &mut self.pointed_to
        }

        fn id(&self) -> NodeID {
            self.id
        }
    }

    #[test]
    fn adding_node() {
        let mut graph: Graph<TestNode> = Graph::new();
        graph.add_node(5);
        graph.add_node(3);

        assert_eq!(
            graph,
            Graph {
                nodes: [
                    (
                        0,
                        TestNode {
                            id: 0,
                            obj: 5,
                            pointed_to: vec![],
                            pointing_to: vec![]
                        }
                    ),
                    (
                        1,
                        TestNode {
                            id: 1,
                            obj: 3,
                            pointed_to: vec![],
                            pointing_to: vec![]
                        }
                    )
                ]
                .iter()
                .cloned()
                .collect::<HashMap<NodeID, TestNode>>(),
                id_counter: 2
            }
        );
    }

    #[test]
    fn removing_valid_node() {
        let mut graph: Graph<TestNode> = Graph::new();
        graph.add_node(5);
        let id = graph.add_node(3);
        graph.remove_node(id).unwrap();

        assert_eq!(
            graph,
            Graph {
                nodes: [(
                    0,
                    TestNode {
                        id: 0,
                        obj: 5,
                        pointed_to: vec![],
                        pointing_to: vec![]
                    }
                )]
                .iter()
                .cloned()
                .collect::<HashMap<NodeID, TestNode>>(),
                id_counter: 2
            }
        );
    }

    #[test]
    fn removing_invalid_node() {
        let mut graph: Graph<TestNode> = Graph::new();
        graph.add_node(5);
        graph.add_node(3);

        assert_eq!(graph.remove_node(20), Err("Invalid Node ID"));
    }

    #[test]
    fn adding_valid_edge() {
        let mut graph: Graph<TestNode> = Graph::new();
        let first_id = graph.add_node(5);
        let second_id = graph.add_node(3);
        graph.add_edge(first_id, second_id).unwrap();

        assert_eq!(
            graph,
            Graph {
                nodes: [
                    (
                        0,
                        TestNode {
                            id: 0,
                            obj: 5,
                            pointed_to: vec![],
                            pointing_to: vec![1]
                        }
                    ),
                    (
                        1,
                        TestNode {
                            id: 1,
                            obj: 3,
                            pointed_to: vec![0],
                            pointing_to: vec![]
                        }
                    )
                ]
                .iter()
                .cloned()
                .collect::<HashMap<NodeID, TestNode>>(),
                id_counter: 2
            }
        );
    }

    #[test]
    fn adding_invalid_edge() {
        let mut graph: Graph<TestNode> = Graph::new();
        let first_id = graph.add_node(5);
        let second_id = graph.add_node(3);
        graph.add_edge(first_id, second_id).unwrap();

        assert_eq!(
            graph.add_edge(10, second_id),
            Err("Invalid Node ID for 'from' node")
        );

        assert_eq!(
            graph.add_edge(first_id, 10),
            Err("Invalid Node ID for 'to' node")
        );

        assert_eq!(
            graph.add_edge(first_id, second_id),
            Err("'from' node already contains an edge to 'to' node")
        )
    }

    #[test]
    fn removing_valid_edge() {
        let mut graph: Graph<TestNode> = Graph::new();
        let first_id = graph.add_node(5);
        let second_id = graph.add_node(3);
        graph.add_edge(first_id, second_id).unwrap();
        graph.add_edge(second_id, first_id).unwrap();
        graph.remove_edge(first_id, second_id).unwrap();

        assert_eq!(
            graph,
            Graph {
                nodes: [
                    (
                        0,
                        TestNode {
                            id: 0,
                            obj: 5,
                            pointed_to: vec![1],
                            pointing_to: vec![]
                        }
                    ),
                    (
                        1,
                        TestNode {
                            id: 1,
                            obj: 3,
                            pointed_to: vec![],
                            pointing_to: vec![0]
                        }
                    ),
                ]
                .iter()
                .cloned()
                .collect::<HashMap<NodeID, TestNode>>(),
                id_counter: 2
            }
        )
    }

    #[test]
    fn removing_invalid_edge() {
        let mut graph: Graph<TestNode> = Graph::new();
        let first_id = graph.add_node(5);
        let second_id = graph.add_node(3);
        graph.add_edge(first_id, second_id).unwrap();

        assert_eq!(
            graph.remove_edge(10, second_id),
            Err("Invalid Node ID for 'from' node")
        );

        assert_eq!(
            graph.remove_edge(first_id, 10),
            Err("Invalid Node ID for 'to' node")
        );

        graph.remove_edge(first_id, second_id).unwrap();

        assert_eq!(
            graph.remove_edge(first_id, second_id),
            Err("No edge exists between the 'from' node and the 'to' node")
        )
    }
}
