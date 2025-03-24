use std::io::Write;

use cairo::ImageSurface;
use log::debug;
use smithay_client_toolkit::shm;
use smithay_client_toolkit::shm::slot::Buffer;

use crate::wayland_ctx::WaylandCtx;

#[derive(Default)]
pub struct ResultMode {
    pub quickshot: bool,
    pub buffer: Option<Buffer>,
    pub start: Option<(i32, i32)>,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

impl ResultMode {
    pub fn new(quickshot: bool) -> ResultMode {
        ResultMode {
            quickshot,
            buffer: None,
            ..Default::default()
        }
    }
    pub fn before(&mut self, wl_ctx: &mut WaylandCtx) {
        if let (
            Some(screencopy_manager),
            Some(output),
            Some(qh),
            Some((start_x, start_y)),
            Some((end_x, end_y)),
        ) = (
            wl_ctx.screencopy_manager.as_ref(),
            wl_ctx.output.as_ref(),
            wl_ctx.qh.as_ref(),
            wl_ctx.start_pos,
            wl_ctx.end_pos,
        ) {
            // 计算左上角坐标
            let x = start_x.min(end_x);
            let y = start_y.min(end_y);

            // 计算宽高并确保至少为1
            let mut width = (end_x - start_x).abs();
            let mut height = (end_y - start_y).abs();
            if width <= 1.0 {
                width = 1.0;
            }
            if height <= 1.0 {
                height = 1.0;
            }
            debug!("start_x: {}, start_y: {}", start_x, start_y);
            debug!("end_x: {}, end_y: {}", end_x, end_y);
            debug!("x: {}, y: {}", x, y);
            debug!("width: {}, height: {}", width, height);
            self.start = Some((x as i32, y as i32));
            self.width = Some(width as i32);
            self.height = Some(height as i32);

            let _screencopy_frame = screencopy_manager.capture_output_region(
                false as i32,
                &output,
                x as i32,      // 修正后的起始x坐标
                y as i32,      // 修正后的起始y坐标
                width as i32,  // 保证至少为1的宽度
                height as i32, // 保证至少为1的高度
                &qh,
                (),
            );
        }
    }

    pub fn to_png(&mut self, path: std::path::PathBuf, wl_ctx: &mut WaylandCtx) {
        match &self.buffer {
            Some(buffer) => {
                let canvas = buffer
                    .canvas(wl_ctx.pool.as_mut().unwrap())
                    .expect("get canvas");
                let cairo_surface = unsafe {
                    ImageSurface::create_for_data(
                        std::slice::from_raw_parts_mut(canvas.as_mut_ptr(), canvas.len()),
                        cairo::Format::Rgb24,
                        self.width.unwrap(),
                        self.height.unwrap(),
                        self.width.unwrap() * 4,
                    )
                    .map_err(|e| format!("Failed to create Cairo surface: {}", e))
                    .unwrap()
                };
                // let output_path = &self.cli.output_path;
                let file = std::fs::File::create(&path).unwrap();
                let mut buffer_writer = std::io::BufWriter::new(file);
                cairo_surface
                    .write_to_png(&mut buffer_writer)
                    .expect("write png");
                buffer_writer.flush().unwrap();
                std::process::exit(0);
            }
            None => {}
        }
    }
}
