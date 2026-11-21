"""A signed-distance sculpt and bake kit: smooth unions, round cones, ellipsoids, value noise, surface nets, and the
UV unwrap (charts, skyline packing, texel raster, gutter dilation) a painted bake needs.

Provenance: copied unchanged, function by function, from Mining Mike (RubenTipparach/godot-sandbox at 1739e89,
2026-11-20): `docs/mockups/tyrant_sculpt_model.py` (smin, smax, ellipsoid, round_cone, hash3, vnoise, smoothstep,
mirror, surface_nets, gradient, seg_dist) and `docs/mockups/tyrant_bake.py` (components, face_pairs, grow_charts,
axis_of, skyline_pack, split_vertices, raster, dilate). The method is that repository's `organic-creature` and
`painted-bake` skills (the Ice Wolf: `ice_wolf_sculpt_model.py`, `ice_wolf_bake.py`, `gen_ice_wolf_sculpt.py`).
The owner, 2026-11-20, on the M25 beavers and otters: "See mining mike how we did that for the wolf".

Used by tools/art/critter_sculpt.py. numpy only.
"""
import math

import numpy as np


def smin(a, b, k):
    h = np.clip(0.5 + 0.5 * (b - a) / k, 0.0, 1.0)
    return b * (1.0 - h) + a * h - k * h * (1.0 - h)


def smax(a, b, k):
    return -smin(-a, -b, k)


def ellipsoid(P, c, r, R=None):
    q = P - np.asarray(c, dtype=np.float64)
    if R is not None:
        q = q @ np.asarray(R, dtype=np.float64)
    r = np.asarray(r, dtype=np.float64)
    k0 = np.sqrt(((q / r) ** 2).sum(1))
    k1 = np.sqrt(((q / (r * r)) ** 2).sum(1))
    return k0 * (k0 - 1.0) / np.maximum(k1, 1e-6)


def round_cone(P, a, b, r1, r2):
    """Inigo Quilez's exact rounded cone: a sphere of r1 at a swept to a sphere of r2 at b."""
    a = np.asarray(a, dtype=np.float64)
    b = np.asarray(b, dtype=np.float64)
    ba = b - a
    l2 = ba @ ba
    rr = r1 - r2
    a2 = l2 - rr * rr
    il2 = 1.0 / l2
    pa = P - a
    y = pa @ ba
    z = y - l2
    x2 = ((pa * l2 - y[:, None] * ba) ** 2).sum(1)
    y2 = y * y * l2
    z2 = z * z * l2
    k = math.copysign(1.0, rr) * rr * rr * x2
    d_cap2 = np.sqrt(x2 + z2) * il2 - r2
    d_cap1 = np.sqrt(x2 + y2) * il2 - r1
    d_side = (np.sqrt(np.maximum(x2 * a2 * il2, 0.0)) + y * rr) * il2 - r1
    return np.where(np.sign(z) * a2 * z2 > k, d_cap2, np.where(np.sign(y) * a2 * y2 < k, d_cap1, d_side))


def hash3(ix, iy, iz, seed):
    n = (ix.astype(np.uint64) * np.uint64(374761393) + iy.astype(np.uint64) * np.uint64(668265263)
         + iz.astype(np.uint64) * np.uint64(2147483647) + np.uint64(seed * 144665 + 1))
    n = (n ^ (n >> np.uint64(13))) * np.uint64(1274126177)
    n = n ^ (n >> np.uint64(16))
    return (n & np.uint64(0xFFFF)).astype(np.float64) / 65535.0


def vnoise(P, freq, seed=0):
    q = P * freq
    i = np.floor(q)
    f = q - i
    u = f * f * (3 - 2 * f)
    i = i.astype(np.int64)
    out = 0.0
    for dx in (0, 1):
        for dy in (0, 1):
            for dz in (0, 1):
                w = (u[:, 0] if dx else 1 - u[:, 0]) * (u[:, 1] if dy else 1 - u[:, 1]) * (u[:, 2] if dz else 1 - u[:, 2])
                out = out + w * hash3(i[:, 0] + dx, i[:, 1] + dy, i[:, 2] + dz, seed)
    return out


