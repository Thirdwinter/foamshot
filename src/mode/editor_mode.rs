use smithay_client_toolkit::shm::slot::Buffer;
use wayland_client::protocol::wl_shm::Format;
use wayland_client::protocol::wl_surface;
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};

use crate::wayland_ctx;

#[derive(Default)]
pub struct EditorMode {
    pub surface: Option<wl_surface::WlSurface>,
    pub buffer: Option<Buffer>,
    pub xdg_surface: Option<xdg_surface::XdgSurface>,
    pub toplevel: Option<xdg_toplevel::XdgToplevel>,
}

impl EditorMode {
    pub fn resize(&mut self, wl_ctx: &mut wayland_ctx::WaylandCtx, width: i32, height: i32) {
        if let Some(toplevel) = &self.toplevel {
            toplevel.set_max_size(width, height);
            toplevel.set_min_size(width, height);
            self.surface.as_ref().unwrap().commit();
        }
    }
    pub fn before(&mut self, wl_ctx: &wayland_ctx::WaylandCtx) {
        let xdg_surface = wl_ctx.xdgwmbase.as_ref().unwrap().0.get_xdg_surface(
            self.surface.as_ref().unwrap(),
            wl_ctx.qh.as_ref().unwrap(),
            (),
        );
        let toplevel = xdg_surface.get_toplevel(wl_ctx.qh.as_ref().unwrap(), ());
        toplevel.set_title("foamshot".to_string());
        toplevel.set_app_id("foamshot".to_string());
        toplevel.set_max_size(1366, 768);
        toplevel.set_min_size(200, 200);
        self.toplevel = Some(toplevel);
        self.surface.as_ref().unwrap().damage(0, 0, 1366, 768);

        self.surface.as_ref().unwrap().commit();
    }
    pub fn on(&mut self, wl_ctx: &mut wayland_ctx::WaylandCtx) {
        let wl_buffer = wl_ctx
            .base_buffers
            .as_ref()
            .unwrap()
            .get(&0)
            .unwrap()
            .wl_buffer()
            .clone();
        // let (buffer, canvas) = wl_ctx
        //     .pool
        //     .as_mut()
        //     .unwrap()
        //     .create_buffer(200, 200, 200 * 4, Format::Argb8888)
        //     .unwrap();
        // self.buffer = (wl_ctx.base_buffers.unwrap().get(1).unwrap());
        // wl_ctx
        //     .base_buffers
        //     .as_ref()
        //     .unwrap()
        //     .get(&0)
        //     .expect("index error")
        //     // canvas.fill(180);
        //     .attach_to(self.surface.as_ref().unwrap())
        //     .unwrap();
        self.surface
            .as_ref()
            .unwrap()
            .attach(Some(&wl_buffer), 1, 1);
        self.surface.as_ref().unwrap().damage(0, 0, 1366, 768);
        self.surface.as_ref().unwrap().commit();
    }
}
