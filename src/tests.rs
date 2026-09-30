use crate::Tree;
use std::panic::{self, AssertUnwindSafe};

#[derive(Debug, PartialEq)]
struct NonCloneData(i32);

#[test]
fn test_replace_preserves_tree_structure_and_traversal() {
    let mut tree = Tree::new();
    let root = tree.add_node(NonCloneData(0));
    let branch = tree.add_child(root, NonCloneData(1));
    let sibling = tree.add_child(root, NonCloneData(2));
    let first_child = tree.add_child(branch, NonCloneData(3));
    let second_child = tree.add_child(branch, NonCloneData(4));
    let grandchild = tree.add_child(first_child, NonCloneData(5));
    let removed_child = tree.add_child(branch, NonCloneData(6));
    tree.remove_subtree(removed_child);

    assert_eq!(
        tree.replace(branch, NonCloneData(10)),
        Some(NonCloneData(1))
    );
    assert_eq!(tree.replace(root, NonCloneData(20)), Some(NonCloneData(0)));

    assert_eq!(tree.len(), 6);
    assert_eq!(tree.next_node_id(), removed_child);
    assert_eq!(tree.parent_index_unchecked(root), None);
    assert_eq!(tree.children(root), &[branch, sibling]);
    assert_eq!(tree.parent_index_unchecked(branch), Some(root));
    assert_eq!(tree.children(branch), &[first_child, second_child]);
    assert_eq!(tree.parent_index_unchecked(first_child), Some(branch));
    assert_eq!(tree.parent_index_unchecked(second_child), Some(branch));
    assert_eq!(tree.children(first_child), &[grandchild]);
    assert_eq!(tree.parent_index_unchecked(grandchild), Some(first_child));
    assert_eq!(tree.get(removed_child), None);

    let mut visited = Vec::new();
    tree.traverse(
        |index, data, visited| visited.push((index, data.0)),
        |_, _, _| {},
        &mut visited,
    );

    assert_eq!(
        visited,
        vec![
            (root, 20),
            (branch, 10),
            (first_child, 3),
            (grandchild, 5),
            (second_child, 4),
            (sibling, 2),
        ]
    );
}

#[test]
fn test_replace_missing_indices_preserves_tree_and_vacant_slots() {
    let mut tree = Tree::new();
    assert_eq!(tree.replace(0, "replacement"), None);
    assert!(tree.is_empty());
    assert_eq!(tree.next_node_id(), 0);

    let root = tree.add_node("root");
    let removed_child = tree.add_child(root, "removed");
    let surviving_child = tree.add_child(root, "surviving");
    tree.remove_subtree(removed_child);

    for index in [removed_child, tree.nodes.len(), usize::MAX] {
        assert_eq!(tree.replace(index, "replacement"), None);
    }

    assert_eq!(
        tree.iter().collect::<Vec<_>>(),
        vec![(root, &"root"), (surviving_child, &"surviving")]
    );
    assert_eq!(tree.len(), 2);
    assert_eq!(tree.children(root), &[surviving_child]);
    assert_eq!(tree.parent_index_unchecked(surviving_child), Some(root));
    assert_eq!(tree.next_node_id(), removed_child);
    assert_eq!(tree.add_child(root, "new child"), removed_child);
    assert_eq!(tree.children(root), &[surviving_child, removed_child]);
}

#[test]
fn test_add_child_rejects_removed_parent_without_mutating_tree() {
    let mut tree = Tree::new();
    let root = tree.add_node("root");
    let removed_child = tree.add_child(root, "removed");
    let surviving_child = tree.add_child(root, "surviving");
    tree.remove_subtree(removed_child);

    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        tree.add_child(removed_child, "replacement");
    }));

    assert!(result.is_err());
    assert_eq!(tree.len(), 2);
    assert_eq!(tree.get(removed_child), None);
    assert_eq!(tree.children(root), &[surviving_child]);
    assert_eq!(tree.next_node_id(), removed_child);
    assert_eq!(tree.add_child(root, "replacement"), removed_child);
    assert_eq!(tree.parent_index_unchecked(removed_child), Some(root));
    assert!(tree.children(removed_child).is_empty());
}

