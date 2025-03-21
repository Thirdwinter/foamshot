mod config;
mod freeze_mode;
mod imp;
mod result_output;
mod select_mode;
mod shot_foam;
mod utility;

fn main() {
    // env_logger::builder()
    //     .filter(Some("shot_fome"), log::LevelFilter::Info)
    //     .init();
    env_logger::init();
    shot_foam::run_main_loop().unwrap();
}
