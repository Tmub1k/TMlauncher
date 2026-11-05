//! Integration and Contract Verification Tests for Milestone 4 Flagship Features.
//! Covers:
//! 1. Zero-Footprint Game Mode (Status transitions, memory trim)
//! 2. Save-State Time Machine (Creation, manifest, SHA-256, auto-rollback safeguard, restore, pin, delete)
//! 3. Shader Cache Reclaimer (Scanning caches, summary contract, clean execution)
//! 4. HowLongToBeat Integration (Query sanitization, curated catalog lookup, heuristic fallback)
//! 5. PCGamingWiki Integration (Configuration data contract, tweak profiles, backup suffix)

use omnilauncher_lib::commands::{
    hltb::{self, HltbRecord},
    pcgw::{self, PcgwRecord},
    save_vault::{self, SaveSnapshot},
    shader_reclaimer::{self, CleanRequest, ShaderCacheSummary},
};
use omnilauncher_lib::core::game_mode;

#[test]
fn test_hltb_query_sanitizer() {
    let raw = "The Witcher 3: Wild Hunt - Game of the Year Edition (v1.32)";
    let cleaned = hltb::sanitize_hltb_query(raw);
    assert_eq!(cleaned, "the witcher 3 wild hunt");

    let raw2 = "BioShock Remastered [Special Edition]";
    let cleaned2 = hltb::sanitize_hltb_query(raw2);
    assert_eq!(cleaned2, "bioshock");
}

#[tokio::test]
async fn test_hltb_curated_and_fallback_queries() {
    // 1. Curated catalog match
    let witcher: HltbRecord = hltb::get_hltb_data("The Witcher 3: Wild Hunt".to_string())
        .await
        .expect("Should return HLTB data");
    assert_eq!(witcher.game_title, "The Witcher 3: Wild Hunt");
    assert!(witcher.main_story_hours >= 45.0);
    assert!(witcher.completionist_hours >= 150.0);

    // 2. Curated catalog match via fuzzy sanitization
    let elden: HltbRecord = hltb::get_hltb_data("Elden Ring: Shadow of the Erdtree Edition".to_string())
        .await
        .expect("Should resolve Elden Ring");
    assert!(elden.main_story_hours >= 50.0);

    // 3. Fallback heuristic for unknown indie
    let unknown: HltbRecord = hltb::get_hltb_data("My Unheard Fantasy Indie RPG 2026".to_string())
        .await
        .expect("Should generate deterministic heuristic");
    assert!(unknown.main_story_hours > 0.0);
    assert!(unknown.completionist_hours >= unknown.main_story_hours);
}

#[tokio::test]
async fn test_pcgamingwiki_data_contracts() {
    let pcgw: PcgwRecord = pcgw::get_pcgamingwiki_data("Cyberpunk 2077".to_string(), None)
        .await
        .expect("Should return PCGW data");
    assert_eq!(pcgw.game_title, "Cyberpunk 2077");
    assert!(pcgw.ultrawide_support);
    assert!(pcgw.fps_uncap_supported);
    assert!(pcgw.fov_slider_supported);
    assert!(!pcgw.tweaks.is_empty());

    let fov_tweak = pcgw.tweaks.iter().find(|t| t.id == "fov_slider");
    assert!(fov_tweak.is_some());
    let tweak = fov_tweak.unwrap();
    assert_eq!(tweak.key, "FieldOfView");
    assert_eq!(tweak.recommended_val, "100");
}

#[tokio::test]
async fn test_shader_reclaimer_scan_and_clean_contract() {
    let summary: ShaderCacheSummary = shader_reclaimer::scan_shader_caches()
        .await
        .expect("Scan should succeed without panicking");

    assert!(summary.scanned_at > 0);
    assert_eq!(summary.caches.len(), summary.item_count);

    // Clean with empty list is a safe no-op
    let clean_res = shader_reclaimer::clean_shader_caches(CleanRequest {
        item_ids: vec!["nonexistent_cache_id_for_testing".to_string()],
        only_orphans: true,
    })
    .await
    .expect("Clean request should succeed");

    assert_eq!(clean_res.cleaned_items_count, 0);
}

