// use log::debug;
// use smithay_client_toolkit::{delegate_shm, shm::ShmHandler};
// use wayland_client::{
//     Dispatch, Proxy,
//     protocol::{wl_compositor, wl_keyboard, wl_output, wl_pointer, wl_registry, wl_seat},
// };
// use wayland_protocols::{
//     wp::cursor_shape::v1::client::wp_cursor_shape_manager_v1, xdg::shell::client::xdg_wm_base,
// };
// use wayland_protocols_wlr::{
//     layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1},
//     screencopy::v1::client::{
//         zwlr_screencopy_frame_v1::{self, ZwlrScreencopyFrameV1},
//         zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1,
//     },
// };
//
// use crate::mode::{self, Mode};
// use crate::{foam_shot::FoamShot, mode::CopyHook};
// impl Dispatch<wl_registry::WlRegistry, ()> for FoamShot {
//     fn event(
//         state: &mut Self,
//         proxy: &wl_registry::WlRegistry,
//         event: <wl_registry::WlRegistry as wayland_client::Proxy>::Event,
//         _data: &(),
//         _conn: &wayland_client::Connection,
//         qh: &wayland_client::QueueHandle<Self>,
//     ) {
//         match event {
//             wl_registry::Event::Global {
//                 name,
//                 interface,
//                 version,
//             } => {
//                 if interface == wl_compositor::WlCompositor::interface().name
//                     && state.wayland_ctx.compositor.is_none()
//                 {
//                     state.wayland_ctx.compositor = Some(proxy.bind(name, version, qh, ()))
//                 } else if interface == wl_seat::WlSeat::interface().name
//                     && state.wayland_ctx.seat.is_none()
//                 {
//                     let seat: wl_seat::WlSeat = proxy.bind(name, version, qh, ());
//                     state.wayland_ctx.pointer = Some(seat.get_pointer(qh, ()));
//                     state.wayland_ctx.keyboard = Some(seat.get_keyboard(qh, ()));
//                     state.wayland_ctx.seat = Some(seat);
//                 } else if interface == wl_output::WlOutput::interface().name
//                     && state.wayland_ctx.output.is_none()
//                 {
//                     let output = proxy.bind(name, version, qh, ());
//                     state.wayland_ctx.output = Some(output);
//                 } else if interface == zwlr_layer_shell_v1::ZwlrLayerShellV1::interface().name
//                     && state.wayland_ctx.layer_shell.is_none()
//                 {
//                     state.wayland_ctx.layer_shell = Some(proxy.bind(name, version, qh, ()));
//                 } else if interface == ZwlrScreencopyManagerV1::interface().name
//                     && state.wayland_ctx.screencopy_manager.is_none()
//                 {
//                     state.wayland_ctx.screencopy_manager = Some(proxy.bind(name, version, qh, ()));
//                     // state.result_output.screencopy_manager =
//                     // Some(proxy.bind(name, version, qh, ()));
//                 } else if interface
//                     == wp_cursor_shape_manager_v1::WpCursorShapeManagerV1::interface().name
//                     && state.wayland_ctx.cursor_shape_manager.is_none()
//                 {
//                     state.wayland_ctx.cursor_shape_manager =
//                         Some(proxy.bind(name, version, qh, ()));
//                     state.wayland_ctx.cursor_shape_device = Some(
//                         state
//                             .wayland_ctx
//                             .cursor_shape_manager
//                             .clone()
//                             .unwrap()
//                             .get_pointer(state.wayland_ctx.pointer.as_ref().unwrap(), qh, ()),
//                     )
//                 } else if interface == xdg_wm_base::XdgWmBase::interface().name
//                     && state.wayland_ctx.xdg_shell.is_none()
//                 {
//                     state.wayland_ctx.xdg_shell = Some(proxy.bind(name, version, qh, ()));
//                 }
//             }
//             wl_registry::Event::GlobalRemove { .. } => {
//                 if state.wayland_ctx.compositor.is_some() {
//                     state.wayland_ctx.compositor = None;
//                 } else if let Some(_seat_name) = &state.wayland_ctx.seat {
//                     state.wayland_ctx.seat = None;
//                 } else if let Some(_shm_name) = &state.wayland_ctx.shm {
//                     state.wayland_ctx.shm = None;
//                 } else if state.wayland_ctx.layer_shell.is_some() {
//                     state.wayland_ctx.layer_shell = None;
//                 }
//             }
//             _ => {}
//         }
//     }
// }
//
// #[allow(unused_variables)]
// impl Dispatch<wl_output::WlOutput, ()> for FoamShot {
//     fn event(
//         state: &mut Self,
//         proxy: &wl_output::WlOutput,
//         event: <wl_output::WlOutput as Proxy>::Event,
//         data: &(),
//         conn: &wayland_client::Connection,
//         qh: &wayland_client::QueueHandle<Self>,
//     ) {
//         match event {
//             // 处理输出设备的模式事件
//             wl_output::Event::Mode {
//                 flags: _,
//                 width,
//                 height,
//                 refresh: _,
//             } => {
//                 state.wayland_ctx.height = Some(height);
//                 state.wayland_ctx.width = Some(width);
//             }
//             // 处理输出设备的几何事件
//             wl_output::Event::Geometry {
//                 x: _,
//                 y: _,
//                 physical_width: _,
//                 physical_height: _,
//                 subpixel: _,
//                 make: _,
//                 model: _,
//                 transform: _,
//             } => {
//                 // 为此输出设备创建一个表面并存储它
//                 if let Some(compositor) = &state.wayland_ctx.compositor {
//                     // TODO:
//                     // state.freeze_mode.surface = Some(compositor.create_surface(qh, 1));
//                     // state.select_mode.surface = Some(compositor.create_surface(qh, 2));
//                 }
//             }
//             _ => {}
//         };
//     }
// }
//
// #[allow(unused_variables)]
// impl Dispatch<wl_pointer::WlPointer, ()> for FoamShot {
//     fn event(
//         state: &mut Self,
//         proxy: &wl_pointer::WlPointer,
//         event: <wl_pointer::WlPointer as Proxy>::Event,
//         data: &(),
//         conn: &wayland_client::Connection,
//         qh: &wayland_client::QueueHandle<Self>,
//     ) {
//         match event {
//             wl_pointer::Event::Enter { surface, .. } => {
//                 if surface == *state.select_mode.surface.as_ref().unwrap() {
//                     debug!("鼠标进入表面2");
//                 }
//             }
//             wl_pointer::Event::Button {
//                 serial,
//                 time,
//                 button,
//                 state: button_state,
//             } => {
//                 if let Some((x, y)) = state.wayland_ctx.current_pos {
//                     if button_state
//                         == wayland_client::WEnum::Value(wl_pointer::ButtonState::Pressed)
//                     {
//                         if let Mode::Await = state.mode {
//                             state.wayland_ctx.start_pos = Some((x, y));
//                             state.mode = Mode::OnDraw;
//                         }
//                     } else if button_state
//                         == wayland_client::WEnum::Value(wl_pointer::ButtonState::Released)
//                     {
//                         if let Mode::OnDraw = state.mode {
//                             state.wayland_ctx.end_pos = Some((x, y));
//                             if state.cli.quickshot {
//                                 state.mode = Mode::Output
//                             } else {
//                                 state.mode = Mode::ShowResult;
//                             }
//                         }
//                     }
//                 } else {
//                     // FIX:
//                     debug!("no pos");
//                 }
//             }
//             wl_pointer::Event::Motion {
//                 surface_x,
//                 surface_y,
//                 time,
//             } => {
//                 state.wayland_ctx.current_pos = Some((surface_x.max(0.0), surface_y.max(0.0)));
//                 if state.wayland_ctx.start_pos.is_none() {
//                     state.wayland_ctx.start_pos = Some((surface_x.max(0.0), surface_y.max(0.0)));
//                 }
//             }
//             _ => {}
//         }
//     }
// }
//
// #[allow(unused_variables)]
// impl Dispatch<wl_keyboard::WlKeyboard, ()> for FoamShot {
//     fn event(
//         state: &mut Self,
//         proxy: &wl_keyboard::WlKeyboard,
//         event: <wl_keyboard::WlKeyboard as Proxy>::Event,
//         data: &(),
//         conn: &wayland_client::Connection,
//         qh: &wayland_client::QueueHandle<Self>,
//     ) {
//         if let wl_keyboard::Event::Key {
//             serial: _,
//             time: _,
//             key,
//             state: key_state,
//         } = event
//         {
//             if let wayland_client::WEnum::Value(wl_keyboard::KeyState::Pressed) = key_state {
//                 println!("{}", key);
//                 match state.mode {
//                     Mode::ShowResult => {
//                         if key == 1 {
//                             state.mode = Mode::PreSelect;
//                         }
//                     }
//                     Mode::Await => {
//                         if key == 30 {
//                             println!("full screen");
//                             state.result_mode.full_screen = true;
//                             state.mode = Mode::Output;
//                         }
//                         if key == 1 {
//                             state.mode = Mode::Exit;
//                         }
//                     }
//                     _ => {}
//                 }
//             }
//         }
//     }
// }
//
// #[allow(unused_variables)]
// impl Dispatch<ZwlrScreencopyFrameV1, ()> for FoamShot {
//     fn event(
//         state: &mut Self,
//         proxy: &ZwlrScreencopyFrameV1,
//         event: <ZwlrScreencopyFrameV1 as wayland_client::Proxy>::Event,
//         data: &(),
//         conn: &wayland_client::Connection,
//         qh: &wayland_client::QueueHandle<Self>,
//     ) {
//         match event {
//             zwlr_screencopy_frame_v1::Event::Buffer {
//                 format,
//                 width,
//                 height,
//                 stride,
//             } => {
//                 debug!(
//                     "format:{:?}, width:{}, height:{}, stride:{}",
//                     format, width, height, stride
//                 );
//                 let buffer = Some(
//                     state
//                         .wayland_ctx
//                         .create_buffer(
//                             width as i32,
//                             height as i32,
//                             stride as i32,
//                             format.into_result().unwrap(),
//                         )
//                         .ok()
//                         .unwrap()
//                         .0,
//                 );
//                 match state.mode {
//                     mode::Mode::Freeze(mode::CopyHook::Request) => {
//                         state.freeze_mode.buffer = buffer;
//                     }
//                     // Mode::Output(CopyHook::Request) => {
//                     //     state.result_mode.buffer = buffer;
//                     // }
//                     _ => {}
//                 }
//             }
//             zwlr_screencopy_frame_v1::Event::BufferDone { .. } => match state.mode {
//                 Mode::Freeze(CopyHook::Request) => {
//                     proxy.copy(state.freeze_mode.buffer.as_mut().unwrap().wl_buffer());
//
//                     state.result_mode.screencopy_frame = Some(proxy.clone());
//                     state.mode = mode::Mode::Freeze(mode::CopyHook::BufferDone);
//                     debug!("set BeforeFreeze");
//                 }
//                 // Mode::Output(CopyHook::Request) => {
//                 //     proxy.copy(state.result_mode.buffer.as_mut().unwrap().wl_buffer());
//                 //     state.mode = mode::Mode::Output(mode::CopyHook::BufferDone);
//                 //     debug!("set Output");
//                 // }
//                 _ => (),
//             },
//             // NOTE: screen is freeze now
//             zwlr_screencopy_frame_v1::Event::Ready { .. } => {
//                 // state.mode = mode::Mode::BeforeFreeze;
//
//                 match state.mode {
//                     Mode::Freeze(CopyHook::BufferDone) => {
//                         state.mode = mode::Mode::Freeze(mode::CopyHook::Ready);
//                         debug!("set Freeze");
//                     }
//                     // Mode::Output(CopyHook::BufferDone) => {
//                     //     state.mode = mode::Mode::Output(mode::CopyHook::Ready);
//                     //     debug!("set Output");
//                     // }
//                     _ => (),
//                 }
//             }
//             zwlr_screencopy_frame_v1::Event::Failed => {
//                 state.mode = mode::Mode::Exit;
//             }
//             _ => (),
//         }
//     }
// }
//
// impl ShmHandler for FoamShot {
//     fn shm_state(&mut self) -> &mut smithay_client_toolkit::shm::Shm {
//         self.wayland_ctx.shm.as_mut().unwrap()
//     }
// }
// delegate_shm!(FoamShot);
//
// impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, i32> for FoamShot {
//     fn event(
//         _state: &mut Self,
//         proxy: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
//         event: <zwlr_layer_surface_v1::ZwlrLayerSurfaceV1 as Proxy>::Event,
//         _data: &i32,
//         _conn: &wayland_client::Connection,
//         _qh: &wayland_client::QueueHandle<Self>,
//     ) {
//         match event {
//             zwlr_layer_surface_v1::Event::Configure {
//                 serial,
//                 width: _,
//                 height: _,
//             } => {
//                 // acknowledge the Configure event
//                 proxy.ack_configure(serial);
//             }
//             zwlr_layer_surface_v1::Event::Closed => {
//                 proxy.destroy();
//             }
//             _ => (),
//         }
//     }
// }
// #[allow(unused_variables)]
// impl Dispatch<xdg_wm_base::XdgWmBase, ()> for FoamShot {
//     fn event(
//         state: &mut Self,
//         proxy: &xdg_wm_base::XdgWmBase,
//         event: <xdg_wm_base::XdgWmBase as Proxy>::Event,
//         data: &(),
//         conn: &wayland_client::Connection,
//         qhandle: &wayland_client::QueueHandle<Self>,
//     ) {
//         match event {
//             xdg_wm_base::Event::Ping { serial } => {
//                 state.wayland_ctx.xdg_shell.as_mut().unwrap().pong(serial);
//             }
//             _ => (),
//         }
//     }
// }
