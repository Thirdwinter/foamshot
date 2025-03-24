mod cli;
mod foam_shot;
mod imp;
mod mode;
mod wayland_ctx;
fn main() {
    env_logger::init();
    foam_shot::run_main_loop();
}
