"""Portraits and UI icons for the NPC mockup (docs/mockups/npcs.html).

Painted by code with the hand-drawn-illustration skill's crayon kit, in the
manner of the owner's friend's upgrade icons. Nothing here is drawn by hand.

    python tools/art/npcs_art.py            # writes docs/mockups/npcs/*.png
    python tools/art/npcs_art.py --sheet    # and a contact sheet beside the friend's icons

A portrait is a bust of a townsperson as the townsfolk change builds them:
hexagonal prisms that read as cubes (owner, survey T2), so the head is a
flat-fronted block, not a round face. Each role carries the one thing that
says what it does (a hammer, a spear, a datapad).
"""

import math
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
KIT = os.path.join(ROOT, ".claude", "skills", "hand-drawn-illustration", "scripts")
sys.path.insert(0, KIT)
from crayon import Icon, INK, circle, ellipse, rect, roundrect, star, rotate, translate, scale, lighter, darker, tone  # noqa: E402

OUT = os.path.join(ROOT, "docs", "mockups", "npcs")


# ---------- a bust ---------------------------------------------------------------------

def bust(ic, skin, shirt, hair=None, hair_style="cap", eyes=(30, 26, 36), shoulders=1.0, mouth="smile", neck=True):
    """Shoulders, neck and a block head, front on, light from the upper left."""
    w = 150 * shoulders
    torso = roundrect(256 - w, 360, 256 + w, 560, 46)
    ic.drawn(torso, shirt, hatch=1.0, shade=0.24, highlight=0.14)
    if neck:
        ic.drawn(rect(222, 300, 290, 372), darker(skin, 0.18), hatch=0.5, highlight=0, shade=0.2, edge_w=6)
    head = roundrect(168, 128, 344, 318, 22)
    ic.drawn(head, skin, hatch=0.7, shade=0.2, highlight=0.18, edge_w=7)
    if hair:
        if hair_style == "cap":
            ic.drawn(roundrect(160, 112, 352, 176, 18), hair, hatch=1.1, highlight=0.1, shade=0.15, edge_w=7)
        elif hair_style == "long":
            ic.drawn(roundrect(160, 112, 352, 176, 18), hair, hatch=1.1, highlight=0.1, shade=0.15, edge_w=7)
            ic.drawn(rect(156, 150, 190, 300), hair, hatch=0.9, highlight=0, shade=0.15, edge_w=6)
            ic.drawn(rect(322, 150, 356, 300), hair, hatch=0.9, highlight=0, shade=0.15, edge_w=6)
        elif hair_style == "beard":
            ic.drawn(roundrect(160, 112, 352, 170, 18), hair, hatch=1.1, highlight=0.1, shade=0.15, edge_w=7)
            ic.drawn([(176, 262), (336, 262), (318, 330), (256, 352), (194, 330)], hair, hatch=1.0, highlight=0.08, shade=0.2, edge_w=6)
    # eyes: two square pupils, the pixel figures' own
    for x in (214, 298):
        ic.flat(rect(x - 13, 206, x + 13, 232), (250, 248, 240), wobble=1)
        ic.flat(rect(x - 6, 210, x + 9, 230), eyes, wobble=1)
    if mouth == "smile" and hair_style != "beard":
        ic.line([(228, 272), (256, 282), (284, 272)], tone(skin, 0.5), 7)
    elif mouth == "flat" and hair_style != "beard":
        ic.line([(232, 276), (282, 276)], tone(skin, 0.5), 7)


# ---------- the roles ------------------------------------------------------------------

def farmer(ic):
    # pitchfork behind the shoulder
    ic.line([(400, 500), (420, 170)], (150, 104, 56), 14, ink=tone((150, 104, 56)), ink_w=8)
    for dx in (-26, 0, 26):
        ic.line([(420 + dx, 170), (420 + dx * 1.1, 84)], (190, 196, 205), 8, ink=INK, ink_w=6)
    ic.line([(392, 174), (448, 166)], (190, 196, 205), 9, ink=INK, ink_w=6)
    bust(ic, (226, 172, 128), (92, 160, 70), hair=(120, 76, 40))
    # straw hat: brim and crown
    ic.drawn(ellipse(256, 132, 196, 40), (236, 200, 90), hatch=1.2, shade=0.25, highlight=0.15)
    ic.drawn(roundrect(186, 54, 326, 136, 30), (236, 200, 90), hatch=1.2, shade=0.25, highlight=0.18)
    ic.line([(190, 116), (322, 116)], (200, 60, 50), 12)
    # dungarees bib
    ic.drawn(rect(196, 400, 316, 520), (70, 110, 170), hatch=0.9, highlight=0.1, edge_w=6)
    for x in (206, 306):
        ic.dot(x, 414, 8, (250, 220, 90), ink=INK)


