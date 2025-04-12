mod wp_cursor_shape_manager_v1;
mod wp_viewporter;
mod xdg;
mod zwlr_layer_shell_v1;
mod zwlr_screencopy_manager_v1;

use log::*;
use smithay_client_toolkit::{
    delegate_shm,
    shm::{ShmHandler, slot::SlotPool},
};
use wayland_client::{
    Dispatch, Proxy,
    globals::GlobalListContents,
    protocol::{
        wl_callback::{self},
        wl_compositor, wl_keyboard, wl_output, wl_pointer, wl_registry, wl_seat, wl_surface,
    },
};
use wayland_protocols::{
    wp::{
        cursor_shape::v1::client::{
            wp_cursor_shape_device_v1::Shape, wp_cursor_shape_manager_v1::WpCursorShapeManagerV1,
        },
        viewporter::client::wp_viewporter::WpViewporter,
    },
    xdg::{shell::client::xdg_wm_base, xdg_output::zv1::client::zxdg_output_manager_v1},
};
use wayland_protocols_wlr::{
    layer_shell::v1::client::zwlr_layer_shell_v1::ZwlrLayerShellV1,
    screencopy::v1::client::zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1,
};

use crate::{
    action::{Action, EditAction, IsFreeze},
    foam_outputs,
    foamshot::FoamShot,
    zwlr_screencopy_mode::ZwlrScreencopyMode,
};

