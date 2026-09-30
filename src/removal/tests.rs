use crate::Tree;
use std::panic::{self, AssertUnwindSafe};

#[derive(Debug, PartialEq)]
struct RemovedData(i32);

#[test]
fn batch_removal_preserves_surviving_children_and_traversal() {
    let mut tree = Tree::new();
    let root = tree.add_node(0);
    let first = tree.add_child(root, 1);
    let removed_branch = tree.add_child(root, 2);
    let middle = tree.add_child(root, 3);
    let removed_leaf = tree.add_child(root, 4);
    let last = tree.add_child(root, 5);
    let removed_descendant = tree.add_child(removed_branch, 6);
    let middle_child = tree.add_child(middle, 7);
    let last_removed_child = tree.add_child(last, 8);
    let last_surviving_child = tree.add_child(last, 9);

    tree.remove_subtrees([removed_branch, removed_leaf, last_removed_child]);

    assert_eq!(tree.len(), 6);
    assert_eq!(tree.children(root), &[first, middle, last]);
    assert_eq!(tree.children(middle), &[middle_child]);
    assert_eq!(tree.children(last), &[last_surviving_child]);
    for index in [
        removed_branch,
        removed_leaf,
        removed_descendant,
        last_removed_child,
    ] {
        assert_eq!(tree.get(index), None);
    }
    assert_eq!(tree.parent_index_unchecked(first), Some(root));
    assert_eq!(tree.parent_index_unchecked(middle), Some(root));
    assert_eq!(tree.parent_index_unchecked(last), Some(root));
    assert_eq!(
        tree.parent_index_unchecked(last_surviving_child),
        Some(last)
    );

    let mut visited = Vec::new();
    tree.traverse(
        |index, _, visited| visited.push(index),
        |_, _, _| {},
        &mut visited,
    );
    assert_eq!(
        visited,
        [
            root,
            first,
            middle,
            middle_child,
            last,
            last_surviving_child
        ]
    );
}

#[test]
fn batch_removal_skips_overlapping_requests_and_reuses_owned_data() {
    for descendant_first in [false, true] {
        let mut tree = Tree::new();
        let root = tree.add_node(RemovedData(0));
        let branch = tree.add_child(root, RemovedData(1));
        let first_child = tree.add_child(branch, RemovedData(2));
        let second_child = tree.add_child(branch, RemovedData(3));
        let grandchild = tree.add_child(second_child, RemovedData(4));
        let sibling = tree.add_child(root, RemovedData(5));
        let requests = if descendant_first {
            [second_child, branch, grandchild, branch, second_child]
        } else {
            [branch, second_child, grandchild, branch, second_child]
        };
        let expected = if descendant_first {
            [
                (second_child, 3),
                (grandchild, 4),
                (branch, 1),
                (first_child, 2),
            ]
        } else {
            [
                (branch, 1),
                (second_child, 3),
                (grandchild, 4),
                (first_child, 2),
            ]
        };

        let mut removed = Vec::new();
        tree.remove_subtrees_with(requests, |index, data| removed.push((index, data)));

        assert_eq!(removed.len(), expected.len());
        for ((index, data), (expected_index, expected_data)) in removed.iter().zip(expected) {
            assert_eq!((*index, data.0), (expected_index, expected_data));
            assert_eq!(tree.get(*index), None);
        }
        assert_eq!(tree.len(), 2);
        assert_eq!(tree.children(root), &[sibling]);

        for (index, data) in removed.into_iter().rev() {
            assert_eq!(tree.next_node_id(), index);
            assert_eq!(tree.add_child(root, data), index);
            assert_eq!(tree.parent_index_unchecked(index), Some(root));
            assert!(tree.children(index).is_empty());
        }
        assert_eq!(tree.len(), 6);
    }
}

