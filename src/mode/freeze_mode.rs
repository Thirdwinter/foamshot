use std::collections::{HashMap, HashSet};

use cairo::{Context, ImageSurface};
use log::*;
use smithay_client_toolkit::shm::slot::Buffer;
use wayland_client::protocol::{wl_shm::Format, wl_surface};
use wayland_protocols_wlr::{
    layer_shell::v1::client::{
        zwlr_layer_shell_v1::{self, Layer},
        zwlr_layer_surface_v1::{self, Anchor, KeyboardInteractivity},
    },
    screencopy::v1::client::zwlr_screencopy_frame_v1,
};

use crate::wayland_ctx::WaylandCtx;

#[derive(Default)]
pub struct FreezeMode {
    pub surface: Option<HashMap<usize, wl_surface::WlSurface>>,
    pub screencopy_frame: Option<HashMap<usize, zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1>>,
    pub layer_surface: Option<HashMap<usize, zwlr_layer_surface_v1::ZwlrLayerSurfaceV1>>,
    pub buffer: Option<HashMap<usize, Buffer>>,

    previous_output_record: Option<HashSet<usize>>,
}

impl FreezeMode {
    pub fn new() -> Self {
        Self {
            ..FreezeMode::default()
        }
    }
    pub fn before(&mut self, wl_ctx: &mut WaylandCtx) {
        self.screencopy_frame = wl_ctx.screencopy_frame.clone();

        // 提前处理 Option 和错误
        let Some(outputs) = &wl_ctx.outputs.as_ref() else {
            error!("无可用 outputs");
            return;
        };

        let layer_shell = wl_ctx.layer_shell.as_ref().expect("Missing layer shell");
        let qh = wl_ctx.qh.as_ref().expect("Missing qh");
        let surfaces = self.surface.as_mut().expect("Missing surfaces");
        let widths = wl_ctx.widths.as_ref().expect("Missing widths");
        let heights = wl_ctx.heights.as_ref().expect("Missing heights");

        let mut layers = HashMap::new();

        for (index, output) in outputs.iter().enumerate() {
            let surface = match surfaces.get_mut(&index) {
                Some(s) => s,
                None => {
                    error!("Missing surface for output {}", index);
                    continue;
                }
            };

            // 创建 layer
            let layer = zwlr_layer_shell_v1::ZwlrLayerShellV1::get_layer_surface(
                &layer_shell.0,
                surface,
                Some(output),
                Layer::Overlay,
                "foam_freeze".to_string(),
                qh,
                index,
            );

            // 配置 layer
            layer.set_anchor(Anchor::all());
            layer.set_exclusive_zone(-1);
            layer.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);
            layers.insert(index, layer);

            // 设置 damage 并提交
            let width = widths.get(&index).copied().unwrap_or_default();
            let height = heights.get(&index).copied().unwrap_or_default();
            surface.damage(0, 0, width, height);
            surface.commit();
        }

