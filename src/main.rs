mod freeze_mode;
mod imp;
mod select_mode;
mod shot_fome;
mod utility;
fn main() {
    println!("Hello, world!");
    shot_fome::run_main_loop().unwrap();
}
