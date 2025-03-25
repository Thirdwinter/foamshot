use wayland_client::globals::GlobalListContents;
use wayland_client::protocol::{wl_compositor, wl_registry, wl_seat, wl_surface};
use wayland_client::{Dispatch, Proxy};
use wayland_protocols::wp::cursor_shape::v1::client::{
    wp_cursor_shape_device_v1, wp_cursor_shape_manager_v1,
};
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1;
use wayland_protocols_wlr::screencopy::v1::client::zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1;

use crate::foam_shot::FoamShot;

mod impl_foam_shot;

// NOTE: unimplemented
impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for FoamShot {
    fn event(
        _state: &mut Self,
        _proxy: &wl_registry::WlRegistry,
        _event: <wl_registry::WlRegistry as wayland_client::Proxy>::Event,
        _data: &GlobalListContents,
        _conn: &wayland_client::Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        // todo!()
    }
}

impl Dispatch<wp_cursor_shape_manager_v1::WpCursorShapeManagerV1, ()> for FoamShot {
    fn event(
        _state: &mut Self,
        _proxy: &wp_cursor_shape_manager_v1::WpCursorShapeManagerV1,
        _event: <wp_cursor_shape_manager_v1::WpCursorShapeManagerV1 as Proxy>::Event,
        _data: &(),
        _conn: &wayland_client::Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        // todo!()
    }
}

impl Dispatch<wp_cursor_shape_device_v1::WpCursorShapeDeviceV1, ()> for FoamShot {
    fn event(
        _state: &mut Self,
        _proxy: &wp_cursor_shape_device_v1::WpCursorShapeDeviceV1,
        _event: <wp_cursor_shape_device_v1::WpCursorShapeDeviceV1 as Proxy>::Event,
        _data: &(),
        _conn: &wayland_client::Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        // todo!()
    }
}

impl Dispatch<wl_surface::WlSurface, i32> for FoamShot {
    fn event(
        _state: &mut Self,
        _proxy: &wl_surface::WlSurface,
        _event: <wl_surface::WlSurface as wayland_client::Proxy>::Event,
        _data: &i32,
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
    ) {
        // todo!()
    }
}

impl Dispatch<zwlr_layer_shell_v1::ZwlrLayerShellV1, ()> for FoamShot {
    fn event(
        _state: &mut Self,
        _proxy: &zwlr_layer_shell_v1::ZwlrLayerShellV1,
        _event: <zwlr_layer_shell_v1::ZwlrLayerShellV1 as Proxy>::Event,
        _data: &(),
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
    ) {
        // todo!()
    }
}
// NOTE: 空实现
impl Dispatch<ZwlrScreencopyManagerV1, ()> for FoamShot {
    fn event(
        _state: &mut Self,
        _proxy: &ZwlrScreencopyManagerV1,
        _event: <ZwlrScreencopyManagerV1 as Proxy>::Event,
        _data: &(),
        _conn: &wayland_client::Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        // todo!()
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for FoamShot {
    fn event(
        _state: &mut Self,
        _proxy: &wl_seat::WlSeat,
        _event: <wl_seat::WlSeat as Proxy>::Event,
        _data: &(),
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
    ) {
        // todo!()
    }
}

impl Dispatch<wl_compositor::WlCompositor, ()> for FoamShot {
    fn event(
        _state: &mut Self,
        _proxy: &wl_compositor::WlCompositor,
        _event: <wl_compositor::WlCompositor as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
    ) {
        // todo!()
    }
}
