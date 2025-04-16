use wayland_client::{Dispatch, Proxy};
use wayland_protocols::wp::cursor_shape::v1::client::{
    wp_cursor_shape_device_v1, wp_cursor_shape_manager_v1,
};

use crate::foamcore::FoamShot;

// NOTE: unused
#[allow(unused_variables)]
impl Dispatch<wp_cursor_shape_manager_v1::WpCursorShapeManagerV1, ()> for FoamShot {
    fn event(
        app: &mut Self,
        proxy: &wp_cursor_shape_manager_v1::WpCursorShapeManagerV1,
        event: <wp_cursor_shape_manager_v1::WpCursorShapeManagerV1 as Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
    }
}
// NOTE: unused
#[allow(unused_variables)]
impl Dispatch<wp_cursor_shape_device_v1::WpCursorShapeDeviceV1, ()> for FoamShot {
    fn event(
        app: &mut Self,
        proxy: &wp_cursor_shape_device_v1::WpCursorShapeDeviceV1,
        event: <wp_cursor_shape_device_v1::WpCursorShapeDeviceV1 as Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
    }
}
