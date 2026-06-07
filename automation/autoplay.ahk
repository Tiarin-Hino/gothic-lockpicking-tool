#Requires AutoHotkey v2.0
#SingleInstance Force
; =====================================================================
;  Gothic Remake Lock Solver -- auto-player (AutoHotkey v2)
;
;  1. In the web tool: Solve, then click "Copy key plan".
;  2. Run this (or the compiled gothic-autoplay.exe).
;  3. It focuses "Gothic 1 Remake", counts down, then presses the keys.
;
;  Abort any time with  Ctrl+Alt+X.
;
;  Settings can be overridden by an autoplay.ini placed next to the exe:
;    [settings]
;    gameTitle=Gothic 1 Remake
;    countdown=5
;    delayBetween=180
;    holdTime=40
; =====================================================================

; ---- defaults ----
gameTitle    := "Gothic 1 Remake"
countdown    := 5      ; seconds before it starts pressing
delayBetween := 180    ; ms between key presses
holdTime     := 40     ; ms each key is held down

; ---- optional ini overrides ----
iniFile := A_ScriptDir "\autoplay.ini"
if FileExist(iniFile) {
    gameTitle    := IniRead(iniFile, "settings", "gameTitle", gameTitle)
    countdown    := Integer(IniRead(iniFile, "settings", "countdown", countdown))
    delayBetween := Integer(IniRead(iniFile, "settings", "delayBetween", delayBetween))
    holdTime     := Integer(IniRead(iniFile, "settings", "holdTime", holdTime))
}

; ---- abort hotkey ----
^!x:: {
    ToolTip "Aborted."
    SetTimer () => ToolTip(), -800
    ExitApp
}

; ---- read & parse the key plan from the clipboard ----
plan := Trim(A_Clipboard)
if (plan = "") {
    MsgBox "Clipboard is empty.`n`nIn the web tool: Solve, then click 'Copy key plan', then run this again.", "Gothic Autoplay", "Iconx"
    ExitApp
}
plan := StrReplace(plan, ",", " ")

; scancodes -> reliable in DirectInput games
codes := Map(
    "w","011", "a","01E", "s","01F", "d","020",
    "up","148", "down","150", "left","14B", "right","14D",
    "space","039", "enter","01C", "esc","001"
)

keys := []
for word in StrSplit(plan, " ") {
    k := StrLower(Trim(word))
    if (k = "")
        continue
    if !codes.Has(k) {
        MsgBox "Unknown key in plan: '" k "'`n`nMake sure you copied the key plan from the web tool.", "Gothic Autoplay", "Iconx"
        ExitApp
    }
    keys.Push(k)
}
if (keys.Length = 0) {
    MsgBox "No valid keys found on the clipboard.", "Gothic Autoplay", "Iconx"
    ExitApp
}

; ---- try to focus the game ----
focused := false
if WinExist(gameTitle) {
    WinActivate
    if WinWaitActive(gameTitle, , 2)
        focused := true
}

; ---- countdown (always; lets you alt-tab if auto-focus missed) ----
Loop countdown {
    remaining := countdown - A_Index + 1
    msg := "Gothic Autoplay starting in " remaining "s`n"
        . keys.Length " key presses queued`n"
        . (focused ? "Game focused." : "Switch to '" gameTitle "' now!") "`n"
        . "(Ctrl+Alt+X to abort)"
    ToolTip msg
    Sleep 1000
}
ToolTip

; ---- last-chance focus, in case the user just switched ----
if WinExist(gameTitle) {
    WinActivate
    WinWaitActive(gameTitle, , 1)
}

; ---- play the sequence ----
for i, k in keys {
    sc := codes[k]
    Send "{sc" sc " down}"
    Sleep holdTime
    Send "{sc" sc " up}"
    ToolTip "Playing " i "/" keys.Length "  (" StrUpper(k) ")"
    Sleep delayBetween
}

ToolTip "Done."
SetTimer () => ToolTip(), -1000
ExitApp
