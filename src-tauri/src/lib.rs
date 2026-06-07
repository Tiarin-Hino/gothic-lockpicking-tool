use serde::{Deserialize, Serialize};
use tauri::{Emitter, WebviewWindow};

#[derive(Deserialize)]
pub struct ExecOpts {
    pub keys: Vec<String>,
    #[serde(rename = "gameTitle")]
    pub game_title: String,
    pub countdown: u32,
    #[serde(rename = "delayMs")]
    pub delay_ms: u64,
    #[serde(rename = "holdMs")]
    pub hold_ms: u64,
}

#[derive(Clone, Serialize)]
struct Progress {
    phase: String,   // "countdown" | "playing" | "done" | "error"
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
fn execute_plan(window: WebviewWindow, opts: ExecOpts) -> Result<(), String> {
    #[cfg(not(windows))]
    {
        let _ = (&window, &opts);
        return Err(
            "Desktop auto-play is currently Windows-only. On this platform, use the move list \
             from the solver and perform the steps in-game."
                .into(),
        );
    }

    #[cfg(windows)]
    {
        // Validate all keys up front so we fail fast with a clear message.
        for k in &opts.keys {
            if winimpl::scancode(k).is_none() {
                return Err(format!("Unknown key in plan: '{}'", k));
            }
        }
        // Run the timed sequence off the UI thread.
        std::thread::spawn(move || winimpl::run_plan(window, opts));
        Ok(())
    }
}

#[cfg(windows)]
mod winimpl {
    use super::{emit, ExecOpts};
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

    /// Maps a plan token to (scancode, is_extended_key).
    pub fn scancode(key: &str) -> Option<(u16, bool)> {
        Some(match key {
            "w" => (0x11, false),
            "a" => (0x1E, false),
            "s" => (0x1F, false),
            "d" => (0x20, false),
            "up" => (0x48, true),
            "down" => (0x50, true),
            "left" => (0x4B, true),
            "right" => (0x4D, true),
            "space" => (0x39, false),
            "enter" => (0x1C, false),
            "esc" => (0x01, false),
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

    /// Tries to find and focus a visible window whose title contains `needle`.
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

    pub fn run_plan(window: WebviewWindow, opts: ExecOpts) {
        let total = opts.keys.len() as u32;

        // First focus attempt.
        let focused = focus_game(&opts.game_title);

        // Countdown (also a safety window to alt-tab if focus failed).
        let mut remaining = opts.countdown;
        while remaining > 0 {
            let msg = if focused {
                format!("Starting in {remaining}s — game focused")
            } else {
                format!("Starting in {remaining}s — switch to '{}'", opts.game_title)
            };
            emit(&window, "countdown", remaining, total, &msg);
            sleep(Duration::from_secs(1));
            remaining -= 1;
        }

        // Last-chance focus right before playing.
        focus_game(&opts.game_title);

        // Play the sequence.
        for (i, k) in opts.keys.iter().enumerate() {
            press(k, opts.hold_ms);
            emit(
                &window,
                "playing",
                (i + 1) as u32,
                total,
                &format!("{}/{}  {}", i + 1, total, k.to_uppercase()),
            );
            sleep(Duration::from_millis(opts.delay_ms));
        }

        emit(&window, "done", total, total, "Done — lock should be open.");
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![execute_plan])
        .run(tauri::generate_context!())
        .expect("error while running Gothic Lockpicking Tool");
}
