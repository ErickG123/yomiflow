use crate::AppState;
use crate::models::audio::{AudioPlayerSnapshot, AudioTrackMetadata, PassiveListeningEntry};
use base64::Engine;
use id3::TagLike;
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};
use rusqlite::params;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::State;
use uuid::Uuid;

pub struct AudioRuntime {
    pub player: Mutex<AudioPlayer>,
    pub db: Arc<Mutex<rusqlite::Connection>>,
}

#[derive(Default)]
pub struct AudioPlayer {
    stream: Option<OutputStream>,
    sink: Option<Sink>,
    track: Option<AudioTrackMetadata>,
    session_id: Option<String>,
    volume: f32,
}

impl AudioRuntime {
    pub fn new(db: Arc<Mutex<rusqlite::Connection>>) -> Self {
        Self {
            player: Mutex::new(AudioPlayer {
                volume: 0.75,
                ..AudioPlayer::default()
            }),
            db,
        }
    }
}

#[tauri::command]
pub fn play_audio(
    path: String,
    runtime: State<'_, Arc<AudioRuntime>>,
) -> Result<AudioTrackMetadata, String> {
    let audio_path = Path::new(&path);
    if !audio_path.is_file() {
        return Err("O ficheiro de áudio não existe.".into());
    }

    let file = File::open(audio_path).map_err(|error| error.to_string())?;
    let decoder = Decoder::try_from(BufReader::new(file)).map_err(|error| error.to_string())?;
    let duration_ms = decoder
        .total_duration()
        .unwrap_or(Duration::ZERO)
        .as_millis() as u64;
    let metadata = extract_metadata(audio_path, duration_ms);
    let mut player = runtime.player.lock().map_err(|error| error.to_string())?;
    if let Some(sink) = player.sink.take() {
        sink.stop();
    }

    let stream = OutputStreamBuilder::open_default_stream().map_err(|error| error.to_string())?;
    let sink = Sink::connect_new(stream.mixer());
    sink.set_volume(player.volume);
    sink.append(decoder);
    sink.play();
    player.stream = Some(stream);
    player.sink = Some(sink);
    player.track = Some(metadata.clone());

    let session_id = Uuid::new_v4().to_string();
    let db = runtime.db.lock().map_err(|error| error.to_string())?;
    db.execute(
            "INSERT INTO audio_library (path, title, artist, album, cover_data_url, duration_ms) VALUES (?1, ?2, ?3, ?4, ?5, ?6) ON CONFLICT(path) DO UPDATE SET title = excluded.title, artist = excluded.artist, album = excluded.album, cover_data_url = excluded.cover_data_url, duration_ms = excluded.duration_ms",
            params![metadata.path, metadata.title, metadata.artist, metadata.album, metadata.cover_data_url, metadata.duration_ms],
        ).map_err(|error| error.to_string())?;
    db.execute(
        "INSERT INTO passive_listening_log (id, track_path, title, artist) VALUES (?1, ?2, ?3, ?4)",
        params![session_id, metadata.path, metadata.title, metadata.artist],
    )
    .map_err(|error| error.to_string())?;
    player.session_id = Some(session_id);
    Ok(metadata)
}

#[tauri::command]
pub fn pause_audio(runtime: State<'_, Arc<AudioRuntime>>) -> Result<(), String> {
    let player = runtime.player.lock().map_err(|error| error.to_string())?;
    if let Some(sink) = &player.sink {
        sink.pause();
    }
    Ok(())
}

#[tauri::command]
pub fn resume_audio(runtime: State<'_, Arc<AudioRuntime>>) -> Result<(), String> {
    let player = runtime.player.lock().map_err(|error| error.to_string())?;
    if let Some(sink) = &player.sink {
        sink.play();
    }
    Ok(())
}

#[tauri::command]
pub fn set_volume(volume: f32, runtime: State<'_, Arc<AudioRuntime>>) -> Result<(), String> {
    if !volume.is_finite() {
        return Err("Volume inválido.".into());
    }
    let volume = volume.clamp(0.0, 1.0);
    let mut player = runtime.player.lock().map_err(|error| error.to_string())?;
    player.volume = volume;
    if let Some(sink) = &player.sink {
        sink.set_volume(volume);
    }
    Ok(())
}

