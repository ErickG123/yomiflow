use crate::AppState;
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use reqwest::multipart::{Form, Part};
use rusqlite::params;
use serde_json::Value;
use std::fs;
use std::sync::mpsc;
use std::time::Duration;
use tauri::{AppHandle, Manager, State};
use tiny_http::{Header, Response, Server};

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_DRIVE_SCOPE: &str = "https://www.googleapis.com/auth/drive.file";

#[tauri::command]
pub async fn authenticate_google_drive(
    client_id: String,
    client_secret: String,
    _app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let redirect_url = RedirectUrl::new("http://localhost:8080/oauth/callback".to_string())
        .map_err(|e| format!("URL de redirecionamento inválida: {}", e))?;

    let auth_url = AuthUrl::new(GOOGLE_AUTH_URL.to_string())
        .map_err(|e| format!("URL de autorização inválida: {}", e))?;
    let token_url = TokenUrl::new(GOOGLE_TOKEN_URL.to_string())
        .map_err(|e| format!("URL de token inválida: {}", e))?;

    let client = BasicClient::new(
        ClientId::new(client_id.trim().to_string()),
        Some(ClientSecret::new(client_secret.trim().to_string())),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(redirect_url);

    let (authorize_url, _csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(GOOGLE_DRIVE_SCOPE.to_string()))
        .add_extra_param("access_type", "offline")
        .add_extra_param("prompt", "consent")
        .url();

    // 1. Inicia micro-servidor local não-bloqueante na porta 8080
    let server = Server::http("127.0.0.1:8080").map_err(|e| {
        format!(
            "Não foi possível abrir o servidor local na porta 8080: {}",
            e
        )
    })?;

    // Abre navegador padrão para o utilizador
    let _ = tauri_plugin_opener::open_url(authorize_url.as_str(), None::<&str>);

    // 2. Canal assíncrono para esperar o callback sem travar o runtime do Tauri
    let (tx, rx) = mpsc::channel::<Result<String, String>>();

    std::thread::spawn(move || {
        let timeout = Duration::from_secs(180);
        let start_time = std::time::Instant::now();

        loop {
            if start_time.elapsed() > timeout {
                let _ = tx.send(Err(
                    "Tempo limite excedido esperando autorização do Google.".to_string(),
                ));
                break;
            }

            match server.recv_timeout(Duration::from_millis(500)) {
                Ok(Some(request)) => {
                    let url = request.url().to_string();
                    if url.starts_with("/oauth/callback") {
                        // Extrai parâmetro 'code'
                        let mut code_found = None;
                        if let Some(query) = url.split('?').nth(1) {
                            for pair in query.split('&') {
                                let mut parts = pair.split('=');
                                if let (Some(k), Some(v)) = (parts.next(), parts.next())
                                    && k == "code"
                                {
                                    code_found = Some(v.to_string());
                                    break;
                                }
                            }
                        }

                        if let Some(code) = code_found {
                            let html = r#"
                                <!DOCTYPE html>
                                <html>
                                <head><meta charset="utf-8"><title>YomiFlow Conectado</title></head>
                                <body style="background:#09090b;color:#f4f4f5;font-family:sans-serif;display:flex;flex-direction:column;align-items:center;justify-content:center;height:90vh;">
                                    <h1 style="color:#10b981;">Autenticação Concluída com Sucesso!</h1>
                                    <p>O YomiFlow agora está conectado ao seu Google Drive.</p>
                                    <p style="color:#71717a;">Você já pode fechar esta aba e voltar ao aplicativo.</p>
                                </body>
                                </html>
                            "#;
                            let mut response = Response::from_string(html);
                            if let Ok(content_type) = Header::from_bytes(
                                &b"Content-Type"[..],
                                &b"text/html; charset=utf-8"[..],
                            ) {
                                response.add_header(content_type);
                            }
                            let _ = request.respond(response);
                            let _ = tx.send(Ok(code));
                            break;
                        } else {
                            let _ = request.respond(Response::from_string(
                                "Código de autorização não encontrado.",
                            ));
                            let _ = tx.send(Err(
                                "Parâmetro 'code' não encontrado no callback.".to_string()
                            ));
                            break;
                        }
                    } else {
                        let _ = request
                            .respond(Response::from_string("Aguardando callback do YomiFlow..."));
                    }
                }
                Ok(None) => {
                    // Timeout do recv_timeout: continua escutando até o timeout geral
                    continue;
                }
                Err(e) => {
                    let _ = tx.send(Err(format!("Erro no servidor HTTP local: {}", e)));
                    break;
                }
            }
        }
    });

    let code = match rx.recv() {
        Ok(res) => res?,
        Err(_) => return Err("Servidor local finalizado inesperadamente.".to_string()),
    };

    // 3. Troca o código pelo access_token e refresh_token
    let token_res = client
        .exchange_code(AuthorizationCode::new(code))
        .request_async(async_http_client)
        .await
        .map_err(|e| format!("Erro ao trocar código por token: {}", e))?;

    let access_token = token_res.access_token().secret().to_string();
    let refresh_token = token_res
        .refresh_token()
        .map(|r| r.secret().to_string())
        .unwrap_or_default();

    // 4. Salva tokens e credenciais no SQLite com persistência segura
    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('gdrive_client_id', ?1)",
            params![client_id],
        )
        .map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('gdrive_client_secret', ?1)",
            params![client_secret],
        )
        .map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('gdrive_access_token', ?1)",
            params![access_token],
        )
        .map_err(|e| e.to_string())?;

        if !refresh_token.is_empty() {
            conn.execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES ('gdrive_refresh_token', ?1)",
                params![refresh_token],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    Ok("Autenticado com sucesso no Google Drive!".to_string())
}

