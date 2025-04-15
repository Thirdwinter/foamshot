use crate::foam_outputs::FoamOutput;
use crate::wayland_ctx::WaylandCtx;
use std::error::Error;

// 捕获区域信息结构体
pub struct CaptureInfo {
    pub min_x: i32,
    pub min_y: i32,
    pub total_width: u32,
    pub total_height: u32,
    pub monitor_ids: Vec<usize>,
}

/// 计算捕获区域信息
pub fn calculate_capture_info(wl_ctx: &WaylandCtx) -> Result<Option<CaptureInfo>, Box<dyn Error>> {
    let outputs = wl_ctx.foam_outputs.as_ref().ok_or("未初始化输出设备")?;

    let mut active_info = Vec::new();
    let mut bounds: Option<(i32, i32, i32, i32)> = None;

    for output in outputs {
        let Some(rect) = &output.subrect else {
            continue;
        };

        let global_x = output.global_x + rect.relative_min_x;
        let global_y = output.global_y + rect.relative_min_y;
        let right = global_x + rect.width;
        let bottom = global_y + rect.height;

        bounds = Some(match bounds {
            Some((px, py, pr, pb)) => (
                px.min(global_x),
                py.min(global_y),
                pr.max(right),
                pb.max(bottom),
            ),
            None => (global_x, global_y, right, bottom),
        });

        active_info.push((rect.monitor_id, global_x, global_y));
    }

    let Some((x, y, r, b)) = bounds else {
        return Ok(None);
    };

    Ok(Some(CaptureInfo {
        min_x: x,
        min_y: y,
        total_width: (r - x) as u32,
        total_height: (b - y) as u32,
        monitor_ids: active_info.iter().map(|(id, _, _)| *id).collect(),
    }))
}

/// 创建透明背景的最终画布
pub fn create_final_surface(
    width: u32,
    height: u32,
) -> Result<cairo::ImageSurface, Box<dyn Error>> {
    let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, width as i32, height as i32)?;

    let ctx = cairo::Context::new(&surface)?;
    ctx.set_source_rgba(0.0, 0.0, 0.0, 0.0);
    ctx.paint()?;

    Ok(surface)
}

/// 处理所有显示器输出
/// 修改最终传入的final_surface
pub fn process_all_outputs(
    wl_ctx: &mut WaylandCtx,
    capture_info: &CaptureInfo,
    final_surface: &cairo::ImageSurface,
) -> Result<(), Box<dyn Error>> {
    let outputs = wl_ctx.foam_outputs.as_mut().ok_or("输出设备未初始化")?;

    for &id in &capture_info.monitor_ids {
        let output = outputs
            .get_mut(id)
            .ok_or_else(|| format!("显示器{}不存在", id))?;

        process_single_output(
            output,
            wl_ctx
                .scm
                .base_canvas
                .as_mut()
                .unwrap()
                .get_mut(&id)
                .unwrap(),
            capture_info,
            final_surface,
        )?;
    }

    Ok(())
}

/// 处理单个显示器输出
pub fn process_single_output(
    output: &mut FoamOutput,
    base_canvas: &mut [u8],
    capture_info: &CaptureInfo,
    final_surface: &cairo::ImageSurface,
) -> Result<(), Box<dyn Error>> {
    let rect = output
        .subrect
        .as_ref()
        .ok_or_else(|| format!("显示器{}没有设置子区域", output.id))?;

    if rect.relative_min_x + rect.width > output.width
        || rect.relative_min_y + rect.height > output.height
    {
        return Err(format!(
            "子区域超出边界 (显示器{}: {}x{}, 子区域: {}x{} @ ({},{}))",
            output.id,
            output.width,
            output.height,
            rect.width,
            rect.height,
            rect.relative_min_x,
            rect.relative_min_y
        )
        .into());
    }

    // let canvas = output
    //     .base_canvas
    //     .as_mut()
    //     .ok_or_else(|| format!("显示器{}的画布未初始化", output.id))?;

    let dest_x = (output.global_x + rect.relative_min_x) - capture_info.min_x;
    let dest_y = (output.global_y + rect.relative_min_y) - capture_info.min_y;

    let sub_surface = create_sub_surface(
        base_canvas,
        output.width,
        rect.relative_min_x,
        rect.relative_min_y,
        rect.width,
        rect.height,
    )?;

    let ctx = cairo::Context::new(final_surface)?;
    ctx.set_source_surface(&sub_surface, dest_x as f64, dest_y as f64)?;
    ctx.paint()?;

    Ok(())
}

/// 创建子区域surface
pub fn create_sub_surface(
    canvas: &mut [u8],
    output_width: i32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Result<cairo::ImageSurface, Box<dyn Error>> {
    let stride = output_width as usize * 4;
    let start_offset = (y as usize * stride) + (x as usize * 4);

    let surface = unsafe {
        cairo::ImageSurface::create_for_data_unsafe(
            canvas.as_mut_ptr().add(start_offset),
            cairo::Format::ARgb32,
            width,
            height,
            stride as i32,
        )?
    };

    Ok(surface)
}
