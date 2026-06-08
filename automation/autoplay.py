"""
Gothic Remake Lock Solver -- auto-player (Python, Windows).

Reads a key plan (space-separated W/A/S/D keys) from the clipboard, focuses the
game, presses the reset key, waits, counts down, then plays the moves -- the
same flow as the desktop app.

Usage:
    1. In the web tool: Solve, then click "Copy key plan".
    2. Run this script:   python autoplay.py
       (optional countdown override:  python autoplay.py 8 )
    3. Press Esc at any time to abort.

No external packages needed -- pure standard library. Keystrokes are sent with
Windows SendInput using hardware scancodes (what most games actually read), so
the automation part is Windows-only. On macOS/Linux, use the web tool and follow
the move list in-game.
"""

import sys
import time
import ctypes
from ctypes import wintypes

# ---- tunables ---------------------------------------------------------------
GAME_TITLE    = "Gothic 1 Remake"  # window to focus (substring match)
RESET_KEY     = "r"                # pressed once before solving
RESET_DELAY   = 1.0                # seconds after reset before playing
COUNTDOWN_SEC = 5                  # seconds before it starts (override via argv[1])
DELAY_BETWEEN = 0.18               # seconds between key presses
HOLD_TIME     = 0.04               # how long each key is held down
# -----------------------------------------------------------------------------

IS_WINDOWS = sys.platform == "win32"

# Scancodes (set 1). Extended keys (arrows) need the EXTENDED flag.
SCAN = {
    "a": 0x1E, "b": 0x30, "c": 0x2E, "d": 0x20, "e": 0x12, "f": 0x21,
    "g": 0x22, "h": 0x23, "i": 0x17, "j": 0x24, "k": 0x25, "l": 0x26,
    "m": 0x32, "n": 0x31, "o": 0x18, "p": 0x19, "q": 0x10, "r": 0x13,
    "s": 0x1F, "t": 0x14, "u": 0x16, "v": 0x2F, "w": 0x11, "x": 0x2D,
    "y": 0x15, "z": 0x2C, "space": 0x39, "enter": 0x1C, "esc": 0x01,
    "up": 0x48, "down": 0x50, "left": 0x4B, "right": 0x4D,
}
EXTENDED = {"up", "down", "left", "right"}

KEYEVENTF_EXTENDEDKEY = 0x0001
KEYEVENTF_KEYUP       = 0x0002
KEYEVENTF_SCANCODE    = 0x0008
INPUT_KEYBOARD        = 1
VK_ESCAPE             = 0x1B

PUL = ctypes.POINTER(ctypes.c_ulong)


class MOUSEINPUT(ctypes.Structure):
    _fields_ = [("dx", wintypes.LONG), ("dy", wintypes.LONG),
                ("mouseData", wintypes.DWORD), ("dwFlags", wintypes.DWORD),
                ("time", wintypes.DWORD), ("dwExtraInfo", PUL)]


class KEYBDINPUT(ctypes.Structure):
    _fields_ = [("wVk", wintypes.WORD), ("wScan", wintypes.WORD),
                ("dwFlags", wintypes.DWORD), ("time", wintypes.DWORD),
                ("dwExtraInfo", PUL)]


class HARDWAREINPUT(ctypes.Structure):
    _fields_ = [("uMsg", wintypes.DWORD), ("wParamL", wintypes.WORD),
                ("wParamH", wintypes.WORD)]


class _INPUTunion(ctypes.Union):
    _fields_ = [("mi", MOUSEINPUT), ("ki", KEYBDINPUT), ("hi", HARDWAREINPUT)]


class INPUT(ctypes.Structure):
    _anonymous_ = ("u",)
    _fields_ = [("type", wintypes.DWORD), ("u", _INPUTunion)]


if IS_WINDOWS:
    user32 = ctypes.windll.user32
    SendInput = user32.SendInput


