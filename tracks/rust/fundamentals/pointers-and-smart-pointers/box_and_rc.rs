// Box and Rc — Heap Allocation, Shared Ownership, and Weak References
//
// Run: rustc box_and_rc.rs && ./box_and_rc

use std::rc::{Rc, Weak};
use std::cell::RefCell;

// ---------- Box<T>: Single Ownership on the Heap ----------

// A recursive AST node. Without Box, this would have infinite size.
// The compiler needs to know the size of AstNode at compile time.
// Box<AstNode> is always pointer-sized (8 bytes), so it's fine.
#[derive(Debug)]
enum AstNode {
    Literal(i64),
    Variable(String),
    Add(Box<AstNode>, Box<AstNode>),
    Multiply(Box<AstNode>, Box<AstNode>),
    Negate(Box<AstNode>),
}

impl AstNode {
    fn evaluate(&self, vars: &std::collections::HashMap<String, i64>) -> i64 {
        match self {
            AstNode::Literal(n) => *n,
            AstNode::Variable(name) => *vars.get(name).unwrap_or(&0),
            AstNode::Add(left, right) => left.evaluate(vars) + right.evaluate(vars),
            AstNode::Multiply(left, right) => left.evaluate(vars) * right.evaluate(vars),
            AstNode::Negate(inner) => -inner.evaluate(vars),
        }
    }

    // Helper constructors so callers don't type Box::new everywhere
    fn lit(n: i64) -> AstNode { AstNode::Literal(n) }
    fn var(s: &str) -> AstNode { AstNode::Variable(s.to_string()) }
    fn add(l: AstNode, r: AstNode) -> AstNode { AstNode::Add(Box::new(l), Box::new(r)) }
    fn mul(l: AstNode, r: AstNode) -> AstNode { AstNode::Multiply(Box::new(l), Box::new(r)) }
    fn neg(inner: AstNode) -> AstNode { AstNode::Negate(Box::new(inner)) }
}

fn box_demo() {
    println!("=== Box<T> ===");

    // Simple heap allocation — mostly useful for large structs or trait objects
    let heap_val: Box<i32> = Box::new(42);
    println!("heap_val = {}", heap_val);      // Deref coercion: Box<i32> -> i32
    println!("heap_val = {}", *heap_val);     // explicit deref

    // Box<dyn Trait>: heterogeneous collection of trait objects
    // Each variant is a different concrete type — Vec needs uniform size,
    // so we box them. The vtable enables dynamic dispatch.
    trait Describable {
        fn describe(&self) -> String;
    }

    struct DatabaseSource { host: String, port: u16 }
    struct FileSource { path: String }
    struct EnvSource;

    impl Describable for DatabaseSource {
        fn describe(&self) -> String { format!("db://{}:{}", self.host, self.port) }
    }
    impl Describable for FileSource {
        fn describe(&self) -> String { format!("file://{}", self.path) }
    }
    impl Describable for EnvSource {
        fn describe(&self) -> String { String::from("env://") }
    }

    let sources: Vec<Box<dyn Describable>> = vec![
        Box::new(DatabaseSource { host: String::from("db.internal"), port: 5432 }),
        Box::new(FileSource { path: String::from("/etc/app/config.yaml") }),
        Box::new(EnvSource),
    ];

    for source in &sources {
        println!("  {}", source.describe());
    }

    // Recursive AST: (x + 3) * -2
    let expr = AstNode::mul(
        AstNode::add(AstNode::var("x"), AstNode::lit(3)),
        AstNode::neg(AstNode::lit(2)),
    );

    let mut vars = std::collections::HashMap::new();
    vars.insert(String::from("x"), 7i64);

    let result = expr.evaluate(&vars);
    println!("(x + 3) * -2 where x=7 = {}", result);  // (7+3)*-2 = -20
}

// ---------- Rc<T>: Multiple Owners, Single Thread ----------

// A directed acyclic graph where nodes can be shared between multiple parents.
// This represents a build dependency graph: each task can be a dependency of
// multiple other tasks.
#[derive(Debug)]
struct BuildTask {
    name: String,
    // deps are shared — multiple tasks can list the same dep
    deps: Vec<Rc<BuildTask>>,
}

impl BuildTask {
    fn new(name: &str) -> Rc<BuildTask> {
        Rc::new(BuildTask {
            name: name.to_string(),
            deps: vec![],
        })
    }

    fn with_deps(name: &str, deps: Vec<Rc<BuildTask>>) -> Rc<BuildTask> {
        Rc::new(BuildTask {
            name: name.to_string(),
            deps,
        })
    }

    // Recursively collect all task names in topological order
    fn collect_order(&self, visited: &mut Vec<String>) {
        for dep in &self.deps {
            dep.collect_order(visited);
        }
        if !visited.contains(&self.name) {
            visited.push(self.name.clone());
        }
    }
}

