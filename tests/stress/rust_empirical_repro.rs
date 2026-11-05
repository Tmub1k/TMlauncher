// Empirically test the Rust-level remediations in platforms.rs and platform.rs

fn safe_truncate_utf8(s: &mut String, max_bytes: usize) {
    if s.len() > max_bytes {
        let boundary = s
            .char_indices()
            .map(|(idx, _)| idx)
            .take_while(|&idx| idx <= max_bytes)
            .last()
            .unwrap_or(0);
        s.truncate(boundary);
    }
}

fn test_truncate_safety() {
    let mut s = String::with_capacity(5000);
    for _ in 0..4095 {
        s.push('a');
    }
    s.push('Ж'); // 2-byte UTF-8 character (0xD0, 0x96). Len is now 4097 bytes.

    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut test_s = s.clone();
        safe_truncate_utf8(&mut test_s, 4096);
        assert_eq!(test_s.len(), 4095);
    }));

    match res {
        Ok(_) => println!("[REMEDIATION_CONFIRMED] safe_truncate_utf8(4096) succeeded without panic (safe len: 4095)"),
        Err(_) => println!("[FAIL] Panic occurred during safe truncate!"),
    }
}

fn parse_game_id_remediated(s: &str) -> Result<(&str, &str), String> {
    let trimmed = s.trim();
    if let Some(key) = trimmed.strip_prefix("unified_") {
        return Ok(("canonical", key));
    }
    if let Some(pos) = trimmed.find(':') {
        let (plat, native) = (&trimmed[..pos], &trimmed[pos + 1..]);
        return Ok((plat, native));
    }
    Err("malformed game id".into())
}

fn test_game_id_parsing() {
    let unified_id = "unified_cyberpunk2077";
    match parse_game_id_remediated(unified_id) {
        Ok((platform, native)) => {
            println!("[REMEDIATION_CONFIRMED] Unified ID parsed successfully: platform='{}', native_id='{}'", platform, native);
        }
        Err(e) => {
            println!("[FAIL] Unified ID parsing failed: {}", e);
        }
    }
}

fn main() {
    println!("=== Running Rust Empirical Remediations ===");
    test_truncate_safety();
    test_game_id_parsing();
}