impl Dispatch<wl_registry::WlRegistry, ()> for FoamShot {
    fn event(
        app: &mut Self,
        proxy: &wl_registry::WlRegistry,
        event: <wl_registry::WlRegistry as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            wl_registry::Event::Global {
                name,
                interface,
                version,
            } => {
                // trace!("Registry global: {} {} {}", name, interface, version);

                let interface_name = interface.as_str();
                match interface_name {
                    // Compositor 绑定
                    _ if interface_name == wl_compositor::WlCompositor::interface().name => {
                        if app.wayland_ctx.compositor.is_none() {
                            let compositor = proxy.bind(name, version, qh, ());
                            app.wayland_ctx.compositor = Some((compositor, name));
                        }
                    }
                    // Seat 绑定及相关资源获取
                    _ if interface_name == wl_seat::WlSeat::interface().name => {
                        if app.wayland_ctx.seat.is_none() {
                            let seat: wl_seat::WlSeat = proxy.bind(name, version, qh, ());
                            let pointer = seat.get_pointer(qh, ());
                            let keyboard = seat.get_keyboard(qh, ());
                            app.wayland_ctx.pointer_helper.pointer = Some(pointer);
                            app.wayland_ctx.keyboard = Some(keyboard);
                            app.wayland_ctx.seat = Some((seat, name));
                        }
                    }
                    // 动态管理 outputs
                    _ if interface_name == wl_output::WlOutput::interface().name => {
                        let outputs = app.wayland_ctx.foam_outputs.as_mut().unwrap();
                        let index = outputs.len();
                        let shm = app.wayland_ctx.shm.as_mut().unwrap();
                        let pool =
                            SlotPool::new(256 * 256 * 4, shm).expect("Failed to create pool");
                        let foam_output = foam_outputs::FoamOutput::new(
                            index,
                            proxy.bind(name, version, qh, index),
                            pool,
                        );
                        outputs.insert(index, foam_output);
                    }
                    // Layer shell 绑定
                    _ if interface_name == ZwlrLayerShellV1::interface().name => {
                        if app.wayland_ctx.layer_shell.is_none() {
                            let layer_shell = proxy.bind(name, version, qh, ());
                            app.wayland_ctx.layer_shell = Some((layer_shell, name));
                        }
                    }
                    // Screencopy manager 绑定
                    _ if interface_name == ZwlrScreencopyManagerV1::interface().name => {
                        if app.wayland_ctx.scm.manager.is_none() {
                            let manager: ZwlrScreencopyManagerV1 =
                                proxy.bind(name, version, qh, ());
                            // app.wayland_ctx.screencopy_manager = Some((manager.clone(), name));
                            app.wayland_ctx.scm = ZwlrScreencopyMode::new((manager, name));
                        }
                    }
                    // Cursor shape 相关绑定
                    _ if interface_name == WpCursorShapeManagerV1::interface().name => {
                        if app
                            .wayland_ctx
                            .pointer_helper
                            .cursor_shape_manager
                            .is_none()
                        {
                            let manager: WpCursorShapeManagerV1 = proxy.bind(name, version, qh, ());
                            app.wayland_ctx.pointer_helper.cursor_shape_manager =
                                Some((manager, name));
                        }
                    }
                    // NOTE: xdg_output_manager 处理多输出
                    _ if interface_name
                        == zxdg_output_manager_v1::ZxdgOutputManagerV1::interface().name =>
                    {
                        if app.wayland_ctx.xdg_output_manager.is_none() {
                            let manager = proxy.bind(name, version, qh, ());
                            app.wayland_ctx.xdg_output_manager = Some((manager, name));
                        }
                    }
                    // xdgwmbase
                    _ if interface_name == xdg_wm_base::XdgWmBase::interface().name => {
                        if app.wayland_ctx.xdgwmbase.is_none() {
                            let base = proxy.bind(name, version, qh, ());
                            app.wayland_ctx.xdgwmbase = Some((base, name));
                        }
                    }
                    // Viewporter
                    _ if interface_name == WpViewporter::interface().name => {
                        if app.wayland_ctx.viewporter.is_none() {
                            let viewporter = proxy.bind(name, version, qh, ());
                            app.wayland_ctx.viewporter = Some((viewporter, name));
                        }
                    }
                    _ => (),
                }
            }
            wl_registry::Event::GlobalRemove { name } => {
                if let Some((_, compositor_name)) = &app.wayland_ctx.compositor {
                    if name == *compositor_name {
                        warn!("WlCompositor was removed");
                        app.wayland_ctx.compositor = None;
                    } else if let Some((_, sate_name)) = &app.wayland_ctx.seat {
                        if name == *sate_name {
                            warn!("WlSeat was removed");
                            app.wayland_ctx.seat = None;
                        }
                    } else if let Some((_, screencopymanager_name)) = &app.wayland_ctx.scm.manager {
                        if name == *screencopymanager_name {
                            warn!("ZwlrScreencopyManagerV1 was removed");
                            app.wayland_ctx.scm.manager = None;
                        }
                    } else if let Some((_, layer_shell_name)) = &app.wayland_ctx.layer_shell {
                        if name == *layer_shell_name {
                            warn!("ZwlrLayerShellV1 was removed");
                            app.wayland_ctx.layer_shell = None;
                        }
                    } else if let Some((_, cursor_shape_manager_name)) =
                        &app.wayland_ctx.pointer_helper.cursor_shape_manager
                    {
                        if name == *cursor_shape_manager_name {
                            warn!("WpCursorShapeManagerV1 was removed");
                            app.wayland_ctx.pointer_helper.cursor_shape_manager = None;
                        }
                    } else if let Some((_, viewporter_name)) = &app.wayland_ctx.viewporter {
                        if name == *viewporter_name {
                            warn!("WpViewporter was removed");
                            app.wayland_ctx.viewporter = None;
                        }
                    }
                }
            }
            _ => (),
        }
    }
}

