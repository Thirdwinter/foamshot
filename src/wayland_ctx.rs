use std::collections::HashMap;

use log::{debug, error, warn};
use smithay_client_toolkit::shm::{
    self,
    slot::{self, Buffer},
};
use wayland_client::{
    QueueHandle,
    protocol::{wl_compositor, wl_keyboard, wl_output, wl_pointer, wl_seat},
};
use wayland_protocols::wp::cursor_shape::v1::client::{wp_cursor_shape_device_v1, wp_cursor_shape_manager_v1};
use wayland_protocols_wlr::{
    layer_shell::v1::client::zwlr_layer_shell_v1,
    screencopy::v1::client::{zwlr_screencopy_frame_v1, zwlr_screencopy_manager_v1},
};

use crate::foam_shot::FoamShot;

#[derive(Default)]
pub struct WaylandCtx {
    /// 全局唯一
    /// u32 is wl_registry name
    pub compositor: Option<(wl_compositor::WlCompositor, u32)>,
    pub seat: Option<(wl_seat::WlSeat, u32)>,
    pub pointer: Option<wl_pointer::WlPointer>,
    pub keyboard: Option<wl_keyboard::WlKeyboard>,
    pub qh: Option<QueueHandle<FoamShot>>,
    pub shm: Option<shm::Shm>,
    pub pool: Option<slot::SlotPool>,
    pub screencopy_manager: Option<(zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1, u32)>,
    pub layer_shell: Option<(zwlr_layer_shell_v1::ZwlrLayerShellV1, u32)>,

    /// 每个输出设备一个
    pub outputs: Option<Vec<wl_output::WlOutput>>,
    pub widths: Option<HashMap<usize, i32>>,
    pub heights: Option<HashMap<usize, i32>>,
    pub scales: Option<HashMap<usize, i32>>,
    /// 初始copy的屏幕
    pub base_buffers: Option<HashMap<usize, Buffer>>,
    pub screencopy_frame: Option<HashMap<usize, zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1>>,
    pub frames_ready: usize,

    /// 光标管理器
    pub cursor_shape_manager: Option<(wp_cursor_shape_manager_v1::WpCursorShapeManagerV1, u32)>,
    pub cursor_shape_device: Option<wp_cursor_shape_device_v1::WpCursorShapeDeviceV1>,
    pub current_pos: Option<(f64, f64)>,
    pub start_pos: Option<(f64, f64)>,
    pub end_pos: Option<(f64, f64)>,
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

    pub fn request_screencopy(&mut self) {
        debug!("发起屏幕copy请求");
        let screencopy_manager = if let Some((ref manager, _)) = self.screencopy_manager {
            manager
        } else {
            error!("screencopy_manager 未初始化");
            return;
        };

        let qh = if let Some(ref qh) = self.qh {
            qh
        } else {
            error!("QueueHandle 未初始化");
            return;
        };

        // 遍历所有 outputs
        if let Some(ref outputs) = self.outputs {
            let mut frames = HashMap::new();
            for (index, output) in outputs.iter().enumerate() {
                let frame = screencopy_manager.capture_output(true as i32, output, qh, index);
                frames.insert(index, frame);
            }
            self.screencopy_frame = Some(frames);
        } else {
            error!("无可用 outputs");
        }
    }

    pub fn test(&mut self) {
        if self.base_buffers.is_none() {
            println!("error to copy")
        } else {
            println!("ok")
        }
    }
}

// impl WaylandCtx {
//     pub fn new(shm: shm::Shm, pool: slot::SlotPool, qh: QueueHandle<FoamShot>) -> Self {
//         Self {
//             qh: Some(qh),
//             shm: Some(shm),
//             pool: Some(pool),
//             ..Default::default()
//         }
//     }
//
//     /// Create a buffer
//     pub fn create_buffer(
//         &mut self,
//         width: i32,
//         height: i32,
//         stride: i32,
//         format: Format,
//     ) -> Result<(Buffer, &mut [u8]), String> {
//         let pool = self.pool.as_mut().ok_or("Wayland pool not initialized")?;
//
//         let (buffer, canvas) = pool
//             .create_buffer(width, height, stride, format)
//             .map_err(|e| format!("Wayland buffer creation failed: {}", e))?;
//
//         Ok((buffer, canvas))
//     }
//
//     /// Set the cursor shape
//     pub fn set_cursor_shape(&mut self, shape: wp_cursor_shape_device_v1::Shape) {
//         if let Some(device) = &self.cursor_shape_device {
//             device.set_shape(1, shape);
//             return;
//         }
//
//         let manager = match self.cursor_shape_manager.as_ref() {
//             Some(manager) => manager,
//             None => return,
//         };
//
//         let pointer = match self.pointer.as_ref() {
//             Some(pointer) => pointer,
//             None => return,
//         };
//
//         let qh = match self.qh.as_ref() {
//             Some(qh) => qh,
//             None => return,
//         };
//
//         let device = manager.get_pointer(pointer, qh, ());
//         device.set_shape(1, shape);
//         self.cursor_shape_device = Some(device);
//     }
// }
