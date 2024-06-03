# easy-tree

[![Crates.io](https://img.shields.io/crates/v/easy-tree.svg)](https://crates.io/crates/easy-tree)
[![Documentation](https://docs.rs/easy-tree/badge.svg)](https://docs.rs/easy-tree)
[![Build and test](https://github.com/antouhou/easy-tree/actions/workflows/test.yml/badge.svg?branch=main)](https://github.com/antouhou/easy-tree/actions)

`easy-tree` is a simple and efficient tree structure library for Rust. It allows you to create and manipulate tree 
structures where each node can have multiple children and a single parent. `easy-tree` also supports recursively 
traversing the tree in a depth-first manner, with two callbacks: one before processing any children and one after 
processing the subtree belonging to that node (meaning children, and their children, and so on).

`easy-tree` is
[available on crates.io](https://crates.io/crates/easy-tree), and
[API Documentation is available on docs.rs](https://docs.rs/easy-tree/).

## Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
easy-tree = "0.1"
```

To enable parallel iteration using the rayon crate, add the following:

```toml
[dependencies]
easy-tree = { version = "0.1", features = ["rayon"] }
```

## Examples

Walk the tree in a depth-first manner:

```rust
use easy_tree::Tree;

fn main() {
    let mut tree = Tree::new();
    let root = tree.add_node(0); // Root node with data 0
    let child1 = tree.add_child(root, 1); // Child node with data 1
    let child2 = tree.add_child(root, 2); // Child node with data 2
    let child3 = tree.add_child(child1, 3); // Child node with data 3

    let mut result = vec![];

    tree.traverse(|index, node, result| {
        result.push(format!("Calling handler for node {}: {}", index, node))
    }, |index, node, result| {
        result.push(format!("Finished handling node {} and all it's children", index))
    }, &mut result);

    assert_eq!(result, vec![
        "Calling handler for node 0: 0",
        "Calling handler for node 1: 1",
        "Calling handler for node 3: 3",
        "Finished handling node 3 and all it's children",
        "Finished handling node 1 and all it's children",
        "Calling handler for node 2: 2",
        "Finished handling node 2 and all it's children",
        "Finished handling node 0 and all it's children",
    ]);
}
```

```rust
use easy_tree::Tree;

fn main() {
    // Create a new tree and add nodes
    let mut tree = Tree::new();
    let root = tree.add_node(0); // Root node with data 0
    let child1 = tree.add_child(root, 1); // Child node with data 1
    let child2 = tree.add_child(root, 2); // Child node with data 2
    let child3 = tree.add_child(child1, 3); // Child node with data 3

// Access nodes and their relationships
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
```

```rust
use easy_tree::Tree;

fn main() {
    // Create a new tree and add nodes
    let mut tree = Tree::new();
    let root = tree.add_node(0); // Root node with data 0
    let child1 = tree.add_child(root, 1); // Child node with data 1
    let child2 = tree.add_child(root, 2); // Child node with data 2
    let child3 = tree.add_child(child1, 3); // Child node with data 3

// Iterate over the nodes in the tree
    for (index, data) in tree.iter() {
        println!("Node {}: {}", index, data);
    }

// Iterate over the nodes in the tree mutably
    for (index, data) in tree.iter_mut() {
        *data += 1;
    }

// Check the modified values
    assert_eq!(tree.get(root), Some(&1));
    assert_eq!(tree.get(child1), Some(&2));
    assert_eq!(tree.get(child2), Some(&3));
    assert_eq!(tree.get(child3), Some(&4));
}
```

## Documentation

[Documentation is available on docs.rs](https://docs.rs/easy-tree/).

## Contributing

Everyone is welcome to contribute in any way or form! For further details,
please read [CONTRIBUTING.md](./CONTRIBUTING.md)

## Authors
- [Anton Suprunchuk](https://github.com/antouhou) - [Website](https://antouhou.com)

Also, see the list of contributors who participated in this project.

## License

This project is licensed under the MIT License - see the
[LICENSE.md](./LICENSE.md) file for details