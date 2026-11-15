#!/usr/bin/env python3
"""Encode a WAV to the game's OGG Vorbis, the same bytes every time.

Written in this repository (2026-11-14) for tools/audio/render_sfx.sh. Fleet Command encodes with
ffmpeg's libvorbis; the Mac's Homebrew ffmpeg has only ffmpeg's own experimental Vorbis encoder, so
this uses libvorbis through libsndfile (python's soundfile) instead, and falls back to ffmpeg's
libvorbis where soundfile is missing. libsndfile gives each stream a random serial number, so the
serial is set to a fixed one afterwards and every page's checksum recomputed: an unchanged cue
encodes to the same bytes and does not churn the repository (Fleet Command's `-serial_offset 0`).

Quality is Vorbis's -q scale (openspec/changes/audio-and-ambience design section 7: placed cues q5,
beds q4); libsndfile's compression level is 1 - q / 10.

Usage: python3 tools/audio/encode_ogg.py IN.wav OUT.ogg [--quality 5]
"""
import argparse
import struct
import subprocess
import sys

SERIAL = 0x50424431  # "PBD1"


def _crc_table() -> list:
    table = []
    for i in range(256):
        r = i << 24
        for _ in range(8):
            r = ((r << 1) ^ 0x04C11DB7) if r & 0x80000000 else (r << 1)
        table.append(r & 0xFFFFFFFF)
    return table


_TABLE = _crc_table()


def _crc(data: bytes) -> int:
    c = 0
    for b in data:
        c = ((c << 8) & 0xFFFFFFFF) ^ _TABLE[((c >> 24) & 0xFF) ^ b]
    return c


def fix_serial(path: str, serial: int = SERIAL) -> None:
    """Rewrite every Ogg page's stream serial number and its CRC (one logical stream per file)."""
    data = bytearray(open(path, "rb").read())
    i = 0
    while i < len(data):
        if data[i:i + 4] != b"OggS":
            raise ValueError(f"{path}: no Ogg page at byte {i}")
        segments = data[i + 26]
        body = sum(data[i + 27:i + 27 + segments])
        end = i + 27 + segments + body
        struct.pack_into("<I", data, i + 14, serial)
        struct.pack_into("<I", data, i + 22, 0)
        struct.pack_into("<I", data, i + 22, _crc(bytes(data[i:end])))
        i = end
    open(path, "wb").write(bytes(data))


def encode(src: str, dst: str, quality: float) -> None:
    try:
        import soundfile as sf
    except ImportError:
        subprocess.run(["ffmpeg", "-hide_banner", "-loglevel", "error", "-y", "-i", src, "-c:a", "libvorbis",
                        "-q:a", str(quality), "-fflags", "+bitexact", "-flags:a", "+bitexact",
                        "-serial_offset", "0", dst], check=True)
        return
    x, rate = sf.read(src, dtype="float32")
    sf.write(dst, x, rate, format="OGG", subtype="VORBIS", compression_level=max(0.0, min(1.0, 1 - quality / 10)))
    fix_serial(dst)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("src")
    ap.add_argument("dst")
    ap.add_argument("--quality", type=float, default=5.0)
    a = ap.parse_args()
    encode(a.src, a.dst, a.quality)
    return 0


if __name__ == "__main__":
    sys.exit(main())