#[tauri::command]
pub async fn backup_database_to_drive(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // 1. Obtém credenciais e tokens salvos
    let (client_id, client_secret, access_token, refresh_token) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let get_val = |k: &str| -> Option<String> {
            conn.query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![k],
                |row| row.get(0),
            )
            .ok()
        };

        (
            get_val("gdrive_client_id"),
            get_val("gdrive_client_secret"),
            get_val("gdrive_access_token"),
            get_val("gdrive_refresh_token"),
        )
    };

    let access_tok = access_token.ok_or_else(|| {
        "Não autenticado no Google Drive. Por favor, conecte a sua conta primeiro.".to_string()
    })?;

    // 2. Se necessário, renova o access_token usando o refresh_token
    let client = reqwest::Client::new();
    let current_token = if let (Some(cid), Some(csec), Some(ref_tok)) =
        (client_id, client_secret, refresh_token)
    {
        if !ref_tok.is_empty() {
            // Tenta renovar o token para evitar erro de token expirado
            let refresh_res = client
                .post(GOOGLE_TOKEN_URL)
                .form(&[
                    ("client_id", cid.as_str()),
                    ("client_secret", csec.as_str()),
                    ("refresh_token", ref_tok.as_str()),
                    ("grant_type", "refresh_token"),
                ])
                .send()
                .await;

            if let Ok(resp) = refresh_res {
                if let Ok(body) = resp.json::<Value>().await {
                    if let Some(new_tok) = body.get("access_token").and_then(|v| v.as_str()) {
                        let conn = state.db.lock().map_err(|e| e.to_string())?;
                        let _ = conn.execute(
                            "INSERT OR REPLACE INTO settings (key, value) VALUES ('gdrive_access_token', ?1)",
                            params![new_tok],
                        );
                        new_tok.to_string()
                    } else {
                        access_tok
                    }
                } else {
                    access_tok
                }
            } else {
                access_tok
            }
        } else {
            access_tok
        }
    } else {
        access_tok
    };

    // 3. Lê o banco de dados principal do YomiFlow
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Não foi possível resolver o diretório de dados: {}", e))?;
    let db_path = app_data_dir.join("yomiflow.db");

    if !db_path.exists() {
        return Err("Ficheiro de banco de dados 'yomiflow.db' não encontrado.".to_string());
    }

    let db_bytes = fs::read(&db_path).map_err(|e| format!("Erro ao ler ficheiro SQLite: {}", e))?;

    // 4. Cria metadata e multipart payload para o Google Drive API
    let metadata_json = serde_json::json!({
        "name": "yomiflow_backup.db",
        "description": "Backup automático do banco de dados do YomiFlow (offline SRS & Dicionário)",
        "mimeType": "application/x-sqlite3"
    });

    let metadata_part = Part::text(metadata_json.to_string())
        .mime_str("application/json; charset=UTF-8")
        .map_err(|e| e.to_string())?;

    let file_part = Part::bytes(db_bytes)
        .file_name("yomiflow_backup.db")
        .mime_str("application/x-sqlite3")
        .map_err(|e| e.to_string())?;

    let multipart = Form::new()
        .part("metadata", metadata_part)
        .part("file", file_part);

    // 5. Envia POST multipart para a API de upload do Google Drive
    let upload_url = "https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart";
    let res = client
        .post(upload_url)
        .bearer_auth(current_token)
        .multipart(multipart)
        .send()
        .await
        .map_err(|e| format!("Falha de conexão com a API do Google Drive: {}", e))?;

    if !res.status().is_success() {
        let err_text = res.text().await.unwrap_or_default();
        return Err(format!("Erro retornado pelo Google Drive: {}", err_text));
    }

    let res_json: Value = res
        .json()
        .await
        .map_err(|e| format!("Falha ao decodificar resposta do Drive: {}", e))?;

    let file_id = res_json
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("backup_concluido")
        .to_string();

    // 6. Atualiza data e hora do último backup nas configurações locais
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let _ = conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('gdrive_last_backup_time', ?1)",
            params![now.to_string()],
        );
    }

    Ok(file_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_pre_upload_database_reading() {
        // Create a temporary simulated SQLite file
        let temp_dir = std::env::temp_dir().join("yomiflow_test_sync");
        let _ = fs::create_dir_all(&temp_dir);
        let test_db_path = temp_dir.join("yomiflow_mock.db");

        let mock_sqlite_header = b"SQLite format 3\0\x04\0\x01\x01\0@  \0\0\0\x01";
        {
            let mut file = fs::File::create(&test_db_path).expect("failed to create temp db");
            file.write_all(mock_sqlite_header)
                .expect("failed to write mock bytes");
        }

        // Simulate pre-upload read
        let bytes_read = fs::read(&test_db_path).expect("failed to read simulated db file");
        assert_eq!(bytes_read.len(), mock_sqlite_header.len());
        assert_eq!(&bytes_read[0..16], b"SQLite format 3\0");

        // Clean up
        let _ = fs::remove_file(test_db_path);
        let _ = fs::remove_dir(temp_dir);
    }
}
