//! A tree library with depth-first traversal and pre- and post-processing callbacks.
//!
//! [`Tree::traverse`] walks the tree and calls your code before and after each node's
//! children. Both callbacks receive the same mutable state, so they can maintain a
//! path or other context as traversal enters and leaves branches.
//!
//! Use [`Tree::traverse_mut`] to update node data during traversal, or
//! [`Tree::traverse_subtree_mut`] to start from a particular node.
//!
//! ## Create a tree
//!
//! ```rust
//! use easy_tree::Tree;
//!
//! let mut tree = Tree::new();
//! let root = tree.add_node("root");
//! let child1 = tree.add_child(root, "child1");
//! let child2 = tree.add_child(root, "child2");
//! let grandchild = tree.add_child(child1, "grandchild");
//!
//! assert_eq!(tree.get(root), Some(&"root"));
//! assert_eq!(tree.get(grandchild), Some(&"grandchild"));
//! assert_eq!(tree.children(root), &[child1, child2]);
//! assert_eq!(tree.parent_index_unchecked(grandchild), Some(child1));
//! ```
//!
//! ## Traverse nodes
//!
//! Process nodes before and after their children in a single call. This example
//! collects both callbacks' messages in the same log.
//!
//! ```rust
//! use easy_tree::Tree;
//!
//! let mut tree = Tree::new();
//! let root = tree.add_node("root");
//! let child1 = tree.add_child(root, "child1");
//! let child2 = tree.add_child(root, "child2");
//!
//! let mut result = vec![];
//! tree.traverse(
//!     |idx, data, result| result.push(format!("Entering node {}: {}", idx, data)),
//!     |idx, data, result| result.push(format!("Leaving node {}: {}", idx, data)),
//!     &mut result,
//! );
//!
//! assert_eq!(result, vec![
//!     "Entering node 0: root",
//!     "Entering node 1: child1",
//!     "Leaving node 1: child1",
//!     "Entering node 2: child2",
//!     "Leaving node 2: child2",
//!     "Leaving node 0: root",
//! ]);
//! ```
//!
//! [`Tree::traverse`] and [`Tree::traverse_mut`] start at index zero. They do nothing
//! if that slot is empty, even when other disconnected roots exist.
//! [`Tree::traverse_subtree_mut`] starts at a caller-selected index.
//!
//! ## Iterate over node data
//!
//! Iterate over nodes and modify their data.
//!
//! ```rust
//! use easy_tree::Tree;
//!
//! let mut tree = Tree::new();
//! let root = tree.add_node(0);
//! let child1 = tree.add_child(root, 1);
//! let child2 = tree.add_child(root, 2);
//!
//! for (_, data) in tree.iter_mut() {
//!     *data += 10;
//! }
//!
//! assert_eq!(tree.get(root), Some(&10));
//! assert_eq!(tree.get(child1), Some(&11));
//! assert_eq!(tree.get(child2), Some(&12));
//! ```
//!
//! ## Iterate in parallel
//!
//! Use the `rayon` feature for parallel processing of nodes.
//!
//! ```rust
//! #[cfg(feature = "rayon")]
//! use easy_tree::Tree;
//! #[cfg(feature = "rayon")]
//! use easy_tree::rayon::iter::ParallelIterator;
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
//! ## License
//!
//! See the [MIT license](https://github.com/antouhou/easy-tree/blob/main/LICENSE.md).

#[cfg(feature = "rayon")]
pub use rayon;
#[cfg(feature = "rayon")]
use rayon::prelude::*;

mod traversal;

/// A node's data, child indices, and optional parent index.
///
/// Use [`Tree::add_node`] or [`Tree::add_child`] to create nodes in a tree.
#[derive(Clone)]
pub struct Node<T> {
    data: T,
    children: Vec<usize>,
    parent: Option<usize>,
}

impl<T> Node<T> {
    /// Creates a standalone node with no parent or children.
    ///
    /// Use [`Tree::add_node`] to insert data into a tree.
    ///
    /// # Example
    ///
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

