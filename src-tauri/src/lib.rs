pub mod commands;
pub mod database;
pub mod db;
pub mod domain;
pub mod importer;
pub mod models;
pub mod services;

use std::sync::{Arc, Mutex};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Emitter, Manager, WindowEvent};

pub struct AppState {
    pub db: Arc<Mutex<rusqlite::Connection>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)]
    {
        let env_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../.env");
        let _ = dotenvy::from_path(env_path);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let conn = db::initialize_database(app.handle())
                .map_err(|e| Box::<dyn std::error::Error>::from(e.to_string()))?;

            let db = Arc::new(Mutex::new(conn));
            let audio_runtime = Arc::new(
                services::audio::AudioRuntime::new(db.clone())
                    .map_err(Box::<dyn std::error::Error>::from)?,
            );

            app.manage(AppState { db });
            app.manage(audio_runtime);

            // Create grouped system tray actions
            let start_pomodoro_i = MenuItem::with_id(
                app,
                "start_pomodoro",
                "Iniciar Pomodoro (25min)",
                true,
                None::<&str>,
            )?;
            let new_note_i = MenuItem::with_id(
                app,
                "new_note",
                "Nova Anota\u{00e7}\u{00e3}o",
                true,
                None::<&str>,
            )?;
            let study_separator = PredefinedMenuItem::separator(app)?;
            let review_i = MenuItem::with_id(
                app,
                "pending_reviews",
                "Revis\u{00f5}es Pendentes (SRS)",
                true,
                None::<&str>,
            )?;
            let zen_mode_i =
                MenuItem::with_id(app, "zen_mode", "Modo Foco (Zen Mode)", true, None::<&str>)?;
            let system_separator = PredefinedMenuItem::separator(app)?;
            let show_i =
                MenuItem::with_id(app, "show_app", "Mostrar YomiFlow", true, None::<&str>)?;
            let quit_separator = PredefinedMenuItem::separator(app)?;
            let quit_i = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;

            let tray_menu = Menu::with_items(
                app,
                &[
                    &start_pomodoro_i,
                    &new_note_i,
                    &study_separator,
                    &review_i,
                    &zen_mode_i,
                    &system_separator,
                    &show_i,
                    &quit_separator,
                    &quit_i,
                ],
            )?;
            // Build system tray
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    let id = event.id.as_ref();
                    match id {
                        "show_app" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "start_pomodoro" | "new_note" | "pending_reviews" | "zen_mode" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                                let _ = window.emit("tray-action", id);
                            }
                        }
                        "quit" => app.exit(0),
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    } else if let TrayIconEvent::DoubleClick {
                        button: MouseButton::Left,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::dictionary::search_dictionary,
            commands::dictionary::search_kanjis,
            commands::dictionary::get_kanji_breakdown,
            commands::srs::add_to_srs,
            commands::srs::remove_from_srs,
            commands::srs::get_due_cards,
            commands::srs::answer_card,
            commands::settings::get_setting,
            commands::settings::set_setting,
            commands::system::get_dashboard_stats,
            commands::system::clear_database,
            commands::importer::import_dictionary,
            commands::importer::import_jlpt_kanjis,
            commands::importer::import_kanjivg_folder,
            commands::importer::import_frequency_list,
            commands::importer::get_kanji_svg,
            commands::srs::get_study_heatmap,
            commands::srs::get_study_streaks,
            commands::reader::load_cbz_metadata,
            commands::reader::get_cbz_page_base64,
            commands::reader::process_ocr,
            commands::reader::scan_manga_library,
            commands::reader::read_epub_file,
            commands::grading::evaluate_kanji_drawing,
            commands::video::extract_subtitles,
            commands::video::cut_audio_clip,
            commands::video::check_ffmpeg_installed,
            commands::ai_ocr::check_model_exists,
            commands::ai_ocr::download_ai_model,
            commands::ai_ocr::process_image_with_ai,
            commands::system::check_onboarding_needed,
            commands::system::complete_onboarding,
            commands::system::has_completed_onboarding,
            commands::kanji::get_kanji_details,
            commands::kanji::save_kanji_mnemonic,
            commands::drive_sync::authenticate_google_drive,
            commands::drive_sync::get_google_drive_oauth_config,
            commands::drive_sync::backup_database_to_drive,
            commands::nlp::analyze_japanese_text,
            commands::nlp::generate_furigana,
            commands::nlp::lint_japanese_text,
            commands::nlp::analyze_text_readability,
            commands::nlp::generate_syntax_tree,
            commands::nlp::inject_furigana_epub,
            commands::nlp::save_notebook_entry,
            commands::nlp::get_notebook_entries,
            commands::nlp::delete_notebook_entry,
            commands::nlp::link_notebook_to_deck,
            commands::srs::get_decks,
            commands::srs::create_deck,
            commands::srs::update_deck,
            commands::srs::delete_deck,
            commands::srs::get_tags,
            commands::srs::create_tag,
            commands::stats::get_review_forecast,
            commands::stats::get_study_history,
            commands::stats::get_jlpt_coverage_stats,
            commands::kana::record_kana_attempt,
            commands::kana::update_kana_score,
            commands::kana::get_kana_characters,
            commands::kana::get_kana_mastery_list,
            commands::kana::get_kana_proficiency_stats,
            commands::news::fetch_nhk_easy_news,
            commands::dictionary::add_custom_word_to_srs,
            commands::dictionary::add_readability_word_to_srs,
            commands::srs::migrate_sm2_to_fsrs,
            commands::reader::save_epub_position,
            commands::reader::get_epub_position,
            commands::stats::get_badges,
            commands::discord::set_discord_zen_presence,
            commands::pomodoro::save_study_session,
            commands::image_occlusion::save_image_occlusion_card,
            commands::image_occlusion::list_image_occlusion_cards,
            commands::image_occlusion::get_image_occlusion_card,
            commands::image_occlusion::answer_image_occlusion_card,
            commands::interleaving::get_interleaved_review_queue,
            commands::interleaving::answer_grammar_review,
            commands::search::global_search,
            commands::audio::play_audio,
            commands::audio::pause_audio,
            commands::audio::resume_audio,
            commands::audio::set_volume,
            commands::audio::seek_to,
            commands::audio::get_audio_player_snapshot,
            commands::audio::get_passive_listening_history,
            commands::audio::get_audio_library,
            commands::audio::import_audio_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while building tauri application");
}
