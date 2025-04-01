use std::collections::HashMap;

use log::info;
use wayland_client::{Dispatch, Proxy};
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};
use wayland_protocols::xdg::xdg_output::zv1::client::{zxdg_output_manager_v1, zxdg_output_v1};

use crate::foamshot::FoamShot;
use crate::helper::monitor_helper::Monitor;

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
#[allow(unused_variables)]
impl Dispatch<zxdg_output_v1::ZxdgOutputV1, i64> for FoamShot {
    fn event(
        app: &mut Self,
        proxy: &zxdg_output_v1::ZxdgOutputV1,
        event: <zxdg_output_v1::ZxdgOutputV1 as Proxy>::Event,
        data: &i64,
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        // 类型转换确保与HashMap键类型一致
        let monitor_id = *data as usize;

        // 使用Entry API保证原子性操作
        let monitors = app
            .wayland_ctx
            .monitors
            .get_or_insert_with(|| HashMap::new());

        let monitor = monitors.entry(monitor_id).or_insert_with(|| Monitor {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            name: String::new(),
            scale: 1,
        });

        match event {
            zxdg_output_v1::Event::LogicalPosition { x, y } => {
                info!("LogicalPosition: ({}, {})", x, y);
                monitor.x = x;
                monitor.y = y;
            }
            zxdg_output_v1::Event::LogicalSize { width, height } => {
                info!("LogicalSize: {}x{}", width, height);
                monitor.width = width;
                monitor.height = height;
            }
            zxdg_output_v1::Event::Description { description } => {
                info!("Description: {}", description);
                // monitor.description = description;
            }
            zxdg_output_v1::Event::Name { name } => {
                info!("Name: {}", name);
                monitor.name = name;
            }
            _ => (),
        }

        // 当所有必要属性收集完成后
        if monitor.is_complete() {
            info!("Monitor {} complete", monitor_id);
        }
    }
}

#[allow(unused_variables)]
impl Dispatch<xdg_wm_base::XdgWmBase, ()> for FoamShot {
    fn event(
        app: &mut Self,
        proxy: &xdg_wm_base::XdgWmBase,
        event: <xdg_wm_base::XdgWmBase as Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            xdg_wm_base::Event::Ping { serial } => {
                proxy.pong(serial);
            }
            _ => (),
        }
    }
}

#[allow(unused_variables)]
impl Dispatch<xdg_surface::XdgSurface, ()> for FoamShot {
    fn event(
        app: &mut Self,
        proxy: &xdg_surface::XdgSurface,
        event: <xdg_surface::XdgSurface as Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            xdg_surface::Event::Configure { serial } => {
                proxy.ack_configure(serial);
            }
            _ => (),
        }
    }
}

#[allow(unused_variables)]
impl Dispatch<xdg_toplevel::XdgToplevel, ()> for FoamShot {
    fn event(
        app: &mut Self,
        proxy: &xdg_toplevel::XdgToplevel,
        event: <xdg_toplevel::XdgToplevel as Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            xdg_toplevel::Event::Close => std::process::exit(0),
            xdg_toplevel::Event::WmCapabilities { capabilities } => {
                // info!("Capabilities: {:?}", capabilities);
            }
            xdg_toplevel::Event::Configure {
                width,
                height,
                states,
            } => {
                // info!("Configure: {}, {}, states: {:?}", width, height, states);
                // app.editor_mode.window_width = Some(width);
                // app.editor_mode.window_height = Some(height);
                // app.editor_mode.resize(&mut app.wayland_ctx, width, height);
            }
            _ => (),
        }
        // todo!()
    }
}
