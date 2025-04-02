use std::{
    cmp::{max, min},
    collections::HashMap,
};

use log::{debug, error};
use smithay_client_toolkit::shm::{
    self,
    slot::{self, Buffer},
};
use wayland_client::{
    QueueHandle,
    protocol::{wl_compositor, wl_keyboard, wl_output, wl_seat},
};
use wayland_protocols::{
    wp::viewporter::client::wp_viewporter,
    xdg::{shell::client::xdg_wm_base, xdg_output::zv1::client::zxdg_output_manager_v1},
};
use wayland_protocols_wlr::{
    layer_shell::v1::client::zwlr_layer_shell_v1,
    screencopy::v1::client::{zwlr_screencopy_frame_v1, zwlr_screencopy_manager_v1},
};

use crate::{
    foamshot::FoamShot,
    helper::{
        monitor_helper::{Monitor, SubRect},
        pointer_helper::PointerHelper,
    },
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
    pub pool: Option<slot::SlotPool>,
    pub screencopy_manager: Option<(zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1, u32)>,
    pub layer_shell: Option<(zwlr_layer_shell_v1::ZwlrLayerShellV1, u32)>,
    pub xdg_output_manager: Option<(zxdg_output_manager_v1::ZxdgOutputManagerV1, u32)>,
    pub xdgwmbase: Option<(xdg_wm_base::XdgWmBase, u32)>,
    pub viewporter: Option<(wp_viewporter::WpViewporter, u32)>,

    pub current_index: Option<usize>,

    /// 每个输出设备一个
    pub outputs: Option<Vec<wl_output::WlOutput>>,
    pub widths: Option<HashMap<usize, i32>>,
    pub heights: Option<HashMap<usize, i32>>,
    pub scales: Option<HashMap<usize, i32>>,
    /// 初始copy的屏幕
    pub base_buffers: Option<HashMap<usize, Buffer>>,
    pub base_canvas: Option<HashMap<usize, Vec<u8>>>,
    pub screencopy_frame: Option<HashMap<usize, zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1>>,

    pub frames_ready: usize,
    pub freeze_ready: usize,

    /// 光标管理器
    pub pointer_helper: PointerHelper,
    pub monitors: Option<HashMap<usize, Monitor>>,
    pub subrects: Option<Vec<SubRect>>,
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
    pub fn request_screencopy_region(&mut self) {
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
        let farme = screencopy_manager.capture_output_region(
            false as i32,
            self.outputs.as_ref().unwrap().get(0).unwrap(),
            0,
            0,
            1800,
            600,
            qh,
            20,
        );
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

    pub fn generate_sub_rects(&mut self) {
        if let Some(monitors) = &self.monitors {
            if let Some(start_index) = self.pointer_helper.start_index {
                if let Some((start_x, start_y)) = self.pointer_helper.start_pos {
                    if let Some((end_x, end_y)) = self.pointer_helper.current_pos {
                        if let Some(monitor) = monitors.get(&start_index) {
                            // 将相对坐标转换为全局坐标
                            let start_global_x = monitor.x + start_x as i32;
                            let start_global_y = monitor.y + start_y as i32;
                            let end_global_x = monitor.x + end_x as i32;
                            let end_global_y = monitor.y + end_y as i32;

                            // 计算矩形的全局边界
                            let rect_min_x = min(start_global_x, end_global_x);
                            let rect_min_y = min(start_global_y, end_global_y);
                            let rect_max_x = max(start_global_x, end_global_x);
                            let rect_max_y = max(start_global_y, end_global_y);

                            let mut sub_rects = Vec::new();

                            for (id, m) in monitors {
                                let intersection_min_x = max(m.x, rect_min_x);
                                let intersection_min_y = max(m.y, rect_min_y);
                                let intersection_max_x = min(m.x + m.width, rect_max_x);
                                let intersection_max_y = min(m.y + m.height, rect_max_y);

                                if intersection_min_x >= intersection_max_x
                                    || intersection_min_y >= intersection_max_y
                                {
                                    continue;
                                }

                                let relative_min_x = intersection_min_x - m.x;
                                let relative_min_y = intersection_min_y - m.y;
                                let width = intersection_max_x - intersection_min_x;
                                let height = intersection_max_y - intersection_min_y;

                                sub_rects.push(SubRect {
                                    monitor_id: *id,
                                    relative_min_x,
                                    relative_min_y,
                                    width,
                                    height,
                                });
                            }

                            self.subrects = Some(sub_rects);
                        }
                    }
                }
            }
        }
    }

    /// 将屏幕截图数据保存到 base_canvas
    pub fn store_copy_canvas(&mut self) {
        let Some(outputs) = self.outputs.as_ref() else {
            error!("无可用 outputs");
            return;
        };

        for (i, _) in outputs.iter().enumerate() {
            let buffer = self.base_buffers.as_ref().unwrap().get(&i).unwrap();
            let canvas = buffer.canvas(self.pool.as_mut().unwrap()).unwrap();

            match &self.base_canvas {
                Some(_) => {
                    self.base_canvas
                        .as_mut()
                        .unwrap()
                        .insert(i as usize, canvas.to_vec());
                }
                None => {
                    self.base_canvas = Some(HashMap::new());
                    self.base_canvas
                        .as_mut()
                        .unwrap()
                        .insert(i as usize, canvas.to_vec());
                }
            }
        }
    }
}
