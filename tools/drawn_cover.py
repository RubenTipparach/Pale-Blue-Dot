"""How much of the sky the cloud shader actually fills at a given cover.
A transcription of clouds.wgsl's density: value noise (smoothstep-interpolated
lattice), three octaves at 0.55/0.30/0.15, times the vertical profile, then
the HZD remap `(shape*profile - open)/(1-open)` with open = 1-cover at low
cover. A column (a pixel seen from above) is drawn if any height in it is
denser than a threshold that reads as cloud."""
import numpy as np
rng = np.random.default_rng(1)
N = 64
lattice = rng.random((N, N, N))
def noise(p):
    c = np.floor(p).astype(int); f = p - c; w = f*f*(3-2*f)
    def at(dx, dy, dz): return lattice[(c[:,0]+dx)%N, (c[:,1]+dy)%N, (c[:,2]+dz)%N]
    x00 = at(0,0,0)*(1-w[:,0]) + at(1,0,0)*w[:,0]
    x10 = at(0,1,0)*(1-w[:,0]) + at(1,1,0)*w[:,0]
    x01 = at(0,0,1)*(1-w[:,0]) + at(1,0,1)*w[:,0]
    x11 = at(0,1,1)*(1-w[:,0]) + at(1,1,1)*w[:,0]
    y0 = x00*(1-w[:,1]) + x10*w[:,1]; y1 = x01*(1-w[:,1]) + x11*w[:,1]
    return y0*(1-w[:,2]) + y1*w[:,2]
def shape(q): return noise(q)*0.55 + noise(q*2.7+7)*0.30 + noise(q*6.1+19)*0.15
def ss(a, b, x): t = np.clip((x-a)/(b-a), 0, 1); return t*t*(3-2*t)
M = 20000
xy = rng.random((M, 2))*40
heights = np.linspace(0.02, 0.98, 12)
dens = []
for h in heights:
    q = np.column_stack([xy, np.full(M, 5.0)]) + h*1.7
    prof = ss(0, 0.08, h)*(1-ss(0.45, 1, h))
    dens.append(shape(q)*prof)
dens = np.array(dens)           # heights x columns
col_max = dens.max(axis=0)
s = shape(np.column_stack([xy, np.full(M, 5.0)]))
print("shape: p1 %.2f p50 %.2f p99 %.2f max %.2f" % tuple(np.percentile(s,[1,50,99]).tolist()+[s.max()]))
print("column max of shape*profile: p50 %.2f p90 %.2f p99 %.2f max %.2f" % tuple(np.percentile(col_max,[50,90,99]).tolist()+[col_max.max()]))
print("cover -> share of columns with any cloud (d > 0.05):")
for cover in [0.05, 0.1, 0.2, 0.3, 0.4, 0.5, 0.7, 1.0]:
    open_ = 1-cover
    d = np.clip((col_max - open_)/max(1-open_, 0.08), 0, 1)
    print("  %.2f -> %.1f%%" % (cover, 100*(d > 0.05).mean()))
