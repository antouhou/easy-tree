use crate::Tree;
use std::mem;

/// Finishes the active subtree and repairs parent links when removal ends or unwinds.
struct SubtreeRemoval<'a, T> {
    tree: &'a mut Tree<T>,
}

impl<'a, T> SubtreeRemoval<'a, T> {
    fn new(tree: &'a mut Tree<T>) -> Self {
        tree.traversal_stack.clear();
        tree.affected_parents.clear();
        Self { tree }
    }

    fn remove_next_node(&mut self) -> Option<(usize, T)> {
        while let Some((index, _)) = self.tree.traversal_stack.pop() {
            if let Some(node) = self.tree.nodes[index].take() {
                self.tree
                    .traversal_stack
                    .extend(node.children.into_iter().map(|child| (child, false)));
                self.tree.free_list.push(index);
                self.tree.node_count -= 1;
                return Some((index, node.data));
            }
        }
        None
    }

    fn remove_subtree_with(&mut self, index: usize, on_remove: &mut impl FnMut(usize, T)) {
        self.tree.traversal_stack.push((index, false));
        while let Some((index, data)) = self.remove_next_node() {
            on_remove(index, data);
        }
    }
}

impl<T> Drop for SubtreeRemoval<'_, T> {
    fn drop(&mut self) {
        while let Some((_, data)) = self.remove_next_node() {
            drop(data);
        }

        for parent_index in self.tree.affected_parents.drain() {
            let Some(parent) = self.tree.nodes[parent_index].as_mut() else {
                continue;
            };
            // Keep the child vector's allocation while checking the other node slots.
            let mut children = mem::take(&mut parent.children);
            children.retain(|&child| self.tree.nodes[child].is_some());
            self.tree.nodes[parent_index].as_mut().unwrap().children = children;
        }

        if self.tree.node_count == 0 {
            self.tree.nodes.clear();
            self.tree.free_list.clear();
        }
    }
}

impl<T> Tree<T> {
    /// Removes a node and all of its descendants from the tree.
    ///
    /// Detaches the node from its parent and makes the removed indices available
    /// for reuse by [`Tree::add_node`] and [`Tree::add_child`].
    /// Use [`Tree::remove_subtree_with`] to receive the removed indices and data,
    /// or [`Tree::remove_subtrees`] to remove several subtrees in one batch.
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
    /// # Panics
    ///
    /// If `on_remove` panics, finishes removing the subtree without further
    /// callbacks and repairs its parent links before continuing to unwind.
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
        let Some(node) = self.nodes.get(index).and_then(|slot| slot.as_ref()) else {
            return;
        };
        if let Some(parent_index) = node.parent
            && let Some(parent) = self.nodes[parent_index].as_mut()
        {
            parent.children.retain(|&child| child != index);
        }

        let mut removal = SubtreeRemoval::new(self);
        removal.remove_subtree_with(index, &mut on_remove);
    }

    /// Removes several subtrees in one call.
    ///
    /// Preserves surviving siblings' order. Missing indices and duplicate or
    /// overlapping requests never cause a node to be removed twice. Removed
    /// indices become available for reuse, as with [`Tree::remove_subtree`].
    /// Use [`Tree::remove_subtrees_with`] to receive the removed indices and data.
    ///
    /// # Example
    ///
    /// ```rust
    /// use easy_tree::Tree;
    ///
    /// let mut tree = Tree::new();
    /// let root = tree.add_node("root");
    /// let first = tree.add_child(root, "first");
    /// let kept = tree.add_child(root, "kept");
    /// let last = tree.add_child(root, "last");
    ///
    /// tree.remove_subtrees([first, last]);
    ///
    /// assert_eq!(tree.len(), 2);
    /// assert_eq!(tree.children(root), &[kept]);
    /// ```
    pub fn remove_subtrees(&mut self, indices: impl IntoIterator<Item = usize>) {
        self.remove_subtrees_with(indices, |_, _| {});
    }

    /// Removes several subtrees, passing each removed index and owned data to a callback.
    ///
    /// Processes requests in iterator order. For each live requested root, calls
    /// `on_remove` for the root, then visits descendants depth-first with siblings
    /// in reverse insertion order. Skips out-of-bounds and already removed indices.
    /// Duplicate and overlapping requests call `on_remove` once per removed node.
    ///
    /// Preserves surviving siblings' order. Removed indices become available for
    /// reuse, as with [`Tree::remove_subtree`].
    ///
    /// # Panics
    ///
    /// If `on_remove` panics, finishes the current subtree without further callbacks
    /// and repairs affected parent links before continuing to unwind. Later requests
    /// are not processed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use easy_tree::Tree;
    ///
    /// let mut tree = Tree::new();
    /// let root = tree.add_node(String::from("root"));
    /// let first = tree.add_child(root, String::from("first"));
    /// let second = tree.add_child(root, String::from("second"));
    /// let descendant = tree.add_child(first, String::from("descendant"));
    ///
    /// let mut removed = Vec::new();
    /// tree.remove_subtrees_with([second, first, descendant], |index, data| {
    ///     removed.push((index, data));
    /// });
    ///
    /// assert_eq!(removed, vec![
    ///     (second, String::from("second")),
    ///     (first, String::from("first")),
    ///     (descendant, String::from("descendant")),
    /// ]);
    /// assert_eq!(tree.len(), 1);
    /// assert!(tree.children(root).is_empty());
    /// ```
    pub fn remove_subtrees_with(
        &mut self,
        indices: impl IntoIterator<Item = usize>,
        mut on_remove: impl FnMut(usize, T),
    ) {
        let mut removal = SubtreeRemoval::new(self);
        for index in indices {
            let Some(node) = removal.tree.nodes.get(index).and_then(|slot| slot.as_ref()) else {
                continue;
            };
            if let Some(parent) = node.parent {
                removal.tree.affected_parents.insert(parent);
            }
            removal.remove_subtree_with(index, &mut on_remove);
        }
    }
}

#[cfg(test)]
mod tests;
