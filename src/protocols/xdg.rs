//! INFO: zxdg_output_manager_v1 && zxdg_output_v1 interface implementation
//! Through these, the physical layout of the display is obtained and applied in a multi-display environment
use log::info;
use wayland_client::{Dispatch, Proxy};
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};
use wayland_protocols::xdg::xdg_output::zv1::client::{zxdg_output_manager_v1, zxdg_output_v1};

use crate::foamcore::FoamShot;

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
impl Dispatch<zxdg_output_v1::ZxdgOutputV1, usize> for FoamShot {
    fn event(
        app: &mut Self,
        proxy: &zxdg_output_v1::ZxdgOutputV1,
        event: <zxdg_output_v1::ZxdgOutputV1 as Proxy>::Event,
        data: &usize,
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        let foam_output = app
            .wlctx
            .foam_outputs
            .as_mut()
            .unwrap()
            .get_mut(*data)
            .unwrap();

        match event {
            zxdg_output_v1::Event::LogicalPosition { x, y } => {
                info!("LogicalPosition: ({}, {})", x, y);
                foam_output.global_x = x;
                foam_output.global_y = y;
            }
            zxdg_output_v1::Event::LogicalSize { width, height } => {
                info!("LogicalSize: {}x{}", width, height);
                foam_output.logical_width = width;
                foam_output.logical_height = height;
            }
            zxdg_output_v1::Event::Description { description } => {
                info!("Description: {}", description);
                // monitor.description = description;
            }
            zxdg_output_v1::Event::Name { name } => {
                info!("Name: {}", name);
                foam_output.name = name;
            }
            _ => (),
        }

        // 当所有必要属性收集完成后
        // if monitor.is_complete() {
        //     info!("Monitor {} complete", monitor_id);
        // }
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
        if let xdg_wm_base::Event::Ping { serial } = event {
            proxy.pong(serial);
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
        if let xdg_surface::Event::Configure { serial } = event {
            proxy.ack_configure(serial);
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
            } => {}
            _ => (),
        }
        // todo!()
    }
}
