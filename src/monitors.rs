use cairo::{Context, ImageSurface};
use log::debug;
use smithay_client_toolkit::shm::slot::{self, Buffer, SlotPool};
use wayland_client::{
    QueueHandle,
    protocol::{wl_output, wl_shm::Format, wl_surface},
};
use wayland_protocols::wp::{
    fractional_scale::v1::client::{
        wp_fractional_scale_manager_v1::WpFractionalScaleManagerV1,
        wp_fractional_scale_v1::WpFractionalScaleV1,
    },
    viewporter::client::{wp_viewport::WpViewport, wp_viewporter},
};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{self, Layer},
    zwlr_layer_surface_v1::{self, Anchor, KeyboardInteractivity},
};

use crate::{cairo_render::draw_base, foamcore::FoamShot, select_rect::SubRect};

/// NOTE: 为物理显示器做的抽象，包含其基础信息
#[derive(Default, Debug)]
pub struct FoamMonitors {
    /// 索引，由output event进行赋值
    pub id: usize,
    /// 显示器的命名，也许会有用
    pub name: String,

    pub output: Option<wl_output::WlOutput>,
    pub width: i32,
    pub height: i32,

    ///显示器 左上角 全局坐标 x
    pub global_x: i32,
    ///显示器 左上角 全局坐标 y
    pub global_y: i32,
    pub logical_width: i32,
    pub logical_height: i32,

    pub base_buffer: Option<Buffer>,
    // add freeze layer surfae to impl set_freeze
    pub surface: Option<wl_surface::WlSurface>,
    pub layer_surface: Option<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1>,
    // TODO: add sub rect with Option
    pub subrect: Option<SubRect>,

    pub need_redraw: bool,

    pub pool: Option<slot::SlotPool>,

    pub scale: Option<Scale>,
}

impl FoamMonitors {
    pub fn new(id: usize, output: wl_output::WlOutput, pool: SlotPool) -> Self {
        Self {
            id,
            output: Some(output),
            name: "unnamed".to_string(),
            pool: Some(pool),
            ..Default::default()
        }
    }

    pub fn convert_pos_to_surface(
        src_output: &FoamMonitors,
        target_output: &FoamMonitors,
        surface_x: f64,
        surface_y: f64,
    ) -> (f64, f64) {
        let global_x = src_output.global_x as f64 + surface_x;
        let global_y = src_output.global_y as f64 + surface_y;

        let dst_x = global_x - target_output.global_x as f64;
        let dst_y = global_y - target_output.global_y as f64;

        (dst_x, dst_y)
    }
    pub fn new_subrect(&mut self, x: i32, y: i32, w: i32, h: i32) {
        if w <= 0 || h <= 0 {
            self.subrect = None
        } else {
            self.subrect = Some(SubRect::new(self.id, x, y, w, h))
        }
    }
    pub fn max_rect(&mut self) {
        self.new_subrect(0, 0, self.width, self.height);
    }
    pub fn clean_rect(&mut self) {
        self.new_subrect(-1, -1, -1, -1);
    }

    pub fn init_layer(
        &mut self,
        layer_shell: &zwlr_layer_shell_v1::ZwlrLayerShellV1,
        qh: &QueueHandle<FoamShot>,
        viewporter: wp_viewporter::WpViewporter,
        fractional_manager: WpFractionalScaleManagerV1,
    ) {
        let id = self.id;
        let output = self.output.as_ref().unwrap();
        let (w, h) = (self.width, self.height);
        let surface = self.surface.as_mut().expect("Missing surfaces");

        // fractional scale
        let fractional = fractional_manager.get_fractional_scale(surface, qh, id);
        let viewport = viewporter.get_viewport(surface, qh, id);
        viewport.set_destination(self.logical_width, self.logical_height);
        self.scale = Some(Scale::new_fractional(fractional, viewport));

        let layer = zwlr_layer_shell_v1::ZwlrLayerShellV1::get_layer_surface(
            layer_shell,
            surface,
            Some(output),
            Layer::Top,
            "foamshot-selection".to_string(),
            qh,
            id,
        );

        // 配置 layer
        layer.set_anchor(Anchor::all());
        layer.set_exclusive_zone(-1);
        layer.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);

