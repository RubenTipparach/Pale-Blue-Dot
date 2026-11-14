#!/usr/bin/env python3
"""Download the CC0 and public domain recordings (and any pack the owner asks for by name under another
licence, listed with its attribution in tools/audio/licence_exceptions.json) Pale Blue Dot's sounds are cut from.

Ported from Fleet Command (tools/audio/fetch_cc0.py at 08374c4). Each pack in tools/audio/cc0_packs.json
is fetched into --dest (default output/sfx-src, git-ignored: the originals are not committed, the cuts
are). An OpenGameArt pack's page is read for its licence and refused unless it says CC0; a Kenney pack
is CC0 by its site; an Internet Archive file is checked against the item's licence URL. Added here:
a Freesound sound (`freesound/<id>`), whose page is read for its licence link and refused unless it is
Creative Commons 0 (publicdomain/zero) or the public domain mark; its high-quality preview (an MP3,
which Freesound serves without an account) is what is downloaded, so a Freesound cut is band-limited
near 16 kHz, plenty for a bed. Archives are unpacked in place. A licence report is written to
<dest>/licences.json for assets/audio/sfx/SOURCES.md.

Usage: python3 tools/audio/fetch_cc0.py [--dest DIR] [pack ...]
Needs: curl, unzip, 7z (p7zip-full) for .7z packs.
"""
import argparse
import html
import json
import os
import re
import subprocess
import sys
import urllib.parse

HERE = os.path.dirname(os.path.abspath(__file__))
# packs the owner asked for under another licence, with its attribution (tools/audio/licence_exceptions.json)
with open(os.path.join(HERE, "licence_exceptions.json")) as _f:
    EXCEPTIONS = {k: v for k, v in json.load(_f).items() if not k.startswith("_")}
REPO = os.path.dirname(os.path.dirname(HERE))
UA = "pale-blue-dot-sfx-fetch/1.0 (+https://github.com/RubenTipparach/pale-blue-dot)"


def get(url: str) -> str:
    return subprocess.run(["curl", "-sL", "-m", "60", "-A", UA, url], capture_output=True,
                          text=True, errors="replace", check=True).stdout


def download(url: str, path: str) -> None:
    if os.path.exists(path) and os.path.getsize(path) > 0:
        return
    subprocess.run(["curl", "-sL", "-m", "1200", "-A", UA, "-o", path, url], check=True)


def unpack(path: str) -> None:
    out = os.path.splitext(path)[0]
    if os.path.isdir(out):
        return
    if path.endswith(".zip"):
        subprocess.run(["unzip", "-qo", path, "-d", out], check=True)
    elif path.endswith(".7z"):
        subprocess.run(["7z", "x", "-y", f"-o{out}", path], check=True, stdout=subprocess.DEVNULL)


def oga(slug: str, dest: str) -> dict:
    page = get(f"https://opengameart.org/content/{slug}")
    m = re.search(r"field-art-licenses.*?</div>\s*</div>", page, re.S)
    lic = sorted(set(re.findall(r"license-name'>([^<]+)<", m.group(0) if m else "")))
    author = re.search(r'field-name-author-submitter.*?<a href="/users/[^"]+">([^<]+)</a>', page, re.S)
    title = re.search(r'<h2>([^<]+)</h2>', page)
    exception = EXCEPTIONS.get(f"oga/{slug}")
    if "CC0" not in lic and not (exception and exception["licence"] in lic):
        return {"pack": slug, "licences": lic, "refused": "no CC0 option"}
    files = sorted(set(re.findall(
        r"https://opengameart.org/sites/default/files/[^\"'<>]+\.(?:zip|7z|wav|ogg|mp3|flac)", page)))
    files = [f for f in files if "/styles/" not in f and "/audio_preview/" not in f]
    d = os.path.join(dest, "oga", slug)
    os.makedirs(d, exist_ok=True)
    for f in files:
        p = os.path.join(d, urllib.parse.unquote(os.path.basename(f)))
        download(f, p)
        unpack(p)
    out = {"pack": slug, "url": f"https://opengameart.org/content/{slug}", "licences": lic,
           "title": html.unescape(title.group(1).strip()) if title else None,
           "author": author.group(1).strip() if author else None, "files": [os.path.basename(f) for f in files]}
    if "CC0" not in lic:
        out["licences"] = [exception["licence"]]
        out["attribution"] = exception["attribution"]
        out["owner"] = exception["owner"]
    return out


