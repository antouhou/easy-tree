use easy_tree::Tree;

fn main() {
    let mut tree = Tree::new();
    let root = tree.add_node("root");
    let child1 = tree.add_child(root, "child1");
    let _grandchild1 = tree.add_child(child1, "grandchild1");
    let _child2 = tree.add_child(root, "child2");

    let mut log = vec![];
    tree.traverse(
        |idx, data, log| log.push(format!("Visiting node {}: {}", idx, data)),
        |idx, data, log| log.push(format!("Finished node {}: {}", idx, data)),
        &mut log,
    );

    println!("{:?}", log);
}
