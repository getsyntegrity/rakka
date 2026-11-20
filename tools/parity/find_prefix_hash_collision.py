import random
M=1<<32
def h(s):
    r=0
    for c in s: r=(r*31+ord(c))%M
    return r
k=64
v=(pow(31,k,M)-1)%M
e=(v & -v).bit_length()-1
u=v>>e
print("e",e)
rng=random.Random(5)
L="abcdefghijklmnopqrstuvwxyz"
while True:
    t="".join(rng.choice(L) for _ in range(k))
    if h(t)%(1<<e)==0: break
need=(-(h(t)>>e)*pow(u,-1,1<<(32-e)))%(1<<(32-e))
mask=(1<<(32-e))-1
n=0
while True:
    s="".join(rng.choice(L) for _ in range(7))
    n+=1
    if h(s)&mask==need: break
print(n,s,t,h(s),h(s+t))
assert h(s)==h(s+t)
