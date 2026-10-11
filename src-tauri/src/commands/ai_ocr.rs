use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use futures_util::StreamExt;
use image::imageops::FilterType;
use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use ort::value::Tensor;
use std::fs::File;
use std::io::Write;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, Window};
use tokenizers::Tokenizer;

const MODEL_FILENAME: &str = "manga-ocr.onnx";
const TOKENIZER_FILENAME: &str = "tokenizer.json";

// Public URLs for MangaOCR ONNX model and tokenizer
const MODEL_DOWNLOAD_URL: &str =
    "https://huggingface.co/kha-white/manga-ocr-base/resolve/main/onnx/model.onnx";
const TOKENIZER_DOWNLOAD_URL: &str =
    "https://huggingface.co/kha-white/manga-ocr-base/resolve/main/tokenizer.json";

// In-memory cache for loaded Session and Tokenizer to prevent reload latency per crop
static MODEL_SESSION: Mutex<Option<Session>> = Mutex::new(None);
static TOKENIZER: Mutex<Option<Tokenizer>> = Mutex::new(None);

#[tauri::command]
pub async fn check_model_exists(app_handle: AppHandle) -> Result<bool, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Falha ao resolver pasta de dados da aplicação: {}", e))?;

    let model_path = app_data_dir.join(MODEL_FILENAME);
    let tokenizer_path = app_data_dir.join(TOKENIZER_FILENAME);

    Ok(model_path.exists() && tokenizer_path.exists())
}

#[tauri::command]
pub async fn download_ai_model(window: Window, app_handle: AppHandle) -> Result<(), String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Falha ao resolver pasta de dados da aplicação: {}", e))?;

    if !app_data_dir.exists() {
        std::fs::create_dir_all(&app_data_dir)
            .map_err(|e| format!("Falha ao criar diretório da aplicação: {}", e))?;
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| e.to_string())?;

    // 1. Download tokenizer if missing
    let tokenizer_path = app_data_dir.join(TOKENIZER_FILENAME);
    if !tokenizer_path.exists() {
        let tokenizer_bytes = client
            .get(TOKENIZER_DOWNLOAD_URL)
            .header("User-Agent", "YomiFlow-App")
            .send()
            .await
            .map_err(|e| format!("Erro ao descarregar tokenizer: {}", e))?
            .bytes()
            .await
            .map_err(|e| e.to_string())?;

        std::fs::write(&tokenizer_path, tokenizer_bytes)
            .map_err(|e| format!("Falha ao gravar tokenizer.json: {}", e))?;
    }

    // 2. Download ONNX Model with real-time stream progress
    let model_path = app_data_dir.join(MODEL_FILENAME);
    let temp_model_path = app_data_dir.join(format!("{}.part", MODEL_FILENAME));

    let response = client
        .get(MODEL_DOWNLOAD_URL)
        .header("User-Agent", "YomiFlow-App")
        .send()
        .await
        .map_err(|e| format!("Erro ao contactar servidor do modelo: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Servidor retornou código de erro HTTP: {}",
            response.status()
        ));
    }

    let total_size = response.content_length().unwrap_or(400_000_000);
    let mut file = File::create(&temp_model_path)
        .map_err(|e| format!("Falha ao criar ficheiro temporário: {}", e))?;

    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;
    let mut last_emitted_percent: u32 = 0;

    let _ = window.emit("model-download-progress", 0u32);

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| format!("Erro no stream de download: {}", e))?;
        file.write_all(&chunk)
            .map_err(|e| format!("Erro ao gravar dados no disco: {}", e))?;

        downloaded += chunk.len() as u64;
        let percent = (((downloaded as f64) / (total_size as f64)) * 100.0).clamp(0.0, 100.0) as u32;

        if percent != last_emitted_percent {
            last_emitted_percent = percent;
            let _ = window.emit("model-download-progress", percent);
        }
    }

    file.flush()
        .map_err(|e| format!("Erro ao finalizar gravação do ficheiro: {}", e))?;
    drop(file);

    if model_path.exists() {
        let _ = std::fs::remove_file(&model_path);
    }
    std::fs::rename(&temp_model_path, &model_path)
        .map_err(|e| format!("Falha ao renomear ficheiro final do modelo: {}", e))?;

    let _ = window.emit("model-download-progress", 100u32);
    let _ = window.emit("model-download-complete", ());

    Ok(())
}

