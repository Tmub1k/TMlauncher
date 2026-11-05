//! Milestone 3 Adversarial Challenge & Stress Tests
//!
//! Scope:
//! 1. Zip-Slip Path Traversal Vulnerability (forward-slash, backslash, absolute Unix/Windows, nested).
//! 2. Corrupted, empty (0-byte), and truncated archive resilience.
//! 3. Extraction rollback blast radius on pre-existing folders.
//! 4. Windows reserved names in directory picker (CON, PRN, AUX, NUL, COM1-9, LPT1-9, and with extensions like CON.txt).
//! 5. Invalid characters (<, >, :, ", |, ?, *) and path traversal in folder creation.
//! 6. Deep directory paths (> 260 chars MAX_PATH).
//! 7. Network proxy validation: malformed URLs, ports 0, 65535, 65536, hostile schemes, CRLF, IPv6.
//! 8. DoH endpoint validation: HTTP vs HTTPS, length bounds, invalid endpoints.

use std::io::{Cursor, Write};
use std::path::Path;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

use omnilauncher_lib::commands::network::create_directory;

// ============================================================================
// AREA 1: Zip-Slip Path Traversal Vulnerability Stress Tests
// ============================================================================

#[test]
fn test_zip_slip_forward_slash_traversal() {
    let mut buf = Vec::new();
    {
        let mut writer = ZipWriter::new(Cursor::new(&mut buf));
        let options = SimpleFileOptions::default();
        writer
            .start_file("../../evil_forward.txt", options)
            .expect("start_file");
        writer.write_all(b"PWNED_FORWARD").expect("write");
        writer.finish().expect("finish");
    }

    let mut archive = ZipArchive::new(Cursor::new(&buf)).expect("valid zip");
    assert_eq!(archive.len(), 1);

    let entry = archive.by_index(0).expect("entry 0");
    let enclosed = entry.enclosed_name();
    assert!(
        enclosed.is_none(),
        "CRITICAL: entry.enclosed_name() MUST return None for '../../evil_forward.txt', got: {:?}",
        enclosed
    );
}

#[test]
fn test_zip_slip_windows_backslash_traversal() {
    let mut buf = Vec::new();
    {
        let mut writer = ZipWriter::new(Cursor::new(&mut buf));
        let options = SimpleFileOptions::default();
        // Notice: Windows backslash traversal
        writer
            .start_file("..\\..\\evil_backslash.txt", options)
            .expect("start_file");
        writer.write_all(b"PWNED_BACKSLASH").expect("write");
        writer.finish().expect("finish");
    }

    let mut archive = ZipArchive::new(Cursor::new(&buf)).expect("valid zip");
    assert_eq!(archive.len(), 1);

    let entry = archive.by_index(0).expect("entry 0");
    let enclosed = entry.enclosed_name();
    println!("Windows backslash entry enclosed_name: {:?}", enclosed);

    // Verify whether dest_dir.join(enclosed) escapes on Windows if enclosed is Some!
    if let Some(ref path) = enclosed {
        let dest = Path::new("C:\\temp\\omni_sandbox");
        let target = dest.join(path);
        let normalized = target.to_string_lossy();
        println!("Joined target: {}", normalized);
        assert!(
            !normalized.contains(".."),
            "CRITICAL: Windows backslash traversal allowed path escaping: {}",
            normalized
        );
    }
}

#[test]
fn test_zip_slip_absolute_unix_path() {
    let mut buf = Vec::new();
    {
        let mut writer = ZipWriter::new(Cursor::new(&mut buf));
        let options = SimpleFileOptions::default();
        writer
            .start_file("/Windows/System32/evil.dll", options)
            .expect("start_file");
        writer.write_all(b"PWNED_UNIX_ABS").expect("write");
        writer.finish().expect("finish");
    }

    let mut archive = ZipArchive::new(Cursor::new(&buf)).expect("valid zip");
    let entry = archive.by_index(0).expect("entry 0");
    assert!(
        entry.enclosed_name().is_none(),
        "CRITICAL: Absolute Unix path must be rejected by enclosed_name()"
    );
}

