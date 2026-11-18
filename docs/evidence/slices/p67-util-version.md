# Evidencia del slice P67 (UTIL-VERSION), 2026-11-18

Fila: AC-UTIL-005.2. Casos VR-01..VR-23. Decisión aplicada: Q-UTIL-6 rechazada por el Owner (se sigue el comportamiento corregido de 2.0.0, `1.2.0 > 1.2.0-M1`), con la condición de medirlo con el jar 2.0.0 ANTES de implementar. Sin decisiones nuevas.

## 1. Medición con el jar 2.0.0 (antes de escribir Rust)

Oráculo `oracle/src/main/java/pilot/VersionOracle.java` (jar `pekko-actor_2.13` 2.0.0, JDK 17). Salida bruta de cada caso (`oracle/run.sh util-core-case`), registrada antes de existir cualquier código Rust de `Version`.

Resultado respecto de la descripción del Owner y de la lectura de la fuente (`Version.scala:156-176` de 2.0.0): COINCIDEN, sin discrepancia.

- `Version("1.2.0").compareTo(Version("1.2-M1")) = 1` y `Version("1.2-M1").compareTo(Version("1.2.0")) = -1` (VR-05, VR-11): la versión final es MAYOR que la de calificador, en ambos órdenes (en 1.7.1 daba -2 y -1).
- Antisimetría de signo sobre la matriz de 19 versiones (361 pares): 0 rotos (VR-10).
- `equals` = `compareTo == 0` y `hashCode` coincide con `equals` en los 361 pares (VR-14).
- Hallazgos que NO están en el contrato y que el oráculo fija: `"1.2.3-"`, `"1.2.3+"`, `"1.2-"` tienen calificador vacío y son iguales a la versión sin calificador; `"1.2.3."` y `"1."` son válidas (el `split` descarta los vacíos finales) mientras `"."`, `".."` y `"..."` dan `IllegalArgumentException("Only 3 digits...")` (0 segmentos) y `".1"`, `"1..2"` dan `NumberFormatException: For input string: ""`; las diferencias son restas de `Int` que dan la vuelta (VR-17: `2147483647.0.0` frente a `-1.0.0` da -2147483648, es decir, menor, sin antisimetría); el resto del calificador se compara como `String.compareTo` bruto (`M9` frente a `M10` = 8); los dígitos Unicode (Nd) se aceptan (`٢.٥.٥` == `2.5.5`).
- El constructor público `new Version(s)` parsea de forma perezosa (VR-21): sin equivalente en Rust (parseo en la construcción), queda como caso PENDING.

### Determinismo (12 corridas por caso)

```
1 distinct output(s) in 12 runs: VR-01 three digits (spec)
1 distinct output(s) in 12 runs: VR-02 more than three digits
1 distinct output(s) in 12 runs: VR-03 two digits (spec)
1 distinct output(s) in 12 runs: VR-04 one digit (spec)
1 distinct output(s) in 12 runs: VR-05 qualifier (spec)
1 distinct output(s) in 12 runs: VR-06 non digits (spec)
1 distinct output(s) in 12 runs: VR-07 dynver (spec)
1 distinct output(s) in 12 runs: VR-08 no digits (spec)
1 distinct output(s) in 12 runs: VR-09 qualifier matrix (raw compareTo, row vs column)
1 distinct output(s) in 12 runs: VR-10 antisymmetry of the matrix
1 distinct output(s) in 12 runs: VR-11 qualifier on one side only
1 distinct output(s) in 12 runs: VR-12 qualifier text order (raw String.compareTo)
1 distinct output(s) in 12 runs: VR-13 equals and hashCode
1 distinct output(s) in 12 runs: VR-14 hashCode agrees with equals
1 distinct output(s) in 12 runs: VR-15 malformed strings
1 distinct output(s) in 12 runs: VR-16 numbers out of int range
1 distinct output(s) in 12 runs: VR-17 negative and wrapping numbers
1 distinct output(s) in 12 runs: VR-18 digits outside ASCII
1 distinct output(s) in 12 runs: VR-19 equal strings short-circuit
1 distinct output(s) in 12 runs: VR-20 version field and Zero
1 distinct output(s) in 12 runs: VR-21 public constructor parses lazily
1 distinct output(s) in 12 runs: VR-22 parse exceptions
1 distinct output(s) in 12 runs: VR-23 sorting
```

