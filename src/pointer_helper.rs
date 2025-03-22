use wayland_client::protocol::wl_pointer;
use wayland_protocols::wp::cursor_shape::v1::client::wp_cursor_shape_device_v1::Shape;
use wayland_protocols::wp::cursor_shape::v1::client::{
    wp_cursor_shape_device_v1, wp_cursor_shape_manager_v1,
};

#[derive(Default)]
pub struct PointerHelper {
    pub pointer: Option<wl_pointer::WlPointer>,
    pub cursor_shape_manager: Option<wp_cursor_shape_manager_v1::WpCursorShapeManagerV1>,
    pub cursor_shape_device: Option<wp_cursor_shape_device_v1::WpCursorShapeDeviceV1>,

    pub current_pos: Option<(f64, f64)>,
    pub pointer_start: Option<(f64, f64)>,
    pub pointer_end: Option<(f64, f64)>,
}

impl PointerHelper {
    pub fn wl_pointer(&mut self) -> wl_pointer::WlPointer {
        self.pointer.as_ref().unwrap().clone()
    }
    pub fn set_cursor_shape(&mut self, serial: u32, shape: Shape) {
        match self.cursor_shape_device {
            Some(ref device) => device.set_shape(serial, shape),
            _ => (),
        }
    }
}
