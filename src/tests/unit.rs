use super::fixtures::*;
use crate::*;

#[test]
fn pressure_safety() {
    let mut engine = Engine::<IndustrialState, IndustrialLogic, EventKind, CommandKind>::new(
        IndustrialState::default(),
    );

    let cmds = engine.process(Event {
        seq: Seq(1),
        ts: Timestamp(1000),
        payload: EventKind::PressureUpdate {
            tag_id: 1,
            value: 80 * SCALE,
        },
    });
    assert_eq!(cmds.len(), 0);

    let cmds = engine.process(Event {
        seq: Seq(2),
        ts: Timestamp(2000),
        payload: EventKind::PressureUpdate {
            tag_id: 1,
            value: 120 * SCALE,
        },
    });
    assert_eq!(cmds.len(), 2);
    assert!(cmds.contains(&Command::Emit(CommandKind::StopPump)));
    assert!(cmds.contains(&Command::Emit(CommandKind::TriggerAlarm { id: 101 })));
    assert!(!engine.state.pump_active);
}

#[test]
fn emergency_stop() {
    let mut engine = Engine::<IndustrialState, IndustrialLogic, EventKind, CommandKind>::new(
        IndustrialState::default(),
    );

    let cmds = engine.process(Event {
        seq: Seq(1),
        ts: Timestamp(1000),
        payload: EventKind::EmergencyStop,
    });
    assert_eq!(cmds.len(), 1);
    assert!(cmds.contains(&Command::Emit(CommandKind::StopPump)));
    assert!(!engine.state.pump_active);
}

#[test]
fn invariants_hold_across_transitions() {
    let mut engine = Engine::<IndustrialState, IndustrialLogic, EventKind, CommandKind>::new(
        IndustrialState::default(),
    );

    let events = [
        Event {
            seq: Seq(1),
            ts: Timestamp(1000),
            payload: EventKind::PressureUpdate {
                tag_id: 1,
                value: 50 * SCALE,
            },
        },
        Event {
            seq: Seq(2),
            ts: Timestamp(2000),
            payload: EventKind::PressureUpdate {
                tag_id: 1,
                value: 150 * SCALE,
            },
        },
        Event {
            seq: Seq(3),
            ts: Timestamp(3000),
            payload: EventKind::EmergencyStop,
        },
    ];

    for event in events {
        let _ = engine.process(event);
        assert!(engine.state.check_invariants());
    }
}

#[test]
fn seq_monotonicity() {
    let mut engine = Engine::<IndustrialState, IndustrialLogic, EventKind, CommandKind>::new(
        IndustrialState::default(),
    );

    let _ = engine.process(Event {
        seq: Seq(1),
        ts: Timestamp(1000),
        payload: EventKind::PressureUpdate {
            tag_id: 1,
            value: 50 * SCALE,
        },
    });
    assert_eq!(engine.state.last_seq(), Seq(1));

    let _ = engine.process(Event {
        seq: Seq(2),
        ts: Timestamp(2000),
        payload: EventKind::PressureUpdate {
            tag_id: 1,
            value: 60 * SCALE,
        },
    });
    assert_eq!(engine.state.last_seq(), Seq(2));
}

#[test]
fn fixed_point_scale() {
    assert_eq!(50i64 * SCALE, 50_000_000);

    let mut engine = Engine::<IndustrialState, IndustrialLogic, EventKind, CommandKind>::new(
        IndustrialState::default(),
    );
    let cmds = engine.process(Event {
        seq: Seq(1),
        ts: Timestamp(1000),
        payload: EventKind::PressureUpdate {
            tag_id: 1,
            value: 50 * SCALE,
        },
    });
    assert_eq!(engine.state.pressure, 50 * SCALE);
    assert_eq!(cmds.len(), 0);
}

#[test]
fn command_buffer_saturates_at_capacity() {
    struct FloodLogic;

    impl Logic<IndustrialState, EventKind, CommandKind> for FloodLogic {
        fn step(
            _: &mut IndustrialState,
            _: Event<EventKind>,
            commands: &mut Vec<Command<CommandKind>, 16>,
        ) {
            for i in 0..20 {
                let _ = commands.push(Command::Emit(CommandKind::TriggerAlarm { id: i }));
            }
        }
    }

    let mut engine = Engine::<IndustrialState, FloodLogic, EventKind, CommandKind>::new(
        IndustrialState::default(),
    );
    let cmds = engine.process(Event {
        seq: Seq(1),
        ts: Timestamp(1000),
        payload: EventKind::EmergencyStop,
    });

    assert_eq!(cmds.len(), 16);
    assert!(engine.state.check_invariants());
}

#[test]
fn full_state_transition_sequence() {
    let mut engine = Engine::<IndustrialState, IndustrialLogic, EventKind, CommandKind>::new(
        IndustrialState::default(),
    );

    assert_eq!(engine.state.pressure, 0);
    assert!(engine.state.pump_active);

    let cmds = engine.process(Event {
        seq: Seq(1),
        ts: Timestamp(1000),
        payload: EventKind::PressureUpdate {
            tag_id: 1,
            value: 50 * SCALE,
        },
    });
    assert_eq!(cmds.len(), 0);
    assert_eq!(engine.state.pressure, 50 * SCALE);
    assert!(engine.state.pump_active);

    let cmds = engine.process(Event {
        seq: Seq(2),
        ts: Timestamp(2000),
        payload: EventKind::PressureUpdate {
            tag_id: 1,
            value: 120 * SCALE,
        },
    });
    assert_eq!(cmds.len(), 2);
    assert!(!engine.state.pump_active);

    let cmds = engine.process(Event {
        seq: Seq(3),
        ts: Timestamp(3000),
        payload: EventKind::EmergencyStop,
    });
    assert_eq!(cmds.len(), 1);
    assert_eq!(engine.state.pressure, 120 * SCALE);
}

#[test]
fn noop_commands() {
    struct NoOpLogic;

    impl Logic<IndustrialState, EventKind, CommandKind> for NoOpLogic {
        fn step(
            _: &mut IndustrialState,
            _: Event<EventKind>,
            commands: &mut Vec<Command<CommandKind>, 16>,
        ) {
            let _ = commands.push(Command::NoOp);
            let _ = commands.push(Command::Emit(CommandKind::StopPump));
            let _ = commands.push(Command::NoOp);
        }
    }

    let mut engine = Engine::<IndustrialState, NoOpLogic, EventKind, CommandKind>::new(
        IndustrialState::default(),
    );
    let cmds = engine.process(Event {
        seq: Seq(1),
        ts: Timestamp(1000),
        payload: EventKind::EmergencyStop,
    });

    assert_eq!(cmds.len(), 3);
    assert!(cmds.contains(&Command::NoOp));
    assert!(cmds.contains(&Command::Emit(CommandKind::StopPump)));
}
