pub type Decimal = i64;
pub const SCALE: i64 = 1_000_000;

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct Seq(pub u64);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Timestamp(pub u64);

#[derive(Clone, Copy, Debug)]
pub struct Event<E> {
    pub seq: Seq,
    pub ts: Timestamp,
    pub payload: E,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Command<C> {
    Emit(C),
    NoOp,
}
