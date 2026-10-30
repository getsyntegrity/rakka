# Evaluación de REA para el port de Pekko a Rust

Fecha: 2026-10-30. REA evaluado: repo `morluto/rea`, commit
`e5c46dff034fef432e772f669bc309f512a6b186` (paquete npm `rea-agents@6.8.0`, el
mismo número de versión que declara el `package.json` de ese commit). Pekko de
referencia: `upstream/pekko`, v1.7.1, commit `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`.

## Veredicto

**No adoptarlo para el port.** Como mucho sirve para algo puntual y opcional:
comparar la salida de terminal de un escenario corrido en Pekko (JVM) contra el
mismo escenario corrido en pekko-rs (`capture_process_scenario` +
`compare_process_captures`). Ese uso no depende de nada específico de Java/Scala,
y un test de integración propio hace lo mismo con menos instalación. REA no lee
bytecode de la JVM, no se conecta a una JVM en ejecución y no entiende Scala. Para
este proyecto, la fuente de verdad sigue siendo el código Scala de
`upstream/pekko` más sus tests.

## Qué es REA (según su documentación)

REA ("Reverse Engineer Anything") es un servidor MCP (Model Context Protocol, el
protocolo con el que un agente como Claude Code llama herramientas externas)
escrito en TypeScript que también trae una CLI (`rea`). Junta herramientas de
ingeniería inversa: binarios nativos (con Hopper, Ghidra, IDA o Kuna como
decompiladores externos), apps JavaScript/Electron, ensamblados .NET, APKs de
Android (decompilados con JADX), sitios web vía navegador, capturas de red,
WASM, BEAM de Erlang, etc. Todo devuelve "Evidence": JSON con digests,
cobertura, limitaciones y cosas desconocidas explícitas. El servidor expone
229 herramientas (`tools/list` contra 6.8.0).

No es un conversor Scala→Rust ni un extractor de especificaciones. Ni el README
ni la carpeta `docs/` mencionan Scala, archivos `.class` sueltos, JARs como
formato propio, JDWP (el protocolo de depuración de la JVM) ni herramientas como
`jstack`/`jcmd`. Lo único "JVM" que soporta es Android (DEX vía JADX) y JEB.

## Capacidades documentadas, clasificadas para este proyecto

| Capacidad | Clasificación | Motivo |
|---|---|---|
| Inventario de archivos/paquetes (`open_binary` + `inspect_artifact`) | Marginal | Lista las entradas de un ZIP/JAR con sus hashes. Para un JAR da la lista de `.class` y nada más; `unzip -l` da lo mismo. |
| Captura de procesos (`capture_process_scenario`, `compare_process_captures`) | Marginal (lo único con algo de valor) | Corre un comando bajo una PTY (terminal virtual), registra la salida, el código de salida y archivos elegidos, y compara dos corridas. Sirve para hacer pruebas diferenciales JVM vs Rust de la salida observable. Tiene una limitación clave para actores: el orden de los mensajes entre actores no es determinista, y hay que escribir a mano una "trace spec" con el orden parcial que se acepta. |
| Decompilación nativa (Hopper/Ghidra/IDA/Kuna, objdump, Rizin, GDB) | No aplica | Pekko es bytecode JVM y tenemos el código fuente. Además requiere decompiladores instalados. |
| Android/JADX/JEB, apktool, Flutter | No aplica | Sirve para APK/DEX, no para JARs de Scala. Necesitaría un decompilador. |
| Frida (instrumentación dinámica) | No aplica | Engancha código nativo; para la JVM sería ruido de bajo nivel. Además es una herramienta nativa externa. |
| JavaScript/Electron, navegador, HAR, WebMCP, entornos RL | No aplica | Otro ecosistema. |
| .NET, Go, Rust (cargo-auditable), ELF/PE, WASM, BEAM, GPU, firmware, pcap, SQLite, etc. | No aplica | Formatos que no tienen relación con Pekko. Lo de Rust solo lee metadatos de dependencias embebidas en un binario. |
| Ledger de obligaciones / cobertura de reconstrucción | Marginal | Es un formato para seguir "qué reconstruiste y con qué prueba", pero se alimenta de Evidence de REA. Para nosotros, la trazabilidad contra los tests de Pekko es más directa. |

