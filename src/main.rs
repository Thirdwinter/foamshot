mod action;
mod freeze_mode;
mod imp;
mod select_mode;
mod shot_fome;
fn main() {
    println!("Hello, world!");
    shot_fome::run_main_loop().unwrap();
}
