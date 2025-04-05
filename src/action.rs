#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
#[allow(unused)]
pub enum Action {
    Init,
    WaitPointerPress,
    OnDraw,
    Exit,
}

impl Default for Action {
    fn default() -> Self {
        Action::Init
    }
}
