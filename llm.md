# DETCORE — LLM Reference

> Crate: `detcore` | Rust 1.81+ | `no_std` | Single dependency: `heapless 0.7`

## CRITICAL RULES

1. `Engine` is generic: `Engine<S, L, E, C>` where `S: State`, `L: Logic<S,E,C>`, `E: Copy`, `C: Clone + PartialEq`
2. `Logic::step` is a **pure function** — same `(state, event)` must always produce same `(new_state, commands)`
3. **No system clock, no randomness, no heap** — time is injected via `Timestamp(u64)`
4. `commands` buffer is `heapless::Vec<Command<C>, 16>` — push returns `Err` when full, never panics
5. `Seq` must be **strictly monotonic increasing** — engine rejects out-of-order events
6. `State::check_invariants()` is called **after every** `process()` call

---

## API

```rust
// Create
let mut engine = Engine::<MyState, MyLogic, MyEvent, MyCommand>::new(initial_state);

// Process
let commands: Vec<Command<MyCommand>, 16> = engine.process(Event {
    seq: Seq(1),
    ts: Timestamp(1000),
    payload: MyEvent::Something,
});

// Inspect
engine.state  // pub field
```

## TYPES

```rust
type Decimal = i64;
const SCALE: i64 = 1_000_000;

struct Seq(pub u64);         // monotonic sequence number
struct Timestamp(pub u64);   // externally-injected timestamp

struct Event<E> {
    pub seq: Seq,
    pub ts: Timestamp,
    pub payload: E,
}

enum Command<C> {
    Emit(C),
    NoOp,
}
```

## TRAITS

```rust
trait State: Clone {
    fn check_invariants(&self) -> bool;
    fn last_seq(&self) -> Seq;
    fn set_last_seq(&mut self, seq: Seq);
}

trait Logic<S: State, E: Copy, C: Clone + PartialEq> {
    fn step(state: &mut S, event: Event<E>, commands: &mut Vec<Command<C>, 16>);
}
```

## ENGINE INTERNALS

```rust
pub struct Engine<S, L, E, C> {
    pub state: S,
    _logic: PhantomData<L>,
    _event: PhantomData<E>,
    _command: PhantomData<C>,
}
```

### `process()` flow

1. Validate `event.seq > state.last_seq()` (`debug_assert!` or `assert!` with `strict`)
2. `state.set_last_seq(event.seq)`
3. `L::step(&mut state, event, &mut commands)`
4. `debug_assert!(state.check_invariants())` (or `assert!` with `strict`)
5. Return `commands`

## FEATURE FLAGS

| Feature | Effect |
|---|---|
| (default) | `debug_assert!` for seq and invariant checks |
| `strict` | `assert!` in release builds — panics on any violation |

---

## IMPLEMENTING A SYSTEM

### 1. Define state

```rust
#[derive(Clone)]
struct MyState {
    last_seq: Seq,
    // ... your fields
}

impl State for MyState {
    fn check_invariants(&self) -> bool { /* safety properties */ }
    fn last_seq(&self) -> Seq { self.last_seq }
    fn set_last_seq(&mut self, seq: Seq) { self.last_seq = seq; }
}
```

### 2. Define events and commands

```rust
#[derive(Clone, Copy)]      // E: Copy required
enum MyEvent { ... }

#[derive(Clone, PartialEq, Eq)]  // C: Clone + PartialEq required
enum MyCommand { ... }
```

### 3. Implement logic

```rust
struct MyLogic;
impl Logic<MyState, MyEvent, MyCommand> for MyLogic {
    fn step(state: &mut MyState, event: Event<MyEvent>, commands: &mut Vec<Command<MyCommand>, 16>) {
        // Pure function: no I/O, no system calls, no randomness
    }
}
```

### 4. Run

```rust
let mut engine = Engine::<MyState, MyLogic, MyEvent, MyCommand>::new(MyState { ... });
let commands = engine.process(Event { seq: Seq(1), ts: Timestamp(0), payload: ... });
```

## MISTAKES TO AVOID

1. **Non-determinism in Logic::step** — no `SystemTime::now()`, no `rand`, no `HashMap` iteration order
2. **Ignoring push result** — `commands.push()` returns `Err` when buffer full; use `let _ =` to acknowledge
3. **Negative Seq gap** — `Seq(5)` after `Seq(3)` is OK; `Seq(3)` after `Seq(5)` panics in debug
4. **Float arithmetic** — use `Decimal` (`i64`) with `SCALE` for deterministic math
5. **Mutable statics in Logic** — breaks determinism; all state must be in `S`
6. **Exceeding 16 commands** — extra pushes silently fail; design logic to stay within limit

## NOT IN SCOPE

No networking, no persistence, no async, no scheduling, no heap allocation, no floating-point.
