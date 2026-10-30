# Spec 24 — AC-IO (1/2)

**Estado:** pendiente de decisiones propuestas. **Filas:** 30. **Tareas:** 5 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-classic.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 15 `AC-SYS-1` (`15-ac-sys-1.md`), por las filas AC-SYS-001.1, AC-SYS-001.2.
- Spec 18 `AC-EXT-1` (`18-ac-ext-1.md`), por las filas AC-EXT-001.1, AC-EXT-001.2.
- Spec 25 `AC-IO-2` (`25-ac-io-2.md`), por las filas AC-IO-010, AC-IO-011. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 05 `AC-CFG-1` (`05-ac-cfg-1.md`), por las filas AC-CFG-006, AC-CFG-016.
- Spec 09 `AC-DSP-1` (`09-ac-dsp-1.md`), por las filas AC-DSP-011.
- Spec 16 `AC-SUP-1` (`16-ac-sup-1.md`), por las filas AC-SUP-004.

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-IO-1`, `Q-IO-2`, `Q-IO-3`, `Q-IO-4`, `Q-IO-5`, `Q-IO-6`, `Q-X-1`, `Q-X-2`, `Q-X-3`, `Q-X-5`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

valores puros: jars oficiales 1.7.1 vía `ORACLE=jars tools/oracle/run.sh`; comportamiento con actores: Pekko compilado desde upstream/pekko vía `tools/oracle/sbt.sh` (procedencia: commit 9a54d12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: IO(ext) punto de entrada y IO.Extension (actor manager por extensión: IO-TCP, IO-UDP-FF,… (+5 filas)

Filas: AC-IO-001, AC-IO-002.1, AC-IO-002.2, AC-IO-002.3, AC-IO-002.4, AC-IO-002.5. Contratos y aserciones: `docs/inventory/contracts/classic-io-ser.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: Cierre: Close, ConfirmedClose, Abort, PeerClosed, ErrorClosed, keepOpenOnPeerClosed y… (+5 filas)

Filas: AC-IO-002.6, AC-IO-002.7, AC-IO-003, AC-IO-004.1, AC-IO-004.2, AC-IO-004.3. Contratos y aserciones: `docs/inventory/contracts/classic-io-ser.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 3: Bucle de selección y ChannelRegistry: interest ops, MaxSelectMillis=10000,… (+5 filas)

Filas: AC-IO-004.4, AC-IO-005, AC-IO-006.1, AC-IO-006.2, AC-IO-006.3, AC-IO-007. Contratos y aserciones: `docs/inventory/contracts/classic-io-ser.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 4: Extensión Dns: DnsExt, Dns.resolve/cached (nueva y deprecada), selección de resolver,… (+5 filas)

