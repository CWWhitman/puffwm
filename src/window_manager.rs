extern crate x11;

use self::x11::xlib;
use std::mem;
use std::os::raw::c_int;
use std::ptr::null;

static mut WM_DETECTED: bool = false;

pub struct WindowManager {
    display: *mut xlib::Display,
    root: xlib::Window,
}

impl WindowManager {
    pub fn new(display: *mut xlib::Display) -> WindowManager {
        if !display.is_null() {
            WindowManager {
                display: display,
                root: unsafe { xlib::XDefaultRootWindow(display) as xlib::Window },
            }
        } else {
            panic!("pointer to display was null");
        }
    }

    pub fn create() -> *mut xlib::Display {
        info!("Getting display pointer");
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

    unsafe extern "C" fn on_wmdetected(
        display: *mut xlib::Display,
        error: *mut xlib::XErrorEvent,
    ) -> c_int {
        assert_eq!((*error).error_code, xlib::BadAccess);
        WM_DETECTED = true;
        0
    }

    unsafe extern "C" fn on_xerror(
        display: *mut xlib::Display,
        error: *mut xlib::XErrorEvent,
    ) -> c_int {
        error!("{:?}", (*error).error_code as u8);
        0
    }

    pub fn run(&self) {
        unsafe {
            xlib::XSetErrorHandler(Some(WindowManager::on_wmdetected));
            xlib::XSelectInput(
                self.display,
                self.root,
                xlib::SubstructureRedirectMask | xlib::SubstructureNotifyMask,
            );
            xlib::XSync(self.display, 0);
            if WM_DETECTED {
                error!("another window manager is running");
                return;
            }
            xlib::XSetErrorHandler(Some(WindowManager::on_xerror));

            loop {
                let mut event = mem::uninitialized();
                xlib::XNextEvent(self.display, &mut event);
                info!("recieved {} event", event.get_type());

                match event.get_type() {
                    xlib::CreateNotify => {}
                    xlib::DestroyNotify => {}
                    xlib::ReparentNotify => {}
                    _ => {
                        warn!("ignored event");
                    }
                }
            }
        }
    }
}

impl Drop for WindowManager {
    fn drop(&mut self) {
        info!("display being dropped");
        unsafe {
            xlib::XCloseDisplay(self.display);
        }
    }
}