#[test]
fn batch_removal_preserves_request_order_across_roots_and_skips_missing_indices() {
    let mut tree = Tree::new();
    let root = tree.add_node("root");
    let branch = tree.add_child(root, "branch");
    let first_child = tree.add_child(branch, "first child");
    let second_child = tree.add_child(branch, "second child");
    let sibling = tree.add_child(root, "sibling");
    let other_root = tree.add_node("other root");
    let other_child = tree.add_child(other_root, "other child");
    let removed_child = tree.add_child(root, "already removed");
    tree.remove_subtree(removed_child);
    let out_of_bounds = tree.nodes.len();

    let mut removed = Vec::new();
    tree.remove_subtrees_with(
        [
            usize::MAX,
            removed_child,
            other_child,
            out_of_bounds,
            branch,
            other_child,
        ],
        |index, data| removed.push((index, data)),
    );

    assert_eq!(
        removed,
        [
            (other_child, "other child"),
            (branch, "branch"),
            (second_child, "second child"),
            (first_child, "first child")
        ],
    );
    assert_eq!(tree.len(), 3);
    assert_eq!(tree.children(root), &[sibling]);
    assert!(tree.children(other_root).is_empty());

    tree.remove_subtrees_with([], |_, _| panic!("empty batch triggered callback"));
    tree.remove_subtrees_with([branch, other_child, out_of_bounds, usize::MAX], |_, _| {
        panic!("missing node triggered callback")
    });
    assert_eq!(tree.len(), 3);
    assert_eq!(tree.children(root), &[sibling]);
}

#[test]
fn batch_removal_resets_storage_after_removing_the_whole_forest() {
    let mut tree = Tree::new();
    let root = tree.add_node(0);
    let child = tree.add_child(root, 1);
    let other_root = tree.add_node(2);
    let other_child = tree.add_child(other_root, 3);

    let mut removed = Vec::new();
    tree.remove_subtrees_with(
        [other_child, other_root, root, child, root, usize::MAX],
        |index, data| removed.push((index, data)),
    );

    assert_eq!(
        removed,
        [(other_child, 3), (other_root, 2), (root, 0), (child, 1)]
    );
    assert!(tree.is_empty());
    assert!(tree.nodes.is_empty());
    assert!(tree.free_list.is_empty());
    assert!(tree.traversal_stack.is_empty());
    assert_eq!(tree.add_node(4), 0);

    tree.clear();
    tree.remove_subtrees_with([0, usize::MAX], |_, _| {
        panic!("empty tree triggered callback")
    });
    assert_eq!(tree.next_node_id(), 0);
}

#[test]
fn batch_removal_finishes_active_subtree_and_detaches_it_when_callback_panics() {
    let mut tree = Tree::new();
    let root = tree.add_node(0);
    let first_leaf = tree.add_child(root, 1);
    let branch = tree.add_child(root, 2);
    let first_child = tree.add_child(branch, 3);
    let second_child = tree.add_child(branch, 4);
    let grandchild = tree.add_child(first_child, 5);
    let sibling = tree.add_child(root, 6);
    let mut callbacks = Vec::new();

    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        tree.remove_subtrees_with([first_leaf, branch, sibling], |index, _| {
            callbacks.push(index);
            assert_ne!(index, branch, "callback failed");
        });
    }));

    assert!(result.is_err());
    assert_eq!(callbacks, [first_leaf, branch]);
    assert_eq!(tree.len(), 2);
    assert_eq!(tree.children(root), &[sibling]);
    for index in [first_leaf, branch, first_child, second_child, grandchild] {
        assert_eq!(tree.get(index), None);
    }
    assert!(tree.traversal_stack.is_empty());

    let mut visited = Vec::new();
    tree.traverse(
        |index, _, visited| visited.push(index),
        |_, _, _| {},
        &mut visited,
    );
    assert_eq!(visited, [root, sibling]);
    tree.remove_subtrees([sibling]);
    assert_eq!(tree.len(), 1);
    assert!(tree.children(root).is_empty());
    assert_eq!(tree.add_child(root, 7), sibling);
}

#[test]
fn single_subtree_removal_resets_storage_when_descendant_callback_panics() {
    let mut tree = Tree::new();
    let root = tree.add_node(0);
    tree.add_child(root, 1);
    let second_child = tree.add_child(root, 2);
    tree.add_child(second_child, 3);
    let mut callbacks = Vec::new();

    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        tree.remove_subtree_with(root, |index, _| {
            callbacks.push(index);
            assert_ne!(index, second_child, "callback failed");
        });
    }));

    assert!(result.is_err());
    assert_eq!(callbacks, [root, second_child]);
    assert!(tree.is_empty());
    assert!(tree.nodes.is_empty());
    assert!(tree.free_list.is_empty());
    assert!(tree.traversal_stack.is_empty());
    assert_eq!(tree.add_node(4), 0);
}
