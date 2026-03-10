# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] — 2026-03-10

Initial public release.

### Added

- `Engine<S, L, E, C>` generic deterministic state machine with `new()` and `process()`.
- `Logic<S, E, C>` trait for pure state transition functions.
- `State` trait with `check_invariants()`, `last_seq()`, `set_last_seq()`.
- `Event<E>` envelope with `Seq`, `Timestamp`, and generic payload.
- `Command<C>` enum (`Emit(C)` | `NoOp`) with bounded `heapless::Vec<_, 16>` output.
- `Decimal` (`i64`) fixed-point type with `SCALE = 1_000_000` for deterministic arithmetic.
- `Seq(u64)` monotonic sequence number with strict ordering enforcement.
- `Timestamp(u64)` externally-injected timestamp for clock-independent execution.
- `strict` feature flag for `assert!`-based checks in release builds (default: `debug_assert!` only).
- `#![no_std]` compatibility — works on embedded targets without heap allocation.
- 8 unit tests, 2 property-based tests (proptest), 1 doc-test.
- 6 Criterion benchmarks: single event, state clone, command generation, heapless vs std, large state.
- Industrial pressure controller example (`examples/industrial_control.rs`).

### Dependencies

- `heapless 0.7` (only runtime dependency)

[Unreleased]: https://github.com/aasyanov/detcore/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/aasyanov/detcore/releases/tag/v0.1.0
