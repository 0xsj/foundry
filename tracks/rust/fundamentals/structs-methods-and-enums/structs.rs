// Structs — Rust
//
// Demonstrates: named-field structs, tuple structs, unit structs,
// struct update syntax, newtype pattern, methods with different receivers.
//
// Run: rustc structs.rs && ./structs

// ---- Named-Field Struct ----

// A route entry in a load balancer.
// All three kinds of derive are used here deliberately:
//   Debug   — so we can print with {:?}
//   Clone   — so we can duplicate entries
//   PartialEq — so tests can compare with ==
#[derive(Debug, Clone, PartialEq)]
struct Endpoint {
    host: String,
    port: u16,
    healthy: bool,
    weight: u8,
}

impl Endpoint {
    // Associated function: the canonical constructor.
    // Takes &str to avoid requiring the caller to allocate a String first.
    fn new(host: &str, port: u16) -> Endpoint {
        Endpoint {
            host: host.to_string(),
            port,
            healthy: true,
            weight: 1,
        }
    }

    // &self — read-only borrow. Does not consume or mutate.
    fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    fn is_routable(&self) -> bool {
        self.healthy && self.weight > 0
    }

    // &mut self — mutates in place. Caller must hold a mutable binding.
    fn mark_unhealthy(&mut self) {
        self.healthy = false;
    }

    fn set_weight(&mut self, w: u8) {
        self.weight = w;
    }

    // self — consuming method. Endpoint is moved into this call.
    // Used for transformations that produce a different type.
    fn with_weight(mut self, w: u8) -> Endpoint {
        self.weight = w;
        self
    }
}

// ---- Struct Update Syntax ----

fn struct_update_demo() {
    let primary = Endpoint::new("api-1.internal", 8080);

    // ..primary moves all fields not listed from primary into standby.
    // primary.host (a String) is moved — primary cannot be used after this.
    let standby = Endpoint {
        host: String::from("api-2.internal"),
        ..primary
        // primary is partially moved here (host was in primary, not overridden)
        // Actually: we override host so primary.host is not moved.
        // port, healthy, weight are Copy types — they are copied, not moved.
        // So primary is still usable here because all moved fields were overridden.
    };

    // primary is still usable because we overrode the only non-Copy field (host).
    println!("primary : {}", primary.address());
    println!("standby : {}", standby.address());

    // Demonstrate moving a non-Copy field:
    let base = Endpoint::new("api-3.internal", 9090);
    let replica = Endpoint {
        port: 9091,
        ..base  // base.host (String) is moved into replica
    };
    // base.host is now invalid. base.port is Copy so it's still there,
    // but base as a whole cannot be used.
    println!("replica : {}", replica.address());
}

// ---- Tuple Structs and Newtype Pattern ----

// Each wraps the same underlying type (u64), but they are distinct types.
// You cannot accidentally pass one where the other is expected.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Milliseconds(u64);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Bytes(usize);

impl Milliseconds {
    fn from_seconds(s: u64) -> Milliseconds {
        Milliseconds(s * 1000)
    }

    fn as_seconds(&self) -> u64 {
        self.0 / 1000
    }

    fn has_elapsed(&self, elapsed: Milliseconds) -> bool {
        elapsed.0 >= self.0
    }
}

impl std::fmt::Display for Milliseconds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}ms", self.0)
    }
}

// A two-field tuple struct.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Size(u32, u32);  // width, height

impl Size {
    fn area(&self) -> u32 {
        self.0 * self.1
    }

    fn is_portrait(&self) -> bool {
        self.1 > self.0
    }
}

// ---- Unit Structs ----

// A marker with no data. Zero size. One possible value.
// Used here as a type-level token for states.
struct Unconfigured;
struct Ready;

// In practice these would be phantom type parameters on a generic struct.
// We'll use them plainly here to illustrate the syntax.
fn configure(_marker: Unconfigured) -> Ready {
    println!("configuring...");
    Ready
}

fn run(_marker: Ready) {
    println!("running with validated config.");
}

fn main() {
    // ----- Named-field struct -----
    println!("=== Endpoints ===");

    let mut ep = Endpoint::new("db.internal", 5432);
    println!("address     : {}", ep.address());
    println!("is_routable : {}", ep.is_routable());

    ep.mark_unhealthy();
    println!("after unhealthy: is_routable = {}", ep.is_routable());

    ep.set_weight(0);
    println!("after weight=0 : is_routable = {}", ep.is_routable());

    // Builder-style with consuming self:
    let high_prio = Endpoint::new("cache.internal", 6379).with_weight(5);
    println!("high_prio weight: {}", high_prio.weight);

    // ----- Struct update -----
    println!("\n=== Struct Update ===");
    struct_update_demo();

    // ----- Tuple struct / newtype -----
    println!("\n=== Newtypes ===");

    let timeout = Milliseconds::from_seconds(30);
    println!("timeout : {}", timeout);
    println!("as secs : {}s", timeout.as_seconds());
    println!("elapsed 5s? {}", timeout.has_elapsed(Milliseconds::from_seconds(5)));
    println!("elapsed 31s? {}", timeout.has_elapsed(Milliseconds::from_seconds(31)));

    // Type safety: this would fail to compile:
    // let b = Bytes(1024);
    // timeout.has_elapsed(b);  // error: expected Milliseconds, got Bytes

    let thumb = Size(160, 120);
    let portrait = Size(100, 200);
    println!("thumb area : {}", thumb.area());
    println!("portrait? thumb={}, portrait={}", thumb.is_portrait(), portrait.is_portrait());

    // Two-field access by position:
    println!("thumb dimensions: {}x{}", thumb.0, thumb.1);

    // ----- Unit struct -----
    println!("\n=== Unit Struct State Machine ===");
    let state = Unconfigured;
    let ready = configure(state);
    // configure(state); // would error: state was moved
    run(ready);
}
