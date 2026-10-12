use crate::AppState;
use crate::models::audio::{AudioPlayerSnapshot, AudioTrackMetadata, PassiveListeningEntry};
use crate::services::audio::AudioRuntime;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub fn play_audio(
    path: String,
    runtime: State<'_, Arc<AudioRuntime>>,
) -> Result<AudioTrackMetadata, String> {
    crate::services::audio::play_audio(path, &runtime)
}

#[tauri::command]
pub fn pause_audio(runtime: State<'_, Arc<AudioRuntime>>) -> Result<(), String> {
    crate::services::audio::pause_audio(&runtime)
}

#[tauri::command]
pub fn resume_audio(runtime: State<'_, Arc<AudioRuntime>>) -> Result<(), String> {
    crate::services::audio::resume_audio(&runtime)
}

#[tauri::command]
pub fn set_volume(volume: f32, runtime: State<'_, Arc<AudioRuntime>>) -> Result<(), String> {
    crate::services::audio::set_volume(volume, &runtime)
}

#[tauri::command]
pub fn seek_to(position_ms: u64, runtime: State<'_, Arc<AudioRuntime>>) -> Result<(), String> {
    crate::services::audio::seek_to(position_ms, &runtime)
}

#[tauri::command]
pub fn get_audio_player_snapshot(
    runtime: State<'_, Arc<AudioRuntime>>,
) -> Result<AudioPlayerSnapshot, String> {
    crate::services::audio::player_snapshot(&runtime)
}

#[tauri::command]
pub fn get_passive_listening_history(
    state: State<'_, AppState>,
) -> Result<Vec<PassiveListeningEntry>, String> {
    let connection = state.db.lock().map_err(|error| error.to_string())?;
    crate::db::audio::listening_history(&connection)
}

#[tauri::command]
pub fn get_audio_library(state: State<'_, AppState>) -> Result<Vec<AudioTrackMetadata>, String> {
    let connection = state.db.lock().map_err(|error| error.to_string())?;
    crate::db::audio::library(&connection)
}

#[tauri::command]
pub async fn import_audio_folder(
    folder_path: String,
    state: State<'_, AppState>,
) -> Result<Vec<AudioTrackMetadata>, String> {
    crate::services::audio::import_audio_folder(folder_path, state.db.clone()).await
}
