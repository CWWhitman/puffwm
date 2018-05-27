#[macro_use]
extern crate log;
extern crate env_logger;

mod window_manager;
use window_manager::WindowManager;

fn main() {
    env_logger::init();

    info!("Creating window manager");
    let mut wm = window_manager::WindowManager::new(WindowManager::create());
    wm.run();
}