#[test]
fn test_add_child_rejects_out_of_bounds_parent_without_mutating_tree() {
    let mut original = Tree::new();
    let root = original.add_node("root");

    for invalid_parent in [original.next_node_id(), usize::MAX] {
        let mut tree = original.clone();
        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            tree.add_child(invalid_parent, "child");
        }));

        assert!(result.is_err());
        assert_eq!(tree.iter().collect::<Vec<_>>(), vec![(root, &"root")]);
        assert_eq!(tree.len(), 1);
        assert!(tree.children(root).is_empty());
        assert_eq!(tree.next_node_id(), original.next_node_id());
    }
}

#[test]
fn test_add_child_to_root_rejects_empty_tree() {
    let mut tree = Tree::new();
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        tree.add_child_to_root("child");
    }));

    assert!(result.is_err());
    assert!(tree.is_empty());
    assert_eq!(tree.iter().next(), None);
    assert_eq!(tree.next_node_id(), 0);
}

#[test]
fn test_parent_and_child_relationships() {
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
fn test_mutable_traversal_preserves_order_after_removal() {
    let mut tree = Tree::new();
    let root = tree.add_node(0);
    let branch = tree.add_child(root, 1);
    let removed_child = tree.add_child(branch, 2);
    let grandchild = tree.add_child(branch, 3);
    let sibling = tree.add_child(root, 4);
    tree.remove_subtree(removed_child);

    let mut visits = Vec::new();
    tree.traverse_mut(
        |index, data, visits| {
            *data += 1;
            visits.push(("enter", index, *data));
        },
        |index, data, visits| {
            *data *= 2;
            visits.push(("leave", index, *data));
        },
        &mut visits,
    );

    assert_eq!(
        visits,
        vec![
            ("enter", root, 1),
            ("enter", branch, 2),
            ("enter", grandchild, 4),
            ("leave", grandchild, 8),
            ("leave", branch, 4),
            ("enter", sibling, 5),
            ("leave", sibling, 10),
            ("leave", root, 2),
        ]
    );

    visits.clear();
    tree.traverse_subtree_mut(
        branch,
        |index, data, visits| {
            *data += 10;
            visits.push(("enter", index, *data));
        },
        |index, data, visits| visits.push(("leave", index, *data)),
        &mut visits,
    );

    assert_eq!(
        visits,
        vec![
            ("enter", branch, 14),
            ("enter", grandchild, 18),
            ("leave", grandchild, 18),
            ("leave", branch, 14),
        ]
    );
    assert_eq!(tree.get(root), Some(&2));
    assert_eq!(tree.get(sibling), Some(&10));
    assert_eq!(tree.get(removed_child), None);
}

#[test]
fn test_tree_traverse() {
    let mut tree = Tree::new();
    let root = tree.add_node(0);
    let child1 = tree.add_child(root, 1);
    let _child2 = tree.add_child(root, 2);
    let _child3 = tree.add_child(child1, 3);

    let mut result = vec![];

    tree.traverse(
        |index, node, result| result.push(format!("Calling handler for node {index}: {node}")),
        |index, _node, result| {
            result.push(format!(
                "Finished handling node {index} and all its children"
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
            "Finished handling node 3 and all its children",
            "Finished handling node 1 and all its children",
            "Calling handler for node 2: 2",
            "Finished handling node 2 and all its children",
            "Finished handling node 0 and all its children",
        ]
    );
}

#[test]
fn test_remove_subtree() {
    let mut tree = Tree::new();
    let root = tree.add_node("root");
    let child1 = tree.add_child(root, "child1");
    let child2 = tree.add_child(root, "child2");
    let grandchild1 = tree.add_child(child1, "grandchild1");
    let _grandchild2 = tree.add_child(child1, "grandchild2");

    assert_eq!(tree.len(), 5);

    tree.remove_subtree(child1);

    assert_eq!(tree.len(), 2);
    assert_eq!(tree.get(root), Some(&"root"));
    assert_eq!(tree.get(child1), None);
    assert_eq!(tree.get(child2), Some(&"child2"));
    assert_eq!(tree.get(grandchild1), None);
    assert_eq!(tree.children(root), &[child2]);
}

#[test]
fn test_remove_subtree_with_owned_items_and_reuse() {
    let mut tree = Tree::new();
    let root = tree.add_node(String::from("root"));
    let branch = tree.add_child(root, String::from("branch"));
    let sibling = tree.add_child(root, String::from("sibling"));
    let first_child = tree.add_child(branch, String::from("first child"));
    let second_child = tree.add_child(branch, String::from("second child"));
    let grandchild = tree.add_child(second_child, String::from("grandchild"));
    let other_root = tree.add_node(String::from("other root"));

    let mut removed = Vec::new();
    tree.remove_subtree_with(branch, |index, item| removed.push((index, item)));

    assert_eq!(
        removed,
        vec![
            (branch, String::from("branch")),
            (second_child, String::from("second child")),
            (grandchild, String::from("grandchild")),
            (first_child, String::from("first child")),
        ]
    );
    assert_eq!(tree.len(), 3);
    assert_eq!(tree.children(root), &[sibling]);
    assert_eq!(tree.parent_index_unchecked(sibling), Some(root));
    assert_eq!(tree.get(root).map(String::as_str), Some("root"));
    assert_eq!(tree.get(sibling).map(String::as_str), Some("sibling"));
    assert_eq!(tree.get(other_root).map(String::as_str), Some("other root"));
    for &(index, _) in &removed {
        assert_eq!(tree.get(index), None);
    }

    for (index, item) in removed.into_iter().rev() {
        assert_eq!(tree.next_node_id(), index);
        assert_eq!(tree.add_child(root, item), index);
        assert_eq!(tree.parent_index_unchecked(index), Some(root));
        assert!(tree.children(index).is_empty());
    }
    assert_eq!(tree.len(), 7);
}

#[test]
fn test_remove_subtree_with_skips_missing_indices() {
    let mut tree = Tree::new();
    let root = tree.add_node("root");
    let child = tree.add_child(root, "child");
    tree.remove_subtree(child);

    for index in [child, tree.nodes.len(), usize::MAX] {
        tree.remove_subtree_with(index, |_, _| panic!("missing node triggered callback"));
    }

    assert_eq!(tree.iter().collect::<Vec<_>>(), vec![(root, &"root")]);
    assert_eq!(tree.len(), 1);
    assert!(tree.children(root).is_empty());
    assert_eq!(tree.next_node_id(), child);

    tree.clear();
    tree.remove_subtree_with(root, |_, _| panic!("empty tree triggered callback"));
    assert!(tree.is_empty());
    assert_eq!(tree.next_node_id(), 0);
}

#[test]
fn test_remove_subtree_with_roots_and_storage_reset() {
    let mut tree = Tree::new();
    let root = tree.add_node("root");
    let child = tree.add_child(root, "child");
    let other_root = tree.add_node("other root");
    let other_child = tree.add_child(other_root, "other child");

    let mut removed = Vec::new();
    tree.remove_subtree_with(root, |index, item| removed.push((index, item)));

    assert_eq!(removed, vec![(root, "root"), (child, "child")]);
    assert_eq!(tree.len(), 2);
    assert_eq!(tree.children(other_root), &[other_child]);
    assert_eq!(tree.parent_index_unchecked(other_child), Some(other_root));

    removed.clear();
    tree.remove_subtree_with(other_root, |index, item| removed.push((index, item)));

    assert_eq!(
        removed,
        vec![(other_root, "other root"), (other_child, "other child")]
    );
    assert!(tree.is_empty());
    assert_eq!(tree.iter().next(), None);
    assert_eq!(tree.next_node_id(), 0);
    assert_eq!(tree.add_node("new root"), 0);
}

#[test]
fn test_remove_leaf_node() {
    let mut tree = Tree::new();
    let root = tree.add_node("root");
    let child1 = tree.add_child(root, "child1");
    let child2 = tree.add_child(root, "child2");

    tree.remove_subtree(child1);

    assert_eq!(tree.len(), 2);
    assert_eq!(tree.get(child1), None);
    assert_eq!(tree.children(root), &[child2]);
}

#[test]
fn test_remove_root() {
    let mut tree = Tree::new();
    let root = tree.add_node("root");
    tree.add_child(root, "child1");
    tree.add_child(root, "child2");

    tree.remove_subtree(root);

    assert!(tree.is_empty());
    assert_eq!(tree.len(), 0);
}

#[test]
fn test_remove_and_reuse() {
    let mut tree = Tree::new();
    let root = tree.add_node(0);
    let child1 = tree.add_child(root, 1);
    tree.add_child(root, 2);
    tree.add_child(child1, 3);

    tree.remove_subtree(child1);

    let new_child = tree.add_child(root, 10);
    assert!(new_child == 3 || new_child == 1);

    assert_eq!(tree.len(), 3);
    assert_eq!(tree.get(new_child), Some(&10));
}

#[test]
fn test_iter_after_remove() {
    let mut tree = Tree::new();
    let root = tree.add_node(0);
    let child1 = tree.add_child(root, 1);
    let child2 = tree.add_child(root, 2);
    tree.add_child(child1, 3);

    tree.remove_subtree(child1);

    let items: Vec<_> = tree.iter().collect();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0], (root, &0));
    assert_eq!(items[1], (child2, &2));
}

#[test]
fn test_traverse_after_remove() {
    let mut tree = Tree::new();
    let root = tree.add_node(0);
    let child1 = tree.add_child(root, 1);
    tree.add_child(root, 2);
    tree.add_child(child1, 3);

    tree.remove_subtree(child1);

    let mut result = vec![];
    tree.traverse(
        |idx, data, result: &mut Vec<String>| result.push(format!("enter {idx}:{data}")),
        |idx, data, result: &mut Vec<String>| result.push(format!("leave {idx}:{data}")),
        &mut result,
    );

    assert_eq!(
        result,
        vec!["enter 0:0", "enter 2:2", "leave 2:2", "leave 0:0",]
    );
}

#[test]
fn test_traverse_after_removing_first_root() {
    let mut tree = Tree::new();
    let root0 = tree.add_node("root0");
    let root1 = tree.add_node("root1");
    tree.add_child(root1, "child");

    tree.remove_subtree(root0);

    let mut result = vec![];
    tree.traverse(
        |idx, data, result: &mut Vec<String>| result.push(format!("enter {idx}:{data}")),
        |idx, data, result: &mut Vec<String>| result.push(format!("leave {idx}:{data}")),
        &mut result,
    );

    assert!(result.is_empty());
}

#[test]
fn test_traverse_mut_after_removing_first_root() {
    let mut tree = Tree::new();
    let root0 = tree.add_node(0);
    let root1 = tree.add_node(10);
    let child = tree.add_child(root1, 20);

    tree.remove_subtree(root0);

    let mut visited = vec![];
    tree.traverse_mut(
        |idx, data, visited: &mut Vec<(usize, i32)>| {
            *data += 1;
            visited.push((idx, *data));
        },
        |_, _, _| {},
        &mut visited,
    );

    assert!(visited.is_empty());
    assert_eq!(tree.get(root1), Some(&10));
    assert_eq!(tree.get(child), Some(&20));
}

#[test]
fn test_remove_idempotent() {
    let mut tree = Tree::new();
    let root = tree.add_node("root");
    let child = tree.add_child(root, "child");

    tree.remove_subtree(child);
    tree.remove_subtree(child); // no-op

    assert_eq!(tree.len(), 1);
}

#[test]
fn test_remove_out_of_bounds() {
    let mut tree = Tree::new();
    let root = tree.add_node("root");

    tree.remove_subtree(999); // no-op

    assert_eq!(tree.len(), 1);
    assert_eq!(tree.get(root), Some(&"root"));
}

#[test]
fn test_add_after_remove_root() {
    let mut tree = Tree::new();
    let root = tree.add_node("root");
    tree.add_child(root, "child");

    tree.remove_subtree(root);
    assert!(tree.is_empty());

    let new_root = tree.add_node("new_root");
    assert_eq!(new_root, 0);
    assert_eq!(tree.get(new_root), Some(&"new_root"));
    assert_eq!(tree.len(), 1);
}