fn rc_demo() {
    println!("\n=== Rc<T> ===");

    // proto is a dependency shared by BOTH api and worker
    let proto = BuildTask::new("generate-proto");
    let migrate = BuildTask::new("db-migrate");

    // Rc::clone increments reference count — does NOT clone the data
    let api = BuildTask::with_deps("build-api", vec![
        Rc::clone(&proto),
        Rc::clone(&migrate),
    ]);
    let worker = BuildTask::with_deps("build-worker", vec![
        Rc::clone(&proto),  // same allocation as api's dep
    ]);
    let deploy = BuildTask::with_deps("deploy", vec![
        api,
        worker,
    ]);

    println!("proto ref count: {}", Rc::strong_count(&proto));  // 3: proto + api + worker

    let mut order = Vec::new();
    deploy.collect_order(&mut order);
    println!("build order: {:?}", order);
    // [generate-proto, db-migrate, build-api, build-worker, deploy]

    // proto will be freed when deploy (and its children) drop,
    // releasing all Rc handles to it
}

// ---------- Weak<T>: Breaking Reference Cycles ----------

// A file system tree where each node knows its parent.
// Problem: if parent holds Rc<Child> and child holds Rc<Parent>,
// neither count ever reaches zero -> memory leak.
// Solution: parent holds Rc<Child> (owning), child holds Weak<Parent> (non-owning).

#[derive(Debug)]
struct FsNode {
    name: String,
    // Parent is Weak — we don't own it, we just know where it is.
    // When the parent is dropped, our Weak handle returns None on upgrade().
    parent: RefCell<Weak<FsNode>>,
    children: RefCell<Vec<Rc<FsNode>>>,
}

impl FsNode {
    fn new(name: &str) -> Rc<FsNode> {
        Rc::new(FsNode {
            name: name.to_string(),
            parent: RefCell::new(Weak::new()),  // no parent yet
            children: RefCell::new(vec![]),
        })
    }

    fn add_child(parent: &Rc<FsNode>, child: Rc<FsNode>) {
        // Set child's parent to a Weak pointer to the parent
        *child.parent.borrow_mut() = Rc::downgrade(parent);
        // Add child to parent's children list
        parent.children.borrow_mut().push(child);
    }

    fn path(&self) -> String {
        // Walk up the tree via Weak::upgrade()
        match self.parent.borrow().upgrade() {
            Some(p) => format!("{}/{}", p.path(), self.name),
            None => self.name.clone(),  // root node
        }
    }
}

fn weak_demo() {
    println!("\n=== Weak<T> — Breaking Cycles ===");

    let root = FsNode::new("/");
    let etc = FsNode::new("etc");
    let nginx = FsNode::new("nginx");
    let conf = FsNode::new("nginx.conf");

    FsNode::add_child(&root, Rc::clone(&etc));
    FsNode::add_child(&etc, Rc::clone(&nginx));
    FsNode::add_child(&nginx, Rc::clone(&conf));

    println!("conf path: {}", conf.path());
    // /etc/nginx/nginx.conf

    // Check that we don't have a cycle:
    println!("root strong count: {}", Rc::strong_count(&root));   // 1 (just root)
    println!("etc strong count: {}", Rc::strong_count(&etc));     // 2 (etc + root's children)
    // (Weak references don't appear in strong_count)

    // When root drops, it drops its children, which drop their children...
    // No cycle — it all unwinds cleanly.
    drop(conf);
    // nginx's children list still holds the allocation, so conf isn't freed yet

    // Verify: after dropping the main handle, upgrade() on a Weak would return None
    let weak_etc = Rc::downgrade(&etc);
    println!("upgrade before drop: {}", weak_etc.upgrade().is_some());  // true

    drop(root);
    drop(etc);
    drop(nginx);
    // Now everything is freed. weak_etc.upgrade() would return None.
    println!("upgrade after drop: {}", weak_etc.upgrade().is_none());   // true
}

fn main() {
    box_demo();
    rc_demo();
    weak_demo();
}

// ---------- Tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_evaluate() {
        let mut vars = std::collections::HashMap::new();
        vars.insert(String::from("x"), 10i64);

        // x * 2 + 1 = 21
        let expr = AstNode::add(
            AstNode::mul(AstNode::var("x"), AstNode::lit(2)),
            AstNode::lit(1),
        );
        assert_eq!(expr.evaluate(&vars), 21);
    }

    #[test]
    fn test_rc_shared_allocation() {
        let shared = BuildTask::new("shared");
        let a = Rc::clone(&shared);
        let b = Rc::clone(&shared);
        assert_eq!(Rc::strong_count(&shared), 3);
        assert!(Rc::ptr_eq(&a, &b));
    }

    #[test]
    fn test_weak_upgrade_returns_none_after_drop() {
        let strong = Rc::new(42);
        let weak = Rc::downgrade(&strong);
        assert!(weak.upgrade().is_some());
        drop(strong);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn test_fs_path() {
        let root = FsNode::new("root");
        let child = FsNode::new("child");
        let grandchild = FsNode::new("grandchild");

        FsNode::add_child(&root, Rc::clone(&child));
        FsNode::add_child(&child, Rc::clone(&grandchild));

        assert_eq!(grandchild.path(), "root/child/grandchild");
    }
}
