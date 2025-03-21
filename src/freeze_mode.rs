use smithay_client_toolkit::shm::slot::Buffer;
use wayland_client::protocol::wl_surface;
use wayland_protocols_wlr::{
    layer_shell::v1::client::zwlr_layer_surface_v1::{self},
    screencopy::v1::client::{zwlr_screencopy_frame_v1, zwlr_screencopy_manager_v1},
};

#[derive(Default)]
pub struct FreezeMode {
    pub surface: Option<wl_surface::WlSurface>,
    // NOTE: freeze screen
    pub screencopy_manager: Option<zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1>,
    pub screencopy_frame: Option<zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1>,
    pub layer_surface: Option<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1>,
    pub buffer: Option<Buffer>,
}

// impl FreezeMode {
//     /// NOTE: 在冻结屏幕前创建一个layer_surface
//     pub fn prev_freeze_screen(
//         &mut self,
//         layer_shell: Option<zwlr_layer_shell_v1::ZwlrLayerShellV1>,
//         output: Option<wl_output::WlOutput>,
//         qh: Option<wayland_client::QueueHandle<ShotFoam>>,
//         phys_width: Option<i32>,
//         phys_height: Option<i32>,
//         hide_cursor: bool,
//     ) {
//         let (phys_width, phys_height, screencopy_manager, surface, layer_shell, output, qh) = check_options!(
//             phys_width,
//             phys_height,
//             self.screencopy_manager.as_ref(),
//             self.surface.as_ref(),
//             layer_shell,
//             output,
//             qh
//         );
//
//         let screencopy_frame =
//             screencopy_manager.capture_output(!hide_cursor as i32, &output, &qh, ());
//         self.screencopy_frame = Some(screencopy_frame);
//
//         // 创建 layer
//         let layer = zwlr_layer_shell_v1::ZwlrLayerShellV1::get_layer_surface(
//             &layer_shell,
//             &surface,
//             Some(&output),
//             Layer::Overlay,
//             "foam_freeze".to_string(),
//             &qh,
//             1,
//         );
//         println!("创建layer");
//         layer.set_anchor(Anchor::all());
//         layer.set_exclusive_zone(-1);
//         layer.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);
//         self.layer_surface = Some(layer);
//
//         surface.damage(0, 0, phys_width, phys_height);
//         surface.commit();
//     }
// }