### Salida medida

```
VR-01 three digits (spec) -> ok [1.2.3]/[1.2.3]=0 eq=true [1.2.3]/[1.2.4]=-1 eq=false [1.2.4]/[1.2.3]=1 eq=false [3.2.1]/[1.2.3]=2 eq=false [3.2.1]/[3.3.1]=-1 eq=false [3.2.0]/[3.2.1]=-1 eq=false [1.2.3]/[1.3.3]=-1 eq=false [1.3.3]/[1.2.3]=1 eq=false [10.0.0]/[9.9.9]=1 eq=false [0.0.10]/[0.0.9]=1 eq=false

VR-02 more than three digits -> ok [1.2.3.4]=IllegalArgumentException[Only 3 digits separated with '.' are supported. [1.2.3.4]] [1.2.3.4.5]=IllegalArgumentException[Only 3 digits separated with '.' are supported. [1.2.3.4.5]] [1.2.3.4-M1]=IllegalArgumentException[Only 3 digits separated with '.' are supported. [1.2.3.4-M1]] [1.2.3.]=ok [1.2.3..]=ok [1.2.3.4.]=IllegalArgumentException[Only 3 digits separated with '.' are supported. [1.2.3.4.]] [1.2.3.x]=IllegalArgumentException[Only 3 digits separated with '.' are supported. [1.2.3.x]]

VR-03 two digits (spec) -> ok [1.2]/[1.2.0]=0 eq=true [1.2.0]/[1.2]=0 eq=true [1.2]/[1.3]=-1 eq=false [1.2.1]/[1.2]=1 eq=false [1.2]/[1.2.1]=-1 eq=false [2.4]/[2.3]=1 eq=false [3.2]/[3.2.7]=-7 eq=false [3.2.7]/[3.2]=7 eq=false [1.2]/[1.2]=0 eq=true [1.10]/[1.9]=1 eq=false

VR-04 one digit (spec) -> ok [1]/[1.0]=0 eq=true [1.0]/[1]=0 eq=true [1]/[1.0.0]=0 eq=true [1.0.0]/[1]=0 eq=true [1.0]/[1.0.0]=0 eq=true [3]/[2]=1 eq=false [2b]/[2a]=1 eq=false [2a]/[2b]=-1 eq=false [2020-09-07]/[2020-08-30]=1 eq=false [2020-08-30]/[2020-09-07]=-1 eq=false [2]/[2]=0 eq=true [10]/[9]=1 eq=false [99999999999]/[99999999998]=1 eq=false [99999999999]/[1]=-1 eq=false

VR-05 qualifier (spec) -> ok [1.2.3-M1]/[1.2.3-M1]=0 eq=true [1.2.0-M1]/[1.2-M1]=0 eq=true [1.2-M1]/[1.2.0-M1]=0 eq=true [1.2-M1]/[1.2.0]=-1 eq=false [1.2.0]/[1.2-M1]=1 eq=false [1.2.3-M2]/[1.2.3-M1]=1 eq=false [1.2.3-M1]/[1.2.3-M2]=-1 eq=false [1.2.3-M1]/[1.2.3]=-1 eq=false [1.2.3]/[1.2.3-M1]=1 eq=false

VR-06 non digits (spec) -> ok [1.x.3]=NumberFormatException[For input string: "x"] [1.2x.3]=NumberFormatException[For input string: "2x"] [1.2.x]=NumberFormatException[For input string: "x"] [1.2.3x]=NumberFormatException[For input string: "3x"] [x.3]=NumberFormatException[For input string: "x"] [1.x]=NumberFormatException[For input string: "x"] [1.2x]=NumberFormatException[For input string: "2x"] [x.y.z]=NumberFormatException[For input string: "z"] [1.2.3-x]=ok [1.2.3-1x]=ok

VR-07 dynver (spec) -> ok [1.0.10+3-1234abcd]/[1.0.10+10-1234abcd]=-7 eq=false [1.0.10+10-1234abcd]/[1.0.10+3-1234abcd]=7 eq=false [1.0.10+3-1234abcd]/[1.0.10+3-1234abcd]=0 eq=true [1.0.10+3-1234abcd]/[1.0.10+3-1234abce]=-1 eq=false [1.0.10-3-1234abcd]/[1.0.10-10-1234abcd]=-7 eq=false [1.0.10-10-1234abcd]/[1.0.10+3-1234abcd]=7 eq=false [1.0+3-1234abcd]/[1.0.0+3-1234abcd]=3 eq=false [1.0+3-1234abcd]/[1.0+10-1234abcd]=-7 eq=false [HEAD+20140707-1030]/[HEAD+20140707-1030]=0 eq=true [HEAD+20140707-1030]/[HEAD+20140707-1031]=-1 eq=false [HEAD+20140707-1030]/[1.0.0]=-1 eq=false [1.0.10+3-1234abcd]/[1.0.10]=3 eq=false [1.0.10]/[1.0.10+3-1234abcd]=-3 eq=false [1.0.10-SNAPSHOT]/[1.0.10+3-1234abcd]=-3 eq=false [1.0.10+3-1234abcd]/[1.0.10-SNAPSHOT]=3 eq=false [1.0.10+99999999999-abc]/[1.0.10+3-abc]=-3 eq=false [1.0.10+3]/[1.0.10+3]=0 eq=true [1.0.10+3]/[1.0.10+4]=-1 eq=false [1.0.10+3-]/[1.0.10+3-a]=1 eq=false

VR-08 no digits (spec) -> ok [foo]/[foo]=0 eq=true [foo]/[bar]=4 eq=false [bar]/[foo]=-4 eq=false [foo]/[1.2.3]=-1 eq=false [1.2.3]/[foo]=1 eq=false [foo]/[0.0.0]=-1 eq=false [1-foo]/[01-foo]=1 eq=false [1-foo]/[02-foo]=1 eq=false [02-foo]/[1-foo]=-1 eq=false [2-foo]/[1-foo]=1 eq=false [foo]/[Foo]=32 eq=false [Foo]/[foo]=-32 eq=false [foo]/[foo1]=-1 eq=false [foo1]/[foo]=1 eq=false

VR-09 qualifier matrix (raw compareTo, row vs column) -> ok [1.2]:0,0,1,1,1,1,1,-1,1,-3,-10,-3,0,1,1,2,1,1,1, [1.2.0]:0,0,1,1,1,1,1,-1,1,-3,-10,-3,0,1,1,2,1,1,1, [1.2-M1]:-1,-1,0,0,-1,-5,-5,-1,-6,-3,-10,-3,-1,1,-32,2,1,1,-2, [1.2.0-M1]:-1,-1,0,0,-1,-5,-5,-1,-6,-3,-10,-3,-1,1,-32,2,1,1,-2, [1.2-M2]:-1,-1,1,1,0,-5,-5,-1,-6,-3,-10,-3,-1,1,-32,2,1,1,1, [1.2.0-RC1]:-1,-1,5,5,5,0,0,-1,-1,-3,-10,-3,-1,5,-27,2,1,1,5, [1.2-RC1]:-1,-1,5,5,5,0,0,-1,-1,-3,-10,-3,-1,5,-27,2,1,1,5, [1.2.1]:1,1,1,1,1,1,1,0,1,1,1,1,1,1,1,2,1,1,1, [1.2.0-SNAPSHOT]:-1,-1,6,6,6,1,1,-1,0,-3,-10,-3,-1,6,-26,2,1,1,6, [1.2.0+3-abc]:3,3,3,3,3,3,3,-1,3,0,-7,0,3,3,3,2,1,1,3, [1.2.0+10-abc]:10,10,10,10,10,10,10,-1,10,7,0,7,10,10,10,2,1,1,10, [1.2.0-3-abc]:3,3,3,3,3,3,3,-1,3,0,-7,0,3,3,3,2,1,1,3, [1.2.0-]:0,0,1,1,1,1,1,-1,1,-3,-10,-3,0,1,1,2,1,1,1, [1.2.0-M]:-1,-1,-1,-1,-1,-5,-5,-1,-6,-3,-10,-3,-1,0,-32,2,1,1,-3, [1.2.0-m1]:-1,-1,32,32,32,27,27,-1,26,-3,-10,-3,-1,32,0,2,1,1,32, [1]:-2,-2,-2,-2,-2,-2,-2,-2,-2,-2,-2,-2,-2,-2,-2,0,1,1,-2, [1-M1]:-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,0,-53,-1, [foo]:-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,53,0,-1, [1.2.0-M1-x]:-1,-1,2,2,-1,-5,-5,-1,-6,-3,-10,-3,-1,3,-32,2,1,1,0,

VR-10 antisymmetry of the matrix -> ok pairs=361 broken=0 

VR-11 qualifier on one side only -> ok [1.2.0]/[1.2.0-M1]=1 eq=false [1.2.0-M1]/[1.2.0]=-1 eq=false [1.2.0]/[1.2-M1]=1 eq=false [1.2-M1]/[1.2.0]=-1 eq=false [1.2.0]/[1.2.0-RC1]=1 eq=false [1.2.0-RC1]/[1.2.0]=-1 eq=false [1.2.0]/[1.2.0-SNAPSHOT]=1 eq=false [1.2.0-SNAPSHOT]/[1.2.0]=-1 eq=false [1.2]/[1.2-M1]=1 eq=false [1.2-M1]/[1.2]=-1 eq=false [1]/[1-M1]=1 eq=false [1-M1]/[1]=-1 eq=false [1.2.3]/[1.2.3-M1]=1 eq=false [1.2.3-M1]/[1.2.3]=-1 eq=false [1.2.3]/[1.2.3-]=0 eq=true [1.2.3-]/[1.2.3]=0 eq=true [2.0.0]/[1.9.9-M1]=1 eq=false [1.9.9-M1]/[2.0.0]=-1 eq=false [1.2.0-M1]/[1.2.1]=-1 eq=false [1.2.1]/[1.2.0-M1]=1 eq=false

VR-12 qualifier text order (raw String.compareTo) -> ok [1.2.0-M1]/[1.2.0-M2]=-1 eq=false [1.2.0-M2]/[1.2.0-M1]=1 eq=false [1.2.0-M1]/[1.2.0-M10]=-1 eq=false [1.2.0-M10]/[1.2.0-M1]=1 eq=false [1.2.0-M9]/[1.2.0-M10]=8 eq=false [1.2.0-M1]/[1.2.0-RC1]=-5 eq=false [1.2.0-RC1]/[1.2.0-M1]=5 eq=false [1.2.0-a]/[1.2.0-B]=31 eq=false [1.2.0-B]/[1.2.0-a]=-31 eq=false [1.2.0-abc]/[1.2.0-abd]=-1 eq=false [1.2.0-ab]/[1.2.0-abcd]=-2 eq=false [1.2.0-abcd]/[1.2.0-ab]=2 eq=false [1.2.0-\u00E9]/[1.2.0-e]=132 eq=false [1.2.0-\uD83D\uDE00]/[1.2.0-\uFFFF]=-10178 eq=false [1.2.0-\uFFFF]/[1.2.0-\uD83D\uDE00]=10178 eq=false [1.2.0-M1]/[1.2.0-m1]=-32 eq=false

VR-13 equals and hashCode -> ok [1.2]=1596886478 [1.2.0]=1596886478 [1]=1596785172 [1.0]=1596785172 [1.0.0]=1596785172 [1.2-M1]=1596888914 [1.2.0-M1]=1596888914 [1.2.3]=1596890585 [foo]=1595012585 [bar]=1595008310 [1.0.10+3-abc]=1596895327 [1.0.10+10-abc]=1596895586 [0.0.0]=1594911011 [0]=1594911011 [x]=1594911131 zero=1594911011 0.0.0

VR-14 hashCode agrees with equals -> ok pairs=361 equal-but-different-hash=0 

VR-15 malformed strings -> ok []=IllegalArgumentException[Empty version not supported.] [.]=IllegalArgumentException[Only 3 digits separated with '.' are supported. [.]] [..]=IllegalArgumentException[Only 3 digits separated with '.' are supported. [..]] [...]=IllegalArgumentException[Only 3 digits separated with '.' are supported. [...]] [1.]=ok [1..]=ok [.1]=NumberFormatException[For input string: ""] [..1]=NumberFormatException[For input string: ""] [1..2]=NumberFormatException[For input string: ""] [.1.2]=NumberFormatException[For input string: ""] [1.2.]=ok [ 1.2]=NumberFormatException[For input string: " 1"] [1.2 ]=NumberFormatException[For input string: "2 "] [1. 2]=NumberFormatException[For input string: " 2"] [-1]=ok [+1]=ok [1.-2]=NumberFormatException[For input string: ""] [1.+2]=NumberFormatException[For input string: ""] [-1.2]=ok [-1.2.3]=ok [1.-2.3]=ok [1.2.-3]=NumberFormatException[For input string: ""] [1.2.+3]=NumberFormatException[For input string: ""] [1.2.3-]=ok [1.2.3+]=ok [1.2.3-+]=ok [1.2.3+-]=ok [1.2-]=ok [1.2+]=ok [1-]=ok [1+]=ok [1.]=ok [a]=ok [a.b]=NumberFormatException[For input string: "b"] [a.b.c]=NumberFormatException[For input string: "c"] [1.2.3-M1+5]=ok [1.2.3+5-M1]=ok [1.2.3+5-]=ok [1.2.3+-5-x]=ok [1.2.3--5-x]=ok [1.2.3-5]=ok [1.2.3-5-]=ok [1.2.3-5x-y]=ok [1.2.3-x-5]=ok [v1.2.3]=NumberFormatException[For input string: "v1"] [1.2.3\u000A]=NumberFormatException[For input string: "3\u000A"] [\u000A]=ok [ ]=ok [1.2.3 ]=NumberFormatException[For input string: "3 "]

VR-16 numbers out of int range -> ok [2147483647.0.0]/[1.0.0]=2147483646 eq=false [2147483648.0.0]/[1.0.0]=NumberFormatException[For input string: "2147483648"] eq=- [1.2147483648.0]/[1.0.0]=NumberFormatException[For input string: "2147483648"] eq=- [1.0.2147483648]/[1.0.0]=NumberFormatException[For input string: "2147483648"] eq=- [1.2.3-2147483648-x]/[1.2.3]=-1 eq=false [2147483647]/[2147483646]=1 eq=false [2147483648]/[2147483647]=-2147483647 eq=false [2147483648]/[2147483649]=-1 eq=false [99999999999]/[99999999999]=0 eq=true [99999999999.1.1]/[1.1.1]=NumberFormatException[For input string: "99999999999"] eq=- [1.2.99999999999-M1]/[1.2.1]=NumberFormatException[For input string: "99999999999"] eq=- [1.99999999999-M1]/[1.2]=NumberFormatException[For input string: "99999999999"] eq=- [1.2+2147483648-x]/[1.2]=-1 eq=false

VR-17 negative and wrapping numbers -> ok [-1.2.3]/[1.2.3]=-2 eq=false [1.2.3]/[-1.2.3]=2 eq=false [-1.2.3]/[-1.2.3]=0 eq=true [-1.2.3]/[-2.2.3]=1 eq=false [2147483647.0.0]/[-1.0.0]=-2147483648 eq=false [-1.0.0]/[2147483647.0.0]=-2147483648 eq=false [-2147483648.0.0]/[2147483647.0.0]=1 eq=false [2147483647.0.0]/[-2147483648.0.0]=-1 eq=false [1.2147483647.0]/[1.-1.0]=-2147483648 eq=false [1.-1.0]/[1.2147483647.0]=-2147483648 eq=false [1.2.-3]/[1.2.3]=NumberFormatException[For input string: ""] eq=- [1.2.3]/[1.2.-3]=NumberFormatException[For input string: ""] eq=- [1.2.-2147483648]/[1.2.2147483647]=NumberFormatException[For input string: ""] eq=- [1.2.2147483647]/[1.2.-2147483648]=NumberFormatException[For input string: ""] eq=-

VR-18 digits outside ASCII -> ok [\u0662.\u0665.\u0665]/[2.5.5]=0 eq=true [2.5.5]/[\u0662.\u0665.\u0665]=0 eq=true [2.5.\u0665]/[2.5.5]=0 eq=true [\uFF12.\uFF15.\uFF15]/[2.5.5]=0 eq=true [\u0662]/[2]=0 eq=true [2]/[\u0662]=0 eq=true [\u0662b]/[2b]=1584 eq=false [\u0662b]/[\u0663b]=-1 eq=false [1.2.3-M\u0661]/[1.2.3-M1]=1584 eq=false [1.\u0662]/[1.2]=0 eq=true [1.\u0662-M1]/[1.2-M1]=0 eq=true

VR-19 equal strings short-circuit -> ok [1.2.3]/[1.2.3]=0 eq=true [foo]/[foo]=0 eq=true [1.2.3-M1]/[1.2.3-M1]=0 eq=true [99999999999]/[99999999999]=0 eq=true [1.2.0+3-abc]/[1.2.0+3-abc]=0 eq=true [-1.2.3]/[-1.2.3]=0 eq=true

VR-20 version field and Zero -> ok [1.2]=[1.2] [1.2.0-M1]=[1.2.0-M1] [foo]=[foo] [2020-09-07]=[2020-09-07] [1.0.10+3-abc]=[1.0.10+3-abc] zero=[0.0.0] 1.2>zero=1 zero>foo=1 0=zero:0

VR-21 public constructor parses lazily -> ok [1.x] same=0 other=NumberFormatException[For input string: "x"] hash=NumberFormatException [1.2.3.4] same=0 other=IllegalArgumentException[Only 3 digits separated with '.' are supported. [1.2.3.4]] hash=IllegalArgumentException [] same=0 other=IllegalArgumentException[Empty version not supported.] hash=IllegalArgumentException [1.2.3] same=0 other=1 hash=1596890585

VR-22 parse exceptions -> ok []=java.lang.IllegalArgumentException cause=null msg=[Empty version not supported.] [1.2.3.4]=java.lang.IllegalArgumentException cause=null msg=[Only 3 digits separated with '.' are supported. [1.2.3.4]] [1.x.3]=java.lang.NumberFormatException cause=null msg=[For input string: "x"] [1.2x]=java.lang.NumberFormatException cause=null msg=[For input string: "2x"] [x.3]=java.lang.NumberFormatException cause=null msg=[For input string: "x"] [1.2.3x]=java.lang.NumberFormatException cause=null msg=[For input string: "3x"] [1.]=ok [.]=java.lang.IllegalArgumentException cause=null msg=[Only 3 digits separated with '.' are supported. [.]] [1.2.3.4.5]=java.lang.IllegalArgumentException cause=null msg=[Only 3 digits separated with '.' are supported. [1.2.3.4.5]] [99999999999.1]=java.lang.NumberFormatException cause=null msg=[For input string: "99999999999"] [1.99999999999.1]=java.lang.NumberFormatException cause=null msg=[For input string: "99999999999"]

VR-23 sorting -> ok 1.1.9 1.2.0-M1 1.2-M1 1.2.0-M2 1.2.0-RC1 1.2.0 1.2 1.2.0+3-abc 1.2.0+10-abc 1.2.1 1.9.0 1.10.0 max=1.10.0 min=1.1.9

```

