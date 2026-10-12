use crate::models::audio::{AudioTrackMetadata, PassiveListeningEntry};
use rusqlite::{Connection, params};

pub fn upsert_track(connection: &Connection, track: &AudioTrackMetadata) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO audio_library (path, title, artist, album, cover_data_url, duration_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(path) DO UPDATE SET title = excluded.title, artist = excluded.artist,
                album = excluded.album, cover_data_url = excluded.cover_data_url,
                duration_ms = excluded.duration_ms",
            params![
                track.path,
                track.title,
                track.artist,
                track.album,
                track.cover_data_url,
                track.duration_ms
            ],
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn start_listening_session(
    connection: &Connection,
    id: &str,
    track: &AudioTrackMetadata,
) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO passive_listening_log (id, track_path, title, artist) VALUES (?1, ?2, ?3, ?4)",
            params![id, track.path, track.title, track.artist],
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn listening_history(connection: &Connection) -> Result<Vec<PassiveListeningEntry>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, track_path, title, artist, listened_ms, started_at, updated_at
             FROM passive_listening_log ORDER BY updated_at DESC LIMIT 100",
        )
        .map_err(|error| error.to_string())?;
    statement
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
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn library(connection: &Connection) -> Result<Vec<AudioTrackMetadata>, String> {
    let mut statement = connection
        .prepare(
            "SELECT path, title, artist, album, cover_data_url, duration_ms
             FROM audio_library ORDER BY imported_at DESC",
        )
        .map_err(|error| error.to_string())?;
    statement
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
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn import_tracks(
    connection: &mut Connection,
    tracks: &[AudioTrackMetadata],
) -> Result<(), String> {
    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    for track in tracks {
        transaction
            .execute(
                "INSERT INTO audio_library (path, title, artist, album, cover_data_url, duration_ms)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(path) DO UPDATE SET title = excluded.title, artist = excluded.artist,
                    album = excluded.album, cover_data_url = excluded.cover_data_url,
                    duration_ms = excluded.duration_ms",
                params![
                    track.path,
                    track.title,
                    track.artist,
                    track.album,
                    track.cover_data_url,
                    track.duration_ms
                ],
            )
            .map_err(|error| error.to_string())?;
    }
    transaction.commit().map_err(|error| error.to_string())
}

pub fn increment_listened_time(connection: &Connection, session_id: &str) -> Result<(), String> {
    connection
        .execute(
            "UPDATE passive_listening_log
             SET listened_ms = listened_ms + 1000, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
            params![session_id],
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn track(path: &str) -> AudioTrackMetadata {
        AudioTrackMetadata {
            path: path.to_owned(),
            title: path.to_owned(),
            artist: "Artist".to_owned(),
            album: "Album".to_owned(),
            cover_data_url: None,
            duration_ms: 1_000,
        }
    }

    #[test]
    fn audio_import_persists_tracks_transactionally_and_history_accumulates_time() {
        let mut connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(
                "CREATE TABLE audio_library (
                    path TEXT PRIMARY KEY, title TEXT, artist TEXT, album TEXT,
                    cover_data_url TEXT, duration_ms INTEGER, imported_at TEXT DEFAULT CURRENT_TIMESTAMP
                );
                CREATE TABLE passive_listening_log (
                    id TEXT PRIMARY KEY, track_path TEXT, title TEXT, artist TEXT,
                    listened_ms INTEGER DEFAULT 0, started_at TEXT DEFAULT CURRENT_TIMESTAMP,
                    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
                );",
            )
            .unwrap();
        import_tracks(&mut connection, &[track("a.mp3"), track("b.mp3")]).unwrap();
        start_listening_session(&connection, "session-1", &track("a.mp3")).unwrap();
        increment_listened_time(&connection, "session-1").unwrap();

        let total_tracks: i64 = connection
            .query_row("SELECT COUNT(*) FROM audio_library", [], |row| row.get(0))
            .unwrap();
        let listened_ms: i64 = connection
            .query_row(
                "SELECT listened_ms FROM passive_listening_log WHERE id = ?1",
                params!["session-1"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(total_tracks, 2);
        assert_eq!(listened_ms, 1_000);
    }
}