def smoothstep(e0, e1, x):
    t = np.clip((x - e0) / (e1 - e0), 0, 1)
    return t * t * (3 - 2 * t)


def mirror(P):
    return np.stack([np.abs(P[:, 0]), P[:, 1], P[:, 2]], 1)


def surface_nets(F):
    s = F < 0
    nx, ny, nz = F.shape
    cs = [s[i:nx - 1 + i, j:ny - 1 + j, k:nz - 1 + k] for k in (0, 1) for j in (0, 1) for i in (0, 1)]
    anyin = np.zeros_like(cs[0])
    allin = np.ones_like(cs[0])
    for c in cs:
        anyin |= c
        allin &= c
    active = anyin & ~allin
    I, J, K = np.nonzero(active)
    vid = -np.ones(active.shape, dtype=np.int64)
    vid[I, J, K] = np.arange(len(I))
    acc = np.zeros((len(I), 3))
    cnt = np.zeros(len(I))
    edges = [((0, 0, 0), (1, 0, 0)), ((0, 1, 0), (1, 1, 0)), ((0, 0, 1), (1, 0, 1)), ((0, 1, 1), (1, 1, 1)),
             ((0, 0, 0), (0, 1, 0)), ((1, 0, 0), (1, 1, 0)), ((0, 0, 1), (0, 1, 1)), ((1, 0, 1), (1, 1, 1)),
             ((0, 0, 0), (0, 0, 1)), ((1, 0, 0), (1, 0, 1)), ((0, 1, 0), (0, 1, 1)), ((1, 1, 0), (1, 1, 1))]
    for a, b in edges:
        fa = F[I + a[0], J + a[1], K + a[2]]
        fb = F[I + b[0], J + b[1], K + b[2]]
        m = (fa < 0) != (fb < 0)
        t = fa[m] / (fa[m] - fb[m])
        pa = np.array(a, float)
        pb = np.array(b, float)
        acc[m] += pa + t[:, None] * (pb - pa)
        cnt[m] += 1
    V = np.stack([I, J, K], 1) + acc / cnt[:, None]
    quads = []
    for axis in range(3):
        e = [0, 0, 0]
        e[axis] = 1
        a0 = s[:nx - e[0], :ny - e[1], :nz - e[2]]
        a1 = s[e[0]:, e[1]:, e[2]:]
        ii, jj, kk = np.nonzero(a0 != a1)
        u, v = [d for d in range(3) if d != axis]
        base = np.stack([ii, jj, kk], 1)
        ok = (base[:, u] >= 1) & (base[:, v] >= 1) & (base[:, u] < active.shape[u]) & (base[:, v] < active.shape[v])
        ok &= base[:, axis] < active.shape[axis]
        base = base[ok]

        def cube(du, dv):
            c = base.copy()
            c[:, u] -= du
            c[:, v] -= dv
            return vid[c[:, 0], c[:, 1], c[:, 2]]
        quads.append(np.stack([cube(1, 1), cube(0, 1), cube(0, 0), cube(1, 0)], 1))
    Q = np.concatenate(quads)
    return V, Q[(Q >= 0).all(1)]


def gradient(P, f, eps=0.25):
    g = np.zeros_like(P)
    for i in range(3):
        d = np.zeros(3)
        d[i] = eps
        g[:, i] = (f(P + d) - f(P - d)) / (2 * eps)
    return g


def seg_dist(P, a, b):
    a = np.asarray(a, float)
    b = np.asarray(b, float)
    ab = b - a
    t = np.clip(((P - a) @ ab) / (ab @ ab), 0, 1)
    return np.linalg.norm(P - (a + t[:, None] * ab), axis=1)


def components(nf, pairs):
    """Connected components of faces joined by `pairs`, by label propagation with pointer jumping."""
    lab = np.arange(nf)
    if len(pairs) == 0:
        return lab
    while True:
        a, b = lab[pairs[:, 0]], lab[pairs[:, 1]]
        m = np.minimum(a, b)
        new = lab.copy()
        np.minimum.at(new, pairs[:, 0], m)
        np.minimum.at(new, pairs[:, 1], m)
        np.minimum.at(new, a, m)
        np.minimum.at(new, b, m)
        while True:
            nxt = new[new]
            if np.array_equal(nxt, new):
                break
            new = nxt
        if np.array_equal(new, lab):
            return lab
        lab = new


