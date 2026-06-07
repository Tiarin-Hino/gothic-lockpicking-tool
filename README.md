# Gothic 1 Remake — Lockpicking Tool

A solver and optional auto-player for the multi-mechanism lockpick puzzle in the
**Gothic 1 Remake**.

- **Web tool** — set up the lock, describe how the mechanisms interact, and get
  the shortest sequence of moves to open it. Runs entirely in your browser,
  nothing to install, works on any OS.
  👉 **https://gothic-lockpicking-tool.tiarinhino.com**
- **Auto-player** *(Windows, optional)* — a tiny program that reads the solution
  and presses the keys for you, auto-focusing the game window.

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

## Auto-player (Windows)

> Convenience only — it just presses the same keys the solver lists.

1. In the web tool, after solving, click **Copy key plan**.
2. Download **`gothic-autoplay.exe`** from the
   [Releases page](../../releases) and run it.
3. It focuses **Gothic 1 Remake**, counts down, and plays the keys.
   Abort any time with **Ctrl + Alt + X**.

### Configuration

Drop an `autoplay.ini` next to the exe to tweak behaviour (a sample ships with
each release):

```ini
[settings]
gameTitle=Gothic 1 Remake
countdown=5         ; seconds before it starts
delayBetween=180    ; ms between presses — raise if the game misses inputs
holdTime=40         ; ms each key is held — raise if presses don't register
```

> **Heads-up:** the exe is **not code-signed**, so Windows SmartScreen may warn
> on first run ("Windows protected your PC" → *More info* → *Run anyway*). The
> full source is in this repo and the exe is built automatically by GitHub
> Actions from `automation/autoplay.ahk` — nothing hidden.

## Linux / macOS / advanced users (Python)

The `automation/` folder also has a cross-platform Python version of the
auto-player (standard library only, no `pip install`):

```bash
# 1. Copy the key plan from the web tool, then:
python automation/autoplay.py        # optional countdown override: autoplay.py 8
```

It reads the plan from your clipboard, counts down, and sends the keys.
On macOS you'll need to grant the terminal **Accessibility** permission;
on Linux it targets X11 (Wayland input injection varies).
Windows users can also double-click `automation/autoplay.bat`.

---

## Repository layout

```
web/index.html                  the solver web tool (static; deploy this)
automation/autoplay.ahk         Windows auto-player source (compiled by CI)
automation/autoplay.ini         sample config
automation/autoplay.py / .bat   cross-platform / Windows Python option
.github/workflows/build-exe.yml compiles the .ahk into a release .exe
```

## Building the exe yourself

Push a tag (`git tag v1.0.0 && git push --tags`) or run the **Build autoplay
exe** workflow manually from the Actions tab. It downloads AutoHotkey v2,
compiles `automation/autoplay.ahk`, and attaches `gothic-autoplay.exe` to a
GitHub Release. To build locally instead, install
[AutoHotkey v2](https://www.autohotkey.com/) and run its `Ahk2Exe` compiler on
`automation/autoplay.ahk`.

## Deploying the web tool (maintainer notes — AWS)

The web tool is a single static file. To host it at
`gothic-lockpicking-tool.tiarinhino.com`:

1. **S3** — create a bucket, upload `web/index.html` (set as index document):
   ```bash
   aws s3 sync web/ s3://YOUR-BUCKET/ --delete
   ```
2. **ACM** — request a certificate for the subdomain in **us-east-1**
   (required for CloudFront).
3. **CloudFront** — create a distribution with the S3 bucket as origin, attach
   the ACM cert, and set the alternate domain name to the subdomain.
4. **DNS** — add a CNAME (or Route 53 alias) from
   `gothic-lockpicking-tool` → the CloudFront distribution domain.

## License

[MIT](LICENSE) — © 2026 Tiarin Hino.

*Single-player convenience tool. Not affiliated with Piranha Bytes / THQ Nordic.*
