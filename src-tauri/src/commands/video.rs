use crate::models::SubtitleDialogue;
use std::path::Path;
use std::process::Command;

/// Checks if the FFmpeg CLI is installed and accessible in the system PATH
#[tauri::command]
pub async fn check_ffmpeg_installed() -> Result<bool, String> {
    let result = Command::new("ffmpeg").arg("-version").output();

    match result {
        Ok(out) => Ok(out.status.success()),
        Err(_) => Ok(false),
    }
}

/// Extracts embedded subtitle dialogues from an MKV, MP4, or external subtitle file using FFmpeg.
/// Automatically handles FFmpeg availability and formats SRT / WebVTT / ASS dialogues.
#[tauri::command]
pub async fn extract_subtitles(video_path: String) -> Result<Vec<SubtitleDialogue>, String> {
    let p = Path::new(&video_path);
    if !p.exists() || !p.is_file() {
        return Err(format!(
            "Ficheiro inválido ou não encontrado: {}",
            video_path
        ));
    }

    // If the file is already a subtitle file (.srt, .vtt, .ass), parse directly
    let ext = p
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if ext == "srt" || ext == "vtt" {
        let content = std::fs::read_to_string(p).map_err(|e| e.to_string())?;
        return Ok(parse_srt_or_vtt(&content));
    }

    // Try extracting first subtitle track using FFmpeg to stdout as WebVTT
    let output = Command::new("ffmpeg")
        .arg("-i")
        .arg(&video_path)
        .arg("-map")
        .arg("0:s:0") // first subtitle track
        .arg("-f")
        .arg("webvtt")
        .arg("-")
        .output();

    match output {
        Ok(out) => {
            if out.status.success() {
                let vtt_text = String::from_utf8_lossy(&out.stdout);
                let dialogues = parse_srt_or_vtt(&vtt_text);
                if !dialogues.is_empty() {
                    return Ok(dialogues);
                }
            }

            // Fallback: Check if there's a sidecar .srt or .ass with same name
            let srt_sidecar = p.with_extension("srt");
            if srt_sidecar.exists()
                && let Ok(c) = std::fs::read_to_string(srt_sidecar)
            {
                return Ok(parse_srt_or_vtt(&c));
            }

            let vtt_sidecar = p.with_extension("vtt");
            if vtt_sidecar.exists()
                && let Ok(c) = std::fs::read_to_string(vtt_sidecar)
            {
                return Ok(parse_srt_or_vtt(&c));
            }

            Ok(Vec::new())
        }
        Err(e) => {
            // If ffmpeg is not installed, look for sidecar subtitle files (.srt/.vtt)
            let srt_sidecar = p.with_extension("srt");
            if srt_sidecar.exists()
                && let Ok(c) = std::fs::read_to_string(srt_sidecar)
            {
                return Ok(parse_srt_or_vtt(&c));
            }

            let vtt_sidecar = p.with_extension("vtt");
            if vtt_sidecar.exists()
                && let Ok(c) = std::fs::read_to_string(vtt_sidecar)
            {
                return Ok(parse_srt_or_vtt(&c));
            }

            Err(format!(
                "FFmpeg não foi encontrado no sistema ou falhou ao extrair legendas: {}. Dica: instale o FFmpeg ou coloque um ficheiro .srt ao lado do vídeo.",
                e
            ))
        }
    }
}

/// Parses WebVTT or SRT timestamps and texts
fn parse_srt_or_vtt(content: &str) -> Vec<SubtitleDialogue> {
    let mut dialogues = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Check for timestamp line like:
        // "00:01:20.000 --> 00:01:23.500" or "00:01:20,000 --> 00:01:23,500"
        if line.contains("-->") {
            let parts: Vec<&str> = line.split("-->").collect();
            if parts.len() == 2 {
                let start_time = parse_timestamp(parts[0].trim());
                // Handle optional VTT positioning attributes after end time
                let end_str = parts[1].split_whitespace().next().unwrap_or("");
                let end_time = parse_timestamp(end_str);

                i += 1;
                let mut text_lines = Vec::new();
                while i < lines.len() && !lines[i].trim().is_empty() {
                    let text_line = lines[i].trim();
                    // Strip HTML/ASS styling tags like <i>, <font>, {\an8}
                    let cleaned = strip_subtitle_tags(text_line);
                    if !cleaned.is_empty() {
                        text_lines.push(cleaned);
                    }
                    i += 1;
                }

                let text = text_lines.join(" ");
                if !text.is_empty() {
                    let next_id = dialogues.len() + 1;
                    dialogues.push(SubtitleDialogue {
                        id: next_id,
                        start_time,
                        end_time,
                        text,
                    });
                }
            }
        }
        i += 1;
    }

    dialogues
}

