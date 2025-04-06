use wayland_client::QueueHandle;
use wayland_client::protocol::wl_pointer;
use wayland_protocols::wp::cursor_shape::v1::client::{
    wp_cursor_shape_device_v1, wp_cursor_shape_manager_v1,
};

use crate::foamshot::FoamShot;

#[derive(Default)]
pub struct PointerHelper {
    pub pointer: Option<wl_pointer::WlPointer>,

    pub cursor_shape_manager: Option<(wp_cursor_shape_manager_v1::WpCursorShapeManagerV1, u32)>,
    pub cursor_shape_device: Option<wp_cursor_shape_device_v1::WpCursorShapeDeviceV1>,

    pub current_pos: Option<(f64, f64)>,
    pub start_pos: Option<(f64, f64)>,
    pub end_pos: Option<(f64, f64)>,

    /// 记录index
    pub start_index: Option<usize>,
    pub end_index: Option<usize>,

    /// 是否在按下
    pub is_pressing: bool,
}

impl PointerHelper {
    /// 确保cursor_shape_device存在
    #[inline(always)]
    fn ensure_cursor_device(
        &mut self,
        qh: &QueueHandle<FoamShot>,
        pointer: &wl_pointer::WlPointer,
    ) {
        if self.cursor_shape_device.is_none() {
            let manager = &self.cursor_shape_manager.as_ref().unwrap().0;
            // let pointer = self.pointer.as_ref().unwrap();
            let device = manager.get_pointer(pointer, qh, ());
            self.cursor_shape_device = Some(device);
        }
    }

    /// 链式设置光标形状
    #[inline(always)]
    pub fn set_cursor_shape(
        &mut self,
        qh: &QueueHandle<FoamShot>,
        serial: u32,
        shape: wp_cursor_shape_device_v1::Shape,
        pointer: &wl_pointer::WlPointer,
    ) -> &mut Self {
        // 确保设备已初始化
        self.ensure_cursor_device(qh, pointer);

        // 安全unwrap，因为ensure_cursor_device保证设备存在
        self.cursor_shape_device
            .as_ref()
            .unwrap()
            .set_shape(serial, shape);

        // 返回self支持链式调用
        self
    }
}
