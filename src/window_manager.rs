extern crate x11;

use self::x11::xlib;
use self::x11::keysym;
use std::collections::HashMap;
use std::mem;
use std::os::raw::*;
use std::ptr::null;
use std::slice;

static mut WM_DETECTED: bool = false;

pub struct WindowManager {
    display: *mut xlib::Display,
    root: xlib::Window,
    clients: HashMap<xlib::Window, xlib::Window>,
}

impl WindowManager {
    pub fn new(display: *mut xlib::Display) -> WindowManager {
        if !display.is_null() {
            WindowManager {
                display: display,
                root: unsafe { xlib::XDefaultRootWindow(display) as xlib::Window },
                clients: HashMap::new(),
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

    pub fn run(&mut self) {
        info!("initializing wm");
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
            info!("done with init");

            info!("inheriting previous windows");
            xlib::XGrabServer(self.display);

            let mut root: xlib::Window = mem::uninitialized();
            let mut parent: xlib::Window = mem::uninitialized();
            let mut children: *mut xlib::Window = mem::uninitialized();
            let mut nchildren: c_uint = mem::uninitialized();

            xlib::XQueryTree(
                self.display,
                self.root,
                &mut root,
                &mut parent,
                &mut children,
                &mut nchildren,
            );
            assert_eq!(root, self.root);

            info!("risky");
            for child in slice::from_raw_parts_mut(children, nchildren as usize) {
                self.frame(*child, true);
            }
            info!("wow really");

            if !children.is_null() {
                xlib::XFree(children as *mut _);
            }

            xlib::XUngrabServer(self.display);

            loop {
                let mut event = mem::uninitialized();
                xlib::XNextEvent(self.display, &mut event);
                info!("recieved {} event", event.get_type());

                self.process_event(&mut event);
            }
        }
    }

    fn process_event(&mut self, mut event: &xlib::XEvent) {
        match event.get_type() {
            xlib::ConfigureRequest => {
                let event: &xlib::XConfigureRequestEvent = event.as_ref();
                let mut changes = xlib::XWindowChanges {
                    x: event.x,
                    y: event.y,
                    width: event.width,
                    height: event.height,
                    border_width: event.border_width,
                    sibling: event.above,
                    stack_mode: event.detail,
                };

                if self.clients.contains_key(&event.window) {
                    let frame = self.clients.get(&event.window);
                    let frame = frame.expect("who did this???");
                    unsafe {
                        xlib::XConfigureWindow(
                            self.display,
                            *frame,
                            event.value_mask as u32,
                            &mut changes as *mut xlib::XWindowChanges,
                        );
                    }
                    // put resize info here
                }

                unsafe {
                    xlib::XConfigureWindow(
                        self.display,
                        event.window,
                        event.value_mask as u32,
                        &mut changes as *mut xlib::XWindowChanges,
                    );
                }
            }

            xlib::MapRequest => {
                let event: &xlib::XMapRequestEvent = event.as_ref();
                self.frame(event.window, false);
                unsafe {
                    xlib::XMapWindow(self.display, event.window);
                }
            }

            xlib::UnmapNotify => {
                let event: &xlib::XUnmapEvent = event.as_ref();
                let window = event.window;

                if !self.clients.contains_key(&window) {
                    info!("ignoring this client, what a bad guy");
                    return;
                }
                self.unframe(event.window);
            }

            xlib::KeyPress => {
                let event: &xlib::XMotionEvent = event.as_ref();

                if (event.state & xlib::Button1Mask) {


            xlib::ConfigureNotify => {}

            xlib::CreateNotify => {}

            xlib::MapNotify => {}

            xlib::DestroyNotify => {}

            xlib::ReparentNotify => {}

            _ => {
                warn!("ignored event");
            }
        }
    }

    fn frame(&mut self, window: xlib::Window, created_before_wm: bool) {
        const BORDER_WIDTH: c_uint = 3;
        const BORDER_COLOR: c_ulong = 0xff82d3;
        const BG_COLOR: c_ulong = 0x333333;

        unsafe {
            let mut window_attrs: xlib::XWindowAttributes = mem::uninitialized();
            xlib::XGetWindowAttributes(
                self.display,
                window,
                &mut window_attrs as *mut xlib::XWindowAttributes,
            );

            if created_before_wm {
                if window_attrs.override_redirect != 0 || window_attrs.map_state != xlib::IsViewable
                {
                    return;
                }
            }

            let frame = xlib::XCreateSimpleWindow(
                self.display,
                self.root,
                window_attrs.x,
                window_attrs.y,
                window_attrs.width as u32,
                window_attrs.height as u32,
                BORDER_WIDTH,
                BORDER_COLOR,
                BG_COLOR,
            );
            xlib::XSelectInput(
                self.display,
                frame,
                xlib::SubstructureRedirectMask | xlib::SubstructureNotifyMask,
            );
            xlib::XAddToSaveSet(self.display, window);
            xlib::XReparentWindow(self.display, window, frame, 0, 0);
            xlib::XMapWindow(self.display, frame);
            //xlib
            info!("framed {:?} in {:?}", window, frame);
            self.clients.insert(window, frame);
            XGrabKey(
                self.display,
                xlib::XKeysymToKeycode(self.display, keysym::XK_h),
                Mod1Mask,
                window,
                false,
                xlib::GrabModeAsync,
                xlib::GrabModeAsync);
        }
            
    }

    fn unframe(&mut self, window: xlib::Window) {
        {
            let frame = self.clients.get(&window);
            let frame = frame.expect("who did this???");

            unsafe {
                xlib::XUnmapWindow(self.display, *frame);
                xlib::XReparentWindow(self.display, window, self.root, 0, 0);
                xlib::XRemoveFromSaveSet(self.display, window);
                xlib::XDestroyWindow(self.display, *frame);
            }
            info!("unframed {:?} from {:?}", window, frame);
        }
        self.clients.remove(&window);
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
