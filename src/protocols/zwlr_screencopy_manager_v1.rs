use log::*;
use wayland_client::{Dispatch, Proxy};
use wayland_protocols_wlr::screencopy::v1::client::{
    zwlr_screencopy_frame_v1, zwlr_screencopy_manager_v1,
};

use crate::action::Action;
use crate::foamshot::FoamShot;

impl Dispatch<zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1, usize> for FoamShot {
    fn event(
        app: &mut Self,
        proxy: &zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1,
        event: <zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1 as Proxy>::Event,
        data: &usize,
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            zwlr_screencopy_frame_v1::Event::Buffer {
                format,
                width,
                height,
                stride,
            } => {
                trace!(
                    "creating buffer: data is {}, width: {}, height: {}, stride: {}, format: {:?}",
                    data, width, height, stride, format
                );
                let current = app
                    .wayland_ctx
                    .foam_outputs
                    .as_mut()
                    .unwrap()
                    .get_mut(data)
                    .unwrap();

                let (buffer, _canvas) = current
                    .pool
                    .as_mut()
                    .unwrap()
                    .create_buffer(
                        width as i32,
                        height as i32,
                        stride as i32,
                        format.into_result().unwrap(),
                    )
                    .unwrap();
                // current.base_buffer = Some(buffer);
                app.wayland_ctx.scm.insert_buffer(*data, buffer).ok();
            }
            zwlr_screencopy_frame_v1::Event::BufferDone => {
                trace!("bufferdone => data:{}, copy frame to buffer", data);
                let buffer = app
                    .wayland_ctx
                    .scm
                    .base_buffers
                    .as_mut()
                    .unwrap()
                    .get_mut(data)
                    .unwrap()
                    .wl_buffer();
                proxy.copy(buffer);
            }
            zwlr_screencopy_frame_v1::Event::Ready { .. } => {
                trace!("data:{}, frame ready", data);
                app.wayland_ctx.scm.copy_ready += 1;
                // app.wayland_ctx.frames_ready += 1;
            }
            zwlr_screencopy_frame_v1::Event::Failed => {
                warn!("buffer copy error");
                app.mode = Action::Exit;
            }
            _ => (),
        }
    }
}

// NOTE: unused
#[allow(unused_variables)]
impl Dispatch<zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1, ()> for FoamShot {
    fn event(
        app: &mut Self,
        proxy: &zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1,
        event: <zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1 as Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
    }
}
