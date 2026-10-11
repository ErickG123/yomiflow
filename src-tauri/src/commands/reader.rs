use crate::models::MangaVolume;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use unrar::Archive;
use walkdir::WalkDir;
use zip::ZipArchive;

fn is_supported_image(name: &str) -> bool {
    let lower = name.to_lowercase();
    (lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".png")
        || lower.ends_with(".webp")
        || lower.ends_with(".gif"))
        && !lower.contains("__macos")
        && !lower.starts_with('.')
        && !lower.ends_with(".ds_store")
}

fn mime_type_from_filename(name: &str) -> &'static str {
    let lower = name.to_lowercase();
    if lower.ends_with(".png") {
        "image/png"
    } else if lower.ends_with(".webp") {
        "image/webp"
    } else if lower.ends_with(".gif") {
        "image/gif"
    } else {
        "image/jpeg"
    }
}

fn is_rar_or_cbr(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.ends_with(".rar") || lower.ends_with(".cbr")
}

// --- ZIP / CBZ IMPLEMENTATIONS ---

pub fn parse_zip_metadata_reader<R: Read + std::io::Seek>(
    reader: R,
) -> Result<Vec<String>, String> {
    let mut archive =
        ZipArchive::new(reader).map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

    let mut pages = Vec::new();

    for i in 0..archive.len() {
        let zip_file = archive
            .by_index(i)
            .map_err(|e| format!("Error accessing archive file at index {}: {}", i, e))?;

        if !zip_file.is_dir() {
            let name = zip_file.name().to_string();
            let base_name = Path::new(&name)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&name);

            if !base_name.starts_with('.') && is_supported_image(&name) {
                pages.push(name);
            }
        }
    }

    pages.sort();
    Ok(pages)
}

fn load_zip_metadata(path: &str) -> Result<Vec<String>, String> {
    let file = File::open(Path::new(path)).map_err(|e| format!("Failed to open file: {}", e))?;
    parse_zip_metadata_reader(file)
}

fn get_zip_page_bytes(path: &str, page_name: &str) -> Result<Vec<u8>, String> {
    let file = File::open(Path::new(path)).map_err(|e| format!("Failed to open file: {}", e))?;
    let mut archive =
        ZipArchive::new(file).map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

    let mut zip_file = archive
        .by_name(page_name)
        .map_err(|e| format!("Failed to find page '{}' in archive: {}", page_name, e))?;

    let mut buffer = Vec::new();
    zip_file
        .read_to_end(&mut buffer)
        .map_err(|e| format!("Failed to read page content: {}", e))?;

    Ok(buffer)
}

// --- RAR / CBR IMPLEMENTATIONS ---

fn load_rar_metadata(path: &str) -> Result<Vec<String>, String> {
    let mut archive = Archive::new(path)
        .open_for_listing()
        .map_err(|e| format!("Failed to open RAR archive: {}", e))?;

    let mut pages = Vec::new();

    while let Ok(Some(header)) = archive.read_header() {
        let is_dir = header.entry().is_directory();
        let filename = header.entry().filename.to_string_lossy().to_string();

        if !is_dir {
            let base_name = Path::new(&filename)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&filename);

            if !base_name.starts_with('.') && is_supported_image(&filename) {
                pages.push(filename);
            }
        }

        archive = header
            .skip()
            .map_err(|e| format!("Failed to advance RAR header: {}", e))?;
    }

    pages.sort();
    Ok(pages)
}

fn get_rar_page_bytes(path: &str, page_name: &str) -> Result<Vec<u8>, String> {
    let mut archive = Archive::new(path)
        .open_for_processing()
        .map_err(|e| format!("Failed to open RAR archive for processing: {}", e))?;

    while let Ok(Some(header)) = archive.read_header() {
        let filename = header.entry().filename.to_string_lossy().to_string();

        if filename == page_name {
            let (data, _) = header
                .read()
                .map_err(|e| format!("Failed to read RAR entry bytes: {}", e))?;
            return Ok(data);
        }

        archive = header
            .skip()
            .map_err(|e| format!("Failed to advance RAR entry: {}", e))?;
    }

    Err(format!("Page '{}' not found in RAR archive", page_name))
}

