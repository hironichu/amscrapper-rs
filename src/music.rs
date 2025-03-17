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
