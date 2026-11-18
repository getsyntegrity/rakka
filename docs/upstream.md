# Referencia upstream y obligaciones de licencia

## Qué versión de Pekko es la fuente de verdad

El comportamiento a reproducir es el de **Apache Pekko 2.0.0** (decisión Q-X-11, aceptada por el Owner el 2026-11-18; sustituye a 1.7.1). Está fijado como submódulo git en `upstream/pekko`:

| Dato | Valor |
|---|---|
| Repositorio | https://github.com/apache/pekko |
| Tag | `v2.0.0` (objeto tag anotado `879512455865e4a81433c8c96fd11c2b95c5bd7d`) |
| Commit | `9fbd2283b1cf701c90a1223bf621942c191db61f` |
| Fecha del commit | 2026-11-02 |
| Scala | 2.13.18 (`project/Dependencies.scala:45`; también 3.3.9 en `:46`) |
| Config (`com.typesafe:config`) | 1.4.9 (`project/Dependencies.scala:63`) |
| JDK mínima | 17 (`project/JdkOptions.scala:26`, `targetJavaVersion = "17"`) |

El commit se verificó con `git ls-remote https://github.com/apache/pekko 'refs/tags/v2.0.0*'` el 2026-11-18 (el objeto tag es `879512455865...`, el commit al que apunta `v2.0.0^{}` es `9fbd2283b1cf...`). La referencia anterior (v1.7.1, commit `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`) queda sólo como historia.

Para recuperarlo en un clon nuevo: `git submodule update --init upstream/pekko`, y comprobar con `git -C upstream/pekko rev-parse HEAD`.

**Regla:** nada bajo `upstream/` se edita. El código propio vive en `crates/`, la documentación propia en `docs/`, las herramientas en `tools/`.

## Oráculo: Pekko 2.0.0 en la JVM

El oráculo oficial (Q-X-12) es el CLI Java del piloto (`oracle/`, JDK 17, `oracle/run.sh`) junto con `rust-anything app record/replay`. El oráculo anterior con `jshell` y goldens (`tools/oracle/`, jars 1.7.1) queda deprecado; sus escenarios se convierten a casos `rust-anything`. Los artefactos de Maven Central contra los que se compila el oráculo:

| Jar | SHA-1 |
|---|---|
| `pekko-actor_2.13-2.0.0.jar` | `873748b67f33d949350d57f00e3d698282a65c7f` |
| `pekko-actor-typed_2.13-2.0.0.jar` | `7bad37818b741144d0d8db403bc3347a06dcf02d` |
| `scala-library-2.13.18.jar` | `eb2086988c25b08705472cc750654570900362e5` |
| `config-1.4.9.jar` | `fc87584fea1adcf5df899e58501a470329e59b52` |

SHA-1 leídos de Maven Central el 2026-11-18. Que el jar publicado corresponda exactamente al commit fijado es una suposición razonable (mismo tag), no verificada byte a byte.

## Obligaciones de licencia

Pekko es Apache License 2.0. Un port que deriva comportamiento, estructura y documentación de él es una obra derivada, así que:

1. **El proyecto se distribuye bajo Apache-2.0** (`LICENSE`, copia de `upstream/pekko/legal/StandardLicense.txt`; `license = "Apache-2.0"` en `Cargo.toml`). Es la elección natural, pero es una decisión del Owner: otra licencia compatible sería posible siempre que se respeten los puntos 2-4.
2. **`NOTICE` reproduce el NOTICE de los artefactos de Pekko** que portamos (`upstream/pekko/legal/pekko-actor-jar-notice.txt`), precedido por la atribución del port (sección 4(d) de la licencia).
3. **Los archivos Rust que traduzcan un archivo Pekko concreto** llevan un encabezado que lo nombre, por ejemplo: `// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorPath.scala (Apache-2.0).` Esto cumple la sección 4(b) (indicar archivos modificados).
4. **Código de terceros dentro de pekko-actor.** Algunos archivos de Pekko son a su vez derivados de otros proyectos (ver `upstream/pekko/LICENSE` desde la línea 203). Si portamos alguno, hay que conservar su aviso:

| Si se porta... | Origen | Obligación |
|---|---|---|
| `routing/MurmurHash.scala` | MurmurHash3 (dominio público) + Scala | mantener aviso EPFL/Lightbend en NOTICE (ya incluido) |
| `util/ccompat/*`, `util/Helpers.scala` | scala-library / scala-collection-compat | ya cubierto en NOTICE |
| `io/dns/DnsSettings.scala` | Netty | ya cubierto en NOTICE |
| `util/UUIDComparator.scala` | java-uuid-generator | ya cubierto en NOTICE |
| `dispatch/AbstractNodeQueue.java`, `AbstractBoundedNodeQueue.java` | cola MPSC de Dmitry Vyukov, **BSD simplificada** | copiar el texto BSD completo de `upstream/pekko/LICENSE` a `LICENSE` del proyecto (o a `LICENSES/`) en el mismo commit que porte el algoritmo |

Si se usa una cola MPSC de un crate externo en lugar de portar la de Pekko, la fila BSD no aplica, pero sí la licencia de ese crate.