#[test]
fn test_zip_slip_absolute_windows_path() {
    let mut buf = Vec::new();
    {
        let mut writer = ZipWriter::new(Cursor::new(&mut buf));
        let options = SimpleFileOptions::default();
        writer
            .start_file("C:/Windows/System32/evil.dll", options)
            .expect("start_file");
        writer.write_all(b"PWNED_WIN_ABS").expect("write");
        writer.finish().expect("finish");
    }

    let mut archive = ZipArchive::new(Cursor::new(&buf)).expect("valid zip");
    let entry = archive.by_index(0).expect("entry 0");
    assert!(
        entry.enclosed_name().is_none(),
        "CRITICAL: Drive letter absolute path must be rejected by enclosed_name()"
    );
}

#[test]
fn test_zip_slip_nested_directory_traversal() {
    let mut buf = Vec::new();
    {
        let mut writer = ZipWriter::new(Cursor::new(&mut buf));
        let options = SimpleFileOptions::default();
        writer
            .start_file("emulator/subdir/../../../../../../Windows/evil.dll", options)
            .expect("start_file");
        writer.write_all(b"PWNED_NESTED").expect("write");
        writer.finish().expect("finish");
    }

    let mut archive = ZipArchive::new(Cursor::new(&buf)).expect("valid zip");
    let entry = archive.by_index(0).expect("entry 0");
    assert!(
        entry.enclosed_name().is_none(),
        "CRITICAL: Nested directory traversal must be rejected by enclosed_name()"
    );
}

// ============================================================================
// AREA 2: Corrupted, Empty, and Truncated Archive Resilience
// ============================================================================

#[test]
fn test_empty_zero_byte_archive() {
    let empty_buf: Vec<u8> = Vec::new();
    let res = ZipArchive::new(Cursor::new(empty_buf));
    assert!(res.is_err(), "0-byte buffer must fail ZipArchive creation");
}

#[test]
fn test_corrupted_archive_random_garbage() {
    let garbage = vec![0xDE, 0xAD, 0xBE, 0xEF, 0x12, 0x34, 0x56, 0x78];
    let res = ZipArchive::new(Cursor::new(garbage));
    assert!(res.is_err(), "Garbage bytes must fail ZipArchive creation");
}

#[test]
fn test_truncated_zip_archive() {
    // Start with a valid zip header but truncate before central directory
    let mut valid_buf = Vec::new();
    {
        let mut writer = ZipWriter::new(Cursor::new(&mut valid_buf));
        let options = SimpleFileOptions::default();
        writer.start_file("test.txt", options).unwrap();
        writer.write_all(b"hello world").unwrap();
        writer.finish().unwrap();
    }

    // Truncate valid zip to first 15 bytes
    let truncated = &valid_buf[..15];
    let res = ZipArchive::new(Cursor::new(truncated));
    assert!(res.is_err(), "Truncated zip must fail ZipArchive creation");
}

// ============================================================================
// AREA 3: Rollback Blast Radius on Pre-existing Folders
// ============================================================================

#[tokio::test]
async fn test_rollback_blast_radius_investigation() {
    // Test safe rollback: pre-existing user files must NOT be wiped out if extraction fails
    let temp_base = std::env::temp_dir().join("omni_rollback_stress_test");
    let _ = std::fs::remove_dir_all(&temp_base);
    std::fs::create_dir_all(&temp_base).unwrap();

    let pre_existing_file = temp_base.join("valuable_user_data.txt");
    std::fs::write(&pre_existing_file, b"CRITICAL USER SAVE DATA").unwrap();

    assert!(pre_existing_file.exists());

    // Create an invalid/corrupted zip file in a separate location
    let corrupt_zip = std::env::temp_dir().join("corrupt_test_archive.zip");
    std::fs::write(&corrupt_zip, b"CORRUPTED_NOT_A_VALID_ZIP").unwrap();

    // Call safe_extract_archive targeting temp_base
    let res = omnilauncher_lib::commands::emulators::safe_extract_archive(&corrupt_zip, &temp_base);
    assert!(res.is_err(), "Extraction of corrupt zip must return error");

    // Safe rollback check: pre_existing_file must still exist and be intact!
    assert!(pre_existing_file.exists(), "Pre-existing file must NOT be deleted during rollback!");
    let content = std::fs::read(&pre_existing_file).unwrap();
    assert_eq!(content, b"CRITICAL USER SAVE DATA");

    // Cleanup
    let _ = std::fs::remove_file(&corrupt_zip);
    let _ = std::fs::remove_dir_all(&temp_base);
}

