mod action;
mod cairo_render;
mod config;
mod foamcore;
mod monitors;
mod notify;
mod pointer_helper;
mod protocols;
mod save_helper;
mod select_rect;
mod wayland_ctx;
mod zwlr_screencopy_mode;

fn main() {
    env_logger::init();

    foamcore::run_main_loop();
}
