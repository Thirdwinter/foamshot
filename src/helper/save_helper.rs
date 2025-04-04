use std::io::Write;

use log::warn;

use crate::wayland_ctx::WaylandCtx;

pub fn save_to_png(wl_ctx: &mut WaylandCtx) -> Result<(), Box<dyn std::error::Error>> {
    let subrects = wl_ctx.subrects.as_ref().unwrap();
    let monitors = wl_ctx.monitors.as_ref().unwrap();

    if subrects.is_empty() {
        warn!("No sub regions found");
        return Ok(());
    }

    // 计算所有子区域的边界坐标
    let (min_x, min_y, total_width, total_height) = {
        let mut first = true;
        let mut min_x = 0;
        let mut min_y = 0;
        let mut max_right = 0;
        let mut max_bottom = 0;

        for subrect in subrects {
            let monitor = monitors.get(&subrect.monitor_id).unwrap();

            // 计算子区域在全局坐标系中的位置
            let global_x = monitor.x + subrect.relative_min_x;
            let global_y = monitor.y + subrect.relative_min_y;
            let global_right = global_x + subrect.width;
            let global_bottom = global_y + subrect.height;

            if first {
                min_x = global_x;
                min_y = global_y;
                max_right = global_right;
                max_bottom = global_bottom;
                first = false;
            } else {
                min_x = min_x.min(global_x);
                min_y = min_y.min(global_y);
                max_right = max_right.max(global_right);
                max_bottom = max_bottom.max(global_bottom);
            }
        }

        // 计算最终尺寸
        let width = (max_right - min_x) as u32;
        let height = (max_bottom - min_y) as u32;
        (min_x, min_y, width, height)
    };

    // 创建精确尺寸的最终图像
    let mut final_surface = cairo::ImageSurface::create(
        cairo::Format::ARgb32,
        total_width as i32,
        total_height as i32,
    )?;

    let ctx = cairo::Context::new(&final_surface)?;

    // 清空画布为透明背景
    ctx.set_source_rgba(0.0, 0.0, 0.0, 0.0);
    ctx.paint()?;

    // 遍历子区域进行绘制
    for subrect in subrects {
        let monitor = monitors.get(&subrect.monitor_id).unwrap();

        // 计算子区域在最终图像中的位置
        let dest_x = (monitor.x + subrect.relative_min_x) - min_x;
        let dest_y = (monitor.y + subrect.relative_min_y) - min_y;

        // 获取显示器原始数据
        let canvas = wl_ctx
            .base_canvas
            .as_mut()
            .unwrap()
            .get_mut(&subrect.monitor_id)
            .unwrap();

        // 创建子区域Surface（相对显示器坐标系）
        let (x, y, w, h) = (
            subrect.relative_min_x as usize,
            subrect.relative_min_y as usize,
            subrect.width as usize,
            subrect.height as usize,
        );

        let stride = monitor.width as usize * 4;
        let start_offset = y * stride + x * 4;

        let sub_surface = unsafe {
            cairo::ImageSurface::create_for_data_unsafe(
                canvas.as_mut_ptr().add(start_offset),
                cairo::Format::ARgb32,
                subrect.width as i32,
                subrect.height as i32,
                stride as i32,
            )?
        };

        // 绘制到最终图像
        ctx.save()?;
        ctx.translate(dest_x as f64, dest_y as f64);
        ctx.set_source_surface(&sub_surface, 0.0, 0.0)?;
        ctx.paint()?;
        ctx.restore()?;
    }

    // 保存文件（增强错误处理）
    let output_path = format!(
        "screenshot_{}.png",
        chrono::Local::now().format("%Y%m%d_%H%M%S")
    );
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&output_path)
        .map_err(|e| format!("创建文件失败: {}", e))?;

    final_surface
        .write_to_png(&mut file)
        .map_err(|e| format!("写入PNG失败: {}", e))?;

    file.flush().map_err(|e| format!("刷新文件失败: {}", e))?;

    Ok(())
}