def kenney(slug: str, dest: str) -> dict:
    page = get(f"https://kenney.nl/assets/{slug}")
    zips = re.findall(r"https://kenney.nl/media/pages/assets/[^\"']+\.zip", page)
    if not zips:
        return {"pack": slug, "refused": "no download link found"}
    d = os.path.join(dest, "kenney")
    os.makedirs(d, exist_ok=True)
    p = os.path.join(d, f"{slug}.zip")
    download(zips[0], p)
    unpack(p)
    return {"pack": slug, "url": f"https://kenney.nl/assets/{slug}", "licences": ["CC0"], "author": "Kenney",
            "files": [os.path.basename(zips[0])]}


def freesound(sound_id: str, dest: str) -> dict:
    """One Freesound sound by its number. The page names the uploader in its URL and its licence in the
    link titled "Go to the full license text"; anything but CC0 or the public domain mark is refused."""
    page = get(f"https://freesound.org/s/{sound_id}/")
    m = re.search(r'title="Go to the full license text" href="([^"]+)"[^>]*>([^<]+)<', page)
    lic_url, lic_name = (m.group(1), m.group(2).strip()) if m else ("", "")
    exception = EXCEPTIONS.get(f"freesound/{sound_id}")
    if "/publicdomain/zero/" not in lic_url and "/publicdomain/mark/" not in lic_url and not (
            exception and exception["licence"] == lic_name):
        return {"pack": sound_id, "licences": [lic_name or "unknown"], "refused": "not CC0 or public domain"}
    url = re.search(r'<meta property="og:url" content="(https://freesound.org/people/([^/]+)/sounds/\d+/)"', page)
    title = re.search(r'<meta property="og:audio:title" content="([^"]*)"', page)
    hq = re.search(r'https://cdn.freesound.org/previews/[^"]+-hq\.mp3', page)
    if not hq:
        return {"pack": sound_id, "licences": [lic_name], "refused": "no preview link found"}
    d = os.path.join(dest, "freesound", sound_id)
    os.makedirs(d, exist_ok=True)
    name = f"{sound_id}.mp3"
    download(hq.group(0), os.path.join(d, name))
    out = {"pack": sound_id, "url": url.group(1) if url else f"https://freesound.org/s/{sound_id}/",
           "licences": ["CC0" if "/zero/" in lic_url else "public domain"], "licence_url": lic_url,
           "title": html.unescape(title.group(1)) if title else None,
           "author": url.group(2) if url else None, "files": [name], "note": "Freesound's HQ preview (MP3)"}
    if exception and "/publicdomain/" not in lic_url:
        out["licences"] = [exception["licence"]]
        out["attribution"] = exception["attribution"]
        out["owner"] = exception["owner"]
    return out


def archive(spec: str, dest: str) -> dict:
    item, _, names = spec.partition(":")
    meta = json.loads(get(f"https://archive.org/metadata/{item}"))
    lic = meta.get("metadata", {}).get("licenseurl", "")
    if "publicdomain" not in lic and "/zero/" not in lic:
        return {"pack": item, "licences": [lic], "refused": "not public domain or CC0"}
    d = os.path.join(dest, "archive", item)
    os.makedirs(d, exist_ok=True)
    want = [n for n in names.split(",") if n]
    for f in meta.get("files", []):
        if f["name"] in want:
            download(f"https://archive.org/download/{item}/{urllib.parse.quote(f['name'])}", os.path.join(d, f["name"]))
    return {"pack": item, "url": f"https://archive.org/details/{item}", "licences": [lic],
            "author": meta.get("metadata", {}).get("creator"), "files": want}


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--dest", default=os.environ.get("SFX_SRC", os.path.join(REPO, "output", "sfx-src")))
    ap.add_argument("packs", nargs="*")
    args = ap.parse_args()
    os.makedirs(args.dest, exist_ok=True)
    with open(os.path.join(HERE, "cc0_packs.json")) as f:
        packs = json.load(f)
    report_path = os.path.join(args.dest, "licences.json")
    report = {}
    if os.path.exists(report_path):
        with open(report_path) as f:
            report = json.load(f)
    fetch = {"oga": oga, "kenney": kenney, "archive": archive, "freesound": freesound}
    for p in packs:
        site, _, slug = p.partition("/")
        key = p.split(":")[0]
        if args.packs and key not in args.packs and slug.split(":")[0] not in args.packs:
            continue
        try:
            report[key] = fetch[site](slug, args.dest)
        except (subprocess.CalledProcessError, json.JSONDecodeError) as e:
            report[key] = {"pack": slug, "refused": f"fetch failed: {e}"}
        r = report[key]
        print(f"{key:55} {'REFUSED ' + r['refused'] if 'refused' in r else ', '.join(r['licences'])}")
        with open(report_path, "w") as f:
            json.dump(report, f, indent=1)
    return 0


if __name__ == "__main__":
    sys.exit(main())
