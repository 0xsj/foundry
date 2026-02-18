# Solution: Pluggable Notification System

## Approach

The solution defines a `Notifier` trait with two required methods and two default methods,
then implements it on three concrete types with different urgent-sending behavior. A
`NotificationRouter` holds `Vec<Box<dyn Notifier>>` for heterogeneous runtime dispatch,
while `send_to_one` uses `impl Notifier` for compile-time static dispatch.

### Key Decisions

**Default methods call required methods**

`send_urgent`'s default implementation calls `self.send()` — a required method. This
is legal because required methods are guaranteed to exist on any implementor. `SmsNotifier`
overrides `send_urgent` with SMS-specific behavior; `EmailNotifier` and `WebhookNotifier`
accept the default. This is the core power of traits over plain interfaces.

**`WebhookNotifier` uses manual Debug to redact secrets**

Deriving `Debug` automatically emits all fields. For any type containing secrets (API
keys, passwords, HMAC secrets), you must write `Debug` manually. The `debug_struct`
builder API in `fmt` makes this explicit and readable.

**`Vec<Box<dyn Notifier>>` for the router, `&impl Notifier` for helpers**

The router needs to hold different notifier types at runtime — only `dyn Trait` supports
this. `send_to_one` only needs to call one notifier and always receives the same concrete
type at each call site — `impl Trait` is the right choice, and it can be inlined.

**`dispatch` uses `Iterator::map` + `collect`**

For the `High` and `Critical` arms, `.iter().map(|ch| ch.send(...)).collect()` is idiomatic
Rust for "apply a function to each element and collect results." It's equivalent to a for loop
that pushes to a Vec, but more composable and harder to get wrong.

**`Priority::PartialOrd`** is derived with variants declared from smallest to largest.
This means `Priority::Normal < Priority::Critical` is true, which can be useful if you
later add threshold comparisons (`if priority >= Priority::High { ... }`).

## Variant Comparison

| Approach | Trade-off |
|---|---|
| `Vec<Box<dyn Notifier>>` (used) | Heterogeneous, runtime dispatch. One vtable indirection per call. |
| `enum NotifierKind { Email(EmailNotifier), Sms(SmsNotifier), ... }` | Homogeneous, static dispatch via match. Adding a new channel type requires modifying the enum. Closed extension. |
| Generic router `Router<T: Notifier>` | All channels must be the same type. Only works for homogeneous fleets. |

## Performance Notes

- `send_to_one(&impl Notifier, ...)`: monomorphized — function is compiled separately for each
  concrete type. The `notifier.send()` call can be inlined by the optimizer.
- `router.dispatch(&dyn Notifier, ...)`: each `ch.send()` is one vtable lookup + pointer
  dereference. For notification sending (I/O-bound), this overhead is immeasurable.
- `Box<dyn Notifier>` in the Vec: each entry is 16 bytes (two pointers) on a 64-bit platform.
  The actual notifier data is on the heap.

## What to Try Next

- Add a `RetryNotifier` wrapper that implements `Notifier` and retries on failure — a real-world
  use of the decorator pattern on top of traits.
- Add a `MulticastNotifier` that holds `Vec<Box<dyn Notifier>>` and fans out — this is essentially
  the `NotificationRouter` as an implementor of `Notifier` (the composite pattern).
- Connect to the Builder pattern: make `NotificationRouterBuilder` that validates channels before
  building.
- Connect to the patterns module: notice how `Notifier` is the Strategy pattern — the router
  doesn't know which notifiers it has, only that they all implement `send`.
