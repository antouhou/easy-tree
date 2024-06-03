#[cfg(feature = "rayon")]
pub use rayon;
#[cfg(feature = "rayon")]
use rayon::prelude::*;

#[derive(Clone)]
pub struct Node<T> {
    data: T,
    children: Vec<usize>,
    parent: Option<usize>,
}

impl<T> Node<T> {
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
pub struct Tree<T> {
    nodes: Vec<Node<T>>,
}

impl<T> Default for Tree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Tree<T> {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn add_node(&mut self, data: T) -> usize {
        let node = Node::new(data);
        let index = self.nodes.len();
        self.nodes.push(node);
        index
    }

    /// Add a child to the parent node.
    pub fn add_child(&mut self, parent: usize, data: T) -> usize {
        let index = self.add_node(data);
        self.nodes[parent].add_child(index);
        self.nodes[index].set_parent(parent);
        index
    }

    pub fn add_child_to_root(&mut self, data: T) -> usize {
        let index = self.add_child(0, data);

        index
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.nodes.get(index).map(|node| &node.data)
    }

    #[inline(always)]
    pub fn get_unchecked(&self, index: usize) -> &T {
        &self.nodes[index].data
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.nodes.get_mut(index).map(|node| &mut node.data)
    }

    #[inline(always)]
    pub fn get_unchecked_mut(&mut self, index: usize) -> &mut T {
        &mut self.nodes[index].data
    }

    pub fn parent_index_unchecked(&self, index: usize) -> Option<usize> {
        self.nodes[index].parent
    }

    pub fn children(&self, index: usize) -> &[usize] {
        &self.nodes[index].children
    }

    pub fn walk_recursive<'a, S>(
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

    pub fn iter(&self) -> impl Iterator<Item = (usize, &T)> {
        self.nodes
            .iter()
            .enumerate()
            .map(|(index, node)| (index, &node.data))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (usize, &mut T)> {
        self.nodes
            .iter_mut()
            .enumerate()
            .map(|(index, node)| (index, &mut node.data))
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
    }
}

#[cfg(feature = "rayon")]
impl<T: Send + Sync> Tree<T> {
    #[cfg(feature = "rayon")]
    pub fn par_iter(&self) -> impl ParallelIterator<Item = (usize, &T)> {
        self.nodes
            .par_iter()
            .enumerate()
            .map(|(index, node)| (index, &node.data))
    }

    #[cfg(feature = "rayon")]
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
}
