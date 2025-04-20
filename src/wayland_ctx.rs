use cairo::{Context, ImageSurface};
use log::{debug, error};
use smithay_client_toolkit::shm::{self, slot::SlotPool};
use wayland_client::{
    QueueHandle,
    protocol::{wl_compositor, wl_keyboard, wl_pointer, wl_seat},
};
use wayland_protocols::{
    wp::{
        cursor_shape::v1::client::wp_cursor_shape_device_v1::Shape,
        fractional_scale::v1::client::wp_fractional_scale_manager_v1::WpFractionalScaleManagerV1,
        viewporter::client::wp_viewporter,
    },
    xdg::{shell::client::xdg_wm_base, xdg_output::zv1::client::zxdg_output_manager_v1},
};
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1;

use crate::{
    config::{self, FoamConfig},
    foamcore::FoamShot,
    frame_queue::FrameQueue,
    monitors,
    pointer_helper::PointerHelper,
    select_rect::SelectRect,
    zwlr_screencopy_mode,
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
    pub fractional_manager: Option<(WpFractionalScaleManagerV1, u32)>,

    pub current_index: Option<usize>,
    /// FIX: 不符合预期的pointer事件，用于记录其中的 surface 索引
    pub unknown_index: Option<usize>,

    pub current_freeze: bool,

    /// 每个输出设备一个
    pub foam_outputs: Option<Vec<monitors::FoamMonitors>>,
    pub layer_ready: usize,

    /// 光标管理器
    pub pointer_helper: PointerHelper,

    pub config: config::FoamConfig,
    pub scm: zwlr_screencopy_mode::ZwlrScreencopyMode,
    pub global_rect: Option<SelectRect>,

    pub fq: FrameQueue,
}

impl WaylandCtx {
    pub fn new(shm: shm::Shm, qh: QueueHandle<FoamShot>, config: FoamConfig) -> Self {
        Self {
            qh: Some(qh),
            fq: FrameQueue::new(SlotPool::new(256 * 256, &shm).ok()),
            shm: Some(shm),
            foam_outputs: Some(Vec::new()),
            config: config::FoamConfig::new(),
            current_freeze: config.freeze,
            ..Default::default()
        }
    }
    pub fn set_one_max(&mut self, target: usize) {
        // 遍历 Vec 的索引和元素
        for (index, foam_output) in self.foam_outputs.as_mut().unwrap().iter_mut().enumerate() {
            if index == target {
                foam_output.max_rect();
            } else {
                foam_output.clean_rect();
            }
        }
    }

    pub fn set_cursor_shape(
        &mut self,
        shape: Shape,
        pointer: &wl_pointer::WlPointer,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.pointer_helper
            .set_cursor_shape(self.qh.as_ref().unwrap(), shape, pointer)?;
        Ok(())
    }

    pub fn init_base_layers(&mut self) {
        for v in self.foam_outputs.as_mut().unwrap().iter_mut() {
            v.init_layer(
                &self.layer_shell.as_ref().unwrap().0,
                self.qh.as_ref().unwrap(),
                self.viewporter.clone().unwrap().0,
                // WARN: THIS WAY IT REQUIRES THE COMPOSITOR TO IMPLEMENT FRACTIONAL SCALE
                // OR ELSE THE UNWRAP IS GONNA CRASH THE PROGRAM
                // TODO: Add fallback
                self.fractional_manager.clone(),
            );
        }
    }

