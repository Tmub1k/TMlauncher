//! Lightweight, non-intrusive media capture.
//!
//! * **Screenshots** – `windows-capture` grabs one GPU frame of the monitor
//!   (no injection, no overlay). If that API is unavailable we fall back to an
//!   FFmpeg `ddagrab` (DXGI Desktop Duplication) single-frame grab. Frames are
//!   re-encoded to lossless WebP on the blocking pool.
//! * **Clips** – an FFmpeg sidecar captures with `ddagrab` (frames stay on the
//!   GPU as D3D11 textures) and encodes with NVENC / AMF / QSV, falling back
//!   to libx264. Output is fragmented MP4, so a crash never corrupts a clip.
//! * **Hotkeys** – registered through `tauri-plugin-global-shortcut`
//!   (built on the `global-hotkey` crate) and rebindable at runtime.
//!
//! Every artifact gets a `<file>.json` sidecar with game metadata so the
//! gallery can group media without a database.

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::Stdio,
    str::FromStr,
    sync::{Arc, RwLock as StdRwLock},
    time::Duration,
};

use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use tokio::{
    io::AsyncWriteExt,
    process::{Child, Command},
    sync::{Mutex, OnceCell, RwLock},
};

use crate::error::{IoResultExt, LauncherError, Result};

pub const EVT_CAPTURED: &str = "media://captured";
pub const EVT_RECORDING: &str = "media://recording";
pub const EVT_ERROR: &str = "media://error";

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScreenshotFormat {
    Webp,
    Png,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HwEncoder {
    Nvenc,
    Amf,
    Qsv,
    Software,
}

impl HwEncoder {
    /// Probe order: dedicated GPU encoders first, CPU last.
    const PROBE_ORDER: [HwEncoder; 4] = [Self::Nvenc, Self::Amf, Self::Qsv, Self::Software];

    /// Filter + codec arguments that consume `ddagrab`'s D3D11 frames.
    fn ffmpeg_args(self) -> &'static [&'static str] {
        match self {
            // NVENC and AMF accept D3D11 surfaces directly: zero-copy.
            Self::Nvenc => &[
                "-c:v", "h264_nvenc", "-preset", "p4", "-tune", "ll", "-rc", "vbr", "-cq", "21", "-b:v", "0",
            ],
            Self::Amf => &[
                "-c:v", "h264_amf", "-usage", "lowlatency", "-quality", "speed", "-rc", "cqp", "-qp_i", "20",
                "-qp_p", "22",
            ],
            // QSV needs the D3D11 device re-derived as a QSV device (still on GPU).
            Self::Qsv => &[
                "-vf", "hwmap=derive_device=qsv,format=qsv", "-c:v", "h264_qsv", "-preset", "veryfast",
                "-global_quality", "23",
            ],
            // Last resort: download to system memory and encode on the CPU.
            Self::Software => &[
                "-vf", "hwdownload,format=bgra,format=yuv420p", "-c:v", "libx264", "-preset", "ultrafast",
                "-crf", "23",
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CaptureConfig {
    /// Accelerator syntax understood by `global-hotkey`, e.g. `F12`, `Ctrl+Shift+R`.
    pub screenshot_hotkey: String,
    pub record_hotkey: String,
    pub output_dir: PathBuf,
    pub ffmpeg_path: PathBuf,
    pub framerate: u32,
    /// 0 = primary monitor.
    pub monitor_index: u32,
    pub screenshot_format: ScreenshotFormat,
    /// Skip auto-detection and force an encoder.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_encoder: Option<HwEncoder>,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        let output_dir = dirs::picture_dir()
            .or_else(dirs::home_dir)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("OmniLauncher");
        Self {
            screenshot_hotkey: "F12".into(),
            record_hotkey: "Ctrl+Shift+R".into(),
            output_dir,
            ffmpeg_path: default_ffmpeg_path(),
            framerate: 60,
            monitor_index: 0,
            screenshot_format: ScreenshotFormat::Webp,
            preferred_encoder: None,
        }
    }
}

/// Prefer a bundled sidecar (`ffmpeg.exe` next to the launcher), else `PATH`.
fn default_ffmpeg_path() -> PathBuf {
    let exe_name = if cfg!(windows) { "ffmpeg.exe" } else { "ffmpeg" };
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|dir| dir.join(exe_name)))
        .filter(|p| p.exists())
        .unwrap_or_else(|| PathBuf::from(exe_name))
}

// ---------------------------------------------------------------------------
// Public data
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameTag {
    pub game_id: String,
    pub title: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaKind {
    Screenshot,
    Clip,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaItem {
    pub path: PathBuf,
    pub kind: MediaKind,
    pub game_id: Option<String>,
    pub game_title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub size_bytes: u64,
    pub encoder: Option<HwEncoder>,
    pub duration_secs: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingStatus {
    pub recording: bool,
    pub path: Option<PathBuf>,
    pub started_at: Option<DateTime<Utc>>,
    pub encoder: Option<HwEncoder>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum RecordingToggle {
    Started { status: RecordingStatus },
    Stopped { clip: MediaItem },
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
enum HotkeyAction {
    Screenshot,
    ToggleRecording,
}

#[derive(Default)]
struct Bindings {
    screenshot: Option<Shortcut>,
    record: Option<Shortcut>,
}

struct ActiveRecording {
    child: Child,
    path: PathBuf,
    tag: Option<GameTag>,
    started_at: DateTime<Utc>,
    encoder: HwEncoder,
}

pub struct MediaCapture {
    app: AppHandle,
    config: RwLock<CaptureConfig>,
    /// Read from the synchronous shortcut callback, hence a std lock that is
    /// never held across an `.await`.
    bindings: StdRwLock<Bindings>,
    current_game: RwLock<Option<GameTag>>,
    recording: Mutex<Option<ActiveRecording>>,
    detected_encoder: OnceCell<HwEncoder>,
    screenshot_gate: Mutex<()>,
}

impl MediaCapture {
    pub fn new(app: AppHandle, config: CaptureConfig) -> Self {
        Self {
            app,
            config: RwLock::new(config),
            bindings: StdRwLock::new(Bindings::default()),
            current_game: RwLock::new(None),
            recording: Mutex::new(None),
            detected_encoder: OnceCell::new(),
            screenshot_gate: Mutex::new(()),
        }
    }

    pub async fn config(&self) -> CaptureConfig {
        self.config.read().await.clone()
    }

    /// Validate, rebind hotkeys, then swap the active configuration.
    pub async fn update_config(&self, next: CaptureConfig) -> Result<()> {
        if !(1..=240).contains(&next.framerate) {
            return Err(LauncherError::InvalidInput("framerate must be between 1 and 240".into()));
        }
        self.register_hotkeys(&next)?;
        *self.config.write().await = next;
        Ok(())
    }

    /// Tag subsequent captures with the game that is currently running.
    pub async fn set_current_game(&self, tag: Option<GameTag>) {
        *self.current_game.write().await = tag;
    }

    // ---- Hotkeys -------------------------------------------------------------

    /// (Re)register the global hotkeys. Synchronous so it can run in `setup`.
    pub fn register_hotkeys(&self, cfg: &CaptureConfig) -> Result<()> {
        let screenshot = parse_shortcut(&cfg.screenshot_hotkey)?;
        let record = parse_shortcut(&cfg.record_hotkey)?;
        if screenshot == record {
            return Err(LauncherError::InvalidInput(
                "screenshot and recording hotkeys must differ".into(),
            ));
        }

        let gs = self.app.global_shortcut();
        let mut bindings = self.bindings.write().unwrap_or_else(|p| p.into_inner());
        for old in [bindings.screenshot.take(), bindings.record.take()].into_iter().flatten() {
            let _ = gs.unregister(old);
        }

        let mut failures = Vec::new();
        match gs.register(screenshot.clone()) {
            Ok(()) => bindings.screenshot = Some(screenshot),
            Err(e) => failures.push(format!("`{}`: {e}", cfg.screenshot_hotkey)),
        }
        match gs.register(record.clone()) {
            Ok(()) => bindings.record = Some(record),
            Err(e) => failures.push(format!("`{}`: {e}", cfg.record_hotkey)),
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(LauncherError::Hotkey(failures.join("; ")))
        }
    }

    fn action_for(&self, shortcut: &Shortcut) -> Option<HotkeyAction> {
        let b = self.bindings.read().unwrap_or_else(|p| p.into_inner());
        if b.screenshot.as_ref() == Some(shortcut) {
            Some(HotkeyAction::Screenshot)
        } else if b.record.as_ref() == Some(shortcut) {
            Some(HotkeyAction::ToggleRecording)
        } else {
            None
        }
    }

    async fn run_action(self: Arc<Self>, action: HotkeyAction) {
        let outcome = match action {
            HotkeyAction::Screenshot => self.take_screenshot().await.map(|_| ()),
            HotkeyAction::ToggleRecording => self.toggle_recording().await.map(|_| ()),
        };
        if let Err(err) = outcome {
            tracing::error!(?action, error = %err, "hotkey capture failed");
            let _ = self.app.emit(EVT_ERROR, err.payload());
        }
    }

    // ---- Screenshots -----------------------------------------------------------

    pub async fn take_screenshot(&self) -> Result<MediaItem> {
        // Debounce key-repeat: one screenshot at a time.
        let _gate = self.screenshot_gate.lock().await;
        let cfg = self.config().await;
        let tag = self.current_game.read().await.clone();

        let dir = media_dir(&cfg, MediaKind::Screenshot, tag.as_ref());
        tokio::fs::create_dir_all(&dir).await.with_path(&dir)?;
        let png_path = dir.join(format!("{}.png", file_stamp()));

        self.grab_frame_png(&cfg, &png_path).await?;

        let final_path = match cfg.screenshot_format {
            ScreenshotFormat::Png => png_path,
            ScreenshotFormat::Webp => transcode_to_webp(png_path).await?,
        };

        let item = MediaItem {
            size_bytes: file_size(&final_path).await,
            path: final_path,
            kind: MediaKind::Screenshot,
            game_id: tag.as_ref().map(|t| t.game_id.clone()),
            game_title: tag.map(|t| t.title),
            created_at: Utc::now(),
            encoder: None,
            duration_secs: None,
        };
        write_sidecar(&item).await?;
        let _ = self.app.emit(EVT_CAPTURED, &item);
        Ok(item)
    }

    async fn grab_frame_png(&self, cfg: &CaptureConfig, target: &Path) -> Result<()> {
        #[cfg(windows)]
        {
            let out = target.to_path_buf();
            let monitor = cfg.monitor_index;
            match tokio::task::spawn_blocking(move || gpu_grab::capture_monitor_png(monitor, &out)).await? {
                Ok(()) => return Ok(()),
                Err(err) => tracing::warn!(error = %err, "windows-capture failed, falling back to ddagrab"),
            }
        }
        ffmpeg_snapshot(cfg, target).await
    }

    // ---- Recording -------------------------------------------------------------

    pub async fn toggle_recording(&self) -> Result<RecordingToggle> {
        let active = self.recording.lock().await.is_some();
        let toggle = if active {
            RecordingToggle::Stopped {
                clip: self.stop_recording().await?,
            }
        } else {
            RecordingToggle::Started {
                status: self.start_recording().await?,
            }
        };
        let _ = self.app.emit(EVT_RECORDING, &toggle);
        Ok(toggle)
    }

    pub async fn status(&self) -> RecordingStatus {
        match self.recording.lock().await.as_ref() {
            Some(r) => RecordingStatus {
                recording: true,
                path: Some(r.path.clone()),
                started_at: Some(r.started_at),
                encoder: Some(r.encoder),
            },
            None => RecordingStatus {
                recording: false,
                path: None,
                started_at: None,
                encoder: None,
            },
        }
    }

    pub async fn start_recording(&self) -> Result<RecordingStatus> {
        let mut slot = self.recording.lock().await;
        if slot.is_some() {
            return Err(LauncherError::InvalidInput("a recording is already running".into()));
        }

        let cfg = self.config().await;
        let encoder = self.resolve_encoder(&cfg).await?;
        let tag = self.current_game.read().await.clone();

        let dir = media_dir(&cfg, MediaKind::Clip, tag.as_ref());
        tokio::fs::create_dir_all(&dir).await.with_path(&dir)?;
        let path = dir.join(format!("{}.mp4", file_stamp()));

        let mut cmd = ffmpeg_command(&cfg);
        cmd.args(recording_args(encoder, &cfg, &path))
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        let mut child = cmd.spawn().map_err(|e| ffmpeg_spawn_error(&cfg, e))?;

        // Catch immediate failures (bad monitor index, encoder refused, …).
        tokio::time::sleep(Duration::from_millis(400)).await;
        if let Some(status) = child.try_wait()? {
            return Err(LauncherError::Encoder(format!(
                "FFmpeg exited immediately ({status}) using {encoder:?}"
            )));
        }

        let started_at = Utc::now();
        tracing::info!(?encoder, path = %path.display(), "recording started");
        *slot = Some(ActiveRecording {
            child,
            path: path.clone(),
            tag,
            started_at,
            encoder,
        });

        Ok(RecordingStatus {
            recording: true,
            path: Some(path),
            started_at: Some(started_at),
            encoder: Some(encoder),
        })
    }

    pub async fn stop_recording(&self) -> Result<MediaItem> {
        let mut rec = self
            .recording
            .lock()
            .await
            .take()
            .ok_or_else(|| LauncherError::InvalidInput("no recording is running".into()))?;

        // `q` on stdin asks FFmpeg to flush and finalize gracefully.
        if let Some(mut stdin) = rec.child.stdin.take() {
            let _ = stdin.write_all(b"q").await;
            let _ = stdin.flush().await;
        }
        match tokio::time::timeout(Duration::from_secs(8), rec.child.wait()).await {
            Ok(Ok(status)) if !status.success() => {
                tracing::warn!(%status, "FFmpeg exited with non-zero status")
            }
            Ok(Ok(_)) => {}
            Ok(Err(e)) => tracing::warn!(error = %e, "waiting for FFmpeg failed"),
            Err(_) => {
                tracing::warn!("FFmpeg did not stop in time; killing (fragmented MP4 stays playable)");
                let _ = rec.child.kill().await;
            }
        }

        let ended = Utc::now();
        let item = MediaItem {
            size_bytes: file_size(&rec.path).await,
            path: rec.path,
            kind: MediaKind::Clip,
            game_id: rec.tag.as_ref().map(|t| t.game_id.clone()),
            game_title: rec.tag.map(|t| t.title),
            created_at: rec.started_at,
            encoder: Some(rec.encoder),
            duration_secs: Some((ended - rec.started_at).num_milliseconds() as f64 / 1000.0),
        };
        write_sidecar(&item).await?;
        let _ = self.app.emit(EVT_CAPTURED, &item);
        Ok(item)
    }

    async fn resolve_encoder(&self, cfg: &CaptureConfig) -> Result<HwEncoder> {
        if let Some(forced) = cfg.preferred_encoder {
            return Ok(forced);
        }
        self.detected_encoder
            .get_or_try_init(|| probe_encoders(cfg.clone()))
            .await
            .copied()
    }

    // ---- Gallery ---------------------------------------------------------------

    /// List captured media from the metadata sidecars, newest first.
    pub async fn list_media(&self) -> Result<Vec<MediaItem>> {
        let cfg = self.config().await;
        let mut items = Vec::new();

        for kind_dir in ["Screenshots", "Clips"] {
            let root = cfg.output_dir.join(kind_dir);
            let Ok(mut games) = tokio::fs::read_dir(&root).await else { continue };
            while let Some(game_dir) = games.next_entry().await.with_path(&root)? {
                let dir = game_dir.path();
                let Ok(mut files) = tokio::fs::read_dir(&dir).await else { continue };
                while let Some(file) = files.next_entry().await.with_path(&dir)? {
                    let path = file.path();
                    if path.extension().and_then(|e| e.to_str()) != Some("json") {
                        continue;
                    }
                    let Ok(raw) = tokio::fs::read(&path).await else { continue };
                    match serde_json::from_slice::<MediaItem>(&raw) {
                        Ok(item) if tokio::fs::try_exists(&item.path).await.unwrap_or(false) => items.push(item),
                        Ok(_) => {}
                        Err(e) => tracing::debug!(path = %path.display(), error = %e, "bad media sidecar"),
                    }
                }
            }
        }

        items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(items)
    }
}

/// Entry point for the global-shortcut plugin callback (runs on the main thread).
pub fn handle_shortcut(app: &AppHandle, shortcut: &Shortcut) {
    let Some(state) = app.try_state::<Arc<MediaCapture>>() else { return };
    let capture = Arc::clone(state.inner());
    if let Some(action) = capture.action_for(shortcut) {
        tauri::async_runtime::spawn(capture.run_action(action));
    }
}

// ---------------------------------------------------------------------------
// FFmpeg plumbing
// ---------------------------------------------------------------------------

fn ffmpeg_command(cfg: &CaptureConfig) -> Command {
    let mut cmd = Command::new(&cfg.ffmpeg_path);
    cmd.kill_on_drop(true);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

fn ffmpeg_spawn_error(cfg: &CaptureConfig, err: std::io::Error) -> LauncherError {
    if err.kind() == std::io::ErrorKind::NotFound {
        LauncherError::Encoder(format!(
            "FFmpeg not found at `{}` (bundle it as a sidecar or add it to PATH)",
            cfg.ffmpeg_path.display()
        ))
    } else {
        LauncherError::Encoder(format!("failed to start FFmpeg: {err}"))
    }
}

fn ddagrab_source(cfg: &CaptureConfig, framerate: u32) -> String {
    format!(
        "ddagrab=output_idx={}:framerate={}:draw_mouse=1",
        cfg.monitor_index, framerate
    )
}

fn recording_args(encoder: HwEncoder, cfg: &CaptureConfig, out: &Path) -> Vec<OsString> {
    let gop = (cfg.framerate * 2).to_string();
    let mut args: Vec<OsString> = ["-hide_banner", "-loglevel", "error", "-y", "-f", "lavfi", "-i"]
        .into_iter()
        .map(OsString::from)
        .collect();
    args.push(ddagrab_source(cfg, cfg.framerate).into());
    args.extend(encoder.ffmpeg_args().iter().map(OsString::from));
    args.extend(
        ["-g", gop.as_str(), "-movflags", "+frag_keyframe+empty_moov+default_base_moof"]
            .into_iter()
            .map(OsString::from),
    );
    args.push(out.as_os_str().to_owned());
    args
}

/// Try each encoder against the real capture pipeline for a few frames.
async fn probe_encoders(cfg: CaptureConfig) -> Result<HwEncoder> {
    let source = ddagrab_source(&cfg, 10);
    for encoder in HwEncoder::PROBE_ORDER {
        let mut cmd = ffmpeg_command(&cfg);
        cmd.args(["-hide_banner", "-loglevel", "error", "-f", "lavfi", "-i", source.as_str()])
            .args(encoder.ffmpeg_args())
            .args(["-frames:v", "3", "-f", "null", "-"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        match tokio::time::timeout(Duration::from_secs(10), cmd.status()).await {
            Ok(Ok(status)) if status.success() => {
                tracing::info!(?encoder, "selected video encoder");
                return Ok(encoder);
            }
            Ok(Err(e)) => return Err(ffmpeg_spawn_error(&cfg, e)),
            _ => tracing::debug!(?encoder, "encoder probe failed"),
        }
    }
    Err(LauncherError::Encoder(
        "no working H.264 encoder (NVENC, AMF, QSV or libx264); FFmpeg ≥ 6.0 is required for ddagrab".into(),
    ))
}

/// Single-frame DXGI grab through FFmpeg (fallback path).
async fn ffmpeg_snapshot(cfg: &CaptureConfig, target: &Path) -> Result<()> {
    let source = ddagrab_source(cfg, 5);
    let mut cmd = ffmpeg_command(cfg);
    cmd.args(["-hide_banner", "-loglevel", "error", "-y", "-f", "lavfi", "-i", source.as_str()])
        .args(["-vf", "hwdownload,format=bgra", "-frames:v", "1"])
        .arg(target)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let status = tokio::time::timeout(Duration::from_secs(10), cmd.status())
        .await
        .map_err(|_| LauncherError::Capture("FFmpeg snapshot timed out".into()))?
        .map_err(|e| ffmpeg_spawn_error(cfg, e))?;
    if status.success() {
        Ok(())
    } else {
        Err(LauncherError::Capture(format!("FFmpeg snapshot failed ({status})")))
    }
}

// ---------------------------------------------------------------------------
// Files & helpers
// ---------------------------------------------------------------------------

fn parse_shortcut(raw: &str) -> Result<Shortcut> {
    Shortcut::from_str(raw.trim())
        .map_err(|e| LauncherError::Hotkey(format!("invalid accelerator `{raw}`: {e}")))
}

fn media_dir(cfg: &CaptureConfig, kind: MediaKind, tag: Option<&GameTag>) -> PathBuf {
    let kind_dir = match kind {
        MediaKind::Screenshot => "Screenshots",
        MediaKind::Clip => "Clips",
    };
    let game_dir = tag.map_or_else(|| "Desktop".to_string(), |t| sanitize_component(&t.title));
    cfg.output_dir.join(kind_dir).join(game_dir)
}

fn file_stamp() -> String {
    Local::now().format("%Y-%m-%d_%H-%M-%S-%3f").to_string()
}

fn sanitize_component(raw: &str) -> String {
    let cleaned: String = raw
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();
    let trimmed = cleaned.trim().trim_end_matches('.');
    if trimmed.is_empty() {
        "Unknown".into()
    } else {
        trimmed.chars().take(80).collect()
    }
}

fn sidecar_path(media: &Path) -> PathBuf {
    let mut s = media.as_os_str().to_owned();
    s.push(".json");
    PathBuf::from(s)
}

async fn write_sidecar(item: &MediaItem) -> Result<()> {
    let path = sidecar_path(&item.path);
    let bytes = serde_json::to_vec_pretty(item)?;
    tokio::fs::write(&path, bytes).await.with_path(&path)
}

async fn file_size(path: &Path) -> u64 {
    tokio::fs::metadata(path).await.map(|m| m.len()).unwrap_or(0)
}

/// PNG → lossless WebP on the blocking pool; removes the intermediate PNG.
async fn transcode_to_webp(png: PathBuf) -> Result<PathBuf> {
    tokio::task::spawn_blocking(move || -> Result<PathBuf> {
        let webp = png.with_extension("webp");
        let img = image::open(&png).map_err(|e| LauncherError::Capture(format!("decode {}: {e}", png.display())))?;
        img.save_with_format(&webp, image::ImageFormat::WebP)
            .map_err(|e| LauncherError::Capture(format!("encode {}: {e}", webp.display())))?;
        std::fs::remove_file(&png).with_path(&png)?;
        Ok(webp)
    })
    .await?
}

// ---------------------------------------------------------------------------
// windows-capture adapter (the only code coupled to that crate's API)
// ---------------------------------------------------------------------------

#[cfg(windows)]
mod gpu_grab {
    use std::path::{Path, PathBuf};

    use windows_capture::{
        capture::{Context, GraphicsCaptureApiHandler},
        frame::{Frame, ImageFormat},
        graphics_capture_api::InternalCaptureControl,
        monitor::Monitor,
        settings::{ColorFormat, CursorCaptureSettings, DrawBorderSettings, Settings},
    };

    use crate::error::{LauncherError, Result};

    type HandlerError = Box<dyn std::error::Error + Send + Sync>;

    /// Captures exactly one frame, writes it, then stops the session.
    struct OneShot {
        target: PathBuf,
    }

    impl GraphicsCaptureApiHandler for OneShot {
        type Flags = PathBuf;
        type Error = HandlerError;

        fn new(ctx: Context<Self::Flags>) -> std::result::Result<Self, Self::Error> {
            Ok(Self { target: ctx.flags })
        }

        fn on_frame_arrived(
            &mut self,
            frame: &mut Frame,
            capture_control: InternalCaptureControl,
        ) -> std::result::Result<(), Self::Error> {
            frame.save_as_image(&self.target, ImageFormat::Png)?;
            capture_control.stop();
            Ok(())
        }

        fn on_closed(&mut self) -> std::result::Result<(), Self::Error> {
            Ok(())
        }
    }

    /// Blocking: run on `spawn_blocking`.
    pub fn capture_monitor_png(monitor_index: u32, target: &Path) -> Result<()> {
        let monitor = if monitor_index == 0 {
            Monitor::primary()
        } else {
            // windows-capture monitor indices are 1-based.
            Monitor::from_index(monitor_index as usize + 1)
        }
        .map_err(|e| LauncherError::Capture(format!("monitor {monitor_index}: {e}")))?;

        let settings = Settings::new(
            monitor,
            CursorCaptureSettings::WithoutCursor,
            DrawBorderSettings::WithoutBorder,
            ColorFormat::Rgba8,
            target.to_path_buf(),
        );
        OneShot::start(settings).map_err(|e| LauncherError::Capture(e.to_string()))?;

        if target.exists() {
            Ok(())
        } else {
            Err(LauncherError::Capture("capture session ended without a frame".into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_reserved_characters() {
        assert_eq!(sanitize_component("Half-Life 2: Episode One"), "Half-Life 2_ Episode One");
        assert_eq!(sanitize_component("  ..  "), "Unknown");
    }

    #[test]
    fn recording_args_end_with_output() {
        let cfg = CaptureConfig::default();
        let args = recording_args(HwEncoder::Nvenc, &cfg, Path::new("clip.mp4"));
        assert_eq!(args.last().unwrap(), "clip.mp4");
        assert!(args.iter().any(|a| a == "h264_nvenc"));
    }
}
