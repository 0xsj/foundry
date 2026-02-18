// DOM-Like Document Tree — Reference Solution
//
// Run tests:  rustc --test main.rs && ./main
// Run binary: rustc main.rs && ./main

use std::rc::{Rc, Weak};
use std::cell::RefCell;

// ---------- Types ----------

pub struct NodeData {
    pub tag: String,
    pub text: Option<String>,
    // Weak to avoid the cycle: parent -> children -> child -> parent
    // A Weak does not keep the allocation alive.
    pub parent: RefCell<Weak<NodeData>>,
    pub children: RefCell<Vec<Rc<NodeData>>>,
}

pub type Node = NodeData;

// ---------- Implementation ----------

impl Node {
    /// Creates a new node with the given tag, no parent, and no children.
    pub fn new(tag: &str) -> Rc<Node> {
        Rc::new(NodeData {
            tag: tag.to_string(),
            text: None,
            parent: RefCell::new(Weak::new()),   // Weak::new() represents "no parent"
            children: RefCell::new(vec![]),
        })
    }

    /// Creates a new leaf node with text content.
    pub fn new_with_text(tag: &str, text: &str) -> Rc<Node> {
        Rc::new(NodeData {
            tag: tag.to_string(),
            text: Some(text.to_string()),
            parent: RefCell::new(Weak::new()),
            children: RefCell::new(vec![]),
        })
    }

    /// Appends child to parent's children and sets child's parent link.
    ///
    /// Why this is an associated function (not a method):
    /// To create a Weak<Node> for the parent, we need a Rc<Node>. Methods only
    /// receive &self, not the Rc. Associated functions let callers pass the Rc directly.
    pub fn add_child(parent: &Rc<Node>, child: Rc<Node>) {
        // If the child already belongs to another parent, detach it first.
        // This prevents the same Rc from appearing in two children lists.
        if let Some(old_parent) = child.parent.borrow().upgrade() {
            // Only detach if the old parent is different from the new one
            if !Rc::ptr_eq(&old_parent, parent) {
                old_parent.children.borrow_mut().retain(|c| !Rc::ptr_eq(c, &child));
            }
        }

        // Set the child's parent to a Weak pointer to parent.
        // Rc::downgrade creates a Weak — does not increment strong count.
        *child.parent.borrow_mut() = Rc::downgrade(parent);

        // Add child to parent's children list.
        parent.children.borrow_mut().push(child);
    }

    /// Removes child from parent's children. Returns true if found and removed.
    ///
    /// Rc::ptr_eq compares pointer addresses — two Rcs are equal iff they point
    /// to the same heap allocation, regardless of the values inside.
    pub fn remove_child(parent: &Rc<Node>, child: &Rc<Node>) -> bool {
        let mut children = parent.children.borrow_mut();
        let before_len = children.len();

        children.retain(|c| !Rc::ptr_eq(c, child));

        let removed = children.len() < before_len;
        if removed {
            // Clear the child's parent reference
            *child.parent.borrow_mut() = Weak::new();
        }
        removed
    }

    /// Returns the parent as Option<Rc<Node>>.
    ///
    /// upgrade() returns None if the Weak was default-constructed (Weak::new())
    /// or if the target allocation has been freed.
    pub fn parent(node: &Rc<Node>) -> Option<Rc<Node>> {
        node.parent.borrow().upgrade()
    }

    /// Returns the depth of the node in the tree (root = 0).
    pub fn depth(node: &Rc<Node>) -> usize {
        let mut depth = 0;
        let mut current = Rc::clone(node);
        // Walk up the tree via parent links until we reach a node with no parent
        loop {
            let maybe_parent = current.parent.borrow().upgrade();
            match maybe_parent {
                Some(p) => {
                    depth += 1;
                    current = p;
                }
                None => break,
            }
        }
        depth
    }

    /// Returns all tags in pre-order traversal, formatted as "depth:tag".
    ///
    /// Pre-order: visit the current node first, then recursively visit children.
    /// We use a helper closure to avoid exposing the depth parameter in the public API.
    pub fn traverse_preorder(node: &Rc<Node>) -> Vec<String> {
        let mut result = Vec::new();
        Self::traverse_helper(node, 0, &mut result);
        result
    }

    fn traverse_helper(node: &Rc<Node>, depth: usize, result: &mut Vec<String>) {
        result.push(format!("{}:{}", depth, node.tag));
        for child in node.children.borrow().iter() {
            Self::traverse_helper(child, depth + 1, result);
        }
    }

    /// DFS search for the first node with the given tag.
    ///
    /// Returns a cloned Rc so the caller gets an owned handle, not a lifetime-bound reference.
    pub fn find(root: &Rc<Node>, tag: &str) -> Option<Rc<Node>> {
        if root.tag == tag {
            return Some(Rc::clone(root));
        }
        // DFS through children
        for child in root.children.borrow().iter() {
            if let Some(found) = Self::find(child, tag) {
                return Some(found);
            }
        }
        None
    }

    /// Returns the number of direct children.
    pub fn child_count(node: &Rc<Node>) -> usize {
        node.children.borrow().len()
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

        assert!(removed);
        assert_eq!(Node::child_count(&div), 0);
        assert!(Node::parent(&span).is_none());
    }

    #[test]
    fn test_remove_child_not_found() {
        let div = Node::new("div");
        let span = Node::new("span");
        let removed = Node::remove_child(&div, &span);
        assert!(!removed);
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

    #[test]
    fn test_parent_weak_returns_none_after_drop() {
        let parent = Node::new("section");
        let child = Node::new("p");
        Node::add_child(&parent, Rc::clone(&child));

        let weak_parent = Rc::downgrade(&parent);
        drop(parent);

        assert!(weak_parent.upgrade().is_none());
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
        let parts: Vec<&str> = entry.splitn(2, ':').collect();
        let depth: usize = parts[0].parse().unwrap();
        let tag = parts[1];
        println!("{}<{}>", "  ".repeat(depth), tag);
    }

    println!("\nFind 'article': {:?}", Node::find(&html, "article").map(|n| n.tag.clone()));
    println!("Depth of p: {}", Node::depth(&p));

    // Demonstrate reparenting
    let aside = Node::new("aside");
    Node::add_child(&body, Rc::clone(&aside));
    println!("\nAfter adding aside to body:");
    println!("body child count: {}", Node::child_count(&body));

    // Move article from main to aside
    Node::add_child(&aside, Rc::clone(&article));
    println!("After moving article to aside:");
    println!("main children: {}", Node::child_count(&main));
    println!("aside children: {}", Node::child_count(&aside));

    println!("\nFull traversal after reparent:");
    for entry in Node::traverse_preorder(&html) {
        let parts: Vec<&str> = entry.splitn(2, ':').collect();
        let depth: usize = parts[0].parse().unwrap();
        let tag = parts[1];
        println!("{}<{}>", "  ".repeat(depth), tag);
    }
}
