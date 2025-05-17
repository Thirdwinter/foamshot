//! INFO: Implementation of the core (general) interface of the Wayland protocol
//! `wl_pointer` and `wl_keyboard` are implemented separately

mod keyboard;
mod pointer;
mod wp_cursor_shape_manager_v1;
mod wp_fractional_scale_v1;
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
        wl_compositor, wl_output, wl_registry, wl_seat, wl_surface,
    },
};
use wayland_protocols::{
    wp::{
        cursor_shape::v1::client::wp_cursor_shape_manager_v1::WpCursorShapeManagerV1,
        fractional_scale::v1::client::wp_fractional_scale_manager_v1::WpFractionalScaleManagerV1,
        viewporter::client::wp_viewporter::WpViewporter,
    },
    xdg::xdg_output::zv1::client::zxdg_output_manager_v1,
};
use wayland_protocols_wlr::{
    layer_shell::v1::client::zwlr_layer_shell_v1::ZwlrLayerShellV1,
    screencopy::v1::client::zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1,
};

use crate::{
    action::Action, foamcore::FoamShot, monitors, zwlr_screencopy_mode::ZwlrScreencopyMode,
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
                        if app.wlctx.compositor.is_none() {
                            let compositor = proxy.bind(name, version, qh, ());
                            app.wlctx.compositor = Some((compositor, name));
                        }
                    }
                    // Seat 绑定及相关资源获取
                    _ if interface_name == wl_seat::WlSeat::interface().name => {
                        if app.wlctx.seat.is_none() {
                            let seat: wl_seat::WlSeat = proxy.bind(name, version, qh, ());
                            let pointer = seat.get_pointer(qh, ());
                            let keyboard = seat.get_keyboard(qh, ());
                            app.wlctx.pointer_helper.pointer = Some(pointer);
                            app.wlctx.keyboard = Some(keyboard);
                            app.wlctx.seat = Some((seat, name));
                        }
                    }
                    // 动态管理 outputs
                    _ if interface_name == wl_output::WlOutput::interface().name => {
                        let outputs = app.wlctx.foam_outputs.as_mut().unwrap();
                        let index = outputs.len();
                        let shm = app.wlctx.shm.as_mut().unwrap();
                        let pool =
                            SlotPool::new(256 * 256 * 4, shm).expect("Failed to create pool");
                        let foam_output = monitors::FoamMonitors::new(
                            index,
                            proxy.bind(name, version, qh, index),
                            pool,
                        );
                        outputs.insert(index, foam_output);
                    }
                    // Layer shell 绑定
                    _ if interface_name == ZwlrLayerShellV1::interface().name => {
                        if app.wlctx.layer_shell.is_none() {
                            let layer_shell = proxy.bind(name, version, qh, ());
                            app.wlctx.layer_shell = Some((layer_shell, name));
                        }
                    }
                    // Screencopy manager 绑定
                    _ if interface_name == ZwlrScreencopyManagerV1::interface().name => {
                        if app.wlctx.scm.manager.is_none() {
                            let manager: ZwlrScreencopyManagerV1 =
                                proxy.bind(name, version, qh, ());
                            // app.wayland_ctx.screencopy_manager = Some((manager.clone(), name));
                            app.wlctx.scm = ZwlrScreencopyMode::new((manager, name));
                        }
                    }
                    // Cursor shape 相关绑定
                    _ if interface_name == WpCursorShapeManagerV1::interface().name => {
                        if app.wlctx.pointer_helper.cursor_shape_manager.is_none() {
                            let manager: WpCursorShapeManagerV1 = proxy.bind(name, version, qh, ());
                            app.wlctx.pointer_helper.cursor_shape_manager = Some((manager, name));
                        }
                    }
                    // NOTE: xdg_output_manager 处理多输出
                    _ if interface_name
                        == zxdg_output_manager_v1::ZxdgOutputManagerV1::interface().name =>
                    {
                        if app.wlctx.xdg_output_manager.is_none() {
                            let manager = proxy.bind(name, version, qh, ());
                            app.wlctx.xdg_output_manager = Some((manager, name));
                        }
                    }
                    // xdgwmbase
                    // _ if interface_name == xdg_wm_base::XdgWmBase::interface().name => {
                    //     if app.wlctx.xdgwmbase.is_none() {
                    //         let base = proxy.bind(name, version, qh, ());
                    //         app.wlctx.xdgwmbase = Some((base, name));
                    //     }
                    // }
                    // Viewporter
                    _ if interface_name == WpViewporter::interface().name => {
                        if app.wlctx.viewporter.is_none() {
                            let viewporter = proxy.bind(name, version, qh, ());
                            app.wlctx.viewporter = Some((viewporter, name));
                        }
                    }
                    // Fractional scale
                    _ if interface_name == WpFractionalScaleManagerV1::interface().name => {
                        if app.wlctx.fractional_manager.is_none() {
                            let fractional = proxy.bind(name, version, qh, ());
                            app.wlctx.fractional_manager = Some((fractional, name));
                        }
                    }
                    _ => (),
                }
            }
            wl_registry::Event::GlobalRemove { name } => {
                if let Some((_, compositor_name)) = &app.wlctx.compositor {
                    if name == *compositor_name {
                        warn!("WlCompositor was removed");
                        app.wlctx.compositor = None;
                    } else if let Some((_, sate_name)) = &app.wlctx.seat {
                        if name == *sate_name {
                            warn!("WlSeat was removed");
                            app.wlctx.seat = None;
                        }
                    } else if let Some((_, screencopymanager_name)) = &app.wlctx.scm.manager {
                        if name == *screencopymanager_name {
                            warn!("ZwlrScreencopyManagerV1 was removed");
                            app.wlctx.scm.manager = None;
                        }
                    } else if let Some((_, layer_shell_name)) = &app.wlctx.layer_shell {
                        if name == *layer_shell_name {
                            warn!("ZwlrLayerShellV1 was removed");
                            app.wlctx.layer_shell = None;
                        }
                    } else if let Some((_, cursor_shape_manager_name)) =
                        &app.wlctx.pointer_helper.cursor_shape_manager
                    {
                        if name == *cursor_shape_manager_name {
                            warn!("WpCursorShapeManagerV1 was removed");
                            app.wlctx.pointer_helper.cursor_shape_manager = None;
                        }
                    } else if let Some((_, viewporter_name)) = &app.wlctx.viewporter {
                        if name == *viewporter_name {
                            warn!("WpViewporter was removed");
                            app.wlctx.viewporter = None;
                        }
                    } else if let Some((_, fractional_manager_name)) = &app.wlctx.fractional_manager
                    {
                        if name == *fractional_manager_name {
                            warn!("WpFractionalScaleManagerV1 was removed");
                            app.wlctx.fractional_manager = None;
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
                // NOTE: WE DO NOT CHANGE SCALE HERE SINCE IT WILL HAPPEN BEFORE WE INIT LAYER
                // GO CHECK ON WlSurface INSTEAD
            }
            wl_output::Event::Mode {
                flags: _,
                width,
                height,
                refresh: _,
            } => {
                let mut foam_output = app.wlctx.foam_outputs.as_mut().unwrap().get_mut(*data);
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

                let Some((xdg_output_manager, _)) = &app.wlctx.xdg_output_manager else {
                    error!("No ZxdgOutputManagerV1 loaded");
                    return;
                };
                // create an xdg_output object for this wl_output
                let _ = xdg_output_manager.get_xdg_output(proxy, qh, *data);

                let Some((compositor, _)) = &app.wlctx.compositor else {
                    error!("No Compositor");
                    return;
                };
                // TODO: create surface
                trace!("create surface");
                let foam_output = app
                    .wlctx
                    .foam_outputs
                    .as_mut()
                    .unwrap()
                    .get_mut(*data)
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
        match event {
            wl_surface::Event::PreferredBufferTransform { transform } => {
                if let Ok(t) = transform.into_result() {
                    proxy.set_buffer_transform(t);
                }
            }
            wl_surface::Event::PreferredBufferScale { factor } => {
                // let mut foam_output = app.wlctx.foam_outputs.as_mut().unwrap().get_mut(*data);
                // foam_output
                //     .as_mut()
                //     .unwrap()
                //     .surface
                //     .as_mut()
                //     .unwrap()
                //     .set_buffer_scale(factor);
                // foam_output
                //     .as_mut()
                //     .unwrap()
                //     .scale
                //     .as_mut()
                //     .unwrap()
                //     .update_normal(factor as u32);
                // NOTE: WE DO NOT CHANGE BUFFER SCALE, USE ViewPorter INSTEAD.
                // proxy.set_buffer_scale(factor);
            }
            _ => {}
        }
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
                Action::OnDraw | Action::OnEdit(_) => {
                    app.wlctx.update_select_region();
                }
                _ => {}
            }
        }
    }
}

impl ShmHandler for FoamShot {
    fn shm_state(&mut self) -> &mut smithay_client_toolkit::shm::Shm {
        self.wlctx.shm.as_mut().unwrap()
    }
}

delegate_shm!(FoamShot);
