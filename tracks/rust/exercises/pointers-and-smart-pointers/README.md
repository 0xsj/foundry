# Exercises: Pointers & Smart Pointers (Rust)

| Type | Status | Description |
|------|--------|-------------|
| [Standard](./standard/) | Available | DOM-like tree with `Rc<RefCell<Node>>` children, `Weak<Node>` parent back-links, add/remove/traverse/find operations |
| [Debugging](./debugging/) | Available | Four bugs: Rc cycle causing memory leak, RefCell double borrow panic, Rc sent across threads, Box where Rc is needed |
| [Code Review](./code-review/) | Available | Plugin system with shared state — over-use of Arc<Mutex<>>, unnecessary RefCell, missing Weak back-references |
| Refactoring | N/A | Smart pointer selection is design, not refactoring — covered in patterns module |
| API Design | N/A | Better suited after traits module (designing generic container APIs) |
| Codebase Navigation | N/A | Better suited after multiple modules are completed |
