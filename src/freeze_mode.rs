use smithay_client_toolkit::shm::slot::Buffer;
use wayland_client::protocol::{wl_output, wl_surface};
use wayland_protocols_wlr::{
    layer_shell::v1::client::{
        zwlr_layer_shell_v1::{self, Layer},
        zwlr_layer_surface_v1::{self, Anchor, KeyboardInteractivity},
    },
    screencopy::v1::client::{zwlr_screencopy_frame_v1, zwlr_screencopy_manager_v1},
};

use crate::shot_fome::ShotFome;

#[derive(Default)]
pub struct FreezeMode {
    pub surface: Option<wl_surface::WlSurface>,
    // NOTE: freeze screen
    pub screencopy_manager: Option<zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1>,
    pub screencopy_frame: Option<zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1>,
    pub layer_surface: Option<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1>,
    pub buffer: Option<Buffer>,
}

impl FreezeMode {
    pub fn prev_freeze_screen(
        &mut self,
        layer_shell: Option<zwlr_layer_shell_v1::ZwlrLayerShellV1>,
        output: Option<wl_output::WlOutput>,
        qh: Option<wayland_client::QueueHandle<ShotFome>>,
        phys_width: Option<i32>,
        phys_height: Option<i32>,
    ) {
        if let (
            Some(phys_width),
            Some(phys_height),
            Some(screencopy_manager),
            Some(surface),
            Some(layer_shell),
            Some(output),
            Some(qh),
        ) = (
            phys_width,
            phys_height,
            &self.screencopy_manager,
            &self.surface,
            layer_shell,
            output,
            qh,
        ) {
            let screencopy_frame = screencopy_manager.capture_output(true as i32, &output, &qh, ());
            self.screencopy_frame = Some(screencopy_frame);
            // NOTE: 创建layer
            let layer = zwlr_layer_shell_v1::ZwlrLayerShellV1::get_layer_surface(
                &layer_shell,
                &surface,
                Some(&output),
                Layer::Overlay,
                "foam_freeze".to_string(),
                &qh,
                1,
            );
            println!("创建layer");
            layer.set_anchor(Anchor::all());
            layer.set_exclusive_zone(-1); // 将表面扩展到锚定边缘
            layer.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);
            self.layer_surface = Some(layer);

            surface.damage(0, 0, phys_width, phys_height);
            surface.commit();
        };
    }
}