def smith(ic):
    ic.glow((380, 150, 90), (255, 140, 40), radius=34, strength=0.7)
    bust(ic, (180, 120, 86), (110, 80, 62), hair=(40, 30, 28), hair_style="beard", shoulders=1.12)
    # leather apron
    ic.drawn(roundrect(176, 392, 336, 560, 20), (150, 92, 48), hatch=1.0, shade=0.25, highlight=0.12)
    ic.line([(176, 400), (220, 368)], (90, 56, 30), 9)
    ic.line([(336, 400), (292, 368)], (90, 56, 30), 9)
    # hammer raised, held in front of the shoulder
    ic.drawn(roundrect(390, 330, 450, 400, 16), (180, 120, 86), hatch=0.6, highlight=0.15, shade=0.2, edge_w=6)
    ic.line([(420, 350), (370, 150)], (150, 104, 56), 16, ink=tone((150, 104, 56)), ink_w=8)
    head = rotate(rect(320, 100, 430, 160), -0.27, (375, 130))
    ic.drawn(head, (120, 128, 140), edge=INK, edge_w=7, highlight=0.2)
    for (x, y, r) in ((440, 70, 22), (470, 130, 16), (330, 60, 14), (452, 196, 12)):
        ic.sparkle(x, y, r, (255, 196, 60))


def trader(ic):
    bust(ic, (196, 140, 98), (160, 60, 120), hair=(30, 24, 26), mouth="smile")
    # turban wrap
    ic.drawn(roundrect(158, 70, 354, 166, 44), (240, 170, 50), hatch=1.2, shade=0.22, highlight=0.18)
    ic.line([(170, 120), (250, 96), (340, 128)], (200, 120, 30), 9)
    ic.dot(256, 110, 14, (90, 200, 220), ink=INK)
    # sash
    ic.drawn([(160, 380), (220, 360), (360, 560), (300, 560)], (240, 170, 50), hatch=0.9, highlight=0.12, edge_w=6)
    # coin pouch and coins
    ic.drawn(circle(400, 430, 60), (160, 110, 60), hatch=1.0, highlight=0.18)
    ic.line([(370, 380), (430, 380)], (90, 56, 30), 10)
    for (x, y) in ((96, 420), (130, 470), (78, 480)):
        ic.drawn(circle(x, y, 30), (250, 200, 60), hatch=0.8, highlight=0.25, edge_w=6, rim_color=(255, 240, 150))
        ic.line([(x - 8, y), (x + 8, y)], (180, 120, 20), 6)
    ic.sparkle(60, 370, 18)


def guard(ic):
    # spear upright
    ic.line([(410, 560), (410, 90)], (130, 92, 52), 14, ink=tone((130, 92, 52)), ink_w=8)
    ic.drawn([(410, 10), (446, 92), (410, 120), (374, 92)], (200, 210, 222), edge=INK, edge_w=7, highlight=0.25)
    bust(ic, (214, 160, 120), (90, 110, 150), mouth="flat")
    # helmet: steel cap with nasal
    ic.drawn(roundrect(156, 84, 356, 190, 50), (150, 162, 180), hatch=1.0, shade=0.25, highlight=0.2, edge=(30, 36, 50))
    ic.drawn(rect(246, 170, 266, 250), (150, 162, 180), hatch=0.5, highlight=0, shade=0, edge=(30, 36, 50), edge_w=6)
    # tabard with a crest
    ic.drawn(rect(186, 380, 326, 560), (200, 60, 50), hatch=1.0, highlight=0.12)
    ic.drawn(star(256, 450, 40, 18, 5, seed=3), (250, 210, 70), hatch=0.6, highlight=0, shade=0, edge_w=6)


def fisher(ic):
    bust(ic, (216, 168, 128), (240, 196, 50), hair=(150, 100, 60), hair_style="beard")
    # sou'wester
    ic.drawn([(130, 160), (190, 76), (322, 76), (382, 160), (382, 190), (130, 190)], (250, 210, 60), hatch=1.1, shade=0.22, highlight=0.18)
    ic.line([(140, 182), (372, 182)], (200, 150, 30), 8)
    # oilskin buttons
    for y in (420, 470, 520):
        ic.dot(256, y, 9, (60, 50, 40), ink=INK)
    # a fish held up
    fish = [(330, 400), (390, 360), (450, 370), (480, 400), (450, 430), (390, 440)]
    tail = [(330, 400), (296, 362), (300, 440)]
    ic.drawn(tail, (70, 170, 200), hatch=0.8, highlight=0, edge_w=6)
    ic.drawn(fish, (90, 190, 220), hatch=1.0, highlight=0.25, rim_color=(190, 240, 250))
    ic.dot(452, 392, 7, (20, 20, 30))
    for (x, y) in ((470, 320), (500, 350)):
        ic.dot(x, y, 8, (170, 230, 250), ink=(40, 90, 120))


