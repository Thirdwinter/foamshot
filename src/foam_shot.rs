use std::collections::HashMap;
use std::hash::Hash;

use log::*;
use smithay_client_toolkit::shm::{Shm, slot::SlotPool};
use wayland_client::{Connection, globals::registry_queue_init};

use crate::mode::{Mode, freeze_mode, select_mode};
use crate::{mode, wayland_ctx};

pub struct FoamShot {
    pub wayland_ctx: wayland_ctx::WaylandCtx,

    pub freeze_mode: freeze_mode::FreezeMode,
    pub select_mode: select_mode::SelectMode,
    // pub result_mode: result_mode::ResultMode,
    pub mode: mode::Mode,
}

pub fn hs_insert<T, V>(state_hm: &mut Option<HashMap<T, V>>, key: T, value: V)
where
    T: Eq + Hash,
    V: Clone,
{
    match state_hm {
        Some(hm) => {
            hm.insert(key, value.clone());
        }
        None => {
            let mut new_hm = HashMap::new();
            new_hm.insert(key, value.clone());
            *state_hm = Some(new_hm);
        }
    }
}
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

    if let None = shot_foam.wayland_ctx.screencopy_manager {
        error!("screencopy manager not available");
        std::process::exit(1);
    } else {
        info!("screencopy manager available");
    }
    shot_foam.wayland_ctx.request_screencopy();

    while shot_foam.wayland_ctx.frames_ready
        != shot_foam.wayland_ctx.outputs.as_ref().unwrap().len()
    {
        event_queue.blocking_dispatch(&mut shot_foam).unwrap();
    }

    // NOTE: 创建layer && surface提交
    shot_foam.freeze_mode.before(&mut shot_foam.wayland_ctx);
    shot_foam.select_mode.before(&mut shot_foam.wayland_ctx);
    // NOTE: 等待处理事件
    event_queue.blocking_dispatch(&mut shot_foam).unwrap();
    // NOTE: buffer attach to surface
    shot_foam.freeze_mode.set_freeze(&mut shot_foam.wayland_ctx);
    shot_foam
        .select_mode
        .await_select(&mut shot_foam.wayland_ctx);

    loop {
        std::thread::sleep(std::time::Duration::from_millis(16));
        event_queue.blocking_dispatch(&mut shot_foam).unwrap();
        match &shot_foam.mode {
            Mode::Exit => std::process::exit(0),
            _ => (),
        }
    }
}

impl FoamShot {
    pub fn new(shm: Shm, pool: SlotPool, qh: wayland_client::QueueHandle<FoamShot>) -> FoamShot {
        // let cli = config::Cli::new();
        Self {
            wayland_ctx: wayland_ctx::WaylandCtx::new(shm, pool, qh),
            freeze_mode: mode::freeze_mode::FreezeMode::new(),
            select_mode: mode::select_mode::SelectMode::default(),
            // result_mode: mode::result_mode::ResultMode::new(cli.quickshot),
            // cli,
            mode: mode::Mode::default(),
        }
    }
}
