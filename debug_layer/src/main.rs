mod imp;
use std::collections::HashMap;

use smithay_client_toolkit::shm::{Shm, slot::SlotPool};
use wayland_client::protocol::{
    wl_compositor, wl_keyboard, wl_output, wl_pointer, wl_seat, wl_surface,
};
use wayland_client::{Connection, globals::registry_queue_init};
use wayland_protocols::xdg::xdg_output::zv1::client::zxdg_output_manager_v1;
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::Layer;
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::{
    Anchor, KeyboardInteractivity,
};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{self},
    zwlr_layer_surface_v1::{self},
};

#[derive(Default)]
pub struct Data {
    pub compositor: Option<wl_compositor::WlCompositor>,
    pub layer_shell: Option<zwlr_layer_shell_v1::ZwlrLayerShellV1>,
    pub layer_surface: Option<HashMap<usize, zwlr_layer_surface_v1::ZwlrLayerSurfaceV1>>,
    pub output: Option<HashMap<usize, wl_output::WlOutput>>,
    pub surface: Option<HashMap<usize, wl_surface::WlSurface>>,
    pub seat: Option<wl_seat::WlSeat>,
    pub keyboard: Option<wl_keyboard::WlKeyboard>,
    pub pointer: Option<wl_pointer::WlPointer>,
    pub shm: Option<Shm>,
    pub pool: Option<SlotPool>,
    pub xdgoutputmanager: Option<zxdg_output_manager_v1::ZxdgOutputManagerV1>,
    pub is_configured: HashMap<usize, bool>,

    pub w: HashMap<usize, i32>,
    pub h: HashMap<usize, i32>,

    pub running: bool,
    pub layer_ready: usize,
}

impl Data {
    fn new() -> Self {
        Self {
            layer_surface: Some(HashMap::new()),
            output: Some(HashMap::new()),
            surface: Some(HashMap::new()),
            w: HashMap::new(),
            h: HashMap::new(),
            is_configured: HashMap::new(),
            running: true,
            ..Default::default()
        }
    }
}

fn main() {
    env_logger::init();

    let mut data: Data = Data::new();

    let connection = Connection::connect_to_env().expect("can't connect to wayland display");
    let (globals, mut event_queue) =
        registry_queue_init::<Data>(&connection).expect("failed to get globals");
    let qh = event_queue.handle();
    let display = connection.display();
    let _registry = display.get_registry(&qh, ());

    let shm = Shm::bind(&globals, &qh).expect("wl_shm is not available");
    data.pool = SlotPool::new(256 * 256, &shm).ok();
    data.shm = Some(shm);
    event_queue.roundtrip(&mut data).unwrap();

    for i in 0..data.output.as_ref().unwrap().len() {
        let v = data.output.as_ref().unwrap().get(&i).unwrap();
        let s = data.compositor.as_ref().unwrap().create_surface(&qh, i);
        let ls = data.layer_shell.as_ref().unwrap();
        let layer = ls.get_layer_surface(
            &s,
            Some(v),
            Layer::Overlay,
            "debug_layer".to_string(),
            &qh,
            i,
        );
        data.is_configured.insert(i, false);
        layer.set_anchor(Anchor::Top | Anchor::Bottom | Anchor::Left | Anchor::Right);
        layer.set_exclusive_zone(-1);
        layer.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);
        s.frame(&qh, i);
        s.commit();

        data.layer_surface.as_mut().unwrap().insert(i, layer);
        data.surface.as_mut().unwrap().insert(i, s);
    }

    event_queue.roundtrip(&mut data).unwrap();

    // event_queue
    //     .blocking_dispatch(&mut data)
    //     .expect("init failed");

    loop {
        event_queue.blocking_dispatch(&mut data).unwrap();
    }
}
