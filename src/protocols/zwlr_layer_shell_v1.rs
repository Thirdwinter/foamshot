use log::debug;
use wayland_client::{Dispatch, Proxy};
use wayland_protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1};

use crate::foamshot::FoamShot;
use crate::mode::Mode;

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
                match app.mode {
                    Mode::Init => {
                        app.freeze_mode
                            .set_freeze_with_udata(&mut app.wayland_ctx, data.clone());
                        app.wayland_ctx.freeze_ready += 1;
                        if app.wayland_ctx.freeze_ready
                            == app.wayland_ctx.outputs.as_ref().unwrap().len()
                        {
                            app.mode = Mode::OnFreeze;
                        }
                    }
                    _ => {}
                }
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
