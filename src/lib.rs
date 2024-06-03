//! A simple tree structure library for Rust.
//!
//! `easy-tree` is a simple and efficient tree structure library for Rust. It allows you to create and manipulate tree
//! structures where each node can have multiple children and a single parent. `easy-tree` also supports recursively
//! traversing the tree in a depth-first manner, with two callbacks: one before processing any children and one after
//! processing the subtree belonging to that node (meaning children, and their children, and so on).
//!
//! # Examples
//!
//! ## Traverse a tree
//! ```
//! use easy_tree::Tree;
//!
//! let mut tree = Tree::new();
//! let root = tree.add_node(0); // Root node with data 0
//! let child1 = tree.add_child(root, 1); // Child node with data 1
//! let child2 = tree.add_child(root, 2); // Child node with data 2
//! let child3 = tree.add_child(child1, 3); // Child node with data 3
//!
//! let mut result = vec![];
//!
//! tree.traverse(|index, node, result| {
//!     result.push(format!("Calling handler for node {}: {}", index, node))
//! }, |index, node, result| {
//!     result.push(format!("Finished handling node {} and all it's children", index))
//! }, &mut result);
//! assert_eq!(result, vec![
//!     "Calling handler for node 0: 0",
//!     "Calling handler for node 1: 1",
//!     "Calling handler for node 3: 3",
//!     "Finished handling node 3 and all it's children",
//!     "Finished handling node 1 and all it's children",
//!     "Calling handler for node 2: 2",
//!     "Finished handling node 2 and all it's children",
//!     "Finished handling node 0 and all it's children",
//! ]);
//! ```
//!
//! ## Iterate over the nodes in a tree
//! ```
//! use easy_tree::Tree;
//!
//! // Create a new tree and add nodes
//! let mut tree = Tree::new();
//! let root = tree.add_node(0); // Root node with data 0
//! let child1 = tree.add_child(root, 1); // Child node with data 1
//! let child2 = tree.add_child(root, 2); // Child node with data 2
//! let child3 = tree.add_child(child1, 3); // Child node with data 3
//!
//! // Access nodes and their relationships
//! assert_eq!(tree.get(root), Some(&0));
//! assert_eq!(tree.get(child1), Some(&1));
//! assert_eq!(tree.get(child2), Some(&2));
//! assert_eq!(tree.get(child3), Some(&3));
//!
//! assert_eq!(tree.parent_index_unchecked(child1), Some(root));
//! assert_eq!(tree.parent_index_unchecked(child2), Some(root));
//! assert_eq!(tree.parent_index_unchecked(child3), Some(child1));
//!
//! assert_eq!(tree.children(root), &[child1, child2]);
//! assert_eq!(tree.children(child1), &[child3]);
//! assert_eq!(tree.children(child2), &[]);
//! assert_eq!(tree.children(child3), &[]);
//! ```
//!
//! ## Modify the nodes in a tree
//! ```
//! use easy_tree::Tree;
//!
//! // Create a new tree and add nodes
//! let mut tree = Tree::new();
//! let root = tree.add_node(0); // Root node with data 0
//! let child1 = tree.add_child(root, 1); // Child node with data 1
//! let child2 = tree.add_child(root, 2); // Child node with data 2
//! let child3 = tree.add_child(child1, 3); // Child node with data 3
//!
//! // Iterate over the nodes in the tree
//! for (index, data) in tree.iter() {
//!     println!("Node {}: {}", index, data);
//! }
//!
//! // Iterate over the nodes in the tree mutably
//! for (index, data) in tree.iter_mut() {
//!     *data += 1;
//! }
//!
//! // Check the modified values
//! assert_eq!(tree.get(root), Some(&1));
//! assert_eq!(tree.get(child1), Some(&2));
//! assert_eq!(tree.get(child2), Some(&3));
//! assert_eq!(tree.get(child3), Some(&4));
//! ```

#[cfg(feature = "rayon")]
pub use rayon;
#[cfg(feature = "rayon")]
use rayon::prelude::*;

#[derive(Clone)]
/// A node in the tree containing data and references to its parent and children.
pub struct Node<T> {
    data: T,
    children: Vec<usize>,
    parent: Option<usize>,
}

