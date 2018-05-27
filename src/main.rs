#[macro_use]
extern crate log;

mod window_manager;
use window_manager::WindowManager;

fn main() {
    trace!("Creating window manager");
    let wm = window_manager::WindowManager::new(WindowManager::create());
    wm.run();
}

