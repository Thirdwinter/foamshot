mod imp;
use std::collections::HashMap;

use smithay_client_toolkit::shm::{Shm, slot::SlotPool};
use wayland_client::protocol::{
    wl_compositor, wl_keyboard, wl_output, wl_pointer, wl_seat, wl_surface,
};
use wayland_client::{Connection, globals::registry_queue_init, protocol::wl_shm::Format};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{self, Layer},
    zwlr_layer_surface_v1::{self, Anchor, KeyboardInteractivity},
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

    event_queue.roundtrip(&mut data).expect("init failed");
    event_queue.blocking_dispatch(&mut data).unwrap();

    for (i, v) in data.output.as_ref().unwrap().iter() {
        let ls = data.layer_shell.as_mut().unwrap();
        let surface = data.surface.as_mut().unwrap().get_mut(i).unwrap();
        let layer = ls.get_layer_surface(
            surface,
            Some(v),
            Layer::Top,
            "debug_layer".to_string(),
            &qh,
            *i,
        );
        layer.set_anchor(Anchor::all());
        layer.set_exclusive_zone(-1);
        layer.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);

        data.layer_surface.as_mut().unwrap().insert(*i, layer);
        let w = data.w.get(i).unwrap();
        let h = data.h.get(i).unwrap();
        surface.damage(0, 0, *w, *h);
        surface.commit();
    }

    while data.layer_ready != data.output.as_mut().unwrap().len() {
        event_queue.blocking_dispatch(&mut data).unwrap();
    }

    for (i, _v) in data.output.as_ref().unwrap().iter() {
        let surface = data.surface.as_mut().unwrap().get_mut(i).unwrap();
        let w = data.w.get(i).unwrap();
        let h = data.h.get(i).unwrap();
        let pool = data.pool.as_mut().unwrap();

        let (buffer, canvas) = pool
            .create_buffer(*w, *h, *w * 4, Format::Argb8888)
            .unwrap();
        canvas.fill(100);
        buffer.attach_to(surface).unwrap();
        surface.damage(0, 0, *w, *h);
        surface.commit();
    }

    while data.running {
        event_queue.blocking_dispatch(&mut data).unwrap();
    }
}
