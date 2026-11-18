# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-py4t6s5v/target-int-map-hash-starts-at-zero/release/examples/classic-parity`

Passed: 1410/1417

Fallback rate: not reported by this build

## Failing cases

### ok-user-info-lone-surrog

Arguments: `path-from-string`

status:

```diff
expected exit code 0, got exit code 2
```

stdout:

```diff
- {"ok":true,"toString":"pekko://sys\ud800@host:2552/user","name":"user","elements":["user"],"uid":0,"toStringWithoutAddress":"/user","toSerializationFormat":"pekko://sys\ud800@host:2552/user","address":{"protocol":"pekko","system":"sys\ud800","host":"host","port":2552,"toString":"pekko://sys\ud800@host:2552"}}
```

stderr:

```diff
+ input outside the API domain: lone UTF-16 surrogate (public inputs are &str; decision Q-X-17)
```

### uc-0248

Arguments: `util-core-case`

stdout:

```diff
- IM-01 empty map -> ok 0 [] ImmutableIntMap() 1 -2147483648 f
+ IM-01 empty map -> ok 0 [] ImmutableIntMap() 0 -2147483648 f
```

### uc-0250

Arguments: `util-core-case`

stdout:

```diff
- IM-03 hashCode is Arrays.hashCode of the key/value array -> ok ok 1281 ok 1231681 ok 1035575201 new 1077921 new new 1
+ IM-03 hashCode is Arrays.hashCode of the key/value array -> ok ok 320 ok 308160 ok 148071520 new 154400 new new 0
```

### uc-0256

Arguments: `util-core-case`

stdout:

```diff
- IM-09 extreme keys and values -> ok ok ok ok ok [-2147483648,-1,0,2147483647] ImmutableIntMap(-2147483648 -> -2147483648, -1 -> 1, 0 -> 0, 2147483647 -> 2147483647) -1835160125 -2147483648 2147483647 new [-2147483648,-1,2147483647] 887474819
+ IM-09 extreme keys and values -> ok ok ok ok ok [-2147483648,-1,0,2147483647] ImmutableIntMap(-2147483648 -> -2147483648, -1 -> 1, 0 -> 0, 2147483647 -> 2147483647) -27705662 -2147483648 2147483647 new [-2147483648,-1,2147483647] -28862
```

### uc-0257

Arguments: `util-core-case`

stdout:

