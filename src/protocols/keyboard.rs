use log::debug;
use wayland_client::protocol::wl_keyboard;
use wayland_client::{Dispatch, Proxy};

use crate::action::{Action, IsFreeze};
use crate::foamcore::FoamShot;

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
                    let current_output = app.wlctx.current_index;
                    app.wlctx.set_one_max(current_output.unwrap());
                    app.action = Action::Output
                }
                KEY_S => match app.action {
                    Action::WaitPointerPress => {}
                    Action::Init => {}
                    Action::Exit => {}
                    _ => app.action = Action::Output,
                },
                KEY_F => {
                    app.wlctx.current_freeze = !app.wlctx.current_freeze;
                    app.action = if app.wlctx.current_freeze {
                        Action::ToggleFreeze(IsFreeze::NewFrameFreeze)
                    } else {
                        Action::ToggleFreeze(IsFreeze::UnFreeze)
                    };
                }
                KEY_ESC => match app.action {
                    Action::OnEdit(a) => {
                        app.action = if app.wlctx.current_freeze {
                            Action::ToggleFreeze(IsFreeze::OldFrameFreeze)
                        } else {
                            Action::ToggleFreeze(IsFreeze::UnFreeze)
                        };
                    }
                    Action::OnRecorder => {
                        debug!("recorder size: {}", app.wlctx.fq.f.len());
                        std::process::exit(0);
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