/// Runs inference on the cropped speech bubble image using MangaOCR ONNX
#[tauri::command]
pub async fn process_image_with_ai(
    base64_image: String,
    app_handle: AppHandle,
) -> Result<String, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Falha ao resolver pasta de dados: {}", e))?;

    let model_path = app_data_dir.join(MODEL_FILENAME);
    let tokenizer_path = app_data_dir.join(TOKENIZER_FILENAME);

    if !model_path.exists() {
        return Err("O modelo MangaOCR ainda não foi transferido.".to_string());
    }

    // 1. Decode base64 bytes
    let raw_base64 = if let Some(idx) = base64_image.find(',') {
        &base64_image[idx + 1..]
    } else {
        &base64_image
    };

    let bytes = BASE64
        .decode(raw_base64.trim())
        .map_err(|e| format!("Erro ao descodificar imagem base64: {}", e))?;

    // 2. Load image and preprocess for MangaOCR vision backbone
    let dynamic_img = image::load_from_memory(&bytes)
        .map_err(|e| format!("Erro ao carregar imagem: {}", e))?;

    // Standard Vision Transformer input shape for MangaOCR: (1, 3, height, width) with height=384, width=384
    let target_h = 384u32;
    let target_w = 384u32;

    let rgb_img = dynamic_img
        .resize_exact(target_w, target_h, FilterType::Triangle)
        .to_rgb8();

    // 3. Build normalized 4D tensor flat buffer (1 * 3 * 384 * 384)
    // Mean: [0.5, 0.5, 0.5], Std: [0.5, 0.5, 0.5] => (pixel / 255.0 - 0.5) / 0.5 = pixel / 127.5 - 1.0
    let mut flat_data = vec![0.0f32; 3 * target_h as usize * target_w as usize];
    let hw = target_h as usize * target_w as usize;

    for (x, y, pixel) in rgb_img.enumerate_pixels() {
        let (r, g, b) = (pixel[0] as f32, pixel[1] as f32, pixel[2] as f32);
        let idx = (y as usize) * (target_w as usize) + (x as usize);
        flat_data[idx] = (r / 255.0 - 0.5) / 0.5;
        flat_data[hw + idx] = (g / 255.0 - 0.5) / 0.5;
        flat_data[2 * hw + idx] = (b / 255.0 - 0.5) / 0.5;
    }

    // 4. Initialize / reuse ONNX Session
    let mut session_guard = MODEL_SESSION.lock().map_err(|e| e.to_string())?;
    if session_guard.is_none() {
        let mut builder = Session::builder().map_err(|e| e.to_string())?;
        builder = builder
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| e.to_string())?;
        let session = builder
            .commit_from_file(&model_path)
            .map_err(|e| format!("Falha ao carregar modelo ONNX: {}", e))?;
        *session_guard = Some(session);
    }
    let session = session_guard.as_mut().unwrap();

    // 5. Initialize / reuse Tokenizer
    let mut tok_guard = TOKENIZER.lock().map_err(|e| e.to_string())?;
    if tok_guard.is_none() {
        let tok = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| format!("Falha ao carregar tokenizer.json: {}", e))?;
        *tok_guard = Some(tok);
    }
    let tokenizer = tok_guard.as_ref().unwrap();

    // 6. Execute inference
    let shape = [1usize, 3, target_h as usize, target_w as usize];
    let input_tensor = Tensor::from_array((shape, flat_data))
        .map_err(|e| format!("Erro ao criar tensor de entrada: {}", e))?;

    let outputs = session
        .run(ort::inputs![input_tensor])
        .map_err(|e| format!("Erro na inferência ONNX: {}", e))?;

    // Extract output token IDs tensor
    let output_value = outputs
        .into_iter()
        .next()
        .ok_or_else(|| "Nenhuma saída retornada pelo modelo".to_string())?;

    // Extract token IDs and decode with tokenizer
    let extracted_text = if let Ok(extracted) = output_value.1.try_extract_tensor::<i64>() {
        let token_ids: Vec<u32> = extracted
            .1
            .iter()
            .filter(|&&id| id > 0)
            .map(|&id| id as u32)
            .collect();

        tokenizer
            .decode(&token_ids, true)
            .map_err(|e| format!("Erro ao descodificar tokens: {}", e))?
    } else if let Ok(extracted) = output_value.1.try_extract_tensor::<i32>() {
        let token_ids: Vec<u32> = extracted
            .1
            .iter()
            .filter(|&&id| id > 0)
            .map(|&id| id as u32)
            .collect();

        tokenizer
            .decode(&token_ids, true)
            .map_err(|e| format!("Erro ao descodificar tokens: {}", e))?
    } else {
        return Err("Formato de saída inesperado do modelo ONNX".to_string());
    };

    // Clean any whitespace between Japanese characters
    let cleaned_text: String = extracted_text
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '\n' && *c != '\r')
        .collect();

    Ok(cleaned_text)
}
