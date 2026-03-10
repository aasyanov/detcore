use crate::{Command, Event, State, Vec};

pub trait Logic<S, E, C>
where
    S: State,
    E: Copy,
    C: Clone + PartialEq,
{
    fn step(state: &mut S, event: Event<E>, commands: &mut Vec<Command<C>, 16>);
}
