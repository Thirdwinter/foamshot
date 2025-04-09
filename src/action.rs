#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
#[allow(unused)]
pub enum Action {
    Init,
    WaitPointerPress,
    ToggleFreeze(IsFreeze),
    OnDraw,
    Exit,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub enum IsFreeze {
    Freeze,
    UnFreeze,
}

impl Default for Action {
    fn default() -> Self {
        Action::Init
    }
}
