use smithay_client_toolkit::{delegate_shm, shm::ShmHandler};
use wayland_client::{
    Dispatch, Proxy,
    globals::GlobalListContents,
    protocol::{wl_compositor, wl_keyboard, wl_output, wl_pointer, wl_registry, wl_seat, wl_surface},
};
use wayland_protocols_wlr::{
    layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1},
    screencopy::v1::client::{zwlr_screencopy_frame_v1, zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1},
};

use crate::{action::Action, shot_fome::ShotFome};

impl Dispatch<wl_registry::WlRegistry, ()> for ShotFome {
    fn event(
        state: &mut Self,
        proxy: &wl_registry::WlRegistry,
        event: <wl_registry::WlRegistry as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            wl_registry::Event::Global { name, interface, version } => {
                if interface == wl_compositor::WlCompositor::interface().name && state.compositor.is_none() {
                    state.compositor = Some(proxy.bind(name, version, qh, ()))
                } else if interface == wl_seat::WlSeat::interface().name && state.seat.is_none() {
                    let seat: wl_seat::WlSeat = proxy.bind(name, version, qh, ());
                    state.pointer = Some(seat.get_pointer(qh, ()));
                    state.keyboard = Some(seat.get_keyboard(qh, ()));
                    state.seat = Some(seat);
                } else if interface == wl_output::WlOutput::interface().name && state.output.is_none() {
                    let output = proxy.bind(name, version, qh, ());
                    state.output = Some(output);
                } else if interface == zwlr_layer_shell_v1::ZwlrLayerShellV1::interface().name && state.layer_shell.is_none() {
                    state.layer_shell = Some(proxy.bind(name, version, qh, ()));
                } else if interface == ZwlrScreencopyManagerV1::interface().name && state.freeze_mode.screencopy_manager.is_none() {
                    state.freeze_mode.screencopy_manager = Some(proxy.bind(name, version, qh, ()));
                }
            }
            wl_registry::Event::GlobalRemove { .. } => {
                if let Some(_) = &state.compositor {
                    state.compositor = None;
                } else if let Some(_seat_name) = &state.seat {
                    state.seat = None;
                } else if let Some(_shm_name) = &state.shm {
                    state.shm = None;
                } else if let Some(_) = &state.layer_shell {
                    state.layer_shell = None;
                }
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_output::WlOutput, ()> for ShotFome {
    fn event(
        state: &mut Self,
        _proxy: &wl_output::WlOutput,
        event: <wl_output::WlOutput as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            // 处理输出设备的模式事件
            wl_output::Event::Mode {
                flags: _,
                width,
                height,
                refresh: _,
            } => {
                state.phys_height = Some(height);
                state.phys_width = Some(width);
            }
            // 处理输出设备的几何事件
            wl_output::Event::Geometry {
                x: _,
                y: _,
                physical_width: _,
                physical_height: _,
                subpixel: _,
                make: _,
                model: _,
                transform: _,
            } => {
                // 为此输出设备创建一个表面并存储它
                if let Some(compositor) = &state.compositor {
                    state.freeze_mode.surface = Some(compositor.create_surface(qh, 1));
                    state.select_mode.surface = Some(compositor.create_surface(qh, 2));
                }
            }
            _ => {}
        };
    }
}

//NOTE: ok
impl Dispatch<wl_pointer::WlPointer, ()> for ShotFome {
    fn event(
        state: &mut Self,
        _proxy: &wl_pointer::WlPointer,
        event: <wl_pointer::WlPointer as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            wl_pointer::Event::Enter { surface, .. } => {
                if surface == *state.freeze_mode.surface.as_ref().unwrap() {
                    println!("鼠标进入表面1");
                    // state.prev_select();
                } else if surface == *state.select_mode.surface.as_ref().unwrap() {
                    println!("鼠标进入表面2");
                }
            }
            // TEST:
            wl_pointer::Event::Button { state: button_state, .. } => {
                if let Some((current_x, current_y)) = state.current_pos {
                    if button_state == wayland_client::WEnum::Value(wl_pointer::ButtonState::Released) {
                        println!("右键松开");
                        state.pointer_end = Some((current_x, current_y));
                        state.action = Action::AfterSelect;
                    } else if button_state == wayland_client::WEnum::Value(wl_pointer::ButtonState::Pressed) {
                        println!("右键按下");

                        state.pointer_start = Some((current_x, current_y));
                        state.action = Action::Onselect;
                    }
                }
            }
            wl_pointer::Event::Motion { surface_x, surface_y, .. } => {
                // println!("鼠标坐标: x: {:.2}, y: {:.2}", surface_x, surface_y);
                // 保存当前鼠标位置
                // TEST:
                // state.action = Some(crate::app::Action::FREEZE);

                state.current_pos = Some((surface_x.max(0.0), surface_y.max(0.0)));
            }
            _ => {}
        }
    }
}

//NOTE: ok
impl Dispatch<wl_keyboard::WlKeyboard, ()> for ShotFome {
    fn event(
        state: &mut Self,
        _proxy: &wl_keyboard::WlKeyboard,
        event: <wl_keyboard::WlKeyboard as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            wl_keyboard::Event::Key {
                serial: _,
                time: _,
                key,
                state: key_state,
            } => match key_state {
                wayland_client::WEnum::Value(wl_keyboard::KeyState::Pressed) => {
                    if key == 1 {
                        state.action = Action::EXIT;
                        println!("ESC key pressed. Exiting...");
                    } else {
                        return;
                    }
                }
                _ => return,
            },
            _ => {}
        }
    }
}

