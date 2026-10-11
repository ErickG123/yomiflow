pub mod commands;
pub mod db;
pub mod domain;
pub mod importer;
pub mod models;

use std::sync::{Arc, Mutex};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, WindowEvent};

pub struct AppState {
    pub db: Arc<Mutex<rusqlite::Connection>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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

            app.manage(AppState {
                db: Arc::new(Mutex::new(conn)),
            });

            // Create context menu items for system tray
            let show_i = MenuItem::with_id(app, "show", "Mostrar YomiFlow", true, None::<&str>)?;
            let review_i = MenuItem::with_id(app, "review", "Revisões Pendentes", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;

            let tray_menu = Menu::with_items(app, &[&show_i, &review_i, &quit_i])?;

            // Build system tray
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "review" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                            let _ = window.eval("window.location.hash = '#/study'; window.location.pathname = '/study';");
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
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
            commands::drive_sync::backup_database_to_drive,
            commands::nlp::analyze_japanese_text,
            commands::nlp::save_notebook_entry,
            commands::nlp::get_notebook_entries,
            commands::nlp::link_notebook_to_deck,
            commands::srs::get_decks,
            commands::srs::create_deck,
            commands::srs::update_deck,
            commands::srs::delete_deck,
            commands::srs::get_tags,
            commands::srs::create_tag,
            commands::stats::get_review_forecast,
            commands::stats::get_study_history,
            commands::kana::record_kana_attempt,
            commands::kana::get_kana_mastery_list,
            commands::kana::get_kana_proficiency_stats,
            commands::news::fetch_nhk_easy_news,
            commands::dictionary::add_custom_word_to_srs
        ])
        .run(tauri::generate_context!())
        .expect("error while building tauri application");
}
