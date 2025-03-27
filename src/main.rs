mod config;
mod foam_shot;
mod imp;
mod mode;
mod state;
mod wayland_ctx;
fn main() {
    env_logger::init();
    foam_shot::run_main_loop();
    // foam_shot::run_main_loop();
}
