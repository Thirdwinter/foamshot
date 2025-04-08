mod action;
mod config;
mod foam_outputs;
mod foamshot;
mod notify;
mod pointer_helper;
mod protocols;
mod save_helper;
mod wayland_ctx;

fn main() {
    env_logger::init();

    foamshot::run_main_loop();
}
