use std::sync::{Arc, Mutex};
use uiautomation::types::Handle;
use uiautomation::{UIAutomation, UIElement};
use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowTextA};

pub const APPLE_MUSIC: &str = "Apple Music";
pub const SPOTIFY: &str = "Spotify";
pub const SPOTIFY_FREE: &str = "Spotify Free";

pub(crate) type GetTargetType = Arc<Mutex<(i32, Option<HWND>)>>;

pub(crate) fn find_app_hwnd(selected: i32) -> Option<HWND> {
    let storage: GetTargetType = Arc::new(Mutex::new((selected, None)));

    let l_param = LPARAM(&storage as *const GetTargetType as isize);

    unsafe {
        let _ = EnumWindows(Some(find_target_process), l_param);
    }

    let state = storage.lock().unwrap().1;
    state
}
extern "system" fn find_target_process(hwnd: HWND, l_param: LPARAM) -> BOOL {
    let mut buffer = [0_u8; 128];
    let read_len = unsafe { GetWindowTextA(hwnd, &mut buffer) };
    let proc_name = String::from_utf8_lossy(&buffer[..read_len as usize]);
    let storage = unsafe { &*(l_param.0 as *const GetTargetType) };
    let mut storage = storage.lock().unwrap();
    let strelectred = match storage.0 {
        1 => SPOTIFY,
        2 => APPLE_MUSIC,
        3 => SPOTIFY_FREE,
        _ => return BOOL(0),
    };

    if proc_name.as_ref().trim() == strelectred.to_string() {
        storage.1.replace(hwnd);
        return BOOL(0);
    }
    //unlock the mutex

    BOOL(1)
}
/// This function is used to grab the Apple Music window
/// It also checks if the window is on the current desktop, if not it moves it to the current desktop
/// If the window is pinned, it unpins it
pub fn grab_window(name: &str, automation: &UIAutomation, move_window: bool) -> Option<UIElement> {
    let intselected = match name {
        SPOTIFY => 1,
        APPLE_MUSIC => 2,
        SPOTIFY_FREE => 3,
        _ => 0,
    };
    let hwnd = find_app_hwnd(intselected)
        .expect("target app handle is not found. make sure the app is running");
    let element = automation.element_from_handle(Handle::from(hwnd.0 as isize));
    if element.is_err() {
        return None;
    }
    let element = element.unwrap();
    if element.get_name().unwrap() == name {
        let desk = winvd::get_desktop_by_window(hwnd);

        return match desk {
            Ok(_) => {
                //only do the following for Apple Music
                if intselected == 2 {
                    if winvd::is_window_on_current_desktop(hwnd).unwrap() {
                        return Some(element);
                    }
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
    }
    None
}
