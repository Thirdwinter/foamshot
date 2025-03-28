use wayland_client::protocol::wl_pointer;
use wayland_protocols::wp::cursor_shape::v1::client::{
    wp_cursor_shape_device_v1, wp_cursor_shape_manager_v1,
};

#[derive(Default)]
pub struct PointerHelper {
    pub pointer: Option<wl_pointer::WlPointer>,

    pub cursor_shape_manager: Option<(wp_cursor_shape_manager_v1::WpCursorShapeManagerV1, u32)>,
    pub cursor_shape_device: Option<wp_cursor_shape_device_v1::WpCursorShapeDeviceV1>,
    pub current_pos: Option<(f64, f64)>,
    pub start_pos: Option<(f64, f64)>,
    pub end_pos: Option<(f64, f64)>,
}

impl PointerHelper {
    pub fn get_pointer(&self) -> &wl_pointer::WlPointer {
        self.pointer.as_ref().unwrap()
    }
}
