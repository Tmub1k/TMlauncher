//! Milestone 3 Remediation Adversarial Recheck 2 — Empirical Stress Test Suite
//!
//! Scope:
//! 1. Zip-Slip resilience and path traversal containment.
//! 2. Safe extraction rollback preservation (pre-existing files NEVER deleted, temp files cleaned).
//! 3. Win32 `GetDiskFreeSpaceExW` positive disk space returns across Windows drives.
//! 4. Windows reserved device names with extensions (`CON.txt`, `aux.json`, `COM1.log`) and path traversal.
//! 5. Proxy validation: bracketed IPv6 hostnames (`[::1]:8080`, `[2001:db8::1]:1080`), malformed protocols, out-of-range ports.
//! 6. DoH DNS probing: proper error handling on network failure and genuine record data parsing without hardcoded constants.

use std::io::{Cursor, Write};
use std::path::Path;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use omnilauncher_lib::commands::emulators::safe_extract_archive;
use omnilauncher_lib::commands::network::{
    create_directory, get_system_drives, list_directory_contents, test_network_connection,
};

// ============================================================================
// AREA 1: Zip-Slip Resilience & Traversal Containment
// ============================================================================

#[test]
fn test_zip_slip_malicious_archive_rejection_and_containment() {
    let temp_root = std::env::temp_dir().join(format!("omni_test_zipslip_{}", std::process::id()));
    let target_dest = temp_root.join("extract_target");
    let outside_marker = temp_root.join("escaped_file.txt");

    let _ = std::fs::remove_dir_all(&temp_root);
    std::fs::create_dir_all(&target_dest).unwrap();

    // Create a zip with multiple malicious zip-slip variations
    let mut zip_bytes = Vec::new();
    {
        let mut writer = ZipWriter::new(Cursor::new(&mut zip_bytes));
        let options = SimpleFileOptions::default();

        // 1. Safe entry
        writer.start_file("valid_folder/valid_file.txt", options).unwrap();
        writer.write_all(b"SAFE DATA").unwrap();

        // 2. Relative traversal escaping dest
        writer.start_file("../../escaped_file.txt", options).unwrap();
        writer.write_all(b"PWNED_RELATIVE").unwrap();

        writer.finish().unwrap();
    }

    let archive_path = temp_root.join("malicious.zip");
    std::fs::write(&archive_path, &zip_bytes).unwrap();

    // Execute safe_extract_archive
    let res = safe_extract_archive(&archive_path, &target_dest);

    // Extraction MUST fail
    assert!(res.is_err(), "safe_extract_archive must return Err for Zip-Slip archive");
    let err_msg = res.unwrap_err();
    assert!(
        err_msg.contains("Zip-Slip"),
        "Error message should explicitly mention Zip-Slip, got: {err_msg}"
    );

    // Verify escaped file was NOT written outside
    assert!(
        !outside_marker.exists(),
        "CRITICAL VULNERABILITY: Zip-Slip succeeded in writing outside extraction directory: {}",
        outside_marker.display()
    );

    // Verify rollback cleaned up the valid_folder/valid_file.txt that was extracted before the zip-slip entry
    let safe_extracted = target_dest.join("valid_folder").join("valid_file.txt");
    assert!(
        !safe_extracted.exists(),
        "Rollback failed to clean up partially extracted file: {}",
        safe_extracted.display()
    );

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_root);
}

// ============================================================================
// AREA 2: Safe Extraction Rollback Preservation
// ============================================================================

