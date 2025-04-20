use log::{debug, error};
use wayland_client::protocol::wl_pointer;
use wayland_client::{Dispatch, Proxy};
use wayland_protocols::wp::cursor_shape::v1::client::wp_cursor_shape_device_v1::Shape;

use crate::action::{Action, EditAction};
use crate::foamcore::{FoamShot, UserTarget};
use crate::monitors;

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
                let _ = app.wlctx.set_cursor_shape(Shape::Default, proxy);
            }

            wl_pointer::Event::Enter {
                serial,
                surface,
                surface_x,
                surface_y,
            } => {
                app.wlctx.pointer_helper.serial = serial;
                let surface_index = match surface.data::<usize>() {
                    Some(idx) => *idx,
                    None => {
                        error!("can not get surface index, exit!");
                        std::process::exit(0)
                    }
                };
                app.wlctx.unknown_index = Some(surface_index);

                // set cursor shape
                if app.wlctx.set_cursor_shape(Shape::Crosshair, proxy).is_err() {
                    app.send_warn("can not set cursor shape");
                }

                let foam_output = match app
                    .wlctx
                    .foam_outputs
                    .as_ref()
                    .and_then(|outputs| outputs.get(surface_index))
                {
                    Some(output) => output,
                    None => {
                        error!("can not get foam_output, exit!");
                        std::process::exit(0)
                    }
                };

                // 转换成相对surface的坐标
                // let x = surface_x + foam_output.global_x as f64;
                // let y = surface_y + foam_output.global_y as f64;
                let (x, y) = foam_output.scale.as_ref().unwrap().calculate_pos((
                    surface_x + foam_output.global_x as f64,
                    surface_y + foam_output.global_y as f64,
                ));

                // 发送多个enter时候，只选择满足坐标约束的
                if x >= 0.0
                    && y >= 0.0
                    && x <= foam_output.width as f64
                    && y <= foam_output.height as f64
                {
                    debug!("surface enter output:{} x:{}, y:{}", foam_output.name, x, y);

                    // NOTE:  full screen mode handle
                    if app.wlctx.config.full_screen {
                        app.wlctx.set_one_max(surface_index);
                        app.action = Action::Output;
                        return;
                    }

                    app.wlctx.current_index = Some(surface_index);
                    app.wlctx
                        .pointer_helper
                        .start_index
                        .get_or_insert(surface_index);

                    // 鼠标未移动时进行初始化
                    if app.wlctx.pointer_helper.g_current_pos.is_none() {
                        let global_pos = (
                            x + foam_output.global_x as f64,
                            y + foam_output.global_y as f64,
                        );
                        app.wlctx.pointer_helper.g_current_pos = Some(global_pos);
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
                        wl_pointer::ButtonState::Pressed => match app.action {
                            Action::WaitPointerPress => {
                                app.wlctx.pointer_helper.start_index = app.wlctx.current_index;
                                app.wlctx.pointer_helper.g_start_pos =
                                    app.wlctx.pointer_helper.g_current_pos;
                                app.wlctx.generate_rects_and_send_frame();
                                app.action = Action::OnDraw;
                            }
                            Action::OnEdit(EditAction::None) => {
                                app.wlctx.pointer_helper.g_start_pos =
                                    app.wlctx.pointer_helper.g_current_pos;

                                if let Some(current_pos) = app.wlctx.pointer_helper.g_current_pos {
                                    if let Some(global_rect) = app.wlctx.global_rect.as_ref() {
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
                        },
                        wl_pointer::ButtonState::Released => {
                            if app.action == Action::OnDraw {
                                app.wlctx.pointer_helper.end_index = app.wlctx.current_index;
                                app.wlctx.pointer_helper.g_end_pos =
                                    app.wlctx.pointer_helper.g_current_pos;
                            }

                            app.action = if app.wlctx.config.edit {
                                Action::OnEdit(EditAction::None)
                            } else {
                                match app.target {
                                    UserTarget::Shot => Action::Output,
                                    UserTarget::Recorder => Action::OnRecorder,
                                }
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
                    app.wlctx.unknown_index,
                    app.wlctx.pointer_helper.start_index,
                ) {
                    (Some(u), Some(s)) => (u, s),
                    _ => {
                        error!("can not get surface index, exit!");
                        std::process::exit(0)
                    }
                };

                let outputs = match app.wlctx.foam_outputs.as_ref() {
                    Some(o) => o,
                    None => {
                        error!("can not get foam_outputs, exit!");
                        std::process::exit(0)
                    }
                };

                let (start_output, unknown_output) =
                    match (outputs.get(start_index), outputs.get(unknown_index)) {
                        (Some(s), Some(u)) => (s, u),
                        _ => return,
                    };

                let (x, y) = monitors::FoamMonitors::convert_pos_to_surface(
                    unknown_output,
                    start_output,
                    surface_x,
                    surface_y,
                );

                let global_pos = (
                    x + start_output.global_x as f64,
                    y + start_output.global_y as f64,
                );

                // TEST: 先凑合着
                let global_pos = start_output
                    .scale
                    .as_ref()
                    .unwrap()
                    .calculate_pos(global_pos);

                app.wlctx.pointer_helper.g_current_pos = Some(global_pos);

                match app.action {
                    Action::OnDraw => {
                        app.wlctx.generate_rects_and_send_frame();
                    }
                    Action::OnEdit(edit_action) => {
                        if let Some(global_rect) = app.wlctx.global_rect.as_ref() {
                            let hit_region_act = global_rect.hit_region(
                                global_pos.0 as i32,
                                global_pos.1 as i32,
                                15,
                            );
                            let _ = app
                                .wlctx
                                .set_cursor_shape(hit_region_act.to_cursor_shape(), proxy);
                        }

                        match edit_action {
                            EditAction::None => {}
                            _ => {
                                if let (Some(start_pos), Some(global_rect)) = (
                                    app.wlctx.pointer_helper.g_start_pos,
                                    app.wlctx.global_rect.as_mut(),
                                ) {
                                    app.action =
                                        global_rect.edit(start_pos, global_pos, app.action);
                                    app.wlctx.process_subrects_and_send();
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => (),
        }
    }
}
