#[allow(static_mut_refs)]
use regex::Regex;
use std::sync::{Arc, Mutex};
use uiautomation::UIElement;
use uiautomation::{UIAutomation, types::Handle};
use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumChildWindows, EnumThreadWindows, GetDesktopWindow, RealGetWindowClassA,
};

use windows::Win32::UI::WindowsAndMessaging::{
    GWL_STYLE, GetWindowLongPtrW, IsWindowVisible, WS_VISIBLE,
};

use winvd::{DesktopEvent, get_desktop_count, listen_desktop_events, switch_desktop};
#[derive(Debug, Clone)]
struct AMusicInfo {
    song: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    duration: Option<i32>,
    current_time: Option<i32>,
    composer_performer_regex: Regex,
}

impl AMusicInfo {
    pub fn new() -> Self {
        Self {
            composer_performer_regex: Regex::new(r"By (.+) \u2014 (.+) \u2014 (.+)").unwrap(),
            song: None,
            artist: None,
            album: None,
            duration: None,
            current_time: None,
        }
    }
    pub fn parse_artist_and_album(mut self, song: &str, composer_as_artist: bool) {
        let song_split: Vec<&str> = song.split(" \u{2014} ").collect();

        let matches = self.composer_performer_regex.captures(song);
        if let Some(captures) = matches {
            let song_composer = captures.get(1).unwrap().as_str();
            let song_performer = captures.get(2).unwrap().as_str();
            self.artist = if composer_as_artist {
                Some(song_composer.into())
            } else {
                Some(song_performer.into())
            };
            self.album = Some(captures.get(3).unwrap().as_str().into());
            return;
        }
        if song_split.len() > 1 {
            self.artist = Some(song_split[0].into());
            self.album = Some(song_split[1].into());
        } else {
            self.artist = Some(song_split[0].into());
            self.album = Some(song_split[0].into());
        }
    }
    pub fn parse_time_string(mut self, time: String) {
        if time.is_empty() {
            return;
        }
        //remove leading "-"
        let mut ptime = time.clone();
        if time.contains("-") {
            ptime = ptime.split("-").collect::<Vec<&str>>()[1].to_string();
        }
        let ctime = ptime.split(":").collect::<Vec<&str>>();
        let min = ctime[0].parse::<i32>().unwrap();
        let sec = ctime[1].parse::<i32>().unwrap();
        let total = min * 60 + sec;
        self.current_time = Some(total);
    }
    // private static int? ParseTimeString(string? time) {

    //     if (time == null) {
    //         return null;
    //     }

    //     // remove leading "-"
    //     if (time.Contains('-')) {
    //         time = time.Split('-')[1];
    //     }

    //     int min = int.Parse(time.Split(":")[0]);
    //     int sec = int.Parse(time.Split(":")[1]);

    //     return min * 60 + sec;
    // }
}
fn find_app_hwnd() -> Option<Vec<HWND>> {
    let storage: GetTargetType = Arc::new(Mutex::new(None));
    let l_param = LPARAM(&storage as *const GetTargetType as isize);

    unsafe {
        let _ = EnumThreadWindows(0, Some(find_target_process), l_param);
        // let desktop_hwnd = GetDesktopWindow();
        // let _ = EnumChildWindows(desktop_hwnd, Some(find_target_process), l_param);
    }

    let state = storage.lock().unwrap().take();
    state
}
// make a Vec where we will store the hwnds of the target apps
type GetTargetType = Arc<Mutex<Option<Vec<HWND>>>>;