def miner(ic):
    # pickaxe over the shoulder
    ic.line([(110, 520), (420, 150)], (150, 104, 56), 16, ink=tone((150, 104, 56)), ink_w=8)
    pick = [(330, 90), (420, 130), (500, 230), (470, 236), (412, 168), (330, 120)]
    ic.drawn(pick, (190, 196, 206), edge=INK, edge_w=7, highlight=0.25)
    bust(ic, (150, 104, 80), (120, 130, 90), hair=(30, 26, 24), mouth="flat")
    # helmet with a lamp
    ic.glow((256, 92, 70), (255, 230, 120), radius=30, strength=0.8)
    ic.drawn(roundrect(156, 90, 356, 186, 46), (240, 180, 40), hatch=1.0, shade=0.25, highlight=0.2)
    ic.drawn(circle(256, 110, 30), (255, 244, 170), hatch=0.4, shade=0, highlight=0, edge=INK, edge_w=6)
    # soot smudges
    for (x, y) in ((206, 250), (300, 256)):
        ic.flat(ellipse(x, y, 18, 8), (110, 76, 60), wobble=1)
    ic.line([(170, 380), (240, 560)], (90, 70, 50), 12)
    ic.drawn(rect(300, 430, 350, 490), (200, 150, 60), hatch=0.6, highlight=0.1, edge_w=6)


def clerk(ic):
    bust(ic, (238, 200, 170), (60, 150, 190), hair=(220, 90, 70), hair_style="long", mouth="smile")
    # headset
    ic.line([(160, 190), (170, 110), (256, 80), (342, 110), (352, 190)], (40, 46, 60), 12)
    ic.drawn(roundrect(140, 180, 176, 240, 10), (40, 46, 60), hatch=0.4, highlight=0.1, edge=INK, edge_w=5)
    ic.line([(160, 236), (210, 286)], (40, 46, 60), 7)
    ic.dot(212, 288, 9, (90, 230, 160), ink=INK)
    # collar stripe, station blue
    ic.line([(180, 380), (256, 420), (332, 380)], (240, 240, 250), 10)
    # datapad
    pad = rotate(roundrect(330, 360, 470, 500, 14), 0.15, (400, 430))
    ic.drawn(pad, (50, 56, 70), hatch=0.5, highlight=0.15, edge=INK, edge_w=7)
    ic.flat(rotate(rect(346, 378, 454, 470), 0.15, (400, 430)), (90, 220, 200), wobble=1)
    for k in range(3):
        ic.line(rotate([(358, 396 + k * 22), (440 - k * 18, 396 + k * 22)], 0.15, (400, 430)), (220, 255, 245), 6)


def smuggler(ic):
    bust(ic, (190, 140, 110), (60, 64, 74), mouth="flat", eyes=(180, 230, 90))
    # hood
    hood = [(140, 340), (150, 150), (200, 80), (312, 80), (362, 150), (372, 340), (330, 330), (330, 180), (182, 180), (182, 330)]
    ic.drawn(hood, (80, 70, 110), hatch=1.1, shade=0.25, highlight=0.12)
    # scarf over the mouth
    ic.drawn(roundrect(176, 252, 336, 320, 14), (140, 50, 60), hatch=1.0, highlight=0.12, edge_w=6)
    # a crate under the arm, stencilled
    crate = rect(330, 400, 480, 520)
    ic.drawn(crate, (170, 120, 70), hatch=1.0, highlight=0.15)
    ic.line([(330, 400), (480, 520)], (110, 76, 40), 9)
    ic.line([(480, 400), (330, 520)], (110, 76, 40), 9)
    ic.sparkle(470, 360, 20, (180, 230, 90))


# ---------- UI icons -------------------------------------------------------------------

def talk(ic):
    bubble = roundrect(70, 100, 440, 360, 70) + []
    ic.drawn(bubble, (90, 190, 230), hatch=1.0, shade=0.22, highlight=0.18, rim_color=(200, 240, 255))
    ic.drawn([(150, 340), (130, 450), (240, 350)], (90, 190, 230), hatch=0.6, highlight=0, shade=0, edge_w=7)
    for x in (170, 256, 342):
        ic.dot(x, 230, 24, (250, 250, 240), ink=tone((90, 190, 230)))


