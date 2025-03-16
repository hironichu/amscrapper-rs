mod scrapper;
mod utils;
use std::ffi::c_void;

use scrapper::AMusicScraper;
use uiautomation::{UIAutomation, UIElement};

pub extern "C" fn init_automation() -> *mut UIAutomation {
    let automation = UIAutomation::new();
    if automation.is_err() {
        return std::ptr::null_mut();
    }
    let automation = automation.unwrap();
    Box::into_raw(Box::new(automation))
}

pub extern "C" fn grap_am_window_ptr(automation: *mut UIAutomation) -> *mut UIElement {
    assert!(!automation.is_null());
    let automation = unsafe { &mut *automation };
    let apple_music = utils::grab_applemusic_window(&automation);
    if apple_music.is_none() {
        return std::ptr::null_mut();
    }
    let window = apple_music.unwrap();
    Box::into_raw(Box::new(window))
}

pub extern "C" fn init_scrapper() {}

pub extern "C" fn init() {}
