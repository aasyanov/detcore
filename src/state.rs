pub trait State: Clone {
    fn check_invariants(&self) -> bool;
    fn last_seq(&self) -> crate::Seq;
    fn set_last_seq(&mut self, seq: crate::Seq);
}
