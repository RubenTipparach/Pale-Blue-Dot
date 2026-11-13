"""Star flicker between consecutive captures (`steady-stars`).

Usage: python tools/star_flicker.py <dir> <tag>, with <dir>/<tag>-120.png to
<tag>-125.png six consecutive frames of one scene. Each star found in a frame
is matched to the nearest star in the next (within 5 px) and its light summed
over 5 x 5 pixels in linear light; prints how many vanish and how much each
one's light changes from frame to frame.
"""
import sys, numpy as np
from PIL import Image
S,tag=sys.argv[1],sys.argv[2]
def load(f):
    a=np.asarray(Image.open(f"{S}/{tag}-{f}.png").convert("RGB")).astype(np.float32)/255
    a=np.where(a<=0.04045,a/12.92,((a+0.055)/1.055)**2.4)   # to linear light
    a=a.mean(2)[120:1500,:]
    return a-np.median(a)
def peaks(s):
    t=np.percentile(s,99.5)
    m=s>max(t,0.004)
    for dy in (-1,0,1):
        for dx in (-1,0,1):
            if dy or dx: m&= s>=np.roll(np.roll(s,dy,0),dx,1)
    ys,xs=np.nonzero(m); keep=(ys>3)&(ys<s.shape[0]-3)&(xs>3)&(xs<s.shape[1]-3)
    return np.stack([ys[keep],xs[keep]],1)
def energy(s,p): y,x=p; return np.clip(s[y-2:y+3,x-2:x+3],0,None).sum()
ratios=[];lost=0;total=0
prev=load(120); pp=peaks(prev)
for f in range(121,126):
    cur=load(f); cp=peaks(cur)
    for p in pp:
        total+=1
        d=np.abs(cp-p).sum(1) if len(cp) else np.array([99])
        j=d.argmin()
        if d[j]>5: lost+=1; continue
        e0=energy(prev,p); e1=energy(cur,cp[j])
        if e0>0: ratios.append(e1/e0)
    prev,pp=cur,cp
r=np.array(ratios)
print(f"{tag}: {total} stars followed; {lost} ({lost/total*100:.1f}%) gone next frame; each star's light (5x5): median change {np.median(np.abs(r-1))*100:.0f}%, over 25% in {(np.abs(np.log2(r))>np.log2(1.25)).mean()*100:.0f}%")