    pub(crate) fn add_child(&mut self, child: usize) {
        self.children.push(child);
    }

    pub(crate) fn set_parent(&mut self, parent: usize) {
        self.parent = Some(parent);
    }
}

/// A tree with depth-first traversal and callbacks before and after each node's children.
///
/// [`Tree::traverse`] runs both callbacks in one walk through the tree. They share
/// mutable state, so one can extend a path on entry and the other can shorten it
/// on exit. [`Tree::traverse_mut`] also lets callbacks modify node data, and
/// [`Tree::traverse_subtree_mut`] starts at a chosen node.
///
/// # Example
/// ```rust
/// use easy_tree::Tree;
///
/// let mut tree = Tree::new();
/// let root = tree.add_node("root");
/// let child = tree.add_child(root, "child");
/// ```
///
/// # Node indices
///
/// Insertions can reuse removed indices, so an old index may refer to a different
/// node. [`Tree::len`] counts live nodes rather than the range of valid indices.
#[derive(Clone)]
pub struct Tree<T> {
    nodes: Vec<Option<Node<T>>>,
    /// Indices of removed nodes available for reuse.
    free_list: Vec<usize>,
    /// Number of live nodes.
    node_count: usize,
    /// Retains stack capacity between mutable traversals and subtree removals.
    traversal_stack: Vec<(usize, bool)>,
}

