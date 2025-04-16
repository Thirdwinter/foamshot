use log::{debug, error};
use smithay_client_toolkit::shm::Shm;
use wayland_client::{Connection, EventQueue, globals::registry_queue_init};

use crate::{
    action::{self, Action, IsFreeze},
    config::{FoamConfig, ImageType},
    notify::{self, NotificationLevel},
    save_helper, wayland_ctx,
};

pub struct FoamShot {
    /// foamshot wayland context
    pub wlctx: wayland_ctx::WaylandCtx,

    pub action: action::Action,
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
    debug!("{:?}", shot_foam.wlctx.config);

    event_queue.roundtrip(&mut shot_foam).expect("init failed");

    shot_foam.wait_copy(&mut event_queue);

    // NOTE: 创建layer && surface提交
    shot_foam.wlctx.init_base_layers();

    // NOTE: 等待处理事件
    event_queue.blocking_dispatch(&mut shot_foam).unwrap();

    loop {
        event_queue.blocking_dispatch(&mut shot_foam).unwrap();
        match &shot_foam.action {
            Action::Init => {}
            Action::WaitPointerPress => {}
            Action::ToggleFreeze(state) => {
                match state {
                    IsFreeze::NewFrameFreeze => {
                        debug!("next is freeze");
                        // 进行屏幕copy 通过计数器等待所有ready完成
                        shot_foam.wait_copy(&mut event_queue);
                    }
                    IsFreeze::UnFreeze | IsFreeze::OldFrameFreeze => {
                        debug!("next is unfreeze");
                    }
                }
                // 发送下一帧，重新附加buffer
                shot_foam.toggle_freeze(&mut event_queue);
                shot_foam.action = Action::WaitPointerPress
            }
            Action::OnDraw => {}
            Action::OnEdit(_a) => {}
            Action::Output => {
                if !shot_foam.wlctx.current_freeze {
                    shot_foam.wait_copy(&mut event_queue);
                }
                match shot_foam.wlctx.config.image_type {
                    ImageType::Png => {
                        if let Err(e) = save_helper::save_to_png(&mut shot_foam.wlctx) {
                            shot_foam.send_error("image saved error");
                            log::error!("save to png error: {}", e);
                        }
                        save_helper::save_to_wl_clipboard(&mut shot_foam.wlctx).unwrap();
                    }
                    ImageType::Jpg => {
                        if let Err(e) = save_helper::save_to_jpg(&mut shot_foam.wlctx, 100) {
                            shot_foam.send_error("image saved error");
                            log::error!("save to jpg error: {}", e);
                        }
                        save_helper::save_to_wl_clipboard(&mut shot_foam.wlctx).ok();
                    }
                }
                shot_foam.send_save_info();
                shot_foam.action = Action::Exit
            }
            Action::Pin => {
                // TODO:
            }
            Action::Exit => std::process::exit(0),
        }
    }
}

impl FoamShot {
    /// 创建新实例
    pub fn new(shm: Shm, qh: wayland_client::QueueHandle<FoamShot>) -> FoamShot {
        let config = FoamConfig::new();
        Self {
            wlctx: wayland_ctx::WaylandCtx::new(shm, qh, config),
            action: Action::default(),
        }
    }

    /// 临时借用 event_queue 进行copy
    /// 发起copy请求 -> 等待全部 output 完成 -> 重置计数器 -> 缓存 canvas
    pub fn wait_copy(&mut self, event_queue: &mut EventQueue<FoamShot>) {
        self.check_ok();

        // NOTE: 先确保屏幕为正常状态
        if self.action == Action::ToggleFreeze(IsFreeze::UnFreeze)
            || self.action == Action::ToggleFreeze(IsFreeze::NewFrameFreeze)
        {
            self.wlctx.unset_freeze();
        }

        // NOTE: 请求全屏copy，之后该去protocols::zwlr_screencopy_manager_v1中依次处理event
        self.wlctx.request_screencopy();

        // 等待所有屏幕copy完成
        while self.wlctx.scm.copy_ready != self.wlctx.foam_outputs.as_ref().unwrap().len() {
            match event_queue.blocking_dispatch(self) {
                Ok(_) => {}
                Err(e) => {
                    error!("error in wait_freeze: {}", e);
                    self.send_error("error about wait screencopy");
                    std::process::exit(0)
                }
            }
        }
        // 重置计数器
        self.wlctx.scm.copy_ready = 0;
        // 存储 copy 到的数据
        self.wlctx.store_copy_canvas();
    }

    /// 上层调用，切换所有输出上的屏幕冻结状态，在调用前需要使用 `wait_freeze` 重新进行屏幕copy
    pub fn toggle_freeze(&mut self, event_queue: &mut EventQueue<FoamShot>) {
        // 收集 Output ID
        let outputs: Vec<_> = if let Some(foam_outputs) = self.wlctx.foam_outputs.as_mut() {
            foam_outputs.iter().enumerate().map(|(i, _)| i).collect()
        } else {
            Vec::new()
        };

        for i in outputs {
            self.wlctx.attach_with_udata(i);
        }

        event_queue.blocking_dispatch(self).unwrap();
    }

    /// if current compositor unsupported zwl screencopy, foamshot will be exit
    pub fn check_ok(&self) {
        // check screencopy manager exists
        if self.wlctx.scm.manager.is_none() {
            self.send_error("this compositor unsupported zwl screencopy, foamshot will be exit");
            std::process::exit(1);
        }
    }

    pub fn send_save_info(&self) {
        notify::send(
            NotificationLevel::Info,
            "image_saved",
            format!(
                "Image saved in {}",
                self.wlctx.config.output_path.clone().display()
            ),
            self.wlctx.config.output_path.to_str().unwrap().to_string(),
            self.wlctx.config.allow_notify,
        );
    }

    pub fn send_error(&self, body: &str) {
        notify::send(
            NotificationLevel::Error,
            "foamshot error",
            body,
            "dialog-error",
            self.wlctx.config.allow_notify,
        );
    }

    pub fn send_warn(&self, body: &str) {
        notify::send(
            NotificationLevel::Warn,
            "foamshot warn",
            body,
            "dialog-warning",
            self.wlctx.config.allow_notify,
        );
    }
}
