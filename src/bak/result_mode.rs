use std::io::Write;
use std::process::{Command, Stdio};

use cairo::{Context, ImageSurface};
use log::{debug, error};
use smithay_client_toolkit::shm::slot::Buffer;
use wayland_protocols_wlr::screencopy::v1::client::zwlr_screencopy_frame_v1;

use super::freeze_mode::FreezeMode;
use crate::config::Cli;
use crate::wayland_ctx::WaylandCtx;

#[derive(Default)]
pub struct ResultMode {
    pub quickshot: bool,
    pub full_screen: bool,
    pub buffer: Option<Buffer>,
    pub start: Option<(i32, i32)>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub screencopy_frame: Option<zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1>,
}

impl ResultMode {
    pub fn new(quickshot: bool) -> Self {
        Self {
            quickshot,
            buffer: None,
            ..Default::default()
        }
    }

    /// 计算截图区域。
    /// 如果 full_screen 为 true 则返回全屏区域，否则根据 WaylandCtx 中的 start_pos 和 end_pos 计算区域。
    fn calculate_region(&self, wl_ctx: &WaylandCtx) -> Option<(f64, f64, f64, f64)> {
        if self.full_screen {
            // 全屏模式下，直接使用屏幕的宽高
            let (w, h) = (wl_ctx.width?, wl_ctx.height?);
            Some((0.0, 0.0, w as f64, h as f64))
        } else {
            // 非全屏模式，需要通过起始和结束坐标计算区域
            let (start_x, start_y) = wl_ctx.start_pos?;
            let (end_x, end_y) = wl_ctx.end_pos?;
            let x = start_x.min(end_x);
            let y = start_y.min(end_y);
            let mut width = (end_x - start_x).abs();
            let mut height = (end_y - start_y).abs();
            // 保证最小尺寸为 1
            if width < 1.0 {
                width = 1.0;
            }
            if height < 1.0 {
                height = 1.0;
            }
            Some((x, y, width, height))
        }
    }

    pub fn to_png_2(&mut self, cli: &Cli, wl_ctx: &mut WaylandCtx, freeze_frame: &mut FreezeMode) {
        // 根据配置计算截图区域
        let (x, y, width, height) = match self.calculate_region(wl_ctx) {
            Some(region) => region,
            None => {
                debug!("无法确定截图区域：缺少必需的屏幕尺寸或区域坐标");
                return;
            }
        };

        if let Some(buffer) = freeze_frame.buffer.as_mut() {
            if let Err(e) = buffer.deactivate() {
                debug!("关闭 buffer 出错：{}", e);
            }
        } else {
            error!("freeze_frame 中未找到 buffer");
            return;
        }

        debug!(
            "截图区域 - x: {}, y: {}, width: {}, height: {}",
            x as i32, y as i32, width as i32, height as i32
        );
        self.start = Some((x as i32, y as i32));
        self.width = Some(width as i32);
        self.height = Some(height as i32);

        // 从 WaylandCtx 的共享内存中获取 canvas
        let pool = wl_ctx.pool.as_mut().expect("WaylandCtx 中缺少 pool");
        let canvas = freeze_frame
            .buffer
            .as_ref()
            .unwrap()
            .canvas(pool)
            .expect("获取 canvas 失败");

        let full_width = wl_ctx.width.expect("WaylandCtx 缺少屏幕宽度");
        let full_height = wl_ctx.height.expect("WaylandCtx 缺少屏幕高度");

        // 为整个画面创建 Cairo ImageSurface
        let cairo_surface = unsafe {
            ImageSurface::create_for_data_unsafe(
                canvas.as_mut_ptr(),
                cairo::Format::Rgb24,
                full_width,
                full_height,
                full_width * 4,
            )
            .expect("创建 Cairo ImageSurface 失败")
        };

        // 为截取区域创建新的 Cairo ImageSurface
        let cropped_surface =
            ImageSurface::create(cairo::Format::Rgb24, width as i32, height as i32)
                .expect("无法创建截取区域的 surface");

        // 使用新的 Context 将指定区域绘制到 cropped_surface 上
        let cr = Context::new(&cropped_surface).expect("创建 Cairo 画布失败");
        cr.set_source_surface(&cairo_surface, -x, -y)
            .expect("设置绘制区域失败");
        cr.paint().expect("绘制截取区域失败");

        // 将裁剪后的图像写入 PNG 文件
        let file = std::fs::File::create(&cli.output_path).expect("无法创建输出文件");
        let mut buffer_writer = std::io::BufWriter::new(file);
        cropped_surface
            .write_to_png(&mut buffer_writer)
            .expect("写入 PNG 失败");
        buffer_writer.flush().expect("刷新文件失败");

        // 如果 auto_copy 选项开启，则复制图片到剪贴板
        if cli.auto_copy {
            let mut png_data = Vec::new();
            cropped_surface
                .write_to_png(&mut png_data)
                .expect("无法写入 PNG 数据到内存");

            let mut process = Command::new("wl-copy")
                .arg("--type")
                .arg("image/png")
                .stdin(Stdio::piped())
                .spawn()
                .expect("启动 wl-copy 进程失败");

            if let Some(stdin) = process.stdin.as_mut() {
                stdin.write_all(&png_data).expect("写入剪贴板数据失败");
            } else {
                error!("无法获取 wl-copy 的标准输入");
            }

            process.wait().expect("等待 wl-copy 进程结束失败");
        }

        // std::process::exit(0);
    }
}
