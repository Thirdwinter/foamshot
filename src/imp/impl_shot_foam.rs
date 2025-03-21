use std::{
    fs::File,
    io::{BufWriter, Write},
};

use cairo::{Format, ImageSurface};
use log::{debug, info};
use smithay_client_toolkit::shm::{self, slot};
use wayland_client::{QueueHandle, protocol::wl_shm};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{self, Layer},
    zwlr_layer_surface_v1::{Anchor, KeyboardInteractivity},
};

use crate::{
    check_options, config, freeze_mode::FreezeMode, result_output::ResultOutput,
    select_mode::SelectMode, shot_foam::ShotFoam, utility::Action,
};

impl ShotFoam {
    pub fn new(shm: shm::Shm, pool: slot::SlotPool, qh: QueueHandle<ShotFoam>) -> Self {
        let freeze_mode = FreezeMode::default();
        let select_mode = SelectMode::default();

        Self {
            compositor: None,
            output: None,
            pool: Some(pool),
            shm: Some(shm),
            seat: None,
            pointer: None,
            keyboard: None,
            layer_shell: None,
            qh: Some(qh.clone()),
            phys_width: None,
            phys_height: None,
            current_pos: None,
            pointer_start: None,
            pointer_end: None,
            cursor_shape_manager: None,
            cursor_shape_device: None,
            action: Action::PreLoad,
            freeze_mode,
            select_mode,
            config: config::Config::new(),
            result_output: ResultOutput::default(),
        }
    }

    /// NOTE: 在冻结屏幕前创建一个layer_surface
    pub fn create_freeze_layer_surface(&mut self) {
        let (phys_width, phys_height, screencopy_manager, surface, layer_shell, output, qh) = check_options!(
            self.phys_width,
            self.phys_height,
            self.freeze_mode.screencopy_manager.as_ref(),
            self.freeze_mode.surface.as_ref(),
            self.layer_shell.as_ref(),
            self.output.as_ref(),
            self.qh.as_ref()
        );
        let screencopy_frame =
            screencopy_manager.capture_output(!self.config.no_cursor as i32, &output, &qh, ());
        self.freeze_mode.screencopy_frame = Some(screencopy_frame);

        // 创建 layer
        let layer = zwlr_layer_shell_v1::ZwlrLayerShellV1::get_layer_surface(
            &layer_shell,
            &surface,
            Some(&output),
            Layer::Overlay,
            "foam_freeze".to_string(),
            &qh,
            1,
        );
        info!("create freeze_layer");
        layer.set_anchor(Anchor::all());
        layer.set_exclusive_zone(-1);
        layer.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);
        self.freeze_mode.layer_surface = Some(layer);

        surface.damage(0, 0, phys_width, phys_height);
        surface.commit();
    }

    /// NOTE: 这一步在第一次帧copy后，将帧附加到freeze_surface
    pub fn create_freeze_buffer(&mut self) {
        let (surface, freeze_buffer, phys_width, phys_height) = check_options!(
            self.freeze_mode.surface.as_ref(),
            self.freeze_mode.buffer.as_ref(),
            self.phys_width,
            self.phys_height
        );
        surface.commit();
        freeze_buffer.attach_to(&surface).unwrap();
        surface.damage(0, 0, phys_width, phys_height);
        surface.set_buffer_scale(1);
        // NOTE: 这里切换状态
        info!("attach freeze_buffer and change action to freeze");
        self.action = Action::Freeze;
        surface.commit();
    }

    /// NOTE: 预先生成select layer_surface
    pub fn create_select_layer_surface(&mut self) {
        let (phys_width, phys_height, surface, layer_shell, output, qh) = check_options!(
            self.phys_width,
            self.phys_height,
            self.select_mode.surface.as_ref(),
            self.layer_shell.as_ref(),
            self.output.as_ref(),
            self.qh.as_ref()
        );
        // NOTE: 创建layer
        let layer = zwlr_layer_shell_v1::ZwlrLayerShellV1::get_layer_surface(
            &layer_shell,
            &surface,
            Some(&output),
            Layer::Overlay,
            "foam_select".to_string(),
            &qh,
            2,
        );

        layer.set_anchor(Anchor::all());

        layer.set_exclusive_zone(-1); // 将表面扩展到锚定边缘

        layer.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);
        self.select_mode.layer_surface = Some(layer);

        surface.damage(0, 0, phys_width, phys_height);
        info!("create select_layer");
        surface.commit();
    }

    /// NOTE: 创建select buffer并附加(白色半透明)
    pub fn create_select_buffer(&mut self) {
        let (phys_width, phys_height, surface, pool) = check_options!(
            self.phys_width,
            self.phys_height,
            self.select_mode.surface.as_ref(),
            self.pool.as_mut()
        );
        let (buffer, canvas) = pool
            .create_buffer(
                phys_width as i32,
                phys_height as i32,
                phys_width as i32 * 4,
                wl_shm::Format::Argb8888,
            )
            .unwrap();
        canvas.fill(100);

        buffer.attach_to(surface).unwrap();
        self.select_mode.buffer = Some(buffer);
        // 请求重绘
        self.select_mode
            .surface
            .as_ref()
            .unwrap()
            .damage_buffer(0, 0, phys_width, phys_height);
        surface.commit();
        self.select_mode.surface.as_ref().unwrap().commit();
    }

    /// NOTE: 进行目标区域的copy
    pub fn pre_output_to_png(&mut self) {
        let (screencopy_manager, output, qh, (start_x, start_y), (end_x, end_y)) = check_options!(
            self.result_output.screencopy_manager.as_ref(),
            self.output.as_ref(),
            self.qh.as_ref(),
            self.pointer_start,
            self.pointer_end
        );

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
        self.result_output.start = Some((x as i32, y as i32));
        self.result_output.width = Some(width as i32);
        self.result_output.height = Some(height as i32);

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

    /// NOTE: 等待copy完成（Ready），然后输出
    pub fn output_to_png(&mut self) {
        let (buffer, pool) = check_options!(self.result_output.buffer.as_ref(), self.pool.as_mut());
        let canvas = buffer.canvas(pool).expect("get canvas");
        let cairo_surface = unsafe {
            ImageSurface::create_for_data(
                std::slice::from_raw_parts_mut(canvas.as_mut_ptr(), canvas.len()),
                Format::Rgb24,
                self.result_output.width.unwrap(),
                self.result_output.height.unwrap(),
                self.result_output.width.unwrap() * 4,
            )
            .map_err(|e| format!("Failed to create Cairo surface: {}", e))
            .unwrap()
        };
        let output_path = &self.config.output_path;
        let file = File::create(&output_path).unwrap();
        let mut buffer_writer = BufWriter::new(file);
        cairo_surface
            .write_to_png(&mut buffer_writer)
            .expect("write png");
        buffer_writer.flush().unwrap();
        std::process::exit(0);
    }
}
