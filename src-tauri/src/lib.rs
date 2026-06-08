use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State, WebviewWindow};

#[derive(Deserialize)]
pub struct ExecOpts {
    pub keys: Vec<String>,
    #[serde(rename = "gameTitle")]
    pub game_title: String,
    #[serde(rename = "resetKey")]
    pub reset_key: String,
    #[serde(rename = "resetDelayMs")]
    pub reset_delay_ms: u64,
    pub countdown: u32,
    #[serde(rename = "delayMs")]
    pub delay_ms: u64,
    #[serde(rename = "holdMs")]
    pub hold_ms: u64,
}

/// Shared cancel flag, set by the Stop button or the global Esc shortcut.
struct AppState {
    abort: Arc<AtomicBool>,
}

#[derive(Clone, Serialize)]
struct Progress {
    phase: String, // "countdown" | "reset" | "playing" | "done" | "stopped" | "error"
    current: u32,
    total: u32,
    message: String,
}

fn emit(window: &WebviewWindow, phase: &str, current: u32, total: u32, message: &str) {
    let _ = window.emit(
        "autoplay",
        Progress {
            phase: phase.to_string(),
            current,
            total,
            message: message.to_string(),
        },
    );
}

#[tauri::command]
fn stop_plan(state: State<AppState>) {
    state.abort.store(true, Ordering::SeqCst);
}

/// Opens an external https URL in the system browser (used for the Ko-fi link).
#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    if !url.starts_with("https://") {
        return Err("Only https URLs are allowed".into());
    }
    #[cfg(windows)]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &url])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(not(windows))]
    {
        let _ = url; // no-op on non-Windows (web build covers those users)
    }
    Ok(())
}

#[tauri::command]
fn execute_plan(
    app: AppHandle,
    window: WebviewWindow,
    state: State<AppState>,
    opts: ExecOpts,
) -> Result<(), String> {
    #[cfg(not(windows))]
    {
        let _ = (&app, &window, &state, &opts);
        return Err(
            "Desktop auto-play is currently Windows-only. On this platform, use the move list \
             from the solver and perform the steps in-game."
                .into(),
        );
    }

    #[cfg(windows)]
    {
        // Validate all plan keys and the reset key up front.
        for k in &opts.keys {
            if winimpl::scancode(k).is_none() {
                return Err(format!("Unknown key in plan: '{}'", k));
            }
        }
        if winimpl::scancode(&opts.reset_key).is_none() {
            return Err(format!("Unknown reset key: '{}'", opts.reset_key));
        }

        state.abort.store(false, Ordering::SeqCst);

        // Register Esc as a global hotkey so it works while the game has focus.
        use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Shortcut};
        let esc = Shortcut::new(None, Code::Escape);
        let _ = app.global_shortcut().register(esc);

        let abort = state.abort.clone();
        let app2 = app.clone();
        std::thread::spawn(move || {
            winimpl::run_plan(&window, opts, abort);
            let esc = Shortcut::new(None, Code::Escape);
            let _ = app2.global_shortcut().unregister(esc);
        });
        Ok(())
    }
}

