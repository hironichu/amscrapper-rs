use crate::music::{AMusicSongInfo, AMusicState, AMusicTimeInfo};
use anyhow::Result;
use regex::Regex;

use uiautomation::UIAutomation;
use uiautomation::UIElement;
use uiautomation::patterns::UIRangeValuePattern;
use uiautomation::types::PropertyConditionFlags;
use uiautomation::variants::Variant;

impl Drop for AMusicScraper {
    fn drop(&mut self) {
        self.automation = None;
        self.window = None;
        self.amsongpanel = None;
        self.amsong_field_panel = None;
    }
}
#[derive(Default, Debug, Clone)]
pub struct AMusicScraper {
    automation: Option<UIAutomation>,
    window: Option<UIElement>,
    amsongpanel: Option<UIElement>,
    amsong_field_panel: Option<UIElement>,
}

unsafe impl Send for AMusicScraper {}
unsafe impl Sync for AMusicScraper {}

impl AMusicScraper {
    pub fn new(automation: UIAutomation, window: UIElement) -> Result<Self, anyhow::Error> {
        let mut _self = Self {
            automation: Some(automation),
            window: Some(window),
            amsongpanel: None,
            amsong_field_panel: None,
        };
        _self.init_elements()?;
        Ok(_self)
    }

    pub fn init_elements(&mut self) -> Result<(), anyhow::Error> {
        if self.window.is_none() {
            return Err(anyhow::Error::msg("No window defined"));
        }
        let window = self.window.as_ref().unwrap();

        let automation = self.automation.as_ref().unwrap();
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
            return Err(anyhow::Error::msg("No song panel"));
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
            return Err(anyhow::Error::msg("No song field panel"));
        }
        let amsong_field_panel = amsong_field_panel.unwrap();
        self.amsong_field_panel = Some(amsong_field_panel);
        self.amsongpanel = Some(amsongpanel);
        Ok(())
    }

    pub fn update_song(&self) -> Option<AMusicSongInfo> {
        //check if the window is still open
        if self.window.is_none() {
            return None;
        }
        let winid = self.window.as_ref().unwrap().get_process_id();
        if winid.is_err() {
            return None;
        }
        //try to get the window from the process id
        let hwnd = winid.unwrap();
        let hwnd = hwnd as isize;
        let hwnd = uiautomation::types::Handle::from(hwnd);
        let automation = self.automation.as_ref().unwrap();
        let window = automation.element_from_handle(hwnd);
        if window.is_err() {
            return None;
        }
        if self.amsong_field_panel.is_none() {
            return None;
        }
        let amsong_field_panel = self.amsong_field_panel.as_ref().unwrap();
        let automation = self.automation.as_ref().unwrap();
        let song_fields = amsong_field_panel.find_all(
            uiautomation::types::TreeScope::Descendants,
            &automation
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
        //check if song_fields contains at least 2 elements
        if song_fields.len() < 2 {
            return None;
        }
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

    pub fn parse_artist_and_album(
        &self,
        song_name: &str,
        song_album_artist: &str,
        composer_as_artist: bool,
    ) -> Option<AMusicSongInfo> {
        let song_split: Vec<&str> = song_album_artist.split(" \u{2014} ").collect();
        let artist: String;
        let album: String;
        let song: String = song_name.into();

        let composer_performer_regex = Regex::new(r"By (.+) \u2014 (.+) \u2014 (.+)").unwrap();
        let matches = composer_performer_regex.captures(song_album_artist);
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
    pub fn update_time(&self) -> Option<AMusicTimeInfo> {
        if self.amsong_field_panel.is_none() {
            return None;
        }
        let amsong_field_panel = self.amsong_field_panel.as_ref().unwrap();
        let automation = self.automation.as_ref().unwrap();
        let current_time_element = amsong_field_panel.find_first(
            uiautomation::types::TreeScope::Children,
            &automation
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
            &automation
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
            &automation
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
    pub fn update_live(&self) -> bool {
        if self.amsong_field_panel.is_none() {
            return false;
        }
        let amsong_field_panel = self.amsong_field_panel.as_ref().unwrap();
        let automation = self.automation.as_ref().unwrap();
        let check = amsong_field_panel.find_first(
            uiautomation::types::TreeScope::Children,
            &automation
                .create_property_condition(
                    uiautomation::types::UIProperty::Name,
                    Variant::from("LIVE"),
                    None,
                )
                .unwrap(),
        );

        check.is_ok()
    }
    pub fn update_status(&self) -> Option<AMusicState> {
        if self.amsongpanel.is_none() {
            return None;
        }
        let amsongpanel = self.amsongpanel.as_ref().unwrap();
        let automation = self.automation.as_ref().unwrap();
        let play_pause_btn = amsongpanel.find_first(
            uiautomation::types::TreeScope::Descendants,
            &automation
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
    pub fn update_data(
        &self,
    ) -> (
        Option<AMusicState>,
        Option<AMusicSongInfo>,
        Option<AMusicTimeInfo>,
    ) {
        (self.update_status(), self.update_song(), self.update_time())
    }
}
