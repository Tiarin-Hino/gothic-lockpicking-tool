# Gothic 1 Remake — Lockpicking Tool

A solver and optional auto-player for the multi-mechanism lockpick puzzle in the
**Gothic 1 Remake**.

- **Web tool** — set up the lock, describe how the mechanisms interact, and get
  the shortest sequence of moves to open it. Runs entirely in your browser,
  nothing to install, works on any OS.
  👉 **https://gothic-lockpicking-tool.tiarinhino.com**
- **Desktop app** *(Windows, optional)* — the same tool as a downloadable app
  with an **Execute** button that focuses the game and plays the moves for you.

---

## How the puzzle works

Each mechanism has **7 holes**; the goal is to get every mechanism centred. You
slide mechanisms left/right, and each mechanism can also push the others
(same direction, opposite direction, or no effect). The web tool searches for
the shortest combination of moves that centres them all.

## Using the web tool

1. Open the [web tool](https://gothic-lockpicking-tool.tiarinhino.com).
2. Set the number of mechanisms and click the holes to match your in-game state.
3. Fill in the **interaction matrix** — for each mechanism, how the others react
   when you move it (None / Same / Opposite). Map the game's *mirror* / *counter*
   to Same / Opposite based on what you observe.
4. Click **Solve**. You get a numbered list of moves.

## Desktop app (Windows)

A single portable `.exe` — the full solver UI plus a one-click auto-player.

1. Download **`gothic-lockpicking-tool.exe`** from the
   [Releases page](../../releases) and run it (no install needed; uses the
   WebView2 runtime that ships with Windows 10/11).
2. Set up the lock and click **Solve** exactly like on the website.
3. In section 4, set the game window title / timings if needed and click
   **Execute in game**. It focuses **Gothic 1 Remake**, counts down, then plays
   the moves. Progress shows live; the countdown lets you alt-tab if focus
   doesn't catch.

> **Heads-up:** the exe is **not code-signed**, so Windows SmartScreen may warn
> on first run ("Windows protected your PC" → *More info* → *Run anyway*). The
> full source is in this repo and the exe is built automatically by GitHub
> Actions — nothing hidden.

## Python auto-player (advanced users)

A script alternative to the desktop app, in the `automation/` folder (standard
library only, no `pip install`). Input is sent via Windows `SendInput`, so the
**automation runs on Windows**. On macOS/Linux, use the **web tool** for the
solution and follow the move list in-game.

```bash
# 1. Copy the key plan from the web tool, then:
python automation/autoplay.py        # optional countdown override: autoplay.py 8
```

It reads the plan from your clipboard, focuses the game (Windows), presses the
reset key, waits, counts down, then plays the moves — the same flow as the
desktop app. Timings and keys are constants at the top of the script:

```python
GAME_TITLE     = "Gothic 1 Remake"
RESET_KEY      = "r"
RESET_DELAY    = 1.0   # seconds after reset before playing
COUNTDOWN_SEC  = 5
DELAY_BETWEEN  = 0.18
HOLD_TIME      = 0.04
```

Press **Esc** to abort mid-run. You can also double-click
`automation/autoplay.bat`.

---

## Repository layout

```
web/index.html                  the solver UI (static; also the app frontend)
src-tauri/                       Tauri desktop app (Rust backend: keys + focus)
automation/autoplay.py / .bat    Python auto-player option (Windows)
automation/autoplay.ahk / .ini   legacy AutoHotkey script (kept for reference)
.github/workflows/build-app.yml  builds the portable .exe on a release
```

## Building the app yourself

Push a tag (`git tag v1.0.0 && git push --tags`) or run the **Build desktop
app** workflow from the Actions tab — it compiles `src-tauri` on a Windows
runner and attaches `gothic-lockpicking-tool.exe` to a GitHub Release. To build
locally, install [Rust](https://rustup.rs/) and the
[Tauri prerequisites](https://tauri.app/start/prerequisites/), then run
`cargo build --release` in `src-tauri/` (or `cargo tauri dev` to run it live).

## License

[MIT](LICENSE) — © 2026 Tiarin Hino.

*Single-player convenience tool. Not affiliated with Piranha Bytes / THQ Nordic.*
