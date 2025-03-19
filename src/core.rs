use crate::{
    music, scrapper,
    utils::{self},
};
use anyhow::Result;
use music::{AMusicSongInfo, AMusicState, AMusicTimeInfo};
use scrapper::AMusicScraper;
use uiautomation::UIAutomation;

pub fn init(move_window: bool) -> Result<AMusicScraper> {
    let automation = UIAutomation::new().unwrap();
    let apple_music = utils::grab_window(&automation, move_window);
    if apple_music.is_none() {
        return Err(anyhow::anyhow!("No Apple Music window found"));
    }
    let window = apple_music.unwrap();

    AMusicScraper::new(automation, window)
}

pub fn playing(scrapper: &AMusicScraper) -> Option<(AMusicSongInfo, AMusicState, AMusicTimeInfo)> {
    let data = scrapper.update_data();
    let song_info = data.1;
    let status = data.0;
    let time_info = data.2;
    if status.is_some() && song_info.is_some() && time_info.is_some() {
        let status = status.unwrap();
        let song_info = song_info.unwrap();
        let time_info = time_info.unwrap();
        return Some((song_info, status, time_info));
    }
    None
}

pub fn update_song(scrapper: &AMusicScraper) -> Result<AMusicSongInfo> {
    let song = scrapper.update_song();
    if song.is_none() {
        return Err(anyhow::anyhow!("No song info found"));
    }
    Ok(song.unwrap())
}
