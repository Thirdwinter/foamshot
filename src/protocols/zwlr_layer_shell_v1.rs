use log::debug;
use wayland_client::protocol::wl_shm::Format;
use wayland_client::{Dispatch, Proxy};
use wayland_protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1};

use crate::foamshot::FoamShot;

impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, usize> for FoamShot {
    fn event(
        app: &mut Self,
        proxy: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        event: <zwlr_layer_surface_v1::ZwlrLayerSurfaceV1 as Proxy>::Event,
        data: &usize,
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            zwlr_layer_surface_v1::Event::Configure {
                serial,
                width,
                height,
            } => {
                debug!("Configure {}: {}x{}", data, width, height);
                proxy.ack_configure(serial);
                proxy.set_size(width, height);
                let (buffer, canvas) = app
                    .wayland_ctx
                    .pool
                    .as_mut()
                    .unwrap()
                    .create_buffer(
                        width as i32,
                        height as i32,
                        width as i32 * 4,
                        Format::Argb8888,
                    )
                    .unwrap();
                canvas.fill(100);
                // let buffers = app
                //     .wayland_ctx
                //     .base_buffers
                //     .as_mut()
                //     .expect("Missing base buffers");
                let surfaces = app.freeze_mode.surface.as_mut().expect("Missing surfaces");
                // let buffer = buffers.get(data).expect("Missing buffer");
                let surface = surfaces.get_mut(data).expect("Missing surface");
                buffer.attach_to(surface).unwrap();
                surface.damage(0, 0, width as i32, height as i32);
                surface.commit();
            }
            zwlr_layer_surface_v1::Event::Closed => {
                proxy.destroy();
            }
            _ => (),
        }
    }
}

// NOTE: unused
#[allow(unused_variables)]
impl Dispatch<zwlr_layer_shell_v1::ZwlrLayerShellV1, ()> for FoamShot {
    fn event(
        app: &mut Self,
        proxy: &zwlr_layer_shell_v1::ZwlrLayerShellV1,
        event: <zwlr_layer_shell_v1::ZwlrLayerShellV1 as Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
    }
}
