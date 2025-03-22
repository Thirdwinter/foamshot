use smithay_client_toolkit::shm::{
    self, Shm,
    slot::{self, SlotPool},
};
use wayland_client::{
    Connection, QueueHandle,
    globals::registry_queue_init,
    protocol::{wl_compositor, wl_keyboard, wl_output, wl_seat},
};
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::{self};

use crate::{
    cli::Cli, freeze_mode::FreezeMode, pointer_helper::PointerHelper, result_output::ResultOutput,
    select_mode::SelectMode, utility::Action,
};

pub struct ShotFoam {
    pub compositor: Option<wl_compositor::WlCompositor>,
    pub output: Option<wl_output::WlOutput>,
    pub pool: Option<slot::SlotPool>,
    pub shm: Option<shm::Shm>,
    pub seat: Option<wl_seat::WlSeat>,
    pub keyboard: Option<wl_keyboard::WlKeyboard>,
    pub layer_shell: Option<zwlr_layer_shell_v1::ZwlrLayerShellV1>,
    pub qh: Option<QueueHandle<ShotFoam>>,

    pub pointer_helper: PointerHelper,

    pub width: Option<i32>,
    pub height: Option<i32>,
    pub action: Action,

    pub freeze_mode: FreezeMode,
    pub select_mode: SelectMode,

    pub result_output: ResultOutput,

    pub cli: Cli,
    // pub pointer: Option<wl_pointer::WlPointer>,
    // pub cursor_shape_manager: Option<wp_cursor_shape_manager_v1::WpCursorShapeManagerV1>,
    // pub cursor_shape_device: Option<wp_cursor_shape_device_v1::WpCursorShapeDeviceV1>,
    // pub current_pos: Option<(f64, f64)>,
    // pub pointer_start: Option<(f64, f64)>,
    // pub pointer_end: Option<(f64, f64)>,
}

pub fn run_main_loop() -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::connect_to_env().unwrap();

    let (globals, mut event_queue) = registry_queue_init::<ShotFoam>(&connection).unwrap();

    let qh = event_queue.handle();
    let display = connection.display();
    let _registry = display.get_registry(&qh, ());
    let shm = Shm::bind(&globals, &qh).expect("wl_shm is not available");
    let pool = SlotPool::new(256 * 256 * 4, &shm).expect("Failed to create pool");
    let mut shot_foam = ShotFoam::new(shm, pool, qh);

    // NOTE: 处理注册事件
    event_queue.roundtrip(&mut shot_foam).unwrap();
    event_queue.blocking_dispatch(&mut shot_foam).unwrap();

    // NOTE: 冻结屏幕 初始帧copy
    shot_foam.create_freeze_layer_surface();
    // NOTE: 生成选择模块的surface
    shot_foam.create_select_layer_surface();

    loop {
        std::thread::sleep(std::time::Duration::from_millis(16));
        event_queue.blocking_dispatch(&mut shot_foam).unwrap();
        let action = &shot_foam.action;
        match &action {
            Action::Onselect => {
                // TODO:
                shot_foam.select_mode.update_select(
                    shot_foam.width,
                    shot_foam.height,
                    shot_foam.pool.as_mut().unwrap(),
                    shot_foam.pointer_helper.pointer_start,
                    shot_foam.pointer_helper.current_pos,
                )
            }
            Action::Freeze => {
                shot_foam.create_select_buffer();
                continue;
            }
            Action::Exit => {
                std::process::exit(0);
            }
            _ => {}
        }
    }
}
