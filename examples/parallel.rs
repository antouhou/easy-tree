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