// --- TAURI EXPORTED COMMANDS ---

#[tauri::command]
pub async fn load_cbz_metadata(path: String) -> Result<Vec<String>, String> {
    let p = Path::new(&path);
    if !p.exists() || !p.is_file() {
        return Err(format!("Ficheiro inválido ou não encontrado: {}", path));
    }

    if is_rar_or_cbr(&path) {
        load_rar_metadata(&path)
    } else {
        load_zip_metadata(&path)
    }
}

#[tauri::command]
pub async fn get_cbz_page_base64(path: String, page_name: String) -> Result<String, String> {
    let p = Path::new(&path);
    if !p.exists() || !p.is_file() {
        return Err(format!("Ficheiro inválido ou não encontrado: {}", path));
    }

    // Guard against directory traversal in page_name
    if page_name.contains("..") || page_name.starts_with('/') || page_name.starts_with('\\') {
        return Err("Nome de página inválido".to_string());
    }

    let bytes = if is_rar_or_cbr(&path) {
        get_rar_page_bytes(&path, &page_name)?
    } else {
        get_zip_page_bytes(&path, &page_name)?
    };

    let mime = mime_type_from_filename(&page_name);
    let encoded = BASE64.encode(&bytes);

    Ok(format!("data:{};base64,{}", mime, encoded))
}

fn find_tesseract_cmd() -> std::process::Command {
    // Try standard "tesseract" first
    if std::process::Command::new("tesseract")
        .arg("-v")
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
    {
        return std::process::Command::new("tesseract");
    }

    // Common installation paths on Windows
    let known_paths = [
        r"C:\Program Files\Tesseract-OCR\tesseract.exe",
        r"C:\Program Files (x86)\Tesseract-OCR\tesseract.exe",
    ];

    for path in known_paths {
        if Path::new(path).exists() {
            return std::process::Command::new(path);
        }
    }

    // Default to "tesseract" and let Command fail with meaningful error
    std::process::Command::new("tesseract")
}

#[tauri::command]
pub async fn process_ocr(base64_image: String) -> Result<String, String> {
    // 1. Remove base64 prefix if present
    let raw_base64 = if let Some(idx) = base64_image.find(',') {
        &base64_image[idx + 1..]
    } else {
        &base64_image
    };

    // Decode base64 bytes
    let bytes = BASE64
        .decode(raw_base64.trim())
        .map_err(|e| format!("Failed to decode base64 image: {}", e))?;

    // 2. Load image from memory and apply preprocessing pipeline
    let original_img = image::load_from_memory(&bytes)
        .map_err(|e| format!("Failed to parse image for OCR: {}", e))?;

    // 2.1 Convert to grayscale (Luma8)
    let gray_img = original_img.to_luma8();
    let (orig_w, orig_h) = (gray_img.width(), gray_img.height());

    // 2.2 Add White Margins / Padding (40-50 pixels padding around the image)
    const PADDING: u32 = 45;
    let padded_w = orig_w + PADDING * 2;
    let padded_h = orig_h + PADDING * 2;

    // Create totally white background image (255 = white in Luma)
    let mut padded_img = image::ImageBuffer::from_pixel(padded_w, padded_h, image::Luma([255u8]));

    // 2.3 Overlay cropped grayscale image centered on the white background
    image::imageops::overlay(&mut padded_img, &gray_img, PADDING as i64, PADDING as i64);

    // 3. Write padded image to temporary PNG file
    let temp_file_path =
        std::env::temp_dir().join(format!("yomiflow_ocr_{}.png", uuid::Uuid::new_v4()));
    padded_img
        .save(&temp_file_path)
        .map_err(|e| format!("Failed to write preprocessed image file: {}", e))?;

    // 4. Execute Tesseract CLI synchronously: tesseract [path] stdout -l jpn_vert
    let mut cmd = find_tesseract_cmd();
    let output = cmd
        .arg(&temp_file_path)
        .arg("stdout")
        .arg("-l")
        .arg("jpn_vert")
        .output();

    // 5. Clean up temporary file regardless of OCR outcome
    let _ = std::fs::remove_file(&temp_file_path);

    let output = match output {
        Ok(out) => out,
        Err(e) => {
            return Err(format!(
                "Failed to run Tesseract OCR. Make sure 'tesseract' is installed and added to PATH: {}",
                e
            ));
        }
    };

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Tesseract failed with error: {}", err_msg.trim()));
    }

    // 6. Clean up output string (strip whitespace, spaces and newlines between characters)
    let raw_text = String::from_utf8_lossy(&output.stdout);
    let cleaned_text: String = raw_text
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '\n' && *c != '\r' && *c != ' ' && *c != '\u{3000}')
        .collect();

    Ok(cleaned_text)
}

