# easy-tree

[![Crates.io](https://img.shields.io/crates/v/easy-tree.svg)](https://crates.io/crates/easy-tree)
[![Documentation](https://docs.rs/easy-tree/badge.svg)](https://docs.rs/easy-tree)
[![Build and test](https://github.com/antouhou/easy-tree/actions/workflows/test.yml/badge.svg?branch=main)](https://github.com/antouhou/easy-tree/actions)

`easy-tree` is a lightweight Rust library for managing and traversing hierarchical data structures. It provides a simple interface for creating trees and supports **recursive depth-first traversal** with pre- and post-processing callbacks.

## Key Features

- **Depth-first traversal**: Easily process nodes with callbacks before and after their subtrees.
- **Simple API**: Add, modify, and retrieve nodes effortlessly.
- **Customizable traversal logic**: Use callbacks to handle specific traversal behaviors.
- **Optional parallel iteration**: Boost performance with [rayon](https://docs.rs/rayon).

## Why Use easy-tree?

1. **Designed for Hierarchical Data**: Ideal for file systems, DOM structures, organizational charts, and more.
2. **Traverse with Precision**: Depth-first traversal with pre- and post-order processing in one simple call.
3. **Memory Efficient**: Minimal overhead with direct references between nodes.
4. **Extensible**: Integrates easily into larger systems and workflows.

---

## Installation

Add `easy-tree` to your `Cargo.toml`:

```toml
[dependencies]
easy-tree = "0.1"
```

To enable parallel iteration:

```toml
[dependencies]
easy-tree = { version = "0.1", features = ["rayon"] }
```

---

## How It Works

### Tree Structure

Each node in the tree has:
- A **data payload** (your custom type)
- A single parent (or `None` for the root)
- A list of child indices

### Traversal Flow

Here’s an illustration of a depth-first traversal order:

```
Root
├── Child 1
│   └── Grandchild 1
├── Child 2
└── Child 3
```

Traversal output:
1. `Visiting Child 1`
2. `Visiting Grandchild 1`
3. `Leaving Grandchild 1`
4. `Leaving Child 1`
5. ...

---

## Examples

### 1. Basic Tree Creation

```rust
use easy_tree::Tree;

fn main() {
    let mut tree = Tree::new();
    let root = tree.add_node("root");
    let child = tree.add_child(root, "child");
    let grandchild = tree.add_child(child, "grandchild");

    assert_eq!(tree.get(root), Some(&"root"));
    assert_eq!(tree.get(grandchild), Some(&"grandchild"));
}
```

### 2. Depth-First Traversal

```rust
use easy_tree::Tree;

fn main() {
    let mut tree = Tree::new();
    let root = tree.add_node("root");
    let child1 = tree.add_child(root, "child1");
    let grandchild1 = tree.add_child(child1, "grandchild1");
    let child2 = tree.add_child(root, "child2");

    let mut log = vec![];
    tree.traverse(
        |idx, data, log| log.push(format!("Visiting node {}: {}", idx, data)),
        |idx, data, log| log.push(format!("Finished node {}: {}", idx, data)),
        &mut log,
    );

    println!("{:?}", log);
}
```

### 3. Parallel Iteration (Optional)

```rust
use easy_tree::Tree;

fn main() {
    let mut tree = Tree::new();
    let root = tree.add_node(0);
    let _child1 = tree.add_child(root, 1);
    let _child2 = tree.add_child(root, 2);

    #[cfg(feature = "rayon")]
    {
        tree.par_iter().for_each(|(idx, data)| {
            println!("Processing node {}: {}", idx, data);
        });
    }
}
```

---

## Use Cases

### Represent a File System

```rust
use easy_tree::Tree;

fn main() {
    let mut fs = Tree::new();
    let root = fs.add_node("root/");
    let home = fs.add_child(root, "home/");
    let user = fs.add_child(home, "user/");
    let file = fs.add_child(user, "file.txt");

    println!("Tree structure:");
    fs.traverse(
        |_, data, _| println!("Entering {}", data),
        |_, data, _| println!("Leaving {}", data),
        &mut (),
    );
}
```

---

## Performance

- **Low Memory Overhead**: Nodes are stored contiguously in a vector.
- **Efficient Traversal**: Iterative depth-first traversal minimizes recursion overhead.
- **Parallel Ready**: Enable the `rayon` feature for concurrent processing.

---

## Advanced Features

1. **Custom Traversal Logic**: Use pre- and post-processing callbacks for fine-grained control.
2. **Flexible Node Access**: Retrieve, update, or delete nodes efficiently.
