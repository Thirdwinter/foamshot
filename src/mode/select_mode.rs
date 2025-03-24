use cairo::{Context, FontSlant, FontWeight, ImageSurface};
use log::{debug, info};
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
pub struct SelectMode {
    pub surface: Option<wl_surface::WlSurface>,
    pub screencopy_frame: Option<zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1>,
    pub layer_surface: Option<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1>,
    pub buffer: Option<Buffer>,
    pub last_pos: (f64, f64),
}
impl SelectMode {
    #[inline]
    pub fn before(&mut self, wl_ctx: &mut WaylandCtx) {
        self.surface = Some(
            wl_ctx
                .compositor
                .as_ref()
                .unwrap()
                .create_surface(wl_ctx.qh.as_mut().unwrap(), 2),
        );
        self.surface.as_ref().unwrap().commit();

        let layer = zwlr_layer_shell_v1::ZwlrLayerShellV1::get_layer_surface(
            &wl_ctx.layer_shell.as_ref().unwrap(),
            &self.surface.as_ref().unwrap(),
            wl_ctx.output.as_ref(),
            Layer::Overlay,
            "foam_select".to_string(),
            &wl_ctx.qh.clone().unwrap(),
            2,
        );
        layer.set_anchor(Anchor::all());
        layer.set_exclusive_zone(-1);
        layer.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);
        self.layer_surface = Some(layer);

        info!("create select_layer");
        self.surface
            .as_ref()
            .unwrap()
            .damage(0, 0, wl_ctx.width.unwrap(), wl_ctx.height.unwrap());
        self.surface.as_ref().unwrap().commit();
    }

    pub fn on(&mut self, wl_ctx: &mut WaylandCtx) {
        match wl_ctx
            .create_buffer(
                wl_ctx.width.unwrap() as i32,
                wl_ctx.height.unwrap() as i32,
                wl_ctx.width.unwrap() as i32 * 4,
                Format::Argb8888,
            )
            .ok()
        {
            Some((buffer, canvas)) => {
                // self.buffer = Some(buffer);
                canvas.fill(100);
                self.buffer = Some(buffer);

                self.buffer
                    .as_mut()
                    .unwrap()
                    .attach_to(self.surface.as_mut().unwrap())
                    .unwrap();
                // self.buffer = Some(buffer);
                debug!("请求重绘");
                self.surface.as_ref().unwrap().damage_buffer(
                    0,
                    0,
                    wl_ctx.width.unwrap(),
                    wl_ctx.height.unwrap(),
                );
                self.surface.as_ref().unwrap().damage(
                    0,
                    0,
                    wl_ctx.width.unwrap() as i32,
                    wl_ctx.height.unwrap() as i32,
                );
                self.surface.as_ref().unwrap().commit();
                debug!("wait for select");
            }
            None => {
                std::process::exit(0);
            }
        }
    }
    pub fn after(&mut self, wl_ctx: &mut WaylandCtx) {
        if let (Some(width), Some(height), Some((start_x, start_y)), Some((end_x, end_y))) = (
            wl_ctx.width,
            wl_ctx.height,
            wl_ctx.start_pos,
            wl_ctx.current_pos,
        ) {
            self.last_pos = (end_x, end_y);

            debug!("update select");
            let (buffer, canvas) = wl_ctx
                .create_buffer(width, height, width * 4, Format::Argb8888)
                .unwrap();
            canvas.fill(0);

            let cairo_surface = unsafe {
                ImageSurface::create_for_data(
                    std::slice::from_raw_parts_mut(canvas.as_mut_ptr(), canvas.len()),
                    cairo::Format::ARgb32,
                    width,
                    height,
                    width * 4,
                )
                .map_err(|e| format!("Failed to create Cairo surface: {}", e))
                .unwrap()
            };

            // 创建 Cairo 上下文
            let ctx = Context::new(&cairo_surface)
                .map_err(|e| format!("Failed to create Cairo context: {}", e))
                .unwrap();

            // 填充整个表面为白色半透明
            ctx.set_source_rgba(1.0, 1.0, 1.0, 0.3);
            ctx.paint().unwrap();

            let width = end_x - start_x;
            let height = end_y - start_y;
            // 清除矩形区域以显示透明
            ctx.set_operator(cairo::Operator::Clear);
            ctx.rectangle(start_x, start_y, width, height);
            ctx.fill().unwrap();

            // 确保宽度和高度为非负整数
            let width = (end_x - start_x).abs() as i32;
            let height = (end_y - start_y).abs() as i32;

            // 创建文本内容
            let text = format!("{}x{}", width, height);

            // 计算文本位置，使其位于矩形的右下角外侧
            let text_extent = ctx.text_extents(&text).unwrap();
            let _text_width = text_extent.width();
            let text_height = text_extent.height();

            // 确定矩形的右下角坐标
            let rect_end_x = f64::max(start_x, end_x);
            let rect_end_y = f64::max(start_y, end_y);

            // 文本位置在矩形的右下角外侧，沿对角线延伸
            let text_x = rect_end_x + 10.0; // 向右偏移10像素
            let text_y = rect_end_y + text_height + 10.0; // 向下偏移10像素加上文本高度

            // 设置文本颜色为黑色
            ctx.set_source_rgb(0.0, 0.0, 0.0);
            ctx.select_font_face("Sans", FontSlant::Normal, FontWeight::Normal);
            ctx.set_font_size(16.0);

            // 绘制文本
            ctx.move_to(text_x, text_y);
            ctx.show_text(&text).unwrap();

            cairo_surface.flush();

            buffer.attach_to(self.surface.as_ref().unwrap()).unwrap();
            self.buffer = Some(buffer);
            // 请求重绘
            self.surface
                .as_ref()
                .unwrap()
                .damage_buffer(0, 0, width, height);
            self.surface.as_ref().unwrap().commit();
        }
    }
}
