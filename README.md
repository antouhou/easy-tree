# easy-tree

[![Crates.io](https://img.shields.io/crates/v/easy-tree.svg)](https://crates.io/crates/easy-tree)
[![Documentation](https://docs.rs/easy-tree/badge.svg)](https://docs.rs/easy-tree)
[![Build and test](https://github.com/antouhou/easy-tree/actions/workflows/test.yml/badge.svg?branch=main)](https://github.com/antouhou/easy-tree/actions)

`easy-tree` is a Rust tree library with depth-first traversal methods. Pass
callbacks to run before and after each node's children, with shared mutable
state available to both. The library walks the tree for you, so you can write
the processing logic without implementing the traversal.

Use `traverse` to read node data, `traverse_mut` to modify it during traversal,
or `traverse_subtree_mut` to start from a particular node.

## Installation

Add the dependency to `Cargo.toml`:

```toml
[dependencies]
easy-tree = "0.5"
```

To enable parallel iteration:

```toml
[dependencies]
easy-tree = { version = "0.5", features = ["rayon"] }
```

## Create a tree

`add_node` inserts a node without a parent. `add_child` attaches a new node to an
existing parent. Both return the inserted node's index.

```rust
use easy_tree::Tree;

fn main() {
    let mut tree = Tree::new();
    let root = tree.add_node("root");
    let child = tree.add_child(root, "child");
    let grandchild = tree.add_child(child, "grandchild");

    assert_eq!(tree.get(grandchild), Some(&"grandchild"));
    assert_eq!(tree.children(root), &[child]);
    assert_eq!(tree.parent_index_unchecked(grandchild), Some(child));
}
```

`get` and `get_mut` return `None` for an out-of-bounds or removed index.
`get_unchecked`, `get_unchecked_mut`, `parent_index_unchecked`, and `children`
panic for those indices. `add_child` rejects a missing parent before it changes
the tree.

## Traverse nodes

One call to `traverse` runs both the before-children and after-children callbacks
for each node. Children are visited in insertion order. Both callbacks receive
the mutable state passed as the final argument.

```rust
use easy_tree::Tree;

fn main() {
    let mut tree = Tree::new();
    let root = tree.add_node("root");
    tree.add_child(root, "child");

    let mut log = Vec::new();
    tree.traverse(
        |_, data, log| log.push(format!("enter {data}")),
        |_, data, log| log.push(format!("leave {data}")),
        &mut log,
    );

    assert_eq!(log, ["enter root", "enter child", "leave child", "leave root"]);
}
```

`traverse` and `traverse_mut` visit index zero and its descendants. They do
nothing if that slot is empty, even if other disconnected roots remain.
`traverse_subtree_mut` starts at a caller-selected index and gives callbacks
mutable access to node data.

### Walk a directory tree

The before-children callback can add a directory to the current path, and the
after-children callback can remove it. Sharing the path between callbacks keeps
it available while processing each directory's descendants.

```rust
use easy_tree::Tree;

fn main() {
    let mut tree = Tree::new();
    let root = tree.add_node("root");
    let home = tree.add_child(root, "home");
    tree.add_child(home, "documents");

    let mut path = Vec::new();
    tree.traverse(
        |_, name, path| {
            path.push(*name);
            println!("/{}", path.join("/"));
        },
        |_, _, path| {
            path.pop();
        },
        &mut path,
    );

    assert!(path.is_empty());
}
```

## Iterate in parallel

With the `rayon` feature enabled, import `ParallelIterator` to use its methods:

```rust
use easy_tree::rayon::iter::ParallelIterator;
use easy_tree::Tree;

fn main() {
    let mut tree = Tree::new();
    let root = tree.add_node(0);
    tree.add_child(root, 1);
    tree.add_child(root, 2);

    tree.par_iter().for_each(|(index, data)| {
        println!("Node {index}: {data}");
    });
}
```

`iter` and `iter_mut` visit every live node in index order, including disconnected
roots. `par_iter` and `par_iter_mut` also include every live node, but parallel
callbacks may run in any order.

## Remove a subtree

`remove_subtree` removes a node and its descendants and detaches it from its parent.
Removing an out-of-bounds or already removed index does nothing.

```rust
use easy_tree::Tree;

fn main() {
    let mut tree = Tree::new();
    let root = tree.add_node("root");
    let child = tree.add_child(root, "child");
    tree.add_child(child, "grandchild");

    tree.remove_subtree(child);

    assert_eq!(tree.len(), 1);
    assert!(tree.children(root).is_empty());
    assert_eq!(tree.get(child), None);
}
```

Insertions can reuse removed indices. Stop using an index after removing its
node, since it may later identify another node. `len` counts live nodes, so it
is not an upper bound on their indices.

Use `remove_subtree_with` to receive each removed index and take ownership of its
data. The callback runs once per removed node, starting with the subtree root,
then visiting descendants depth-first with siblings in reverse insertion order.
It is never called for an out-of-bounds or already removed index.

```rust
use easy_tree::Tree;

let mut tree = Tree::new();
let root = tree.add_node(String::from("root"));
let child = tree.add_child(root, String::from("child"));

let mut removed = Vec::new();
tree.remove_subtree_with(child, |index, item| removed.push((index, item)));

assert_eq!(removed, vec![(child, String::from("child"))]);
assert_eq!(tree.len(), 1);
```
