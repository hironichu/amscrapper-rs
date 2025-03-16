use regex::Regex;
use std::sync::{Arc, Mutex};
use uiautomation::UIElement;
use uiautomation::patterns::UIRangeValuePattern;
use uiautomation::types::PropertyConditionFlags;
use uiautomation::variants::Variant;
use uiautomation::{UIAutomation, types::Handle};
use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::EnumThreadWindows;

type GetTargetType = Arc<Mutex<Option<Vec<HWND>>>>;

#[derive(Debug, Clone)]
struct AMusicSongInfo {
    song: String,
    artist: String,
    album: String,
}
#[derive(Debug, Clone)]
struct AMusicTimeInfo {
    duration: i32,
    remaining_duration: i32,
    current_time: i32,
    total: i32,
}
#[derive(Debug, Clone)]
struct AMusicState {
    playing: bool,
    live: bool,
}
impl AMusicState {}
impl Drop for AMusicScraper {
    fn drop(&mut self) {
        self.composer_performer_regex = Regex::new("").unwrap();
        self.automation = UIAutomation::new().unwrap();
        self.window = None;
        self.amsongpanel = None;
        self.amsong_field_panel = None;
    }
}
impl Drop for AMusicSongInfo {
    fn drop(&mut self) {
        self.song = String::new();
        self.artist = String::new();
        self.album = String::new();
    }
}

impl Drop for AMusicTimeInfo {
    fn drop(&mut self) {
        self.duration = 0;
        self.remaining_duration = 0;
        self.current_time = 0;
        self.total = 0;
    }
}
impl Drop for AMusicState {
    fn drop(&mut self) {
        self.playing = false;
        self.live = false;
    }
}
#[derive(Debug, Clone)]
struct AMusicScraper {
    composer_performer_regex: Regex,
    automation: UIAutomation,
    window: Option<UIElement>,
    amsongpanel: Option<UIElement>,
    amsong_field_panel: Option<UIElement>,
}

impl AMusicScraper {
    pub fn new(automation: UIAutomation, window: UIElement) -> Self {
        let mut _self = Self {
            composer_performer_regex: Regex::new(r"By (.+) \u2014 (.+) \u2014 (.+)").unwrap(),
            automation,
            window: Some(window),
            amsongpanel: None,
            amsong_field_panel: None,
        };
        _self.init_elements();
        _self
    }

    fn init_elements(&mut self) {
        if self.window.is_none() {
            return;
        }
        let window = self.window.as_ref().unwrap();
        let automation = self.automation.clone();
        let amsongpanel = window.find_first(
            uiautomation::types::TreeScope::Descendants,
            &automation
                .create_property_condition(
                    uiautomation::types::UIProperty::AutomationId,
                    Variant::from("TransportBar"),
                    None,
                )
                .unwrap(),
        );
        if amsongpanel.is_err() {
            println!("No song panel found");
            return;
        }
        let amsongpanel = amsongpanel.unwrap();

        let amsong_field_panel = amsongpanel.find_first(
            uiautomation::types::TreeScope::Children,
            &automation
                .create_property_condition(
                    uiautomation::types::UIProperty::AutomationId,
                    Variant::from("LCD"),
                    None,
                )
                .unwrap(),
        );
        if amsong_field_panel.is_err() {
            println!("No song field panel found");
            return;
        }
        let amsong_field_panel = amsong_field_panel.unwrap();
        self.amsong_field_panel = Some(amsong_field_panel);
        self.amsongpanel = Some(amsongpanel);
    }

    fn update_song(&self) -> Option<AMusicSongInfo> {
        if self.amsong_field_panel.is_none() {
            return None;
        }
        let amsong_field_panel = self.amsong_field_panel.clone().unwrap();
        let song_fields = amsong_field_panel.find_all(
            uiautomation::types::TreeScope::Descendants,
            &self
                .automation
                .create_property_condition(
                    uiautomation::types::UIProperty::AutomationId,
                    Variant::from("myScrollViewer"),
                    None,
                )
                .unwrap(),
        );
        if song_fields.is_err() {
            return None;
        }
        let song_fields = song_fields.unwrap();

        let mut song_name_element = song_fields[0].clone();
        let mut song_album_artist_element = song_fields[1].clone();

        if song_name_element
            .get_bounding_rectangle()
            .unwrap()
            .get_bottom()
            > song_album_artist_element
                .get_bounding_rectangle()
                .unwrap()
                .get_bottom()
        {
            song_name_element = song_fields[1].clone();
            song_album_artist_element = song_fields[0].clone();
        }

        let song_name = song_name_element.get_name();
        let song_album_artist = song_album_artist_element.get_name();
        if song_name.is_err() || song_album_artist.is_err() {
            return None;
        }
        let song_name = song_name.unwrap();

        self.parse_artist_and_album(&song_name, &song_album_artist.unwrap(), false)
    }

