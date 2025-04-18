use wayland_protocols::wp::cursor_shape::v1::client::wp_cursor_shape_device_v1::Shape;

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
#[allow(unused)]
#[derive(Default)]
pub enum Action {
    #[default]
    Init,
    WaitPointerPress,
    ToggleFreeze(IsFreeze),
    OnDraw,
    OnEdit(EditAction),
    OnRecorder,
    Output,
    Exit,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub enum IsFreeze {
    NewFrameFreeze,
    OldFrameFreeze,
    UnFreeze,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub enum EditAction {
    #[default]
    None,
    Left,        // 左边
    Right,       // 右边
    Top,         // 上边
    Bottom,      // 下边
    TopLeft,     // 左上角
    TopRight,    // 右上角
    BottomLeft,  // 左下角
    BottomRight, // 右下角
    Move,
}
impl EditAction {
    pub fn to_cursor_shape(&self) -> Shape {
        match self {
            EditAction::None => Shape::Default,
            EditAction::Left | EditAction::Right => Shape::EwResize, // 左右拖动用水平双向箭头
            EditAction::Top | EditAction::Bottom => Shape::NsResize, // 上下拖动用垂直双向箭头
            EditAction::TopLeft | EditAction::BottomRight => Shape::NwseResize, // 左上-右下用对角双向箭头
            EditAction::TopRight | EditAction::BottomLeft => Shape::NeswResize, // 右上-左下用对角双向箭头
            EditAction::Move => Shape::Move,                                    // 移动用移动光标
        }
    }
}
