use log::*;
use smithay_client_toolkit::shm::{Shm, slot::SlotPool};
use wayland_client::{Connection, globals::registry_queue_init};
use wayland_protocols::wp::cursor_shape::v1::client::wp_cursor_shape_device_v1;

use crate::mode::{CopyHook, Mode, freeze_mode, result_mode, select_mode};
use crate::{config, mode, wayland_ctx};

pub struct FoamShot {
    pub wayland_ctx: wayland_ctx::WaylandCtx,

    pub cli: config::Cli,
    pub freeze_mode: freeze_mode::FreezeMode,
    pub select_mode: select_mode::SelectMode,
    pub result_mode: result_mode::ResultMode,
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

    info!("into loop");
    loop {
        std::thread::sleep(std::time::Duration::from_millis(16));
        event_queue.blocking_dispatch(&mut shot_foam).unwrap();
        match &shot_foam.mode {
            Mode::Freeze(CopyHook::Request) => {
                shot_foam.freeze_mode.before(&mut shot_foam.wayland_ctx);
            }
            Mode::Freeze(CopyHook::BufferDone) => {
                shot_foam.select_mode.before(&mut shot_foam.wayland_ctx);

                // NOTE: see ./imp/impl_foam_shot.rs for details
            }
            Mode::Freeze(CopyHook::Ready) => {
                shot_foam.freeze_mode.on(&mut shot_foam.wayland_ctx);

                shot_foam.mode = Mode::PreSelect;
            }
            Mode::PreSelect => {
                shot_foam.select_mode.on(&mut shot_foam.wayland_ctx);
                shot_foam
                    .wayland_ctx
                    .set_cursor_shape(wp_cursor_shape_device_v1::Shape::Crosshair);
                shot_foam.mode = Mode::Await;
            }
            Mode::Await => {
                // NOTE: 这个模式不做处理，用于等待鼠标按下
            }
            Mode::OnDraw => {
                if let Some((end_x, end_y)) = shot_foam.wayland_ctx.current_pos {
                    if (end_x, end_y) == shot_foam.select_mode.last_pos {
                        // NOTE: 鼠标没有移动
                        continue;
                    }
                }
                shot_foam.select_mode.after(&mut shot_foam.wayland_ctx);
                event_queue.roundtrip(&mut shot_foam).unwrap();
            }
            Mode::ShowResult => {}
            // Mode::Output(CopyHook::Request) => {
            //     shot_foam.result_mode.to_png_2(
            //         &mut shot_foam.cli,
            //         &mut shot_foam.wayland_ctx,
            //         &mut shot_foam.freeze_mode,
            //     );
            //
            //     // shot_foam.result_mode.before(&mut shot_foam.wayland_ctx);
            // }
            Mode::Output => {
                shot_foam.result_mode.to_png_2(
                    &mut shot_foam.cli,
                    &mut shot_foam.wayland_ctx,
                    &mut shot_foam.freeze_mode,
                );
                shot_foam.mode = Mode::Exit;
            }
            Mode::Exit => {
                std::process::exit(0);
            }
            _ => (),
        }
    }
}

impl FoamShot {
    pub fn new(shm: Shm, pool: SlotPool, qh: wayland_client::QueueHandle<FoamShot>) -> FoamShot {
        let cli = config::Cli::new();
        Self {
            wayland_ctx: wayland_ctx::WaylandCtx::new(shm, pool, qh),
            freeze_mode: mode::freeze_mode::FreezeMode::new(cli.no_cursor),
            select_mode: mode::select_mode::SelectMode::default(),
            result_mode: mode::result_mode::ResultMode::new(cli.quickshot),
            cli,
            mode: mode::Mode::default(),
        }
    }

    #[allow(unused)]
    pub fn test_mode(&mut self) {
        self.mode = Mode::Await;
        println!("FoamShot test_mode called; new mode: {:?}", self.mode);
    }
}