impl<T> Node<T> {
    /// Creates a new node with the given data.
    pub fn new(data: T) -> Self {
        Self {
            data,
            children: Vec::new(),
            parent: None,
        }
    }

    pub(crate) fn add_child(&mut self, child: usize) {
        self.children.push(child);
    }

    pub(crate) fn set_parent(&mut self, parent: usize) {
        self.parent = Some(parent);
    }
}

#[derive(Clone)]
/// A tree structure containing nodes.
pub struct Tree<T> {
    nodes: Vec<Node<T>>,
}

impl<T> Default for Tree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Tree<T> {
    /// Creates a new empty tree.
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Adds a new node with the given data to the tree and returns its index. The nodes
    /// added with this method will be disconnected from the tree, so use it only for the root node.
    /// For adding children, use the `add_child` method.
    pub fn add_node(&mut self, data: T) -> usize {
        let node = Node::new(data);
        let index = self.nodes.len();
        self.nodes.push(node);
        index
    }

    /// Adds a child node with the given data to the specified parent node and returns the child's
    /// index.
    pub fn add_child(&mut self, parent: usize, data: T) -> usize {
        let index = self.add_node(data);
        self.nodes[parent].add_child(index);
        self.nodes[index].set_parent(parent);
        index
    }

    /// Adds a child node with the given data to the root node and returns the child's index.
    pub fn add_child_to_root(&mut self, data: T) -> usize {
        self.add_child(0, data)
    }

    /// Returns a reference to the data of the node at the given index, or `None` if the index is
    /// out of bounds.
    pub fn get(&self, index: usize) -> Option<&T> {
        self.nodes.get(index).map(|node| &node.data)
    }

    #[inline(always)]
    /// Returns a reference to the data of the node at the given index without bounds checking.
    pub fn get_unchecked(&self, index: usize) -> &T {
        &self.nodes[index].data
    }

    /// Returns a mutable reference to the data of the node at the given index, or `None` if the
    /// index is out of bounds.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.nodes.get_mut(index).map(|node| &mut node.data)
    }

    #[inline(always)]
    /// Returns a mutable reference to the data of the node at the given index without bounds
    /// checking.
    pub fn get_unchecked_mut(&mut self, index: usize) -> &mut T {
        &mut self.nodes[index].data
    }

    /// Returns the index of the parent node of the node at the given index, or `None` if the node
    /// has no parent.
    pub fn parent_index_unchecked(&self, index: usize) -> Option<usize> {
        self.nodes[index].parent
    }

    /// Returns a slice of the indices of the children of the node at the given index.
    pub fn children(&self, index: usize) -> &[usize] {
        &self.nodes[index].children
    }

    /// Walks the tree recursively, applying the given functions before and after processing the
    /// children of each node.
    pub fn traverse<'a, S>(
        &'a self,
        mut before_processing_children: impl FnMut(usize, &'a T, &mut S),
        mut after_processing_the_subtree: impl FnMut(usize, &'a T, &mut S),
        s: &mut S,
    ) {
        let mut stack = vec![(0, false)];

        while let Some((index, children_visited)) = stack.pop() {
            if children_visited {
                // All children are processed, call f2
                let node = &self.nodes[index];
                after_processing_the_subtree(index, &node.data, s);
            } else {
                // Call f and mark this node's children for processing
                let node = &self.nodes[index];
                before_processing_children(index, &node.data, s);

                // Re-push the current node with children_visited set to true
                stack.push((index, true));

                // Push all children onto the stack
                for &child in node.children.iter().rev() {
                    stack.push((child, false));
                }
            }
        }
    }

    /// Returns an iterator over the indices and data of the nodes in the tree.
    pub fn iter(&self) -> impl Iterator<Item = (usize, &T)> {
        self.nodes
            .iter()
            .enumerate()
            .map(|(index, node)| (index, &node.data))
    }

    /// Returns a mutable iterator over the indices and data of the nodes in the tree.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (usize, &mut T)> {
        self.nodes
            .iter_mut()
            .enumerate()
            .map(|(index, node)| (index, &mut node.data))
    }

    /// Returns `true` if the tree contains no nodes.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns the number of nodes in the tree.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Removes all nodes from the tree.
    pub fn clear(&mut self) {
        self.nodes.clear();
    }
}

