/// Implementation of a directed, cyclic graph for ink objects
/// Nodes in the graph are not assigned IDs but are expected to generate their own
/// unique IDs. This is so nodes can be referenced by their hashes.
use std::collections::HashMap;
use std::hash::Hash;

/// The Graph struct holds all the relevant information of the graph.
/// Contains a HashMap of Nodes
/// This should be created with `Graph::new()` or `Default::default()`
#[derive(Eq, PartialEq)]
pub struct Graph<N: Node<I>, I: Hash + Eq> {
    pub nodes: HashMap<I, N>,
}

/// A graph node
/// Must store/return an ID, a vec of nodes it is pointing to,
/// and a vec of nodes pointing to it. The implementation requires ID's
/// to be generated from the Node, because all ink objects have hashes
/// used as IDs.
pub trait Node<I: Hash + Eq> {
    type Inner;

    fn pointing_to(&mut self) -> &mut Vec<I>;
    fn pointed_to(&mut self) -> &mut Vec<I>;
    fn id(&self) -> I;
}

impl<N: Node<I>, I: Hash + Eq + Clone> Graph<N, I> {
    /// Create a new graph
    pub fn new() -> Self {
        Graph {
            nodes: HashMap::new(),
        }
    }

    /// Add a node to the graph
    /// Returns the ID of the created node.
    pub fn add_node(&mut self, node: N) -> Result<I, &'static str> {
        let node_id = node.id();

        if self.nodes.contains_key(&node_id) {
            return Err("Node ID is not unique");
        }

        self.nodes.insert(node.id(), node);
        Ok(node_id)
    }

    /// Remove a node by ID. Fails if the node ID is not found.
    pub fn remove_node(&mut self, id: I) -> Result<(), &'static str> {
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
    pub fn add_edge(&mut self, from: I, to: I) -> Result<(), &'static str> {
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

        from_pointing_to.push(to.clone());

        let to_pointed_to = self.nodes.get_mut(&to).unwrap().pointed_to();

        if to_pointed_to.contains(&from) {
            return Err("'to' node already contains an edge from 'from' node");
        }

        to_pointed_to.push(from);

        Ok(())
    }

    /// Remove an edge between two nodes by ID. Fails if the node IDs are not found.
    pub fn remove_edge(&mut self, from: I, to: I) -> Result<(), &'static str> {
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

impl<N: Node<I>, I: Hash + Eq> Default for Graph<N, I> {
    fn default() -> Self {
        Graph {
            nodes: Default::default(),
        }
    }
}

impl<N, I> std::fmt::Debug for Graph<N, I>
where
    N: Node<I> + std::fmt::Debug,
    I: Hash + Eq + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Graph").field("nodes", &self.nodes).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct TestNode {
        id: usize,
        obj: usize,
        pointing_to: Vec<usize>,
        pointed_to: Vec<usize>,
    }

    impl TestNode {
        fn new(obj: usize, id: usize) -> Self {
            TestNode {
                id,
                obj,
                pointing_to: vec![],
                pointed_to: vec![],
            }
        }
    }

    impl Node<usize> for TestNode {
        type Inner = usize;

        fn pointing_to(&mut self) -> &mut Vec<usize> {
            &mut self.pointing_to
        }

        fn pointed_to(&mut self) -> &mut Vec<usize> {
            &mut self.pointed_to
        }

        fn id(&self) -> usize {
            self.id
        }
    }

    #[test]
    fn adding_node() {
        let mut graph: Graph<TestNode, usize> = Graph::new();
        graph.add_node(TestNode::new(5, 0)).unwrap();
        graph.add_node(TestNode::new(3, 1)).unwrap();

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
                .collect::<HashMap<usize, TestNode>>(),
            }
        );
    }

    #[test]
    fn removing_valid_node() {
        let mut graph: Graph<TestNode, usize> = Graph::new();
        graph.add_node(TestNode::new(5, 0)).unwrap();
        let id = graph.add_node(TestNode::new(3, 1)).unwrap();
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
                .collect::<HashMap<usize, TestNode>>(),
            }
        );
    }

    #[test]
    fn removing_invalid_node() {
        let mut graph: Graph<TestNode, usize> = Graph::new();
        graph.add_node(TestNode::new(5, 0)).unwrap();
        graph.add_node(TestNode::new(3, 1)).unwrap();

        assert_eq!(graph.remove_node(20), Err("Invalid Node ID"));
    }

    #[test]
    fn adding_valid_edge() {
        let mut graph: Graph<TestNode, usize> = Graph::new();
        let first_id = graph.add_node(TestNode::new(5, 0)).unwrap();
        let second_id = graph.add_node(TestNode::new(3, 1)).unwrap();
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
                .collect::<HashMap<usize, TestNode>>(),
            }
        );
    }

    #[test]
    fn adding_invalid_edge() {
        let mut graph: Graph<TestNode, usize> = Graph::new();
        let first_id = graph.add_node(TestNode::new(5, 0)).unwrap();
        let second_id = graph.add_node(TestNode::new(3, 1)).unwrap();
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
        let mut graph: Graph<TestNode, usize> = Graph::new();
        let first_id = graph.add_node(TestNode::new(5, 0)).unwrap();
        let second_id = graph.add_node(TestNode::new(3, 1)).unwrap();
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
                .collect::<HashMap<usize, TestNode>>(),
            }
        )
    }

    #[test]
    fn removing_invalid_edge() {
        let mut graph: Graph<TestNode, usize> = Graph::new();
        let first_id = graph.add_node(TestNode::new(5, 0)).unwrap();
        let second_id = graph.add_node(TestNode::new(3, 1)).unwrap();
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

    #[test]
    fn adding_duplicate_node_id() {
        let mut graph: Graph<TestNode, usize> = Graph::new();
        graph.add_node(TestNode::new(5, 0)).unwrap();

        assert_eq!(
            graph.add_node(TestNode::new(3, 0)),
            Err("Node ID is not unique")
        );
    }
}