#[tauri::command]
pub async fn scan_manga_library(root_path: String) -> Result<Vec<MangaVolume>, String> {
    let root = Path::new(&root_path);
    if !root.exists() || !root.is_dir() {
        return Err(format!("Diretório inválido ou inexistente: {}", root_path));
    }

    let mut volumes = Vec::new();

    for entry in WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            if ext == "cbz" || ext == "zip" || ext == "rar" || ext == "cbr" {
                let filename = path
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or("")
                    .to_string();

                // Skip hidden files like .DS_Store or files starting with '.'
                if filename.starts_with('.') {
                    continue;
                }

                // Extract the immediate parent directory name as manga title
                let title = path
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|f| f.to_str())
                    .unwrap_or("Unknown")
                    .to_string();

                let filepath = path.to_string_lossy().to_string();

                volumes.push(MangaVolume {
                    title,
                    filename,
                    filepath,
                });
            }
        }
    }

    // Sort volumes by title, then by filename
    volumes.sort_by(|a, b| {
        a.title
            .cmp(&b.title)
            .then_with(|| a.filename.cmp(&b.filename))
    });

    Ok(volumes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};
    use zip::write::{SimpleFileOptions, ZipWriter};

    #[test]
    fn test_parse_zip_metadata_reader_in_memory() {
        let mut buffer = Cursor::new(Vec::new());
        {
            let mut writer = ZipWriter::new(&mut buffer);
            let options = SimpleFileOptions::default();

            // Add valid image files
            writer.start_file("02_page.png", options).unwrap();
            writer.write_all(b"fake image data").unwrap();

            writer.start_file("01_cover.jpg", options).unwrap();
            writer.write_all(b"fake image data").unwrap();

            // Add non-image files / hidden files
            writer.start_file(".DS_Store", options).unwrap();
            writer.write_all(b"ignored").unwrap();

            writer.start_file("metadata.txt", options).unwrap();
            writer.write_all(b"ignored").unwrap();

            writer.start_file("__MACOSX/page.jpg", options).unwrap();
            writer.write_all(b"ignored").unwrap();

            writer.finish().unwrap();
        }

        buffer.set_position(0);
        let pages = parse_zip_metadata_reader(buffer).unwrap();

        assert_eq!(pages.len(), 2);
        assert_eq!(pages[0], "01_cover.jpg");
        assert_eq!(pages[1], "02_page.png");
    }

    #[test]
    fn test_is_supported_image() {
        assert!(is_supported_image("test.jpg"));
        assert!(is_supported_image("test.jpeg"));
        assert!(is_supported_image("test.png"));
        assert!(is_supported_image("test.webp"));
        assert!(!is_supported_image(".hidden.jpg"));
        assert!(!is_supported_image("readme.txt"));
        assert!(!is_supported_image("__MACOSX/image.png"));
    }
}
