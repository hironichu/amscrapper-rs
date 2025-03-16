use std::sync::{Arc, Mutex};
use uiautomation::types::Handle;
use uiautomation::{UIAutomation, UIElement};
use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::EnumThreadWindows;

static mut AUTOMATION: Option<UIAutomation> = None;

pub(crate) type GetTargetType = Arc<Mutex<Option<HWND>>>;
pub(crate) fn find_app_hwnd(automation: &UIAutomation) -> Option<HWND> {
    let storage: GetTargetType = Arc::new(Mutex::new(None));
    let l_param = LPARAM(&storage as *const GetTargetType as isize);
    unsafe {
        AUTOMATION = Some(automation.clone());
        let _ = EnumThreadWindows(0, Some(find_target_process), l_param);
    }

    let state = storage.lock().unwrap().take();
    state
}

#[allow(static_mut_refs)]
extern "system" fn find_target_process(hwnd: HWND, l_param: LPARAM) -> BOOL {
    let storage = unsafe { &*(l_param.0 as *const GetTargetType) };
    let mut storage = storage.lock().unwrap();
    if unsafe { AUTOMATION.is_none() } {
        return BOOL(1);
    }
    let automation = unsafe { AUTOMATION.as_ref().unwrap() };

    let found = automation
        .element_from_handle(Handle::from(hwnd.0 as isize))
        .unwrap()
        .find_first(
            uiautomation::types::TreeScope::Element,
            &automation
                .create_property_condition(
                    uiautomation::types::UIProperty::Name,
                    uiautomation::variants::Variant::from("Apple Music"),
                    None,
                )
                .unwrap(),
        );
    if found.is_ok() {
        storage.replace(hwnd);
        return BOOL(0);
    }
    BOOL(1)
}

pub(crate) fn grab_applemusic_window(
    automation: &UIAutomation,
    movewindow: bool,
) -> Option<UIElement> {
    let app_hwn = find_app_hwnd(automation)
        .expect("target app handle is not found. make sure the app is running");

    let element = automation.element_from_handle(Handle::from(app_hwn.0 as isize));
    if element.is_err() {
        return None;
    }
    let element = element.unwrap();
    if element.get_name().unwrap() == "Apple Music" {
        let desk = winvd::get_desktop_by_window(app_hwn);
        return match desk {
            Ok(_) => {
                if winvd::is_window_on_current_desktop(app_hwn).unwrap() {
                    return Some(element);
                }
                if winvd::is_pinned_app(app_hwn).unwrap() {
                    winvd::unpin_app(app_hwn).unwrap();
                }
                if winvd::is_pinned_window(app_hwn).unwrap() {
                    winvd::unpin_window(app_hwn).unwrap();
                }
                if !movewindow {
                    winvd::move_window_to_desktop(winvd::get_current_desktop().unwrap(), &app_hwn)
                        .unwrap();
                } else {
                    winvd::pin_window(app_hwn).unwrap();
                }
                Some(element)
            }
            Err(_) => None,
        };
    } else {
        None
    }
}
