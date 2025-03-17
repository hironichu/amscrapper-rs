use crate::{music, scrapper, utils};
use anyhow::Result;
use music::{AMusicSongInfo, AMusicState, AMusicTimeInfo};
use scrapper::AMusicScraper;
use uiautomation::UIAutomation;

pub fn init(move_window: bool) -> Result<AMusicScraper> {
    #[cfg(not(target_os = "windows"))]
    compile_error!("Only Windows is supported");

    let automation = UIAutomation::new().unwrap();
    let apple_music = utils::grab_applemusic_window(&automation, move_window);
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
    if status.is_some() {
        let status = status.unwrap();
        if status.playing {
            return Some((song_info.unwrap(), status, time_info.unwrap()));
        }
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
