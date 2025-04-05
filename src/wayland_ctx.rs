use std::{
    cmp::{max, min},
    collections::HashMap,
};

use log::{debug, error};
use smithay_client_toolkit::shm::{
    self,
    slot::{self},
};
use wayland_client::{
    QueueHandle,
    protocol::{wl_compositor, wl_keyboard, wl_seat},
};
use wayland_protocols::{
    wp::viewporter::client::wp_viewporter,
    xdg::{shell::client::xdg_wm_base, xdg_output::zv1::client::zxdg_output_manager_v1},
};
use wayland_protocols_wlr::{
    layer_shell::v1::client::zwlr_layer_shell_v1,
    screencopy::v1::client::zwlr_screencopy_manager_v1,
};

use crate::{foam_outputs, foamshot::FoamShot, pointer_helper::PointerHelper};

#[derive(Default)]
pub struct WaylandCtx {
    /// 全局唯一
    /// u32 is wl_registry name
    pub compositor: Option<(wl_compositor::WlCompositor, u32)>,
    pub seat: Option<(wl_seat::WlSeat, u32)>,
    pub keyboard: Option<wl_keyboard::WlKeyboard>,
    pub qh: Option<QueueHandle<FoamShot>>,
    pub shm: Option<shm::Shm>,
    pub pool: Option<slot::SlotPool>,
    pub screencopy_manager: Option<(zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1, u32)>,
    pub layer_shell: Option<(zwlr_layer_shell_v1::ZwlrLayerShellV1, u32)>,
    pub xdg_output_manager: Option<(zxdg_output_manager_v1::ZxdgOutputManagerV1, u32)>,
    pub xdgwmbase: Option<(xdg_wm_base::XdgWmBase, u32)>,
    pub viewporter: Option<(wp_viewporter::WpViewporter, u32)>,

    pub current_index: Option<usize>,

    /// 每个输出设备一个
    pub foam_outputs: Option<HashMap<usize, foam_outputs::FoamOutput>>,
    pub frames_ready: usize,
    pub layer_ready: usize,

    /// 光标管理器
    pub pointer_helper: PointerHelper,
}

impl WaylandCtx {
    pub fn new(shm: shm::Shm, pool: slot::SlotPool, qh: QueueHandle<FoamShot>) -> Self {
        Self {
            qh: Some(qh),
            shm: Some(shm),
            foam_outputs: Some(HashMap::new()),
            pool: Some(pool),
            ..Default::default()
        }
    }

    pub fn init_base_layers(&mut self) {
        for (_, v) in self.foam_outputs.as_mut().unwrap().iter_mut() {
            v.init_layer(
                &self.layer_shell.as_ref().unwrap().0,
                self.qh.as_ref().unwrap(),
            );
        }
    }

    pub fn set_freeze_with_udata(&mut self, udata: usize) {
        let mut foam_output = self.foam_outputs.as_mut().unwrap().get_mut(&udata);
        foam_output
            .as_mut()
            .unwrap()
            .set_freeze(self.pool.as_mut().unwrap());
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
        let foam_outputs = self.foam_outputs.as_mut().unwrap();
        for (index, foam_output) in foam_outputs.iter_mut() {
            let frame = screencopy_manager.capture_output(
                true as i32,
                foam_output.output.as_ref().unwrap(),
                qh,
                *index,
            );
            foam_output.screencopy_frame = Some(frame);
        }
    }

    pub fn generate_sub_rects(&mut self) {
        let foam_outputs = self.foam_outputs.as_mut().unwrap();
        let start_index = self.pointer_helper.start_index.unwrap();
        let (start_x, start_y) = self.pointer_helper.start_pos.unwrap();
        let (end_x, end_y) = self.pointer_helper.current_pos.unwrap();
        let start_output = foam_outputs.get_mut(&start_index).unwrap();
        let (start_gx, start_gy, end_gx, end_gy) = (
            start_output.global_x + start_x as i32,
            start_output.global_y + start_y as i32,
            start_output.global_x + end_x as i32,
            start_output.global_y + end_y as i32,
        );
        // 计算矩形的全局边界
        let rect_min_x = min(start_gx, end_gx);
        let rect_min_y = min(start_gy, end_gy);
        let rect_max_x = max(start_gx, end_gx);
        let rect_max_y = max(start_gy, end_gy);

        for (_id, m) in foam_outputs {
            let intersection_min_x = max(m.global_x, rect_min_x);
            let intersection_min_y = max(m.global_y, rect_min_y);
            let intersection_max_x = min(m.global_x + m.width, rect_max_x);
            let intersection_max_y = min(m.global_y + m.height, rect_max_y);

            if intersection_min_x >= intersection_max_x || intersection_min_y >= intersection_max_y
            {
                continue;
            }

            let relative_min_x = intersection_min_x - m.global_x;
            let relative_min_y = intersection_min_y - m.global_y;
            let width = intersection_max_x - intersection_min_x;
            let height = intersection_max_y - intersection_min_y;
            m.new_subrect(relative_min_x, relative_min_y, width, height);
        }
    }

    pub fn update_select_region(&mut self) {
        for (_, v) in self.foam_outputs.as_mut().unwrap().iter_mut() {
            v.update_select_subrect(self.pool.as_mut().unwrap());
        }
    }

    pub fn store_copy_canvas(&mut self) {
        for (_, v) in self.foam_outputs.as_mut().unwrap().iter_mut() {
            v.store_canvas(self.pool.as_mut().unwrap());
        }
    }
}
