use foamshot::foamcore;

fn main() {
    // Init the env logger
    env_logger::init();
    // run the main event loop
    foamcore::run_main_loop();
}
