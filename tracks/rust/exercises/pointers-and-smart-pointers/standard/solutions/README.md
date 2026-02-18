# Solution: DOM-Like Document Tree

## Approach

The solution stores nodes as `Rc<NodeData>` for shared ownership and uses `RefCell` for
interior mutability. Parent back-references use `Weak<NodeData>` to prevent reference cycles.
All tree operations are associated functions rather than `&self` methods because they need
access to the `Rc` handle (not just the inner `NodeData`).

## Key Decisions

**Why `Rc<Node>` not `Box<Node>`**

`Box<T>` means exactly one owner. Tree nodes need to be referenced from multiple places:
- The parent's `children` list holds an `Rc`
- The caller holds an `Rc` (to call methods on a specific node)
- The traversal code holds temporary `Rc` clones

A `Box` parent can only have one child. `Rc` allows both the children list and external
callers to hold the same node.

**Why `Weak<Node>` for parent**

If the parent held `Rc<Child>` and the child held `Rc<Parent>`, you'd have a cycle:
neither reference count ever reaches zero. The nodes would leak. `Weak<Node>` for the
parent reference breaks the cycle — it doesn't contribute to the strong count, so when
the parent is dropped, its count reaches zero and it's freed (which drops the children,
which drops their children).

**Why associated functions, not methods**

```rust
// Method signature — only has &self:
fn add_child(&self, child: Rc<Node>) {
    // To create Weak<Node>, we need: Rc::downgrade(&parent_rc)
    // But we only have &self, not Rc<Node>. Can't make a Weak.
}

// Associated function — caller passes the Rc:
fn add_child(parent: &Rc<Node>, child: Rc<Node>) {
    *child.parent.borrow_mut() = Rc::downgrade(parent);  // works
}
```

**`Rc::ptr_eq` for identity comparison**

Nodes don't implement `PartialEq`. Even if they did, structural equality (same tag, same
children) isn't what we want — we want "is this the exact same node in memory?" `Rc::ptr_eq`
compares the raw pointer addresses.

**`borrow()` / `borrow_mut()` scope discipline**

In `add_child`, we carefully drop the `order` borrow before taking a `store` borrow. If you
hold two `borrow_mut()` handles simultaneously (even on different fields), the second call
panics. The solution releases each borrow by letting it go out of scope (`drop(borrow)`) before
taking the next one.

## Variants

| Approach | Trade-off |
|---|---|
| `Rc<RefCell<Node>>` (used) | Single-threaded only, flexible mutation |
| `Arc<Mutex<Node>>` | Thread-safe but heavy — use only if nodes are sent across threads |
| `Arena allocation` | Much faster (bump allocator), no smart pointers needed, but requires external crate |
| Indices into a `Vec` | Common in games/compilers — avoid heap allocation entirely at cost of ergonomics |

## What to Try Next

- Add a `Node::render(&self) -> String` that serializes the tree to an HTML string
- Add `attributes: HashMap<String, String>` to nodes and test that `RefCell` correctly
  prevents two simultaneous mutable borrows of `attributes`
- Try converting to `Arc<Mutex<Node>>` and see what breaks (hint: `Mutex` isn't `RefCell`)
- Connect to the patterns module: the Composite pattern is exactly this tree structure
