use smithay_client_toolkit::shm::{
    self,
    slot::{self, Buffer},
};
use wayland_client::{
    QueueHandle,
    protocol::{wl_compositor, wl_keyboard, wl_output, wl_pointer, wl_seat, wl_shm::Format},
};
use wayland_protocols::wp::cursor_shape::v1::client::{
    wp_cursor_shape_device_v1, wp_cursor_shape_manager_v1,
};
use wayland_protocols_wlr::{
    layer_shell::v1::client::zwlr_layer_shell_v1,
    screencopy::v1::client::zwlr_screencopy_manager_v1,
};

use crate::foam_shot::FoamShot;

#[derive(Default)]
pub struct WaylandCtx {
    pub compositor: Option<wl_compositor::WlCompositor>,
    pub output: Option<wl_output::WlOutput>,
    pub pool: Option<slot::SlotPool>,
    pub shm: Option<shm::Shm>,
    pub seat: Option<wl_seat::WlSeat>,
    pub pointer: Option<wl_pointer::WlPointer>,
    pub keyboard: Option<wl_keyboard::WlKeyboard>,
    pub layer_shell: Option<zwlr_layer_shell_v1::ZwlrLayerShellV1>,
    pub qh: Option<QueueHandle<FoamShot>>,

    pub cursor_shape_manager: Option<wp_cursor_shape_manager_v1::WpCursorShapeManagerV1>,
    pub cursor_shape_device: Option<wp_cursor_shape_device_v1::WpCursorShapeDeviceV1>,

    pub current_pos: Option<(f64, f64)>,
    pub start_pos: Option<(f64, f64)>,
    pub end_pos: Option<(f64, f64)>,
    pub screencopy_manager: Option<zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1>,

    pub width: Option<i32>,
    pub height: Option<i32>,
}

impl WaylandCtx {
    pub fn new(shm: shm::Shm, pool: slot::SlotPool, qh: QueueHandle<FoamShot>) -> Self {
        Self {
            qh: Some(qh),
            shm: Some(shm),
            pool: Some(pool),
            ..Default::default()
        }
    }
    pub fn create_buffer(
        &mut self,
        width: i32,
        height: i32,
        stride: i32,
        format: Format,
    ) -> Result<(Buffer, &mut [u8]), String> {
        let pool = self.pool.as_mut().ok_or("Wayland pool not initialized")?;

        let (buffer, canvas) = pool
            .create_buffer(width, height, stride, format)
            .map_err(|e| format!("Wayland buffer creation failed: {}", e))?;

        Ok((buffer, canvas))
    }

    pub fn set_cursor_shape(&mut self, shape: wp_cursor_shape_device_v1::Shape) {
        match &self.cursor_shape_device {
            Some(device) => {
                device.set_shape(1, shape);
            }
            None => {
                let x = self.cursor_shape_manager.as_ref().unwrap().get_pointer(
                    self.pointer.as_ref().unwrap(),
                    self.qh.as_ref().unwrap(),
                    (),
                );
                x.set_shape(1, shape);
                self.cursor_shape_device = Some(x);
            }
        }
    }
}
