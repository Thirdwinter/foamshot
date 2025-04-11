use crate::wayland_ctx::WaylandCtx;
use image::{ImageBuffer, Rgb};
use log::warn;
use std::io::Write;

use super::common::{calculate_capture_info, create_final_surface, process_all_outputs};

pub fn save_to_jpg(wl_ctx: &mut WaylandCtx, quality: u8) -> Result<(), Box<dyn std::error::Error>> {
    let capture_info = match calculate_capture_info(wl_ctx)? {
        Some(info) => info,
        None => {
            warn!("未找到有效截图区域");
            return Ok(());
        }
    };

    let mut final_surface =
        create_final_surface(capture_info.total_width, capture_info.total_height)?;

    process_all_outputs(wl_ctx, &capture_info, &final_surface)?;

    // 将Cairo surface转换为RGB格式
    let width = final_surface.width() as u32;
    let height = final_surface.height() as u32;
    let stride = final_surface.stride() as usize;
    let data = final_surface.data().unwrap();

    // 直接创建RGB格式的ImageBuffer
    let mut rgb_buffer: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);

    // 复制像素数据（Cairo使用ARGB32格式，需要转换为RGB）
    for y in 0..height {
        for x in 0..width {
            let offset = (y as usize * stride) + (x as usize * 4);
            // Cairo ARGB32格式：[B, G, R, A]
            let b = data[offset];
            let g = data[offset + 1];
            let r = data[offset + 2];
            // 忽略alpha通道，直接使用RGB值
            rgb_buffer.put_pixel(x, y, Rgb([r, g, b]));
        }
    }

    // 保存为JPEG
    let output_path = wl_ctx.config.output_path.clone();
    let mut output_file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output_path.clone())
        .map_err(|e| format!("创建文件失败: {}", e))?;

    // 将ImageBuffer编码为JPEG并写入文件
    let mut jpeg_data = Vec::new();
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_data, quality);

    encoder
        .encode(
            rgb_buffer.as_raw(),
            width,
            height,
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|e| format!("JPEG编码失败: {}", e))?;

    output_file
        .write_all(&jpeg_data)
        .map_err(|e| format!("写入JPEG失败: {}", e))?;

    output_file
        .flush()
        .map_err(|e| format!("刷新文件失败: {}", e))?;

    Ok(())
}
