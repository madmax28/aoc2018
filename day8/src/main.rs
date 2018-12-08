use std::fs;

#[derive(Debug)]
enum Error {
    InvalidInput,
    BuildTree,
    InvalidNodeId,
}

type NodeId = usize;

#[derive(Debug, PartialEq)]
struct Node {
    children: Vec<NodeId>,
    metadata: Vec<u32>,
}

impl Node {
    fn with_capacity(num_children: usize, num_metadata: usize) -> Self {
        Node {
            children: Vec::with_capacity(num_children),
            metadata: Vec::with_capacity(num_metadata),
        }
    }
}

#[derive(Debug)]
struct NodeState {
    id: NodeId,
    child_cnt: u32,
    data_cnt: u32,
}

impl NodeState {
    fn new(id: NodeId, child_cnt: u32, data_cnt: u32) -> Self {
        NodeState {
            id,
            child_cnt,
            data_cnt,
        }
    }
}

#[derive(Debug)]
struct Tree {
    nodes: Vec<Node>,
}

impl Tree {
    fn from_nums(nums: &[u32]) -> Result<Self, Error> {
        let mut nodes: Vec<Node> = Vec::new();
        let mut stack: Vec<NodeState> = Vec::new();
        let mut next_id = 0;

        let mut it = nums.iter();
        while let Some(num) = it.next() {
            if let Some(s) = stack.last_mut() {
                if s.child_cnt > 0 {
                    s.child_cnt -= 1;
                    nodes[s.id].children.push(next_id);
                    let (child_cnt, data_cnt) = (*num, *it.next().ok_or(Error::InvalidInput)?);
                    stack.push(NodeState::new(next_id, child_cnt, data_cnt));
                    nodes.push(Node::with_capacity(child_cnt as usize, data_cnt as usize));
                    next_id += 1;
                } else if s.data_cnt > 0 {
                    s.data_cnt -= 1;
                    nodes[s.id].metadata.push(*num);
                    if s.data_cnt == 0 {
                        stack.pop();
                    }
                } else {
                    return Err(Error::BuildTree);
                }
            } else {
                // First node
                let (child_cnt, data_cnt) = (*num, *it.next().ok_or(Error::InvalidInput)?);
                stack.push(NodeState::new(next_id, child_cnt, data_cnt));
                nodes.push(Node::with_capacity(child_cnt as usize, data_cnt as usize));
                next_id += 1;
            }
        }

        Ok(Tree { nodes })
    }

    fn get_node_value(&self, mut id: NodeId) -> Result<u32, Error> {
        if id >= self.nodes.len() {
            return Err(Error::InvalidNodeId);
        }

        let mut to_visit = vec![id];
        let mut value = 0;
        while !to_visit.is_empty() {
            id = to_visit.pop().unwrap();
            let node = &self.nodes[id];
            if !node.children.is_empty() {
                for data in node
                    .metadata
                    .iter()
                    .filter(|&&data| data > 0 && (data as NodeId) <= node.children.len())
                {
                    to_visit.push(node.children[*data as NodeId - 1]);
                }
            } else {
                value += node.metadata.iter().sum::<u32>()
            }
        }
        Ok(value)
    }
}

fn main() -> Result<(), Box<std::error::Error>> {
    let input = fs::read_to_string("input")?;
    let nums: Vec<u32> = input
        .trim()
        .split(' ')
        .map(|s| s.parse())
        .collect::<Result<_, _>>()?;

    let tree = Tree::from_nums(&nums).expect("error building tree");
    println!(
        "Metadata sum: {:?}",
        tree.nodes
            .iter()
            .flat_map(|node| &node.metadata)
            .sum::<u32>()
    );
    println!(
        "Node 0 value: {}",
        tree.get_node_value(0).expect("invalid node id")
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const NUMS: &[u32] = &[2, 3, 0, 3, 10, 11, 12, 1, 1, 0, 1, 99, 2, 1, 1, 2];

    #[test]
    fn build_tree() {
        let tree =
            Tree::from_nums(NUMS).unwrap_or_else(|err| panic!("building tree failed: {:?}", err));

        let mut nodes = tree.nodes.iter();
        assert_eq!(
            *nodes.next().expect("node missing"),
            Node {
                children: [1, 2].to_vec(),
                metadata: [1, 1, 2].to_vec(),
            }
        );
        assert_eq!(
            *nodes.next().expect("node missing"),
            Node {
                children: [].to_vec(),
                metadata: [10, 11, 12].to_vec(),
            }
        );
        assert_eq!(
            *nodes.next().expect("node missing"),
            Node {
                children: [3].to_vec(),
                metadata: [2].to_vec(),
            }
        );
        assert_eq!(
            *nodes.next().expect("node missing"),
            Node {
                children: [].to_vec(),
                metadata: [99].to_vec(),
            }
        );
    }

    #[test]
    fn find_value() {
        let tree =
            Tree::from_nums(NUMS).unwrap_or_else(|err| panic!("building tree failed: {:?}", err));
        assert_eq!(tree.get_node_value(0).expect("invalid node id"), 66);
    }
}
