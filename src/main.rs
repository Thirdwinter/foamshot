mod action;
mod cairo_render;
mod config;
mod foamcore;
mod frame_queue;
mod monitors;
mod notify;
mod pointer_helper;
mod protocols;
mod save_helper;
mod select_rect;
mod wayland_ctx;
mod zwlr_screencopy_mode;

fn main() {
    // Init the env logger
    env_logger::init();
    // run the main event loop
    foamcore::run_main_loop();
}
