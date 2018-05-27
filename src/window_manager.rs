extern crate x11;

use std::ptr::{
    null,
    null_mut,
};
use self::x11::xlib;

pub struct WindowManager {
    display: *mut xlib::Display,
    root: xlib::Window,
}

impl WindowManager {
    pub fn new(display: *mut xlib::Display) -> WindowManager {
        if !display.is_null() {
            WindowManager {
                display: display,
                root: unsafe { xlib::XDefaultRootWindow(display) as xlib::Window}
            }
        } else {
            panic!("pointer to display was null");
        }
    }


    pub fn create() -> *mut xlib::Display {
        trace!("Getting display pointer");
        let display;
        unsafe {
            display = xlib::XOpenDisplay(null());
            if display.is_null() {
                error!("Failed to open X display");
                panic!("fail");
            }
        }

        display
    }

    pub fn run(&self) {
        /* nothing */
    }
}

impl Drop for WindowManager {
    fn drop(&mut self) {
        unsafe {
            xlib::XCloseDisplay(self.display);
        }
    }
}

