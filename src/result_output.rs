use smithay_client_toolkit::shm::slot::Buffer;
use wayland_client::protocol::wl_surface;
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1;
use wayland_protocols_wlr::screencopy::v1::client::{
    zwlr_screencopy_frame_v1, zwlr_screencopy_manager_v1,
};

#[derive(Default)]
pub struct ResultOutput {
    pub _surface: Option<wl_surface::WlSurface>,
    // NOTE: freeze screen
    pub screencopy_manager: Option<zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1>,
    pub _screencopy_frame: Option<zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1>,
    pub _layer_surface: Option<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1>,
    pub buffer: Option<Buffer>,
    pub start: Option<(i32, i32)>,
    pub width: Option<i32>,
    pub height: Option<i32>,
}