/// Cuts an audio clip from a video file between start_time and end_time,
/// saves it in the application's SRS audio media directory, and returns the absolute output file path.
#[tauri::command]
pub async fn cut_audio_clip(
    app_handle: tauri::AppHandle,
    video_path: String,
    start_time: f32,
    end_time: f32,
    output_filename: String,
) -> Result<String, String> {
    use tauri::Manager;

    let video_p = Path::new(&video_path);
    if !video_p.exists() || !video_p.is_file() {
        return Err(format!(
            "Ficheiro de vídeo inválido ou não encontrado: {}",
            video_path
        ));
    }

    if end_time <= start_time {
        return Err("O tempo de fim deve ser superior ao tempo de início".to_string());
    }

    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Erro ao obter pasta de dados da aplicação: {}", e))?;

    let media_dir = app_data_dir.join("media");
    if !media_dir.exists() {
        std::fs::create_dir_all(&media_dir)
            .map_err(|e| format!("Erro ao criar pasta de mídia: {}", e))?;
    }

    // Sanitize output_filename to prevent path traversal vulnerabilities
    let clean_base = Path::new(&output_filename)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("clip")
        .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");

    let filename = if clean_base.ends_with(".mp3") {
        clean_base
    } else {
        format!("{}.mp3", clean_base)
    };

    let out_path = media_dir.join(&filename);
    let duration = end_time - start_time;

    // Execute ffmpeg to cut audio: -ss start -i video -t duration -vn -acodec libmp3lame -q:a 2 out.mp3
    let status = Command::new("ffmpeg")
        .arg("-y") // overwrite output file if exists
        .arg("-ss")
        .arg(format!("{:.3}", start_time))
        .arg("-i")
        .arg(&video_path)
        .arg("-t")
        .arg(format!("{:.3}", duration))
        .arg("-vn") // no video stream
        .arg("-acodec")
        .arg("libmp3lame")
        .arg("-q:a")
        .arg("2")
        .arg(&out_path)
        .status()
        .map_err(|e| format!("Falha ao executar FFmpeg: {}. Certifique-se de que o FFmpeg está instalado no PATH do sistema.", e))?;

    if !status.success() {
        return Err("O FFmpeg falhou ao recortar o áudio do vídeo".to_string());
    }

    Ok(out_path.to_string_lossy().to_string())
}

fn parse_timestamp(ts: &str) -> f64 {
    // Standard timestamp formats: HH:MM:SS.mmm or MM:SS.mmm or with comma
    let cleaned = ts.replace(',', ".");
    let parts: Vec<&str> = cleaned.split(':').collect();

    match parts.len() {
        3 => {
            let h = parts[0].parse::<f64>().unwrap_or(0.0);
            let m = parts[1].parse::<f64>().unwrap_or(0.0);
            let s = parts[2].parse::<f64>().unwrap_or(0.0);
            h * 3600.0 + m * 60.0 + s
        }
        2 => {
            let m = parts[0].parse::<f64>().unwrap_or(0.0);
            let s = parts[1].parse::<f64>().unwrap_or(0.0);
            m * 60.0 + s
        }
        _ => parts[0].parse::<f64>().unwrap_or(0.0),
    }
}

fn strip_subtitle_tags(raw: &str) -> String {
    let mut result = String::new();
    let mut in_html_tag = false;
    let mut in_ass_tag = false;
    for c in raw.chars() {
        if c == '<' {
            in_html_tag = true;
        } else if c == '>' && in_html_tag {
            in_html_tag = false;
        } else if c == '{' {
            in_ass_tag = true;
        } else if c == '}' && in_ass_tag {
            in_ass_tag = false;
        } else if !in_html_tag && !in_ass_tag {
            result.push(c);
        }
    }

    result.trim().to_string()
}