## 2. ROJO (Version con el esqueleto: parse devuelve Ok sin partir, pekko_compare = 0, hash_code = 0)

Los casos se registraron con `rust-anything app record` (22 casos, uc-0148..uc-0169) y la salida del oráculo ya estaba medida (sección 1) antes de existir el código Rust.

```
1111/1132 applicable + 1 not applicable
DIVERGES: uc-0148 ... uc-0165, uc-0167, uc-0168, uc-0169   (21 casos)
```

uc-0166 (VR-19, cadenas iguales) pasaba con el esqueleto: caracterización.

## 3. VERDE

Primera implementación: 1130/1132 (VR-10 y VR-14 divergían solo por el espacio final que deja el oráculo cuando la lista de pares rotos está vacía; corregido en el gemelo Rust del caso, no en `Version`). Después:

```
1132/1132 applicable + 1 not applicable
```

Pruebas unitarias nuevas: `util::version::tests` (7) y `actor::jstring::tests` (2).

## 4. Mutantes nuevos (solo ellos + control)

```
| mutant | outcome | passed/total |
| version-final-below-qualified (defecto de 1.7.1) | DETECTED | 1126/1132 |
| version-dynver-count-ignored | DETECTED | 1128/1132 |
| version-difference-saturates | DETECTED | 1131/1132 |
| control (unmodified) | CLEAN | 1132/1132 |
57 mutant patterns checked, 0 stale
```

## 5. Gates

```
replay: 1132/1132 applicable + 1 not applicable
cargo test --workspace: 26 binarios de pruebas, 0 fallos
cargo clippy --workspace --all-targets -- -D warnings: limpio
cargo fmt --all --check: ok
cargo deny check licenses: ok
cobertura: util/version.rs 99.64 % regiones / 99.44 % líneas; TOTAL 97.82 % regiones, 97.70 % líneas
tools/package/check.py: failures=0, citation_problems=0
unittest tools/package/tests: 35 OK
```