Filas: AC-IO-008.1, AC-IO-008.2, AC-IO-008.3, AC-IO-008.4, AC-IO-008.5, AC-IO-009.1. Contratos y aserciones: `docs/inventory/contracts/classic-io-ser.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 5: AsyncDnsResolver: caché con localhost precargado, TTL mínimo, literales IP inmediatos,… (+5 filas)

Filas: AC-IO-009.2, AC-IO-009.3, AC-IO-009.4, AC-IO-009.5, AC-IO-009.6, AC-IO-009.7. Contratos y aserciones: `docs/inventory/contracts/classic-io-ser.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AC-IO-001 | IO(ext) punto de entrada y IO.Extension (actor manager por extensión: IO-TCP,… | `IoExtension` trait con `fn manager(&self) -> ActorRef`; `io::tcp(system)`,… | actor-tests/src/test/scala/org/apache/pekko/io/TcpIntegrationSpec.scala:43-175 ;… | Q-IO-1 |
| AC-IO-002.1 | Tcp: jerarquía de mensajes… | `enum TcpCommand`, `enum TcpEvent`, `struct CommandFailed { cmd, cause: Option<..> }` | actor-tests/src/test/scala/org/apache/pekko/io/TcpConnectionSpec.scala:227-310 | Q-IO-3, Q-X-3 |
| AC-IO-002.2 | Tcp.Bind y TcpListener: Bound, backlog, puerto 0, batch-accept, pull mode… | `TcpListener` actor con `TcpEvent::Bound` | actor-tests/src/test/scala/org/apache/pekko/io/TcpListenerSpec.scala:36-142 ;… | Q-IO-1, Q-IO-2 |
| AC-IO-002.3 | Tcp.Connect y TcpOutgoingConnection: resolución DNS, timeout de conexión,… | `TcpConnect` flujo con `TcpStream::connect` + timeout | actor-tests/src/test/scala/org/apache/pekko/io/TcpConnectionSpec.scala:101-107,630-672,84… | Q-IO-1, Q-IO-2, Q-IO-3, Q-IO-4 |
| AC-IO-002.4 | Register y flujo de lectura: Received, Suspend/ResumeReading, pull mode,… | `TcpConnection` actor con `read_loop` y flag `reading_suspended` | actor-tests/src/test/scala/org/apache/pekko/io/TcpConnectionSpec.scala:122-177,362-420 | Q-IO-1, Q-IO-2 |
| AC-IO-002.5 | Escritura: Write/WritePath/CompoundWrite, ack y NoAck, rechazo con… | `WriteQueue` de una sola entrada + `PendingWrite` | actor-tests/src/test/scala/org/apache/pekko/io/TcpConnectionSpec.scala:179-360,614-626,71… | Q-IO-2, Q-IO-3 |
| AC-IO-002.6 | Cierre: Close, ConfirmedClose, Abort, PeerClosed, ErrorClosed,… | `TcpConnection` con estados `Connected`, `PeerSentEof`, `ClosingWithPendingWrite`,… | actor-tests/src/test/scala/org/apache/pekko/io/TcpConnectionSpec.scala:422-626 ;… | Q-IO-2, Q-IO-3 |
| AC-IO-002.7 | Ciclo de vida de conexión: death pact con comandante y handler,… | `TcpConnection` con `watch(handler)` y temporizador de registro | actor-tests/src/test/scala/org/apache/pekko/io/TcpConnectionSpec.scala:674-708,843-869 ;… | Q-IO-1 |
| AC-IO-003 | Configuración pekko.io.tcp: TcpExt.Settings y SelectionHandlerSettings… | `TcpSettings::from_config(&Config) -> Result<TcpSettings, ConfigError>` | actor-tests/src/test/scala/org/apache/pekko/io/CapacityLimitSpec.scala:27-53 ;… | Q-IO-1, Q-IO-3, Q-X-1 |
| AC-IO-004.1 | TcpManager/UdpManager/UdpConnectedManager y SelectorBasedManager: pool de… | `SelectorPoolManager` | actor-tests/src/test/scala/org/apache/pekko/io/TcpListenerSpec.scala:36 (usa el router… | Q-IO-1 |
| AC-IO-004.2 | Protección de capacidad del selector: max-channels,… | `SelectorCapacityGuard` | actor-tests/src/test/scala/org/apache/pekko/io/CapacityLimitSpec.scala:27-53 ;… | Q-IO-1 |
| AC-IO-004.3 | Estrategias de supervisión de IO: connectionSupervisorStrategy y estrategia de… | `IoSupervisorStrategy` | derivada del código (ver bloque de contrato) | Q-X-5 |
| AC-IO-004.4 | Bucle de selección y ChannelRegistry: interest ops, MaxSelectMillis=10000,… | `trait ChannelRegistry` (costura para tests) + reactor | actor-tests/src/test/scala/org/apache/pekko/io/TcpConnectionSpec.scala:883-1142 (soporte… | Q-IO-1, Q-IO-2 |
| AC-IO-005 | Opciones de socket: Inet.SocketOption(+V2), Inet.SO.*, Tcp.SO.*,… | `trait SocketOption` sobre `socket2::Socket` | actor-tests/src/test/scala/org/apache/pekko/io/TcpConnectionSpec.scala:101-120 ;… | Q-IO-3 |
| AC-IO-006.1 | Udp.Bind y UdpListener: Bound, recepción por lotes (receive-throughput),… | `UdpListener` actor sobre `UdpSocket` | actor-tests/src/test/scala/org/apache/pekko/io/UdpIntegrationSpec.scala:31-130 ;… | Q-IO-1 |
| AC-IO-006.2 | Udp.Send y SimpleSender (WithUdpSend/UdpSender): ack, cola de una posición,… | `UdpSender` actor y `UdpSend` command | actor-tests/src/test/scala/org/apache/pekko/io/UdpIntegrationSpec.scala:31-130 | Q-IO-1, Q-IO-5 |
| AC-IO-006.3 | Configuración pekko.io.udp y UdpSettings (nr-of-selectors, max-channels=4096,… | `UdpSettings::from_config` | derivada del código (ver bloque de contrato) | Q-IO-3 |
| AC-IO-007 | UDP conectado: UdpConnected… | `UdpConnectedActor` | actor-tests/src/test/scala/org/apache/pekko/io/UdpConnectedIntegrationSpec.scala:59-174 | Q-IO-1, Q-IO-5 |
| AC-IO-008.1 | Extensión Dns: DnsExt, Dns.resolve/cached (nueva y deprecada), selección de… | `Dns` trait con `cached()`/`resolve()`; `DnsExt` | actor-tests/src/test/scala/org/apache/pekko/io/TcpConnectionSpec.scala:652-660 ;… | Q-IO-4, Q-X-2 |
| AC-IO-008.2 | DnsProtocol (Resolve/Resolved, Ip/Srv), ResourceRecord (A, AAAA, CNAME, SRV,… | `DnsRequest`, `DnsResolved`, `ResourceRecord`, `CachePolicy` | actor-tests/src/test/scala/org/apache/pekko/io/dns/internal/AsyncDnsResolverSpec.scala:56… | - |
| AC-IO-008.3 | InetAddressDnsResolver (resolver "inet-address"): positive-ttl/negative-ttl,… | `BlockingDnsResolver` | actor-tests/src/test/scala/org/apache/pekko/io/InetAddressDnsResolverSpec.scala:31-149 | Q-IO-4 |
| AC-IO-008.4 | SimpleDnsCache y PeriodicCacheCleanup: expiración exacta, limpieza periódica,… | `SimpleDnsCache` con reloj inyectable | actor-tests/src/test/scala/org/apache/pekko/io/SimpleDnsCacheSpec.scala:33-76 | - |
| AC-IO-008.5 | SimpleDnsManager, DnsProvider/InetAddressDnsProvider/AsyncDnsProvider: manager… | `DnsManager` actor | derivada del código (ver bloque de contrato) | Q-IO-1, Q-X-2 |
| AC-IO-009.1 | DnsSettings (async-dns): nameservers, resolve-timeout, positive/negative-ttl,… | `AsyncDnsSettings::from_config` | actor-tests/src/test/scala/org/apache/pekko/io/dns/DnsSettingsSpec.scala:40-146 ;… | Q-IO-6, Q-X-1 |
| AC-IO-009.2 | AsyncDnsResolver: caché con localhost precargado, TTL mínimo, literales IP… | `AsyncDnsResolver` actor | actor-tests/src/test/scala/org/apache/pekko/io/dns/internal/AsyncDnsResolverSpec.scala:55… | Q-IO-6 |
| AC-IO-009.3 | DnsClient (UDP): Bind efímero, preguntas A/AAAA/SRV, ids en vuelo,… | `DnsClient` actor sobre UDP + `TcpDnsClient` | actor-tests/src/test/scala/org/apache/pekko/io/dns/internal/DnsClientSpec.scala:28-296 | - |
| AC-IO-009.4 | TcpDnsClient: DNS sobre TCP con prefijo de longitud de 2 bytes, reconexión,… | `TcpDnsClient` actor | actor-tests/src/test/scala/org/apache/pekko/io/dns/internal/TcpDnsClientSpec.scala:31-130 | - |
| AC-IO-009.5 | Códec DNS: Message/MessageFlags/OpCode/ResponseCode, Question, DomainName… | `DnsMessage::{write, parse}`, `DomainName::{write, parse}` | actor-tests/src/test/scala/org/apache/pekko/io/dns/internal/DomainNameSpec.scala:33-65 ;… | - |
| AC-IO-009.6 | ResolvConfParser: search/domain/options ndots de /etc/resolv.conf | `ResolvConf::parse_lines` | actor-tests/src/test/scala/org/apache/pekko/io/dns/internal/ResolvConfParserSpec.scala:27… | Q-IO-6 |
| AC-IO-009.7 | IdGenerator: políticas thread-local-random, secure-random,… | `DnsIdGenerator` | actor-tests/src/test/scala/org/apache/pekko/io/dns/IdGeneratorSpec.scala:25-37 | - |

## Gates y criterio de cierre

```bash
cargo fmt --all --check
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo llvm-cov --workspace --lcov --output-path target/lcov.info   # cobertura de líneas nuevas o modificadas >= 90 %; reportar comando, resultado y líneas no cubiertas con motivo
```

La spec se cierra únicamente cuando: (1) todos los gates pasan; (2) cada aserción `A#` de cada fila tiene un test que se observó fallar y luego pasar; (3) la cobertura de líneas nuevas o modificadas es >= 90 % y está documentada; (4) las columnas `impl` y `validation` del inventario reflejan lo demostrado (`partial` si queda una decisión abierta o una diferencia); (5) las diferencias observables respecto de Pekko están anotadas con su `Q-`. Tests verdes no equivalen a paridad.

## Siguiente

Spec 25 `AC-IO-2` (`25-ac-io-2.md`).
