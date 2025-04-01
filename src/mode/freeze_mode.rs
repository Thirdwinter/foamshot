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
    pub fn set_freeze(&mut self, wl_ctx: &mut WaylandCtx) {
        // 提前解包必要的 Option 和引用，减少重复操作
        let buffers = wl_ctx.base_buffers.as_mut().expect("Missing base buffers");
        let Some(outputs) = wl_ctx.outputs.as_ref() else {
            error!("无可用 outputs");
            return;
        };
        let surfaces = self.surface.as_ref().expect("Missing surfaces");
        let widths = wl_ctx.widths.as_ref().expect("Missing widths");
        let heights = wl_ctx.heights.as_ref().expect("Missing heights");

        for (index, _) in outputs.iter().enumerate() {
            let Some(buffer) = buffers.get(&index) else {
                error!("Missing buffer for output {}", index);
                continue;
            };
            let Some(surface) = surfaces.get(&index) else {
                error!("Missing surface for output {}", index);
                continue;
            };

            buffer.attach_to(surface).unwrap();
            let width = widths.get(&index).copied().unwrap_or_default();
            let height = heights.get(&index).copied().unwrap_or_default();
            surface.damage(0, 0, width, height);
            surface.commit();
        }
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
}
