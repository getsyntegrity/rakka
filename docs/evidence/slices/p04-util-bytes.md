# Evidencia del slice P04 (UTIL-BYTES), 2026-11-18

Filas: AC-UTIL-006.1..006.8, AC-UTIL-022 y AC-UTIL-016.3 (ByteIterator, según Q-UTIL-20 recomendada). Decisiones aplicadas: Q-UTIL-12 (c) (newtype sobre `bytes::Bytes`), Q-UTIL-14 (b) (el `hashCode` de Scala no se reproduce), Q-UTIL-8 (c), Q-UTIL-11 (a), Q-X-3 (A), Q-CFG-11, Q-X-17 (entradas `&str`), Q-UTIL-17 (`java_class()` aparte del `Display`). Decisiones nuevas, propuestas en `docs/owner-batch-02.md` (### P04): Q-UTIL-22..Q-UTIL-26. Fuentes de 2.0.0 leídas: `ByteString.scala` (único archivo en 2.0.0, con la nueva `ByteString2`), `scala-2.13/.../ByteIterator.scala`, `UnsynchronizedByteArrayInputStream.java`; los specs `ByteIteratorSpec`, `ByteStringInputStreamSpec` y `UnsynchronizedByteArrayInputStreamSpec`. Sin código de la JDK: el comportamiento de `String`, `Charset`, `Base64`, `ByteBuffer` y `InputStream` sale del oráculo (Q-X-18); el decodificador UTF-8 y el de Base64 se escribieron a partir de los casos medidos y se validaron contra el jar con corridas diferenciales (abajo). No hace falta especificación cleanroom.

## Diseño del oráculo

Casos nuevos en la familia util-core, al final de la lista (uc-0454..uc-0536; los ids de los casos anteriores no se movieron). Son guiones compartidos por el oráculo Java (`BytesOracle.java`) y su gemelo Rust (`crates/rakka-actor/examples/classic-parity/bytes.tsv`, `bytes_cases.rs`): un intérprete por familia (`bytestring`, `builder`, `iterator`, `stream`) que imprime un token por operación, con datos `xHEX` y operaciones `rand SEMILLA N` / `rnd` del generador xorshift común (sin reloj, sin sleeps, sin hilos). Tres ids se miden pero Rust no los produce (lista `PENDING`): BS-APPLY-1C y BS-HEAD-EMPTY (Q-UTIL-23) e IT-CAT (Q-UTIL-24).

```
83 casos nuevos x 12 corridas: oráculo y gemelo, 0 casos con más de una salida distinta (loop de 12 corridas de util-core-all)
oracle: 83 new cases x 12 runs, 0 cases with more than one distinct output
rust twin: 83 new cases x 12 runs, 0 cases with more than one distinct output
```

## ROJO (arnés sin los casos del gemelo registrados)

```
$ python3.12 tools/parity/replay.py
1416/1498 applicable + 1 not applicable        (82 divergentes: uc-0454..uc-0535)
$ cargo test -p rakka-actor --test recorded_pekko
no Rust case for BS-01 take drop dropRight on a compact value with extreme arguments
test recorded_util_core_cases_match_pekko ... FAILED
```

(uc-0536, ST-06, se añadió después de este rojo: readNBytes, skipNBytes y transferTo, que el spec de 2.0.0 afirma.)

## VERDE

```
$ python3.12 tools/parity/replay.py
1499/1499 applicable + 1 not applicable
cargo test --workspace: 457 passed, 0 failed (44 tests nuevos en byte_string/tests.rs: ByteStringSpec, ByteStringBuilderSpec, ByteIteratorSpec, ByteStringInputStreamSpec y UnsynchronizedByteArrayInputStreamSpec portados, con propiedades sobre un xorshift fijo)
cargo clippy --workspace --all-targets -- -D warnings: limpio; cargo fmt --all --check: limpio; cargo deny check licenses: ok (nueva dependencia: bytes 1.12, MIT)
cargo llvm-cov (tools/parity/coverage.sh): 98.73 % de líneas del crate (byte_string/*: 98.8..100 %)
python3.12 tools/package/check.py: failures=0; unittest tools/package/tests: OK
```

El gemelo coincidió con el oráculo a la primera salvo seis divergencias que el propio oráculo corrigió (abajo): el rojo de arriba es el del arnés sin el gemelo, y los mutantes demuestran que los casos detectan los errores probables. Los casos de aserciones de los specs que el gemelo ya cumplía quedan como caracterización.

## Diferencial ad hoc (no grabado)

Una corrida de 1900 entradas aleatorias contra el jar, sin divergencias: 400 sopas de bytes con UTF-8 al borde (`E2 E1 ED 80 A0 BF F0 F4 8F 90 F5 C0 FF`), 400 de bytes uniformes, 600 sopas de texto de Base64, 300 ida y vuelta Base64 y 200 búsquedas de subsecuencias. Las grabadas son BS-31 (23 entradas) y BS-32 (60).

## Hallazgos medidos que el contrato no recogía (2.0.0 frente a 1.7.1)

- `ByteString.scala` es un único archivo en 2.0.0 y añade `ByteString2` (identidad de serialización 3); `ByteIterator.scala` sigue en `scala-2.13`.
- 2.0.0 añade `lastIndexOf`, `lastIndexOfSlice`, `endsWith(Array)`, `startsWith(Array)`, `indexOfSlice(Array)`, `contains(Byte)` y `decodeString(charset, from, until)`; `matchesAt`, `compareBytesTo` y `decodeStringUnchecked` son `private[pekko]`.
- `takeRight(Int.MinValue)` devuelve todo (desborda `length - n`); `grouped(0)` falla con `size=0 must be positive`.
- `getShort/Int/Long` de un iterador de array no consumen nada si faltan bytes (el multi-fragmento consume lo disponible); `getLongPart(n, LITTLE_ENDIAN)` desplaza un Int; `putLongPart` con n < 0 corrompe el builder y con n > 8 desplaza módulo 64.
- `++` entre iteradores de arrays distintos conserva sólo el lado derecho (IT-CAT).
- `UnsynchronizedByteArrayInputStream.skip(n)` avanza `min(n, restante)` (el contrato de 1.7.1 decía n completo y ArithmeticException); añade `readAllBytes`, `readNBytes`, `skipNBytes`, `transferTo`.
- La JVM reemplaza un sustituto codificado (`ED A0..BF xx`) por un solo U+FFFD (BS-19); `Illegal base64 character` imprime el byte con signo en hexadecimal.

## Mutantes nuevos (solo ellos + control)

```
| mutant | outcome | passed/total |
|---|---|---|
| byte-string-take-right-saturates | DETECTED | 1496/1499 |
| utf8-surrogate-replaced-byte-by-byte | DETECTED | 1497/1499 |
| long-part-little-endian-shifts-as-long | DETECTED | 1498/1499 |
| control (unmodified) | CLEAN | 1499/1499 |
--check-patterns: 80 mutant patterns checked, 0 stale
```

## Pendiente

- AC-UTIL-006.1: texto y clase de la excepción de `apply`/`head`/`last` sobre un valor compacto y del vacío (Q-UTIL-23).
- AC-UTIL-006.4: `++` entre iteradores (Q-UTIL-24).
- AC-UTIL-006.6: charsets fuera de UTF-8/US-ASCII/ISO-8859-1 (Q-UTIL-22).
- AC-UTIL-006.8: serialización (Q-UTIL-26; espera AC-SER).
- AC-UTIL-022: ByteBufferCleaner (Q-UTIL-25).
- AC-UTIL-016.3: espera la respuesta a Q-UTIL-20.