        self.layer_surface = Some(layers);
    }

    pub fn set_freeze_with_udata(&mut self, wl_ctx: &mut WaylandCtx, udata: usize) {
        // 提前解包必要的 Option 和引用，减少重复操作
        // let buffers = wl_ctx.base_buffers.as_mut().expect("Missing base buffers");
        let surfaces = self.surface.as_ref().expect("Missing surfaces");
        let widths = wl_ctx.widths.as_ref().expect("Missing widths");
        let heights = wl_ctx.heights.as_ref().expect("Missing heights");

        // for (index, _) in outputs.iter().enumerate() {
        let (buffer, canvas) = wl_ctx
            .pool
            .as_mut()
            .unwrap()
            .create_buffer(
                *widths.get(&udata).unwrap_or(&0),
                *heights.get(&udata).unwrap_or(&0),
                widths.get(&udata).unwrap_or(&0) * 4,
                Format::Argb8888,
            )
            .unwrap();
        if let Some(surface) = surfaces.get(&udata) {
            canvas.copy_from_slice(wl_ctx.base_canvas.as_ref().unwrap().get(&udata).unwrap());
            let cairo_surface = unsafe {
                ImageSurface::create_for_data_unsafe(
                    canvas.as_mut_ptr(),
                    cairo::Format::ARgb32,
                    *widths.get(&udata).unwrap_or(&0),
                    *heights.get(&udata).unwrap_or(&0),
                    widths.get(&udata).unwrap_or(&0) * 4,
                )
                .expect("创建 Cairo ImageSurface 失败")
            };
            let cr = Context::new(&cairo_surface).expect("创建 Cairo 画布失败");
            cr.set_source_rgba(1.0, 1.0, 1.0, 0.5); // 半透明（50%透明度）的白色
            cr.paint().unwrap();

            buffer.attach_to(surface).unwrap();
            match self.buffer {
                Some(_) => {
                    self.buffer.as_mut().unwrap().insert(udata, buffer);
                }
                None => {
                    self.buffer = Some(HashMap::new());
                    self.buffer.as_mut().unwrap().insert(udata, buffer);
                }
            }

            let width = widths.get(&udata).copied().unwrap_or_default();
            let height = heights.get(&udata).copied().unwrap_or_default();
            surface.damage(0, 0, width, height);
            surface.commit();
        };
    }

    pub fn update_select_region(&mut self, wl_ctx: &mut WaylandCtx) -> Option<()> {
        // 获取 widths 和 heights，如果为 None 则返回
        let widths = wl_ctx.widths.as_ref()?;
        let heights = wl_ctx.heights.as_ref()?;
        let surfaces = self.surface.as_ref()?;

        // 获取 subrects，如果为 None 则返回
        let subrects = wl_ctx.subrects.as_ref()?;

        let mut previous_outputs = self.previous_output_record.take().unwrap_or_default();
        self.previous_output_record = Some(subrects.iter().map(|v| v.monitor_id).collect());
        subrects.iter().for_each(|v| {
            // filter outputs from the previous update and got removed in this update
            previous_outputs.remove(&v.monitor_id);
        });

        for index in previous_outputs.into_iter() {
            // 获取 buffer，如果为 None 则跳过当前迭代
            let buffer = self.buffer.as_ref().unwrap().get(&index).unwrap();

            // 获取 canvas，如果 pool 为 None 则返回
            let pool = wl_ctx.pool.as_mut().unwrap();
            let canvas = buffer.canvas(pool).unwrap();
            canvas.copy_from_slice(wl_ctx.base_canvas.as_ref().unwrap().get(&index).unwrap());

            // 创建 Cairo Surface
            let cairo_surface = unsafe {
                ImageSurface::create_for_data_unsafe(
                    canvas.as_mut_ptr(),
                    cairo::Format::ARgb32,
                    *widths.get(&index).unwrap_or(&0),
                    *heights.get(&index).unwrap_or(&0),
                    widths.get(&index).unwrap_or(&0) * 4,
                )
                .unwrap()
            };

            let cr = Context::new(&cairo_surface).unwrap();

            // 获取Cairo表面尺寸
            let surface_width = cairo_surface.width() as f64;
            let surface_height = cairo_surface.height() as f64;

            // 设置半透明白色
            cr.set_source_rgba(1.0, 1.0, 1.0, 0.5);

            // 绘制覆盖整个表面的矩形
            cr.rectangle(0.0, 0.0, surface_width, surface_height);

            // 填充路径区域
            cr.fill().unwrap();

            let surface = surfaces.get(&index).unwrap();

            buffer.attach_to(surface).unwrap(); // 如果 attach_to 失败则返回

            surface.damage_buffer(
                0,
                0,
                *widths.get(&index).unwrap_or(&0),
                *heights.get(&index).unwrap_or(&0),
            );

            // 提交 surface
            surface.commit();
        }

        for v in subrects.iter() {
            let index = v.monitor_id;

            // 获取 buffer，如果为 None 则跳过当前迭代
            let buffer = self.buffer.as_ref().unwrap().get(&index).unwrap();

            // 获取 canvas，如果 pool 为 None 则返回
            let pool = wl_ctx.pool.as_mut().unwrap();
            let canvas = buffer.canvas(pool)?;
            canvas.copy_from_slice(wl_ctx.base_canvas.as_ref().unwrap().get(&index).unwrap());

            // 创建 Cairo Surface
            let cairo_surface = unsafe {
                ImageSurface::create_for_data_unsafe(
                    canvas.as_mut_ptr(),
                    cairo::Format::ARgb32,
                    *widths.get(&index).unwrap_or(&0),
                    *heights.get(&index).unwrap_or(&0),
                    widths.get(&index).unwrap_or(&0) * 4,
                )
                .unwrap()
            };

            let (x, y, w, h) = (v.relative_min_x, v.relative_min_y, v.width, v.height);
            let cr = Context::new(&cairo_surface).unwrap();

            // 获取Cairo表面尺寸
            let surface_width = cairo_surface.width() as f64;
            let surface_height = cairo_surface.height() as f64;

            // 设置半透明白色
            cr.set_source_rgba(1.0, 1.0, 1.0, 0.5);

            // 绘制覆盖整个表面的矩形
            cr.rectangle(0.0, 0.0, surface_width, surface_height);

            // 添加内部矩形路径（作为裁剪区域）
            cr.rectangle(x.into(), y.into(), w.into(), h.into());

            // 使用奇偶填充规则，形成环形区域
            cr.set_fill_rule(cairo::FillRule::EvenOdd);

            // 填充路径区域
            cr.fill().unwrap();

            let surface = surfaces.get(&index).unwrap();

            buffer.attach_to(surface).unwrap(); // 如果 attach_to 失败则返回

            surface.damage_buffer(
                0,
                0,
                *widths.get(&index).unwrap_or(&0),
                *heights.get(&index).unwrap_or(&0),
            );

            // 提交 surface
            surface.commit();
        }

        Some(())
    }

    #[allow(unused)]
    pub fn unset_freeze(&mut self, wl_ctx: &mut WaylandCtx) {
        let Some(outputs) = wl_ctx.outputs.as_ref() else {
            error!("无可用 outputs");
            return;
        };

        let surfaces = self.surface.as_ref().expect("Missing surfaces");
        let widths = wl_ctx.widths.as_ref().expect("Missing widths");
        let heights = wl_ctx.heights.as_ref().expect("Missing heights");

        for (index, _) in outputs.iter().enumerate() {
            let Some(surface) = surfaces.get(&index) else {
                error!("Missing surface for output {}", index);
                continue;
            };

            surface.attach(None, 0, 0);

            let width = widths.get(&index).copied().unwrap_or_default();
            let height = heights.get(&index).copied().unwrap_or_default();
            surface.damage(0, 0, width, height);
            surface.commit();
        }
    }

    #[allow(unused)]
    pub fn set_freeze(&mut self, wl_ctx: &mut WaylandCtx) {
        // 提前解包必要的 Option 和引用，减少重复操作
        // let buffers = wl_ctx.base_buffers.as_mut().expect("Missing base buffers");
        let Some(outputs) = wl_ctx.outputs.as_ref() else {
            error!("无可用 outputs");
            return;
        };
        let surfaces = self.surface.as_ref().expect("Missing surfaces");
        let widths = wl_ctx.widths.as_ref().expect("Missing widths");
        let heights = wl_ctx.heights.as_ref().expect("Missing heights");

        for (index, _) in outputs.iter().enumerate() {
            let (buffer, canvas) = wl_ctx
                .pool
                .as_mut()
                .unwrap()
                .create_buffer(
                    *widths.get(&index).unwrap_or(&0),
                    *heights.get(&index).unwrap_or(&0),
                    widths.get(&index).unwrap_or(&0) * 4,
                    Format::Argb8888,
                )
                .unwrap(); // TODO: buffer.attach_to(surface, height, stride, format)
            // let Some(buffer) = buffers.get(&index) else {
            //     error!("Missing buffer for output {}", index);
            //     continue;
            // };
            let Some(surface) = surfaces.get(&index) else {
                error!("Missing surface for output {}", index);
                continue;
            };
            canvas.copy_from_slice(wl_ctx.base_canvas.as_ref().unwrap().get(&index).unwrap());

            buffer.attach_to(surface).unwrap();
            match self.buffer {
                Some(_) => {
                    self.buffer.as_mut().unwrap().insert(index, buffer);
                }
                None => {
                    self.buffer = Some(HashMap::new());
                    self.buffer.as_mut().unwrap().insert(index, buffer);
                }
            }

            let width = widths.get(&index).copied().unwrap_or_default();
            let height = heights.get(&index).copied().unwrap_or_default();
            surface.damage(0, 0, width, height);
            surface.commit();
        }
    }
}
