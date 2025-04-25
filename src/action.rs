//! INFO: Some enumerations that define the execution steps or behaviors and states of the program.

use wayland_protocols::wp::cursor_shape::v1::client::wp_cursor_shape_device_v1::Shape;

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
#[allow(unused)]
#[derive(Default)]
/// INFO: The current behavior, or state, of the program
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
#[allow(clippy::enum_variant_names)]
/// INFO: Defines the state of the screen
pub enum IsFreeze {
    /// Actual screen freeze state.
    NewFrameFreeze,
    /// The last frozen state, used to reset the drawing rectangle.
    OldFrameFreeze,
    /// The actual screen is not frozen.
    UnFreeze,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Hash, Eq)]
/// INFO: Defines operations on rectangles based on mouse position
pub enum EditAction {
    #[default]
    /// The mouse position has nothing to do with the action rectangle
    None,
    /// The mouse is near the left side of the drawing rectangle
    Left,
    /// The mouse is near the right side of the drawing rectangle
    Right,
    /// The mouse is near the top side of the drawing rectangle
    Top,
    /// The mouse is near the bottom side of the drawing rectangle
    Bottom,
    /// The mouse is near the upper left corner of the drawn rectangle
    TopLeft,
    /// The mouse is near the upper right corner of the drawn rectangle
    TopRight,
    /// The mouse is near the lower left corner of the drawn rectangle
    BottomLeft,
    /// The mouse is near the lower right corner of the drawn rectangle
    BottomRight,
    /// The mouse is inside the rectangle, dragging it will move the rectangle.
    Move,
}
impl EditAction {
    #[allow(clippy::wrong_self_convention)]
    pub fn to_cursor_shape(&self) -> Shape {
        match self {
            EditAction::None => Shape::Crosshair,
            EditAction::Left | EditAction::Right => Shape::EwResize, // 左右拖动用水平双向箭头
            EditAction::Top | EditAction::Bottom => Shape::NsResize, // 上下拖动用垂直双向箭头
            EditAction::TopLeft | EditAction::BottomRight => Shape::NwseResize, // 左上-右下用对角双向箭头
            EditAction::TopRight | EditAction::BottomLeft => Shape::NeswResize, // 右上-左下用对角双向箭头
            EditAction::Move => Shape::Move,                                    // 移动用移动光标
        }
    }
}
