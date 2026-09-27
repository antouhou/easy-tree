use super::Tree;

impl<T> Tree<T> {
    /// Traverses the tree in depth-first order, with callbacks before and after
    /// each node's children.
    ///
    /// Calls `before_processing_children` on entry and `after_processing_the_subtree`
    /// on exit. Both callbacks share `state`. Visits children in insertion order.vv
    ///
    /// Starts at index zero and does nothing if that slot has no node. Other
    /// disconnected roots are not visited.
    ///
    /// # Example
    ///
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
    pub fn traverse<S>(
        &self,
        mut before_processing_children: impl FnMut(usize, &T, &mut S),
        mut after_processing_the_subtree: impl FnMut(usize, &T, &mut S),
        state: &mut S,
    ) {
        if !matches!(self.nodes.first(), Some(Some(_))) {
            return;
        }

        let mut stack = vec![(0, false)];

        while let Some((index, children_visited)) = stack.pop() {
            let node = self.nodes[index].as_ref().unwrap();
            if children_visited {
                after_processing_the_subtree(index, &node.data, state);
            } else {
                before_processing_children(index, &node.data, state);

                stack.push((index, true));

                // Reverse the push order to visit siblings in insertion order.
                for &child in node.children.iter().rev() {
                    stack.push((child, false));
                }
            }
        }
    }

    /// Traverses a subtree with callbacks that can modify node data before and
    /// after each node's children.
    ///
    /// Calls `before_processing_children` on entry and `after_processing_the_subtree`
    /// on exit. Both callbacks share `state`. Visits `start` and its descendants
    /// in depth-first order, with children in insertion order. Does nothing if
    /// `start` is out of bounds or removed.
    pub fn traverse_subtree_mut<S>(
        &mut self,
        start: usize,
        mut before_processing_children: impl FnMut(usize, &mut T, &mut S),
        mut after_processing_the_subtree: impl FnMut(usize, &mut T, &mut S),
        state: &mut S,
    ) {
        if !matches!(self.nodes.get(start), Some(Some(_))) {
            return;
        }

        self.traversal_stack.clear();
        self.traversal_stack.push((start, false));

        while let Some((index, children_visited)) = self.traversal_stack.pop() {
            if children_visited {
                let node = self.nodes[index].as_mut().unwrap();
                after_processing_the_subtree(index, &mut node.data, state);
            } else {
                let node = self.nodes[index].as_mut().unwrap();
                before_processing_children(index, &mut node.data, state);

                self.traversal_stack.push((index, true));

                // Reverse the push order to visit siblings in insertion order.
                for &child in node.children.iter().rev() {
                    self.traversal_stack.push((child, false));
                }
            }
        }
    }

    /// Traverses the tree with callbacks that can modify node data before and
    /// after each node's children.
    ///
    /// Both callbacks share `state`. Uses the depth-first callback order of
    /// [`Tree::traverse_subtree_mut`], starting at index zero. Does nothing if that
    /// slot has no node. Other disconnected roots are not visited.
    pub fn traverse_mut<S>(
        &mut self,
        before_processing_children: impl FnMut(usize, &mut T, &mut S),
        after_processing_the_subtree: impl FnMut(usize, &mut T, &mut S),
        state: &mut S,
    ) {
        self.traverse_subtree_mut(
            0,
            before_processing_children,
            after_processing_the_subtree,
            state,
        );
    }
}
