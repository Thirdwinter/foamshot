use log::{debug, info};
use smithay_client_toolkit::{delegate_shm, shm::ShmHandler};
use wayland_client::{
    Connection, Dispatch, Proxy, QueueHandle,
    globals::GlobalListContents,
    protocol::{
        wl_callback, wl_compositor, wl_keyboard, wl_output, wl_pointer,
        wl_registry::{self},
        wl_seat,
        wl_shm::Format,
        wl_surface,
    },
};
use wayland_protocols::xdg::xdg_output::zv1::client::{
    zxdg_output_manager_v1::{self, ZxdgOutputManagerV1},
    zxdg_output_v1,
};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{self, ZwlrLayerShellV1},
    zwlr_layer_surface_v1::{self},
};

use crate::Data;

impl Dispatch<wl_registry::WlRegistry, ()> for Data {
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
                info!("Registry global: {} {} {}", name, interface, version);

                let interface_name = interface.as_str();
                match interface_name {
                    // Compositor 绑定
                    _ if interface_name == wl_compositor::WlCompositor::interface().name => {
                        if app.compositor.is_none() {
                            let compositor = proxy.bind(name, version, qh, ());
                            app.compositor = Some(compositor);
                        }
                    }
                    // 动态管理 outputs
                    _ if interface_name == wl_output::WlOutput::interface().name => {
                        let outputs = app.output.as_mut().unwrap();
                        let index = outputs.len();
                        outputs.insert(index, proxy.bind(name, version, qh, index));
                        debug!("interface name: {}, index: {}", interface_name, index);
                    }

                    // Seat 绑定及相关资源获取
                    _ if interface_name == wl_seat::WlSeat::interface().name => {
                        if app.seat.is_none() {
                            let seat: wl_seat::WlSeat = proxy.bind(name, version, qh, ());
                            let pointer = seat.get_pointer(qh, ());
                            let keyboard = seat.get_keyboard(qh, ());
                            app.pointer = Some(pointer);
                            app.keyboard = Some(keyboard);
                            app.seat = Some(seat);
                        }
                    }
                    // Layer shell 绑定
                    _ if interface_name == ZwlrLayerShellV1::interface().name => {
                        if app.layer_shell.is_none() {
                            let layer_shell = proxy.bind(name, version, qh, ());
                            app.layer_shell = Some(layer_shell);
                        }
                    }

                    _ if interface_name == ZxdgOutputManagerV1::interface().name => {
                        if app.xdgoutputmanager.is_none() {
                            let x = proxy.bind(name, version, qh, ());
                            app.xdgoutputmanager = Some(x);
                        }
                    }
                    _ => (),
                }
            }
            wl_registry::Event::GlobalRemove { name: _ } => {}
            _ => (),
        }
    }
}

#[allow(unused_variables)]
impl Dispatch<wl_output::WlOutput, usize> for Data {
    fn event(
        app: &mut Self,
        proxy: &wl_output::WlOutput,
        event: <wl_output::WlOutput as wayland_client::Proxy>::Event,
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
                app.w.insert(*data, width);
                app.h.insert(*data, height);
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
            }
            _ => {}
        };
    }
}

#[allow(unused_variables)]
impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, usize> for Data {
    fn event(
        app: &mut Self,
        proxy: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        event: <zwlr_layer_surface_v1::ZwlrLayerSurfaceV1 as Proxy>::Event,
        data: &usize,
        _conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            zwlr_layer_surface_v1::Event::Configure {
                serial,
                width,
                height,
            } => {
                if *app.is_configured.get(data).unwrap() {
                    return;
                }
                debug!("layer: {} configured", data);
                proxy.ack_configure(serial);
                app.layer_ready += 1;

                let surface = app.surface.as_mut().unwrap().get_mut(data).unwrap();
                let w = app.w.get(data).unwrap();
                let h = app.h.get(data).unwrap();
                let pool = app.pool.as_mut().unwrap();

                let (buffer, canvas) = pool
                    .create_buffer(*w, *h, *w * 4, Format::Argb8888)
                    .unwrap();
                canvas.fill(0);
                buffer.attach_to(surface).unwrap();
                surface.damage(0, 0, *w, *h);
                surface.frame(qh, *data);
                surface.commit();
                *app.is_configured.get_mut(data).unwrap() = true;
            }
            zwlr_layer_surface_v1::Event::Closed => {
                proxy.destroy();
            }
            _ => {}
        }
    }
}

