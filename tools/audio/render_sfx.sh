#!/usr/bin/env bash
# Build Pale Blue Dot's sound effects and ambience beds. Ported from Fleet Command
# (tools/audio/render_sfx.sh at 08374c4, itself after Crab Breeze's at 0e569e3), which already
# dropped Crab Breeze's muffled and far copies; here kira filters live (openspec/changes/
# audio-and-ambience, design section 1), so none are made either.
#
# tools/audio/gen_sfx.py synthesises every synthesised cue into output/sfx with a manifest,
# tools/audio/cut_cc0.py cuts the recorded cues (and takes) from CC0 and public domain originals
# (fetched once by tools/audio/fetch_cc0.py into $SFX_SRC, default output/sfx-src, git-ignored),
# then each becomes the game's copy, 48 kHz OGG Vorbis (assets/audio/sfx/<cue>.ogg, q5, mono;
# ambience beds amb_* in assets/audio/ambience, stereo, q4; tools/audio/encode_ogg.py, the same
# bytes every run) and a share copy for the Sounds page (docs/mockups/sounds/<cue>.mp3, beside
# the page, published with it through docs/mockups/sounds.files.json).
#
# Usage: tools/audio/render_sfx.sh [cue ...]      (no cues: all of them)
# Needs: python3 with numpy (scipy and soundfile make it fast and bit-exact), ffmpeg, curl, unzip.
set -euo pipefail
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
OUT="$REPO/output/sfx"
OGG="$REPO/assets/audio/sfx"
AMB="$REPO/assets/audio/ambience"
MP3="$REPO/docs/mockups/sounds"
mkdir -p "$OUT"
if [ "$#" -gt 0 ]; then
  python3 "$REPO/tools/audio/gen_sfx.py" --out "$OUT" --only "$@"
else
  rm -f "$OUT"/*.wav
  python3 "$REPO/tools/audio/gen_sfx.py" --out "$OUT"
fi
# an empty spec and no synthesised cue (as ported, before the first set): nothing to write
if [ "$(python3 -c "import json,sys; print(sum(1 for k in json.load(open(sys.argv[1])) if not k.startswith('_')))" "$REPO/tools/audio/cc0_cues.json")" = "0" ] \
   && ! compgen -G "$OUT/*.wav" > /dev/null; then
  echo "no cues yet (tools/audio/cc0_cues.json is empty and gen_sfx.py has no sounds): nothing written"
  exit 0
fi
mkdir -p "$OGG" "$AMB" "$MP3"
SRC="${SFX_SRC:-$REPO/output/sfx-src}"
export SFX_SRC="$SRC"
# nothing to fetch while the pack list is empty
if [ "$(python3 -c "import json,sys; print(len(json.load(open(sys.argv[1]))))" "$REPO/tools/audio/cc0_packs.json")" != "0" ]; then
  [ -f "$SRC/licences.json" ] || python3 "$REPO/tools/audio/fetch_cc0.py" --dest "$SRC"
fi
python3 "$REPO/tools/audio/cut_cc0.py" --src "$SRC" --out "$OUT" "$@"
shopt -s nullglob
for wav in "$OUT"/*.wav; do
  name="$(basename "$wav" .wav)"
  if [ "$#" -gt 0 ]; then
    base="${name%_[0-9]*}"
    if [[ ! " $* " == *" $name "* && ! " $* " == *" $base "* ]]; then continue; fi
  fi
  if [[ "$name" == amb_* ]]; then
    # ambience beds: stereo, not placed, in their own folder; a lighter share copy for the page
    python3 "$REPO/tools/audio/encode_ogg.py" "$wav" "$AMB/$name.ogg" --quality 4
    ffmpeg -hide_banner -loglevel error -y -i "$wav" -c:a libmp3lame -b:a 96k -map_metadata -1 -fflags +bitexact -flags:a +bitexact "$MP3/$name.mp3"
  else
    python3 "$REPO/tools/audio/encode_ogg.py" "$wav" "$OGG/$name.ogg" --quality 5
    ffmpeg -hide_banner -loglevel error -y -i "$wav" -c:a libmp3lame -b:a 128k -map_metadata -1 -fflags +bitexact -flags:a +bitexact "$MP3/$name.mp3"
  fi
done
# the record of every pack used, AUDIO_SOURCES.md, and the Sounds page
if [ -f "$SRC/licences.json" ]; then cp "$SRC/licences.json" "$REPO/tools/audio/cc0_licences.json"; fi
[ -f "$REPO/tools/audio/cc0_licences.json" ] || echo "{}" > "$REPO/tools/audio/cc0_licences.json"
python3 "$REPO/tools/audio/list_sources.py"
python3 "$REPO/tools/audio/gen_sfx_page.py"
ogg=( "$OGG"/*.ogg ); amb=( "$AMB"/*.ogg )
echo "wrote ${#ogg[@]} sounds to assets/audio/sfx, ${#amb[@]} beds to assets/audio/ambience"
