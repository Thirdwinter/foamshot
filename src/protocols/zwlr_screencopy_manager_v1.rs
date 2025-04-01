use std::collections::HashMap;

use log::{error, trace};
use wayland_client::{Dispatch, Proxy};
use wayland_protocols_wlr::screencopy::v1::client::{
    zwlr_screencopy_frame_v1, zwlr_screencopy_manager_v1,
};

use crate::foamshot::FoamShot;
use crate::mode::Mode;

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
                let (buffer, _canvas) = app
                    .wayland_ctx
                    .pool
                    .as_mut()
                    .unwrap()
                    .create_buffer(
                        width as i32,
                        height as i32,
                        stride as i32,
                        format.into_result().expect("Unsupported format"),
                    )
                    .unwrap();

                match &app.wayland_ctx.base_buffers {
                    Some(_) => {
                        app.wayland_ctx
                            .base_buffers
                            .as_mut()
                            .unwrap()
                            .insert(*data, buffer);
                    }
                    None => {
                        app.wayland_ctx.base_buffers = Some(HashMap::new());
                        app.wayland_ctx
                            .base_buffers
                            .as_mut()
                            .unwrap()
                            .insert(*data, buffer);
                    }
                }
            }
            zwlr_screencopy_frame_v1::Event::BufferDone { .. } => {
                let Some(buffer) = &app.wayland_ctx.base_buffers else {
                    error!("Could not load WlBuffers");
                    return;
                };
                trace!("data:{}, copy frame to buffer", data);
                // copy frame to buffer, sends Ready when successful
                proxy.copy(buffer.get(data).unwrap().wl_buffer());
            }
            zwlr_screencopy_frame_v1::Event::Ready { .. } => {
                trace!("data:{}, frame ready", data);
                app.wayland_ctx.frames_ready += 1;
                // let Some(outputs) = &app.wayland_ctx.outputs.as_ref() else {
                //     error!("无可用 outputs");
                //     return;
                // };
                //
                // // if let Some(outputs) = app.wayland_ctx.outputs.as_ref().clone() {
                // for (i, _) in outputs.iter().enumerate() {
                //     log::debug!("output {}", i);
                //     let buffer = app
                //         .wayland_ctx
                //         .base_buffers
                //         .as_ref()
                //         .unwrap()
                //         .get(&i)
                //         .unwrap();
                //     let canvas = buffer
                //         .canvas(app.wayland_ctx.pool.as_mut().unwrap())
                //         .unwrap();
                //
                //     match &app.wayland_ctx.base_canvas {
                //         Some(_) => {
                //             app.wayland_ctx
                //                 .base_canvas
                //                 .as_mut()
                //                 .unwrap()
                //                 .insert(*data, canvas.to_vec());
                //         }
                //         None => {
                //             app.wayland_ctx.base_canvas = Some(HashMap::new());
                //             app.wayland_ctx
                //                 .base_canvas
                //                 .as_mut()
                //                 .unwrap()
                //                 .insert(*data, canvas.to_vec());
                //         }
                //     }
                // }
                // }
            }
            zwlr_screencopy_frame_v1::Event::Failed => {
                app.mode = Mode::Exit;
            }
            _ => (),
        }
    }
}
