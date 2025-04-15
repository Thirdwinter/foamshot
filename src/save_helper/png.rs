use crate::wayland_ctx::WaylandCtx;
use cairo::ImageSurface;
use log::warn;
use std::io::Write;

use super::common::{calculate_capture_info, create_final_surface, process_all_outputs};

pub fn save_to_png(wl_ctx: &mut WaylandCtx) -> Result<ImageSurface, Box<dyn std::error::Error>> {
    let capture_info = match calculate_capture_info(wl_ctx)? {
        Some(info) => info,
        None => {
            warn!("未找到有效截图区域");
            return Err("未找到有效截图区域".into());
        }
    };

    let final_surface = create_final_surface(capture_info.total_width, capture_info.total_height)?;

    process_all_outputs(wl_ctx, &capture_info, &final_surface)?;

    let output_path = wl_ctx.config.output_path.clone();

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output_path.clone())
        .map_err(|e| format!("创建文件失败: {}", e))?;

    final_surface
        .write_to_png(&mut file)
        .map_err(|e| format!("写入PNG失败: {}", e))?;

    file.flush().map_err(|e| format!("刷新文件失败: {}", e))?;

    Ok(final_surface)
}
