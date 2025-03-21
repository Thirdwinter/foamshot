mod config;
mod freeze_mode;
mod imp;
mod result_output;
mod select_mode;
mod shot_foam;
mod utility;
fn main() {
    println!("Hello, world!");
    let _ = config::Config::new();
    shot_foam::run_main_loop().unwrap();
}