#[allow(unused_variables)]
impl Dispatch<wl_callback::WlCallback, usize> for Data {
    fn event(
        app: &mut Self,
        proxy: &wl_callback::WlCallback,
        event: <wl_callback::WlCallback as Proxy>::Event,
        data: &usize,
        conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_callback::Event::Done { callback_data } = event {
            let surface = app.surface.as_mut().unwrap().get_mut(data).unwrap();
            let w = app.w.get(data).unwrap();
            let h = app.h.get(data).unwrap();
            let pool = app.pool.as_mut().unwrap();

            let (buffer, canvas) = pool
                .create_buffer(*w, *h, *w * 4, Format::Argb8888)
                .unwrap();
            canvas.fill(100);
            buffer.attach_to(surface).unwrap();
            surface.damage(0, 0, *w, *h);
            surface.commit();
        }
        // todo!()
    }
}

#[allow(unused_variables)]
impl Dispatch<wl_surface::WlSurface, usize> for Data {
    fn event(
        state: &mut Self,
        proxy: &wl_surface::WlSurface,
        event: <wl_surface::WlSurface as wayland_client::Proxy>::Event,
        data: &usize,
        conn: &wayland_client::Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        if let wl_surface::Event::Enter { output } = event {
            let output_idx = match output.data::<usize>() {
                Some(idx) => *idx,
                None => std::process::exit(0),
            };
            debug!("surface {} enter output: {}", data, output_idx);
        }
    }
}

#[allow(unused_variables)]
impl Dispatch<wl_pointer::WlPointer, ()> for Data {
    fn event(
        state: &mut Self,
        proxy: &wl_pointer::WlPointer,
        event: <wl_pointer::WlPointer as Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        if let wl_pointer::Event::Enter {
            serial,
            surface,
            surface_x,
            surface_y,
        } = event
        {
            let surface_index = match surface.data::<usize>() {
                Some(idx) => *idx,
                None => std::process::exit(0),
            };
            debug!(
                "pointer enter surface: {}, surface_x: {}, surface_y: {}",
                surface_index, surface_x, surface_y
            );
        }
    }
}

#[allow(unused_variables)]
impl Dispatch<wl_keyboard::WlKeyboard, ()> for Data {
    fn event(
        state: &mut Self,
        proxy: &wl_keyboard::WlKeyboard,
        event: <wl_keyboard::WlKeyboard as Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        const KEY_ESC: u32 = 1;

        if let wl_keyboard::Event::Key {
            key,
            state: wayland_client::WEnum::Value(wl_keyboard::KeyState::Pressed),
            ..
        } = event
        {
            debug!("Key pressed: {}", key);

            if key == KEY_ESC {
                std::process::exit(0);
            }
        }
    }
}

#[allow(unused_variables)]
impl Dispatch<wl_seat::WlSeat, ()> for Data {
    fn event(
        state: &mut Self,
        proxy: &wl_seat::WlSeat,
        event: <wl_seat::WlSeat as wayland_client::Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

#[allow(unused_variables)]
impl Dispatch<zxdg_output_v1::ZxdgOutputV1, usize> for Data {
    fn event(
        state: &mut Self,
        proxy: &zxdg_output_v1::ZxdgOutputV1,
        event: <zxdg_output_v1::ZxdgOutputV1 as Proxy>::Event,
        data: &usize,
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        if let zxdg_output_v1::Event::Name { name } = event {
            debug!("output: {}, name: {}", data, name);
        }
    }
}

#[allow(unused_variables)]
impl Dispatch<zxdg_output_manager_v1::ZxdgOutputManagerV1, ()> for Data {
    fn event(
        state: &mut Self,
        proxy: &zxdg_output_manager_v1::ZxdgOutputManagerV1,
        event: <zxdg_output_manager_v1::ZxdgOutputManagerV1 as Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
    }
}

#[allow(unused_variables)]
impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for Data {
    fn event(
        state: &mut Self,
        proxy: &wl_registry::WlRegistry,
        event: <wl_registry::WlRegistry as wayland_client::Proxy>::Event,
        data: &GlobalListContents,
        conn: &wayland_client::Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

#[allow(unused_variables)]
impl Dispatch<wl_compositor::WlCompositor, ()> for Data {
    fn event(
        state: &mut Self,
        proxy: &wl_compositor::WlCompositor,
        event: <wl_compositor::WlCompositor as wayland_client::Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

#[allow(unused_variables)]
impl Dispatch<zwlr_layer_shell_v1::ZwlrLayerShellV1, ()> for Data {
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

impl ShmHandler for Data {
    fn shm_state(&mut self) -> &mut smithay_client_toolkit::shm::Shm {
        self.shm.as_mut().unwrap()
    }
}

delegate_shm!(Data);
