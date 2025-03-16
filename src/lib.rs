mod scrapper;
mod utils;
use anyhow::Result;
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

pub fn update_data(scrapper: &AMusicScraper) {
    scrapper.update_data();
}

pub fn update_song(scrapper: &AMusicScraper) -> Result<scrapper::AMusicSongInfo> {
    let song = scrapper.update_song();
    if song.is_none() {
        return Err(anyhow::anyhow!("No song info found"));
    }
    Ok(song.unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        let scrapper = init(false).unwrap();
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
}
