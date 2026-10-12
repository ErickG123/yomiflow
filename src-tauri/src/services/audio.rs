use crate::db::audio as audio_repository;
use crate::models::audio::{AudioPlayerSnapshot, AudioTrackMetadata};
use base64::Engine;
use id3::TagLike;
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
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

pub fn play_audio(path: String, runtime: &AudioRuntime) -> Result<AudioTrackMetadata, String> {
    let audio_path = Path::new(&path);
    if !audio_path.is_file() {
        return Err("Audio file does not exist.".to_owned());
    }
    let file = File::open(audio_path).map_err(|error| error.to_string())?;
    let decoder = Decoder::try_from(BufReader::new(file)).map_err(|error| error.to_string())?;
    let duration_ms = decoder
        .total_duration()
        .unwrap_or(Duration::ZERO)
        .as_millis() as u64;
    let metadata = extract_metadata(audio_path, duration_ms);
    let session_id = Uuid::new_v4().to_string();
    {
        let db = runtime.db.lock().map_err(|error| error.to_string())?;
        audio_repository::upsert_track(&db, &metadata)?;
        audio_repository::start_listening_session(&db, &session_id, &metadata)?;
    }

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
    player.session_id = Some(session_id);
    Ok(metadata)
}

pub fn pause_audio(runtime: &AudioRuntime) -> Result<(), String> {
    if let Some(sink) = runtime
        .player
        .lock()
        .map_err(|error| error.to_string())?
        .sink
        .as_ref()
    {
        sink.pause();
    }
    Ok(())
}

pub fn resume_audio(runtime: &AudioRuntime) -> Result<(), String> {
    if let Some(sink) = runtime
        .player
        .lock()
        .map_err(|error| error.to_string())?
        .sink
        .as_ref()
    {
        sink.play();
    }
    Ok(())
}

pub fn set_volume(volume: f32, runtime: &AudioRuntime) -> Result<(), String> {
    if !volume.is_finite() {
        return Err("Volume must be finite.".to_owned());
    }
    let mut player = runtime.player.lock().map_err(|error| error.to_string())?;
    player.volume = volume.clamp(0.0, 1.0);
    if let Some(sink) = &player.sink {
        sink.set_volume(player.volume);
    }
    Ok(())
}

pub fn seek_to(position_ms: u64, runtime: &AudioRuntime) -> Result<(), String> {
    if let Some(sink) = runtime
        .player
        .lock()
        .map_err(|error| error.to_string())?
        .sink
        .as_ref()
    {
        sink.try_seek(Duration::from_millis(position_ms))
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub fn player_snapshot(runtime: &AudioRuntime) -> Result<AudioPlayerSnapshot, String> {
    let player = runtime.player.lock().map_err(|error| error.to_string())?;
    let sink = player.sink.as_ref();
    Ok(AudioPlayerSnapshot {
        track: player.track.clone(),
        is_playing: sink.is_some_and(|sink| !sink.is_paused() && !sink.empty()),
        position_ms: sink.map_or(0, |sink| sink.get_pos().as_millis() as u64),
        volume: player.volume,
    })
}

pub async fn import_audio_folder(
    folder_path: String,
    db: Arc<Mutex<rusqlite::Connection>>,
) -> Result<Vec<AudioTrackMetadata>, String> {
    let root = PathBuf::from(folder_path);
    if !root.is_dir() {
        return Err("The selected audio folder does not exist or cannot be read.".to_owned());
    }
    tokio::task::spawn_blocking(move || {
        let mut paths = walkdir::WalkDir::new(&root)
            .follow_links(false)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| {
                        matches!(
                            extension.to_ascii_lowercase().as_str(),
                            "mp3" | "wav" | "m4a" | "flac" | "ogg"
                        )
                    })
            })
            .map(|entry| entry.path().to_path_buf())
            .collect::<Vec<_>>();
        paths.sort_by_key(|path| {
            path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase()
        });

        let album = root
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_owned();
        let tracks = paths
            .into_iter()
            .map(|path| {
                let absolute_path = path.canonicalize().unwrap_or(path.clone());
                let mut metadata = extract_metadata(&absolute_path, 0);
                if metadata.album.is_empty() {
                    metadata.album = album.clone();
                }
                metadata
            })
            .collect::<Vec<_>>();
        let mut connection = db.lock().map_err(|error| error.to_string())?;
        audio_repository::import_tracks(&mut connection, &tracks)?;
        Ok(tracks)
    })
    .await
    .map_err(|error| error.to_string())?
}

pub fn start_listening_tracker(runtime: Arc<AudioRuntime>) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(1));
            let active_session = runtime.player.lock().ok().and_then(|player| {
                player
                    .sink
                    .as_ref()
                    .is_some_and(|sink| !sink.is_paused() && !sink.empty())
                    .then(|| player.session_id.clone())
                    .flatten()
            });
            if let Some(session_id) = active_session
                && let Ok(connection) = runtime.db.lock()
            {
                let _ = audio_repository::increment_listened_time(&connection, &session_id);
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
        .unwrap_or_else(|| "Unknown track".to_owned());
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
