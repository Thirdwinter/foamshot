use log::*;
use smithay_client_toolkit::shm::slot::SlotPool;
use wayland_client::{Dispatch, Proxy};
use wayland_protocols_wlr::screencopy::v1::client::{
    zwlr_screencopy_frame_v1, zwlr_screencopy_manager_v1,
};

use crate::{action::Action, foamcore::FoamShot};

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
                let current = app
                    .wlctx
                    .foam_outputs
                    .as_mut()
                    .unwrap()
                    .get_mut(*data)
                    .unwrap();
                let shm = app.wlctx.shm.as_mut().unwrap();
                let pool = SlotPool::new(stride as usize * height as usize, shm)
                    .expect("Failed to create pool");

                current.pool = Some(pool);

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
                app.wlctx.scm.insert_buffer(*data, buffer).ok();
            }
            zwlr_screencopy_frame_v1::Event::BufferDone => {
                trace!("bufferdone => data:{}, copy frame to buffer", data);
                let buffer = app
                    .wlctx
                    .scm
                    .base_buffers
                    .as_mut()
                    .unwrap()
                    .get_mut(data)
                    .unwrap()
                    .wl_buffer();
                proxy.copy(buffer);
            }
            zwlr_screencopy_frame_v1::Event::Ready {
                tv_sec_hi, // 时间戳的秒数（高32位）
                tv_sec_lo, // 时间戳的秒数（低32位）
                tv_nsec,   // 时间戳中的纳秒部分
            } => {
                let seconds: u64 = ((tv_sec_hi as u64) << 32) | (tv_sec_lo as u64);
                // 转换为精确时间戳
                let timestamp = (seconds * 1_000_000_000) + tv_nsec as u64;
                trace!("data:{}, timestamp:{} frame ready", timestamp, data);
                proxy.destroy();
                app.wlctx.scm.copy_ready += 1;
            }
            zwlr_screencopy_frame_v1::Event::Failed => {
                warn!("buffer copy error");
                app.action = Action::Exit;
            }
            _ => (),
        }
    }
}

// NOTE: ne event
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
