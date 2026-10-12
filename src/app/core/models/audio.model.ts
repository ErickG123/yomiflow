export interface AudioTrackMetadata {
  path: string
  title: string
  artist: string
  album: string
  cover_data_url: string | null
  duration_ms: number
}

export interface AudioPlayerSnapshot {
  track: AudioTrackMetadata | null
  is_playing: boolean
  position_ms: number
  volume: number
}

export interface PassiveListeningEntry {
  id: string
  track_path: string
  title: string
  artist: string
  listened_ms: number
  started_at: string
  updated_at: string
}
