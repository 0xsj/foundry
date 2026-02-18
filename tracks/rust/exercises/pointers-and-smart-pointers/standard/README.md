# Standard Exercise: DOM-Like Document Tree

## Scenario

You're building a lightweight virtual document tree for a server-side HTML renderer. The
tree represents a document with nested elements: a `<body>` contains `<section>` nodes,
sections contain `<div>` nodes, divs contain `<p>` nodes, and so on. Nodes need to be
attached and detached dynamically (the renderer can reuse subtrees across renders).

The tricky ownership requirement: each node must be able to navigate to its parent without
holding a strong reference (that would create reference cycles). Children are owned by their
parent; back-references from children to parents must be non-owning.

## Brief

Implement a `Node` struct representing a document tree node. Nodes are always heap-allocated
and reference-counted (`Rc<Node>`). Children are stored as `Rc<Node>`. Parent back-references
are stored as `Weak<Node>`. Internal mutation (adding/removing children) happens through
`RefCell`.

## Acceptance Criteria

1. **`NodeData` struct** with fields:
   - `tag: String` — the element tag (e.g., `"div"`, `"p"`, `"section"`)
   - `text: Option<String>` — optional text content
   - `parent: RefCell<Weak<Node>>` — non-owning parent link
   - `children: RefCell<Vec<Rc<Node>>>` — owned children

2. **`Node` type alias**: `type Node = NodeData;`
   (Nodes are always used as `Rc<Node>` — the Rc is external)

3. **`Node::new(tag: &str) -> Rc<Node>`**
   - Creates a new node with no parent and no children
   - `text` starts as `None`

4. **`Node::new_with_text(tag: &str, text: &str) -> Rc<Node>`**
   - Creates a new leaf node with text content

5. **`Node::add_child(parent: &Rc<Node>, child: Rc<Node>)`**
   - Appends `child` to `parent`'s children
   - Sets `child`'s parent to a `Weak` pointing to `parent`
   - If `child` already has a parent, remove it from the old parent first
   - Associated function (not `&self` method) because it needs an `Rc<Node>` to make the `Weak`

6. **`Node::remove_child(parent: &Rc<Node>, child: &Rc<Node>) -> bool`**
   - Removes `child` from `parent`'s children list
   - Clears `child`'s parent reference
   - Returns `true` if the child was found and removed, `false` otherwise
   - Use `Rc::ptr_eq` to compare nodes by identity (not by value)

7. **`Node::parent(node: &Rc<Node>) -> Option<Rc<Node>>`**
   - Returns the parent as `Option<Rc<Node>>` (upgrading the Weak)
   - Returns `None` if the node is the root or the parent was dropped

8. **`Node::depth(node: &Rc<Node>) -> usize`**
   - Returns the depth of the node in the tree (root = 0)
   - Walk up via parent links

9. **`Node::traverse_preorder(node: &Rc<Node>) -> Vec<String>`**
   - Returns all node tags in pre-order (parent before children)
   - Format each entry as `"<depth>:<tag>"` (e.g., `"0:body"`, `"1:section"`, `"2:div"`)

10. **`Node::find(root: &Rc<Node>, tag: &str) -> Option<Rc<Node>>`**
    - BFS or DFS search for the first node with the given tag
    - Returns a clone of the `Rc` (not a reference) so the caller can work with it
    - Returns `None` if not found

11. **`Node::child_count(node: &Rc<Node>) -> usize`**
    - Returns the number of direct children

## Constraints

- No external crates — stdlib only
- Use `Rc::ptr_eq` to compare node identity (not structural equality)
- The tree must not leak memory (no strong reference cycles)
- All tests in the starter file must pass

## Hints

<details>
<summary>Hint 1: Why add_child takes Rc<Node>, not &self</summary>

`add_child` needs to store a `Weak<Node>` pointer to the parent inside the child. To
create a `Weak<Node>`, you call `Rc::downgrade(&parent_rc)`. That requires an `Rc<Node>`,
not a `&Node`. Associated functions (`Node::add_child(parent, child)`) can receive the `Rc`
directly. If it were a method (`&self`), you'd only have `&Node`, not `Rc<Node>`.

</details>

<details>
<summary>Hint 2: Upgrading a Weak</summary>

`Weak<Node>::upgrade()` returns `Option<Rc<Node>>`. Use it to navigate to the parent:

```rust
fn parent(node: &Rc<Node>) -> Option<Rc<Node>> {
    node.parent.borrow().upgrade()
}
```

</details>

<details>
<summary>Hint 3: remove_child — comparing Rc by identity</summary>

Two `Rc<Node>` values pointing to the same node are not structurally equal (unless you
derive PartialEq). Use `Rc::ptr_eq` to check if two Rcs point to the same allocation:

```rust
children.retain(|c| !Rc::ptr_eq(c, child));
```

</details>

<details>
<summary>Hint 4: pre-order traversal</summary>

Pre-order means: visit the current node first, then recursively visit each child.

```rust
fn traverse_preorder(node: &Rc<Node>) -> Vec<String> {
    let mut result = Vec::new();
    fn visit(node: &Rc<Node>, depth: usize, result: &mut Vec<String>) {
        result.push(format!("{}:{}", depth, node.tag));
        for child in node.children.borrow().iter() {
            visit(child, depth + 1, result);
        }
    }
    visit(node, 0, &mut result);
    result
}
```

</details>
