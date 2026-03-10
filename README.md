# DETCORE — Deterministic Logic Core

[![CI](https://github.com/aasyanov/detcore/actions/workflows/ci.yml/badge.svg)](https://github.com/aasyanov/detcore/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/detcore.svg)](https://crates.io/crates/detcore)
[![docs.rs](https://docs.rs/detcore/badge.svg)](https://docs.rs/detcore)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Minimal `no_std` deterministic state machine engine for Rust. Single dependency.

```toml
[dependencies]
detcore = "0.1"
```

## Overview

DETCORE is a strict, generic, zero-allocation execution engine for deterministic state machines. Designed for industrial controllers, embedded systems, safety-critical applications, and any domain where identical inputs must always produce identical outputs.

**Not** a scheduler, async runtime, event bus, or distributed system.

## Architecture

```
Event(seq, ts, payload)
    │
    ▼
Engine::process
    ├── validate seq > last_seq  (monotonic ordering)
    ├── Logic::step(state, event, commands)  (pure transition)
    └── check_invariants()  (post-condition)
    │
    ▼
Vec<Command<C>, 16>  (bounded, stack-allocated)
```

- **Deterministic**: no system clock, no randomness, no heap
- **Fixed-point arithmetic**: `Decimal = i64`, `SCALE = 1_000_000` — avoids float precision issues
- **Monotonic sequencing**: events rejected if `seq` is not strictly increasing
- **Invariant checking**: `State::check_invariants()` runs after every transition
- **Bounded output**: `heapless::Vec<_, 16>` — no allocation, no overflow panic
- **`no_std`**: works on embedded targets, microcontrollers, bare metal

## Quick Start

```rust
use detcore::{Engine, Logic, Event, Command, State, Seq, Timestamp, Vec, SCALE};

#[derive(Clone)]
struct PumpState { pressure: i64, last_seq: Seq }

impl State for PumpState {
    fn check_invariants(&self) -> bool { self.pressure >= 0 }
    fn last_seq(&self) -> Seq { self.last_seq }
    fn set_last_seq(&mut self, seq: Seq) { self.last_seq = seq }
}

#[derive(Clone, Copy)]
enum PumpEvent { PressureUpdate(i64) }

#[derive(Clone, PartialEq, Eq)]
enum PumpCommand { StopPump }

struct PumpLogic;
impl Logic<PumpState, PumpEvent, PumpCommand> for PumpLogic {
    fn step(
        state: &mut PumpState,
        event: Event<PumpEvent>,
        commands: &mut Vec<Command<PumpCommand>, 16>,
    ) {
        match event.payload {
            PumpEvent::PressureUpdate(p) => {
                state.pressure = p;
                if p > 100 * SCALE {
                    let _ = commands.push(Command::Emit(PumpCommand::StopPump));
                }
            }
        }
    }
}

let mut engine = Engine::<PumpState, PumpLogic, PumpEvent, PumpCommand>::new(
    PumpState { pressure: 0, last_seq: Seq(0) },
);

let cmds = engine.process(Event {
    seq: Seq(1),
    ts: Timestamp(0),
    payload: PumpEvent::PressureUpdate(120 * SCALE),
});
// cmds contains: [Emit(StopPump)]
```

## Public API

### Types

| Type | Description |
|---|---|
| `Decimal` | `i64` fixed-point type |
| `SCALE` | `1_000_000` — 6 decimal places of precision |
| `Seq(u64)` | Monotonic event sequence number |
| `Timestamp(u64)` | Externally-injected timestamp (not system clock) |
| `Event<E>` | Envelope: `seq` + `ts` + `payload` |
| `Command<C>` | `Emit(C)` or `NoOp` |

### Traits

| Trait | Methods | Description |
|---|---|---|
| `State` | `check_invariants()`, `last_seq()`, `set_last_seq()` | System state with safety properties |
| `Logic<S,E,C>` | `step(state, event, commands)` | Pure deterministic transition function |

### Engine

| Method | Description |
|---|---|
| `Engine::new(state)` | Create engine with initial state |
| `engine.process(event)` | Process event, return `Vec<Command<C>, 16>` |

## Configuration

### `strict` Feature

Enables `assert!` for sequence validation and invariant checks in release builds. Default behavior uses `debug_assert!` only.

```toml
[dependencies]
detcore = { version = "0.1", features = ["strict"] }
```

| Mode | Seq violation | Invariant failure |
|---|---|---|
| Default (debug) | `debug_assert!` panic | `debug_assert!` panic |
| Default (release) | Silent | Silent |
| `strict` (release) | `assert!` panic | `assert!` panic |

## Test Suite

```
cargo test: 10 passed, 0 failed, 1 doc-test passed
cargo clippy: 0 warnings
```

| Category | Count | Description |
|---|---|---|
| Unit tests | 8 | Pressure safety, emergency stop, invariants, seq monotonicity, fixed-point, capacity, transitions, NoOp |
| Property tests | 2 | Arbitrary pressure values preserve invariants, sequence monotonicity across ranges |
| Doc-tests | 1 | Quick start example in lib.rs |
| Benchmarks | 6 | Single event, state clone, command generation, heapless vs std, large state |

## Dependencies

Single runtime dependency: `heapless 0.7`.

Dev-dependencies: `criterion` (benchmarks), `proptest` (property tests).

## File Structure

```
detcore/
├── src/
│   ├── lib.rs          # Crate root, re-exports
│   ├── types.rs        # Seq, Timestamp, Decimal, SCALE, Event, Command
│   ├── state.rs        # State trait
│   ├── logic.rs        # Logic trait
│   ├── engine.rs       # Engine struct
│   └── tests/
│       ├── fixtures.rs # Shared test types
│       ├── unit.rs     # Unit tests
│       └── property.rs # Property-based tests (proptest)
├── examples/
│   └── industrial_control.rs
├── benches/
│   └── engine_benchmark.rs
├── Cargo.toml
├── LICENSE
├── CHANGELOG.md
└── README.md
```

## License

MIT
