use std::collections::HashMap;

use log::*;
use smithay_client_toolkit::{delegate_shm, shm::ShmHandler};
use wayland_client::{
    Dispatch, Proxy,
    globals::GlobalListContents,
    protocol::{
        wl_compositor, wl_keyboard, wl_output, wl_pointer, wl_registry, wl_seat, wl_surface,
    },
};
use wayland_protocols::{
    wp::cursor_shape::v1::client::{wp_cursor_shape_device_v1, wp_cursor_shape_manager_v1},
    xdg::xdg_output::zv1::client::{zxdg_output_manager_v1, zxdg_output_v1},
};
use wayland_protocols_wlr::{
    layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1},
    screencopy::v1::client::{zwlr_screencopy_frame_v1, zwlr_screencopy_manager_v1},
};

use crate::foam_shot::{FoamShot, hs_insert};
use crate::mode::Mode;

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
                            let outputs = app.wayland_ctx.outputs.get_or_insert_with(Vec::new);
                            let index = outputs.len();
                            outputs.push(proxy.bind(name, version, qh, index));
                        }

                        // Layer shell 绑定
                        _ if interface_name
                            == zwlr_layer_shell_v1::ZwlrLayerShellV1::interface().name =>
                        {
                            if app.wayland_ctx.layer_shell.is_none() {
                                let layer_shell = proxy.bind(name, version, qh, ());
                                app.wayland_ctx.layer_shell = Some((layer_shell, name));
                            }
                        }

                        // Screencopy manager 绑定
                        _ if interface_name
                            == zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1::interface()
                                .name =>
                        {
                            if app.wayland_ctx.screencopy_manager.is_none() {
                                let manager = proxy.bind(name, version, qh, ());
                                app.wayland_ctx.screencopy_manager = Some((manager, name));
                            }
                        }

                        // Cursor shape 相关绑定
                        _ if interface_name
                            == wp_cursor_shape_manager_v1::WpCursorShapeManagerV1::interface()
                                .name =>
                        {
                            if app
                                .wayland_ctx
                                .pointer_helper
                                .cursor_shape_manager
                                .is_none()
                            {
                                let manager: wp_cursor_shape_manager_v1::WpCursorShapeManagerV1 =
                                    proxy.bind(name, version, qh, ());
                                let pointer = app.wayland_ctx.pointer_helper.get_pointer();
                                let device = manager.get_pointer(pointer, qh, ());
                                app.wayland_ctx.pointer_helper.cursor_shape_manager =
                                    Some((manager, name));
                                app.wayland_ctx.pointer_helper.cursor_shape_device = Some(device);
                            }
                        }
                        _ if interface_name
                            == zxdg_output_manager_v1::ZxdgOutputManagerV1::interface().name =>
                        {
                            if app.wayland_ctx.xdg_output_manager.is_none() {
                                let manager = proxy.bind(name, version, qh, ());
                                app.wayland_ctx.xdg_output_manager = Some((manager, name));
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
                    }
                }
            }
            _ => (),
        }
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
// NOTE: unused
#[allow(unused_variables)]
impl Dispatch<zwlr_layer_shell_v1::ZwlrLayerShellV1, ()> for FoamShot {
    fn event(
        app: &mut Self,
        proxy: &zwlr_layer_shell_v1::ZwlrLayerShellV1,
        event: <zwlr_layer_shell_v1::ZwlrLayerShellV1 as Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
    }
}
// NOTE: unused
#[allow(unused_variables)]
impl Dispatch<zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1, ()> for FoamShot {
    fn event(
        app: &mut Self,
        proxy: &zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1,
        event: <zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1 as Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
    }
}
// NOTE: unused
#[allow(unused_variables)]
impl Dispatch<wp_cursor_shape_manager_v1::WpCursorShapeManagerV1, ()> for FoamShot {
    fn event(
        app: &mut Self,
        proxy: &wp_cursor_shape_manager_v1::WpCursorShapeManagerV1,
        event: <wp_cursor_shape_manager_v1::WpCursorShapeManagerV1 as Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
    }
}
// NOTE: unused
#[allow(unused_variables)]
impl Dispatch<wp_cursor_shape_device_v1::WpCursorShapeDeviceV1, ()> for FoamShot {
    fn event(
        app: &mut Self,
        proxy: &wp_cursor_shape_device_v1::WpCursorShapeDeviceV1,
        event: <wp_cursor_shape_device_v1::WpCursorShapeDeviceV1 as Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
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
            wl_pointer::Event::Enter {
                serial,
                surface,
                surface_x,
                surface_y,
            } => match app.wayland_ctx.pointer_helper.start_pos {
                Some(_) => (),
                None => {
                    app.wayland_ctx.pointer_helper.start_pos = Some((surface_x, surface_y));
                }
            },
            _ => (),
        }
    }
}
// TODO:
#[allow(unused_variables)]
impl Dispatch<wl_keyboard::WlKeyboard, ()> for FoamShot {
    fn event(
        state: &mut Self,
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
                // debug!("{}", key);
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
                hs_insert(&mut app.wayland_ctx.widths, *data, width);
                hs_insert(&mut app.wayland_ctx.heights, *data, height);
            }
            wl_output::Event::Geometry {
                x: _,
                y: _,
                physical_width: _,
                physical_height: _,
                subpixel: _,
                make: _,
                model: _,
                transform,
            } => {
                debug!("Received wl_output::Event::Geometry for output {}", data);
                // describes transformations that clients and compositors apply to buffer contents

                let Some((xdg_output_manager, _)) = &app.wayland_ctx.xdg_output_manager else {
                    error!("No ZxdgOutputManagerV1 loaded");
                    return;
                };
                // create an xdg_output object for this wl_output
                xdg_output_manager.get_xdg_output(proxy, &qh, *data as i64);

                let Some((compositor, _)) = &app.wayland_ctx.compositor else {
                    error!("No Compositor");
                    return;
                };
                // TODO: create surface
                trace!("create surface");
                hs_insert(
                    &mut app.freeze_mode.surface,
                    *data,
                    compositor.create_surface(&qh, 1),
                );
                hs_insert(
                    &mut app.select_mode.surface,
                    *data,
                    compositor.create_surface(&qh, 1),
                );
            }
            _ => {}
        };
    }
}

