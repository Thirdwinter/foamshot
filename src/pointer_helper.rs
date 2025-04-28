//! INFO: Defines the wayland pointer wrapper, including the use of wp_cursor_shape
use std::error::Error;

use wayland_client::QueueHandle;
use wayland_client::protocol::wl_pointer;
use wayland_protocols::wp::cursor_shape::v1::client::{
    wp_cursor_shape_device_v1, wp_cursor_shape_manager_v1,
};

use crate::foamcore::FoamShot;

#[derive(Default)]
pub struct PointerHelper {
    pub pointer: Option<wl_pointer::WlPointer>,
    // TODO:
    // pub cursor_surface: Option<wl_surface::WlSurface>,
    pub cursor_shape_manager: Option<(wp_cursor_shape_manager_v1::WpCursorShapeManagerV1, u32)>,
    pub cursor_shape_device: Option<wp_cursor_shape_device_v1::WpCursorShapeDeviceV1>,

    pub g_current_pos: Option<(f64, f64)>,
    pub g_start_pos: Option<(f64, f64)>,
    pub g_end_pos: Option<(f64, f64)>,

    /// 记录index
    pub start_index: Option<usize>,
    pub end_index: Option<usize>,

    /// 最新的surface enter 序列号
    pub serial: u32,
}

impl PointerHelper {
    /// 确保cursor_shape_device存在
    #[inline(always)]
    fn ensure_cursor_device(
        &mut self,
        qh: &QueueHandle<FoamShot>,
        pointer: &wl_pointer::WlPointer,
    ) -> Result<(), Box<dyn Error>> {
        if self.cursor_shape_device.is_none() {
            let manager = self
                .cursor_shape_manager
                .as_ref()
                .ok_or("Cursor shape manager is not initialized")?;
            let device = manager.0.get_pointer(pointer, qh, ());
            self.cursor_shape_device = Some(device);
        }
        Ok(())
    }

    /// 链式设置光标形状
    #[inline(always)]
    pub fn set_cursor_shape(
        &mut self,
        qh: &QueueHandle<FoamShot>,
        shape: wp_cursor_shape_device_v1::Shape,
        pointer: &wl_pointer::WlPointer,
    ) -> Result<(), Box<dyn Error>> {
        // 确保设备已初始化
        self.ensure_cursor_device(qh, pointer)?;

        // 安全unwrap，因为ensure_cursor_device保证设备存在
        self.cursor_shape_device
            .as_ref()
            .unwrap()
            .set_shape(self.serial, shape);

        Ok(())
    }
}
