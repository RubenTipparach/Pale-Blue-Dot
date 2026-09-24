---
name: obs-record
description: Record an application window with OBS Studio, convert it to MP4, and optionally drop it into a Google Drive (or any synced) folder. Use when the user wants a video or screen recording of a game, demo, test run or app ("record a flight", "capture a video of X", "make a clip and upload it"). It can launch the program itself, start timing on a line of its output, stop after a duration or on another output line, and record at a reduced resolution.
metadata:
  author: Pale Blue Dot (Claude Code)
  version: "1.0"
---

# Record a window with OBS

`scripts/obs_record.py` does the whole job in one command. It restores everything it changes:
- OBS's video settings, current scene and recording format;
- the temporary scene it creates;
- OBS's WebSocket server switch;
- OBS itself: if OBS wasn't running, the script starts it and closes it again.

## Before the first run on a machine

- **OBS:** OBS Studio 28 or later, opened at least once. The obs-websocket password must be set in **Tools → WebSocket Server Settings**; the script reads it from OBS's config and never prints it.
- **Python packages:** `pip install obsws-python imageio-ffmpeg`. imageio-ffmpeg bundles ffmpeg, so no separate install is needed.
- **WebSocket switch:** if the WebSocket server is switched off and OBS is closed, the script turns it on for the run and back off after. If OBS is **open** with the server off, the script stops and says so. Ask the user to close OBS or enable the server; do not edit the config under a running OBS, because OBS overwrites it on exit.

## Usage

```bash
python .claude/skills/obs-record/scripts/obs_record.py \
  --launch "<program and args>" --window-exe <program>.exe \
  --start-on "<regex in its output>" --duration <seconds> \
  --scale 0.5 --fps 60 --out <name>.mp4 [--copy-to "<folder>"] [--contact-sheet]
```

- **Target:** `--window-exe` (matches the executable) or `--window-title` (substring of the title). Without `--launch`, the window must already be open.
- **Timing:** recording starts once the window is captured. `--duration` counts from the `--start-on` line when given, otherwise from the start. `--stop-on` ends it early on an output line, and `--max-seconds` is a hard cap.
- **Loading screens are cut automatically.** Flat-colour frames at the start (a white or black window while the program loads) are detected and removed with a frame-exact re-encode. `--keep-blank-lead` keeps them.
- **Size:** the canvas takes the window's own size, and `--scale 0.5` records at half of it.
- **Output:** OBS records MKV (safe if anything crashes) and the script remuxes it to MP4 without re-encoding. `--contact-sheet` also writes `<out>_frames.png` with six frames across the video. Use it to **look at the result before you call it done**: a menu or dialog over the scene, a black capture or the wrong window are all easy to miss otherwise.
- **Upload:** `--copy-to` copies the MP4 into a folder. For Google Drive, use the Google Drive for Desktop folder (for example `G:/My Drive/<folder>`); Drive for Desktop uploads it. The Drive connector's create-file tool takes file contents inline, which cannot carry a video, so confirm the upload by searching Drive for the file title instead.

## Troubleshooting

Run the script in the foreground, or with `python -u` and its output sent to a file. Do not kill it with an outer timeout: a killed run cannot clean up. The next run undoes a killed run's WebSocket change from its backup file, `config.json.obs-record-backup`.

- **"OBS Studio did not properly shut down / Run in Safe Mode?"** OBS was force-closed. Safe Mode disables WebSockets, so the script cannot connect. The script clears OBS's stale markers in `%APPDATA%/obs-studio/.sentinel` before it launches OBS, and closes OBS politely. Never start OBS with `--minimize-to-tray`: a tray-only OBS cannot be closed politely.
- **"OBS is already running" dialog** while no OBS window exists: a dead `obs64.exe` is still listed with status "Unknown" (seen after a force-close during a capture). It blocks new instances until it clears; if it has not cleared, a reboot does it.
- **"the captured window reports no size"**: the window is minimized or never drew a frame.

## Pale Blue Dot recipes

The release app is `target/release/pbd-app.exe`, and its window belongs to `pbd-app.exe`.
- **Open a named world:** a plain launch opens the saves menu. Pass `--world <name>` to open a world directly. The world's save lands in `target/release/saves/<name>`; delete it after a throwaway run.
- **Loading:** the game's window is plain white for about 4 s while it builds the planet; the automatic trim removes it.
- **Tour to the far side of the planet:** `--launch "target/release/pbd-app.exe --tour --world obs-demo" --window-exe pbd-app.exe --start-on "weather spun up" --duration 37.5`. The tour cruises at 600 m/s, 1 km up, and `--verify-flight` measures a full lap at 64.7 s, so the far side is about 34 s after the flight starts.
- **Timing a new route:** get the duration from `--verify-flight` or from the route's own numbers, not by guessing.
