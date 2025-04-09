#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
#[allow(unused)]
#[derive(Default)]
pub enum Action {
    #[default]
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

