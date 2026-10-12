#!/usr/bin/env python3
"""Mean channel value of a capture, which is how a LIGHTING change is measured.

A picture cannot tell you that a cave is too bright; it can only look wrong,
and "looks wrong" is not a number anybody can hold a change against. The frame
mean is: a cave twenty metres underground reading at four fifths of an open
meadow at midday is a light model that has no dark in it, whatever the picture
suggests.

The hotbar strip is left out by default, because it is UI drawn at a fixed
brightness and it would flatten every comparison it appears in.

    python3 tools/frame_mean.py cave.png meadow.png
    python3 tools/frame_mean.py shot.png --rows 1.0    # the whole frame

No third-party dependency, like every other generator here: stdlib and zlib.
"""

import struct
import sys
import zlib


def read_png(path):
    """Width, height and RGB bytes of an 8-bit truecolour PNG."""
    data = open(path, "rb").read()
    pos, width, height, idat = 8, None, None, b""
    while pos < len(data):
        length = struct.unpack(">I", data[pos : pos + 4])[0]
        kind = data[pos + 4 : pos + 8]
        body = data[pos + 8 : pos + 8 + length]
        if kind == b"IHDR":
            width, height, depth, colour = struct.unpack(">IIBB", body[:10])
            if (depth, colour) != (8, 2):
                raise SystemExit(f"{path}: expected 8-bit RGB, got depth {depth} type {colour}")
        elif kind == b"IDAT":
            idat += body
        pos += 12 + length
    raw = zlib.decompress(idat)
    stride = width * 3
    out = bytearray()
    previous = bytearray(stride)
    at = 0
    for _ in range(height):
        filt = raw[at]
        at += 1
        line = bytearray(raw[at : at + stride])
        at += stride
        for x in range(stride):
            left = line[x - 3] if x >= 3 else 0
            up = previous[x]
            upleft = previous[x - 3] if x >= 3 else 0
            if filt == 1:
                line[x] = (line[x] + left) & 255
            elif filt == 2:
                line[x] = (line[x] + up) & 255
            elif filt == 3:
                line[x] = (line[x] + (left + up) // 2) & 255
            elif filt == 4:
                guess = left + up - upleft
                da, db, dc = abs(guess - left), abs(guess - up), abs(guess - upleft)
                near = left if (da <= db and da <= dc) else (up if db <= dc else upleft)
                line[x] = (line[x] + near) & 255
        out += line
        previous = line
    return width, height, out


def mean(path, rows):
    width, height, pixels = read_png(path)
    last = max(1, int(height * rows))
    total = count = 0
    for y in range(last):
        base = y * width * 3
        for x in range(width):
            at = base + x * 3
            total += pixels[at] + pixels[at + 1] + pixels[at + 2]
            count += 3
    return total / count


def main(argv):
    rows = 0.85
    paths = []
    it = iter(argv)
    for arg in it:
        if arg == "--rows":
            rows = float(next(it))
        else:
            paths.append(arg)
    if not paths:
        raise SystemExit(__doc__)
    for path in paths:
        print(f"{mean(path, rows):6.2f}  of 255   {path}")


if __name__ == "__main__":
    main(sys.argv[1:])
