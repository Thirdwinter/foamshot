#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
#[allow(unused)]
#[derive(Default)]
pub enum Action {
    #[default]
    Init,
    WaitPointerPress,
    ToggleFreeze(IsFreeze),
    OnDraw,
    OnEdit,
    Exit,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub enum IsFreeze {
    NewFrameFreeze,
    OldFrameFreeze,
    UnFreeze,
}
