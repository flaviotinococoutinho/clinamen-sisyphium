#!/usr/bin/env python3
"""Independent integer model of docs/06-consilium.md; emits deterministic search vectors.

The search and state decoding are independent of the Rust implementation. Only the
four cost tables are calibrated. --baseline retains the initial table and digest.
"""
import argparse
from pathlib import Path
import sys

BASE=10
STRATUM=18
LAYERS=(2,5,1,1,2,1,1,0)
STILUS=(0,0,34,3)

from dataclasses import dataclass, replace
from itertools import product
MASK=(1<<64)-1
KNUTH=0x9E3779B97F4A7C15
INTERVALS=(42,125,250,1000,60000)
EXP=(255,198,154,120,93,73,56,44,34,26,20,16,12,9,7,5)
@dataclass(frozen=True)
class R:
    stilus:int=0
    strata:int=4
    tick:int=42
    interval:int=42
    layers:int=199
    lines:int=6
@dataclass(frozen=True)
class H:
    width:int=320
    height:int=180
    compute:int=255
    hz:int=24
    cells:bool=False
    bits:int=8
@dataclass(frozen=True)
class W:
    seed:int
    conatus:int
    onus:int
    calm:int
    modus:int
    ramus:int
    eventus:int
    closures:int

def mix(x):
    x ^= KNUTH
    x=((x ^ (x>>32))*KNUTH)&MASK
    x=((x ^ (x>>29))*KNUTH)&MASK
    return x ^ (x>>32)
def galois(g):
    g=g or 0xACE1
    return (g>>1) ^ (0xB400 if g&1 else 0)
def raw(v,m):
    return W(v&65535 or 0xACE1,(v>>16)&255,(v>>24)&255,(v>>56)&63,v>>62,(m>>20)&3,(m>>16)&15,m&16383)
def genesis(seed):
    return W(seed&65535 or 0xACE1,192,32,24,0,3,(seed>>16)&7,0)
def tick(e):
    return 42 if e>=179 else 125 if e>=77 else 250 if e>=39 else 60000
