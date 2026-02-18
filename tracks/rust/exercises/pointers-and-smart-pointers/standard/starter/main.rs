// DOM-Like Document Tree — Pointers & Smart Pointers Exercise (Rust)
//
// Implement a tree of Nodes where:
//   - Nodes are heap-allocated and reference-counted (Rc<Node>)
//   - Children are owned by their parent (Vec<Rc<Node>>)
//   - Parents are referenced weakly to avoid cycles (Weak<Node>)
//   - Interior mutation uses RefCell
//
// Run tests: rustc --test main.rs && ./main

use std::rc::{Rc, Weak};
use std::cell::RefCell;

// ---------- Types ----------

pub struct NodeData {
    pub tag: String,
    pub text: Option<String>,
    pub parent: RefCell<Weak<NodeData>>,
    pub children: RefCell<Vec<Rc<NodeData>>>,
}

pub type Node = NodeData;

// ---------- Implementation ----------

impl Node {
    /// Creates a new node with the given tag, no parent, and no children.
    pub fn new(tag: &str) -> Rc<Node> {
        todo!()
    }

    /// Creates a new leaf node with text content.
    pub fn new_with_text(tag: &str, text: &str) -> Rc<Node> {
        todo!()
    }

    /// Appends child to parent's children list and sets child's parent link.
    /// If child already has a parent, removes it from the old parent first.
    pub fn add_child(parent: &Rc<Node>, child: Rc<Node>) {
        todo!()
    }

    /// Removes child from parent's children. Returns true if found and removed.
    /// Uses Rc::ptr_eq for identity comparison (not structural equality).
    pub fn remove_child(parent: &Rc<Node>, child: &Rc<Node>) -> bool {
        todo!()
    }

    /// Returns the parent node, or None if root or parent was dropped.
    pub fn parent(node: &Rc<Node>) -> Option<Rc<Node>> {
        todo!()
    }

    /// Returns the depth of the node (root = 0).
    pub fn depth(node: &Rc<Node>) -> usize {
        todo!()
    }

    /// Returns all tags in pre-order traversal, formatted as "depth:tag".
    pub fn traverse_preorder(node: &Rc<Node>) -> Vec<String> {
        todo!()
    }

    /// Finds the first node with the given tag (DFS or BFS — your choice).
    /// Returns a clone of the Rc so the caller can use it.
    pub fn find(root: &Rc<Node>, tag: &str) -> Option<Rc<Node>> {
        todo!()
    }

    /// Returns the number of direct children.
    pub fn child_count(node: &Rc<Node>) -> usize {
        todo!()
    }
}