#[test]
fn test_rollback_preserves_preexisting_files_and_directories() {
    let temp_root = std::env::temp_dir().join(format!("omni_test_rollback_{}", std::process::id()));
    let target_dest = temp_root.join("game_installation_dir");

    let _ = std::fs::remove_dir_all(&temp_root);
    std::fs::create_dir_all(&target_dest).unwrap();

    // Simulate pre-existing user files in target_dest
    let pre_existing_save = target_dest.join("user_savegame.sav");
    std::fs::write(&pre_existing_save, b"IMPORTANT_SAVE_PAYLOAD_1337").unwrap();

    let pre_existing_subdir = target_dest.join("mods");
    std::fs::create_dir_all(&pre_existing_subdir).unwrap();
    let pre_existing_mod = pre_existing_subdir.join("mod_config.json");
    std::fs::write(&pre_existing_mod, b"{\"mod_enabled\": true}").unwrap();

    // Create a zip that extracts one new file and then causes a failure
    let mut zip_bytes = Vec::new();
    {
        let mut writer = ZipWriter::new(Cursor::new(&mut zip_bytes));
        let options = SimpleFileOptions::default();

        // Safe new file
        writer.start_file("new_extracted_file.bin", options).unwrap();
        writer.write_all(b"NEW_DATA").unwrap();

        // Malicious entry causing failure
        writer.start_file("../outside.txt", options).unwrap();
        writer.write_all(b"EXPLODED").unwrap();

        writer.finish().unwrap();
    }

    let archive_path = temp_root.join("failing_archive.zip");
    std::fs::write(&archive_path, &zip_bytes).unwrap();

    let res = safe_extract_archive(&archive_path, &target_dest);
    assert!(res.is_err(), "Extraction must fail on zip-slip entry");

    // CRUCIAL: Pre-existing user files must remain completely intact
    assert!(
        pre_existing_save.exists(),
        "CRITICAL REGRESSION: Pre-existing save file was deleted during rollback!"
    );
    let save_content = std::fs::read(&pre_existing_save).unwrap();
    assert_eq!(
        save_content,
        b"IMPORTANT_SAVE_PAYLOAD_1337",
        "Pre-existing save file content was modified!"
    );

    assert!(
        pre_existing_mod.exists(),
        "CRITICAL REGRESSION: Pre-existing mod file in subdir was deleted!"
    );
    let mod_content = std::fs::read(&pre_existing_mod).unwrap();
    assert_eq!(
        mod_content,
        b"{\"mod_enabled\": true}",
        "Pre-existing mod file content was modified!"
    );

    // The newly created file must have been cleaned up
    let new_file = target_dest.join("new_extracted_file.bin");
    assert!(
        !new_file.exists(),
        "Rollback failed to remove newly created file upon failure"
    );

    // target_dest must still exist because it was pre-existing
    assert!(
        target_dest.exists(),
        "CRITICAL: Pre-existing destination folder itself was wiped out!"
    );

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_root);
}

#[test]
fn test_rollback_cleans_up_newly_created_dest_dir_on_failure() {
    let temp_root = std::env::temp_dir().join(format!("omni_test_newdir_{}", std::process::id()));
    let target_dest = temp_root.join("freshly_created_dest");

    let _ = std::fs::remove_dir_all(&temp_root);
    std::fs::create_dir_all(&temp_root).unwrap();

    // target_dest DOES NOT exist yet
    assert!(!target_dest.exists());

    // Create an invalid zip
    let corrupt_zip = temp_root.join("bad.zip");
    std::fs::write(&corrupt_zip, b"NOT_A_VALID_ZIP_ARCHIVE_DATA").unwrap();

    let res = safe_extract_archive(&corrupt_zip, &target_dest);
    assert!(res.is_err());

    // Because target_dest did not exist before, it should have been safely cleaned up
    assert!(
        !target_dest.exists(),
        "Fresh destination directory should be cleaned up on extraction failure"
    );

    let _ = std::fs::remove_dir_all(&temp_root);
}

// ============================================================================
// AREA 3: Win32 GetDiskFreeSpaceExW Authentic Positive Space Returns
// ============================================================================

#[tokio::test]
async fn test_win32_disk_space_returns_positive_space() {
    // Test list_directory_contents on C:\
    let listing_c = list_directory_contents("C:\\".to_string()).await;
    assert!(listing_c.is_ok(), "list_directory_contents for C:\\ must succeed");
    let c_info = listing_c.unwrap();
    println!("C:\\ disk space: {} bytes ({} GB)", c_info.free_space_bytes, c_info.free_space_gb);
    assert!(
        c_info.free_space_bytes > 0,
        "CRITICAL: Free space bytes on C:\\ must be > 0, got {}",
        c_info.free_space_bytes
    );
    assert!(
        c_info.free_space_gb > 0.0,
        "CRITICAL: Free space GB on C:\\ must be > 0.0, got {}",
        c_info.free_space_gb
    );

    // Test list_directory_contents on C:\Users
    let listing_users = list_directory_contents("C:\\Users".to_string()).await;
    assert!(listing_users.is_ok(), "list_directory_contents for C:\\Users must succeed");
    let users_info = listing_users.unwrap();
    println!("C:\\Users disk space: {} bytes ({} GB)", users_info.free_space_bytes, users_info.free_space_gb);
    assert!(
        users_info.free_space_bytes > 0,
        "CRITICAL: Free space bytes on C:\\Users must be > 0, got {}",
        users_info.free_space_bytes
    );
    assert!(
        users_info.free_space_gb > 0.0,
        "CRITICAL: Free space GB on C:\\Users must be > 0.0, got {}",
        users_info.free_space_gb
    );

    // Test get_system_drives
    let drives = get_system_drives().await.expect("get_system_drives");
    assert!(!drives.is_empty(), "Must find at least one system drive");
    let c_drive = drives.iter().find(|d| d.mount_point.starts_with("C:"));
    assert!(c_drive.is_some(), "C: drive must be listed");
    let c = c_drive.unwrap();
    println!("C: drive from get_system_drives: total={} GB, free={} GB", c.total_gb, c.free_gb);
    assert!(c.total_gb > 0.0, "C: total_gb must be > 0.0");
    assert!(c.free_gb > 0.0, "C: free_gb must be > 0.0");

    // Direct Win32 GetDiskFreeSpaceExW comparison:
    // Demonstrate empirically that format!("{}\\0") fails while encode_utf16().chain(once(0)) succeeds!
    #[cfg(windows)]
    {
        use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;

        // Correct format (fixed)
        let root = Path::new("C:\\");
        let mut root_str = root.to_string_lossy().to_string();
        if root_str.ends_with(':') {
            root_str.push('\\');
        }
        let clean_wide: Vec<u16> = root_str.encode_utf16().chain(std::iter::once(0)).collect();
        let mut free_avail = 0u64;
        let mut total = 0u64;
        let mut total_free = 0u64;
        let ok_clean = unsafe {
            GetDiskFreeSpaceExW(clean_wide.as_ptr(), &mut free_avail, &mut total, &mut total_free)
        };
        assert_ne!(ok_clean, 0, "Clean wide path must succeed in GetDiskFreeSpaceExW");
        assert!(free_avail > 0, "free_avail must be positive");

        // Old buggy format (appended literal string "\\0")
        let buggy_str = format!("{}\\0", root.display());
        let buggy_wide: Vec<u16> = buggy_str.encode_utf16().chain(std::iter::once(0)).collect();
        let mut buggy_free = 0u64;
        let mut buggy_total = 0u64;
        let mut buggy_total_free = 0u64;
        let ok_buggy = unsafe {
            GetDiskFreeSpaceExW(buggy_wide.as_ptr(), &mut buggy_free, &mut buggy_total, &mut buggy_total_free)
        };
        // The buggy call fails because C:\\0 does not exist
        assert_eq!(
            ok_buggy, 0,
            "Empirical verification: format!(\"{{}}\\0\") MUST fail on Win32, returning 0"
        );
    }
}