```diff
- IM-21 random sequence seed 21 -> ok f evaluated/new 963 same -2147483648 evaluated/new -2147483648 new evaluated/new 776490 f f ok new -2147483648 776458 776458 -2147483648 2 f ok ok evaluated/new ok ok evaluated/new -1962884029 ok new f lazy/same 478002626 ok t 477999743 t same ok lazy/same t -1 lazy/same 0 ok lazy/same f ok f t new 475973955 ok -204084415 ok -2147483648 t lazy/same -200390331 -200390331 t f ok ok new -2147483648 1944323003 ok t 599323129 1 -1 lazy/same ok 599323129 599323129 ok ok lazy/same lazy/same t 1300387709 lazy/same ok new 225519773 t ok -2147483648 same evaluated/new ok ok 0 t lazy/same ok new ok t -2 ok lazy/same t new new f 2 -1 ok lazy/same ok 2 ok 299104734 2 2 new f new -1374560828 lazy/same evaluated/new ok ok 2 lazy/same 1625240389 t same lazy/same new ok ok 0 ok new -688764250 ok ok ok t new t -1667813591 ok ok -1848185046 ok 2 same lazy/same new -1848185046 -1848185046 ok lazy/same ok new new ok ok ok t ok lazy/same t 1646465894 ok -1096888732 ok 0 ok evaluated/new ok ok new ok -2 -2 lazy/same -1745779306 ok lazy/same t 0 -1158973292 ok -2 t ok ok -2 1360986642 ok -2 ok f 0 -1158971370 ok new ok -1158971370 new ok ok t lazy/same t new lazy/same f -2 379092918 ok 379096762 -2 ok ok ok same -2147483648 new 0 f -975787170 -2147483648 lazy/same t -975787170 t evaluated/new f f lazy/same ok 1478127747 new 1 t ok 777063169 new 939354947 new same -2 -2147483648 1 ok -2 -2 ok 1 -2147483648 ok 1475000898 lazy/same new ok -2147483648 -2 -2147483648 same 163490022 ok evaluated/new 1188348483 lazy/same 2 ok new ok lazy/same ok 1688151523 new f ok new lazy/same ok ok ok f 1846460800 same lazy/same ok new evaluated/new ok same 1847532315 ok t lazy/same f lazy/same lazy/same evaluated/new ok ok lazy/same new ok -1903612909 same -1903612909 lazy/same -2 t t t ok lazy/same ok ok -768472141 lazy/same new evaluated/new ok new lazy/same 0 644620181 t ok 0 ok t 1 t t lazy/same ok t t lazy/same -2 ok lazy/same 633656056 t 633656056 t f new ok ok t ok new new ok -2147483648 -2147483648 -2 -1 t 1751305369 0 ok evaluated/new ok -2 -2147483648 t new ok ok same 2 ok t ok ok t ok t ok 0 1864722236 lazy/same new t ok ok new 187364189 187364189 1 1 ok ok t ok ok -2 [-2147483648,-5,0,1,2,7,100,2147483647] ImmutableIntMap(-2147483648 -> 0, -5 -> -2, 0 -> 2, 1 -> -1, 2 -> 1, 7 -> -1, 100 -> 1, 2147483647 -> 0) 8 -1529402599
+ IM-21 random sequence seed 21 -> ok f evaluated/new 2 same -2147483648 evaluated/new -2147483648 new evaluated/new -147031 f f ok new -2147483648 -147063 -147063 -2147483648 2 f ok ok evaluated/new ok ok evaluated/new -1669481022 ok new f lazy/same -2020013311 ok t -2020016194 t same ok lazy/same t -1 lazy/same 0 ok lazy/same f ok f t new -2022041982 ok 1592866944 ok -2147483648 t lazy/same 1596561028 1596561028 t f ok ok new -2147483648 -553692934 ok t 892726136 1 -1 lazy/same ok 892726136 892726136 ok ok lazy/same lazy/same t 1593790716 lazy/same ok new 2022471132 t ok -2147483648 same evaluated/new ok ok 0 t lazy/same ok new ok t -2 ok lazy/same t new new f 2 -1 ok lazy/same ok 2 ok -1054204963 2 2 new f new -1081157821 lazy/same evaluated/new ok ok 2 lazy/same 117688580 t same lazy/same new ok ok 0 ok new 1108187109 ok ok ok t new t 129137768 ok ok 939230441 ok 2 same lazy/same new 939230441 939230441 ok lazy/same ok new new ok ok ok t ok lazy/same t 138914085 ok 1844768867 ok 0 ok evaluated/new ok ok new ok -2 -2 lazy/same 1195878293 ok lazy/same t 0 1782684307 ok -2 t ok ok -2 7676945 ok -2 ok f 0 1782686229 ok new ok 1782686229 new ok ok t lazy/same t new lazy/same f -2 -1128458891 ok -1128455047 -2 ok ok ok same -2147483648 new 0 f -682384163 -2147483648 lazy/same t -682384163 t evaluated/new f f lazy/same ok -29424062 new 1 t ok -730488640 new 1232757954 new same -2 -2147483648 1 ok -2 -2 ok 1 -2147483648 ok 1768403905 lazy/same new ok -2147483648 -2 -2147483648 same 1960441381 ok evaluated/new -319203326 lazy/same 2 ok new ok lazy/same ok 334841826 new f ok new lazy/same ok ok ok f 2139863807 same lazy/same ok new evaluated/new ok same 2140935322 ok t lazy/same f lazy/same lazy/same evaluated/new ok ok lazy/same new ok 883802578 same 883802578 lazy/same -2 t t t ok lazy/same ok ok -2121781838 lazy/same new evaluated/new ok new lazy/same 0 -862931628 t ok 0 ok t 1 t t lazy/same ok t t lazy/same -2 ok lazy/same -719653641 t -719653641 t f new ok ok t ok new new ok -2147483648 -2147483648 -2 -1 t -746710568 0 ok evaluated/new ok -2 -2147483648 t new ok ok same 2 ok t ok ok t ok t ok 0 -1585773189 lazy/same new t ok ok new -1320187620 -1320187620 1 1 ok ok t ok ok -2 [-2147483648,-5,0,1,2,7,100,2147483647] ImmutableIntMap(-2147483648 -> 0, -5 -> -2, 0 -> 2, 1 -> -1, 2 -> 1, 7 -> -1, 100 -> 1, 2147483647 -> 0) 8 1412255000
```

### uc-0258

Arguments: `util-core-case`

stdout:

```diff
- IM-22 random sequence seed 22 -> ok ok -2147483648 -2147483648 evaluated/new ok 743406753 ok -2147483648 ok 1 0 lazy/same same ok 0 1796935329 t lazy/same -2147483648 ok lazy/same 1796935329 ok ok ok ok 1162734395 ok ok lazy/same ok ok same lazy/same ok 172412217 f ok 172412217 same new ok -1404889989 0 lazy/same -1404889989 0 -1404889989 t 0 ok t lazy/same evaluated/new new 190503900 -2147483648 t t ok t 1 new f new -489081035 lazy/same -489081035 t evaluated/new ok ok ok -2053679688 f ok ok evaluated/new t ok t t 965781979 lazy/same 965781979 new ok ok ok 1571063168 new evaluated/new ok ok t t 1 ok -205791236 new lazy/same 1698104863 1698104863 t ok ok ok ok ok new ok ok ok ok -542555851 t -542555851 new ok ok same -848976097 0 -848976097 t evaluated/new new t 1 ok -2 -228520365 new ok -2 1 new ok new ok -1226526351 -1226526351 -1226526351 ok 2 -2 ok ok new t ok f 250749269 -1 t 250749269 lazy/same ok lazy/same ok t t new t same new ok -2147483648 new evaluated/new ok -1882488994 new evaluated/new t new -1 -2147483648 2050235238 t t 2050235238 new f ok lazy/same ok 2 -1299427421 -2 lazy/same new evaluated/new t new f ok new ok 1184970277 ok evaluated/new evaluated/new same ok -1 ok t ok lazy/same -719574692 lazy/same new ok ok ok -1930107168 evaluated/new lazy/same t 2 t ok t -2147483648 lazy/same lazy/same ok ok new -2147483648 ok t lazy/same -2147483648 2 ok ok -572706879 ok ok new ok lazy/same 451701279 451701279 t t ok new evaluated/new 365865983 ok ok -1141685826 -1141685826 t -1 new ok t lazy/same ok new lazy/same ok -1029904037 ok ok ok new -1 ok ok new evaluated/new 1749451864 ok ok ok ok -35188140 t ok -2 0 new same t -2009680589 ok ok new ok lazy/same -1 lazy/same -1125927408 ok ok ok lazy/same 1 lazy/same t t new -299474195 ok -2147483648 new -2147483648 same ok ok ok same evaluated/new 1 694202899 new -1 new ok evaluated/new t lazy/same -1 -135944398 t t -2 lazy/same -135944398 new new -519847850 evaluated/new lazy/same f -733731244 -733731244 t same lazy/same lazy/same -2 new ok ok same ok ok -2 ok -2147483648 ok ok lazy/same ok 1617118967 ok 1 new -2 -2 -825440421 evaluated/new t 377378905 lazy/same lazy/same ok -1 t 377378905 new -1702012296 -1702012296 t ok t t ok same -1 ok ok lazy/same -2123407592 new -1188451980 lazy/same -1 ok -1186604938 f -1186604938 t same [-2147483648,-1,0,1,2,7,2147483647] ImmutableIntMap(-2147483648 -> -2, -1 -> -1, 0 -> -2, 1 -> -2, 2 -> 1, 7 -> 1, 2147483647 -> 2) 7 -1186604938
+ IM-22 random sequence seed 22 -> ok ok -2147483648 -2147483648 evaluated/new ok -144096928 ok -2147483648 ok 1 0 lazy/same same ok 0 -701080608 t lazy/same -2147483648 ok lazy/same -701080608 ok ok ok ok 1456137402 ok ok lazy/same ok ok same lazy/same ok -1335139592 f ok -1335139592 same new ok -1111486982 0 lazy/same -1111486982 0 -1111486982 t 0 ok t lazy/same evaluated/new new 483906907 -2147483648 t t ok t 1 new f new 1318373428 lazy/same 1318373428 t evaluated/new ok ok ok -1760276681 f ok ok evaluated/new t ok t t -387527718 lazy/same -387527718 new ok ok ok 217753471 new evaluated/new ok ok t t 1 ok -1559100933 new lazy/same 190553054 190553054 t ok ok ok ok ok new ok ok ok ok 301916020 t 301916020 new ok ok same 2092681502 0 2092681502 t evaluated/new new t 1 ok -2 -1581830062 new ok -2 1 new ok new ok -933123344 -933123344 -933123344 ok 2 -2 ok ok new t ok f -1256802540 -1 t -1256802540 lazy/same ok lazy/same ok t t new t same new ok -2147483648 new evaluated/new ok -85537635 new evaluated/new t new -1 -2147483648 -437277595 t t -437277595 new f ok lazy/same ok 2 497523938 -2 lazy/same new evaluated/new t new f ok new ok -1313045660 ok evaluated/new evaluated/new same ok -1 ok t ok lazy/same 2067840795 lazy/same new ok ok ok 857308319 evaluated/new lazy/same t 2 t ok t -2147483648 lazy/same lazy/same ok ok new -2147483648 ok t lazy/same -2147483648 2 ok ok 271764992 ok ok new ok lazy/same -901608418 -901608418 t t ok new evaluated/new 1210337854 ok ok -297213955 -297213955 t -1 new ok t lazy/same ok new lazy/same ok 1911753562 ok ok ok new -1 ok ok new evaluated/new 241900055 ok ok ok ok -1388497837 t ok -2 0 new same t 777734898 ok ok new ok lazy/same -1 lazy/same 1661488079 ok ok ok lazy/same 1 lazy/same t t new -1807026004 ok -2147483648 new -2147483648 same ok ok ok same evaluated/new 1 -659106798 new -1 new ok evaluated/new t lazy/same -1 -1489254095 t t -2 lazy/same -1489254095 new new -226444843 evaluated/new lazy/same f 2053684243 2053684243 t same lazy/same lazy/same -2 new ok ok same ok ok -2 ok -2147483648 ok ok lazy/same ok 263809270 ok 1 new -2 -2 1961975066 evaluated/new t -975930792 lazy/same lazy/same ok -1 t -975930792 new 1085403191 1085403191 t ok t t ok same -1 ok ok lazy/same 818250007 new 1598963507 lazy/same -1 ok 1600810549 f 1600810549 t same [-2147483648,-1,0,1,2,7,2147483647] ImmutableIntMap(-2147483648 -> -2, -1 -> -1, 0 -> -2, 1 -> -2, 2 -> 1, 7 -> 1, 2147483647 -> 2) 7 1600810549
```