#[tokio::test]
async fn test_game_mode_transitions_and_memory_trim() {
    // 1. Initial status
    let status_before = game_mode::get_game_mode_status(0);
    assert!(!status_before.active);

    // 2. Enter game mode
    let status_active = game_mode::enter_game_mode(1);
    assert!(status_active.active);
    assert_eq!(status_active.active_sessions_count, 1);
    assert_eq!(status_active.process_priority, "IDLE_PRIORITY_CLASS");

    // 3. Trim working set
    let trim = game_mode::trim_working_set();
    assert!(trim.success);

    // 4. Exit game mode
    let status_after = game_mode::exit_game_mode();
    assert!(!status_after.active);
    assert_eq!(status_after.active_sessions_count, 0);
    assert_eq!(status_after.process_priority, "NORMAL_PRIORITY_CLASS");
}

#[tokio::test]
async fn test_save_vault_end_to_end_lifecycle() {
    let temp_dir = std::env::temp_dir().join(format!("omnilauncher_m4_vault_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let save_dir = temp_dir.join("save_dir");
    std::fs::create_dir_all(&save_dir).unwrap();

    let file_a = save_dir.join("save_data.bin");
    let file_b = save_dir.join("profile.json");
    std::fs::write(&file_a, b"INITIAL_HERO_LEVEL_10").unwrap();
    std::fs::write(&file_b, b"{\"gold\": 5000}").unwrap();

    let game_id = "test_rpg_hero";

    // 1. Create initial snapshot
    let snap: SaveSnapshot = save_vault::create_save_snapshot(
        game_id.to_string(),
        Some("Before Boss Fight".to_string()),
        Some("manual".to_string()),
        Some(save_dir.to_string_lossy().to_string()),
    )
    .await
    .expect("Snapshot creation should succeed");

    assert_eq!(snap.game_id, game_id);
    assert!(snap.archive_size_bytes > 0);
    assert!(!snap.sha256_checksum.is_empty());
    assert_eq!(snap.file_count, 2);

    // 2. List snapshots
    let list = save_vault::list_save_snapshots(game_id.to_string())
        .await
        .expect("Listing snapshots should succeed");
    assert!(list.iter().any(|s| s.snapshot_id == snap.snapshot_id));

    // 3. Pin snapshot
    let is_pinned = save_vault::toggle_snapshot_pin(game_id.to_string(), snap.snapshot_id.clone())
        .await
        .expect("Toggle pin should succeed");
    assert!(is_pinned);

    // 4. Simulate save corruption / modification
    std::fs::write(&file_a, b"CORRUPTED_AFTER_DEATH").unwrap();
    assert_eq!(std::fs::read(&file_a).unwrap(), b"CORRUPTED_AFTER_DEATH");

    // 5. Restore snapshot (verifies pre-restore safeguard is automatically captured!)
    let restore_result = save_vault::restore_save_snapshot(
        game_id.to_string(),
        snap.snapshot_id.clone(),
    )
    .await
    .expect("Restore should succeed");

    assert!(restore_result.success);
    assert!(restore_result.safety_snapshot_id.is_some());

    // Verify content was genuinely restored!
    let restored_bytes = std::fs::read(&file_a).unwrap();
    assert_eq!(restored_bytes, b"INITIAL_HERO_LEVEL_10");

    // Verify safeguard was added to the snapshots list
    let list_after = save_vault::list_save_snapshots(game_id.to_string())
        .await
        .unwrap();
    assert!(list_after.len() >= 2);
    let safeguard = list_after.iter().find(|s| s.kind == "safeguard");
    assert!(safeguard.is_some());

    // 6. Delete snapshot
    let del_res = save_vault::delete_save_snapshot(game_id.to_string(), snap.snapshot_id.clone())
        .await;
    assert!(del_res.is_ok());

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);
}
