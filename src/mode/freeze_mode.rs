use std::collections::HashMap;

use log::*;
use wayland_client::protocol::wl_surface;
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
    // pub buffer: Option<HashMap<usize, Buffer>>,
}

impl FreezeMode {
    pub fn new() -> Self {
        Self { ..FreezeMode::default() }
    }
    pub fn before(&mut self, wl_ctx: &mut WaylandCtx) {
        self.screencopy_frame = wl_ctx.screencopy_frame.clone();

        // 遍历所有 outputs
        if let Some(ref outputs) = wl_ctx.outputs {
            let mut layers = HashMap::new();
            for (index, output) in outputs.iter().enumerate() {
                let layer = zwlr_layer_shell_v1::ZwlrLayerShellV1::get_layer_surface(
                    &wl_ctx.layer_shell.as_ref().unwrap().0,
                    self.surface.as_mut().unwrap().get(&index).unwrap(),
                    Some(output),
                    Layer::Overlay,
                    "foam_freeze".to_string(),
                    &wl_ctx.qh.clone().unwrap(),
                    1,
                );
                layer.set_anchor(Anchor::all());
                layer.set_exclusive_zone(-1);
                layer.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);

                layers.insert(index, layer);
            }
            self.layer_surface = Some(layers);
        } else {
            error!("无可用 outputs");
            return;
        }
        if let Some(ref surfaces) = self.surface {
            for (index, surface) in surfaces.iter().enumerate() {
                surface.1.damage(
                    0,
                    0,
                    *wl_ctx.widths.clone().unwrap().get(&index).unwrap(),
                    *wl_ctx.heights.clone().unwrap().get(&index).unwrap(),
                );
                surface.1.commit();
            }
        }
    }
    pub fn set_freeze(&mut self, wl_ctx: &mut WaylandCtx) {
        if let Some(ref buffers) = wl_ctx.base_buffers {
            for (index, buffer) in buffers.iter().enumerate() {
                buffer.1.attach_to(self.surface.as_ref().unwrap().get(&index).unwrap()).unwrap();
                self.surface.as_mut().unwrap().get(&index).unwrap().damage(
                    0,
                    0,
                    *wl_ctx.widths.clone().unwrap().get(&index).unwrap(),
                    *wl_ctx.heights.clone().unwrap().get(&index).unwrap(),
                );
                self.surface.as_mut().unwrap().get(&index).unwrap().commit();
            }
        } else {
            println!("no buffer")
        }
    }

    #[allow(unused)]
    pub fn unset_freeze(&mut self, wl_ctx: &mut WaylandCtx) {
        if let Some(ref surfaces) = self.surface {
            for (index, surface) in surfaces.iter().enumerate() {
                {
                    surface.1.attach(None, 0, 0);
                    surface.1.damage(
                        0,
                        0,
                        *wl_ctx.widths.clone().unwrap().get(&index).unwrap(),
                        *wl_ctx.heights.clone().unwrap().get(&index).unwrap(),
                    );
                    surface.1.commit();
                }
            }
        }
    }
}
