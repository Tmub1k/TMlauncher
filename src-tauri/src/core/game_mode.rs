//! Zero-Footprint Game Mode controller (`core/game_mode.rs`).
//!
//! Minimizes launcher CPU and memory footprint during gameplay sessions by:
//! 1. Lowering current process priority to `IDLE_PRIORITY_CLASS`.
//! 2. Flushing memory working set pages via Win32 `SetProcessWorkingSetSize`.
//! 3. Emitting `gamemode://status` to throttle frontend rendering and timers.
//! 4. Restoring `NORMAL_PRIORITY_CLASS` when games terminate.

use std::sync::atomic::{AtomicBool, Ordering};
use serde::{Deserialize, Serialize};

static GAME_MODE_ACTIVE: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProcessPriorityLevel {
    IdlePriorityClass,
    BelowNormalPriorityClass,
    NormalPriorityClass,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameModeStatus {
    pub active: bool,
    pub game_running: bool,
    pub is_hibernating: bool,
    pub active_sessions_count: usize,
    pub process_priority: String,
    pub last_trimmed_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryTrimResult {
    pub success: bool,
    pub bytes_freed: u64,
    pub trimmed_at: i64,
}

/// Sets current process priority class.
pub fn set_process_priority(idle: bool) {
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Threading::{
            GetCurrentProcess, SetPriorityClass, IDLE_PRIORITY_CLASS, NORMAL_PRIORITY_CLASS,
        };

        let prio = if idle {
            IDLE_PRIORITY_CLASS
        } else {
            NORMAL_PRIORITY_CLASS
        };

        unsafe {
            let handle = GetCurrentProcess();
            SetPriorityClass(handle, prio);
        }
    }
    #[cfg(not(windows))]
    {
        let _ = idle;
    }
}

/// Flushes process working set memory via Win32 `SetProcessWorkingSetSize`.
pub fn trim_working_set() -> MemoryTrimResult {
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Threading::GetCurrentProcess;

        extern "system" {
            fn SetProcessWorkingSetSize(
                hprocess: windows_sys::Win32::Foundation::HANDLE,
                dwminimumworkingsetsize: usize,
                dwmaximumworkingsetsize: usize,
            ) -> windows_sys::Win32::Foundation::BOOL;
        }

        let ok = unsafe {
            let handle = GetCurrentProcess();
            // Passing (SIZE_T)-1, (SIZE_T)-1 flushes pages to disk/standby
            SetProcessWorkingSetSize(handle, usize::MAX, usize::MAX)
        };

        MemoryTrimResult {
            success: ok != 0,
            bytes_freed: 0,
            trimmed_at: chrono::Utc::now().timestamp(),
        }
    }

    #[cfg(not(windows))]
    {
        MemoryTrimResult {
            success: true,
            bytes_freed: 0,
            trimmed_at: chrono::Utc::now().timestamp(),
        }
    }
}

pub fn enter_game_mode(sessions_count: usize) -> GameModeStatus {
    GAME_MODE_ACTIVE.store(true, Ordering::SeqCst);
    set_process_priority(true);
    let trim = trim_working_set();

    GameModeStatus {
        active: true,
        game_running: true,
        is_hibernating: true,
        active_sessions_count: sessions_count,
        process_priority: "IDLE_PRIORITY_CLASS".to_string(),
        last_trimmed_at: Some(trim.trimmed_at),
    }
}

pub fn exit_game_mode() -> GameModeStatus {
    GAME_MODE_ACTIVE.store(false, Ordering::SeqCst);
    set_process_priority(false);

    GameModeStatus {
        active: false,
        game_running: false,
        is_hibernating: false,
        active_sessions_count: 0,
        process_priority: "NORMAL_PRIORITY_CLASS".to_string(),
        last_trimmed_at: None,
    }
}

pub fn get_game_mode_status(sessions_count: usize) -> GameModeStatus {
    let active = GAME_MODE_ACTIVE.load(Ordering::SeqCst);
    GameModeStatus {
        active,
        game_running: active,
        is_hibernating: active,
        active_sessions_count: sessions_count,
        process_priority: if active {
            "IDLE_PRIORITY_CLASS".to_string()
        } else {
            "NORMAL_PRIORITY_CLASS".to_string()
        },
        last_trimmed_at: None,
    }
}