    fn parse_artist_and_album(
        &self,
        song_name: &str,
        song_album_artist: &str,
        composer_as_artist: bool,
    ) -> Option<AMusicSongInfo> {
        let song_split: Vec<&str> = song_album_artist.split(" \u{2014} ").collect();
        let artist: String;
        let album: String;
        let song: String = song_name.into();
        let matches = self.composer_performer_regex.captures(song_album_artist);
        if let Some(captures) = matches {
            let song_composer = captures.get(1).unwrap().as_str();
            let song_performer = captures.get(2).unwrap().as_str();
            artist = if composer_as_artist {
                song_composer.into()
            } else {
                song_performer.into()
            };
            album = captures.get(3).unwrap().as_str().into();
            return Some(AMusicSongInfo {
                song,
                artist,
                album,
            });
        }
        if song_split.len() > 1 {
            artist = song_split[0].into();
            album = song_split[1].into();
        } else {
            artist = song_split[0].into();
            album = song_split[0].into();
        }
        Some(AMusicSongInfo {
            song,
            artist,
            album,
        })
    }
    fn update_time(&self) -> Option<AMusicTimeInfo> {
        if self.amsong_field_panel.is_none() {
            return None;
        }
        let amsong_field_panel = self.amsong_field_panel.clone().unwrap();
        let current_time_element = amsong_field_panel.find_first(
            uiautomation::types::TreeScope::Children,
            &self
                .automation
                .create_property_condition(
                    uiautomation::types::UIProperty::AutomationId,
                    Variant::from("CurrentTime"),
                    Some(PropertyConditionFlags::All),
                )
                .unwrap(),
        );
        let mut current_time;
        let mut remaining_duration;
        let mut total;
        if current_time_element.is_err() {
            current_time = 0;
        } else {
            let current_timeelem = current_time_element.unwrap().get_name().unwrap();

            let mut current_time_split = current_timeelem.split(":");
            let min = current_time_split.next().unwrap().parse::<i32>().unwrap();
            let sec = current_time_split.next().unwrap().parse::<i32>().unwrap();
            current_time = min * 60 + sec;
        }
        let remaining_duration_element = amsong_field_panel.find_first(
            uiautomation::types::TreeScope::Children,
            &self
                .automation
                .create_property_condition(
                    uiautomation::types::UIProperty::AutomationId,
                    Variant::from("Duration"),
                    Some(PropertyConditionFlags::All),
                )
                .unwrap(),
        );
        if remaining_duration_element.is_err() {
            remaining_duration = 0;
        } else {
            let remaining_durationelem = remaining_duration_element.unwrap().get_name().unwrap();

            let mut duration_split = remaining_durationelem.split(":");
            let min = duration_split.next().unwrap().parse::<i32>().unwrap();
            let sec = duration_split.next().unwrap().parse::<i32>().unwrap();
            remaining_duration = min * 60 + sec;
        }
        total = current_time + remaining_duration;

        let lcd_scrubber = amsong_field_panel.find_first(
            uiautomation::types::TreeScope::Descendants,
            &self
                .automation
                .create_property_condition(
                    uiautomation::types::UIProperty::AutomationId,
                    Variant::from("LCDScrubber"),
                    None,
                )
                .unwrap(),
        );

        if lcd_scrubber.is_ok() {
            let scrubber_pos = lcd_scrubber.unwrap();
            let pattern: UIRangeValuePattern = scrubber_pos.get_pattern().unwrap();

            let slider_max = pattern.get_maximum();
            let slider_min = pattern.get_minimum();
            let slider_val = pattern.get_value();
            if slider_max.is_err() || slider_min.is_err() || slider_val.is_err() {
                //
            } else {
                let slider_val = slider_val.unwrap();
                let slider_max = slider_max.unwrap();

                if current_time == 0 {
                    current_time = slider_val.round() as i32;
                }
                if remaining_duration == 0 {
                    remaining_duration = slider_max.round() as i32 - current_time;
                }
                if total == 0 {
                    total = remaining_duration + current_time;
                }
            }
        }

        Some(AMusicTimeInfo {
            duration: total,
            remaining_duration,
            current_time,
            total,
        })
    }
    fn update_live(&self) -> bool {
        if self.amsong_field_panel.is_none() {
            return false;
        }
        let amsong_field_panel = self.amsong_field_panel.clone().unwrap();
        let check = amsong_field_panel.find_first(
            uiautomation::types::TreeScope::Children,
            &self
                .automation
                .create_property_condition(
                    uiautomation::types::UIProperty::Name,
                    Variant::from("LIVE"),
                    None,
                )
                .unwrap(),
        );

        check.is_ok()
    }
    fn update_status(&self) -> Option<AMusicState> {
        if self.amsongpanel.is_none() {
            println!("No manel");
            return None;
        }
        let amsongpanel = self.amsongpanel.clone().unwrap();
        let play_pause_btn = amsongpanel.find_first(
            uiautomation::types::TreeScope::Descendants,
            &self
                .automation
                .create_property_condition(
                    uiautomation::types::UIProperty::AutomationId,
                    Variant::from("TransportControl_PlayPauseStop"),
                    None,
                )
                .unwrap(),
        );
        if play_pause_btn.is_err() {
            return None;
        }

        let play_pause_btn = play_pause_btn.unwrap();

        Some(AMusicState {
            playing: play_pause_btn.get_name().unwrap() == "Pause",
            live: self.update_live(),
        })
    }

    pub fn update_data(&self) {
        self.update_song();
        self.update_time();
        self.update_live();
        self.update_status();
    }
}
fn find_app_hwnd() -> Option<Vec<HWND>> {
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

fn grab_applemusic_window(automation: &UIAutomation) -> Option<UIElement> {
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

fn main() {
    #[cfg(not(target_os = "windows"))]
    compile_error!("Only Windows is supported");

    let automation = UIAutomation::new().unwrap();
    let apple_music = grab_applemusic_window(&automation);
    if apple_music.is_none() {
        println!("No Apple Music window found");
        return;
    }
    let window = apple_music.unwrap();

    let scrapper = AMusicScraper::new(automation, window);
    scrapper.update_data();
    let song_info = scrapper.update_song().unwrap();
    let status = scrapper.update_status().unwrap();
    if status.playing {
        let timeinfo = scrapper.update_time().unwrap();
        println!("Artist: {}", song_info.artist);
        println!("Name : {}", song_info.song);
        println!("Album: {}", song_info.album);
        println!("Duration: {}", timeinfo.duration);
        println!("Remaining Duration: {}", timeinfo.remaining_duration);
        println!("Current Time: {}", timeinfo.current_time);
    } else {
        println!("Paused");
    }
}
