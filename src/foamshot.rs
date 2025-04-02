use std::collections::HashMap;
use std::hash::Hash;

use log::*;
use smithay_client_toolkit::shm::{Shm, slot::SlotPool};
use wayland_client::{Connection, globals::registry_queue_init};

use crate::mode::{Mode, freeze_mode};
use crate::{mode, wayland_ctx};

pub struct FoamShot {
    pub wayland_ctx: wayland_ctx::WaylandCtx,

    pub freeze_mode: freeze_mode::FreezeMode,
    pub editor_mode: mode::editor_mode::EditorMode,
    pub mode: mode::Mode,
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

    // check screencopy manager exists
    if let None = shot_foam.wayland_ctx.screencopy_manager {
        error!("screencopy manager not available");
        std::process::exit(1);
    } else {
        info!("screencopy manager available");
    }

    shot_foam.wayland_ctx.request_screencopy();

    // 等待所有屏幕copy完成
    while shot_foam.wayland_ctx.frames_ready
        != shot_foam.wayland_ctx.outputs.as_ref().unwrap().len()
    {
        event_queue.blocking_dispatch(&mut shot_foam).unwrap();
    }
    // 存储 copy 到的数据
    shot_foam.wayland_ctx.store_copy_canvas();

    // NOTE: 创建layer && surface提交
    shot_foam.freeze_mode.before(&mut shot_foam.wayland_ctx);

    // NOTE: 等待处理事件
    event_queue.blocking_dispatch(&mut shot_foam).unwrap();

    // NOTE: buffer attach to surface
    // shot_foam.freeze_mode.set_freeze(&mut shot_foam.wayland_ctx);

    println!("{:?}", shot_foam.wayland_ctx.monitors.as_ref().unwrap());

    loop {
        std::thread::sleep(std::time::Duration::from_millis(16));
        event_queue.blocking_dispatch(&mut shot_foam).unwrap();
        match &shot_foam.mode {
            Mode::Init => {}
            Mode::OnFreeze => {}
            Mode::OnDraw => {
                shot_foam
                    .freeze_mode
                    .update_select_region(&mut shot_foam.wayland_ctx);
            }
            Mode::Exit => {
                shot_foam.wayland_ctx.generate_sub_rects();
                println!("{:?}", shot_foam.wayland_ctx.subrects.as_ref().unwrap());
                let m = shot_foam
                    .wayland_ctx
                    .monitors
                    .as_ref()
                    .unwrap()
                    .get(
                        shot_foam
                            .wayland_ctx
                            .pointer_helper
                            .start_index
                            .as_ref()
                            .unwrap(),
                    )
                    .unwrap();
                let (sx, sy) = shot_foam
                    .wayland_ctx
                    .pointer_helper
                    .start_pos
                    .clone()
                    .unwrap();
                let (ex, ey) = shot_foam
                    .wayland_ctx
                    .pointer_helper
                    .end_pos
                    .clone()
                    .unwrap();

                // 将相对坐标转换为全局坐标
                let start_global_x = m.x + sx as i32;
                let start_global_y = m.y + sy as i32;
                let end_global_x = m.x + ex as i32;
                let end_global_y = m.y + ey as i32;

                // // 确定左上角坐标
                let x0 = start_global_x.min(end_global_x);
                let y0 = start_global_y.min(end_global_y);
                //
                // // 计算宽高（绝对值保证结果为正数）
                let width = (ex - sx).abs();
                let height = (ey - sy).abs();
                println!(
                    "({},{}) ({},{})",
                    x0 as i32, y0 as i32, width as i32, height as i32
                );

                std::process::exit(0)
            }
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
            editor_mode: mode::editor_mode::EditorMode::default(),
            // result_mode: mode::result_mode::ResultMode::new(cli.quickshot),
            // cli,
            mode: mode::Mode::default(),
        }
    }
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
