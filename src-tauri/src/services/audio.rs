use crate::db::audio as audio_repository;
use crate::models::audio::{AudioPlayerSnapshot, AudioTrackMetadata};
use base64::Engine;
use id3::TagLike;
use rodio::{Decoder, OutputStreamBuilder, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender, SyncSender};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;

pub struct AudioRuntime {
    commands: Sender<AudioCommand>,
    db: Arc<Mutex<rusqlite::Connection>>,
}

enum AudioCommand {
    Play {
        path: PathBuf,
        track: AudioTrackMetadata,
        session_id: String,
        response: SyncSender<Result<(), String>>,
    },
    Pause(SyncSender<Result<(), String>>),
    Resume(SyncSender<Result<(), String>>),
    SetVolume(f32, SyncSender<Result<(), String>>),
    Seek(Duration, SyncSender<Result<(), String>>),
    Snapshot(SyncSender<AudioPlayerSnapshot>),
}

impl AudioRuntime {
    pub fn new(db: Arc<Mutex<rusqlite::Connection>>) -> Result<Self, String> {
        let (commands, receiver) = mpsc::channel();
        let snapshot = Arc::new(Mutex::new(AudioPlayerSnapshot {
            track: None,
            is_playing: false,
            position_ms: 0,
            volume: 0.75,
        }));
        let thread_snapshot = snapshot.clone();
        let thread_db = db.clone();
        std::thread::Builder::new()
            .name("yomiflow-audio".to_owned())
            .spawn(move || audio_thread(receiver, thread_db, thread_snapshot))
            .map_err(|error| error.to_string())?;

        Ok(Self { commands, db })
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

    drop(decoder);
    let (response, result) = mpsc::sync_channel(0);
    runtime
        .commands
        .send(AudioCommand::Play {
            path: audio_path.to_path_buf(),
            track: metadata.clone(),
            session_id,
            response,
        })
        .map_err(|error| error.to_string())?;
    result.recv().map_err(|error| error.to_string())??;
    Ok(metadata)
}

pub fn pause_audio(runtime: &AudioRuntime) -> Result<(), String> {
    send_audio_command(runtime, AudioCommand::Pause)
}

pub fn resume_audio(runtime: &AudioRuntime) -> Result<(), String> {
    send_audio_command(runtime, AudioCommand::Resume)
}

pub fn set_volume(volume: f32, runtime: &AudioRuntime) -> Result<(), String> {
    if !volume.is_finite() {
        return Err("Volume must be finite.".to_owned());
    }
    let (response, result) = mpsc::sync_channel(0);
    runtime
        .commands
        .send(AudioCommand::SetVolume(volume.clamp(0.0, 1.0), response))
        .map_err(|error| error.to_string())?;
    result.recv().map_err(|error| error.to_string())?
}

pub fn seek_to(position_ms: u64, runtime: &AudioRuntime) -> Result<(), String> {
    let (response, result) = mpsc::sync_channel(0);
    runtime
        .commands
        .send(AudioCommand::Seek(
            Duration::from_millis(position_ms),
            response,
        ))
        .map_err(|error| error.to_string())?;
    result.recv().map_err(|error| error.to_string())?
}

pub fn player_snapshot(runtime: &AudioRuntime) -> Result<AudioPlayerSnapshot, String> {
    let (response, result) = mpsc::sync_channel(0);
    runtime
        .commands
        .send(AudioCommand::Snapshot(response))
        .map_err(|error| error.to_string())?;
    result.recv().map_err(|error| error.to_string())
}

fn send_audio_command(
    runtime: &AudioRuntime,
    command: impl FnOnce(SyncSender<Result<(), String>>) -> AudioCommand,
) -> Result<(), String> {
    let (response, result) = mpsc::sync_channel(0);
    runtime
        .commands
        .send(command(response))
        .map_err(|error| error.to_string())?;
    result.recv().map_err(|error| error.to_string())?
}

fn audio_thread(
    receiver: Receiver<AudioCommand>,
    db: Arc<Mutex<rusqlite::Connection>>,
    snapshot: Arc<Mutex<AudioPlayerSnapshot>>,
) {
    // CoreAudio streams are thread-affine and deliberately stay on this worker.
    let stream = OutputStreamBuilder::open_default_stream().ok();
    let mut sink: Option<Sink> = None;
    let mut track: Option<AudioTrackMetadata> = None;
    let mut session_id: Option<String> = None;
    let mut volume = 0.75;

    loop {
        match receiver.recv_timeout(Duration::from_secs(1)) {
            Ok(AudioCommand::Play {
                path,
                track: next_track,
                session_id: next_session_id,
                response,
            }) => {
                let result = (|| {
                    let stream = stream
                        .as_ref()
                        .ok_or_else(|| "Audio output device is unavailable.".to_owned())?;
                    let file = File::open(&path).map_err(|error| error.to_string())?;
                    let decoder = Decoder::try_from(BufReader::new(file))
                        .map_err(|error| error.to_string())?;
                    if let Some(previous_sink) = sink.take() {
                        previous_sink.stop();
                    }
                    let next_sink = Sink::connect_new(stream.mixer());
                    next_sink.set_volume(volume);
                    next_sink.append(decoder);
                    next_sink.play();
                    sink = Some(next_sink);
                    track = Some(next_track);
                    session_id = Some(next_session_id);
                    Ok(())
                })();
                let _ = response.send(result);
            }
            Ok(AudioCommand::Pause(response)) => {
                if let Some(sink) = &sink {
                    sink.pause();
                }
                let _ = response.send(Ok(()));
            }
            Ok(AudioCommand::Resume(response)) => {
                if let Some(sink) = &sink {
                    sink.play();
                }
                let _ = response.send(Ok(()));
            }
            Ok(AudioCommand::SetVolume(next_volume, response)) => {
                volume = next_volume;
                if let Some(sink) = &sink {
                    sink.set_volume(volume);
                }
                let _ = response.send(Ok(()));
            }
            Ok(AudioCommand::Seek(position, response)) => {
                let result = sink.as_ref().map_or(Ok(()), |sink| {
                    sink.try_seek(position).map_err(|error| error.to_string())
                });
                let _ = response.send(result);
            }
            Ok(AudioCommand::Snapshot(response)) => {
                update_snapshot(&snapshot, sink.as_ref(), track.as_ref(), volume);
                let current = snapshot.lock().map(|state| state.clone());
                if let Ok(current) = current {
                    let _ = response.send(current);
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if let (Some(sink), Some(session_id)) = (&sink, &session_id)
                    && !sink.is_paused()
                    && !sink.empty()
                    && let Ok(connection) = db.lock()
                {
                    let _ = audio_repository::increment_listened_time(&connection, session_id);
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
        update_snapshot(&snapshot, sink.as_ref(), track.as_ref(), volume);
    }
}

fn update_snapshot(
    snapshot: &Mutex<AudioPlayerSnapshot>,
    sink: Option<&Sink>,
    track: Option<&AudioTrackMetadata>,
    volume: f32,
) {
    if let Ok(mut state) = snapshot.lock() {
        state.track = track.cloned();
        state.is_playing = sink.is_some_and(|sink| !sink.is_paused() && !sink.empty());
        state.position_ms = sink.map_or(0, |sink| sink.get_pos().as_millis() as u64);
        state.volume = volume;
    }
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
