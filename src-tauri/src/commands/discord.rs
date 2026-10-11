use discord_rich_presence::{DiscordIpc, DiscordIpcClient, activity};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const DISCORD_APP_ID: &str = "1348392817283731456";

// Thread-safe wrapper for Discord client
static DISCORD_CLIENT: Mutex<Option<DiscordIpcClient>> = Mutex::new(None);

#[tauri::command]
pub fn set_discord_zen_presence(enabled: bool) -> Result<(), String> {
    let mut client_lock = DISCORD_CLIENT.lock().map_err(|e| e.to_string())?;

    if enabled {
        let mut client = match client_lock.take() {
            Some(c) => c,
            None => {
                let mut c = DiscordIpcClient::new(DISCORD_APP_ID);
                if let Err(e) = c.connect() {
                    eprintln!("Failed to connect to Discord IPC: {:?}", e);
                    return Ok(()); // Fail gracefully if Discord is not running
                }
                c
            }
        };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let timestamps = activity::Timestamps::new().start(now);
        let assets = activity::Assets::new()
            .large_image("yomiflow_logo")
            .large_text("YomiFlow - Imersão em Japonês");

        let activity_payload = activity::Activity::new()
            .state("Modo Imersão")
            .details("Foco Profundo - Modo Imersão")
            .timestamps(timestamps)
            .assets(assets);

        if let Err(e) = client.set_activity(activity_payload) {
            eprintln!("Failed to set Discord activity: {}", e);
        }

        *client_lock = Some(client);
    } else if let Some(mut client) = client_lock.take() {
        let _ = client.clear_activity();
        let _ = client.close();
    }

    Ok(())
}