        self.layer_surface = Some(layer);
        surface.damage(0, 0, w, h);
        surface.commit();
    }

    pub fn freeze_attach(&mut self, base_canvas: &[u8]) {
        debug!("fn: freeze_attach");
        let (w, h) = (self.width, self.height);
        let surface = self.surface.as_ref().expect("Missing surfaces");
        let pool = self.pool.as_mut().unwrap();
        let (buffer, canvas) = pool.create_buffer(w, h, w * 4, Format::Argb8888).unwrap();
        canvas.copy_from_slice(base_canvas);

        draw_base(canvas, w, h);

        buffer.attach_to(surface).unwrap();
        surface.damage_buffer(0, 0, w, h);
        surface.commit();
        self.base_buffer = Some(buffer)
    }
    pub fn clean_attach(&mut self) {
        let (w, h) = (self.width, self.height);
        let surface = self.surface.as_ref().expect("Missing surfaces");
        let pool = self.pool.as_mut().unwrap();
        let (buffer, canvas) = pool.create_buffer(w, h, w * 4, Format::Argb8888).unwrap();
        canvas.fill(0);
        buffer.attach_to(surface).unwrap();
        surface.damage_buffer(0, 0, w, h);
        surface.commit();
        self.base_buffer = Some(buffer)
    }
    pub fn no_freeze_attach(&mut self) {
        let (w, h) = (self.width, self.height);
        let surface = self.surface.as_ref().expect("Missing surfaces");
        let pool = self.pool.as_mut().unwrap();
        let (buffer, canvas) = pool.create_buffer(w, h, w * 4, Format::Argb8888).unwrap();
        canvas.fill(0);
        draw_base(canvas, w, h);

        buffer.attach_to(surface).unwrap();
        surface.damage_buffer(0, 0, w, h);
        surface.commit();
        self.base_buffer = Some(buffer)
    }

    /// 该方法用于绘制所属输出上的子矩形
    pub fn update_select_subrect(&mut self, base_canvas: &[u8], freeze: bool) {
        if self.subrect.is_none() {
            return;
        }

        let (w, h) = (self.width, self.height);
        let surface = self.surface.as_ref().expect("Missing surfaces");
        let pool = self.pool.as_mut().unwrap();
        let (buffer, canvas) = pool.create_buffer(w, h, w * 4, Format::Argb8888).unwrap();

        if freeze {
            canvas.copy_from_slice(base_canvas);
        } else {
            canvas.fill(0);
        }

        let cairo_surface = unsafe {
            ImageSurface::create_for_data_unsafe(
                canvas.as_mut_ptr(),
                cairo::Format::ARgb32,
                w,
                h,
                w * 4,
            )
            .unwrap()
        };

        let cr = Context::new(&cairo_surface).unwrap();

        // 获取Cairo表面尺寸
        let surface_width = cairo_surface.width() as f64;
        let surface_height = cairo_surface.height() as f64;

        // 设置半透明白色
        cr.set_source_rgba(0.8, 0.8, 0.8, 0.3);
        cr.rectangle(0.0, 0.0, surface_width, surface_height);

        let subrect = self.subrect.as_ref().unwrap();

        let (x, y, rw, rh) = (
            subrect.relative_min_x,
            subrect.relative_min_y,
            subrect.width,
            subrect.height,
        );

        // 添加内部矩形路径（作为裁剪区域）
        cr.rectangle(x.into(), y.into(), rw.into(), rh.into());

        // 使用奇偶填充规则，形成环形区域
        cr.set_fill_rule(cairo::FillRule::EvenOdd);

        // 填充路径区域
        cr.fill().unwrap();

        // 添加边框（根据与显示器边缘的重合情况决定是否绘制）
        cr.save().unwrap(); // 保存当前状态
        cr.set_line_width(2.0); // 设置边框宽度
        cr.set_source_rgba(0.0, 0.0, 0.0, 1.0); // 设置边框颜色为黑色

        // 判断是否绘制左边
        if subrect.relative_min_x > 0 {
            cr.move_to(x.into(), y.into());
            cr.line_to(x.into(), (y + rh).into());
        }

        // 判断是否绘制上边
        if subrect.relative_min_y > 0 {
            cr.move_to(x.into(), y.into());
            cr.line_to((x + rw).into(), y.into());
        }

        // 判断是否绘制右边
        if (x + rw) < self.width {
            cr.move_to((x + rw).into(), y.into());
            cr.line_to((x + rw).into(), (y + rh).into());
        }

        // 判断是否绘制下边
        if (y + rh) < self.height {
            cr.move_to(x.into(), (y + rh).into());
            cr.line_to((x + rw).into(), (y + rh).into());
        }

        cr.stroke().unwrap(); // 绘制边框
        cr.restore().unwrap(); // 恢复状态

        buffer.attach_to(surface).unwrap(); // 如果 attach_to 失败则返回

        surface.damage_buffer(0, 0, w, h);

        // 提交 surface
        surface.commit();
        self.need_redraw = false;
        self.base_buffer = Some(buffer)
    }
}

#[derive(Debug)]
pub struct Scale {
    normal: u32,
    fractional: Option<(u32, WpFractionalScaleV1, WpViewport)>,
}
impl Scale {
    fn new_fractional(fractional_client: WpFractionalScaleV1, viewprot: WpViewport) -> Self {
        Self {
            normal: 1,
            fractional: Some((0, fractional_client, viewprot)),
        }
    }
    fn new_normal() -> Self {
        Self {
            normal: 1,
            fractional: None,
        }
    }
    fn is_fractional(&self) -> bool {
        self.fractional.is_some()
    }
    pub fn update_normal(&mut self, normal: u32) -> bool {
        let changed = self.normal != normal;
        self.normal = normal;
        changed
    }
    pub fn update_fraction(&mut self, fraction: u32) -> bool {
        if let Some(fractional) = self.fractional.as_mut() {
            let changed = fractional.0 != fraction;
            fractional.0 = fraction;
            changed
        } else {
            false
        }
    }
    pub fn calculate_pos(&self, pos: &mut (f64, f64)) {
        if let Some(fractional) = self.fractional.as_ref() {
            let mut scale = fractional.0;
            if scale == 0 {
                scale = 120
            }
            let scale_f64 = scale as f64 / 120.;
            pos.0 *= scale_f64;
            pos.1 *= scale_f64;
        } else {
            pos.0 *= self.normal as f64;
            pos.1 *= self.normal as f64;
        }
    }
}
impl Drop for Scale {
    fn drop(&mut self) {
        #[allow(clippy::option_map_unit_fn)]
        self.fractional.as_ref().map(|(_, f, v)| {
            f.destroy();
            v.destroy();
        });
    }
}
