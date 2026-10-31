#!/usr/bin/env python3
"""Dependency cycles of the inventory: strongly connected components of the row graph and of the spec graph.
Run from anywhere: python3 -I tools/package/cycles.py [--edges]. Read-only; reports which rows/specs cannot be ordered linearly."""
import os
os.chdir(os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)),"..","..")))
import csv,re,collections,sys
csv.field_size_limit(10**9)
rows={}
for f in ('classic','typed'):
    for r in csv.DictReader(open(f'docs/inventory/actor-{f}.tsv'),delimiter='\t',quoting=csv.QUOTE_NONE): rows[r['id']]=r
ch=collections.defaultdict(list)
for i in rows:
    if '.' in i: ch[i.rsplit('.',1)[0]].append(i)
def exp(t): return [t] if t in rows else ch.get(t,[])
cm={x[0]:x for x in csv.reader(open('docs/contract-map.tsv'),delimiter='\t')}
spec=lambda i: cm[i][3] if i in cm else None
# row graph
g={i:set() for i in rows}
for i,r in rows.items():
    for t in re.findall(r'A[CT]-[A-Z]+-\d+(?:\.\d+)?',r['depends_on']):
        for e in exp(t):
            if e!=i: g[i].add(e)
def scc(g):
    idx={};low={};st=[];on=set();out=[];c=[0]
    sys.setrecursionlimit(100000)
    def sc(v):
        idx[v]=low[v]=c[0];c[0]+=1;st.append(v);on.add(v)
        for w in g[v]:
            if w not in idx: sc(w);low[v]=min(low[v],low[w])
            elif w in on: low[v]=min(low[v],idx[w])
        if low[v]==idx[v]:
            comp=[]
            while True:
                w=st.pop();on.discard(w);comp.append(w)
                if w==v:break
            out.append(comp)
    for v in g:
        if v not in idx: sc(v)
    return out
rs=[c for c in scc(g) if len(c)>1]
print('row graph SCC>1:',len(rs),sorted((len(c) for c in rs),reverse=True))
for n,c in enumerate(sorted(rs,key=len,reverse=True),1):
    print(f'\n== SCC-R{n} ({len(c)} filas)')
    for i in sorted(c):
        inside=sorted(e for e in g[i] if e in c)
        print(f'  {i} [{spec(i)}] {rows[i]["feature"][:70]} -> {",".join(inside)}')
# spec graph
sg=collections.defaultdict(set)
for i in rows:
    for e in g[i]:
        a,b=spec(i),spec(e)
        if a and b and a!=b: sg[a].add(b)
for v in list(sg.values()): 
    for w in v: sg.setdefault(w,set())
ss=[c for c in scc(dict(sg)) if len(c)>1]
print('spec graph SCC>1:',len(ss),[len(c) for c in ss])
for c in ss: print(sorted(c))
