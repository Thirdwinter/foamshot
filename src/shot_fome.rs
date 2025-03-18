use smithay_client_toolkit::shm::{
    self, Shm,
    slot::{self, SlotPool},
};
use wayland_client::{
    Connection, QueueHandle,
    globals::registry_queue_init,
    protocol::{wl_compositor, wl_keyboard, wl_output, wl_pointer, wl_seat},
};
use wayland_protocols::wp::cursor_shape::v1::client::{
    wp_cursor_shape_device_v1, wp_cursor_shape_manager_v1,
};
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::{self};

use crate::{freeze_mode::FreezeMode, select_mode::SelectMode, utility::Action};

pub struct ShotFome {
    pub compositor: Option<wl_compositor::WlCompositor>,
    pub output: Option<wl_output::WlOutput>,
    pub pool: Option<slot::SlotPool>,
    pub shm: Option<shm::Shm>,
    pub seat: Option<wl_seat::WlSeat>,
    pub pointer: Option<wl_pointer::WlPointer>,
    pub keyboard: Option<wl_keyboard::WlKeyboard>,
    pub layer_shell: Option<zwlr_layer_shell_v1::ZwlrLayerShellV1>,
    pub qh: Option<QueueHandle<ShotFome>>,

    pub cursor_shape_manager: Option<wp_cursor_shape_manager_v1::WpCursorShapeManagerV1>,
    pub cursor_shape_device: Option<wp_cursor_shape_device_v1::WpCursorShapeDeviceV1>,

    pub phys_width: Option<i32>,
    pub phys_height: Option<i32>,
    pub current_pos: Option<(f64, f64)>,
    pub pointer_start: Option<(f64, f64)>,
    pub pointer_end: Option<(f64, f64)>,
    pub action: Action,

    pub freeze_mode: FreezeMode,
    pub select_mode: SelectMode,
}

impl ShotFome {
    pub fn new(shm: shm::Shm, pool: slot::SlotPool, qh: QueueHandle<ShotFome>) -> Self {
        let freeze_mode = FreezeMode::default();
        let select_mode = SelectMode::default();

        Self {
            compositor: None,
            output: None,
            pool: Some(pool),
            shm: Some(shm),
            seat: None,
            pointer: None,
            keyboard: None,
            layer_shell: None,
            qh: Some(qh.clone()),
            phys_width: None,
            phys_height: None,
            current_pos: None,
            pointer_start: None,
            pointer_end: None,
            cursor_shape_manager: None,
            cursor_shape_device: None,
            action: Action::PRELOAD,
            freeze_mode,
            select_mode,
        }
    }
}
pub fn run_main_loop() -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::connect_to_env().unwrap();

    let (globals, mut event_queue) = registry_queue_init::<ShotFome>(&connection).unwrap();

    let qh = event_queue.handle();
    let display = connection.display();
    let _registry = display.get_registry(&qh, ());
    let shm = Shm::bind(&globals, &qh).expect("wl_shm is not available");
    let pool = SlotPool::new(256 * 256 * 4, &shm).expect("Failed to create pool");
    let mut shot_foam = ShotFome::new(shm, pool, qh);

    // NOTE: 处理注册事件
    event_queue.roundtrip(&mut shot_foam).unwrap();
    event_queue.blocking_dispatch(&mut shot_foam).unwrap();

    // shot_foam.prev_freeze_screen();
    // NOTE: 逻辑封装到对应的模块

    // NOTE: 冻结屏幕 初始帧copy
    shot_foam.freeze_mode.prev_freeze_screen(
        shot_foam.layer_shell.clone(),
        shot_foam.output.clone(),
        shot_foam.qh.clone(),
        shot_foam.phys_width,
        shot_foam.phys_height,
    );
    // NOTE: 生成选择模块的surface
    shot_foam.select_mode.prev_select(
        shot_foam.phys_width,
        shot_foam.phys_height,
        shot_foam.layer_shell.clone(),
        shot_foam.output.clone(),
        shot_foam.qh.clone(),
    );

    loop {
        std::thread::sleep(std::time::Duration::from_millis(16));
        event_queue.blocking_dispatch(&mut shot_foam).unwrap();
        let action = &shot_foam.action;
        match &action {
            Action::Onselect => {
                // TODO:
                shot_foam.select_mode.update_select(
                    shot_foam.phys_width,
                    shot_foam.phys_height,
                    &mut shot_foam.pool.as_mut().unwrap(),
                    shot_foam.pointer_start,
                    shot_foam.current_pos,
                )
            }
            Action::FREEZE => {
                continue;
            }
            Action::EXIT => {
                std::process::exit(0);
            }
            _ => {}
        }
    }
}
