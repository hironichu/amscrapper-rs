mod scrapper;
mod utils;
use scrapper::AMusicScraper;
use uiautomation::UIAutomation;

fn main() {
    #[cfg(not(target_os = "windows"))]
    compile_error!("Only Windows is supported");

    let automation = UIAutomation::new().unwrap();
    let apple_music = utils::grab_applemusic_window(&automation);
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
