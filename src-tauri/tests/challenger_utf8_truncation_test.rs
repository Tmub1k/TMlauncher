//! Challenger Adversarial Tests: Multi-byte UTF-8 Character Boundary Truncation Safety
//!
//! Stress-tests omnilauncher_lib::commands::platforms::safe_truncate_utf8 against
//! boundary collisions with 1, 2, 3, and 4-byte Unicode characters, edge cases,
//! and randomized fuzzing.

use omnilauncher_lib::commands::platforms::safe_truncate_utf8;
use std::panic::{catch_unwind, AssertUnwindSafe};

#[test]
fn test_exact_boundary_no_mutation() {
    let mut s = "hello world".to_string();
    safe_truncate_utf8(&mut s, 11);
    assert_eq!(s, "hello world");
    assert_eq!(s.len(), 11);

    safe_truncate_utf8(&mut s, 100);
    assert_eq!(s, "hello world");
}

#[test]
fn test_zero_max_bytes() {
    let mut s = "🦀🦀🦀".to_string();
    safe_truncate_utf8(&mut s, 0);
    assert_eq!(s, "");
    assert_eq!(s.len(), 0);

    let mut empty = String::new();
    safe_truncate_utf8(&mut empty, 0);
    assert_eq!(empty, "");

    let mut empty2 = String::new();
    safe_truncate_utf8(&mut empty2, 4096);
    assert_eq!(empty2, "");
}

#[test]
fn test_2byte_utf8_boundary_collision() {
    // 'Ж' is 2 bytes (0xD0 0x96)
    // 4095 ASCII 'a' bytes + 1 'Ж' = 4097 bytes.
    let mut s = "a".repeat(4095);
    s.push('Ж');
    assert_eq!(s.len(), 4097);

    // If max_bytes is 4096, cutting at 4096 would split 'Ж' in half!
    // safe_truncate_utf8 must retreat to byte 4095.
    let res = catch_unwind(AssertUnwindSafe(|| {
        let mut test_s = s.clone();
        safe_truncate_utf8(&mut test_s, 4096);
        test_s
    }));

    assert!(res.is_ok(), "safe_truncate_utf8 must not panic on 2-byte boundary");
    let truncated = res.unwrap();
    assert_eq!(truncated.len(), 4095);
    assert_eq!(truncated, "a".repeat(4095));

    // If max_bytes is 4097, it can keep 'Ж'
    let mut test_s2 = s.clone();
    safe_truncate_utf8(&mut test_s2, 4097);
    assert_eq!(test_s2.len(), 4097);
    assert!(test_s2.ends_with('Ж'));
}

#[test]
fn test_3byte_utf8_boundary_collision() {
    // '語' is 3 bytes (0xE8 0xAA 0x9E)
    // Prefix with 10 ASCII characters -> indices: 0..9 are 'x', 10..12 is '語'
    let base = "xxxxxxxxxx"; // 10 bytes
    let mut s = base.to_string();
    s.push('語'); // 13 bytes total

    // Split at byte 11 (1 byte into '語')
    let mut s11 = s.clone();
    safe_truncate_utf8(&mut s11, 11);
    assert_eq!(s11, base, "Should retreat to 10 bytes on byte 11 cut");

    // Split at byte 12 (2 bytes into '語')
    let mut s12 = s.clone();
    safe_truncate_utf8(&mut s12, 12);
    assert_eq!(s12, base, "Should retreat to 10 bytes on byte 12 cut");

    // Cut at byte 13 (exact end of '語')
    let mut s13 = s.clone();
    safe_truncate_utf8(&mut s13, 13);
    assert_eq!(s13, s, "Should keep entire character on byte 13 cut");
}

