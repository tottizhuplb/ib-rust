#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunState {
    Starting,
    Connecting,
    Connected,
    Recovering,
    ShuttingDown,
}