#[cfg(windows)]
mod winimpl {
    use super::{emit, ExecOpts};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::{thread::sleep, time::Duration};
    use tauri::WebviewWindow;
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
        KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, VIRTUAL_KEY,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowTextLengthW, GetWindowTextW, IsWindowVisible, SetForegroundWindow,
        ShowWindow, SW_RESTORE,
    };

    /// Maps a key token to (scancode, is_extended_key). Accepts a-z plus a few named keys.
    pub fn scancode(key: &str) -> Option<(u16, bool)> {
        let key = key.trim().to_lowercase();
        Some(match key.as_str() {
            "a" => (0x1E, false),
            "b" => (0x30, false),
            "c" => (0x2E, false),
            "d" => (0x20, false),
            "e" => (0x12, false),
            "f" => (0x21, false),
            "g" => (0x22, false),
            "h" => (0x23, false),
            "i" => (0x17, false),
            "j" => (0x24, false),
            "k" => (0x25, false),
            "l" => (0x26, false),
            "m" => (0x32, false),
            "n" => (0x31, false),
            "o" => (0x18, false),
            "p" => (0x19, false),
            "q" => (0x10, false),
            "r" => (0x13, false),
            "s" => (0x1F, false),
            "t" => (0x14, false),
            "u" => (0x16, false),
            "v" => (0x2F, false),
            "w" => (0x11, false),
            "x" => (0x2D, false),
            "y" => (0x15, false),
            "z" => (0x2C, false),
            "space" => (0x39, false),
            "enter" => (0x1C, false),
            "esc" => (0x01, false),
            "up" => (0x48, true),
            "down" => (0x50, true),
            "left" => (0x4B, true),
            "right" => (0x4D, true),
            _ => return None,
        })
    }

    fn send(sc: u16, extended: bool, keyup: bool) {
        let mut flags: KEYBD_EVENT_FLAGS = KEYEVENTF_SCANCODE;
        if keyup {
            flags |= KEYEVENTF_KEYUP;
        }
        if extended {
            flags |= KEYEVENTF_EXTENDEDKEY;
        }
        let input = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(0),
                    wScan: sc,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        unsafe {
            SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        }
    }

    fn press(key: &str, hold_ms: u64) {
        if let Some((sc, ext)) = scancode(key) {
            send(sc, ext, false);
            sleep(Duration::from_millis(hold_ms));
            send(sc, ext, true);
        }
    }

    /// Sleeps up to `ms`, checking the abort flag in small slices.
    /// Returns true if aborted.
    fn sleep_abortable(ms: u64, abort: &Arc<AtomicBool>) -> bool {
        let mut left = ms;
        while left > 0 {
            if abort.load(Ordering::SeqCst) {
                return true;
            }
            let slice = left.min(50);
            sleep(Duration::from_millis(slice));
            left -= slice;
        }
        abort.load(Ordering::SeqCst)
    }

    struct FindCtx {
        needle: String,
        hwnd: HWND,
    }

    unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let ctx = &mut *(lparam.0 as *mut FindCtx);
        if !IsWindowVisible(hwnd).as_bool() {
            return BOOL(1);
        }
        let len = GetWindowTextLengthW(hwnd);
        if len == 0 {
            return BOOL(1);
        }
        let mut buf = vec![0u16; (len + 1) as usize];
        let read = GetWindowTextW(hwnd, &mut buf);
        if read > 0 {
            let title = String::from_utf16_lossy(&buf[..read as usize]);
            if title.to_lowercase().contains(&ctx.needle) {
                ctx.hwnd = hwnd;
                return BOOL(0); // found -> stop enumerating
            }
        }
        BOOL(1)
    }

    fn focus_game(needle: &str) -> bool {
        let mut ctx = FindCtx {
            needle: needle.to_lowercase(),
            hwnd: HWND(std::ptr::null_mut()),
        };
        unsafe {
            let _ = EnumWindows(Some(enum_proc), LPARAM(&mut ctx as *mut _ as isize));
            if !ctx.hwnd.0.is_null() {
                let _ = ShowWindow(ctx.hwnd, SW_RESTORE);
                let _ = SetForegroundWindow(ctx.hwnd);
                return true;
            }
        }
        false
    }

    pub fn run_plan(window: &WebviewWindow, opts: ExecOpts, abort: Arc<AtomicBool>) {
        let total = opts.keys.len() as u32;

        let focused = focus_game(&opts.game_title);

        // Countdown (also a window to alt-tab if focus failed).
        let mut remaining = opts.countdown;
        while remaining > 0 {
            if abort.load(Ordering::SeqCst) {
                emit(window, "stopped", 0, total, "Stopped.");
                return;
            }
            let msg = if focused {
                format!("Starting in {remaining}s — game focused")
            } else {
                format!("Starting in {remaining}s — switch to '{}'", opts.game_title)
            };
            emit(window, "countdown", remaining, total, &msg);
            sleep(Duration::from_secs(1));
            remaining -= 1;
        }

        focus_game(&opts.game_title);

        // Reset the lock, then wait for it to settle.
        if abort.load(Ordering::SeqCst) {
            emit(window, "stopped", 0, total, "Stopped.");
            return;
        }
        emit(
            window,
            "reset",
            0,
            total,
            &format!("Pressing reset ({})…", opts.reset_key.to_uppercase()),
        );
        press(&opts.reset_key, opts.hold_ms);
        if sleep_abortable(opts.reset_delay_ms, &abort) {
            emit(window, "stopped", 0, total, "Stopped.");
            return;
        }

        // Play the sequence.
        for (i, k) in opts.keys.iter().enumerate() {
            if abort.load(Ordering::SeqCst) {
                emit(window, "stopped", (i) as u32, total, "Stopped.");
                return;
            }
            press(k, opts.hold_ms);
            emit(
                window,
                "playing",
                (i + 1) as u32,
                total,
                &format!("{}/{}  {}", i + 1, total, k.to_uppercase()),
            );
            if sleep_abortable(opts.delay_ms, &abort) {
                emit(window, "stopped", (i + 1) as u32, total, "Stopped.");
                return;
            }
        }

        emit(window, "done", total, total, "Done — lock should be open.");
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    use tauri_plugin_global_shortcut::ShortcutState;
                    if event.state() == ShortcutState::Pressed {
                        if let Some(state) = app.try_state::<AppState>() {
                            state.abort.store(true, Ordering::SeqCst);
                        }
                    }
                })
                .build(),
        )
        .manage(AppState {
            abort: Arc::new(AtomicBool::new(false)),
        })
        .invoke_handler(tauri::generate_handler![execute_plan, stop_plan, open_url])
        .run(tauri::generate_context!())
        .expect("error while running Gothic Lockpicking Tool");
}