#[test]
fn test_4byte_utf8_boundary_collision() {
    // '🦀' is 4 bytes (0xF0 0x9F 0xA6 0x80)
    // 4093 ASCII bytes + '🦀' = 4097 bytes total.
    let prefix = "c".repeat(4093);
    let mut s = prefix.clone();
    s.push('🦀'); // 4097 bytes

    // Case 1: cut at 4094 (1st byte into crab)
    let mut s4094 = s.clone();
    safe_truncate_utf8(&mut s4094, 4094);
    assert_eq!(s4094.len(), 4093);
    assert_eq!(s4094, prefix);

    // Case 2: cut at 4095 (2nd byte into crab)
    let mut s4095 = s.clone();
    safe_truncate_utf8(&mut s4095, 4095);
    assert_eq!(s4095.len(), 4093);
    assert_eq!(s4095, prefix);

    // Case 3: cut at 4096 (3rd byte into crab) - canonical 4096 launch_args limit!
    let mut s4096 = s.clone();
    let res = catch_unwind(AssertUnwindSafe(|| {
        safe_truncate_utf8(&mut s4096, 4096);
    }));
    assert!(res.is_ok(), "Must not panic when 4096 cuts 3rd byte of 4-byte emoji");
    assert_eq!(s4096.len(), 4093);
    assert_eq!(s4096, prefix);

    // Case 4: cut at 4097 (exact crab boundary)
    let mut s4097 = s.clone();
    safe_truncate_utf8(&mut s4097, 4097);
    assert_eq!(s4097.len(), 4097);
    assert_eq!(s4097, s);
}

#[test]
fn test_pure_multibyte_string_truncation() {
    // 10 4-byte emojis in SMP (Supplementary Multilingual Plane)
    // 🎮 = U+1F3AE (4 bytes)
    // 🚀 = U+1F680 (4 bytes)
    // 🦀 = U+1F980 (4 bytes)
    // 🔥 = U+1F525 (4 bytes)
    // 👾 = U+1F47E (4 bytes)
    // 🎯 = U+1F3AF (4 bytes)
    // 🏆 = U+1F3C6 (4 bytes)
    // 💎 = U+1F48E (4 bytes)
    // 🦄 = U+1F984 (4 bytes)
    // 🌟 = U+1F31F (4 bytes)
    let emojis = "🎮🚀🦀🔥👾🎯🏆💎🦄🌟";
    assert_eq!(emojis.chars().count(), 10);
    assert_eq!(emojis.len(), 40);

    for max_bytes in 0..=45 {
        let mut s = emojis.to_string();
        safe_truncate_utf8(&mut s, max_bytes);
        assert!(s.len() <= max_bytes);
        assert_eq!(s.len() % 4, 0, "4-byte emoji string should only cut on 4-byte multiples");
        // Check valid UTF-8
        assert!(std::str::from_utf8(s.as_bytes()).is_ok());
    }
}

#[test]
fn test_fuzz_random_unicode_boundaries() {
    let test_chars = [
        'a', 'Z', '9', '-', '_', ' ', // 1-byte
        'ñ', 'é', 'Ж', 'д', 'λ', 'Ω', // 2-byte
        '語', '本', '한', '글', 'ह', 'ी', // 3-byte
        '🦀', '🚀', '🎮', '🌟', '𝄞', '🦄', // 4-byte
    ];

    // Build a large heterogeneous string of 2,000 mixed characters
    let mut rng_state: u64 = 0x12345678_9ABCDEF0;
    let mut next_rand = || {
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (rng_state >> 33) as usize
    };

    let mut corpus = String::with_capacity(10000);
    for _ in 0..2000 {
        let ch = test_chars[next_rand() % test_chars.len()];
        corpus.push(ch);
    }
    let total_len = corpus.len();
    assert!(total_len > 4000);

    // Test 1,000 arbitrary cut points
    for _ in 0..1000 {
        let max_bytes = next_rand() % (total_len + 100);
        let mut s = corpus.clone();

        let res = catch_unwind(AssertUnwindSafe(|| {
            safe_truncate_utf8(&mut s, max_bytes);
        }));

        assert!(res.is_ok(), "Panic during fuzz truncate at max_bytes={}", max_bytes);
        assert!(s.len() <= max_bytes, "Truncated len {} exceeds max_bytes {}", s.len(), max_bytes);
        assert!(std::str::from_utf8(s.as_bytes()).is_ok(), "Invalid UTF-8 produced");
    }
}