def face_pairs(T):
    nf = len(T)
    E = np.concatenate([T[:, [0, 1]], T[:, [1, 2]], T[:, [2, 0]]])
    F = np.tile(np.arange(nf), 3)
    E = np.sort(E, 1)
    key = E[:, 0].astype(np.int64) * 10_000_000 + E[:, 1]
    o = np.argsort(key, kind="stable")
    k = key[o]
    same = k[1:] == k[:-1]
    return np.stack([F[o][:-1][same], F[o][1:][same]], 1)


def grow_charts(fn, area, part, pairs, cone=50.0, merge=75.0, min_faces=60):
    """Charts by region growing: from the largest unassigned face, take every connected face of the same body part
    whose normal is within `cone` degrees of the seed's, so a chart projects flat along that normal without folding
    and stretches at most 1 / cos(cone). Tiny charts then join a neighbour within `merge` degrees."""
    from collections import deque
    nf = len(fn)
    adj_n = np.bincount(pairs.ravel(), minlength=nf)
    ptr = np.concatenate([[0], np.cumsum(adj_n)])
    adj = np.empty(ptr[-1], dtype=np.int64)
    fill = ptr[:-1].copy()
    for a, b in ((pairs[:, 0], pairs[:, 1]), (pairs[:, 1], pairs[:, 0])):
        o = np.argsort(a, kind="stable")
        aa, bb = a[o], b[o]
        pos = fill[aa] + (np.arange(len(aa)) - np.searchsorted(aa, aa))
        adj[pos] = bb
        fill += np.bincount(aa, minlength=nf)
    chart = -np.ones(nf, dtype=np.int64)
    axis = []
    c_lim = math.cos(math.radians(cone))
    order = np.argsort(-area)
    nc = 0
    for seed in order:
        if chart[seed] >= 0:
            continue
        d = fn[seed]
        chart[seed] = nc
        q = deque([seed])
        acc = fn[seed] * area[seed]
        while q:
            f = q.popleft()
            for g in adj[ptr[f]:ptr[f + 1]]:
                if chart[g] < 0 and part[g] == part[seed] and fn[g] @ d > c_lim:
                    chart[g] = nc
                    q.append(g)
        axis.append(d)
        nc += 1
    axis = np.array(axis)
    cnt = np.bincount(chart, minlength=nc)
    m_lim = math.cos(math.radians(merge))
    for _ in range(4):
        small = cnt[chart] < min_faces
        cand = pairs[small[pairs[:, 0]] & ~small[pairs[:, 1]]]
        cand = np.concatenate([cand, pairs[small[pairs[:, 1]] & ~small[pairs[:, 0]]][:, ::-1]])
        moved = False
        for f, g in cand:
            if cnt[chart[f]] < min_faces and fn[f] @ axis[chart[g]] > m_lim:
                cnt[chart[f]] -= 1
                chart[f] = chart[g]
                cnt[chart[g]] += 1
                moved = True
        if not moved:
            break
    _, chart = np.unique(chart, return_inverse=True)
    first = np.zeros(chart.max() + 1, dtype=np.int64)
    first[chart[::-1]] = np.arange(nf)[::-1]
    return chart, axis_of(fn, area, chart)


def axis_of(fn, area, chart):
    acc = np.zeros((chart.max() + 1, 3))
    np.add.at(acc, chart, fn * area[:, None])
    return acc / np.maximum(np.linalg.norm(acc, axis=1), 1e-12)[:, None]


def skyline_pack(wh, size):
    """Bottom-left skyline packing of rectangles (tallest first). Returns positions or None if they do not fit."""
    sky = np.zeros(size, dtype=np.int64)
    pos = np.zeros((len(wh), 2))
    for c in np.argsort(-(wh[:, 1] * 4096 + wh[:, 0])):
        w, h = int(wh[c, 0]), int(wh[c, 1])
        if w > size or h > size:
            return None
        # The lowest place the rectangle fits: max of the skyline under it, for every x.
        if w == 1:
            tops = sky
        else:
            from numpy.lib.stride_tricks import sliding_window_view
            tops = sliding_window_view(sky, w).max(1)
        x = int(np.argmin(tops))
        y = int(tops[x])
        if y + h > size:
            return None
        sky[x:x + w] = y + h
        pos[c] = (x, y)
    return pos


