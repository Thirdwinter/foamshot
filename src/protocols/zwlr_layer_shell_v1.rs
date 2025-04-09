use log::debug;
use wayland_client::{Dispatch, Proxy};
use wayland_protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1};

use crate::action::Action;
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
                if width == 0 || height == 0 {
                    return;
                }
                debug!("Configure {}: {}x{}", data, width, height);
                proxy.ack_configure(serial);
                // proxy.set_size(width, height);
                if app.mode == Action::Init {
                    debug!("layer show");
                    app.wayland_ctx.attach_with_udata(*data);
                    app.wayland_ctx.layer_ready += 1;
                    if app.wayland_ctx.layer_ready
                        == app.wayland_ctx.foam_outputs.as_ref().unwrap().len()
                    {
                        app.wayland_ctx.current_freeze = app.wayland_ctx.config.freeze;
                        app.mode = Action::WaitPointerPress;

                        app.wayland_ctx.layer_ready = 0;
                    }
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
