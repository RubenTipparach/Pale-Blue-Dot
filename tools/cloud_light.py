"""Measurement instrument for openspec/changes/cloud-lighting.

A transcription of `cloud_density`, `cloud_shadow` and the lighting in
`cloud_march` (assets/shaders/clouds.wgsl), run over a flat, horizontally
uniform slab with the shipped weather.ron values. It prints what a cloud's top
(viewed from above) and base (viewed from below) come out as, so the change can
be judged on numbers before and after. It is a copy of the shader's arithmetic
and is only as right as that copy: when the shader changes, change this with it
and say so in the change's design.

    python3 tools/cloud_light.py
"""
import math
TH=260.0
def ss(a,b,x):
    t=min(max((x-a)/(b-a),0),1); return t*t*(3-2*t)
def dens(z,shape,thr):  # z metres above slab base
    if z<0 or z>TH: return 0.0
    h=z/TH; prof=ss(0,.3,h)*(1-ss(.62,1,h))
    d=max(shape*prof-thr,0)/max(1-thr,1e-3); return d*(2-d)
def shadow(z,sun_el,shape,thr):
    step=TH*0.32; dep=0
    for i in range(4):
        dep+=dens(z+math.sin(sun_el)*step*(i+.5),shape,thr)
    return math.exp(-dep*step*0.05)
def march(from_top,sun_el,shape,cover):
    thr=0.61+(0.2016-0.61)*cover; bd=0.34+(0.12-0.34)*cover; ext=0.0115+(0.03-0.0115)*cover
    day=math.sin(sun_el)  # zenith sun dot for a column
    steps=12; st=TH/steps; T=1; L=0
    for i in range(steps):
        z=TH-st*(i+.5) if from_top else st*(i+.5)
        d=dens(z,shape,thr)
        if d<=0: continue
        sh=shadow(z,sun_el,shape,thr)
        lit=(bd+(1-bd)*sh)*(0.045+day*0.9)
        f=math.exp(-d*st*ext); L+=lit*T*(1-f); T*=f
    a=1-T
    return L/a if a>1e-4 else 0, a

for cover,shape in [(0.2,0.9),(0.2,1.0),(0.6,0.8),(1.0,0.6),(1.0,0.9)]:
    for el in [60,20]:
        top,at=march(True,math.radians(el),shape,cover); base,ab=march(False,math.radians(el),shape,cover)
        print(f"cover {cover} shape {shape} sun {el:>2}deg: top {top:.3f} (alpha {at:.2f}) base {base:.3f} (alpha {ab:.2f}) base/top {base/top if top else 0:.2f}")
