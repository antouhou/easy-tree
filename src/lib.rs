//! # easy-tree
//!
//! `easy-tree` is a lightweight library for creating and manipulating tree structures in Rust.
//! It provides a simple and efficient interface for managing hierarchical data and supports
//! **depth-first traversal** with pre- and post-processing callbacks for flexible operations.
//!
//! ## Features
//!
//! - **Simple API**: Easily create, add, and retrieve nodes in the tree.
//! - **Depth-first traversal**: Recursively traverse the tree with callbacks before and after processing subtrees.
//! - **Flexible node access**: Access parent-child relationships and modify node data.
//! - **Optional parallel iteration**: Speed up iteration with [rayon](https://docs.rs/rayon) when enabled.
//!
//! ## Use Cases
//!
//! `easy-tree` is ideal for representing and traversing hierarchical data, such as:
//! - **File systems**
//! - **Organizational charts**
//! - **Abstract syntax trees (ASTs)**
//! - **Graph-like structures with one parent per node**
//!
//! # Examples
//!
//! ## 1. Basic Tree Operations
//! ```rust
//! use easy_tree::Tree;
//!
//! fn main() {
//!     let mut tree = Tree::new();
//!     let root = tree.add_node("root");
//!     let child1 = tree.add_child(root, "child1");
//!     let child2 = tree.add_child(root, "child2");
//!     let grandchild = tree.add_child(child1, "grandchild");
//!
//!     assert_eq!(tree.get(root), Some(&"root"));
//!     assert_eq!(tree.get(grandchild), Some(&"grandchild"));
//!     assert_eq!(tree.children(root), &[child1, child2]);
//!     assert_eq!(tree.parent_index_unchecked(grandchild), Some(child1));
//! }
//! ```
//!
//! ## 2. Depth-First Traversal
//! Process nodes before and after their children using callbacks.
//!
//! ```rust
//! use easy_tree::Tree;
//!
//! fn main() {
//!     let mut tree = Tree::new();
//!     let root = tree.add_node("root");
//!     let child1 = tree.add_child(root, "child1");
//!     let child2 = tree.add_child(root, "child2");
//!
//!     let mut result = vec![];
//!     tree.traverse(
//!         |idx, data, result| result.push(format!("Entering node {}: {}", idx, data)),
//!         |idx, data, result| result.push(format!("Leaving node {}: {}", idx, data)),
//!         &mut result,
//!     );
//!
//!     assert_eq!(result, vec![
//!         "Entering node 0: root",
//!         "Entering node 1: child1",
//!         "Leaving node 1: child1",
//!         "Entering node 2: child2",
//!         "Leaving node 2: child2",
//!         "Leaving node 0: root",
//!     ]);
//! }
//! ```
//!
//! ## 3. Iteration
//!
//! Iterate over nodes and modify their data.
//!
//! ```rust
//! use easy_tree::Tree;
//!
//! fn main() {
//!     let mut tree = Tree::new();
//!     let root = tree.add_node(0);
//!     let child1 = tree.add_child(root, 1);
//!     let child2 = tree.add_child(root, 2);
//!
//!     for (idx, data) in tree.iter_mut() {
//!         *data += 10;
//!     }
//!
//!     assert_eq!(tree.get(root), Some(&10));
//!     assert_eq!(tree.get(child1), Some(&11));
//!     assert_eq!(tree.get(child2), Some(&12));
//! }
//! ```
//!
//! ## 4. Parallel Iteration (Optional)
//!
//! Use the `rayon` feature for parallel processing of nodes.
//!
//! ```rust
//! #[cfg(feature = "rayon")]
//! use easy_tree::Tree;
//! #[cfg(feature = "rayon")]
//! use rayon::prelude::*;
//!
//! #[cfg(feature = "rayon")]
//! fn main() {
//!     let mut tree = Tree::new();
//!     let root = tree.add_node(0);
//!     tree.add_child(root, 1);
//!     tree.add_child(root, 2);
//!
//!     tree.par_iter().for_each(|(idx, data)| {
//!         println!("Processing node {}: {}", idx, data);
//!     });
//! }
//!
//! #[cfg(not(feature = "rayon"))]
//! fn main() {}
//! ```
//!
//! ## API Overview
//!
//! - `Tree<T>`: Represents the tree structure containing nodes of type `T`.
//! - `Node<T>`: Represents a single node in the tree.
//! - `Tree::add_node(data: T) -> usize`: Adds a new root node.
//! - `Tree::add_child(parent: usize, data: T) -> usize`: Adds a child node to a parent.
//! - `Tree::traverse`: Walks the tree recursively with customizable callbacks.
//! - `Tree::iter` / `Tree::iter_mut`: Provides immutable and mutable iterators over the nodes.
//!
//! ## Contributing
//! Contributions are welcome! For more details, see the [GitHub repository](https://github.com/antouhou/easy-tree).
//!
//! ## License
//! This project is licensed under the MIT License. See [LICENSE](https://github.com/antouhou/easy-tree/blob/main/LICENSE) for details.