### uc-0259

Arguments: `util-core-case`

stdout:

```diff
- IM-23 random sequence seed 23 -> ok 1 evaluated/new evaluated/new new -2147483648 1179 same evaluated/new 772862 f ok ok ok f evaluated/new f 1453234017 1453234017 t ok -1 same same 1 ok lazy/same -1 ok evaluated/new ok ok lazy/same ok -172043874 -172043874 -172043874 new evaluated/new lazy/same -1 t ok ok new -2147483648 lazy/same lazy/same new new 1160757308 t lazy/same t 1160757308 2 2 ok ok 2 f lazy/same ok t lazy/same lazy/same new lazy/same new f new ok same 1022836862 ok ok 1055283963 1055283963 -2147483648 ok same ok ok same same ok same f -761396809 ok lazy/same t -2147483648 -761396809 lazy/same -761396809 lazy/same -761396809 2 new ok t -577326050 evaluated/new ok 1 same f ok new new ok ok same ok ok -1670554210 lazy/same new ok new f evaluated/new ok lazy/same new 1364401160 lazy/same -2147483648 -1 ok ok ok t same 1224725857 t ok -2147483648 -3323426 ok ok -1 t 799160348 ok -2147483648 -1 ok -1502566373 ok t new new ok lazy/same ok -2147483648 -2 ok 91299037 new -1866725958 ok ok lazy/same lazy/same new ok evaluated/new -815734944 ok ok -2 lazy/same ok lazy/same 981216411 lazy/same 981216411 ok lazy/same lazy/same same t 981220255 2 ok lazy/same ok lazy/same ok 1880928217 1880928217 same -2 lazy/same ok ok f new -1182909897 -2 f new same ok ok new ok t ok 434599283 lazy/same ok f lazy/same 434599283 lazy/same lazy/same lazy/same -2147483648 lazy/same 434599283 434599283 t ok -2 new -2 t lazy/same new -2147483648 lazy/same ok f f -519846890 -2147483648 -2 lazy/same lazy/same t ok same ok lazy/same lazy/same ok 2 ok new t new ok new 2035842532 -2147483648 evaluated/new -2060413507 t ok ok -2 ok lazy/same -95177828 new 926684254 lazy/same ok f ok 2 lazy/same t f lazy/same 39180573 0 lazy/same 39180573 -2 2 evaluated/new t evaluated/new lazy/same t 1892195450 ok ok 38525052 ok 2 lazy/same ok lazy/same lazy/same t ok lazy/same -1550043205 ok new -1 -1891546885 -2 new ok ok lazy/same t ok 2 lazy/same ok ok lazy/same -2 ok f 132165240 t new -1 926339191 same ok ok -1922370315 -1 ok new ok lazy/same ok t new t lazy/same 250688947 250688947 250688947 ok ok f -1 -2 ok t ok new lazy/same evaluated/new new -2147483648 f f ok ok 143434304 t 143434304 -2147483648 lazy/same t -2 lazy/same ok ok ok ok lazy/same -1 432815867 432815867 -2 t ok ok t ok new ok ok ok ok -2 -1 ok ok same new same same ok lazy/same ok 2 [-2147483648,-1,0,1,7,100,2147483647] ImmutableIntMap(-2147483648 -> -2, -1 -> 1, 0 -> 2, 1 -> 1, 7 -> -1, 100 -> 0, 2147483647 -> -1) 7 -613811561
+ IM-23 random sequence seed 23 -> ok 0 evaluated/new evaluated/new new -2147483648 218 same evaluated/new -150659 f ok ok ok f evaluated/new f -1044781920 -1044781920 t ok -1 same same 1 ok lazy/same -1 ok evaluated/new ok ok lazy/same ok -1525353571 -1525353571 -1525353571 new evaluated/new lazy/same -1 t ok ok new -2147483648 lazy/same lazy/same new new 1454160315 t lazy/same t 1454160315 2 2 ok ok 2 f lazy/same ok t lazy/same lazy/same new lazy/same new f new ok same -1475179075 ok ok -1442731974 -1442731974 -2147483648 ok same ok ok same same ok same f -467993802 ok lazy/same t -2147483648 -467993802 lazy/same -467993802 lazy/same -467993802 2 new ok t 1219625309 evaluated/new ok 1 same f ok new new ok ok same ok ok 126397149 lazy/same new ok new f evaluated/new ok lazy/same new -1133614777 lazy/same -2147483648 -1 ok ok ok t same 1518128864 t ok -2147483648 -1510875235 ok ok -1 t -708391461 ok -2147483648 -1 ok 1439091226 ok t new new ok lazy/same ok -2147483648 -2 ok -1416252772 new -1573322951 ok ok lazy/same lazy/same new ok evaluated/new 2125922655 ok ok -2 lazy/same ok lazy/same -372093286 lazy/same -372093286 ok lazy/same lazy/same same t -372089442 2 ok lazy/same ok lazy/same ok 527618520 527618520 same -2 lazy/same ok ok f new 1604505590 -2 f new same ok ok new ok t ok -1072952526 lazy/same ok f lazy/same -1072952526 lazy/same lazy/same lazy/same -2147483648 lazy/same -1072952526 -1072952526 t ok -2 new -2 t lazy/same new -2147483648 lazy/same ok f f -226443883 -2147483648 -2 lazy/same lazy/same t ok same ok lazy/same lazy/same ok 2 ok new t new ok new -462173405 -2147483648 evaluated/new -1767010500 t ok ok -2 ok lazy/same -1448487525 new -580867555 lazy/same ok f ok 2 lazy/same t f lazy/same -1468371236 0 lazy/same -1468371236 -2 2 evaluated/new t evaluated/new lazy/same t -1558299975 ok ok 882996923 ok 2 lazy/same ok lazy/same lazy/same t ok lazy/same -705571334 ok new -1 1050110714 -2 new ok ok lazy/same t ok 2 lazy/same ok ok lazy/same -2 ok f -1221144457 t new -1 -581212618 same ok ok 1019287284 -1 ok new ok lazy/same ok t new t lazy/same 544091954 544091954 544091954 ok ok f -1 -2 ok t ok new lazy/same evaluated/new new -2147483648 f f ok ok 436837311 t 436837311 -2147483648 lazy/same t -2 lazy/same ok ok ok ok lazy/same -1 -920493830 -920493830 -2 t ok ok t ok new ok ok ok ok -2 -1 ok ok same new same same ok lazy/same ok 2 [-2147483648,-1,0,1,7,100,2147483647] ImmutableIntMap(-2147483648 -> -2, -1 -> 1, 0 -> 2, 1 -> 1, 7 -> -1, 100 -> 0, 2147483647 -> -1) 7 -2121363370
```
