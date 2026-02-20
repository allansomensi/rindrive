#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Waiting,
    Ready,
    Auditing,
    Finished,
}