// TODO:
#[allow(unused_variables)]
impl Dispatch<wl_pointer::WlPointer, ()> for FoamShot {
    fn event(
        app: &mut Self,
        proxy: &wl_pointer::WlPointer,
        event: <wl_pointer::WlPointer as Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            wl_pointer::Event::Leave { serial, surface } => {
                let _ = app
                    .wayland_ctx
                    .set_cursor_shape(serial, Shape::Default, proxy);
            }

            wl_pointer::Event::Enter {
                serial,
                surface,
                surface_x,
                surface_y,
            } => {
                let surface_index = match surface.data::<usize>() {
                    Some(idx) => *idx,
                    None => {
                        error!("can not get surface index, exit!");
                        std::process::exit(0)
                    }
                };
                app.wayland_ctx.unknown_index = Some(surface_index);

                // set cursor shape
                if let Err(_) = app
                    .wayland_ctx
                    .set_cursor_shape(serial, Shape::Crosshair, proxy)
                {
                    app.send_warn("can not set cursor shape");
                }

                let foam_output = match app
                    .wayland_ctx
                    .foam_outputs
                    .as_ref()
                    .and_then(|outputs| outputs.get(&surface_index))
                {
                    Some(output) => output,
                    None => {
                        error!("can not get foam_output, exit!");
                        std::process::exit(0)
                    }
                };

                let x = surface_x + foam_output.global_x as f64;
                let y = surface_y + foam_output.global_y as f64;

                // 发送多个enter时候，只选择满足坐标约束的
                if x >= 0.0
                    && y >= 0.0
                    && x <= foam_output.width as f64
                    && y <= foam_output.height as f64
                {
                    debug!("surface enter output:{} x:{}, y:{}", foam_output.name, x, y);

                    // NOTE:  full screen mode handle
                    if app.wayland_ctx.config.full_screen {
                        app.wayland_ctx.set_one_max(surface_index);
                        app.action = Action::Exit;
                        return;
                    }

                    app.wayland_ctx.current_index = Some(surface_index);
                    app.wayland_ctx
                        .pointer_helper
                        .start_index
                        .get_or_insert(surface_index);

                    // 鼠标未移动时进行初始化
                    if app.wayland_ctx.pointer_helper.g_current_pos.is_none() {
                        let global_pos = (
                            x + foam_output.global_x as f64,
                            y + foam_output.global_y as f64,
                        );
                        app.wayland_ctx.pointer_helper.g_current_pos = Some(global_pos);
                    }
                }
            }

            wl_pointer::Event::Button {
                serial,
                time,
                button,
                state,
            } => {
                if let Ok(button_state) = state.into_result() {
                    match button_state {
                        wl_pointer::ButtonState::Pressed => {
                            app.wayland_ctx.pointer_helper.is_pressing = true;
                            match app.action {
                                Action::WaitPointerPress => {
                                    app.wayland_ctx.pointer_helper.start_index =
                                        app.wayland_ctx.current_index;
                                    app.wayland_ctx.pointer_helper.g_start_pos =
                                        app.wayland_ctx.pointer_helper.g_current_pos;
                                    app.wayland_ctx.generate_rects_and_send_frame();
                                    app.action = Action::OnDraw;
                                }
                                Action::OnEdit(EditAction::None) => {
                                    app.wayland_ctx.pointer_helper.g_start_pos =
                                        app.wayland_ctx.pointer_helper.g_current_pos;

                                    if let Some(current_pos) =
                                        app.wayland_ctx.pointer_helper.g_current_pos
                                    {
                                        if let Some(global_rect) =
                                            app.wayland_ctx.global_rect.as_ref()
                                        {
                                            let hit_region = global_rect.hit_region(
                                                current_pos.0 as i32,
                                                current_pos.1 as i32,
                                                15,
                                            );
                                            app.action = Action::OnEdit(hit_region);
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        wl_pointer::ButtonState::Released => {
                            app.wayland_ctx.pointer_helper.is_pressing = false;

                            if app.action == Action::OnDraw {
                                app.wayland_ctx.pointer_helper.end_index =
                                    app.wayland_ctx.current_index;
                                app.wayland_ctx.pointer_helper.g_end_pos =
                                    app.wayland_ctx.pointer_helper.g_current_pos;
                            }

                            app.action = if app.wayland_ctx.config.edit {
                                Action::OnEdit(EditAction::None)
                            } else {
                                Action::Exit
                            };
                        }
                        _ => (),
                    }
                }
            }

            wl_pointer::Event::Motion {
                time,
                surface_x,
                surface_y,
            } => {
                let (unknown_index, start_index) = match (
                    app.wayland_ctx.unknown_index,
                    app.wayland_ctx.pointer_helper.start_index,
                ) {
                    (Some(u), Some(s)) => (u, s),
                    _ => {
                        error!("can not get surface index, exit!");
                        std::process::exit(0)
                    }
                };

                let outputs = match app.wayland_ctx.foam_outputs.as_ref() {
                    Some(o) => o,
                    None => {
                        error!("can not get foam_outputs, exit!");
                        std::process::exit(0)
                    }
                };

                let (start_output, unknown_output) =
                    match (outputs.get(&start_index), outputs.get(&unknown_index)) {
                        (Some(s), Some(u)) => (s, u),
                        _ => return,
                    };

                let (x, y) = foam_outputs::FoamOutput::convert_pos_to_surface(
                    unknown_output,
                    start_output,
                    surface_x,
                    surface_y,
                );

                let global_pos = (
                    x + start_output.global_x as f64,
                    y + start_output.global_y as f64,
                );
                app.wayland_ctx.pointer_helper.g_current_pos = Some(global_pos);

                match app.action {
                    Action::OnDraw => {
                        app.wayland_ctx.generate_rects_and_send_frame();
                    }
                    Action::OnEdit(edit_action) => match edit_action {
                        // NOTE: 待编辑时候移动光标选择设置合适的形状
                        EditAction::None => {
                            if let Some(global_rect) = app.wayland_ctx.global_rect.as_ref() {
                                let hit_region = global_rect.hit_region(
                                    global_pos.0 as i32,
                                    global_pos.1 as i32,
                                    15,
                                );
                                let _ = app.wayland_ctx.set_cursor_shape(
                                    0,
                                    hit_region.to_cursor_shape(),
                                    proxy,
                                );
                            }
                        }
                        _ => {
                            let _ = app.wayland_ctx.set_cursor_shape(
                                0,
                                edit_action.to_cursor_shape(),
                                proxy,
                            );
                            if let (Some(start_pos), Some(global_rect)) = (
                                app.wayland_ctx.pointer_helper.g_start_pos,
                                app.wayland_ctx.global_rect.as_mut(),
                            ) {
                                app.action = global_rect.edit(start_pos, global_pos, app.action);
                                app.wayland_ctx.process_subrects_and_send();
                            }
                        }
                    },
                    _ => {}
                }
            }
            _ => (),
        }
    }
}

// TODO:
#[allow(unused_variables)]
impl Dispatch<wl_keyboard::WlKeyboard, ()> for FoamShot {
    fn event(
        app: &mut Self,
        proxy: &wl_keyboard::WlKeyboard,
        event: <wl_keyboard::WlKeyboard as Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        const KEY_F: u32 = 33;
        const KEY_ESC: u32 = 1;
        const KEY_A: u32 = 30;
        const KEY_S: u32 = 31;

        // 使用模式匹配替代多重if嵌套
        if let wl_keyboard::Event::Key {
            key,
            state: wayland_client::WEnum::Value(wl_keyboard::KeyState::Pressed),
            ..
        } = event
        {
            debug!("Key pressed: {}", key);

            match key {
                KEY_A => {
                    let current_output = app.wayland_ctx.current_index;
                    app.wayland_ctx.set_one_max(current_output.unwrap());
                    app.action = Action::Exit
                }
                KEY_S => match app.action {
                    Action::WaitPointerPress => {
                        return;
                    }
                    Action::Init => {
                        return;
                    }
                    Action::Exit => {
                        return;
                    }
                    _ => app.action = Action::Exit,
                },
                KEY_F => {
                    app.wayland_ctx.current_freeze = !app.wayland_ctx.current_freeze;
                    app.action = if app.wayland_ctx.current_freeze {
                        Action::ToggleFreeze(IsFreeze::NewFrameFreeze)
                    } else {
                        Action::ToggleFreeze(IsFreeze::UnFreeze)
                    };
                }
                KEY_ESC => match app.action {
                    Action::OnEdit(a) => {
                        app.action = if app.wayland_ctx.current_freeze {
                            Action::ToggleFreeze(IsFreeze::OldFrameFreeze)
                        } else {
                            Action::ToggleFreeze(IsFreeze::UnFreeze)
                        };
                    }
                    _ => {
                        std::process::exit(0);
                    }
                },
                _ => {}
            }
        }
    }
}
// TODO:
#[allow(unused_variables)]
impl Dispatch<wl_output::WlOutput, usize> for FoamShot {
    fn event(
        app: &mut Self,
        proxy: &wl_output::WlOutput,
        event: <wl_output::WlOutput as Proxy>::Event,
        data: &usize,
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            wl_output::Event::Scale { factor } => {
                let mut foam_output = app.wayland_ctx.foam_outputs.as_mut().unwrap().get_mut(data);
                foam_output.as_mut().unwrap().scale = factor.into()
            }
            wl_output::Event::Mode {
                flags: _,
                width,
                height,
                refresh: _,
            } => {
                let mut foam_output = app.wayland_ctx.foam_outputs.as_mut().unwrap().get_mut(data);
                foam_output.as_mut().unwrap().width = width;
                foam_output.as_mut().unwrap().height = height;
                // hs_insert(&mut app.wayland_ctx.widths, *data, width);
                // hs_insert(&mut app.wayland_ctx.heights, *data, height);
            }
            wl_output::Event::Geometry {
                x,
                y,
                physical_width,
                physical_height,
                subpixel: _,
                make: _,
                model: _,
                transform,
            } => {
                debug!(
                    "wl_output::Event::Geometry => output:{} | x:{} | y:{} | physical_width:{} | physical_height:{} | transform:{:?}",
                    data, x, y, physical_width, physical_height, transform
                );

                let Some((xdg_output_manager, _)) = &app.wayland_ctx.xdg_output_manager else {
                    error!("No ZxdgOutputManagerV1 loaded");
                    return;
                };
                // create an xdg_output object for this wl_output
                let _ = xdg_output_manager.get_xdg_output(proxy, qh, *data);

                let Some((compositor, _)) = &app.wayland_ctx.compositor else {
                    error!("No Compositor");
                    return;
                };
                // TODO: create surface
                trace!("create surface");
                let foam_output = app
                    .wayland_ctx
                    .foam_outputs
                    .as_mut()
                    .unwrap()
                    .get_mut(data)
                    .unwrap();
                foam_output.surface = Some(compositor.create_surface(qh, *data));
            }
            _ => {}
        };
    }
}

#[allow(unused_variables)]
impl Dispatch<wl_surface::WlSurface, usize> for FoamShot {
    fn event(
        app: &mut Self,
        proxy: &wl_surface::WlSurface,
        event: <wl_surface::WlSurface as Proxy>::Event,
        data: &usize,
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

// NOTE: unused
#[allow(unused_variables)]
impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for FoamShot {
    fn event(
        app: &mut Self,
        proxy: &wl_registry::WlRegistry,
        event: <wl_registry::WlRegistry as Proxy>::Event,
        data: &GlobalListContents,
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        // todo!()
    }
}
// NOTE: unused
#[allow(unused_variables)]
impl Dispatch<wl_compositor::WlCompositor, ()> for FoamShot {
    fn event(
        app: &mut Self,
        proxy: &wl_compositor::WlCompositor,
        event: <wl_compositor::WlCompositor as Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
    }
}
// NOTE: unused
#[allow(unused_variables)]
impl Dispatch<wl_seat::WlSeat, ()> for FoamShot {
    fn event(
        app: &mut Self,
        proxy: &wl_seat::WlSeat,
        event: <wl_seat::WlSeat as Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

#[allow(unused_variables)]
impl Dispatch<wl_callback::WlCallback, usize> for FoamShot {
    fn event(
        app: &mut Self,
        proxy: &wl_callback::WlCallback,
        event: <wl_callback::WlCallback as Proxy>::Event,
        data: &usize,
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        if let wl_callback::Event::Done { callback_data } = event {
            match app.action {
                Action::OnDraw => {
                    app.wayland_ctx.update_select_region();
                }
                Action::OnEdit(a) => {
                    app.wayland_ctx.update_select_region();
                }
                _ => {}
            }
        }
    }
}

impl ShmHandler for FoamShot {
    fn shm_state(&mut self) -> &mut smithay_client_toolkit::shm::Shm {
        self.wayland_ctx.shm.as_mut().unwrap()
    }
}

delegate_shm!(FoamShot);