extern "system" fn find_target_process(hwnd: HWND, l_param: LPARAM) -> BOOL {
    let mut buffer = [0_u8; 128];
    let read_len = unsafe { RealGetWindowClassA(hwnd, &mut buffer) };
    let proc_name = String::from_utf8_lossy(&buffer[..read_len as usize]);

    // if proc_name == "Microsoft.UI.Content.DesktopChildSiteBridge" {
    // }
    // let desk = winvd::get_desktop_by_window(hwnd);
    // match desk {
    //     Ok(desk) => {
    //         println!("Desktop: {:?}", desk);
    //     }
    //     Err(_) => {
    //         // println!("Error: {:?}", err);
    //     }
    // }
    println!("Which desktop : {:?}", winvd::get_desktop_by_window(hwnd));
    let storage = unsafe { &*(l_param.0 as *const GetTargetType) };
    let mut storage = storage.lock().unwrap();
    storage.get_or_insert_with(|| Vec::new());
    storage.as_mut().unwrap().push(hwnd);
    // return BOOL(1);

    BOOL(1)
}
// const TARGET_APP_TITLE: &str = "AppleMusic";
fn main() {
    // let desktop = winvd::get_current_desktop().unwrap();
    // println!("Current Desktop: {:?}", desktop);
    let app_hwn =
        find_app_hwnd().expect("target app handle is not found. make sure the app is running");
    //loop through that
    let automation = UIAutomation::new().unwrap();
    let cache = automation.create_cache_request().unwrap();
    let root = automation.get_root_element_build_cache(&cache).unwrap();
    println!("Root Name : {:?}", root.get_name());
    println!("Root Class : {:?}", root.get_classname());
    println!("Count : {:?}", app_hwn.len());
    let apple_music: Option<UIElement> = app_hwn.iter().find_map(|hwnd| {
        let desk = winvd::get_desktop_by_window(*hwnd);
        let element = automation
            .element_from_handle(Handle::from(hwnd.0 as isize))
            .expect("Couldnt find handle");
        let apple_music_window_filter = automation
            .create_matcher()
            .from(element)
            .timeout(200)
            .contains_name("Apple Music")
            .find_first()
            .expect("Could not find any window with Apple Music");

        let apple_music = apple_music_window_filter;

        match desk {
            Ok(_) => {
                if winvd::is_window_on_current_desktop(*hwnd).unwrap() {
                    return Some(apple_music);
                }
                if winvd::is_pinned_app(*hwnd).unwrap() {
                    winvd::unpin_app(*hwnd).unwrap();
                }
                if winvd::is_pinned_window(*hwnd).unwrap() {
                    winvd::unpin_window(*hwnd).unwrap();
                }
                winvd::pin_app(*hwnd).unwrap();
                None
            }
            Err(_) => None,
        }
    });
    // app_hwn.iter().for_each(|hwnd| {
    //     let element = automation
    //         .element_from_handle(Handle::from(hwnd.0))
    //         .unwrap();
    //     println!("Elem Name : {:?}", element.get_name());
    //     let matcher = automation.create_matcher().from(element).timeout(1000);
    //     // .name("Apple Music");
    //     matcher.find_all().unwrap().iter().for_each(|elem| {
    //         println!("Name {}", elem.get_name().unwrap());
    //         println!("Class {}", elem.get_classname().unwrap());
    //     });
    //     // .classname("WinUIDesktopWin32WindowClass");
    //     // match matcher.find_first() {
    //     //     Ok(elem) => {
    //     //         println!("Found Apple Music");
    //     //         println!("Name {}", elem.get_name().unwrap());
    //     //         println!("Class {}", elem.get_classname().unwrap());
    //     //         Some(elem)
    //     //     }
    //     //     Err(e) => None,
    //     // }
    //     // None
    // });
    // return;
    // let matcher = automation
    //     .create_matcher()
    //     .from(app.unwrap())
    //     .timeout(2500)
    //     .classname("NamedContainerAutomationPeer");

    // if let Ok(amsong_panel) = matcher.find_first() {
    //     let childrens = automation
    //         .create_matcher()
    //         .from(amsong_panel)
    //         .classname("TextBlock")
    //         .find_all();
    //     match childrens {
    //         Ok(childrens) => {
    //             if childrens.len() <= 0 {
    //                 panic!("No container");
    //             }

    //             for child in childrens {
    //                 let song = child.get_name().unwrap();
    //                 println!("Song: {}", song);
    //             }
    //             // let song = childrens[0].get_name().unwrap();
    //             // let artist_and_album = childrens[1].get_name().unwrap();
    //             // song_info.set_song(&song.clone());
    //             // song_info.parse_artist_and_album(&artist_and_album.clone(), false);
    //             // println!("Song: {}", song_info.song);
    //         }
    //         Err(e) => {
    //             println!("No song playing");
    //             return;
    //         }
    //     }
    // } else {
    //     println!("No song panel");
    //     return;
    // }
    // let storage: GetTargetType = Arc::new(Mutex::new(None));
    // unsafe {
    //     let l_param = LPARAM(&storage as *const GetTargetType as isize);
    //     let desktop_hwnd = GetDesktopWindow();
    //     let test = windows::Win32::UI::WindowsAndMessaging::EnumWindows(desktop_hwnd, lparam);
    // }
    // assert_ne!(app_hwn, HWND::default(), "app handle must not be empty");
    // println!("handle for '{}' is {:?}", TARGET_APP_TITLE, app_hwn);
    // let proc_list = winprocinfo::get_list().expect("Failed to retrieve process list");
    // let process_name = "AppleMusic.exe";
    // println!("\nSearch by process name: {}", process_name);
    // let procs = proc_list.search_by_name(process_name)[0];

    // println!("Found PID: {}", procs.unique_process_id);
    // procs.threads.iter().for_each(|thread| {
    //     println!("Thread ID: {}", thread.);
    // });
    // let mut test = unsafe {
    // windows::Win32::System::Threading::GetProcessHandleCount(procs.unique_process_id);
    // };
    // println!("Desktops: {:?}", get_desktop_count().unwrap());
    // let (tx, rx) = std::sync::mpsc::channel::<DesktopEvent>();
    // let _notifications_thread = listen_desktop_events(tx);
    // std::thread::spawn(|| {
    //     for item in rx {
    //         println!("{:?}", item);
    //     }
    // });
    // switch_desktop(1).unwrap();
}