#[tauri::command]
pub fn seek_to(position_ms: u64, runtime: State<'_, Arc<AudioRuntime>>) -> Result<(), String> {
    let player = runtime.player.lock().map_err(|error| error.to_string())?;
    if let Some(sink) = &player.sink {
        sink.try_seek(Duration::from_millis(position_ms))
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_audio_player_snapshot(
    runtime: State<'_, Arc<AudioRuntime>>,
) -> Result<AudioPlayerSnapshot, String> {
    let player = runtime.player.lock().map_err(|error| error.to_string())?;
    let sink = player.sink.as_ref();
    Ok(AudioPlayerSnapshot {
        track: player.track.clone(),
        is_playing: sink.is_some_and(|sink| !sink.is_paused() && !sink.empty()),
        position_ms: sink.map_or(0, |sink| sink.get_pos().as_millis() as u64),
        volume: player.volume,
    })
}

#[tauri::command]
pub fn get_passive_listening_history(
    state: State<'_, AppState>,
) -> Result<Vec<PassiveListeningEntry>, String> {
    let db = state.db.lock().map_err(|error| error.to_string())?;
    let mut statement = db.prepare(
        "SELECT id, track_path, title, artist, listened_ms, started_at, updated_at FROM passive_listening_log ORDER BY updated_at DESC LIMIT 100",
    ).map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok(PassiveListeningEntry {
                id: row.get(0)?,
                track_path: row.get(1)?,
                title: row.get(2)?,
                artist: row.get(3)?,
                listened_ms: row.get(4)?,
                started_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_audio_library(state: State<'_, AppState>) -> Result<Vec<AudioTrackMetadata>, String> {
    let db = state.db.lock().map_err(|error| error.to_string())?;
    let mut statement = db.prepare("SELECT path, title, artist, album, cover_data_url, duration_ms FROM audio_library ORDER BY imported_at DESC").map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok(AudioTrackMetadata {
                path: row.get(0)?,
                title: row.get(1)?,
                artist: row.get(2)?,
                album: row.get(3)?,
                cover_data_url: row.get(4)?,
                duration_ms: row.get(5)?,
            })
        })
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn start_listening_tracker(runtime: Arc<AudioRuntime>) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(1));
            let active_session = runtime.player.lock().ok().and_then(|player| {
                let is_playing = player
                    .sink
                    .as_ref()
                    .is_some_and(|sink| !sink.is_paused() && !sink.empty());
                is_playing.then(|| player.session_id.clone()).flatten()
            });
            if let Some(session_id) = active_session
                && let Ok(db) = runtime.db.lock()
            {
                let _ = db.execute(
                "UPDATE passive_listening_log SET listened_ms = listened_ms + 1000, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
                [session_id],
            );
            }
        }
    });
}

fn extract_metadata(path: &Path, duration_ms: u64) -> AudioTrackMetadata {
    let tag = id3::Tag::read_from_path(path).ok();
    let title = tag
        .as_ref()
        .and_then(TagLike::title)
        .map(str::to_owned)
        .or_else(|| {
            path.file_stem()
                .and_then(|name| name.to_str())
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "Unknown track".into());
    let artist = tag
        .as_ref()
        .and_then(TagLike::artist)
        .unwrap_or_default()
        .to_owned();
    let album = tag
        .as_ref()
        .and_then(TagLike::album)
        .unwrap_or_default()
        .to_owned();
    let cover_data_url = tag
        .as_ref()
        .and_then(|tag| tag.pictures().next())
        .map(|picture| {
            let encoded = base64::engine::general_purpose::STANDARD.encode(&picture.data);
            format!("data:{};base64,{encoded}", picture.mime_type)
        });
    AudioTrackMetadata {
        path: path.to_string_lossy().into_owned(),
        title,
        artist,
        album,
        cover_data_url,
        duration_ms,
    }
}
