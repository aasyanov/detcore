use crate::{Command, Event, Logic, State, Vec};

pub struct Engine<S, L, E, C>
where
    L: Logic<S, E, C>,
    S: State,
    E: Copy,
    C: Clone + PartialEq,
{
    pub state: S,
    _logic: core::marker::PhantomData<L>,
    _event: core::marker::PhantomData<E>,
    _command: core::marker::PhantomData<C>,
}

impl<S, L, E, C> Engine<S, L, E, C>
where
    L: Logic<S, E, C>,
    S: State,
    E: Copy,
    C: Clone + PartialEq,
{
    pub fn new(initial_state: S) -> Self {
        #[cfg(feature = "strict")]
        assert!(
            initial_state.check_invariants(),
            "Initial state violates invariants"
        );

        Self {
            state: initial_state,
            _logic: core::marker::PhantomData,
            _event: core::marker::PhantomData,
            _command: core::marker::PhantomData,
        }
    }

    pub fn process(&mut self, event: Event<E>) -> Vec<Command<C>, 16> {
        let mut commands = Vec::new();

        let last_seq = self.state.last_seq();
        debug_assert!(event.seq > last_seq, "Event seq not monotonic");
        #[cfg(feature = "strict")]
        assert!(event.seq > last_seq, "Event seq not monotonic");

        self.state.set_last_seq(event.seq);
        L::step(&mut self.state, event, &mut commands);

        debug_assert!(self.state.check_invariants(), "State invariants violated");
        #[cfg(feature = "strict")]
        assert!(self.state.check_invariants(), "State invariants violated");

        commands
    }
}
