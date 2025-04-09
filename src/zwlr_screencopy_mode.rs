use std::collections::HashMap;

use smithay_client_toolkit::shm::slot::{Buffer, SlotPool};
use wayland_client::QueueHandle;
use wayland_client::protocol::wl_output;
use wayland_protocols_wlr::screencopy::v1::client::zwlr_screencopy_manager_v1;

use crate::foamshot::FoamShot;

#[derive(Default)]
/// NOTE: 统一管理所有输出的screen copy
pub struct ZwlrScreencopyMode {
    pub manager: Option<(zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1, u32)>,
    pub copy_ready: usize,
    pub base_buffers: Option<HashMap<usize, Buffer>>,
    pub base_canvas: Option<HashMap<usize, Vec<u8>>>,
}

impl ZwlrScreencopyMode {
    pub fn new(manager: (zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1, u32)) -> Self {
        Self {
            manager: Some(manager),
            copy_ready: 0,
            base_buffers: Some(HashMap::new()),
            base_canvas: Some(HashMap::new()),
        }
    }
    pub fn request_copy_one(
        &mut self,
        cursor: bool,
        output: &wl_output::WlOutput,
        qh: &QueueHandle<FoamShot>,
        udata: usize,
    ) {
        let manager = if let Some((ref manager, _)) = self.manager {
            manager
        } else {
            return;
        };
        manager.capture_output(cursor as i32, output, qh, udata);
    }

    pub fn insert_buffer(&mut self, udata: usize, buffer: Buffer) -> Result<(), String> {
        match self.base_buffers.as_mut().unwrap().insert(udata, buffer) {
            Some(_) => Ok(()), // 键已存在，旧值被丢弃
            None => Ok(()),    // 键不存在，插入成功
        }
    }

    pub fn insert_canvas(&mut self, udata: usize, pool: &mut SlotPool) {
        let buffer = self.base_buffers.as_ref().unwrap().get(&udata).unwrap();
        let canvas = buffer.canvas(pool).unwrap().to_vec();
        self.base_canvas.as_mut().unwrap().insert(udata, canvas);
    }
}
