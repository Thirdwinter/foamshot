use std::io::Write;

use log::{info, warn};

use crate::wayland_ctx::WaylandCtx;

pub fn save_to_png(wl_ctx: &mut WaylandCtx) -> Result<(), Box<dyn std::error::Error>> {
    // 第一阶段：只读操作收集必要信息
    let (min_x, min_y, total_width, total_height, monitor_ids) = {
        let outputs = wl_ctx.foam_outputs.as_ref().ok_or("未初始化输出设备")?;

        // 收集有效区域和对应的显示器ID
        let mut active_info = Vec::new();
        let mut bounds: Option<(i32, i32, i32, i32)> = None;

        for output in outputs.values() {
            if let Some(rect) = &output.subrect {
                // 计算全局坐标
                let global_x = output.global_x + rect.relative_min_x;
                let global_y = output.global_y + rect.relative_min_y;
                let right = global_x + rect.width;
                let bottom = global_y + rect.height;

                // 更新边界
                bounds = Some(match bounds {
                    Some((px, py, pr, pb)) => (
                        px.min(global_x),
                        py.min(global_y),
                        pr.max(right),
                        pb.max(bottom),
                    ),
                    None => (global_x, global_y, right, bottom),
                });

                // 记录需要处理的显示器ID
                active_info.push((rect.monitor_id, global_x, global_y));
            }
        }

        if active_info.is_empty() {
            warn!("未找到有效截图区域");
            return Ok(());
        }

        let (x, y, r, b) = bounds.unwrap();
        let total_width = (r - x) as u32;
        let total_height = (b - y) as u32;

        // 提取需要处理的ID列表
        let monitor_ids: Vec<_> = active_info.iter().map(|(id, _, _)| *id).collect();

        (x, y, total_width, total_height, monitor_ids)
    };

    // 第二阶段：可变操作处理每个显示器
    let outputs = wl_ctx.foam_outputs.as_mut().ok_or("输出设备未初始化")?;

    // 创建最终画布
    let final_surface = cairo::ImageSurface::create(
        cairo::Format::ARgb32,
        total_width as i32,
        total_height as i32,
    )?;

    // 清空画布
    {
        let ctx = cairo::Context::new(&final_surface)?;
        ctx.set_source_rgba(0.0, 0.0, 0.0, 0.0);
        ctx.paint()?;
    }

    // 逐个处理显示器
    for &id in &monitor_ids {
        // 安全获取可变引用（每次循环单独借用）
        let output = outputs
            .get_mut(&id)
            .ok_or_else(|| format!("显示器{}不存在", id))?;

        let rect = output
            .subrect
            .as_ref()
            .ok_or_else(|| format!("显示器{}没有设置子区域", id))?;

        // 安全检查
        if rect.relative_min_x + rect.width > output.width
            || rect.relative_min_y + rect.height > output.height
        {
            return Err(format!(
                "子区域超出边界 (显示器{}: {}x{}, 子区域: {}x{} @ ({},{})",
                id,
                output.width,
                output.height,
                rect.width,
                rect.height,
                rect.relative_min_x,
                rect.relative_min_y
            )
            .into());
        }

        // 获取画布数据
        let canvas = output
            .base_canvas
            .as_mut()
            .ok_or_else(|| format!("显示器{}的画布未初始化", id))?;

        // 计算目标位置
        let dest_x = (output.global_x + rect.relative_min_x) - min_x;
        let dest_y = (output.global_y + rect.relative_min_y) - min_y;

        // 创建子区域surface
        let stride = output.width as usize * 4;
        let start_offset =
            (rect.relative_min_y as usize * stride) + (rect.relative_min_x as usize * 4);

        // 内存边界检查
        // let required_bytes = start_offset + (rect.height as usize * stride) * 4;
        // if required_bytes > canvas.len() {
        //     return Err(format!(
        //         "内存越界 (显示器{}: 需要{}字节，实际{}字节)",
        //         id,
        //         required_bytes,
        //         canvas.len()
        //     )
        //     .into());
        // }

        let sub_surface = unsafe {
            cairo::ImageSurface::create_for_data_unsafe(
                canvas.as_mut_ptr().add(start_offset),
                cairo::Format::ARgb32,
                rect.width,
                rect.height,
                stride as i32,
            )?
        };

        // 绘制到最终图像
        let ctx = cairo::Context::new(&final_surface)?;
        ctx.set_source_surface(&sub_surface, dest_x.into(), dest_y.into())?;
        ctx.paint()?;
    }

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

    info!("Screenshot saved to: {}", output_path);
    Ok(())
}
// // 保存文件（增强错误处理）
// let output_path = format!(
//     "screenshot_{}.png",
//     chrono::Local::now().format("%Y%m%d_%H%M%S")
// );
// let mut file = std::fs::OpenOptions::new()
//     .write(true)
//     .create_new(true)
//     .open(&output_path)
//     .map_err(|e| format!("创建文件失败: {}", e))?;
//
// final_surface
//     .write_to_png(&mut file)
//     .map_err(|e| format!("写入PNG失败: {}", e))?;
//
// file.flush().map_err(|e| format!("刷新文件失败: {}", e))?;
//
// info!("Screenshot saved to: {}", output_path);
// Ok(())