impl<T> Tree<T> {
    /// Creates a new, empty tree.
    ///
    /// # Example
    ///
    /// ```rust
    /// use easy_tree::Tree;
    ///
    /// let tree: Tree<i32> = Tree::new();
    /// ```
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            free_list: Vec::new(),
            node_count: 0,
            traversal_stack: Vec::new(),
        }
    }

    /// Adds a node with no parent and returns its index.
    ///
    /// Reuses a vacant slot when available. The first insertion into an empty
    /// tree uses index zero.
    ///
    /// # Example
    ///
    /// ```rust
    /// use easy_tree::Tree;
    ///
    /// let mut tree = Tree::new();
    /// let root = tree.add_node("root");
    /// ```
    pub fn add_node(&mut self, data: T) -> usize {
        let node = Node::new(data);
        self.node_count += 1;
        if let Some(index) = self.free_list.pop() {
            self.nodes[index] = Some(node);
            index
        } else {
            let index = self.nodes.len();
            self.nodes.push(Some(node));
            index
        }
    }

    /// Returns the index that the next insertion will use.
    pub fn next_node_id(&self) -> usize {
        self.free_list.last().copied().unwrap_or(self.nodes.len())
    }

    /// Adds a child to an existing node and returns the child's index.
    ///
    /// # Panics
    ///
    /// Panics before changing the tree if `parent` is out of bounds or removed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use easy_tree::Tree;
    ///
    /// let mut tree = Tree::new();
    /// let root = tree.add_node("root");
    /// let child = tree.add_child(root, "child");
    /// ```
    pub fn add_child(&mut self, parent: usize, data: T) -> usize {
        assert!(
            matches!(self.nodes.get(parent), Some(Some(_))),
            "parent node does not exist"
        );
        let index = self.add_node(data);
        self.nodes[parent].as_mut().unwrap().add_child(index);
        self.nodes[index].as_mut().unwrap().set_parent(parent);
        index
    }

    /// Adds a child to the node at index zero and returns the child's index.
    ///
    /// # Panics
    ///
    /// Panics before changing the tree if index zero has no node.
    ///
    /// # Example
    ///
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

    /// Returns a node's data, or `None` if the index is out of bounds or removed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use easy_tree::Tree;
    ///
    /// let mut tree = Tree::new();
    /// let root = tree.add_node(42);
    /// assert_eq!(tree.get(root), Some(&42));
    /// ```
    pub fn get(&self, index: usize) -> Option<&T> {
        self.nodes
            .get(index)
            .and_then(|slot| slot.as_ref().map(|node| &node.data))
    }

    /// Returns a node's data, panicking if the node does not exist.
    ///
    /// Use [`Tree::get`] to receive `None` for a missing node.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds or refers to a removed node.
    ///
    /// # Example
    ///
    /// ```rust
    /// use easy_tree::Tree;
    ///
    /// let mut tree = Tree::new();
    /// let root = tree.add_node(42);
    ///
    /// assert_eq!(tree.get_unchecked(root), &42);
    /// ```
    #[inline(always)]
    pub fn get_unchecked(&self, index: usize) -> &T {
        &self.nodes[index].as_ref().unwrap().data
    }

    /// Returns mutable node data, or `None` if the index is out of bounds or removed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use easy_tree::Tree;
    ///
    /// let mut tree = Tree::new();
    /// let root = tree.add_node(42);
    /// *tree.get_mut(root).unwrap() = 43;
    /// assert_eq!(tree.get(root), Some(&43));
    /// ```
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.nodes
            .get_mut(index)
            .and_then(|slot| slot.as_mut().map(|node| &mut node.data))
    }

    /// Returns mutable node data, panicking if the node does not exist.
    ///
    /// Use [`Tree::get_mut`] to receive `None` for a missing node.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds or refers to a removed node.
    ///
    /// # Example
    ///
    /// ```rust
    /// use easy_tree::Tree;
    ///
    /// let mut tree = Tree::new();
    /// let root = tree.add_node(42);
    ///
    /// *tree.get_unchecked_mut(root) = 99;
    /// assert_eq!(tree.get_unchecked(root), &99);
    /// ```
    #[inline(always)]
    pub fn get_unchecked_mut(&mut self, index: usize) -> &mut T {
        &mut self.nodes[index].as_mut().unwrap().data
    }

    /// Returns the parent index of a node, if it has a parent.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds or refers to a removed node.
    ///
    /// # Example
    ///
    /// ```rust
    /// use easy_tree::Tree;
    ///
    /// let mut tree = Tree::new();
    /// let root = tree.add_node(42);
    /// let child = tree.add_child(root, 99);
    /// assert_eq!(tree.parent_index_unchecked(child), Some(root));
    /// ```
    pub fn parent_index_unchecked(&self, index: usize) -> Option<usize> {
        self.nodes[index].as_ref().unwrap().parent
    }

    /// Returns a node's child indices in insertion order.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds or refers to a removed node.
    ///
    /// # Example
    ///
    /// ```rust
    /// use easy_tree::Tree;
    ///
    /// let mut tree = Tree::new();
    /// let root = tree.add_node("root");
    /// let child = tree.add_child(root, "child");
    /// assert_eq!(tree.children(root), &[child]);
    /// ```
    pub fn children(&self, index: usize) -> &[usize] {
        &self.nodes[index].as_ref().unwrap().children
    }

    /// Iterates over live node indices and data in index order.
    pub fn iter(&self) -> impl Iterator<Item = (usize, &T)> {
        self.nodes
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| slot.as_ref().map(|node| (index, &node.data)))
    }

    /// Iterates over live node indices and mutable data in index order.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (usize, &mut T)> {
        self.nodes
            .iter_mut()
            .enumerate()
            .filter_map(|(index, slot)| slot.as_mut().map(|node| (index, &mut node.data)))
    }

    /// Returns `true` if the tree contains no nodes.
    pub fn is_empty(&self) -> bool {
        self.node_count == 0
    }

    /// Returns the number of live nodes, excluding removed slots.
    pub fn len(&self) -> usize {
        self.node_count
    }

    /// Removes all nodes from the tree.
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.free_list.clear();
        self.node_count = 0;
    }

    /// Removes a node and all of its descendants from the tree.
    ///
    /// Detaches the node from its parent and makes the removed indices available
    /// for reuse by [`Tree::add_node`] and [`Tree::add_child`].
    /// Use [`Tree::remove_subtree_with`] to receive the removed indices and data.
    ///
    /// If `index` is out of bounds or refers to a previously removed node, this method
    /// is a no-op.
    ///
    /// # Example
    ///
    /// ```rust
    /// use easy_tree::Tree;
    ///
    /// let mut tree = Tree::new();
    /// let root = tree.add_node("root");
    /// let child1 = tree.add_child(root, "child1");
    /// let child2 = tree.add_child(root, "child2");
    /// let grandchild = tree.add_child(child1, "grandchild");
    ///
    /// assert_eq!(tree.len(), 4);
    ///
    /// tree.remove_subtree(child1);
    ///
    /// assert_eq!(tree.len(), 2);
    /// assert_eq!(tree.get(child1), None);
    /// assert_eq!(tree.get(grandchild), None);
    /// assert_eq!(tree.children(root), &[child2]);
    /// ```
    pub fn remove_subtree(&mut self, index: usize) {
        self.remove_subtree_with(index, |_, _| {});
    }

    /// Removes a node and its descendants, passing each index and owned data to a callback.
    ///
    /// Calls `on_remove` once per removed node, starting with `index`, then visiting
    /// descendants depth-first with siblings in reverse insertion order.
    /// Detaches the subtree from its parent and makes removed indices available
    /// for reuse, as with [`Tree::remove_subtree`].
    ///
    /// If `index` is out of bounds or already removed, does nothing and never
    /// calls `on_remove`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use easy_tree::Tree;
    ///
    /// let mut tree = Tree::new();
    /// let root = tree.add_node(String::from("root"));
    /// let child = tree.add_child(root, String::from("child"));
    /// let grandchild = tree.add_child(child, String::from("grandchild"));
    ///
    /// let mut removed = Vec::new();
    /// tree.remove_subtree_with(child, |index, item| removed.push((index, item)));
    ///
    /// assert_eq!(removed, vec![
    ///     (child, String::from("child")),
    ///     (grandchild, String::from("grandchild")),
    /// ]);
    /// assert_eq!(tree.len(), 1);
    /// assert!(tree.children(root).is_empty());
    /// ```
    pub fn remove_subtree_with(&mut self, index: usize, mut on_remove: impl FnMut(usize, T)) {
        if !matches!(self.nodes.get(index), Some(Some(_))) {
            return;
        }

        if let Some(parent_idx) = self.nodes[index].as_ref().unwrap().parent
            && let Some(parent) = self.nodes[parent_idx].as_mut()
        {
            parent.children.retain(|&child| child != index);
        }

        self.traversal_stack.clear();
        self.traversal_stack.push((index, false));
        while let Some((current, _)) = self.traversal_stack.pop() {
            if let Some(node) = self.nodes[current].take() {
                self.traversal_stack
                    .extend(node.children.into_iter().map(|child| (child, false)));
                self.free_list.push(current);
                self.node_count -= 1;
                on_remove(current, node.data);
            }
        }

        // Reset storage when the tree becomes empty, so the next add_node
        // starts fresh from index 0.
        if self.node_count == 0 {
            self.nodes.clear();
            self.free_list.clear();
        }
    }
}

impl<T> Default for Tree<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "rayon")]
impl<T: Send + Sync> Tree<T> {
    /// Returns a parallel iterator over the indices and data of the nodes in the tree.
    pub fn par_iter(&self) -> impl ParallelIterator<Item = (usize, &T)> {
        self.nodes
            .par_iter()
            .enumerate()
            .filter_map(|(index, slot)| slot.as_ref().map(|node| (index, &node.data)))
    }

    /// Returns a mutable parallel iterator over the indices and data of the nodes in the tree.
    pub fn par_iter_mut(&mut self) -> impl ParallelIterator<Item = (usize, &mut T)> {
        self.nodes
            .par_iter_mut()
            .enumerate()
            .filter_map(|(index, slot)| slot.as_mut().map(|node| (index, &mut node.data)))
    }
}

#[cfg(test)]
mod tests;
