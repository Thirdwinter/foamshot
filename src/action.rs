#[derive(Debug, Clone, Copy)]
#[allow(unused)]
pub enum Action {
    PRELOAD,
    FREEZE,
    Onselect,
    AfterSelect,
    EXIT,
}

impl Default for Action {
    fn default() -> Self {
        Action::PRELOAD
    }
}
