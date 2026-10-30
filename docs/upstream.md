# Referencia upstream y obligaciones de licencia

## Qué versión de Pekko es la fuente de verdad

El comportamiento a reproducir es el de **Apache Pekko 1.7.1**. Está fijado como submódulo git en `upstream/pekko`:

| Dato | Valor |
|---|---|
| Repositorio | https://github.com/apache/pekko |
| Tag | `v1.7.1` (objeto tag anotado `305306adecddd2b0ac5990c53dad30f47c3b250f`) |
| Commit | `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` |

El commit se verificó contra GitHub con `git ls-remote https://github.com/apache/pekko 'refs/tags/v1.7.1*'` el 2026-10-30. El submódulo se clonó con `--reference` a una copia local ya existente para no bajar todo el historial, y después se reempaquetó (`git repack -a -d`) y se quitó el archivo `alternates`, así que no depende de esa copia.

Para recuperarlo en un clon nuevo: `git submodule update --init upstream/pekko`, y comprobar con `git -C upstream/pekko rev-parse HEAD`.

**Regla:** nada bajo `upstream/` se edita. El código propio vive en `crates/`, la documentación propia en `docs/`, las herramientas en `tools/`.

## Oráculo: los jars oficiales

Además del código fuente, se usan los artefactos publicados de 1.7.1 para comparar comportamiento (ver `tools/oracle/`). `tools/oracle/fetch-jars.sh` los baja de Maven Central a `.oracle/jars/` (ignorado por git) y verifica cada uno contra el SHA-1 publicado y contra el fijado en el script:

| Jar | SHA-1 |
|---|---|
| `pekko-actor_2.13-1.7.1.jar` | `6c23c1bd418399622e365ab0f4b53b32b7ca4e5d` |
| `pekko-actor-typed_2.13-1.7.1.jar` | `2a7bd88664707aab6e405e8ca2ad3957ed21cdb9` |
| `scala-library-2.13.18.jar` | `eb2086988c25b08705472cc750654570900362e5` |
| `config-1.4.9.jar` | `fc87584fea1adcf5df899e58501a470329e59b52` |
| `slf4j-api-2.0.17.jar` | `d9e58ac9c7779ba3bf8142aff6c830617a7fe60f` |

Las versiones de Scala (2.13.18) y Config (1.4.9) son las de `upstream/pekko/project/Dependencies.scala:47,66`. Que el jar publicado corresponda exactamente al commit fijado es una suposición razonable (es el release oficial) pero no está verificada byte a byte; si un resultado del oráculo contradice la fuente, gana la fuente y se registra la discrepancia.

## Obligaciones de licencia

Pekko es Apache License 2.0. Un port que deriva comportamiento, estructura y documentación de él es una obra derivada, así que:

1. **El proyecto se distribuye bajo Apache-2.0** (`LICENSE`, copia de `upstream/pekko/legal/StandardLicense.txt`; `license = "Apache-2.0"` en `Cargo.toml`). Es la elección natural, pero es una decisión del Owner: otra licencia compatible sería posible siempre que se respeten los puntos 2-4.
2. **`NOTICE` reproduce el NOTICE de los artefactos de Pekko** que portamos (`upstream/pekko/legal/pekko-actor-jar-notice.txt`), precedido por la atribución del port (sección 4(d) de la licencia).
3. **Los archivos Rust que traduzcan un archivo Pekko concreto** llevan un encabezado que lo nombre, por ejemplo: `// Derived from Apache Pekko 1.7.1 actor/src/main/scala/org/apache/pekko/actor/ActorPath.scala (Apache-2.0).` Esto cumple la sección 4(b) (indicar archivos modificados).
4. **Código de terceros dentro de pekko-actor.** Algunos archivos de Pekko son a su vez derivados de otros proyectos (ver `upstream/pekko/LICENSE` desde la línea 203). Si portamos alguno, hay que conservar su aviso:

| Si se porta... | Origen | Obligación |
|---|---|---|
| `routing/MurmurHash.scala` | MurmurHash3 (dominio público) + Scala | mantener aviso EPFL/Lightbend en NOTICE (ya incluido) |
| `util/ccompat/*`, `util/Helpers.scala` | scala-library / scala-collection-compat | ya cubierto en NOTICE |
| `io/dns/DnsSettings.scala` | Netty | ya cubierto en NOTICE |
| `util/UUIDComparator.scala` | java-uuid-generator | ya cubierto en NOTICE |
| `dispatch/AbstractNodeQueue.java`, `AbstractBoundedNodeQueue.java` | cola MPSC de Dmitry Vyukov, **BSD simplificada** | copiar el texto BSD completo de `upstream/pekko/LICENSE` a `LICENSE` del proyecto (o a `LICENSES/`) en el mismo commit que porte el algoritmo |

Si se usa una cola MPSC de un crate externo en lugar de portar la de Pekko, la fila BSD no aplica, pero sí la licencia de ese crate.