def split_vertices(T, chart, corner_uv):
    """One vertex per (original vertex, chart): the mesh with its UV seams."""
    key = chart[:, None].astype(np.int64) * 10_000_000 + T
    flat = key.reshape(-1)
    uniq, first, inv = np.unique(flat, return_index=True, return_inverse=True)
    src = T.reshape(-1)[first]
    uv = corner_uv.reshape(-1, 2)[first]
    return src, inv.reshape(-1, 3), uv


def raster(T, uv, size):
    """Every texel inside a triangle: (texel y, x, triangle, barycentrics)."""
    tx = uv * size
    tx[..., 1] = (1 - uv[..., 1]) * size
    A = tx[T]                                                     # (nf, 3, 2)
    lo = np.floor(A.min(1) - 0.5).astype(int)
    hi = np.ceil(A.max(1) - 0.5).astype(int)
    ext = np.maximum(hi - lo + 1, 1)
    out_y, out_x, out_t, out_b = [], [], [], []
    m = np.maximum(ext[:, 0], ext[:, 1])
    for cap in (4, 8, 16, 32, 64, 128):
        sel = np.nonzero((m <= cap) & (m > cap // 2) if cap > 4 else (m <= cap))[0]
        if len(sel) == 0:
            continue
        oy, ox = np.meshgrid(np.arange(cap), np.arange(cap), indexing="ij")
        oy, ox = oy.reshape(-1), ox.reshape(-1)
        for s in range(0, len(sel), max(1, 400000 // (cap * cap))):
            ts = sel[s:s + max(1, 400000 // (cap * cap))]
            X = lo[ts, 0][:, None] + ox[None, :] + 0.5
            Y = lo[ts, 1][:, None] + oy[None, :] + 0.5
            a, b, c = A[ts, 0], A[ts, 1], A[ts, 2]
            den = (b[:, 1] - c[:, 1]) * (a[:, 0] - c[:, 0]) + (c[:, 0] - b[:, 0]) * (a[:, 1] - c[:, 1])
            den = np.where(np.abs(den) < 1e-12, 1e-12, den)[:, None]
            l0 = ((b[:, 1] - c[:, 1])[:, None] * (X - c[:, 0][:, None]) + (c[:, 0] - b[:, 0])[:, None] * (Y - c[:, 1][:, None])) / den
            l1 = ((c[:, 1] - a[:, 1])[:, None] * (X - c[:, 0][:, None]) + (a[:, 0] - c[:, 0])[:, None] * (Y - c[:, 1][:, None])) / den
            l2 = 1 - l0 - l1
            eps = -0.02
            inside = (l0 >= eps) & (l1 >= eps) & (l2 >= eps) & (X < size) & (Y < size) & (X >= 0) & (Y >= 0)
            ii, jj = np.nonzero(inside)
            out_y.append((Y[ii, jj] - 0.5).astype(np.int32))
            out_x.append((X[ii, jj] - 0.5).astype(np.int32))
            out_t.append(ts[ii])
            bc = np.stack([l0[ii, jj], l1[ii, jj], l2[ii, jj]], 1)
            bc = np.clip(bc, 0, None)
            out_b.append(bc / bc.sum(1)[:, None])
    return (np.concatenate(out_y), np.concatenate(out_x), np.concatenate(out_t), np.concatenate(out_b))


def dilate(img, filled, passes=8):
    """Grow the baked texels into the gutters so filtering and mips never read the background."""
    img = img.copy()
    filled = filled.copy()
    for _ in range(passes):
        acc = np.zeros_like(img)
        cnt = np.zeros(filled.shape)
        for dy, dx in ((-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (1, 1), (-1, 1), (1, -1)):
            f = np.roll(np.roll(filled, dy, 0), dx, 1)
            acc += np.roll(np.roll(img, dy, 0), dx, 1) * f[..., None]
            cnt += f
        grow = (~filled) & (cnt > 0)
        img[grow] = acc[grow] / cnt[grow][:, None]
        filled |= grow
    img[~filled] = img[filled].mean(0)
    return img