#[cfg(feature = "rayon")]
pub use rayon;
#[cfg(feature = "rayon")]
use rayon::prelude::*;

/// Represents a single node in a tree structure.
///
/// Each node contains:
/// - **data**: A payload of generic type `T`.
/// - **children**: A list of indices referring to its child nodes.
/// - **parent**: An optional index referring to its parent node (or `None` if the node is a root).
///
/// Normally, you should use the `Tree::add_node` and
/// `Tree::add_child` methods to create nodes and add them to the tree. There's no need to
/// address `Node` directly in most cases.
#[derive(Clone)]
pub struct Node<T> {
    data: T,
    children: Vec<usize>,
    parent: Option<usize>,
}

impl<T> Node<T> {
    /// Creates a new node with the given data. Normally, you should use the `Tree::add_node` and
    /// `Tree::add_child` methods to create nodes and add them to the tree. There's no need to
    /// address `Node` directly in most cases.
    ///
    /// # Parameters
    /// - `data`: The data to associate with this node.
    ///
    /// # Returns
    /// A new `Node` instance.
    ///
    /// # Example
    /// ```
    /// use easy_tree::Node;
    ///
    /// let node = Node::new("example");
    /// ```
    pub fn new(data: T) -> Self {
        Self {
            data,
            children: Vec::new(),
            parent: None,
        }
    }

    /// Adds a child to this node.
    ///
    /// # Parameters
    /// - `child`: The index of the child node to add.
    ///
    /// # Internal Use
    /// This method is used internally by the `Tree` struct.
    pub(crate) fn add_child(&mut self, child: usize) {
        self.children.push(child);
    }

    /// Sets the parent for this node.
    ///
    /// # Parameters
    /// - `parent`: The index of the parent node to set.
    ///
    /// # Internal Use
    /// This method is used internally by the `Tree` struct.
    pub(crate) fn set_parent(&mut self, parent: usize) {
        self.parent = Some(parent);
    }
}

/// A tree structure containing multiple nodes of generic type `T`.
///
/// Each node in the tree is indexed by its position in the internal vector.
/// The tree supports operations for adding, accessing, and traversing nodes.
///
/// # Example
/// ```rust
/// use easy_tree::Tree;
///
/// let mut tree = Tree::new();
/// let root = tree.add_node("root");
/// let child = tree.add_child(root, "child");
/// ```
#[derive(Clone)]
pub struct Tree<T> {
    nodes: Vec<Node<T>>,
}

impl<T> Default for Tree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Tree<T> {
    /// Creates a new, empty tree.
    ///
    /// # Returns
    /// A `Tree` instance with no nodes.
    ///
    /// # Example
    /// ```rust
    /// use easy_tree::Tree;
    ///
    /// let tree: Tree<i32> = Tree::new();
    /// ```
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Adds a new node to the tree.
    ///
    /// This method is typically used to add a root node or a disconnected node.
    ///
    /// # Parameters
    /// - `data`: The data to associate with the new node.
    ///
    /// # Returns
    /// The index of the newly added node.
    ///
    /// # Example
    /// ```rust
    /// use easy_tree::Tree;
    ///
    /// let mut tree = Tree::new();
    /// let root = tree.add_node("root");
    /// ```
    pub fn add_node(&mut self, data: T) -> usize {
        let node = Node::new(data);
        let index = self.nodes.len();
        self.nodes.push(node);
        index
    }

