use log::{debug, info};
use smithay_client_toolkit::shm::slot::Buffer;
use wayland_client::protocol::wl_surface;
use wayland_protocols_wlr::{
    layer_shell::v1::client::{
        zwlr_layer_shell_v1::{self, Layer},
        zwlr_layer_surface_v1::{self, Anchor, KeyboardInteractivity},
    },
    screencopy::v1::client::zwlr_screencopy_frame_v1,
};

use crate::wayland_ctx::WaylandCtx;

#[derive(Default)]
pub struct FreezeMode {
    pub surface: Option<wl_surface::WlSurface>,
    pub screencopy_frame: Option<zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1>,
    pub layer_surface: Option<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1>,
    pub buffer: Option<Buffer>,
    pub hide_cursor: bool,
}

impl FreezeMode {
    pub fn new(hide_cursor: bool) -> Self {
        Self {
            hide_cursor,
            ..FreezeMode::default()
        }
    }
    pub fn before(&mut self, wl_ctx: &mut WaylandCtx) {
        self.surface = Some(
            wl_ctx
                .compositor
                .as_ref()
                .unwrap()
                .create_surface(wl_ctx.qh.as_mut().unwrap(), 1),
        );
        info!("create freeze_surface");

        // NOTE: 发起屏幕copy请求
        debug!("发起屏幕copy请求");
        self.screencopy_frame = Some(wl_ctx.screencopy_manager.as_ref().unwrap().capture_output(
            true as i32,
            wl_ctx.output.as_ref().unwrap(),
            &wl_ctx.qh.clone().unwrap(),
            (),
        ));
        // 创建 layer
        let layer = zwlr_layer_shell_v1::ZwlrLayerShellV1::get_layer_surface(
            &wl_ctx.layer_shell.as_ref().unwrap(),
            &self.surface.as_ref().unwrap(),
            wl_ctx.output.as_ref(),
            Layer::Overlay,
            "foam_freeze".to_string(),
            &wl_ctx.qh.clone().unwrap(),
            1,
        );
        layer.set_anchor(Anchor::all());
        layer.set_exclusive_zone(-1);
        layer.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);
        self.layer_surface = Some(layer);

        info!("create freeze_layer");
        self.surface
            .as_ref()
            .unwrap()
            .damage(0, 0, wl_ctx.width.unwrap(), wl_ctx.height.unwrap());
        self.surface.as_ref().unwrap().commit();
        debug!("after freeze before hook")
    }

    pub fn on(&mut self, wl_ctx: &mut WaylandCtx) {
        self.buffer
            .as_ref()
            .unwrap()
            .attach_to(self.surface.as_ref().unwrap())
            .unwrap();
        self.surface
            .as_ref()
            .unwrap()
            .damage(0, 0, wl_ctx.width.unwrap(), wl_ctx.height.unwrap());
        self.surface.as_ref().unwrap().commit();
    }
    #[allow(unused)]
    pub fn after(&mut self) {}
}
