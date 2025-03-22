mod cli;
mod freeze_mode;
mod imp;
mod pointer_helper;
mod result_output;
mod select_mode;
mod shot_foam;
mod utility;

fn main() {
    env_logger::init();
    shot_foam::run_main_loop().unwrap();
}