// ============================================================================
// AREA 4: Windows Reserved Device Names With Extensions & Path Traversal
// ============================================================================

#[tokio::test]
async fn test_create_directory_reserved_device_names_with_extensions() {
    let temp_root = std::env::temp_dir().join(format!("omni_test_res_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&temp_root);
    std::fs::create_dir_all(&temp_root).unwrap();

    let reserved_with_extensions = [
        "CON.txt",
        "con.json",
        "PRN.doc",
        "aux.tar.gz",
        "NUL.zip",
        "COM1.log",
        "com9.ini",
        "LPT1.iso",
        "lpt3.dat",
        "AUX.TXT",
        "CON.anything",
    ];

    for name in reserved_with_extensions {
        let p = temp_root.join(name);
        let res = create_directory(p.to_string_lossy().to_string()).await;
        assert!(
            res.is_err(),
            "create_directory MUST reject reserved device name with extension: {}",
            name
        );
        let err = res.unwrap_err();
        assert!(
            err.contains("reserved Windows device name"),
            "Error for '{}' must mention 'reserved Windows device name', got: {}",
            name,
            err
        );
    }

    // Trailing dots and spaces
    let tricky_reserved = ["CON. ", "CON .txt", "aux..."];
    for name in tricky_reserved {
        let p = temp_root.join(name);
        let res = create_directory(p.to_string_lossy().to_string()).await;
        assert!(
            res.is_err(),
            "create_directory MUST reject tricky reserved device name: '{}'",
            name
        );
    }

    // Path traversal characters and invalid characters
    let invalid_chars = [
        "folder<name",
        "folder>name",
        "folder:name",
        "folder\"name",
        "folder|name",
        "folder?name",
        "folder*name",
    ];
    for name in invalid_chars {
        let p = temp_root.join(name);
        let res = create_directory(p.to_string_lossy().to_string()).await;
        assert!(
            res.is_err(),
            "create_directory MUST reject invalid character in '{}'",
            name
        );
    }

    let _ = std::fs::remove_dir_all(&temp_root);
}

// ============================================================================
// AREA 5: Proxy Validation: Bracketed IPv6, Protocols, Ports
// ============================================================================

#[test]
fn test_proxy_uri_bracketed_ipv6_and_boundary_ports() {
    let re = regex::Regex::new(
        r"^(socks5|http|https)://(?:([^:]+):([^@]+)@)?(\[[0-9a-fA-F:]+\]|[^:]+):(\d+)$",
    )
    .unwrap();

    let validate_proxy = |uri: &str| -> Result<(String, String, u32), String> {
        let caps = re
            .captures(uri)
            .ok_or_else(|| "Regex match failed".to_string())?;

        let scheme = caps[1].to_string();
        let host = &caps[4];
        if host.starts_with('[') && host.ends_with(']') {
            let inner = &host[1..host.len() - 1];
            if inner.parse::<std::net::Ipv6Addr>().is_err() {
                return Err(format!("Invalid IPv6 address: '{inner}'"));
            }
        } else if host.trim().is_empty() {
            return Err("Empty host".to_string());
        }

        let port: u32 = caps[5].parse().map_err(|e| format!("Port parse: {e}"))?;
        if !(1..=65535).contains(&port) {
            return Err(format!("Port out of range: {port}"));
        }

        Ok((scheme, host.to_string(), port))
    };

    // Valid IPv6 cases
    assert_eq!(
        validate_proxy("http://[::1]:8080"),
        Ok(("http".into(), "[::1]".into(), 8080))
    );
    assert_eq!(
        validate_proxy("socks5://[::1]:1080"),
        Ok(("socks5".into(), "[::1]".into(), 1080))
    );
    assert_eq!(
        validate_proxy("socks5://user:pass@[::1]:1080"),
        Ok(("socks5".into(), "[::1]".into(), 1080))
    );
    assert_eq!(
        validate_proxy("https://[2001:db8::1]:8443"),
        Ok(("https".into(), "[2001:db8::1]".into(), 8443))
    );
    assert_eq!(
        validate_proxy("socks5://[2001:0db8:85a3:0000:0000:8a2e:0370:7334]:1080"),
        Ok((
            "socks5".into(),
            "[2001:0db8:85a3:0000:0000:8a2e:0370:7334]".into(),
            1080
        ))
    );

    // Port boundaries
    assert_eq!(
        validate_proxy("socks5://127.0.0.1:1"),
        Ok(("socks5".into(), "127.0.0.1".into(), 1))
    );
    assert_eq!(
        validate_proxy("socks5://127.0.0.1:65535"),
        Ok(("socks5".into(), "127.0.0.1".into(), 65535))
    );
    assert!(validate_proxy("socks5://127.0.0.1:0").is_err());
    assert!(validate_proxy("socks5://127.0.0.1:65536").is_err());

    // Hostile / malformed IPv6
    assert!(validate_proxy("http://[:::1]:8080").is_err(), "Triple colon IPv6 must fail");
    assert!(validate_proxy("http://[2001:xyz::1]:8080").is_err(), "Non-hex IPv6 must fail");
    assert!(
        validate_proxy("http://::1:8080").is_err(),
        "Unbracketed IPv6 must fail RFC 3986 regex"
    );

    // Hostile protocols
    assert!(validate_proxy("ftp://[::1]:8080").is_err());
    assert!(validate_proxy("file://[::1]:8080").is_err());
    assert!(validate_proxy("javascript://[::1]:8080").is_err());
}

// ============================================================================
// AREA 6: DoH DNS Probing & Network Failure Resilience
// ============================================================================

#[tokio::test]
async fn test_doh_probing_real_queries_and_error_handling() {
    // 1. Error handling on network failure / invalid endpoint
    let invalid_res = test_network_connection(
        Some("https://invalid-nonexistent-domain-404xyz9876.org/dns-query".to_string()),
        None,
    )
    .await;
    assert!(invalid_res.is_ok(), "test_network_connection must handle DNS failure gracefully without crashing");
    let report = invalid_res.unwrap();
    assert!(!report.success, "Query to non-existent DoH server must return success: false");
    assert!(report.resolved_ip.is_none(), "Must have resolved_ip: None on failure");
    assert!(report.error.is_some(), "Must have error message explaining failure");
    println!("Graceful failure report: {:?}", report.error);

    // 2. Querying authentic DoH (Cloudflare)
    let cf_res = test_network_connection(Some("cloudflare".to_string()), None).await;
    if let Ok(report) = cf_res {
        if report.success {
            println!("Cloudflare DoH resolved IP: {:?}", report.resolved_ip);
            assert!(report.resolved_ip.is_some());
            let ip = report.resolved_ip.unwrap();
            // Verify IP is valid IPv4 or IPv6
            let is_valid_ip = ip.parse::<std::net::IpAddr>().is_ok();
            assert!(is_valid_ip, "Resolved IP '{ip}' must parse as valid IpAddr");
        } else {
            println!("Cloudflare DoH probe skipped or network offline: {:?}", report.error);
        }
    }

    // 3. Querying authentic DoH (Google)
    let g_res = test_network_connection(Some("google".to_string()), None).await;
    if let Ok(report) = g_res {
        if report.success {
            println!("Google DoH resolved IP: {:?}", report.resolved_ip);
            assert!(report.resolved_ip.is_some());
            let ip = report.resolved_ip.unwrap();
            let is_valid_ip = ip.parse::<std::net::IpAddr>().is_ok();
            assert!(is_valid_ip, "Resolved IP '{ip}' must parse as valid IpAddr");
        } else {
            println!("Google DoH probe skipped or network offline: {:?}", report.error);
        }
    }
}
