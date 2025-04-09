use std::{
    cmp::{max, min},
    collections::HashMap,
};

use log::{debug, error};
use smithay_client_toolkit::shm::{self};
use wayland_client::{
    QueueHandle,
    protocol::{wl_compositor, wl_keyboard, wl_pointer, wl_seat},
};
use wayland_protocols::{
    wp::{
        cursor_shape::v1::client::wp_cursor_shape_device_v1::Shape,
        viewporter::client::wp_viewporter,
    },
    xdg::{shell::client::xdg_wm_base, xdg_output::zv1::client::zxdg_output_manager_v1},
};
use wayland_protocols_wlr::{
    layer_shell::v1::client::zwlr_layer_shell_v1,
    screencopy::v1::client::zwlr_screencopy_manager_v1,
};

use crate::{
    config, foam_outputs, foamshot::FoamShot, pointer_helper::PointerHelper, zwlr_screencopy_mode,
};

#[derive(Default)]
pub struct WaylandCtx {
    /// 全局唯一
    /// u32 is wl_registry name
    pub compositor: Option<(wl_compositor::WlCompositor, u32)>,
    pub seat: Option<(wl_seat::WlSeat, u32)>,
    pub keyboard: Option<wl_keyboard::WlKeyboard>,
    pub qh: Option<QueueHandle<FoamShot>>,
    pub shm: Option<shm::Shm>,
    // pub pool: Option<slot::SlotPool>,
    pub screencopy_manager: Option<(zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1, u32)>,
    pub layer_shell: Option<(zwlr_layer_shell_v1::ZwlrLayerShellV1, u32)>,
    pub xdg_output_manager: Option<(zxdg_output_manager_v1::ZxdgOutputManagerV1, u32)>,
    pub xdgwmbase: Option<(xdg_wm_base::XdgWmBase, u32)>,
    pub viewporter: Option<(wp_viewporter::WpViewporter, u32)>,

    pub current_index: Option<usize>,
    /// FIX: 不符合预期的pointer事件，用于记录其中的 surface 索引
    pub unknow_index: Option<usize>,

    pub current_freeze: bool,

    /// 每个输出设备一个
    pub foam_outputs: Option<HashMap<usize, foam_outputs::FoamOutput>>,
    pub layer_ready: usize,

    /// 光标管理器
    pub pointer_helper: PointerHelper,

    pub config: config::FoamConfig,
    pub scm: zwlr_screencopy_mode::ZwlrScreencopyMode,
}

impl WaylandCtx {
    pub fn new(shm: shm::Shm, qh: QueueHandle<FoamShot>) -> Self {
        let config = config::FoamConfig::new();
        Self {
            qh: Some(qh),
            shm: Some(shm),
            foam_outputs: Some(HashMap::new()),
            config: config::FoamConfig::new(),
            current_freeze: config.freeze,
            ..Default::default()
        }
    }

    pub fn set_cursor_shape(
        &mut self,
        serial: u32,
        shape: Shape,
        pointer: &wl_pointer::WlPointer,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.pointer_helper
            .set_cursor_shape(self.qh.as_ref().unwrap(), serial, shape, pointer)?;
        Ok(())
    }

    pub fn init_base_layers(&mut self) {
        for (_, v) in self.foam_outputs.as_mut().unwrap().iter_mut() {
            v.init_layer(
                &self.layer_shell.as_ref().unwrap().0,
                self.qh.as_ref().unwrap(),
            );
        }
    }

    pub fn attach_with_udata(&mut self, udata: usize) {
        let mut foam_output = self.foam_outputs.as_mut().unwrap().get_mut(&udata);
        if self.current_freeze {
            let base_canvas = self
                .scm
                .base_canvas
                .as_mut()
                .unwrap()
                .get_mut(&udata)
                .unwrap();
            foam_output.as_mut().unwrap().freeze_attach(base_canvas);
        } else {
            foam_output.as_mut().unwrap().no_freeze_attach();
        }
    }

    pub fn unset_freeze(&mut self) {
        for (_i, v) in self.foam_outputs.as_mut().unwrap().iter_mut() {
            v.clean_attach();
        }
    }

    pub fn request_screencopy(&mut self) {
        debug!("发起屏幕copy请求");
        let _screencopy_manager = if let Some((ref manager, _)) = self.screencopy_manager {
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
            // let frame = screencopy_manager.capture_output(
            //     self.config.cursor as i32,
            //     foam_output.output.as_ref().unwrap(),
            //     qh,
            //     *index,
            // );
            // foam_output.screencopy_frame = Some(frame);
            self.scm.request_copy_one(
                self.config.cursor,
                foam_output.output.as_ref().unwrap(),
                qh,
                *index,
            );
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

        for m in foam_outputs.values_mut() {
            let intersection_min_x = max(m.global_x, rect_min_x);
            let intersection_min_y = max(m.global_y, rect_min_y);
            let intersection_max_x = min(m.global_x + m.width, rect_max_x);
            let intersection_max_y = min(m.global_y + m.height, rect_max_y);

            // Ensure that the intersection is valid
            if intersection_min_x < intersection_max_x && intersection_min_y < intersection_max_y {
                let relative_min_x = intersection_min_x - m.global_x;
                let relative_min_y = intersection_min_y - m.global_y;
                let width = intersection_max_x - intersection_min_x;
                let height = intersection_max_y - intersection_min_y;
                m.new_subrect(relative_min_x, relative_min_y, width, height);
            } else {
                m.subrect = None;
            }

            // debug!("id:{}, subrect:{:?}", _id, m.subrect);
        }
    }

    pub fn toggle_freeze_reattach(&mut self) {
        for (i, v) in self.foam_outputs.as_mut().unwrap().iter_mut() {
            v.send_next_frame(self.qh.as_ref().unwrap(), *i);
        }
    }

    pub fn update_select_region(&mut self) {
        for (i, v) in self.foam_outputs.as_mut().unwrap().iter_mut() {
            v.send_next_frame(self.qh.as_ref().unwrap(), *i);
            // v.update_select_subrect();
        }
    }

    pub fn store_copy_canvas(&mut self) {
        for (i, v) in self.foam_outputs.as_mut().unwrap().iter_mut() {
            let pool = v.pool.as_mut().unwrap();
            self.scm.insert_canvas(*i, pool);
            // v.store_canvas();
        }
    }

    pub fn before_output_collect_canvas(&mut self) {
        for (_i, v) in self.foam_outputs.as_mut().unwrap().iter_mut() {
            // let pool = v.pool.as_mut().unwrap();
            // self.scm.insert_canvas(*i, pool);
            v.store_canvas();
        }
    }
}