    /// 重新将缓冲区附加到surface，生成新的一帧，此处仅可附加 `freeze`/`no_freeze` 两种的内容
    pub fn attach_with_udata(&mut self, udata: usize) {
        let mut foam_output = self.foam_outputs.as_mut().unwrap().get_mut(udata);
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

    /// 判定全局矩形是否完全在一个显示器内
    #[allow(unused)]
    pub fn is_rectangle_in_monitor(&self) -> bool {
        false
    }
    #[allow(unused)]
    /// 检查矩形是否在某个显示器内或跨越多个显示器, 返回-1跨多个显示器，返回非负数则为显示器id，不为-1的其它值则不在显示器内
    pub fn find_monitor_for_rectangle(&self) -> i32 {
        -2
    }

    /// 用一个空的buffer附加到surface，使屏幕恢复正常状态，用来 toggle freeze 前清空屏幕以便进行copy
    pub fn unset_freeze(&mut self) {
        for v in self.foam_outputs.as_mut().unwrap().iter_mut() {
            v.clean_attach();
        }
    }

    pub fn no_freeze_attach_with_udata(&mut self, udata: usize) {
        self.foam_outputs
            .as_mut()
            .unwrap()
            .iter_mut()
            .find(|m| m.id == udata)
            .as_mut()
            .unwrap()
            .clean_attach();
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
        for (index, foam_output) in foam_outputs.iter_mut().enumerate() {
            self.scm.request_copy_one(
                self.config.cursor,
                foam_output.output.as_ref().unwrap(),
                qh,
                index,
            );
        }
    }
    /// 通过 pointer_helper 坐标计算全局父矩形
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
    /// 根据父矩形计算每个输出上的子矩形，如果存在，对应输出的surface请求下一帧
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

        for output in foam_outputs {
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

    /// 计算一个最小矩形可以覆盖显示器坐标系中所有输出
    pub fn calculate_bounding_rect(&self) -> (i32, i32, i32, i32) {
        // 获取显示器数组
        let outputs = self.foam_outputs.as_ref().unwrap();

        // 初始化边界
        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;

        // 遍历每个显示器，更新边界
        for output in outputs {
            min_x = min_x.min(output.global_x);
            min_y = min_y.min(output.global_y);
            max_x = max_x.max(output.global_x + output.width);
            max_y = max_y.max(output.global_y + output.height);
        }

        // 返回最小矩形：左上角坐标和宽高
        (min_x, min_y, max_x - min_x, max_y - min_y)
    }

    /// TODO: 一次绘制一个大小包含所有输出的表面，随后将其分配给所有输出
    #[allow(unused)]
    pub fn draw_all_outputs(&mut self) {
        let (x, y, w, h) = self.calculate_bounding_rect();
        let parent_surface = ImageSurface::create(cairo::Format::ARgb32, w, h)
            .expect("Couldn't create parent surface");
        let cr = Context::new(&parent_surface).ok().unwrap();
        cr.set_source_rgba(0.8, 0.8, 0.8, 0.3);
        cr.paint().unwrap();

        // 遍历每个显示器，生成对应的子 Surface
        if let Some(outputs) = &mut self.foam_outputs {
            for output in outputs.iter_mut() {
                // 计算子 Surface 的位置和大小
                let x = output.global_x - x;
                let y = output.global_y - y;
                let w = output.width;
                let h = output.height;

                // 创建子 Surface
                let mut sub_surface = ImageSurface::create(cairo::Format::ARgb32, w, h)
                    .expect("Couldn't create sub surface");

                // 将父 Surface 的对应区域绘制到子 Surface
                let sub_cr = Context::new(&sub_surface).expect("Couldn't create sub context");
                sub_cr
                    .set_source_surface(&parent_surface, -(x as f64), -(y as f64))
                    .ok();
                sub_cr.paint().expect("Couldn't paint to sub surface");
                sub_cr.target().flush(); // 确保绘制完成

                // sub_surface.write_to_png(stream)
                let d = sub_surface.data().ok().unwrap().to_vec();
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
        for (i, v) in self.foam_outputs.as_mut().unwrap().iter_mut().enumerate() {
            if !v.need_redraw {
                continue;
            }
            let base_canvas = self.scm.base_canvas.as_mut().unwrap().get_mut(&i).unwrap();

            v.update_select_subrect(base_canvas, self.current_freeze);
        }
    }

    pub fn storage_copy_canvas(&mut self) {
        for (i, v) in self.foam_outputs.as_mut().unwrap().iter_mut().enumerate() {
            let pool = v.pool.as_mut().unwrap();
            self.scm.insert_canvas(i, pool);
        }
    }
}
