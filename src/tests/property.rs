use super::fixtures::*;
use crate::*;
use proptest::prelude::*;

proptest! {
    #[test]
    fn any_pressure_preserves_invariants(pressure in 0..i64::MAX) {
        let mut engine = Engine::<IndustrialState, IndustrialLogic, EventKind, CommandKind>::new(IndustrialState::default());

        let _ = engine.process(Event {
            seq: Seq(1),
            ts: Timestamp(1000),
            payload: EventKind::PressureUpdate { tag_id: 1, value: pressure },
        });

        prop_assert!(engine.state.check_invariants());
    }

    #[test]
    fn seq_remains_monotonic(seq1 in 1..1000u64, seq2 in 1001..2000u64) {
        let mut engine = Engine::<IndustrialState, IndustrialLogic, EventKind, CommandKind>::new(IndustrialState::default());

        let _ = engine.process(Event {
            seq: Seq(seq1), ts: Timestamp(1000),
            payload: EventKind::PressureUpdate { tag_id: 1, value: 50 * SCALE },
        });

        let _ = engine.process(Event {
            seq: Seq(seq2), ts: Timestamp(2000),
            payload: EventKind::PressureUpdate { tag_id: 1, value: 60 * SCALE },
        });

        prop_assert_eq!(engine.state.last_seq(), Seq(seq2));
    }
}