def quest(ic):
    scroll = roundrect(110, 70, 400, 440, 20)
    ic.drawn(scroll, (240, 222, 170), hatch=1.0, shade=0.22, highlight=0.15)
    for y in (70, 440):
        ic.drawn(roundrect(80, y - 32, 430, y + 32, 30), (200, 160, 100), hatch=0.9, highlight=0.15, edge_w=7)
    ic.drawn(rect(232, 140, 280, 300), (240, 170, 40), hatch=0.6, highlight=0.1, edge=(140, 70, 10), edge_w=7)
    ic.drawn(circle(256, 360, 28), (240, 170, 40), hatch=0.6, highlight=0.1, edge=(140, 70, 10), edge_w=7)
    ic.sparkle(430, 140, 26)
    ic.sparkle(86, 260, 18)


def trade(ic):
    # two coins and a pair of hands meeting, simplified to a balance
    ic.line([(256, 80), (256, 420)], (150, 104, 56), 18, ink=tone((150, 104, 56)), ink_w=8)
    ic.line([(110, 150), (402, 150)], (190, 150, 70), 16, ink=tone((190, 150, 70)), ink_w=8)
    ic.drawn(rect(176, 410, 336, 450), (150, 104, 56), hatch=0.6, highlight=0.1, edge_w=7)
    for x, c in ((130, (250, 200, 60)), (382, (90, 200, 120))):
        ic.line([(x - 30, 160), (x, 270), (x + 30, 160)], (90, 70, 40), 5)
        ic.drawn(ellipse(x, 290, 82, 30), (200, 160, 90), hatch=0.8, highlight=0.15, edge_w=7)
        ic.drawn(circle(x, 252, 34), c, hatch=0.8, highlight=0.25, rim_color=lighter(c, 0.5), edge_w=6)


def heart(ic):
    pts = []
    for i in range(80):
        t = 2 * math.pi * i / 80
        x = 16 * math.sin(t) ** 3
        y = -(13 * math.cos(t) - 5 * math.cos(2 * t) - 2 * math.cos(3 * t) - math.cos(4 * t))
        pts.append((256 + x * 12.5, 270 + y * 12.5))
    ic.drawn(pts, (240, 70, 100), rim_color=(255, 180, 200), edge=(130, 20, 50))
    ic.sparkle(420, 110, 26, (255, 210, 230))
    ic.sparkle(110, 130, 18, (255, 210, 230))


def schedule(ic):
    # a day dial: sun half and moon half
    ic.drawn(circle(256, 256, 200), (60, 70, 120), hatch=1.0, shade=0.22, highlight=0.12)
    ic.drawn([(256, 256)] + [(256 + 200 * math.cos(a / 40 * math.pi + math.pi), 256 + 200 * math.sin(a / 40 * math.pi + math.pi)) for a in range(41)],
             (250, 190, 70), hatch=1.0, highlight=0.15, shade=0.1, edge=None)
    ic.drawn(circle(256, 160, 52), (255, 236, 120), hatch=0.6, highlight=0.2, edge=(190, 120, 20), edge_w=6)
    moon = circle(256, 360, 48)
    ic.drawn(moon, (230, 236, 255), hatch=0.6, highlight=0.15, edge=(70, 80, 140), edge_w=6)
    ic.flat(circle(276, 344, 42), (60, 70, 120), wobble=1)
    ic.line([(256, 256), (380, 190)], (240, 240, 240), 12, ink=INK, ink_w=6)
    ic.dot(256, 256, 16, (240, 240, 240), ink=INK)
    ic.outline(circle(256, 256, 200), (24, 26, 50), 9)


ICONS = {
    "farmer": (farmer, 11), "smith": (smith, 12), "trader": (trader, 13), "guard": (guard, 14),
    "fisher": (fisher, 15), "miner": (miner, 16), "clerk": (clerk, 17), "smuggler": (smuggler, 18),
    "ui_talk": (talk, 21), "ui_quest": (quest, 22), "ui_trade": (trade, 23), "ui_heart": (heart, 24), "ui_schedule": (schedule, 25),
}


def main():
    os.makedirs(OUT, exist_ok=True)
    paths = []
    for name, (fn, seed) in ICONS.items():
        ic = Icon(seed)
        fn(ic)
        p = os.path.join(OUT, name + ".png")
        ic.save(p)
        paths.append(p)
        print("wrote", p)
    if "--sheet" in sys.argv:
        sheet = os.path.join(os.environ.get("TEMP", "."), "npcs_sheet.png")
        subprocess.run([sys.executable, os.path.join(KIT, "sheet.py"), sheet] + paths + ["--ref", "vitality", "--ref", "plating", "--ref", "eagle eye", "--ref", "inferno"], check=True)


if __name__ == "__main__":
    main()
