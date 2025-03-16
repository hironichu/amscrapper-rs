use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AMusicSongInfo {
    pub song: String,
    pub artist: String,
    pub album: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AMusicTimeInfo {
    pub duration: i32,
    pub remaining_duration: i32,
    pub current_time: i32,
    pub total: i32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AMusicState {
    pub playing: bool,
    pub live: bool,
}
impl AMusicState {}

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