#[cfg(feature = "rayon")]
impl<T: Send + Sync> Tree<T> {
    #[cfg(feature = "rayon")]
    /// Returns a parallel iterator over the indices and data of the nodes in the tree.
    pub fn par_iter(&self) -> impl ParallelIterator<Item = (usize, &T)> {
        self.nodes
            .par_iter()
            .enumerate()
            .map(|(index, node)| (index, &node.data))
    }

    #[cfg(feature = "rayon")]
    /// Returns a mutable parallel iterator over the indices and data of the nodes in the tree.
    pub fn par_iter_mut(&mut self) -> impl ParallelIterator<Item = (usize, &mut T)> {
        self.nodes
            .par_iter_mut()
            .enumerate()
            .map(|(index, node)| (index, &mut node.data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree() {
        let mut tree = Tree::new();
        let root = tree.add_node(0);
        let child1 = tree.add_child(root, 1);
        let child2 = tree.add_child(root, 2);
        let child3 = tree.add_child(child1, 3);

        assert_eq!(tree.get(root), Some(&0));
        assert_eq!(tree.get(child1), Some(&1));
        assert_eq!(tree.get(child2), Some(&2));
        assert_eq!(tree.get(child3), Some(&3));

        assert_eq!(tree.parent_index_unchecked(child1), Some(root));
        assert_eq!(tree.parent_index_unchecked(child2), Some(root));
        assert_eq!(tree.parent_index_unchecked(child3), Some(child1));

        assert_eq!(tree.children(root), &[child1, child2]);
        assert_eq!(tree.children(child1), &[child3]);
        assert_eq!(tree.children(child2), &[]);
        assert_eq!(tree.children(child3), &[]);
    }

    #[test]
    fn test_tree_iter() {
        let mut tree = Tree::new();
        let root = tree.add_node(0);
        let child1 = tree.add_child(root, 1);
        let child2 = tree.add_child(root, 2);
        let child3 = tree.add_child(child1, 3);

        let mut iter = tree.iter();
        assert_eq!(iter.next(), Some((root, &0)));
        assert_eq!(iter.next(), Some((child1, &1)));
        assert_eq!(iter.next(), Some((child2, &2)));
        assert_eq!(iter.next(), Some((child3, &3)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_tree_iter_mut() {
        let mut tree = Tree::new();
        let root = tree.add_node(0);
        let child1 = tree.add_child(root, 1);
        let child2 = tree.add_child(root, 2);
        let child3 = tree.add_child(child1, 3);

        let mut iter = tree.iter_mut();
        assert_eq!(iter.next(), Some((root, &mut 0)));
        assert_eq!(iter.next(), Some((child1, &mut 1)));
        assert_eq!(iter.next(), Some((child2, &mut 2)));
        assert_eq!(iter.next(), Some((child3, &mut 3)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_tree_traverse() {
        let mut tree = Tree::new();
        let root = tree.add_node(0); // Root node with data 0
        let child1 = tree.add_child(root, 1); // Child node with data 1
        let _child2 = tree.add_child(root, 2); // Child node with data 2
        let _child3 = tree.add_child(child1, 3); // Child node with data 3

        let mut result = vec![];

        tree.traverse(
            |index, node, result| {
                result.push(format!("Calling handler for node {}: {}", index, node))
            },
            |index, _node, result| {
                result.push(format!(
                    "Finished handling node {} and all it's children",
                    index
                ))
            },
            &mut result,
        );

        assert_eq!(
            result,
            vec![
                "Calling handler for node 0: 0",
                "Calling handler for node 1: 1",
                "Calling handler for node 3: 3",
                "Finished handling node 3 and all it's children",
                "Finished handling node 1 and all it's children",
                "Calling handler for node 2: 2",
                "Finished handling node 2 and all it's children",
                "Finished handling node 0 and all it's children",
            ]
        );
    }
}
