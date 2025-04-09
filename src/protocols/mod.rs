mod wp_cursor_shape_manager_v1;
mod wp_viewporter;
mod xdg;
mod zwlr_layer_shell_v1;
mod zwlr_screencopy_manager_v1;

use image::codecs::png::ApngDecoder;
use log::*;
use smithay_client_toolkit::{
    delegate_shm,
    shm::{ShmHandler, slot::SlotPool},
};
use wayland_client::{
    Dispatch, Proxy,
    globals::GlobalListContents,
    protocol::{
        wl_callback, wl_compositor, wl_keyboard, wl_output, wl_pointer, wl_registry, wl_seat,
        wl_surface,
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
    action::{Action, IsFreeze},
    foam_outputs,
    foamshot::FoamShot,
    notify,
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
                trace!("Registry global: {} {} {}", name, interface, version);

                // 使用更清晰的模式匹配结构
                match interface.as_str() {
                    interface_name => match interface_name {
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
                            if app.wayland_ctx.screencopy_manager.is_none() {
                                let manager: ZwlrScreencopyManagerV1 =
                                    proxy.bind(name, version, qh, ());
                                app.wayland_ctx.screencopy_manager = Some((manager.clone(), name));
                                app.wayland_ctx.scm =
                                    ZwlrScreencopyMode::new((manager.clone(), name))
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
                                let manager: WpCursorShapeManagerV1 =
                                    proxy.bind(name, version, qh, ());
                                app.wayland_ctx.pointer_helper.cursor_shape_manager =
                                    Some((manager, name));
                                // FIX: this time meybe can not get wl_pointer
                                // let pointer =
                                //     app.wayland_ctx.pointer_helper.pointer.as_ref().unwrap();
                                // let device = manager.get_pointer(pointer, qh, ());
                                // app.wayland_ctx.pointer_helper.cursor_shape_device = Some(device);
                            }
                        }
                        // NOTE: xdg_output_manager 处理 多输出
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
                        _ if interface_name == WpViewporter::interface().name => {
                            if app.wayland_ctx.viewporter.is_none() {
                                let viewporter = proxy.bind(name, version, qh, ());
                                app.wayland_ctx.viewporter = Some((viewporter, name));
                            }
                        }

                        _ => (),
                    },
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
                    } else if let Some((_, screencopymanager_name)) =
                        &app.wayland_ctx.screencopy_manager
                    {
                        if name == *screencopymanager_name {
                            warn!("ZwlrScreencopyManagerV1 was removed");
                            app.wayland_ctx.screencopy_manager = None;
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
                app.wayland_ctx
                    .set_cursor_shape(serial, Shape::Default, proxy)
                    .ok();
            }
            wl_pointer::Event::Enter {
                serial,
                surface,
                surface_x,
                surface_y,
            } => {
                let a: Option<&usize> = surface.data();
                app.wayland_ctx.unknow_index = a.copied();
                // debug!(
                //     "serial {}: pointer enter surface {}, surface_x:{}, surface_y:{}",
                //     serial,
                //     a.unwrap(),
                //     surface_x,
                //     surface_y
                // );

                if let Err(e) = app
                    .wayland_ctx
                    .set_cursor_shape(serial, Shape::Crosshair, proxy)
                {
                    notify::send(notify::NotificationLevel::Warn, "can not set cursor shape")
                }

                let foam_output = app
                    .wayland_ctx
                    .foam_outputs
                    .as_ref()
                    .unwrap()
                    .get(a.unwrap())
                    .unwrap();

                // NOTE: 给定坐标+对应output的LogicalPosition => 相对于surface左上角的相对坐标
                let (x, y) = (
                    surface_x + foam_output.global_x as f64,
                    surface_y + foam_output.global_y as f64,
                );
                // FIX:
                if x >= 0.0
                    && y >= 0.0
                    && x <= foam_output.width as f64
                    && y <= foam_output.height as f64
                {
                    debug!("surface enter output:{} x:{}, y:{}", foam_output.name, x, y);

                    app.wayland_ctx.current_index = Some(a.unwrap().clone());
                    match app.wayland_ctx.pointer_helper.start_index {
                        Some(_) => (),
                        None => {
                            app.wayland_ctx.pointer_helper.start_index = Some(a.unwrap().clone());
                        }
                    }
                    match app.wayland_ctx.pointer_helper.current_pos {
                        Some(_) => (),
                        None => {
                            app.wayland_ctx.pointer_helper.current_pos = Some((x, y));
                        }
                    }
                }
            }
            wl_pointer::Event::Button {
                serial,
                time,
                button,
                state,
            } => {
                if let Some(state) = state.into_result().ok() {
                    match state {
                        wl_pointer::ButtonState::Pressed => {
                            app.wayland_ctx.pointer_helper.is_pressing = true;

                            app.wayland_ctx.pointer_helper.start_index =
                                app.wayland_ctx.current_index;

                            app.wayland_ctx.pointer_helper.start_pos =
                                app.wayland_ctx.pointer_helper.current_pos.clone();

                            app.wayland_ctx.generate_sub_rects();

                            app.mode = Action::OnDraw;
                        }
                        wl_pointer::ButtonState::Released => {
                            app.wayland_ctx.pointer_helper.is_pressing = false;
                            app.wayland_ctx.pointer_helper.end_index =
                                app.wayland_ctx.current_index;

                            app.wayland_ctx.pointer_helper.end_pos =
                                app.wayland_ctx.pointer_helper.current_pos.clone();

                            app.mode = Action::Exit;
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
                // debug!("Pointer::Motion => x: {}, y: {}", surface_x, surface_y);
                let unknow_index = app.wayland_ctx.unknow_index.unwrap();
                let start_index = app.wayland_ctx.pointer_helper.start_index.unwrap();
                let start_output = app
                    .wayland_ctx
                    .foam_outputs
                    .as_ref()
                    .unwrap()
                    .get(&start_index)
                    .unwrap();
                let unkonw_output = app
                    .wayland_ctx
                    .foam_outputs
                    .as_ref()
                    .unwrap()
                    .get(&unknow_index)
                    .unwrap();
                // debug!("motion: surface_x:{}, surface_y:{}", surface_x, surface_y);
                let (x, y) = foam_outputs::FoamOutput::convert_pos_to_surface(
                    unkonw_output,
                    start_output,
                    surface_x,
                    surface_y,
                );

                app.wayland_ctx.pointer_helper.current_pos = Some((x, y));
                match app.mode {
                    Action::OnDraw => {
                        // TODO:
                        app.wayland_ctx.generate_sub_rects();
                    }
                    _ => (),
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
        if let wl_keyboard::Event::Key {
            serial: _,
            time: _,
            key,
            state: key_state,
        } = event
        {
            if let wayland_client::WEnum::Value(wl_keyboard::KeyState::Pressed) = key_state {
                debug!("{}", key);
                if key == 33 {
                    // TODO: pressed `f` to toggle freeze
                    if app.wayland_ctx.current_freeze {
                        app.mode = Action::ToggleFreeze(IsFreeze::UnFreeze);
                        app.wayland_ctx.current_freeze = false;
                    } else {
                        app.mode = Action::ToggleFreeze(IsFreeze::Freeze);
                        app.wayland_ctx.current_freeze = true;
                    }
                }
                if key == 1 {
                    std::process::exit(0);
                }
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
                let _ = xdg_output_manager.get_xdg_output(proxy, &qh, *data);

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
                foam_output.surface = Some(compositor.create_surface(&qh, *data));
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
        let outputs = app.wayland_ctx.foam_outputs.as_mut().unwrap();
        match app.mode {
            Action::ToggleFreeze(_) => {
                debug!("will be re attach_all");
                app.wayland_ctx.attach_with_udata(*data);
                app.mode = Action::WaitPointerPress
            }
            Action::OnDraw => {
                if outputs.get_mut(data).unwrap().is_dirty() {
                    let base_canvas = app
                        .wayland_ctx
                        .scm
                        .base_canvas
                        .as_mut()
                        .unwrap()
                        .get_mut(&data)
                        .unwrap();
                    outputs
                        .get_mut(data)
                        .unwrap()
                        .update_select_subrect(base_canvas, app.wayland_ctx.current_freeze);
                }
            }
            _ => {}
        }

        // todo!()
    }
}

impl ShmHandler for FoamShot {
    fn shm_state(&mut self) -> &mut smithay_client_toolkit::shm::Shm {
        self.wayland_ctx.shm.as_mut().unwrap()
    }
}

delegate_shm!(FoamShot);