## Qué probé

Una sola operación pertinente: inspeccionar el JAR publicado
`pekko-actor_2.13-1.7.1.jar` desde el servidor MCP por stdio (entrada/salida
estándar), sin registrarlo en ninguna configuración de agente.

El setup fue todo local, en el scratchpad de la sesión:

- La Node del sistema (v24.17.0) no cumple el `engines` de REA
  (`^22.23.4 || ^24.22.1 || >=26.12.1`) y npm tiene `engine-strict=true`. Bajé
  Node v24.22.2 de nodejs.org (verificada con `SHASUMS256.txt`) y la usé solo en
  esa carpeta.
- Instalé `rea-agents@6.8.0` con `--ignore-scripts`, para que no corriera ningún
  script de instalación (por ejemplo el de `frida`, que baja un binario nativo).
  No ejecuté `install.sh` ni `npx rea-agents setup`, que tocan la configuración
  de los agentes.
- El JAR no estaba en `~/.m2`, `~/.ivy2` ni `~/.cache/coursier`; lo bajé de
  Maven Central y verifiqué su SHA-1. SHA-256:
  `454c101239b1106e2cacf0b705dd645a9f9196f0c448099305748852789d26a9`.
- Redirigí `XDG_CACHE_HOME`, `XDG_STATE_HOME`, `XDG_DATA_HOME` y
  `XDG_CONFIG_HOME` al scratchpad para que REA no dejara estado en `~`.

### Qué observó

`open_binary` reconoció el archivo como `format: "zip"`, `kind: "archive"`,
`architecture: null`. `inspect_artifact` (en modo `detail: "summary"`) devolvió,
en unos 1,4 s:

- 1930 nodos (1929 entradas del ZIP más la raíz), 1900 archivos y 30
  directorios. Cobertura `complete`. Ninguna contradicción de integridad.
- **0 hipótesis, 0 ramas sin explorar, 0 próximas pruebas sugeridas.**
- Limitaciones que declara la propia herramienta: *"does not execute the
  artifact or claim format-specific semantics"* e *"Inventory does not establish
  application runtime behavior"*.

Para comparar, `unzip -l` da 1929 entradas, 1894 de ellas `.class`. O sea que
REA trató el JAR como un ZIP genérico: no leyó clases, métodos, firmas, el
`reference.conf` ni nada sobre actores. No obtuvo ninguna evidencia de
comportamiento de Pekko.

### Qué no pudo hacer, o no intenté

- No hay ninguna herramienta para bytecode JVM o Scala. Busqué en las
  descripciones de las 229 herramientas los términos JVM, `.class`, bytecode,
  jar, JDWP, jstack y jcmd: solo aparece `inspect_android_package`.
- No probé `capture_process_scenario` sobre un programa Pekko. Para correr
  Pekko hacen falta, además del JAR, `scala-library` 2.13 y
  `com.typesafe:config`, y sbt no está instalado. Lo dejé fuera del alcance de
  esta prueba. Esa captura solo vería la salida de terminal y los archivos, no
  los mensajes entre actores.

## Cómo encajaría en el flujo de trabajo

Si en algún momento queremos pruebas diferenciales de punta a punta, el uso
razonable sería este:

1. Escribir el mismo escenario dos veces: un `main` Scala/Java con Pekko y un
   binario Rust con pekko-rs, los dos imprimiendo eventos (por ejemplo
   `started`, `received X`, `stopped`).
2. Correr cada uno con `rea capture-process` y comparar los resultados con
   `rea compare-process-captures a.json b.json trace-spec.json`.

Una diferencia que encuentre REA es una pista, no una conclusión. Cada
diferencia hay que confirmarla contra el código de `upstream/pekko`
(`actor/src/main/scala/...`) y contra su test equivalente en
`actor-tests`/`actor-typed-tests`, porque:

- El orden entre actores distintos no está garantizado; Pekko solo garantiza el
  orden por par emisor→receptor.
- Que la salida coincida no prueba que la semántica sea la misma: supervisión,
  dead letters y la ruta de `PoisonPill`/`stop` pueden no imprimir nada.

Rechacé la alternativa de adoptar REA como herramienta estándar del agente.
Instalarlo agrega un MCP de 229 herramientas (el `tools/list` pesa unos 6 MB de
JSON con los esquemas completos), exige una Node más nueva que la del sistema y
no aporta nada sobre la JVM. Para lo único que suma, alcanza con un test de
integración que compare la salida de las dos implementaciones.

## Comandos para reproducir

```sh
S=/tmp/rea-eval   # cualquier carpeta vacía fuera del proyecto
mkdir -p $S/{node,rea-install,jar,out,xdg/{cache,state,data,config}}

# REA a un commit fijo (solo para leer docs)
git clone https://github.com/morluto/rea $S/rea
git -C $S/rea checkout e5c46dff034fef432e772f669bc309f512a6b186

# Node local que cumple el engines de REA
cd $S/node
curl -fLO https://nodejs.org/dist/v24.22.2/node-v24.22.2-linux-x64.tar.xz
curl -fLO https://nodejs.org/dist/v24.22.2/SHASUMS256.txt
grep ' node-v24.22.2-linux-x64.tar.xz$' SHASUMS256.txt | sha256sum -c -
tar xJf node-v24.22.2-linux-x64.tar.xz
export PATH=$S/node/node-v24.22.2-linux-x64/bin:$PATH
export XDG_CACHE_HOME=$S/xdg/cache XDG_STATE_HOME=$S/xdg/state \
       XDG_DATA_HOME=$S/xdg/data XDG_CONFIG_HOME=$S/xdg/config

# REA sin scripts de instalación
cd $S/rea-install && echo '{"private":true}' > package.json
npm install --ignore-scripts --no-audit --no-fund rea-agents@6.8.0

# JAR de Pekko
cd $S/jar
U=https://repo1.maven.org/maven2/org/apache/pekko/pekko-actor_2.13/1.7.1/pekko-actor_2.13-1.7.1.jar
curl -fLO $U && echo "$(curl -fsL $U.sha1)  pekko-actor_2.13-1.7.1.jar" | sha1sum -c -
```

La prueba se hizo por MCP stdio con un script Python chico (`mcp_drive.py`). El
script manda `initialize` y `notifications/initialized`, después `tools/list`,
y después estas llamadas en la misma sesión:

```json
[["open_binary", {"path": "<S>/jar/pekko-actor_2.13-1.7.1.jar"}],
 ["inspect_artifact", {"detail": "summary"}]]
```

El servidor se lanza con `node $S/rea-install/node_modules/rea-agents/scripts/rea.mjs mcp`.
Desde la CLI, el equivalente aproximado es
`$S/rea-install/node_modules/.bin/rea inspect-artifact <jar> --json`.

## Nota posterior: qué usamos en su lugar

La comparación "mismo escenario en Pekko y en pekko-rs" se resolvió sin REA ni sbt: `tools/oracle/run.sh` corre un script `jshell` (Java 21, ya instalado) contra los jars oficiales de Pekko 1.7.1, `scala-library` 2.13.18 y `config` 1.4.9, con SHA-1 verificados (ver `docs/upstream.md`). La primera corrida (`tools/oracle/path-address.jsh`, ~3 s) produjo `crates/pekko-actor/tests/oracle/path-address.golden` y mostró casos borde que dependen de `java.net.URI` y no se leen fácilmente en el código Scala (por ejemplo, `pekko://@host:1` se acepta con `system` vacío y `pekko://s@h:70000` se acepta). Ese golden lo compara un test Rust, sin capturas de terminal ni dependencia de REA. Para escenarios concurrentes, el plan es el mismo: el script `jshell` imprime eventos y el test Rust compara contra un conjunto de órdenes permitidos, no contra una única secuencia.
