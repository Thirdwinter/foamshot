use log::*;
use smithay_client_toolkit::shm::{slot::SlotPool, Shm};
use wayland_client::{globals::registry_queue_init, Connection};

use crate::{
    action::{self, Action},
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
    let pool = SlotPool::new(256 * 256 * 4, &shm).expect("Failed to create pool");
    let mut shot_foam = FoamShot::new(shm, pool, qh);

    event_queue.roundtrip(&mut shot_foam).expect("init failed");

    shot_foam.check_ok();

    // NOTE: 请求全屏copy，之后该去protocols::zwlr_screencopy_manager_v1中依次处理event
    shot_foam.wayland_ctx.request_screencopy();

    // 等待所有屏幕copy完成
    while shot_foam.wayland_ctx.frames_ready
        != shot_foam.wayland_ctx.foam_outputs.as_ref().unwrap().len()
    {
        event_queue.blocking_dispatch(&mut shot_foam).unwrap();
    }
    // 存储 copy 到的数据
    shot_foam.wayland_ctx.store_copy_canvas();

    // NOTE: 创建layer && surface提交
    shot_foam.wayland_ctx.init_base_layers();
    // shot_foam.freeze_mode.before(&mut shot_foam.wayland_ctx);

    // NOTE: 等待处理事件
    event_queue.blocking_dispatch(&mut shot_foam).unwrap();

    loop {
        event_queue.blocking_dispatch(&mut shot_foam).unwrap();
        match &shot_foam.mode {
            Action::Init => {}
            Action::WaitPointerPress => {}
            Action::OnDraw => {
                shot_foam.wayland_ctx.update_select_region();
            }
            Action::Exit => {
                shot_foam.wayland_ctx.generate_sub_rects();
                save_helper::save_to_png(&mut shot_foam.wayland_ctx).unwrap();

                std::process::exit(0)
            }
        }
    }
}

impl FoamShot {
    pub fn new(shm: Shm, pool: SlotPool, qh: wayland_client::QueueHandle<FoamShot>) -> FoamShot {
        Self {
            wayland_ctx: wayland_ctx::WaylandCtx::new(shm, pool, qh),
            mode: Action::default(),
        }
    }

    /// if current compositor unsupported zwl screencopy, foamshot will be exit
    pub fn check_ok(&self) {
        // check screencopy manager exists
        if let None = self.wayland_ctx.screencopy_manager {
            error!("screencopy manager not available");
            std::process::exit(1);
        } else {
            info!("screencopy manager available");
        }
    }
}
