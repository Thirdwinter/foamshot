use std::collections::HashMap;

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
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1;

use crate::{
    config, foam_outputs, foamshot::FoamShot, pointer_helper::PointerHelper,
    select_rect::SelectRect, zwlr_screencopy_mode,
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
    // pub screencopy_manager: Option<(zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1, u32)>,
    pub layer_shell: Option<(zwlr_layer_shell_v1::ZwlrLayerShellV1, u32)>,
    pub xdg_output_manager: Option<(zxdg_output_manager_v1::ZxdgOutputManagerV1, u32)>,
    pub xdgwmbase: Option<(xdg_wm_base::XdgWmBase, u32)>,
    pub viewporter: Option<(wp_viewporter::WpViewporter, u32)>,

    pub current_index: Option<usize>,
    /// FIX: 不符合预期的pointer事件，用于记录其中的 surface 索引
    pub unknown_index: Option<usize>,

    pub current_freeze: bool,

    /// 每个输出设备一个
    pub foam_outputs: Option<HashMap<usize, foam_outputs::FoamOutput>>,
    pub layer_ready: usize,

    /// 光标管理器
    pub pointer_helper: PointerHelper,

    pub config: config::FoamConfig,
    pub scm: zwlr_screencopy_mode::ZwlrScreencopyMode,
    pub global_rect: Option<SelectRect>,
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
    pub fn set_one_max(&mut self, traget: usize) {
        for (i, v) in self.foam_outputs.as_mut().unwrap() {
            if *i == traget {
                v.max_rect();
            } else {
                v.clean_rect();
            }
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
                self.viewporter.clone().unwrap().0,
            );
        }
    }

    /// 重新将缓冲区附加到surface，生成新的一帧，此处仅可附加 `freeze`/`no_freeze` 两种的内容
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

    /// 用一个空的buffer附加到surface，使屏幕恢复正常状态，用来 toggle freeze 前清空屏幕以便进行copy
    pub fn unset_freeze(&mut self) {
        for (_i, v) in self.foam_outputs.as_mut().unwrap().iter_mut() {
            v.clean_attach();
        }
    }

    /// 所有输出设备发起全屏捕获请求
    pub fn request_screencopy(&mut self) {
        debug!("发起屏幕copy请求");
        let _screencopy_manager = if let Some((ref manager, _)) = self.scm.manager {
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
            self.scm.request_copy_one(
                self.config.cursor,
                foam_output.output.as_ref().unwrap(),
                qh,
                *index,
            );
        }
    }
    pub fn compute_global_rect(&mut self) {
        // 解包起始位置和当前位置
        let (start_x, start_y) = self.pointer_helper.g_start_pos.unwrap();
        let (end_x, end_y) = self.pointer_helper.g_current_pos.unwrap();

        // 转换到全局坐标系
        let (start_gx, start_gy) = (start_x as i32, start_y as i32);
        let (end_gx, end_gy) = (end_x as i32, end_y as i32);

        // 计算父矩形边界
        let rect = SelectRect::new(
            start_gx.min(end_gx),
            start_gy.min(end_gy),
            start_gx.max(end_gx),
            start_gy.max(end_gy),
        );

        self.global_rect = Some(rect);
    }
    pub fn process_subrects_and_send(&mut self) {
        let foam_outputs = self.foam_outputs.as_mut().unwrap();
        let rect = self.global_rect.as_ref().unwrap();

        let SelectRect {
            sx: min_x,
            sy: min_y,
            ex: max_x,
            ey: max_y,
            ..
        } = *rect;

        for output in foam_outputs.values_mut() {
            // 计算与当前输出的交集区域
            let intersect_left = output.global_x.max(min_x);
            let intersect_top = output.global_y.max(min_y);
            let intersect_right = (output.global_x + output.width).min(max_x);
            let intersect_bottom = (output.global_y + output.height).min(max_y);

            // 判断有效交集区域
            if intersect_left < intersect_right && intersect_top < intersect_bottom {
                // 计算相对于输出的局部坐标
                let local_x = intersect_left - output.global_x;
                let local_y = intersect_top - output.global_y;
                let width = intersect_right - intersect_left;
                let height = intersect_bottom - intersect_top;

                // 更新输出状态
                output.new_subrect(local_x, local_y, width, height);
                output.need_redraw = true;

                // 提交surface更新
                if let Some(surface) = &mut output.surface {
                    let qh = self.qh.as_ref().unwrap();
                    surface.frame(qh, output.id);
                    surface.set_buffer_scale(output.scale.round() as i32);
                    surface.commit();
                }
            } else {
                // 清理无效区域
                output.subrect = None;
                // TODO: 这里应该不用显示设置为false
                output.need_redraw = false;
            }
        }
    }

    /// 在鼠标按下和拖动时候被调用，为每个output生成子矩形，如果成功生成，对应output标记为需要重绘, 且surface将发送帧回调
    pub fn generate_rects_and_send_frame(&mut self) {
        self.compute_global_rect();
        self.process_subrects_and_send();
    }

    /// 在wl_callback中被调用，为需要重绘的输出更新下一帧
    pub fn update_select_region(&mut self) {
        for (i, v) in self.foam_outputs.as_mut().unwrap().iter_mut() {
            if !v.need_redraw {
                continue;
            }
            let base_canvas = self.scm.base_canvas.as_mut().unwrap().get_mut(i).unwrap();

            v.update_select_subrect(base_canvas, self.current_freeze);
        }
    }

    pub fn store_copy_canvas(&mut self) {
        for (i, v) in self.foam_outputs.as_mut().unwrap().iter_mut() {
            let pool = v.pool.as_mut().unwrap();
            self.scm.insert_canvas(*i, pool);
            // v.store_canvas();
        }
    }
}
