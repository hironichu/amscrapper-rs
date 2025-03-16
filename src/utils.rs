use std::sync::{Arc, Mutex};
use uiautomation::types::Handle;
use uiautomation::{UIAutomation, UIElement};
use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowTextA};
/// Unsafe operation, but it's to avoid looping through every Window in every desktop, to avoid having huge start time
/// This should be used only in the context of this application
// static mut AUTOMATION: Option<UIAutomation> = None;

pub(crate) type GetTargetType = Arc<Mutex<Option<HWND>>>;
pub(crate) fn find_app_hwnd() -> Option<HWND> {
    let storage: GetTargetType = Arc::new(Mutex::new(None));
    let l_param = LPARAM(&storage as *const GetTargetType as isize);

    unsafe {
        let _ = EnumWindows(Some(find_target_process), l_param);
    }

    let state = storage.lock().unwrap().take();
    state
}

extern "system" fn find_target_process(hwnd: HWND, l_param: LPARAM) -> BOOL {
    let mut buffer = [0_u8; 128];
    let read_len = unsafe { GetWindowTextA(hwnd, &mut buffer) };
    let proc_name = String::from_utf8_lossy(&buffer[..read_len as usize]);
    if proc_name.as_ref() == "Apple Music" {
        let storage = unsafe { &*(l_param.0 as *const GetTargetType) };
        let mut storage = storage.lock().unwrap();
        storage.replace(hwnd);
        return BOOL(0);
    }

    BOOL(1)
}
/// This function is used to grab the Apple Music window
/// It also checks if the window is on the current desktop, if not it moves it to the current desktop
/// If the window is pinned, it unpins it
pub(crate) fn grab_applemusic_window(
    automation: &UIAutomation,
    move_window: bool,
) -> Option<UIElement> {
    let hwnd =
        find_app_hwnd().expect("target app handle is not found. make sure the app is running");
    // app_hwn.iter().find_map(|hwnd| {
    let element = automation.element_from_handle(Handle::from(hwnd.0 as isize));
    if element.is_err() {
        return None;
    }
    let element = element.unwrap();
    if element.get_name().unwrap() == "Apple Music" {
        let desk = winvd::get_desktop_by_window(hwnd);
        return match desk {
            Ok(_) => {
                if winvd::is_window_on_current_desktop(hwnd).unwrap() {
                    return Some(element);
                }
                if winvd::is_pinned_app(hwnd).unwrap() {
                    winvd::unpin_app(hwnd).unwrap();
                }
                if winvd::is_pinned_window(hwnd).unwrap() {
                    winvd::unpin_window(hwnd).unwrap();
                }
                if move_window {
                    winvd::move_window_to_desktop(winvd::get_current_desktop().unwrap(), &hwnd)
                        .unwrap();
                } else {
                    winvd::pin_window(hwnd).unwrap();
                }
                Some(element)
            }
            Err(_) => None,
        };
    } else {
        None
    }
    // })
}
