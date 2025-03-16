use std::sync::{Arc, Mutex};
use uiautomation::types::Handle;
use uiautomation::{UIAutomation, UIElement};
use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::EnumThreadWindows;

pub(crate) type GetTargetType = Arc<Mutex<Option<Vec<HWND>>>>;
pub(crate) fn find_app_hwnd() -> Option<Vec<HWND>> {
    let storage: GetTargetType = Arc::new(Mutex::new(None));
    let l_param = LPARAM(&storage as *const GetTargetType as isize);

    unsafe {
        let _ = EnumThreadWindows(0, Some(find_target_process), l_param);
    }

    let state = storage.lock().unwrap().take();
    state
}

extern "system" fn find_target_process(hwnd: HWND, l_param: LPARAM) -> BOOL {
    let storage = unsafe { &*(l_param.0 as *const GetTargetType) };
    let mut storage = storage.lock().unwrap();
    storage.get_or_insert_with(|| Vec::new());
    storage.as_mut().unwrap().push(hwnd);

    BOOL(1)
}

pub(crate) fn grab_applemusic_window(automation: &UIAutomation) -> Option<UIElement> {
    let app_hwn =
        find_app_hwnd().expect("target app handle is not found. make sure the app is running");
    app_hwn.iter().find_map(|hwnd| {
        let element = automation.element_from_handle(Handle::from(hwnd.0 as isize));
        if element.is_err() {
            return None;
        }
        let element = element.unwrap();
        if element.get_name().unwrap() == "Apple Music" {
            let desk = winvd::get_desktop_by_window(*hwnd);
            return match desk {
                Ok(_) => {
                    if winvd::is_window_on_current_desktop(*hwnd).unwrap() {
                        return Some(element);
                    }
                    if winvd::is_pinned_app(*hwnd).unwrap() {
                        winvd::unpin_app(*hwnd).unwrap();
                    }
                    if winvd::is_pinned_window(*hwnd).unwrap() {
                        winvd::unpin_window(*hwnd).unwrap();
                    }
                    // winvd::move_window_to_desktop( winvd::get_current_desktop().unwrap(), hwnd).unwrap();
                    winvd::pin_window(*hwnd).unwrap();
                    Some(element)
                }
                Err(_) => None,
            };
        } else {
            None
        }
    })
}
