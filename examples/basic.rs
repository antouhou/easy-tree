use easy_tree::Tree;

fn main() {
    let mut tree = Tree::new();
    let root = tree.add_node("root");
    let child = tree.add_child(root, "child");
    let grandchild = tree.add_child(child, "grandchild");

    assert_eq!(tree.get(root), Some(&"root"));
    assert_eq!(tree.get(grandchild), Some(&"grandchild"));
}
