// pub mod editor_mode;
// pub mod freeze_mode;
// pub mod select_mode;

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
#[allow(unused)]
pub enum Mode {
    Init,
    OnFreeze,
    OnDraw,
    Exit,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Init
    }
}