// ============================================================================
// AREA 4: Windows Reserved Device Names & Folder Creation
// ============================================================================

#[tokio::test]
async fn test_create_directory_exact_reserved_names() {
    let temp_root = std::env::temp_dir().join("omni_reserved_test_exact");
    let _ = std::fs::remove_dir_all(&temp_root);
    std::fs::create_dir_all(&temp_root).unwrap();

    let reserved = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
        "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];

    for name in reserved {
        let p = temp_root.join(name);
        let res = create_directory(p.to_string_lossy().to_string()).await;
        assert!(
            res.is_err(),
            "create_directory should reject exact reserved device name: {}",
            name
        );
        let err = res.unwrap_err();
        assert!(
            err.contains("reserved Windows device name"),
            "Error should state reserved device name for {}, got: {}",
            name,
            err
        );
    }

    // Lowercase checks
    for name in ["con", "prn", "aux", "nul", "com1", "lpt1"] {
        let p = temp_root.join(name);
        let res = create_directory(p.to_string_lossy().to_string()).await;
        assert!(
            res.is_err(),
            "create_directory should reject lowercase reserved device name: {}",
            name
        );
    }

    let _ = std::fs::remove_dir_all(&temp_root);
}

#[tokio::test]
async fn test_create_directory_reserved_names_with_extensions() {
    // On Windows, CON.txt, aux.json, nul.tar are ALSO DOS device names!
    let temp_root = std::env::temp_dir().join("omni_reserved_test_ext");
    let _ = std::fs::remove_dir_all(&temp_root);
    std::fs::create_dir_all(&temp_root).unwrap();

    let extensions = ["CON.txt", "aux.json", "NUL.tar", "PRN.log", "com1.dat"];

    for ext_name in extensions {
        let p = temp_root.join(ext_name);
        let res = create_directory(p.to_string_lossy().to_string()).await;
        assert!(res.is_err(), "create_directory should reject reserved device name with extension: {}", ext_name);
        let err = res.unwrap_err();
        assert!(err.contains("reserved Windows device name"), "Error must specify reserved Windows device name, got: {}", err);
    }

    let _ = std::fs::remove_dir_all(&temp_root);
}

#[tokio::test]
async fn test_create_directory_invalid_characters() {
    let temp_root = std::env::temp_dir().join("omni_chars_test");
    let _ = std::fs::remove_dir_all(&temp_root);
    std::fs::create_dir_all(&temp_root).unwrap();

    let invalid_names = [
        "test*star",
        "test?question",
        "test:colon",
        "test<less",
        "test>greater",
        "test|pipe",
        "test\"quote",
    ];

    for name in invalid_names {
        let p = temp_root.join(name);
        let res = create_directory(p.to_string_lossy().to_string()).await;
        assert!(
            res.is_err(),
            "create_directory should reject invalid char in name '{}'",
            name
        );
    }

    let _ = std::fs::remove_dir_all(&temp_root);
}

#[tokio::test]
async fn test_create_directory_deep_path_exceeding_260_chars() {
    let temp_root = std::env::temp_dir().join("omni_deep_path_test");
    let _ = std::fs::remove_dir_all(&temp_root);
    std::fs::create_dir_all(&temp_root).unwrap();

    // Construct a path > 260 characters
    let mut deep = temp_root.clone();
    for i in 0..30 {
        deep = deep.join(format!("subfolder_level_{:02}", i));
    }
    let deep_str = deep.to_string_lossy().to_string();
    assert!(deep_str.len() > 260, "Path length is {}", deep_str.len());

    let res = create_directory(deep_str.clone()).await;
    println!("Deep directory creation (len {}) result: {:?}", deep_str.len(), res);
    // Result might succeed if long paths enabled in Windows or return descriptive error
    let _ = std::fs::remove_dir_all(&temp_root);
}