    /// Adds a child node to an existing node in the tree.
    ///
    /// # Parameters
    /// - `parent`: The index of the parent node.
    /// - `data`: The data to associate with the new child node.
    ///
    /// # Returns
    /// The index of the newly added child node.
    ///
    /// # Example
    /// ```rust
    /// use easy_tree::Tree;
    ///
    /// let mut tree = Tree::new();
    /// let root = tree.add_node("root");
    /// let child = tree.add_child(root, "child");
    /// ```
    pub fn add_child(&mut self, parent: usize, data: T) -> usize {
        let index = self.add_node(data);
        self.nodes[parent].add_child(index);
        self.nodes[index].set_parent(parent);
        index
    }

    /// Adds a child node to the tree root.
    ///
    /// # Parameters
    /// - `data`: The data to associate with the new child node.
    ///
    /// # Returns
    /// The index of the newly added child node.
    ///
    /// # Example
    /// ```rust
    /// use easy_tree::Tree;
    ///
    /// let mut tree = Tree::new();
    /// let root = tree.add_node("root");
    /// let child = tree.add_child_to_root("child");
    /// ```
    pub fn add_child_to_root(&mut self, data: T) -> usize {
        self.add_child(0, data)
    }

    /// Retrieves a reference to the data stored in a node.
    ///
    /// # Parameters
    /// - `index`: The index of the node to access.
    ///
    /// # Returns
    /// `Some(&T)` if the node exists, or `None` if the index is out of bounds.
    ///
    /// # Example
    /// ```rust
    /// use easy_tree::Tree;
    ///
    /// let mut tree = Tree::new();
    /// let root = tree.add_node(42);
    /// assert_eq!(tree.get(root), Some(&42));
    /// ```
    pub fn get(&self, index: usize) -> Option<&T> {
        self.nodes.get(index).map(|node| &node.data)
    }

    /// Retrieves a reference to the data stored in a node without bounds checking.
    ///
    /// This method is faster than [`Tree::get`] because it does not perform any bounds checking.
    /// However, it is unsafe to use if the provided index is out of bounds or invalid.
    ///
    /// # Parameters
    /// - `index`: The index of the node to access.
    ///
    /// # Returns
    /// A reference to the data stored in the node.
    ///
    /// # Safety
    /// Ensure that:
    /// - The `index` is within the valid range of node indices in the tree (0 to `Tree::len() - 1`).
    /// - The node at the given index exists and has not been removed (if applicable).
    ///
    /// # Example
    /// ```rust
    /// use easy_tree::Tree;
    ///
    /// let mut tree = Tree::new();
    /// let root = tree.add_node(42);
    ///
    /// // Safe use: The index is valid.
    /// assert_eq!(tree.get_unchecked(root), &42);
    ///
    /// // Unsafe use: Accessing an invalid index would cause undefined behavior.
    /// // let invalid = tree.get_unchecked(999); // Avoid this!
    /// ```
    #[inline(always)]
    pub fn get_unchecked(&self, index: usize) -> &T {
        &self.nodes[index].data
    }

