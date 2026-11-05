//! OmniLauncher backend entry point.

pub mod commands;
pub mod core;
pub mod error;
pub mod platforms;

use std::{sync::Arc, time::Duration};

use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::ShortcutState;

use crate::core::{
    config::{AppConfig, ConfigStore},
    discord_rpc::DiscordRpcService,
    media_capture::{self, MediaCapture},
    platform::{PlatformConnector, PlatformRegistry},
    runner::ProcessRunner,
};
use crate::platforms::{
    emulators::EmulatorPlatform, epic::EpicPlatform, itch::ItchPlatform, roblox::RobloxPlatform,
    steam::SteamPlatform,
};

/// Emitted with a [`core::platform::ScanReport`] whenever the library is rescanned.
pub const EVT_LIBRARY_UPDATED: &str = "library://updated";

pub fn run() {
    init_tracing();

    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            let config_dir = app.path().app_config_dir()?;
            let data_dir = app.path().app_data_dir()?;

            // ---- Configuration -------------------------------------------------
            let config = AppConfig::load(&config_dir)?;
            app.manage(ConfigStore::new(config_dir, config.clone()));

            // ---- Shared HTTP client ---------------------------------------------
            let http = reqwest::Client::builder()
                .user_agent(concat!("OmniLauncher/", env!("CARGO_PKG_VERSION")))
                .timeout(Duration::from_secs(20))
                .build()?;
            app.manage(http.clone());

            // ---- Platform connectors -------------------------------------------
            let steam = Arc::new(SteamPlatform::new(http.clone(), config.steam.clone()));
            let epic = Arc::new(EpicPlatform::new());
            let roblox = Arc::new(RobloxPlatform::new(http.clone(), &data_dir.join("roblox")));
            let itch = Arc::new(ItchPlatform::new());
            let emulators = Arc::new(EmulatorPlatform::new(EmulatorPlatform::default_presets()));

            let registry = Arc::new(PlatformRegistry::new(vec![
                steam.clone() as Arc<dyn PlatformConnector>,
                epic.clone() as Arc<dyn PlatformConnector>,
                roblox.clone() as Arc<dyn PlatformConnector>,
                itch as Arc<dyn PlatformConnector>,
                emulators as Arc<dyn PlatformConnector>,
            ]));
            app.manage(registry.clone());
            app.manage(steam);
            app.manage(epic);
            app.manage(roblox);

            // ---- Media capture + global hotkeys --------------------------------
            let media = Arc::new(MediaCapture::new(handle.clone(), config.capture.clone()));
            app.manage(media.clone());

            // ---- Universal Discord Rich Presence + Process Runner --------------
            let discord = Arc::new(DiscordRpcService::new(None));
            app.manage(discord.clone());
            let runner = ProcessRunner::new(handle.clone(), discord, media.clone());
            app.manage(runner);

            // ---- Show, focus, and apply Windows 11 Mica vibrancy ---------------
            if let Some(window) = app.get_webview_window("main") {
                if let Err(err) = commands::window::apply_effect(
                    &window,
                    commands::window::WindowEffectKind::Mica,
                ) {
                    tracing::warn!(error = %err, "window vibrancy unavailable");
                }
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();

                if let Err(err) = app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_handler(|app, shortcut, event| {
                            if matches!(event.state(), ShortcutState::Pressed) {
                                media_capture::handle_shortcut(app, shortcut);
                            }
                        })
                        .build(),
                ) {
                    tracing::warn!(error = %err, "global shortcut plugin unavailable");
                } else if let Err(err) = media.register_hotkeys(&config.capture) {
                    tracing::warn!(error = %err, "could not register capture hotkeys");
                }

                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }

            // ---- Initial library scan (non-blocking) ---------------------------
            tauri::async_runtime::spawn(async move {
                let report = registry.scan_all().await;
                tracing::info!(
                    games = report.games.len(),
                    failures = report.errors.len(),
                    "initial library scan finished"
                );
                let _ = handle.emit(EVT_LIBRARY_UPDATED, &report);
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::library::get_library,
            commands::library::refresh_library,
            commands::library::launch_game,
            commands::library::install_game,
            commands::library::get_active_sessions,
            commands::library::sync_platform,
            commands::library::set_steam_credentials,
            commands::library::get_steam_credentials,
            commands::library::get_steam_cloud_saves,
            commands::library::search_steamgriddb_posters,
            commands::platforms::get_steam_profile,
            commands::platforms::get_steam_achievements,
            commands::platforms::repair_steam_achievements,
            commands::platforms::sync_steam_cloud,
            commands::platforms::get_epic_library,
            commands::platforms::get_roblox_recent_places,
            commands::platforms::get_unified_library,
            commands::platforms::get_game_properties,
            commands::platforms::save_game_properties,
            commands::platforms::verify_game_files,
            commands::media::take_screenshot,
            commands::media::toggle_recording,
            commands::media::recording_status,
            commands::media::list_media,
            commands::media::delete_media,
            commands::media::get_capture_config,
            commands::media::set_capture_config,
            commands::roblox::roblox_get_fast_flags,
            commands::roblox::roblox_set_fast_flags,
            commands::roblox::roblox_apply_preset,
            commands::roblox::roblox_set_multi_instance,
            commands::roblox::roblox_multi_instance_status,
            commands::roblox::roblox_detect_place,
            commands::roblox::roblox_register_protocol,
            commands::roblox::roblox_launch_place,
            commands::roblox::roblox_get_popular_experiences,
            commands::roblox::roblox_search_experiences,
            commands::social::get_unified_friends,
            commands::social::get_discord_rpc_status,
            commands::social::set_discord_rpc_enabled,
            commands::social::set_discord_rpc_activity,
            commands::deals::get_deals_radar,
            commands::deals::get_steam_store_showcase,
            commands::deals::search_steam_store,
            commands::deals::get_itch_store_showcase,
            commands::roulette::spin_backlog_roulette,
            commands::roulette::get_hltb_benchmark,
            commands::window::set_window_effect,
            commands::window::open_in_app_store_window,
            commands::window::get_store_page_html,
            commands::store::get_store_recommendations,
            commands::store::get_preferred_currency,
            commands::store::set_preferred_currency,
            commands::store::convert_currency,
            commands::store::get_game_description,
            commands::emulators::get_emulator_catalog,
            commands::emulators::install_emulator,
            commands::emulators::launch_emulator,
            commands::network::get_system_drives,
            commands::network::list_directory_contents,
            commands::network::create_directory,
            commands::network::get_network_settings,
            commands::network::update_network_settings,
            commands::network::test_network_connection,
            commands::platforms::open_game_directory,
            commands::game_mode::enter_game_mode,
            commands::game_mode::exit_game_mode,
            commands::game_mode::get_game_mode_status,
            commands::game_mode::trim_process_memory,
            commands::save_vault::create_save_snapshot,
            commands::save_vault::list_save_snapshots,
            commands::save_vault::restore_save_snapshot,
            commands::save_vault::delete_save_snapshot,
            commands::save_vault::toggle_snapshot_pin,
            commands::save_vault::open_save_folder,
            commands::save_vault::detect_game_save_locations,
            commands::save_vault::get_save_vault_summary,
            commands::shader_reclaimer::scan_shader_caches,
            commands::shader_reclaimer::clean_shader_caches,
            commands::hltb::get_hltb_data,
            commands::hltb::get_batch_hltb_data,
            commands::pcgw::get_pcgamingwiki_data,
            commands::pcgw::apply_pcgw_tweak,
            commands::pcgw::revert_pcgw_tweak,
            commands::pcgw::open_game_config_directory,
        ])
        .run(tauri::generate_context!())
        .expect("error while running OmniLauncher");
}

fn init_tracing() {
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("omnilauncher_lib=info,warn"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