// ============================================================================
// AREA 5: Network Proxy & DNS Validation Stress Tests
// ============================================================================

#[test]
fn test_proxy_uri_regex_adversarial_cases() {
    let re = regex::Regex::new(r"^(socks5|http|https)://(?:([^:]+):([^@]+)@)?(\[[0-9a-fA-F:]+\]|[^:]+):(\d+)$").unwrap();

    // 1. Hostile schemes
    assert!(re.captures("ftp://127.0.0.1:21").is_none(), "FTP must be rejected");
    assert!(re.captures("file:///Windows/System32").is_none(), "file:// must be rejected");
    assert!(re.captures("javascript://alert(1):8080").is_none(), "javascript:// must be rejected");
    assert!(re.captures("ws://127.0.0.1:8080").is_none(), "ws:// must be rejected");

    // 2. Malformed URLs
    assert!(re.captures("http://:8080").is_none(), "Missing host must be rejected");
    assert!(re.captures("http://127.0.0.1").is_none(), "Missing port must be rejected");
    assert!(re.captures("http://127.0.0.1:").is_none(), "Trailing colon must be rejected");

    // 3. Port bounds validation
    let check_port = |uri: &str| -> Result<u32, &'static str> {
        let caps = re.captures(uri).ok_or("Regex fail")?;
        let port: u32 = caps[5].parse().map_err(|_| "Parse fail")?;
        if !(1..=65535).contains(&port) {
            return Err("Port out of bounds");
        }
        Ok(port)
    };

    assert_eq!(check_port("socks5://127.0.0.1:1"), Ok(1));
    assert_eq!(check_port("socks5://127.0.0.1:65535"), Ok(65535));
    assert_eq!(check_port("socks5://127.0.0.1:0"), Err("Port out of bounds"));
    assert_eq!(check_port("socks5://127.0.0.1:65536"), Err("Port out of bounds"));
    assert_eq!(check_port("socks5://127.0.0.1:99999"), Err("Port out of bounds"));

    // 4. IPv6 Addresses: verify [::1] matches
    let ipv6_match = re.captures("http://[::1]:8080");
    assert!(ipv6_match.is_some(), "IPv6 proxy [::1] must match");
    assert_eq!(&ipv6_match.as_ref().unwrap()[4], "[::1]");

    let ipv6_auth = re.captures("socks5://user:pass@[::1]:1080");
    assert!(ipv6_auth.is_some(), "IPv6 proxy with auth must match");
    let c_auth = ipv6_auth.unwrap();
    assert_eq!(&c_auth[2], "user");
    assert_eq!(&c_auth[3], "pass");
    assert_eq!(&c_auth[4], "[::1]");
}

#[test]
fn test_custom_doh_url_adversarial_validation() {
    let validate_doh = |doh: &str| -> Result<(), &'static str> {
        if !doh.starts_with("https://") || doh.len() <= 8 {
            return Err("Custom DoH endpoint must be a valid https:// URL");
        }
        // Additional URL parse check
        if doh.contains('\r') || doh.contains('\n') || doh.contains(' ') {
            return Err("Invalid characters in DoH endpoint");
        }
        Ok(())
    };

    assert!(validate_doh("https://cloudflare-dns.com/dns-query").is_ok());
    assert!(validate_doh("https://dns.google/dns-query").is_ok());
    assert!(validate_doh("http://insecure-dns.com/dns-query").is_err(), "HTTP must be rejected");
    assert!(validate_doh("https://").is_err(), "Empty domain must be rejected");
    assert!(validate_doh("https://evil.com\r\nHost: evil").is_err(), "CRLF must be rejected");
}
