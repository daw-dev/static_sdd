#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Shift(usize),
    Reduce(usize),
    Accept,
}