def _send(scan, keyup, extended):
    flags = KEYEVENTF_SCANCODE
    if keyup:
        flags |= KEYEVENTF_KEYUP
    if extended:
        flags |= KEYEVENTF_EXTENDEDKEY
    ki = KEYBDINPUT(0, scan, flags, 0, None)
    inp = INPUT(INPUT_KEYBOARD, _INPUTunion(ki=ki))
    SendInput(1, ctypes.byref(inp), ctypes.sizeof(INPUT))


def press(key):
    key = key.strip().lower()
    if key not in SCAN:
        print("  ! unknown key skipped:", key)
        return
    scan = SCAN[key]
    ext = key in EXTENDED
    _send(scan, False, ext)
    time.sleep(HOLD_TIME)
    _send(scan, True, ext)


def esc_pressed():
    """True if Esc is currently down (Windows only)."""
    if not IS_WINDOWS:
        return False
    return bool(user32.GetAsyncKeyState(VK_ESCAPE) & 0x8000)


def focus_game(needle):
    """Find a visible window whose title contains `needle` and bring it forward."""
    if not IS_WINDOWS:
        return False
    needle = needle.lower()
    found = []

    WNDENUMPROC = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)

    def cb(hwnd, _lparam):
        if not user32.IsWindowVisible(hwnd):
            return True
        length = user32.GetWindowTextLengthW(hwnd)
        if length == 0:
            return True
        buf = ctypes.create_unicode_buffer(length + 1)
        user32.GetWindowTextW(hwnd, buf, length + 1)
        if needle in buf.value.lower():
            found.append(hwnd)
            return False  # stop enumerating
        return True

    user32.EnumWindows(WNDENUMPROC(cb), 0)
    if found:
        user32.ShowWindow(found[0], 9)          # SW_RESTORE
        user32.SetForegroundWindow(found[0])
        return True
    return False


def sleep_abortable(seconds):
    """Sleep, returning True early if Esc is pressed."""
    end = time.monotonic() + seconds
    while time.monotonic() < end:
        if esc_pressed():
            return True
        time.sleep(0.03)
    return esc_pressed()


def get_clipboard():
    """Read clipboard text via tkinter (stdlib, no install)."""
    try:
        import tkinter
        r = tkinter.Tk()
        r.withdraw()
        text = r.clipboard_get()
        r.destroy()
        return text
    except Exception as e:
        print("Could not read clipboard:", e)
        return ""


def main():
    if not IS_WINDOWS:
        print("Auto-play input is Windows-only (uses SendInput).")
        print("On this platform, use the web tool and follow the move list in-game.")
        return

    countdown = COUNTDOWN_SEC
    if len(sys.argv) > 1:
        try:
            countdown = int(sys.argv[1])
        except ValueError:
            pass

    plan = get_clipboard().strip()
    keys = [k for k in plan.replace(",", " ").split() if k]
    if not keys:
        print("Clipboard is empty or has no key plan.")
        print("In the web tool: Solve, then click 'Copy key plan', then re-run this.")
        return

    bad = [k for k in keys + [RESET_KEY] if k.lower() not in SCAN]
    if bad:
        print("Unknown key(s) in plan:", ", ".join(bad))
        return

    print("Key plan (%d presses): %s" % (len(keys), " ".join(k.upper() for k in keys)))
    focused = focus_game(GAME_TITLE)
    print("Game window: %s" % ("focused" if focused else "NOT found — switch to it manually"))
    print("Press Esc to abort.")

    for s in range(countdown, 0, -1):
        print("  starting in %d ..." % s, end="\r", flush=True)
        if sleep_abortable(1):
            print("\nAborted.")
            return
    print()

    focus_game(GAME_TITLE)

    print("Reset (%s) ..." % RESET_KEY.upper())
    press(RESET_KEY)
    if sleep_abortable(RESET_DELAY):
        print("Aborted.")
        return

    for i, k in enumerate(keys, 1):
        if esc_pressed():
            print("Aborted.")
            return
        press(k)
        print("  [%d/%d] %s" % (i, len(keys), k.upper()))
        if sleep_abortable(DELAY_BETWEEN):
            print("Aborted.")
            return

    print("Done.")


if __name__ == "__main__":
    main()
