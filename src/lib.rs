#![no_std]

//! # DETCORE — Deterministic Logic Core
//!
//! Minimal `no_std` deterministic state machine engine for Rust.
//! Single dependency (`heapless`).
//!
//! DETCORE is a strict, generic, zero-allocation execution engine for
//! deterministic state machines. Designed for industrial controllers,
//! embedded systems, safety-critical applications, and any domain where
//! identical inputs must always produce identical outputs.
//!
//! **Not** a scheduler, async runtime, event bus, or distributed system.
//!
//! # Architecture
//!
//! The engine processes events through a pure transition function and
//! produces bounded command output:
//!
//! ```text
//! Event(seq, ts, payload)
//!     │
//!     ▼
//! Engine::process
//!     ├── validate seq > last_seq  (monotonic ordering)
//!     ├── Logic::step(state, event, commands)  (pure transition)
//!     └── check_invariants()  (post-condition)
//!     │
//!     ▼
//! Vec<Command<C>, 16>  (bounded, stack-allocated)
//! ```
//!
//! - **Deterministic**: no system clock, no randomness, no heap
//! - **Fixed-point arithmetic**: `Decimal = i64`, `SCALE = 1_000_000`
//! - **Monotonic sequencing**: events rejected if `seq` is not strictly increasing
//! - **Invariant checking**: `State::check_invariants()` runs after every transition
//! - **Bounded output**: `heapless::Vec<_, 16>` — no allocation, no overflow panic
//!
//! # Quick Start
//!
//! ```rust
//! use detcore::{Engine, Logic, Event, Command, State, Seq, Timestamp, Vec, SCALE};
//!
//! #[derive(Clone)]
//! struct PumpState { pressure: i64, last_seq: Seq }
//!
//! impl State for PumpState {
//!     fn check_invariants(&self) -> bool { self.pressure >= 0 }
//!     fn last_seq(&self) -> Seq { self.last_seq }
//!     fn set_last_seq(&mut self, seq: Seq) { self.last_seq = seq }
//! }
//!
//! #[derive(Clone, Copy)]
//! enum PumpEvent { PressureUpdate(i64) }
//!
//! #[derive(Clone, PartialEq, Eq)]
//! enum PumpCommand { StopPump }
//!
//! struct PumpLogic;
//! impl Logic<PumpState, PumpEvent, PumpCommand> for PumpLogic {
//!     fn step(state: &mut PumpState, event: Event<PumpEvent>, commands: &mut Vec<Command<PumpCommand>, 16>) {
//!         match event.payload {
//!             PumpEvent::PressureUpdate(p) => {
//!                 state.pressure = p;
//!                 if p > 100 * SCALE {
//!                     let _ = commands.push(Command::Emit(PumpCommand::StopPump));
//!                 }
//!             }
//!         }
//!     }
//! }
//!
//! let mut engine = Engine::<PumpState, PumpLogic, PumpEvent, PumpCommand>::new(
//!     PumpState { pressure: 0, last_seq: Seq(0) },
//! );
//! let commands = engine.process(Event {
//!     seq: Seq(1), ts: Timestamp(0),
//!     payload: PumpEvent::PressureUpdate(120 * SCALE),
//! });
//! assert_eq!(commands.len(), 1);
//! ```
//!
//! # Safety Modes
//!
//! | Mode | Behavior |
//! |---|---|
//! | Default (debug) | `debug_assert!` on seq violation and invariant failure |
//! | `--features strict` | `assert!` in release builds — panics on any violation |

pub mod engine;
pub mod logic;
pub mod state;
pub mod types;

pub use engine::*;
pub use logic::*;
pub use state::*;
pub use types::*;

pub use heapless::Vec;

#[cfg(test)]
mod tests;
