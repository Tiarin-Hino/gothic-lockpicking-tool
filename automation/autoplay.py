"""
Gothic Remake Lock Solver -- auto-player.

Reads a key plan (space-separated W/A/S/D keys) from the clipboard, counts down
so you can alt-tab into the game, then presses the keys for you.

Usage:
    1. In the web tool: Solve, then click "Copy key plan".
    2. Run this script:   python autoplay.py
       (optional countdown override:  python autoplay.py 8 )
    3. Switch to the game window before the countdown hits zero.

No external packages needed -- pure standard library. Keystrokes are sent with
Windows SendInput using hardware scancodes, which is what most games actually
read (plain SendKeys / pyautogui often gets ignored).
"""

import sys
import time
import ctypes
from ctypes import wintypes

# ---- tunables ---------------------------------------------------------------
COUNTDOWN_SEC = 5      # seconds before it starts pressing (override via argv[1])
DELAY_BETWEEN = 0.18   # seconds between key presses
HOLD_TIME     = 0.04   # how long each key is held down
# -----------------------------------------------------------------------------

# Scancodes (set 1). Extended keys (arrows) need the EXTENDED flag.
SCAN = {
    'a': 0x1E, 'b': 0x30, 'c': 0x2E, 'd': 0x20, 'e': 0x12, 'f': 0x21,
    'g': 0x22, 'h': 0x23, 'i': 0x17, 'j': 0x24, 'k': 0x25, 'l': 0x26,
    'm': 0x32, 'n': 0x31, 'o': 0x18, 'p': 0x19, 'q': 0x10, 'r': 0x13,
    's': 0x1F, 't': 0x14, 'u': 0x16, 'v': 0x2F, 'w': 0x11, 'x': 0x2D,
    'y': 0x15, 'z': 0x2C, 'space': 0x39,
    'up': 0x48, 'down': 0x50, 'left': 0x4B, 'right': 0x4D,
    'enter': 0x1C, 'esc': 0x01,
}
EXTENDED = {'up', 'down', 'left', 'right'}

KEYEVENTF_EXTENDEDKEY = 0x0001
KEYEVENTF_KEYUP       = 0x0002
KEYEVENTF_SCANCODE    = 0x0008
INPUT_KEYBOARD        = 1

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


SendInput = ctypes.windll.user32.SendInput


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

    print("Key plan (%d presses): %s" % (len(keys), " ".join(k.upper() for k in keys)))
    print("Switch to the GAME window now...")
    for s in range(countdown, 0, -1):
        print("  starting in %d ..." % s, end="\r", flush=True)
        time.sleep(1)
    print("\nGo!")

    for i, k in enumerate(keys, 1):
        press(k)
        print("  [%d/%d] %s" % (i, len(keys), k.upper()))
        time.sleep(DELAY_BETWEEN)

    print("Done.")


if __name__ == "__main__":
    main()
