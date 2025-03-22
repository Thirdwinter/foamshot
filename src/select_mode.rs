use cairo::{Context, FontSlant, FontWeight, Format, ImageSurface};
use smithay_client_toolkit::shm::slot::{self, Buffer};
use wayland_client::protocol::{wl_shm, wl_surface};
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::{self};

use crate::check_options;

#[derive(Default)]
pub struct SelectMode {
    pub surface: Option<wl_surface::WlSurface>,
    pub layer_surface: Option<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1>,
    pub buffer: Option<Buffer>,
    pub last_pos: (f64, f64),
}

impl SelectMode {
    /// NOTE: 更新选择区域
    pub fn update_select(
        &mut self,
        phys_width: Option<i32>,
        phys_height: Option<i32>,
        pool: &mut slot::SlotPool,
        pointer_start: Option<(f64, f64)>,
        current_pos: Option<(f64, f64)>,
    ) {
        let (phys_width, phys_height, (start_x, start_y), (end_x, end_y)) =
            check_options!(phys_width, phys_height, pointer_start, current_pos);
        self.last_pos = (end_x, end_y);

        let (buffer, canvas) = pool
            .create_buffer(
                phys_width,
                phys_height,
                phys_width * 4,
                wl_shm::Format::Argb8888,
            )
            .unwrap();
        canvas.fill(0);
        let cairo_surface = unsafe {
            ImageSurface::create_for_data(
                std::slice::from_raw_parts_mut(canvas.as_mut_ptr(), canvas.len()),
                Format::ARgb32,
                phys_width,
                phys_height,
                phys_width * 4,
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
            .damage_buffer(0, 0, phys_width, phys_height);
        self.surface.as_ref().unwrap().commit();
    }
}