#[allow(unused_variables)]
impl Dispatch<wl_surface::WlSurface, u8> for FoamShot {
    fn event(
        app: &mut Self,
        proxy: &wl_surface::WlSurface,
        event: <wl_surface::WlSurface as Proxy>::Event,
        data: &u8,
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1, usize> for FoamShot {
    fn event(
        app: &mut Self,
        proxy: &zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1,
        event: <zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1 as Proxy>::Event,
        data: &usize,
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            zwlr_screencopy_frame_v1::Event::Buffer {
                format,
                width,
                height,
                stride,
            } => {
                trace!(
                    "creating buffer: data is {}, width: {}, height: {}, stride: {}, format: {:?}",
                    data, width, height, stride, format
                );
                let (buffer, _canvas) = app
                    .wayland_ctx
                    .pool
                    .as_mut()
                    .unwrap()
                    .create_buffer(
                        width as i32,
                        height as i32,
                        stride as i32,
                        format.into_result().expect("Unsupported format"),
                    )
                    .unwrap();

                match &app.wayland_ctx.base_buffers {
                    Some(_) => {
                        app.wayland_ctx
                            .base_buffers
                            .as_mut()
                            .unwrap()
                            .insert(*data, buffer);
                    }
                    None => {
                        app.wayland_ctx.base_buffers = Some(HashMap::new());
                        app.wayland_ctx
                            .base_buffers
                            .as_mut()
                            .unwrap()
                            .insert(*data, buffer);
                    }
                }
            }
            zwlr_screencopy_frame_v1::Event::BufferDone { .. } => {
                let Some(buffer) = &app.wayland_ctx.base_buffers else {
                    error!("Could not load WlBuffers");
                    return;
                };
                trace!("data:{}, copy frame to buffer", data);
                // copy frame to buffer, sends Ready when successful
                proxy.copy(buffer.get(data).unwrap().wl_buffer());
            }
            zwlr_screencopy_frame_v1::Event::Ready { .. } => {
                trace!("data:{}, frame ready", data);
                app.wayland_ctx.frames_ready += 1;
            }
            zwlr_screencopy_frame_v1::Event::Failed => {
                app.mode = Mode::Exit;
            }
            _ => (),
        }
    }
}

impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, i32> for FoamShot {
    fn event(
        _state: &mut Self,
        proxy: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        event: <zwlr_layer_surface_v1::ZwlrLayerSurfaceV1 as Proxy>::Event,
        _data: &i32,
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            zwlr_layer_surface_v1::Event::Configure {
                serial,
                width: _,
                height: _,
            } => {
                // acknowledge the Configure event
                proxy.ack_configure(serial);
            }
            zwlr_layer_surface_v1::Event::Closed => {
                proxy.destroy();
            }
            _ => (),
        }
    }
}
// NOTE: unused
#[allow(unused_variables)]
impl Dispatch<zxdg_output_manager_v1::ZxdgOutputManagerV1, ()> for FoamShot {
    fn event(
        app: &mut Self,
        proxy: &zxdg_output_manager_v1::ZxdgOutputManagerV1,
        event: <zxdg_output_manager_v1::ZxdgOutputManagerV1 as Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        // todo!()
    }
}
// TODO:
impl Dispatch<zxdg_output_v1::ZxdgOutputV1, i64> for FoamShot {
    fn event(
        state: &mut Self,
        proxy: &zxdg_output_v1::ZxdgOutputV1,
        event: <zxdg_output_v1::ZxdgOutputV1 as Proxy>::Event,
        data: &i64,
        conn: &wayland_client::Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            zxdg_output_v1::Event::LogicalPosition { x, y } => {
                info!("ZxdgOutputV1::Event::LogicalPosition: {}, {}", x, y);
            }
            zxdg_output_v1::Event::LogicalSize { width, height } => {
                info!("ZxdgOutputV1::Event::LogicalSize: {}, {}", width, height);
            }
            zxdg_output_v1::Event::Description { description } => {
                info!("ZxdgOutputV1::Event::Description: {}", description);
            }
            zxdg_output_v1::Event::Name { name } => {
                info!("ZxdgOutputV1::Event::Name: {}", name);
            }
            _ => (),
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
