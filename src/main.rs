mod config;
mod foamshot;
mod helper;
mod mode;
mod protocols;
mod wayland_ctx;

fn main() {
    // init log
    env_logger::init();

    foamshot::run_main_loop();
}