// ---------- Tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Construction --

    #[test]
    fn test_new_node_has_no_parent_or_children() {
        let node = Node::new("div");
        assert_eq!(node.tag, "div");
        assert_eq!(node.text, None);
        assert_eq!(Node::child_count(&node), 0);
        assert!(Node::parent(&node).is_none());
    }

    #[test]
    fn test_new_with_text() {
        let node = Node::new_with_text("p", "Hello, world");
        assert_eq!(node.tag, "p");
        assert_eq!(node.text, Some(String::from("Hello, world")));
    }

    // -- add_child --

    #[test]
    fn test_add_child_sets_parent() {
        let body = Node::new("body");
        let section = Node::new("section");

        Node::add_child(&body, Rc::clone(&section));

        assert_eq!(Node::child_count(&body), 1);
        let parent = Node::parent(&section).expect("section should have parent");
        assert!(Rc::ptr_eq(&parent, &body));
    }

    #[test]
    fn test_add_multiple_children() {
        let ul = Node::new("ul");
        let li1 = Node::new_with_text("li", "Item 1");
        let li2 = Node::new_with_text("li", "Item 2");
        let li3 = Node::new_with_text("li", "Item 3");

        Node::add_child(&ul, Rc::clone(&li1));
        Node::add_child(&ul, Rc::clone(&li2));
        Node::add_child(&ul, Rc::clone(&li3));

        assert_eq!(Node::child_count(&ul), 3);
        assert!(Rc::ptr_eq(&Node::parent(&li1).unwrap(), &ul));
        assert!(Rc::ptr_eq(&Node::parent(&li3).unwrap(), &ul));
    }

    #[test]
    fn test_add_child_moves_from_old_parent() {
        let old_parent = Node::new("section");
        let new_parent = Node::new("article");
        let child = Node::new("p");

        Node::add_child(&old_parent, Rc::clone(&child));
        assert_eq!(Node::child_count(&old_parent), 1);

        // Move child to new_parent
        Node::add_child(&new_parent, Rc::clone(&child));

        assert_eq!(Node::child_count(&old_parent), 0, "old parent should lose child");
        assert_eq!(Node::child_count(&new_parent), 1);
        let parent = Node::parent(&child).unwrap();
        assert!(Rc::ptr_eq(&parent, &new_parent));
    }

    // -- remove_child --

    #[test]
    fn test_remove_child_found() {
        let div = Node::new("div");
        let span = Node::new("span");

        Node::add_child(&div, Rc::clone(&span));
        assert_eq!(Node::child_count(&div), 1);

        let removed = Node::remove_child(&div, &span);

        assert!(removed, "remove_child should return true");
        assert_eq!(Node::child_count(&div), 0);
        assert!(Node::parent(&span).is_none(), "span should have no parent after removal");
    }

    #[test]
    fn test_remove_child_not_found() {
        let div = Node::new("div");
        let span = Node::new("span");

        let removed = Node::remove_child(&div, &span);
        assert!(!removed, "remove_child should return false when not found");
    }

    // -- depth --

    #[test]
    fn test_depth() {
        let html = Node::new("html");
        let body = Node::new("body");
        let section = Node::new("section");
        let p = Node::new("p");

        Node::add_child(&html, Rc::clone(&body));
        Node::add_child(&body, Rc::clone(&section));
        Node::add_child(&section, Rc::clone(&p));

        assert_eq!(Node::depth(&html), 0);
        assert_eq!(Node::depth(&body), 1);
        assert_eq!(Node::depth(&section), 2);
        assert_eq!(Node::depth(&p), 3);
    }

    // -- traverse_preorder --

    #[test]
    fn test_traverse_preorder_single_node() {
        let node = Node::new("p");
        assert_eq!(Node::traverse_preorder(&node), vec!["0:p"]);
    }

    #[test]
    fn test_traverse_preorder_tree() {
        //        body (depth 0)
        //       /    \
        // header    main  (depth 1)
        //              \
        //              article  (depth 2)

        let body = Node::new("body");
        let header = Node::new("header");
        let main = Node::new("main");
        let article = Node::new("article");

        Node::add_child(&body, Rc::clone(&header));
        Node::add_child(&body, Rc::clone(&main));
        Node::add_child(&main, Rc::clone(&article));

        let traversal = Node::traverse_preorder(&body);
        assert_eq!(traversal, vec!["0:body", "1:header", "1:main", "2:article"]);
    }

    // -- find --

    #[test]
    fn test_find_root() {
        let body = Node::new("body");
        let found = Node::find(&body, "body");
        assert!(found.is_some());
        assert!(Rc::ptr_eq(&found.unwrap(), &body));
    }

    #[test]
    fn test_find_deep_node() {
        let root = Node::new("html");
        let body = Node::new("body");
        let section = Node::new("section");
        let target = Node::new_with_text("p", "target");

        Node::add_child(&root, Rc::clone(&body));
        Node::add_child(&body, Rc::clone(&section));
        Node::add_child(&section, Rc::clone(&target));

        let found = Node::find(&root, "p");
        assert!(found.is_some());
        assert!(Rc::ptr_eq(&found.unwrap(), &target));
    }

    #[test]
    fn test_find_not_found() {
        let root = Node::new("html");
        assert!(Node::find(&root, "table").is_none());
    }

    // -- No memory cycles --
    // Weak references don't keep the allocation alive.
    // If we drop the parent, the weak handle should return None.

    #[test]
    fn test_parent_weak_returns_none_after_drop() {
        let parent = Node::new("section");
        let child = Node::new("p");
        Node::add_child(&parent, Rc::clone(&child));

        // Grab a direct Weak before dropping parent
        let weak_parent = Rc::downgrade(&parent);

        // Drop the parent (and its children list, which holds the last Rc to child)
        drop(parent);

        // The weak pointer to the former parent should now be dead
        assert!(weak_parent.upgrade().is_none(),
            "weak reference to dropped parent should return None");
    }
}

fn main() {
    let html = Node::new("html");
    let body = Node::new("body");
    let header = Node::new("header");
    let h1 = Node::new_with_text("h1", "Welcome");
    let main = Node::new("main");
    let article = Node::new("article");
    let p = Node::new_with_text("p", "Hello, document tree!");

    Node::add_child(&html, Rc::clone(&body));
    Node::add_child(&body, Rc::clone(&header));
    Node::add_child(&header, Rc::clone(&h1));
    Node::add_child(&body, Rc::clone(&main));
    Node::add_child(&main, Rc::clone(&article));
    Node::add_child(&article, Rc::clone(&p));

    println!("Pre-order traversal:");
    for entry in Node::traverse_preorder(&html) {
        let indent = "  ".repeat(entry.split(':').next().unwrap().parse::<usize>().unwrap());
        let tag = entry.split(':').nth(1).unwrap();
        println!("{}<{}>", indent, tag);
    }

    println!("\nFind 'article': {:?}", Node::find(&html, "article").map(|n| n.tag.clone()));
    println!("Depth of p: {}", Node::depth(&p));
    println!("Strong count on body: {}", Rc::strong_count(&body));
}