// NOTE: configure event
impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, i32> for ShotFome {
    fn event(
        _state: &mut Self,
        proxy: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        event: <zwlr_layer_surface_v1::ZwlrLayerSurfaceV1 as Proxy>::Event,
        data: &i32,
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            zwlr_layer_surface_v1::Event::Configure { serial, width: _, height: _ } => {
                // acknowledge the Configure event
                println!("data:{}", data);
                proxy.ack_configure(serial);
            }
            zwlr_layer_surface_v1::Event::Closed => {
                proxy.destroy();
            }
            _ => (),
        }
    }
}

// NOTE: copy a frame
impl Dispatch<zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1, ()> for ShotFome {
    fn event(
        state: &mut Self,
        proxy: &zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1,
        event: <zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1 as Proxy>::Event,
        _data: &(),
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            zwlr_screencopy_frame_v1::Event::Buffer { format, width, height, stride } => {
                let (buffer, _canvas) = state
                    .pool
                    .as_mut()
                    .unwrap()
                    .create_buffer(width as i32, height as i32, stride as i32, format.into_result().unwrap())
                    .map_err(|e| format!("Failed to create buffer: {}", e))
                    .unwrap();
                state.freeze_mode.buffer = Some(buffer);
            }
            zwlr_screencopy_frame_v1::Event::BufferDone { .. } => {
                // all buffer types are reported, proceed to send copy request
                // after copy -> wait for Event::Ready
                let Some(buffer) = &state.freeze_mode.buffer else {
                    return;
                };
                // copy frame to buffer, sends Ready when successful
                proxy.copy(&buffer.wl_buffer());
            }
            // NOTE: screen is freeze now
            zwlr_screencopy_frame_v1::Event::Ready { .. } => {
                state.action = Action::FREEZE;
                let surface = state.freeze_mode.surface.as_ref().unwrap();
                surface.commit(); // 在附加任何缓冲区之前提交

                println!("将缓冲区附加到表面");
                state.freeze_mode.buffer.as_mut().unwrap().attach_to(&surface).unwrap();
                surface.damage(0, 0, state.phys_width.unwrap(), state.phys_height.unwrap());
                println!("提交表面");
                surface.set_buffer_scale(1);
                surface.commit();
                // TODO:
                state.select_mode.before_select_handle(state.phys_width, state.phys_height, state.pool.as_mut().unwrap());
            }
            zwlr_screencopy_frame_v1::Event::Failed => {
                state.action = Action::EXIT;
                // state.exit = true;
            }
            _ => (),
        }
    }
}

//NOTE: 空实现
impl Dispatch<wl_surface::WlSurface, i32> for ShotFome {
    fn event(
        _state: &mut Self,
        _proxy: &wl_surface::WlSurface,
        _event: <wl_surface::WlSurface as wayland_client::Proxy>::Event,
        _data: &i32,
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
    ) {
        // todo!()
    }
}
//NOTE: 空实现
impl Dispatch<wl_seat::WlSeat, ()> for ShotFome {
    fn event(
        _state: &mut Self,
        _proxy: &wl_seat::WlSeat,
        _event: <wl_seat::WlSeat as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
    ) {
        // todo!()
    }
}
//NOTE: 空实现
impl Dispatch<wl_compositor::WlCompositor, ()> for ShotFome {
    fn event(
        _state: &mut Self,
        _proxy: &wl_compositor::WlCompositor,
        _event: <wl_compositor::WlCompositor as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
    ) {
        // todo!()
    }
}
// NOTE: unimplemented
impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for ShotFome {
    fn event(
        _state: &mut Self,
        _proxy: &wl_registry::WlRegistry,
        _event: <wl_registry::WlRegistry as wayland_client::Proxy>::Event,
        _data: &GlobalListContents,
        _conn: &wayland_client::Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        // todo!()
    }
}

//NOTE: 空实现
impl Dispatch<zwlr_layer_shell_v1::ZwlrLayerShellV1, ()> for ShotFome {
    fn event(
        _state: &mut Self,
        _proxy: &zwlr_layer_shell_v1::ZwlrLayerShellV1,
        _event: <zwlr_layer_shell_v1::ZwlrLayerShellV1 as Proxy>::Event,
        _data: &(),
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
    ) {
        // todo!()
    }
}

// NOTE: 空实现
impl Dispatch<ZwlrScreencopyManagerV1, ()> for ShotFome {
    fn event(
        _state: &mut Self,
        _proxy: &ZwlrScreencopyManagerV1,
        _event: <ZwlrScreencopyManagerV1 as Proxy>::Event,
        _data: &(),
        _conn: &wayland_client::Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        // todo!()
    }
}

impl ShmHandler for ShotFome {
    fn shm_state(&mut self) -> &mut smithay_client_toolkit::shm::Shm {
        self.shm.as_mut().unwrap()
    }
}
delegate_shm!(ShotFome);