    /// Retrieves a mutable reference to the data stored in a node.
    ///
    /// # Parameters
    /// - `index`: The index of the node to access.
    ///
    /// # Returns
    /// `Some(&mut T)` if the node exists, or `None` if the index is out of bounds.
    ///
    /// # Example
    /// ```rust
    /// use easy_tree::Tree;
    ///
    /// let mut tree = Tree::new();
    /// let root = tree.add_node(42);
    /// *tree.get_mut(root).unwrap() = 43;
    /// assert_eq!(tree.get(root), Some(&43));
    /// ```
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.nodes.get_mut(index).map(|node| &mut node.data)
    }

    /// Retrieves a mutable reference to the data stored in a node without bounds checking.
    ///
    /// This method is faster than [`Tree::get_mut`] because it does not perform any bounds checking.
    /// However, it is unsafe to use if the provided index is out of bounds or invalid.
    ///
    /// # Parameters
    /// - `index`: The index of the node to access.
    ///
    /// # Returns
    /// A mutable reference to the data stored in the node.
    ///
    /// # Safety
    /// Ensure that:
    /// - The `index` is within the valid range of node indices in the tree (0 to `Tree::len() - 1`).
    /// - The node at the given index exists and has not been removed (if applicable).
    /// - No other references to the same node are active during this call, to avoid data races or aliasing violations.
    ///
    /// # Example
    /// ```rust
    /// use easy_tree::Tree;
    ///
    /// let mut tree = Tree::new();
    /// let root = tree.add_node(42);
    ///
    /// // Safe use: The index is valid.
    /// *tree.get_unchecked_mut(root) = 99;
    /// assert_eq!(tree.get_unchecked(root), &99);
    ///
    /// // Unsafe use: Accessing an invalid index would cause undefined behavior.
    /// // let invalid = tree.get_unchecked_mut(999); // Avoid this!
    /// ```
    #[inline(always)]
    pub fn get_unchecked_mut(&mut self, index: usize) -> &mut T {
        &mut self.nodes[index].data
    }


    /// Returns the parent index of a node, if it has a parent.
    ///
    /// # Parameters
    /// - `index`: The index of the node.
    ///
    /// # Returns
    /// `Some(parent_index)` if the node has a parent, or `None` otherwise.
    ///
    /// # Panics
    /// This method panics if the index is out of bounds.
    ///
    /// # Example
    /// ```rust
    /// use easy_tree::Tree;
    ///
    /// let mut tree = Tree::new();
    /// let root = tree.add_node(42);
    /// let child = tree.add_child(root, 99);
    /// assert_eq!(tree.parent_index_unchecked(child), Some(root));
    /// ```
    pub fn parent_index_unchecked(&self, index: usize) -> Option<usize> {
        self.nodes[index].parent
    }

    /// Returns a slice of the indices of the children of a node.
    ///
    /// # Parameters
    /// - `index`: The index of the node.
    ///
    /// # Returns
    /// A slice containing the indices of the node's children.
    ///
    /// # Panics
    /// This method panics if the index is out of bounds.
    ///
    /// # Example
    /// ```rust
    /// use easy_tree::Tree;
    ///
    /// let mut tree = Tree::new();
    /// let root = tree.add_node("root");
    /// let child = tree.add_child(root, "child");
    /// assert_eq!(tree.children(root), &[child]);
    /// ```
    pub fn children(&self, index: usize) -> &[usize] {
        &self.nodes[index].children
    }

    /// Traverses the tree in a depth-first manner.
    ///
    /// The traversal applies two callbacks:
    /// - `before_processing_children`: Called before processing the children of a node.
    /// - `after_processing_the_subtree`: Called after processing all children of a node.
    ///
    /// # Parameters
    /// - `before_processing_children`: A function to apply before visiting children.
    /// - `after_processing_the_subtree`: A function to apply after visiting children.
    /// - `state`: Mutable state to share across callbacks.
    ///
    /// # Example
    /// ```rust
    /// use easy_tree::Tree;
    ///
    /// let mut tree = Tree::new();
    /// let root = tree.add_node("root");
    /// let child = tree.add_child(root, "child");
    ///
    /// let mut log = vec![];
    /// tree.traverse(
    ///     |idx, data, log| log.push(format!("Entering node {}: {}", idx, data)),
    ///     |idx, data, log| log.push(format!("Leaving node {}: {}", idx, data)),
    ///     &mut log,
    /// );
    /// ```
    pub fn traverse<'a, S>(
        &'a self,
        mut before_processing_children: impl FnMut(usize, &'a T, &mut S),
        mut after_processing_the_subtree: impl FnMut(usize, &'a T, &mut S),
        s: &mut S,
    ) {
        if self.is_empty() {
            return;
        }

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
