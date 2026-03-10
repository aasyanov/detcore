use criterion::{black_box, criterion_group, criterion_main, Criterion};
use detcore::*;

// Example state for benchmarking
#[derive(Clone)]
struct BenchState {
    counter: i64,
    last_seq: Seq,
}

impl State for BenchState {
    fn check_invariants(&self) -> bool {
        self.counter >= 0
    }

    fn last_seq(&self) -> Seq {
        self.last_seq
    }

    fn set_last_seq(&mut self, seq: Seq) {
        self.last_seq = seq;
    }
}

impl Default for BenchState {
    fn default() -> Self {
        Self {
            counter: 0,
            last_seq: Seq(0),
        }
    }
}

#[derive(Clone, Copy)]
enum BenchEvent {
    Increment { value: i64 },
    Reset,
}

#[derive(Clone, PartialEq, Eq)]
enum BenchCommand {
    Updated { new_value: i64 },
    ResetDone,
}

struct BenchLogic;

impl Logic<BenchState, BenchEvent, BenchCommand> for BenchLogic {
    fn step(
        state: &mut BenchState,
        event: Event<BenchEvent>,
        commands: &mut Vec<Command<BenchCommand>, 16>,
    ) {
        match event.payload {
            BenchEvent::Increment { value } => {
                state.counter += value;
                let _ = commands.push(Command::Emit(BenchCommand::Updated {
                    new_value: state.counter,
                }));
            }
            BenchEvent::Reset => {
                state.counter = 0;
                let _ = commands.push(Command::Emit(BenchCommand::ResetDone));
            }
        }
    }
}

fn bench_single_event_processing(c: &mut Criterion) {
    let mut engine =
        Engine::<BenchState, BenchLogic, BenchEvent, BenchCommand>::new(BenchState::default());

    let event = Event {
        seq: Seq(1),
        ts: Timestamp(1000),
        payload: BenchEvent::Increment { value: 42 },
    };

    c.bench_function("single_event_processing", |b| {
        b.iter(|| {
            let commands = engine.process(black_box(event));
            black_box(commands);
        });
    });
}

fn bench_state_clone_overhead(c: &mut Criterion) {
    let state = BenchState {
        counter: 12345,
        last_seq: Seq(100),
    };

    c.bench_function("state_clone", |b| {
        b.iter(|| black_box(state.clone()));
    });
}

fn bench_command_generation(c: &mut Criterion) {
    let mut engine =
        Engine::<BenchState, BenchLogic, BenchEvent, BenchCommand>::new(BenchState::default());

    let events = [
        Event {
            seq: Seq(1),
            ts: Timestamp(1000),
            payload: BenchEvent::Increment { value: 1 },
        },
        Event {
            seq: Seq(2),
            ts: Timestamp(2000),
            payload: BenchEvent::Increment { value: 2 },
        },
        Event {
            seq: Seq(3),
            ts: Timestamp(3000),
            payload: BenchEvent::Increment { value: 3 },
        },
        Event {
            seq: Seq(4),
            ts: Timestamp(4000),
            payload: BenchEvent::Reset,
        },
    ];

    c.bench_function("command_generation_overhead", |b| {
        b.iter(|| {
            let mut total_commands = 0;
            for event in events.iter() {
                let commands = engine.process(black_box(*event));
                total_commands += commands.len();
            }
            black_box(total_commands);
        });
    });
}

fn bench_memory_overhead(c: &mut Criterion) {
    c.bench_function("heapless_vec_operations", |b| {
        b.iter(|| {
            let mut vec = heapless::Vec::<Command<BenchCommand>, 16>::new();
            for i in 0..16 {
                let cmd = if i % 2 == 0 {
                    Command::Emit(BenchCommand::Updated {
                        new_value: i as i64,
                    })
                } else {
                    Command::NoOp
                };
                let _ = vec.push(cmd);
            }
            black_box(vec);
        });
    });
}

fn bench_comparison_with_std(c: &mut Criterion) {
    // Compare with std::vec (simulating traditional approach)
    c.bench_function("std_vec_equivalent", |b| {
        b.iter(|| {
            let mut vec = std::vec::Vec::new();
            for i in 0..16 {
                if i % 2 == 0 {
                    vec.push(i as i64);
                } else {
                    vec.push(0); // NoOp equivalent
                }
            }
            black_box(vec);
        });
    });

    c.bench_function("detcore_heapless_equivalent", |b| {
        b.iter(|| {
            let mut vec = heapless::Vec::<i64, 16>::new();
            for i in 0..16 {
                let _ = vec.push(if i % 2 == 0 { i as i64 } else { 0 });
            }
            black_box(vec);
        });
    });
}

fn bench_large_state_operations(c: &mut Criterion) {
    // Simulate larger state (like industrial systems with many tags)
    #[derive(Clone)]
    struct LargeState {
        tags: [i64; 64], // 64 tags instead of 1
        last_seq: Seq,
    }

    impl State for LargeState {
        fn check_invariants(&self) -> bool {
            self.tags.iter().all(|&x| x >= 0)
        }

        fn last_seq(&self) -> Seq {
            self.last_seq
        }

        fn set_last_seq(&mut self, seq: Seq) {
            self.last_seq = seq;
        }
    }

    impl Default for LargeState {
        fn default() -> Self {
            Self {
                tags: [0; 64],
                last_seq: Seq(0),
            }
        }
    }

    #[derive(Clone, Copy)]
    enum LargeEvent {
        UpdateTag { index: usize, value: i64 },
    }

    #[derive(Clone, PartialEq, Eq)]
    enum LargeCommand {
        TagUpdated { index: usize, value: i64 },
    }

    struct LargeLogic;

    impl Logic<LargeState, LargeEvent, LargeCommand> for LargeLogic {
        fn step(
            state: &mut LargeState,
            event: Event<LargeEvent>,
            commands: &mut Vec<Command<LargeCommand>, 16>,
        ) {
            match event.payload {
                LargeEvent::UpdateTag { index, value } => {
                    if index < state.tags.len() {
                        state.tags[index] = value;
                        let _ =
                            commands.push(Command::Emit(LargeCommand::TagUpdated { index, value }));
                    }
                }
            }
        }
    }

    let mut engine =
        Engine::<LargeState, LargeLogic, LargeEvent, LargeCommand>::new(LargeState::default());

    let event = Event {
        seq: Seq(1),
        ts: Timestamp(1000),
        payload: LargeEvent::UpdateTag {
            index: 5,
            value: 12345,
        },
    };

    c.bench_function("large_state_processing", |b| {
        b.iter(|| {
            let commands = engine.process(black_box(event));
            black_box(commands);
        });
    });
}

criterion_group!(
    benches,
    bench_single_event_processing,
    bench_state_clone_overhead,
    bench_command_generation,
    bench_memory_overhead,
    bench_comparison_with_std,
    bench_large_state_operations
);
criterion_main!(benches);
