use log::debug;
use smithay_client_toolkit::shm::Shm;
use wayland_client::{Connection, EventQueue, globals::registry_queue_init};

use crate::{
    action::{self, Action, IsFreeze},
    config::{FoamConfig, ImageType},
    notify::{self},
    save_helper, wayland_ctx,
};

pub struct FoamShot {
    /// foamshot wayland context
    pub wayland_ctx: wayland_ctx::WaylandCtx,

    pub mode: action::Action,
}

/// run
pub fn run_main_loop() {
    let connection = Connection::connect_to_env().expect("can't connect to wayland display");
    let (globals, mut event_queue) =
        registry_queue_init::<FoamShot>(&connection).expect("failed to get globals");
    let qh = event_queue.handle();
    let display = connection.display();
    let _registry = display.get_registry(&qh, ());

    let shm = Shm::bind(&globals, &qh).expect("wl_shm is not available");
    let mut shot_foam = FoamShot::new(shm, qh);
    debug!("{:?}", shot_foam.wayland_ctx.config);

    event_queue.roundtrip(&mut shot_foam).expect("init failed");

    shot_foam.wait_freeze(&mut event_queue);

    // NOTE: 创建layer && surface提交
    shot_foam.wayland_ctx.init_base_layers();

    // NOTE: 等待处理事件
    event_queue.blocking_dispatch(&mut shot_foam).unwrap();

    loop {
        event_queue.blocking_dispatch(&mut shot_foam).unwrap();
        match &shot_foam.mode {
            Action::Init => {}
            Action::WaitPointerPress => {}
            Action::ToggleFreeze(state) => {
                match state {
                    IsFreeze::Freeze => {
                        debug!("next is freeze");

                        // 进行屏幕copy 通过计数器等待所有ready完成
                        shot_foam.wait_freeze(&mut event_queue);
                        // 发送下一帧，重新附加buffer
                        shot_foam.toggle_freeze(&mut event_queue);
                    }
                    IsFreeze::UnFreeze => {
                        debug!("next is unfreeze");

                        shot_foam.toggle_freeze(&mut event_queue);
                    }
                }
                shot_foam.mode = Action::WaitPointerPress
            }
            Action::OnDraw => {
                shot_foam.wayland_ctx.update_select_region();
            }
            Action::Exit => {
                shot_foam.wayland_ctx.config = FoamConfig::new();
                for (_i, v) in shot_foam.wayland_ctx.foam_outputs.as_ref().unwrap() {
                    debug!("{:?}", v.subrect)
                }
                // shot_foam.wayland_ctx.generate_sub_rects();
                // shot_foam.wayland_ctx.before_output_collect_canvas();
                if !shot_foam.wayland_ctx.current_freeze {
                    shot_foam.wait_freeze(&mut event_queue);
                }
                match shot_foam.wayland_ctx.config.image_type {
                    ImageType::Png => {
                        if let Err(e) = save_helper::save_to_png(&mut shot_foam.wayland_ctx) {
                            notify::send(notify::NotificationLevel::Error, "save to png error");
                            log::error!("save to png error: {}", e);
                        }
                        save_helper::save_to_wl_clipboard(&mut shot_foam.wayland_ctx).unwrap();
                    }
                    ImageType::Jpg => {
                        if let Err(e) = save_helper::save_to_jpg(&mut shot_foam.wayland_ctx, 100) {
                            notify::send(notify::NotificationLevel::Error, "save to jpg error");
                            log::error!("save to jpg error: {}", e);
                        }
                        save_helper::save_to_wl_clipboard(&mut shot_foam.wayland_ctx).ok();
                    }
                }

                std::process::exit(0)
            }
        }
    }
}

impl FoamShot {
    pub fn new(shm: Shm, qh: wayland_client::QueueHandle<FoamShot>) -> FoamShot {
        Self {
            wayland_ctx: wayland_ctx::WaylandCtx::new(shm, qh),
            mode: Action::default(),
        }
    }

    /// 临时借用 event_queue 进行copy
    /// 发起copy请求 -> 等待全部 output 完成 -> 重置计数器 -> 缓存 canvas
    pub fn wait_freeze(&mut self, event_queue: &mut EventQueue<FoamShot>) {
        self.check_ok();

        // NOTE: 先确保屏幕为正常状态
        if self.mode == Action::ToggleFreeze(IsFreeze::UnFreeze)
            || self.mode == Action::ToggleFreeze(IsFreeze::Freeze)
        {
            self.wayland_ctx.unset_freeze();
        }

        // NOTE: 请求全屏copy，之后该去protocols::zwlr_screencopy_manager_v1中依次处理event
        self.wayland_ctx.request_screencopy();

        // 等待所有屏幕copy完成
        while self.wayland_ctx.scm.copy_ready
            != self.wayland_ctx.foam_outputs.as_ref().unwrap().len()
        {
            event_queue.blocking_dispatch(self).unwrap();
        }
        // 重置计数器
        self.wayland_ctx.scm.copy_ready = 0;
        // 存储 copy 到的数据
        self.wayland_ctx.store_copy_canvas();
    }

    pub fn toggle_freeze(&mut self, event_queue: &mut EventQueue<FoamShot>) {
        // 收集 Output ID
        let outputs: Vec<_> = if let Some(foam_outputs) = self.wayland_ctx.foam_outputs.as_mut() {
            foam_outputs.iter().map(|(i, _)| *i).collect()
        } else {
            Vec::new()
        };

        for i in outputs {
            self.wayland_ctx.attach_with_udata(i);
        }

        event_queue.blocking_dispatch(self).unwrap();
    }

    /// if current compositor unsupported zwl screencopy, foamshot will be exit
    pub fn check_ok(&self) {
        // check screencopy manager exists
        if self.wayland_ctx.screencopy_manager.is_none() {
            notify::send(
                notify::NotificationLevel::Error,
                "this compositor unsupported zwl screencopy, foamshot will be exit",
            );
            std::process::exit(1);
        }
    }
}
