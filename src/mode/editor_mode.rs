use cairo::{Context, ImageSurface};
use log::debug;
use smithay_client_toolkit::shm::slot::Buffer;
use wayland_client::protocol::wl_shm::Format;
use wayland_client::protocol::wl_surface;
use wayland_protocols::wp::viewporter::client::wp_viewport;
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};

use crate::wayland_ctx;

#[derive(Default)]
pub struct EditorMode {
    pub surface: Option<wl_surface::WlSurface>,
    pub xdg_surface: Option<xdg_surface::XdgSurface>,
    pub toplevel: Option<xdg_toplevel::XdgToplevel>,
    pub buffer: Option<Buffer>,
    pub viewport: Option<wp_viewport::WpViewport>,
    pub window_width: Option<i32>,
    pub window_height: Option<i32>,
}

impl EditorMode {
    pub fn before(&mut self, wl_ctx: &wayland_ctx::WaylandCtx) {
        self.viewport = Some(wl_ctx.viewporter.as_ref().unwrap().0.get_viewport(
            self.surface.as_ref().unwrap(),
            wl_ctx.qh.as_ref().unwrap(),
            (),
        ));
        let xdg_surface = wl_ctx.xdgwmbase.as_ref().unwrap().0.get_xdg_surface(
            self.surface.as_ref().unwrap(),
            wl_ctx.qh.as_ref().unwrap(),
            (),
        );
        let toplevel = xdg_surface.get_toplevel(wl_ctx.qh.as_ref().unwrap(), ());
        self.xdg_surface = Some(xdg_surface);
        toplevel.set_title("abc".to_string());
        toplevel.set_app_id("abc".to_string());
        toplevel.set_max_size(500, 281);
        toplevel.set_min_size(500, 281);
        self.toplevel = Some(toplevel);
        self.viewport
            .as_ref()
            .unwrap()
            .set_source(0.0, 0.0, 1366.0, 768.0);
        self.viewport.as_ref().unwrap().set_destination(500, 281);
        self.surface.as_ref().unwrap().damage(0, 0, 1366, 768);

        self.surface.as_ref().unwrap().commit();
    }

    pub fn on(&mut self, wl_ctx: &mut wayland_ctx::WaylandCtx) {
        let canvas1 = wl_ctx.base_canvas.as_ref().unwrap().get(&1).unwrap();
        let (buf, current_canvas) = wl_ctx
            .pool
            .as_mut()
            .unwrap()
            .create_buffer(1366, 768, 1366 * 4, Format::Argb8888)
            .unwrap();
        if current_canvas.len() == canvas1.len() {
            debug!(
                "can.len(){} == canvas1.len(){}",
                current_canvas.len(),
                canvas1.len()
            );
        }
        current_canvas.copy_from_slice(&canvas1);
        let cairo_surface = unsafe {
            ImageSurface::create_for_data(
                std::slice::from_raw_parts_mut(current_canvas.as_mut_ptr(), current_canvas.len()),
                cairo::Format::ARgb32,
                1366,
                768,
                1366 * 4,
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

        buf.attach_to(self.surface.as_ref().unwrap()).unwrap();
        self.surface.as_ref().unwrap().damage(0, 0, 1366, 768);
        self.surface.as_ref().unwrap().commit();
    }
}