def force(r,h,w,e):
    quiet=w.modus==1 or e<39
    mode=2 if h.cells else 3 if h.bits==1 or quiet else 0 if w.modus==2 and r.stilus==2 else r.stilus
    t=60000 if quiet else tick(e)
    minimum=60000 if h.hz==0 else next(i for i in INTERVALS if i>=max(t,(1000+h.hz-1)//h.hz))
    return replace(r,stilus=mode,strata=max(r.strata,2) if w.modus==2 else r.strata,tick=t,interval=max(r.interval,minimum))
def floor(h,w,e):
    return force(R(1,1,tick(e),60000,0,0),h,w,e)
def cost(r,h):
    units=BASE+STRATUM*max(0,r.strata-1)+STILUS[r.stilus]
    units+=sum(c*(r.lines if k==2 else 1) for k,c in enumerate(LAYERS) if r.layers&(1<<k))
    scale=max(1,min(h.width//320,h.height//180)) if r.stilus==1 else 1
    return units*(h.width//scale)*(h.height//scale)*1000//(57600*r.interval)
def value(r,w,e):
    active=w.ramus!=3
    dream=w.modus==2
    values=(8*(1+active),5*(1+(w.onus>128)),2*r.lines*(1+active),6*(13<=w.eventus<=15),3*((w.closures>>1)!=0),7*((w.conatus>200 and w.onus>200) or dream),2*(1+(w.calm>=48)),2*(1+dream))
    score=sum(v for k,v in enumerate(values) if r.layers&(1<<k))
    score+=max(0,r.strata-1)*2*(1+dream)
    score+={42:6 if e>=179 else 0,125:4,250:2,1000:1,60000:0}[r.interval]
    score+=((8 if w.modus==3 else 4),3,(6 if 39<=e<=76 and w.calm>=48 else 2),(6 if w.modus==1 else 1))[r.stilus]
    return score
def search(h,w,e):
    allowance=h.compute*e//64
    least=floor(h,w,e)
    if allowance==0 or cost(least,h)>allowance:
        return least
    active=199 | (8 if 13<=w.eventus<=15 else 0) | (16 if w.closures>>1 else 0) | (32 if (w.conatus>200 and w.onus>200) or w.modus==2 else 0)
    base=force(R(tick=tick(e),interval=0,layers=active),h,w,e)
    cur=base if cost(base,h)<=allowance else least
    best=cur
    maximum=max(value(force(R(stilus=s,interval=0,layers=255),h,w,e),w,e) for s in range(4))
    score=lambda r:8*value(r,w,e)-cost(r,h)
    g=w.seed
    for _ in range(64):
        if 5*value(best,w,e)>=4*maximum:
            break
        g=galois(g); dim=g%5
        g=galois(g); pick=g>>4
        changes=({'stilus':pick%4},{'strata':1+pick%4},{'interval':INTERVALS[pick%5]},{'layers':cur.layers^(1<<(pick%8))},{'lines':pick%7})
        p=force(replace(cur,**changes[dim]),h,w,e)
        if p==cur or cost(p,h)>allowance:
            continue
        delta=score(cur)-score(p)
        accepted=delta<=0
        if delta>0 and e//16>0:
            g=galois(g)
            accepted=(g&255)<EXP[min(15,4*delta//(e//16))]
        if accepted:
            cur=p
            if score(p)>score(best):
                best=p
    return best

HOSTS=(H(1920,1080,255,24),H(640,360,96,24),H(3840,2160,255,60),H(1280,720,40,60,True),H(320,180,255,24,False,1),H(272,60,96,1,False,4),H(3840,2160,10,60,False,1))

def vectors():
    for seed,h,e in product(range(1,33),HOSTS,(12,40,60,100,150,200,255)):
        v,m=mix(seed),mix(seed^KNUTH)
        yield v,m,h,e,search(h,raw(v,m),e)
    # Zero-budget cases include compute=1, energy=1, whose integer budget is zero.
    for wh,mode,ce,cells in product(((320,180),(1920,1080)),range(4),((0,255),(255,0),(0,0),(1,1)),(False,True)):
        h=H(*wh,ce[0],cells=cells)
        v=1 | (192<<16) | (32<<24) | (24<<56) | (mode<<62)
        m=3<<20
        w=raw(v,m)
        r=search(h,w,ce[1])
        assert r==floor(h,w,ce[1])
        yield v,m,h,ce[1],r

def corpus():
    rows=['verbum\tmemoria\twidth\theight\tcompute\thz\tcells\tbits\tenergy\tstilus\tstrata\ttick\tinterval\tlayers\tlines']
    stream=bytearray()
    for index,(v,m,h,e,r) in enumerate(vectors()):
        rows.append('\t'.join(map(str,(v,m,h.width,h.height,h.compute,h.hz,int(h.cells),h.bits,e,r.stilus,r.strata,r.tick,r.interval,r.layers,r.lines))))
        if index<1568:
            stream.extend((r.stilus,r.strata));stream.extend(r.tick.to_bytes(4,'little'));stream.extend(r.interval.to_bytes(4,'little'));stream.extend((r.layers,r.lines))
    digest=0xCBF29CE484222325
    for octet in stream:
        digest=((digest^octet)*0x100000001B3)&MASK
    return '\n'.join(rows)+'\n',digest

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check',type=Path,help='compare with a committed corpus')
    parser.add_argument('--baseline',action='store_true',help='verify the initial 1568-vector digest')
    args=parser.parse_args()
    if args.baseline:
        BASE,STRATUM,LAYERS,STILUS=10,3,(6,5,1,2,1,4,1,4),(0,0,2,3)
    text,digest=corpus()
    if args.baseline and digest!=0x61B689BF40123868:
        parser.exit(1,'initial model digest mismatch\n')
    if args.check:
        if args.check.read_text(encoding='utf-8')!=text:
            parser.exit(1,'independent corpus mismatch\n')
    elif not args.baseline:
        sys.stdout.write(text)
    print(f'Independent model: 1568 searches + 64 zero-budget cases; FNV-1a={digest:016X}',file=sys.stderr)
