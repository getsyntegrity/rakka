# Contratos verificables: grupo `classic-system` (SYS, EVT, TK)

Fuente: Apache Pekko 1.7.1 (`upstream/pekko`, commit `9a54d129…`). Todas las rutas son relativas a `upstream/pekko`. Las citas `path:line` fueron generadas desde lecturas con números de línea y validadas mecánicamente (existencia de archivo y rango, más un conjunto de anclas de texto en las líneas clave). `analysis=extracted` sólo si se leyó la implementación y el cuerpo de los tests citados; si no, `listed` con el motivo en `notes`. Las aserciones marcadas *no afirmada por spec* salen del código, no de un test.

Convención de `impl`/`validation`: todas las filas del grupo están `pending` (ninguna implementación existe aún). Decisiones transversales referenciadas sin redefinir: Q-X-1 (HOCON), Q-X-2 (registro de fábricas), Q-X-3 (mensajes dinámicos), Q-X-4 (runtime), Q-X-5 (fallos), Q-X-6 (jerarquía de eventos), Q-X-7 (loggers actor vs backend), Q-X-8 (typed sobre classic).

## AC-SYS-001.1 — Validación del nombre de ActorSystem (regex y mensaje)

*Unidad SYS · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] El nombre debe cumplir ^[a-zA-Z0-9][a-zA-Z0-9-_]*$ (match completo); si no, IllegalArgumentException con mensaje fijo. Crear un sistema con un nombre inválido falla antes de arrancar nada. Nombres válidos: ASCII alfanumérico inicial, luego alfanuméricos, '-' o '_'. Los nombres inválidos del test incluyen prefijo '-' o '_', espacio, '*', '@', '#', '$', '%', '/'.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:837-840 (validación en el cuerpo de ActorSystemImpl)
- Scaladoc/doc pública: n/a (el patrón aparece sólo en el código y en el spec)
- reference.conf: n/a

**Condiciones.**
- Normal: 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_' es válido (actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:163-165).
- Error: IllegalArgumentException con mensaje exacto `invalid ActorSystem name [<name>], must contain only word characters (i.e. [a-zA-Z0-9] plus non-leading '-' or '_')` (actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:837-840; el spec sólo verifica el tipo, el texto es derivado del código).
- Límite: cadena vacía no matchea (primer carácter obligatorio); el spec actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:155-161 lo cubre por propiedad (cualquier String que no matchee el regex).

**Orden / concurrencia / ciclo de vida.** Se valida en el constructor de ActorSystemImpl, antes de crear settings/proveedor/hilos; no hay efectos secundarios si falla.

**Dependencias.** Filas: -. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `ActorSystemName::parse(&str) -> Result<ActorSystemName, InvalidSystemName>`

**Diferencias observables.** Rust: tipo `ActorSystemName` validado en construcción. Diferencia observable: el error es `Err`, no excepción (Q-X-5); el regex se aplica sobre Unicode como ASCII estricto (igual que Java: `[a-zA-Z0-9]` no admite letras no ASCII).

**Aserciones.**
- A1: los nombres '-hallowelt', '_hallowelt', 'hallo welt', 'hallo*welt', 'hallo@welt', 'hallo#welt', 'hallo$welt', 'hallo%welt', 'hallo/welt' lanzan IllegalArgumentException — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:138-153.
- A2: todo String que no matchee `^[a-zA-Z0-9][a-zA-Z0-9-_]*$` lanza IllegalArgumentException (propiedad) — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:155-161.
- A3: el nombre con los 64 caracteres válidos (a-z, A-Z, 0-9, '-', '_') se acepta — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:163-165.
- A4: el mensaje de error es el literal de actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:837-840 — derivada del código, *no afirmada por spec*.

**Oráculo.** Jar oficial pekko-actor 1.7.1: escenario propuesto `tools/oracle/sys-name-validation.jsh` (no existe) que llama `ActorSystem.create(n)` y vuelca excepción+mensaje para una tabla de nombres; procedencia: jars oficiales.

**Decisiones.** Q-X-5

## AC-SYS-001.2 — Constructores de ActorSystem: apply/create y sus sobrecargas, nombre por defecto "default"

*Unidad SYS · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual] Todas las sobrecargas de `ActorSystem.apply`/`create` convergen en un único punto, `apply(name, setup: ActorSystemSetup)`, que resuelve classLoader, config y executor por defecto desde el `BootstrapSetup` contenido en el setup y construye `new ActorSystemImpl(name, appConfig, cl, defaultEC, None, setup).start()`. `ActorSystem()`/`create()` usan el nombre `"default"`. Un argumento `null` (Java) o `None` (Scala) de config, classLoader o executor significa "usar el valor por defecto": config = `ConfigFactory.load(cl)`, classLoader = `findClassLoader()`, executor = `pekko.actor.default-dispatcher.default-executor.fallback`. El sistema devuelto ya está arrancado (secuencia de AC-SYS-002).

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:192-327 (todas las sobrecargas `create`/`apply`)
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:262 (`apply()` = `apply("default")`), 272 (`apply(name)`), 278-285 (cuerpo común), 291-292 (`apply(name, BootstrapSetup)` = `create(name, ActorSystemSetup.create(..))`), 323-328 (`apply(name, Option..)` empaqueta en `BootstrapSetup`)
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:48-179 (`BootstrapSetup`, `ProviderSelection`: ver AC-SYS-018 y AC-CFG-015)
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:846-860 (fusión de config dentro de `ActorSystemImpl`: `applicationConfig.withFallback(ConfigFactory.defaultReference(classLoader))` más `amendSlf4jConfig`)
- reference.conf: `pekko.actor.default-dispatcher.default-executor.fallback` (documentado en ActorSystem.scala:241-245)
- specs: actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:253-264; actor-tests/src/test/scala-2.13+/org/apache/pekko/actor/ActorSystemScala213PlusSpec.scala:29-35; actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemDispatcherSpec.scala:53-98,121-150

**Precondiciones.** `name` cumple AC-SYS-001.1 (se valida en el constructor de `ActorSystemImpl`, no en `apply`). La config proporcionada debe superar `checkValid` contra la referencia (ActorSystem.scala:395-400, ver AC-CFG).

**Comportamiento.**
- `apply(name, setup)`: `bootstrapSettings = setup.get[BootstrapSetup]`; `cl = bootstrap.classLoader.getOrElse(findClassLoader())`; `appConfig = bootstrap.config.getOrElse(ConfigFactory.load(cl))`; `defaultEC = bootstrap.defaultExecutionContext` (ActorSystem.scala:278-285).
- La config del usuario tiene precedencia; lo que falta cae a `defaultReference(classLoader)` (ActorSystem.scala:852-854). El proveedor se elige primero por `BootstrapSetup.actorRefProvider` y, si falta, por `pekko.actor.provider` (ActorSystem.scala:410-416).
- El executor pasado se usa como default executor de todos los dispatchers con `executor = "default-executor"`, salvo que `pekko.actor.default-dispatcher.executor` esté configurado explícitamente (entonces se ignora): ActorSystem.scala:241-245 y ActorSystemDispatcherSpec.scala:77-98.
- Los constructores públicos nunca pasan `guardianProps` (siempre `None`); sólo el constructor interno de `ActorSystemImpl` acepta `Some(props)` (AC-SYS-009).

**Errores.**
- Nombre inválido: `IllegalArgumentException` antes de arrancar nada (AC-SYS-001.1).
- Config inválida respecto a la referencia: `ConfigException` de `checkValid` al construir `Settings` (ActorSystem.scala:395-400). Versión de config distinta de `pekko.version`: `ConfigurationException("Pekko JAR version [..] does not match the provided config version [..]")` (ActorSystem.scala:485-487; pertenece a AC-CFG).
- Fallo durante `start()`: se llama `terminate()` y se relanza el error (AC-SYS-002).

**Orden / concurrencia / ciclo de vida.** La construcción es síncrona: cuando la sobrecarga retorna, el sistema está arrancado. Cada llamada crea un sistema independiente (no hay registro global; dos sistemas con el mismo nombre pueden coexistir: ActorSystemSpec.scala:301 crea otro `ActorSystem()` mientras el del spec vive).

**Dependencias.** Filas: AC-SYS-001.1 ; AC-SYS-002 ; AC-SYS-018 ; AC-CFG-001 ; AC-CFG-015. Externas: Typesafe Config.

**Equivalente Rust propuesto.** `ActorSystem::builder(name).config(cfg).execution_context(handle).provider(sel).build() -> Result<ActorSystem, SystemError>` y `ActorSystem::default()` (nombre `"default"`); atajos `ActorSystem::new(name)` y `ActorSystem::with_config(name, cfg)`. Viabilidad alta: el builder reproduce la composición de `BootstrapSetup`; `Option` sustituye a `null`.

**Diferencias observables.**
- No hay sobrecargas con `ClassLoader` ni `DynamicAccess` (Q-SYS-6): la resolución de clases por nombre se sustituye por el registro de fábricas (Q-X-2).
- `defaultExecutionContext` se mapea al runtime/handle elegido en Q-X-4; hasta esa decisión la sobrecarga con executor queda `partial`.
- Todas las formas Java/Scala colapsan en un único builder (no existe la distinción `create`/`apply`).
- `build()` devuelve `Err` en vez de lanzar (Q-X-5).

**Aserciones.**
- A1: `ActorSystem()` crea un sistema cuyo `whenTerminated` no está completo hasta `terminate()`, y tras terminar entrega `Terminated(rootGuardian)` con ambos flags `true` — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:253-264 (usa el nombre por defecto sin afirmarlo).
- A2: `Using(ActorSystem()) { .. }` deja el sistema terminado al salir (AutoCloseable.close) — actor-tests/src/test/scala-2.13+/org/apache/pekko/actor/ActorSystemScala213PlusSpec.scala:29-35.
- A3: con `defaultExecutionContext = Some(ec)` el executor recibe las ejecuciones del dispatcher por defecto (el probe observa `"called"` y el actor responde `"pong"`) — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemDispatcherSpec.scala:53-75.
- A4: si el default-dispatcher tiene `executor` configurado explícitamente (`fork-join-executor`) el executor pasado no se usa (`ecProbe.expectNoMessage()`) — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemDispatcherSpec.scala:77-98.
- A5: el executor pasado también se usa para el `internal-dispatcher` cuando éste se redirige al default-dispatcher — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemDispatcherSpec.scala:121-150.
- A6: `ActorSystem()` produce un sistema llamado `"default"` — derivada de ActorSystem.scala:263 y 272, *no afirmada por spec*. Caracterización propuesta: `assert_eq!(ActorSystem::default().name(), "default")`.
- A7: `apply(name, bootstrapSetup)` ≡ `create(name, ActorSystemSetup.create(bootstrapSetup))` ≡ `apply(name, setup)` — derivada de ActorSystem.scala:291-292,208, *no afirmada por spec*. Caracterización propuesta: el builder con `provider(..)` y `config(..)` produce el mismo `settings` que la ruta por setup.

**Validación prevista.** A1-A5: tests portados 1:1 (A3-A5 dependen de Q-X-4). A6-A7: tests de caracterización propios. Diferencial con el jar oficial (escenario propuesto `tools/oracle/sys-constructors.jsh`) para A6 y la precedencia de config (un `Config` parcial más defaults): justificado porque la fusión es observable por `settings.config`.

**Decisiones.** Q-X-1 (HOCON), Q-X-4 (runtime/executor), Q-X-5 (Err vs excepción), Q-SYS-6 (ClassLoader/DynamicAccess; efecto: se omiten las sobrecargas con ClassLoader).

## AC-SYS-002 — Secuencia de arranque (start): provider.init (guardián raíz, cadena de Watch /user→/system→/, loggers por defecto), deadLetterListener, unsubscriber, chequeo de módulos, extensiones, log-config-on-start

*Unidad SYS · visibilidad internal-required · analysis `extracted`*

**Contrato observable.** [contractual interno] Orden fijo dentro de `_start` (lazy, ejecutada una sola vez): `registerOnTermination(stopScheduler())` → `provider.init(this)` → `_initialized = true` → si `log-dead-letters > 0`, `systemActorOf(DeadLetterListener, "deadLetterListener")` → `eventStream.startUnsubscriber()` → `ManifestInfo.checkSameVersion("Apache Pekko", allModules, logWarning = true)` → `loadExtensions()` si no `terminating` → `logConfiguration()` si `log-config-on-start`. Ante `NonFatal(e)`: `terminate()` (o `stopScheduler` si `terminate` falla) y se relanza `e`. Al volver, el sistema tiene guardianes, loggers por defecto, listener de dead letters (si procede), unsubscriber del EventStream y extensiones cargadas.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1078-1114 (`_initialized`, `assertInitialized`, `_start`, `start()`)
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1198-1291 (registro de extensiones y `loadExtensions`)
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:620-627 (`LocalActorRefProvider.init`)
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:981-986 (stdout logger previo a `start`)
- reference.conf: `pekko.log-config-on-start`, `pekko.log-dead-letters`, `pekko.library-extensions`, `pekko.extensions`
- specs: actor-tests/src/test/scala/org/apache/pekko/actor/ExtensionSpec.scala:100-121,154-173

**Precondiciones.** Objeto `ActorSystemImpl` completamente construido (eventStream, scheduler, provider, dispatchers y mailboxes ya creados: ActorSystem.scala:952-1000). Hasta `provider.init` el único logger activo es el stdout de arranque (ActorSystem.scala:953-954).

**Comportamiento.**
- `provider.init`: guarda `system`, `rootGuardian.start()`, `systemGuardian ! Watch(guardian, systemGuardian)`, `rootGuardian ! Watch(systemGuardian, rootGuardian)` (cadena de vigilancia: la muerte de /user detiene /system y ésta /), luego `eventStream.startDefaultLoggers(system)` (ActorRefProvider.scala:613-620).
- `deadLetterListener` es hijo de `/system` creado con `systemActorOf` sólo si `settings.LogDeadLetters > 0` (ActorSystem.scala:1068-1069).
- `loadExtensions`: primero `pekko.library-extensions` con `throwOnLoadFail = true`, luego `pekko.extensions` con `throwOnLoadFail = false`; para cada FQCN se intenta `getObjectFor` y, si falla, `createInstanceFor` (ActorSystem.scala:1230-1260). Duplicados no crean dos instancias (`registerExtension` es idempotente: ActorSystem.scala:1195-1221).
- Si `terminating` ya es `true` antes de cargar extensiones, éstas no se cargan (ActorSystem.scala:1072-1073).

**Errores.**
- Extensión de `pekko.library-extensions` que no se puede cargar o no es `ExtensionId`: `RuntimeException("While trying to load extension [fqcn]", cause)` / `"[fqcn] is not an 'ExtensionIdProvider' or 'ExtensionId'"` y el sistema falla (ActorSystem.scala:1250-1253).
- Extensión de `pekko.extensions` no cargable: se loguea `error("While trying to load extension [{}], skipping...")` y el arranque continúa (ActorSystem.scala:1249,1252).
- Extensión (de cualquier lista) cuyo `createExtension` lanza: la excepción se propaga y `registerExtension` la cachea (ActorSystem.scala:1209-1212); el arranque falla → `terminate()` + relanzar (ActorSystem.scala:1075-1081).
- `assertInitialized()` antes de `_initialized`: `IllegalStateException` (AC-SYS-023).

**Orden / concurrencia / ciclo de vida.** `_start` es `lazy val` ⇒ ejecución única y thread-safe. `_initialized=true` ocurre antes del `deadLetterListener` (habilita `assertInitialized` para extensiones). Todo el arranque es síncrono en el hilo que llama a `apply`.

**Dependencias.** Filas: AC-SYS-003.1 ; AC-SYS-010 ; AC-SYS-020 ; AC-SYS-023 ; AC-EXT-003 ; AC-EVT-007 ; AC-EVT-009 ; AC-EVT-011. Externas: ninguna.

**Equivalente Rust propuesto.** `ActorSystemBuilder::build()` ejecuta la secuencia en el mismo orden y, ante error, termina el sistema y devuelve `Err`. Las extensiones se cargan desde un registro de fábricas por nombre (Q-X-2): `library-extensions` → `Err(SystemError::ExtensionLoad)` ante fallo; `extensions` → log de error y continuar. Viabilidad alta; el orden es puramente secuencial.

**Diferencias observables.**
- Carga reflexiva por FQCN (objeto Scala o clase con constructor sin argumentos) → registro explícito de fábricas (Q-X-2); el texto `While trying to load extension [..]` se conserva.
- `build()` es síncrono pero los loggers como actores dependen de Q-X-7.
- Los nombres de actores de sistema (`log1-DefaultLogger`, `deadLetterListener`, `eventStreamUnsubscriber-N`) se conservan si se mantiene la jerarquía `/system`.

**Aserciones.**
- A1: una extensión listada en `pekko.extensions` que falla al crearse hace fallar `ActorSystem(..)` con `RuntimeException` — actor-tests/src/test/scala/org/apache/pekko/actor/ExtensionSpec.scala:100-110.
- A2: una extensión listada en `pekko.extensions` que no existe NO impide el arranque y se loguea `error("While trying to load extension [org.apache.pekko.actor.MissingExtension], skipping.")` — actor-tests/src/test/scala/org/apache/pekko/actor/ExtensionSpec.scala:112-121.
- A3: una `library-extension` que falla al crearse hace fallar el arranque propagando la excepción original (`FailingTestExtension.TestException`) — actor-tests/src/test/scala/org/apache/pekko/actor/ExtensionSpec.scala:154-163.
- A4: una `library-extension` inexistente hace fallar el arranque con `RuntimeException` — actor-tests/src/test/scala/org/apache/pekko/actor/ExtensionSpec.scala:165-173.
- A5: el `deadLetterListener` se crea bajo `/system` sólo si `log-dead-letters > 0` — derivada de ActorSystem.scala:1068-1069, *no afirmada por spec*. Caracterización propuesta: con `log-dead-letters = off` el hijo `deadLetterListener` no existe en `/system`; con `on` existe.
- A6: la cadena de Watch /user → /system → / está establecida antes de cualquier actor de usuario — derivada de ActorRefProvider.scala:617-618, *no afirmada por spec* (comportamiento observable indirecto: ActorSystemSpec.scala:328-334 afirma que fallar `/user` termina el sistema, ver AC-SYS-008).
- A7: tras un fallo de arranque el sistema queda terminado y el error se propaga — derivada de ActorSystem.scala:1075-1081; indirectamente observada por A1/A3 (la excepción llega al llamador), *el estado terminado no está afirmado por spec*.
- A8: `log-config-on-start = on` emite `info(settings.toString)` una vez — derivada de ActorSystem.scala:920,1074, *no afirmada por spec*.
- A9: `loadExtensions` no se ejecuta si el sistema ya está terminando — derivada de ActorSystem.scala:1072-1073, *no afirmada por spec*.
- A10: `assertInitialized()` lanza `IllegalStateException` mientras `_initialized` sea `false` — ver AC-SYS-023.

**Validación prevista.** A1-A4: tests portados 1:1 (con `FailingTestExtension`/`MissingExtension` reescritos como fábricas registradas). A5-A9: tests de caracterización con aserciones sobre el árbol `/system` y el EventStream. Diferencial con el jar oficial (escenario `tools/oracle/sys-start-tree.jsh`: `printTree` tras arrancar) sólo para el conjunto de hijos de `/system`; el texto de `printTree` no se compara (AC-SYS-013).

**Decisiones.** Q-X-2 (efecto: extensiones por registro), Q-X-5, Q-X-7 (loggers como actores o backend), Q-SYS-4 (efecto: el protocolo de hooks del guardián de sistema forma parte de /system; ver AC-SYS-003.2).

## AC-SYS-003.1 — Jerarquía de guardianes "/", "/user", "/system" y su comportamiento ante mensajes ajenos

*Unidad SYS · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] Los tres guardianes existen; un mensaje cualquiera enviado a ellos se publica como UnhandledMessage (log Warning 'unhandled message from Actor[<deadLetters path>]: <msg>'), no como DeadLetter. / (root), /user y /system son actores; rootGuardian es su propio padre; el padre de rootGuardian es el 'bubble walker'.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:322-333 (Guardian)
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:545-559 (rootGuardian)
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:565-591 (guardian /user)
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:593-606 (systemGuardian /system)
- docs: n/a (el link a supervision.md del inventario original no se verificó)

**Condiciones.**
- Guardian: `Terminated(_) => context.stop(self)`; `StopChild(child) => context.stop(child)` (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:326-329).
- Los guardianes no pierden hijos en reinicio: `preRestart` vacío (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:332,380).
- rootGuardian.getSingleChild: 'temp' -> tempContainer, 'deadLetters' -> deadLetters, otros -> extraNames o hijo real (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:553-558).
- Con guardianProps (AC-SYS-009) /user usa esos Props y su dispatcher (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:569-584).
- Error: root guardian usa estrategia que loguea 'guardian failed, shutting down system' y Stop (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:525-529).

**Orden / concurrencia / ciclo de vida.** Primero root, luego /user (reserveChild + initChild + start), luego /system. Los tres están en `lazy val`: se materializan en provider.init.

**Dependencias.** Filas: AC-SUP-014 ; AC-CELL-004 ; AC-EVT-010. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `RootGuardian`, `UserGuardian`, `SystemGuardian` como actores internos

**Diferencias observables.** Rust: tres actores internos con mailbox sin límite (UnboundedMessageQueueSemantics, actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:324,340). Diferencia: ninguna observable si el path y el warning se conservan.

**Aserciones.**
- A1: `rootGuardian ! "Hello, Mr. Root Guardian"` produce exactamente 1 warning `unhandled message from Actor[<path de deadLetters>]: Hello, Mr. Root Guardian` — actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:69-80.
- A2: ídem para /user — actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:82-93.
- A3: ídem para /system — actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:95-106.
- A4: el hijo de un actor se resuelve por path con nombre URL-encoded `pekko%3A%2F%2FClusterSystem%40127.0.0.1%3A7355` y mantiene `path.name` — actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:48-64.

**Oráculo.** Jar oficial: escenario propuesto `sys-guardians-unhandled.jsh` (envía a /, /user, /system y captura eventos); procedencia jars oficiales.

**Decisiones.** Q-X-3

## AC-SYS-003.2 — Protocolo de hooks de terminación del guardián de sistema (RegisterTerminationHook / TerminationHook / TerminationHookDone)

*Unidad SYS · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual docs] /system registra hooks (sender de RegisterTerminationHook, vigilado con watch); cuando /user termina notifica TerminationHook a todos, espera TerminationHookDone o Terminated de cada uno, detiene los loggers por defecto y se detiene; la raíz para al último. Mecanismo interno por el que remote/cluster (y extensiones) retrasan el apagado de /system. En local sin remote nadie lo usa.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:302-315 (mensajes)
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:338-381 (SystemGuardian)
- Scaladoc/doc pública: n/a
- reference.conf: n/a

**Condiciones.**
- `RegisterTerminationHook` sólo se acepta si `sender() != context.system.deadLetters` (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:358).
- Antes de Terminated(guardian): un hook que muere se elimina del set (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:353-356).
- Terminated(guardian): `context.become(terminating)`, enviar TerminationHook a todos, y comprobar si ya está vacío (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:346-352).
- En `terminating`: `Terminated(a)` o `TerminationHookDone` (de sender()) quitan el hook y, si el set queda vacío, `eventStream.stopDefaultLoggers(system)` y `context.stop(self)` (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:363-377).

**Orden / concurrencia / ciclo de vida.** /system vigila a /user (Watch en init). Los loggers se detienen después de que todos los hooks terminaron, de modo que los hooks aún pueden loguear.

**Dependencias.** Filas: AC-SYS-003.1 ; AC-DW-001 ; AC-EVT-011. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `SystemGuardian` con `HashSet<ActorRef>` de hooks; mensajes internos `RegisterTerminationHook/TerminationHook/TerminationHookDone`

**Diferencias observables.** Rust: mismos mensajes como tipos internos. Diferencia: ninguna observable local; sin remote es código sin consumidor (Q-SYS-4).

**Aserciones.**
- A1: un RegisterTerminationHook cuyo sender es deadLetters se ignora — derivada del código actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:358, *no afirmada por spec*.
- A2: al terminar /user, cada hook registrado recibe TerminationHook — actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:351, *no afirmada por spec*.
- A3: /system no se detiene hasta que cada hook responde TerminationHookDone o termina — actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:364-365,373-377, *no afirmada por spec*.
- A4: stopDefaultLoggers corre inmediatamente antes de context.stop(self) del guardián de sistema — actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:374-376, *no afirmada por spec*.
- A5: un hook que termina antes de Terminated(guardian) se retira del set — actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:353-356, *no afirmada por spec*.

**Oráculo.** Parcial: con jar oficial se puede registrar un actor-hook vía `system.systemActorOf` + `RegisterTerminationHook` (clase private[pekko]; requiere jshell en package pekko.actor). Escenario propuesto `sys-termination-hook.jsh`; procedencia jars oficiales.

**Decisiones.** Q-SYS-4

## AC-SYS-004 — terminate(): idempotente; corre CoordinatedShutdown si run-by-actor-system-terminate, si no finalTerminate (stop de /user en cascada)

*Unidad SYS · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual] Devuelve whenTerminated; múltiples llamadas no fallan; al completar, whenTerminated está completo con Terminated(rootGuardian) con existenceConfirmed=true y addressTerminated=true y es la misma instancia que devuelve terminate(). terminate pone `terminating=true`; si CoordinatedShutdown está habilitado delega en `CoordinatedShutdown.run(ActorSystemTerminateReason)` (que luego llama finalTerminate); si no, `finalTerminate()`.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1120-1135 (terminate)
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1143-1148 (finalTerminate: detiene deadLetterListener si !log-dead-letters-during-shutdown; guardian.stop())
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:455-494 (bubble walker completa terminationPromise)
- reference.conf: pekko.coordinated-shutdown.run-by-actor-system-terminate / terminate-actor-system (reference.conf, sección coordinated-shutdown)

**Condiciones.**
- Combinación run-by-actor-system-terminate=on y terminate-actor-system=off: ConfigurationException en Settings (actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:477-480, ver AC-SYS-022).
- `aborting` evita CoordinatedShutdown (actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1122).
- Idempotente: finalTerminate dice 'these actions are idempotent' (actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1144).

**Orden / concurrencia / ciclo de vida.** `terminating=true` se fija antes de cualquier otra acción (impide nuevas extensiones y rechaza actorOf, AC-SYS-007). El resultado del Future se completa cuando el bubble-walker recibe DeathWatchNotification de /.

**Dependencias.** Filas: AC-CS-004 ; AC-CS-006 ; AC-SYS-005. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `async fn terminate(&self) -> Terminated` y `fn terminate_nowait(&self)`

**Diferencias observables.** Rust: `terminate()` devuelve un future compartido (Q-X-4); `Terminated{actor: root, existence_confirmed: true, address_terminated: true}`.

**Aserciones.**
- A1: `whenTerminated` no está completo antes de terminate y lo está tras `Await` — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:253-259.
- A2: `terminated.actor == provider.rootGuardian`, `addressTerminated==true`, `existenceConfirmed==true` — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:260-262.
- A3: el valor de `whenTerminated` es la misma instancia que devuelve `terminate()` (theSameInstanceAs) — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:263.
- A4: llamar `terminate()` y luego `registerOnTermination` sólo ejecuta el callback una vez o lanza Rejected (exactamente 1 de las dos) — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:275-289.
- A5: terminate() es idempotente (múltiples llamadas no fallan) — derivada de actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1143-1148, *no afirmada por spec*.
- A6: `sys.terminate()` ejecuta `CoordinatedShutdown` con la razón `ActorSystemTerminateReason` (`shutdownReason() == Some(ActorSystemTerminateReason)`) y `whenTerminated` queda completo — actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:547-556 (auditoría 2026-10-31: este spec sí afirma el comportamiento de esta fila).
- A7: con `run-by-actor-system-terminate = off`, `terminate()` NO ejecuta `CoordinatedShutdown` (`shutdownReason() == None`) pero el sistema termina — actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:558-571.
- A8: `terminate-actor-system = off` con `run-by-actor-system-terminate = on` falla con `ConfigurationException` al crear el sistema — actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:573-583.
- A9: `CoordinatedShutdown(sys).run(CustomReason)` termina el `ActorSystem` y fija `shutdownReason()` — actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:536-545 (integración AC-CS-006.2).

**Oráculo.** Jar oficial: escenario propuesto `sys-terminate.jsh` (terminate dos veces y volcado de Terminated). Procedencia: jars oficiales.

**Decisiones.** Q-X-4

## AC-SYS-005 — whenTerminated / getWhenTerminated

*Unidad SYS · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual] Future[Terminated] compartido; se completa después de que corrieron todos los callbacks registrados con registerOnTermination. Await sobre whenTerminated sólo vuelve cuando todos los callbacks terminaron.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1031-1034
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1335-1368 (TerminationCallbacks.terminationFuture = done.future)
- reference.conf: n/a

**Condiciones.**
- El `done` Promise se completa vía cadena: cada add() encadena su ejecución antes del anterior (ver AC-SYS-006).

**Orden / concurrencia / ciclo de vida.** Todos los callbacks terminan antes de que whenTerminated se complete (cada callback corre con `andThen` y completa su promesa siguiente).

**Dependencias.** Filas: AC-SYS-006 ; AC-UTIL-003. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `system.when_terminated() -> impl Future<Output=Terminated> + Clone` (Shared)

**Diferencias observables.** Rust: `Shared<..>` o canal watch. Diferencia: ninguna observable.

**Aserciones.**
- A1: un callback de 50 ms (dilatado) registrado con registerOnTermination ya corrió cuando `Await.ready(whenTerminated)` vuelve — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:237-251.
- A2: `whenTerminated.isCompleted` es true tras terminar y false antes — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:253-259.

**Oráculo.** Jar oficial: escenario propuesto `sys-terminate.jsh` (comparte con AC-SYS-004).

**Decisiones.** Q-X-4

## AC-SYS-006 — registerOnTermination: callbacks en orden inverso al registro (encadenados); tras terminar → RejectedExecutionException

*Unidad SYS · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] Ejecución en orden inverso al registro; registrar después de terminado lanza RejectedExecutionException("ActorSystem already terminated."). Si se registra durante la terminación: o se ejecuta una vez o se rechaza (nunca ambas ni ninguna). El Scaladoc de TerminationCallbacks.add dice 'callbacks are executed in reverse order of insertion'. Implementación: lista enlazada de promesas (CAS sobre AtomicReference).

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1115-1116
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1335-1368 (TerminationCallbacks)
- reference.conf: n/a

**Condiciones.**
- `add` tras completar upstream: `RejectedExecutionException("ActorSystem already terminated.")` (actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1353).
- Los callbacks corren en el ExecutionContext del dispatcher del sistema (actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1031 usa `(dispatcher)`).

**Orden / concurrencia / ciclo de vida.** Callback N+1 corre antes que N (cada nuevo callback se ejecuta cuando completa `p.future` y el anterior espera a ése: `some.completeWith(p.future.andThen{r.run()})`). Serie, no paralelo.

**Dependencias.** Filas: AC-SYS-005. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** Pila de closures `Mutex<Option<Vec<Callback>>>`

**Diferencias observables.** Rust: pila LIFO ejecutada secuencialmente; el hilo/tarea que ejecuta es del runtime (Q-X-4).

**Aserciones.**
- A1: 10 callbacks registrados en orden 1..10 se ejecutan en orden 10..1 — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:215-235.
- A2: tras `system.close()`, `registerOnTermination` lanza RejectedExecutionException con mensaje exactamente `ActorSystem already terminated.` — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:266-273.
- A3: `terminate()` seguido de `registerOnTermination`: el contador (callback o rechazo) vale 1 — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:275-289.
- A4: registrar callbacks antes de terminar no los ejecuta hasta la terminación — derivada de actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1343-1359, *no afirmada por spec*.

**Oráculo.** Jar oficial: escenario propuesto `sys-termination-callbacks.jsh`.

**Decisiones.** Q-X-4

## AC-SYS-007 — Creación de actores durante el apagado se rechaza

*Unidad SYS · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] actorOf externo con el guardián terminando → IllegalStateException("cannot create children while terminating or terminated"); una vez que falla, falla siempre; los actores creados antes de fallar no quedan huérfanos. Mientras /user está en estado terminating (childrenRefs.isTerminating) los attachChild externos lanzan; los hijos creados antes están todos terminados o aún en UnstartedCell.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/dungeon/Children.scala:313-314 (throw)
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:944-961

**Condiciones.**
- Error: IllegalStateException con mensaje exacto `cannot create children while terminating or terminated` (Children.scala:314).
- Una vez que `failing==true` crear no puede volver a tener éxito (actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:312).

**Orden / concurrencia / ciclo de vida.** Condición de carrera intencionada: `terminate()` desde un scheduler a 100 ms mientras otro hilo crea actores sin parar; el invariante es monotonía (éxitos, luego fallos).

**Dependencias.** Filas: AC-CELL-015 ; AC-SYS-004. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `Err(SpawnError::SystemTerminating)` desde `actor_of`

**Diferencias observables.** Rust: el estado 'terminating' del contenedor de hijos del guardián debe ser atómico con respecto a reserveChild. Diferencia: error tipado.

**Aserciones.**
- A1: una vez que actorOf lanza IllegalStateException no vuelve a tener éxito (`failing should not be true`) — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:312.
- A2: tras terminar el sistema, ningún actor creado queda no terminado salvo UnstartedCell — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:326-328.
- A3: el sistema termina en < 10 s de uptime bajo creación continua — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:319-323.

**Oráculo.** Jar oficial: escenario propuesto `sys-create-during-terminate.jsh` (no determinista en tiempo; comparar sólo las invariantes A1-A2).

**Decisiones.** Q-X-4, Q-X-5

## AC-SYS-008 — Fallo o escalamiento de /user apaga el sistema; guardian-supervisor-strategy configurable

*Unidad SYS · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] Kill sobre /user (ActorKilledException) apaga el sistema; con guardian-supervisor-strategy=StoppingSupervisorStrategy un hijo de /user que falla se detiene (Terminated con existenceConfirmed=true, addressTerminated=false); con una estrategia que escala, el fallo apaga el sistema. El root guardian usa una estrategia que loguea 'guardian failed, shutting down system' y hace Stop de /user. /user usa la estrategia configurada (por defecto DefaultSupervisorStrategy).

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:517-534
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:423 (SupervisorStrategyClass)
- reference.conf:123 (pekko.actor.guardian-supervisor-strategy)

**Condiciones.**
- Kill a /user -> EventFilter[ActorKilledException] observa el error y whenTerminated se completa (actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:333-336).
- Estrategia que lanza `Exception("hello")` en un hijo con StoppingSupervisorStrategy: el hijo muere, el sistema sigue (actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:360-373).
- Estrategia `Escalate` -> el sistema termina (actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:376-392).

**Orden / concurrencia / ciclo de vida.** Fallo de /user -> reportado al root guardian -> `SupervisorStrategy.Stop` sobre /user -> /system (que vigila a /user) ejecuta hooks -> raíz para.

**Dependencias.** Filas: AC-SUP-014 ; AC-SYS-003.1 ; AC-CFG-005. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `GuardianSupervisorStrategy` cargado por nombre (Q-X-2)

**Diferencias observables.** Rust: estrategia por tipo registrado. Diferencia: FQCN de config pasa a clave de registro (Q-X-2).

**Aserciones.**
- A1: `system.actorSelection("/user") ! Kill` genera un ActorKilledException y el sistema termina — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:331-337.
- A2: con StoppingSupervisorStrategy, un `Exception("hello")` en un hijo de /user: 1 evento de error, y el probe recibe `Terminated(a)(existenceConfirmed=true, addressTerminated=false)` — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:353-374.
- A3: con una estrategia que escala, la excepción de un hijo provoca el apagado del sistema — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:376-392.

**Oráculo.** Jar oficial: escenario propuesto `sys-guardian-strategy.jsh`.

**Decisiones.** Q-X-2, Q-X-5

## AC-SYS-009 — Guardián de usuario custom (guardianProps; lo usa actor-typed)

*Unidad SYS · visibilidad internal-required · analysis `extracted`*

**Contrato observable.** [contractual por spec] Con guardianProps, actorOf top-level desde afuera lanza UnsupportedOperationException (con o sin nombre); /user usa el dispatcher indicado por los Props (default-dispatcher si SameAsParent). Con guardian custom el sistema no permite crear actores desde afuera: sólo el guardián (typed) puede crearlos.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:944-961
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:569-584

**Condiciones.**
- UnsupportedOperationException con nombre: `cannot create top-level actor [<name>] from the outside on ActorSystem with custom user guardian` (actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:947-948); sin nombre: `cannot create top-level actor from the outside on ActorSystem with custom user guardian` (actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:960-961).
- Dispatcher: `Deploy.DispatcherSameAsParent` -> DefaultDispatcherId; si no, `props.dispatcher` (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:573-575).

**Orden / concurrencia / ciclo de vida.** n/a: construcción.

**Dependencias.** Filas: AC-SYS-003.1 ; AC-DSP-006. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** parámetro interno `guardian_props: Option<Props>` del builder

**Diferencias observables.** Rust: `actor_of` devuelve `Err(Unsupported)`. El guardián custom lo usa actor-typed (Q-X-8).

**Aserciones.**
- A1: `sys.actorOf(Props.empty)` y `sys.actorOf(Props.empty, "empty")` lanzan UnsupportedOperationException — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:403-409.
- A2: con `guardianProps = Some(Props.empty)` (sin dispatcher en los props) el guardián `/user` usa `pekko.actor.default-dispatcher`, mientras que sin `guardianProps` usa `pekko.actor.internal-dispatcher` — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemDispatcherSpec.scala:152-168 (auditoría 2026-10-31: el spec está en la clase `ActorSystemDispatchersSpec`; antes figuraba como `ActorSystemDispatcherSpec:156` sin leer).
- A3: `pekko.actor.internal-dispatcher = pekko.actor.default-dispatcher` redirige el guardián al default-dispatcher — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemDispatcherSpec.scala:105-119.

**Oráculo.** Jar oficial: se puede reproducir con `new ActorSystemImpl(..., Some(Props.empty), ...)` desde jshell en el paquete pekko.actor. Escenario propuesto `sys-custom-guardian.jsh`.

**Decisiones.** Q-X-8, Q-X-5

## AC-SYS-010 — systemActorOf (actores bajo /system)

*Unidad SYS · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual] `ExtendedActorSystem.systemActorOf(props, name)` crea un hijo del guardián de sistema con `systemService = true` (se omite el `Deployer`: no se aplica configuración `pekko.actor.deployment` externa) y devuelve su `ActorRef`. Es la forma de crear actores internos de librerías (loggers, `deadLetterListener`, extensiones, `testActor-N`/`testProbe-N` del testkit). Los actores de sistema cuelgan de `/system`, que se detiene sólo después de que `/user` haya terminado.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:771-778 (declaración y Scaladoc), 924-925 (implementación)
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:666-674 (`systemService` se pasa a `provider.actorOf`), 586-599 (`systemGuardian`)
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:188-198 (`testActor` vía `systemActorOf`)
- specs: actor-tests/src/test/scala/org/apache/pekko/routing/ConfiguredLocalRoutingSpec.scala:196-207; actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:167-189

**Precondiciones.** El sistema ha pasado `provider.init` (el guardián de sistema existe). Se puede invocar durante `start` (lo hace el arranque para el `deadLetterListener`).

**Comportamiento.** `systemGuardian.underlying.attachChild(props, name, systemService = true)` (ActorSystem.scala:924-925): reserva el nombre bajo `/system`, crea el actor con la ruta `/system/<name>` y lo arranca. Nombre inválido o duplicado ⇒ las mismas reglas que `actorOf` (AC-REF-009.1 ; AC-PATH-008). A diferencia de `system.actorOf`, no está bloqueado por `guardianProps` custom (ActorSystem.scala:927-937 sólo afecta a `actorOf`).

**Errores.** `InvalidActorNameException` por nombre inválido/duplicado; `ConfigurationException` por dispatcher/mailbox inexistente (AC-SYS-016). El tipo de retorno es `ActorRef` (no `Option`).

**Orden / concurrencia / ciclo de vida.** Un actor bajo `/system` recibe `Terminate` al detenerse `/system`, que sólo ocurre tras `Terminated(guardian)` y tras completar los `TerminationHook` registrados (AC-SYS-003.2). Por eso los loggers siguen vivos mientras `/user` termina.

**Dependencias.** Filas: AC-SYS-003.1 ; AC-SYS-016 ; AC-REF-009.1 ; AC-PATH-008 ; AC-TK-001.1. Externas: ninguna.

**Equivalente Rust propuesto.** `system.system_actor_of(props, name) -> Result<ActorRef, SpawnError>`, expuesto en un trait de extensión `ExtendedActorSystem` (modo `#[doc(hidden)]` o feature `unstable`), porque en Pekko sólo es accesible tras un cast a `ExtendedActorSystem` y su uso previsto es de librerías. Viabilidad alta.

**Diferencias observables.** Rust: método de trait de extensión en vez de downcast; devuelve `Result`. El efecto observable (ruta `/system/<name>`, orden de parada) se conserva.

**Aserciones.**
- A1: un actor creado con `systemActorOf` bajo un padre de sistema puede alojar un router cuyos parámetros salen de la config (`/sys-parent/round` → 6 rutees, `round-robin-pool`), es decir, el hijo de un actor de sistema SÍ aplica deployment por config aunque el actor de sistema no — actor-tests/src/test/scala/org/apache/pekko/routing/ConfiguredLocalRoutingSpec.scala:196-207 (el test está marcado como excepción: "we don't really support deployment configuration of system actors").
- A2: los actores creados por el testkit con `systemActorOf` aparecen en la ruta `pekko://<sistema>/system/testProbe…` (el filtro de log espera `Actor[pekko://LogDeadLetters/system/testProbe.*`) — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:176-182.
- A3: el path de un actor creado con `systemActorOf(props, "x")` es `/system/x` y `parent == systemGuardian` — derivada de ActorSystem.scala:924-925, *no afirmada por spec directa*. Caracterización propuesta: `system_actor_of(p, "x").path() == "/system/x"`.
- A4: `systemActorOf` con nombre duplicado falla con `InvalidActorNameException` — derivada de AC-SYS-016 A1-A2 (misma ruta `attachChild`), *no afirmada por spec para /system*.
- A5: tras `terminate()`, los actores de `/system` se detienen después de los de `/user` — derivada de ActorRefProvider.scala:339-345 (`Terminated(guardian)` dispara `context.become(terminating)`), *no afirmada por spec*; observable en CoordinatedShutdown (AC-CS).

**Validación prevista.** A1-A2: test portado (A2 como aserción del path de un `TestProbe` creado por el harness). A3-A5: caracterización con el árbol de actores del sistema. Diferencial con el jar oficial para A3-A5 (escenario `tools/oracle/sys-system-actor-of.jsh`: crear, imprimir `path`, terminar y registrar el orden de `postStop`): justificado porque el orden de parada es observable en el jar.

**Decisiones.** ninguna específica. Nota de corrección: el inventario citaba ActorSelectionSpec:162,245 para esta fila; esos tests (ActorSelectionSpec.scala:162-177,245-256) seleccionan `/`, `/user` y `/system` y no usan `systemActorOf`; se reasignan a AC-SYS-003.1 / AC-SEL.

## AC-SYS-011 — abort(): apagado forzado sin CoordinatedShutdown

*Unidad SYS · visibilidad internal-required · analysis `extracted`*

**Contrato observable.** [contractual Scaladoc, interno] `ActorSystemImpl.abort()` pone `aborting = true` y llama `terminate()`. Con `aborting`, `terminate()` salta `CoordinatedShutdown` y ejecuta directamente `finalTerminate()`; además, cada actor que se detiene envía a sí mismo `DeathWatchNotification(ref, existenceConfirmed = true, addressTerminated = false)` por cada hijo no local, de modo que no espera la confirmación de hijos desplegados en remoto. Sólo la ruta local es observable en rakka (sin remote).

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1154-1165 (`aborting`, `abort`), 1089-1104 (`terminate`), 1119-1121 (`isTerminating` incluye `aborting`)
- actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala:186-196 (`terminate()` de la celda: rama `systemImpl.aborting`)
- specs: ninguno en pekko-actor (grep de `abort()`/`aborting` sobre actor-tests, testkit y actor/src/test no devuelve usos sobre ActorSystem)

**Precondiciones.** Sistema arrancado. `abort` no es parte de la API pública de `ActorSystem` (sólo `ActorSystemImpl`); lo usa código interno de remote/cluster.

**Comportamiento.**
- Con `aborting == true`: `terminate()` hace `terminating = true` y, como `!aborting` es falso, llama `finalTerminate()` (detener el `deadLetterListener` si `!LogDeadLettersDuringShutdown` y `guardian.stop()`), sin ejecutar ninguna tarea de `CoordinatedShutdown` (ActorSystem.scala:1089-1104,1112-1117).
- `isTerminating()` es `true` desde `abort()` (ActorSystem.scala:1119-1121).
- En `FaultHandling.terminate()`, al detenerse un actor con hijos remotos y `aborting`, se auto-envía `DeathWatchNotification(ref, true, false)` por cada hijo cuyo `ActorRefScope.isLocal` sea falso (FaultHandling.scala:190-196). En un sistema 100 % local no hay tales hijos.

**Errores.** Ninguno propio. `abort()` no lanza; igual que `terminate()` es idempotente.

**Orden / concurrencia / ciclo de vida.** Ver AC-SYS-004. `aborting` es `@volatile`. No hay ventana en la que `CoordinatedShutdown` corra parcialmente: la decisión se toma en la entrada de `terminate()`.

**Dependencias.** Filas: AC-SYS-004 ; AC-CS-006.2 ; AC-DW-012. Externas: ninguna.

**Equivalente Rust propuesto.** `fn abort(&self)` en el trait de extensión interno (no público): marca el sistema y llama a la ruta de terminación sin ejecutar las fases de `CoordinatedShutdown`. La rama de hijos remotos queda sin equivalente por ausencia de remote (frontera explícita: AC-DW-012 cubre `AddressTerminated`).

**Diferencias observables.** Rust: igual para el sistema local. Diferencia: la rama `ActorRefScope.isLocal == false` no es alcanzable sin remote; se documenta, no se implementa.

**Aserciones.**
- A1: tras `abort()`, ninguna tarea registrada en `CoordinatedShutdown` se ejecuta y `whenTerminated` termina — derivada de ActorSystem.scala:1091,1131-1134, *no afirmada por spec*. Caracterización propuesta: registrar una tarea en una fase, llamar `abort()`, esperar `when_terminated` y comprobar que la tarea no corrió.
- A2: `abort()` es idempotente y compatible con una llamada posterior a `terminate()` — derivada de `terminating`/`aborting` volátiles (ActorSystem.scala:1087-1123), *no afirmada por spec*.
- A3: durante `abort()` `isTerminating()` devuelve `true` — derivada de ActorSystem.scala:1119-1121, *no afirmada por spec*.

**Validación prevista.** Caracterización propia para A1-A3 y diferencial con el jar oficial (escenario `tools/oracle/sys-abort.jsh`: `system.asInstanceOf[ActorSystemImpl].abort()` con una tarea registrada en `CoordinatedShutdown`). Justificación: no hay spec upstream; el oráculo fija el comportamiento real.

**Decisiones.** ninguna específica. Nota de corrección: FaultHandling.scala:189-195 del inventario se leyó; el rango real de la rama es 190-196.

## AC-SYS-012 — Accesores: name, settings, startTime, uptime (s), eventStream, log, deadLetters, scheduler, dispatchers, mailboxes, dispatcher; toString = address raíz

*Unidad SYS · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual] Superficie de consulta de `ActorSystem`: `name` (el nombre validado), `settings` (`Settings` + `toString` redactado: AC-SYS-022), `startTime` (ms desde epoch, fijado una vez al construir: `val`), `uptime` (segundos enteros = `(now - startTime) / 1000`, truncado), `eventStream`, `log` (adaptador al eventStream cuyo `logSource` es `"<clase de ActorSystemImpl>(<nombre>)"`), `deadLetters`, `scheduler`, `dispatchers`, `dispatcher` (executor implícito por defecto = `dispatchers.defaultGlobalDispatcher`), `mailboxes`; constructores de rutas `system / name`, `system / iterable`, `child`, `descendant` (ruta bajo `/user`); `logConfiguration()`; `toString` = dirección raíz (`pekko://<nombre>`). El sistema es un handle compartible entre hilos.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:529-640 (declaraciones y Scaladoc de los accesores), 572-577 (`startTime`, `uptime`)
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:937 (`logConfiguration`), 953-998 (eventStream, logFilter, log, scheduler, provider, mailboxes, dispatchers, dispatcher), 1002-1009 (`whenTerminated`, guardianes, `/`), 1262 (`toString`)
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:552-566 (`child`, `descendant`)
- specs: actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:319,412-428; actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:537-576 (usa `system.name` y `system.settings.config`)

**Precondiciones.** Ninguna tras el arranque; durante la construcción los accesores de proveedor pueden fallar (AC-SYS-023).

**Comportamiento.**
- `startTime = System.currentTimeMillis` al instanciar `ActorSystem` (ActorSystem.scala:572); `uptime` crece monótonamente por segundos truncados (ActorSystem.scala:577).
- `log` se construye como `MarkerLoggingAdapter(eventStream, getClass.getName + "(" + name + ")", this.getClass, logFilter)` (ActorSystem.scala:961-963).
- `/(actorName)` = `guardian.path / actorName`; `/(Iterable)` análogo; los nombres no se validan al construir la ruta (ActorSystem.scala:1008-1009).
- `toString = lookupRoot.path.root.address.toString` (ActorSystem.scala:1262), p. ej. `pekko://name`.

**Errores.** Ninguno propio (los accesores son totales tras el arranque).

**Orden / concurrencia / ciclo de vida.** Los accesores son lecturas de campos inmutables (`val`) o de `@volatile`; seguros desde cualquier hilo. `uptime` depende del reloj de pared (no monótono).

**Dependencias.** Filas: AC-SYS-022 ; AC-SYS-018 ; AC-EVT-006 ; AC-DSP-001 ; AC-SCH-001. Externas: reloj de sistema.

**Equivalente Rust propuesto.** Métodos del handle `ActorSystem` (`Clone` sobre `Arc`): `name() -> &str`, `settings() -> &Settings`, `start_time() -> SystemTime`, `uptime() -> Duration` (con `as_secs()` para paridad), `event_stream()`, `log()`, `dead_letters()`, `scheduler()`, `dispatchers()`, `mailboxes()`, `dispatcher()`, `child(&str) -> ActorPath`, `descendant(..)`, `log_configuration()`, `Display` = dirección raíz. Viabilidad alta.

**Diferencias observables.**
- `startTime: Long` (ms) → `SystemTime`; `uptime` en segundos enteros se conserva como `Duration`/`as_secs`.
- `implicit def dispatcher` (ExecutionContext) → `dispatcher()` devuelve un handle de ejecución; la integración con el runtime depende de Q-X-4.
- El `logSource` de `system.log` incluye el nombre de clase JVM de `ActorSystemImpl` (Q-EVT-1): se propone `"rakka::ActorSystemImpl(<nombre>)"` o un nombre de tipo documentado.

**Aserciones.**
- A1: `system.settings.toString` no contiene el nombre de usuario ni el home y contiene `<username>` — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:412-428 (ver AC-SYS-022).
- A2: `uptime` se expresa en segundos (el test espera que un sistema recién creado esté por debajo de 10 s: `system.uptime >= 10` implica fallo) — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:319.
- A3: el nombre y la config de un sistema existente se pueden reutilizar para crear otro (`ActorSystem(system.name, system.settings.config)`) — actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:537 (uso, no aserción de los accesores).
- A4: `toString` del sistema es `pekko://<nombre>` — derivada de ActorSystem.scala:1262 y de `RootActorPath(Address("pekko", name))` (ActorRefProvider.scala:402), *no afirmada por spec*. Caracterización: `assert_eq!(system.to_string(), "pekko://nombre")` (o el esquema que se decida en AC-PATH).
- A5: `uptime()` es `(now - startTime) / 1000` truncado — derivada de ActorSystem.scala:577, *no afirmada por spec*. Caracterización con reloj inyectable.
- A6: `system / "a"` = `/user/a` y `system.child("a")` equivale — derivada de ActorSystem.scala:557,1008, *no afirmada por spec*.
- A7: `startTime` no cambia durante la vida del sistema — derivada de `val startTime` (ActorSystem.scala:572), *no afirmada por spec*.

**Validación prevista.** A1 portado; A2 como parte del test portado de AC-SYS-007; A4-A7 caracterización propia (con reloj de prueba para `uptime`). Diferencial con el jar oficial para A4 y A6 (escenario `tools/oracle/sys-accessors.jsh`), justificado porque el formato de `toString` y de rutas es observable por logs y por `ActorSelection`.

**Decisiones.** Q-EVT-1 (nombre de clase JVM en `log`), Q-X-4 (`dispatcher`), Q-SYS-5 (efecto sobre `settings.toString`).

## AC-SYS-013 — ExtendedActorSystem: provider, guardian, systemGuardian, systemActorOf, threadFactory, dynamicAccess, printTree

*Unidad SYS · visibilidad internal-required · analysis `extracted`*

**Contrato observable.** [contractual para extensiones, `@DoNotInherit`] `ExtendedActorSystem` añade a `ActorSystem` la superficie para integradores: `provider`, `guardian`, `systemGuardian`, `systemActorOf` (AC-SYS-010), `threadFactory`, `dynamicAccess`, `logFilter`, y los internos `printTree`, `uid`, `finalTerminate`, `isTerminating`. `printTree` recorre la jerarquía desde la raíz y devuelve un texto indentado; sólo se usa para diagnóstico (`ActorSystemSpec` lo imprime si el apagado falla y `TestKit.shutdownActorSystem` lo loguea en caso de timeout).

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:753-822 (declaración de `ExtendedActorSystem`)
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:836 (`uid`), 909-918 (`threadFactory`, `dynamicAccess`), 956-959 (`logFilter`), 1264-1301 (`printTree`)
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:1027 (uso de `printTree`)
- specs: actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:319-323; actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorHierarchySpec.scala:565,639 (sólo imprimen)

**Precondiciones.** Obtención por downcast `system.asInstanceOf[ExtendedActorSystem]` (en Pekko Classic) o por `ExtensionId.createExtension(system: ExtendedActorSystem)`.

**Comportamiento.**
- `printTree`: por nodo imprime `"-> " | indent+"⌊-> "`, `path.name`, `simpleName(ref)`, clase del actor (`real.actor.getClass` o `"null"`), `" status=" + mailbox.currentStatus`, el estado del contenedor de hijos (`Terminating(reason) + toDie`, `Terminated`/`EmptyChildren`, `N children`) y recursa por los hijos ordenados con prefijo `|` / espacios (ActorSystem.scala:1264-1301). Para refs sin celda imprime `indent + name + simpleName`.
- `threadFactory` es `MonitorableThreadFactory(name, Daemonicity, classLoader, uncaughtExceptionHandler)` (ActorSystem.scala:909-910): los hilos llevan el nombre del sistema (observable en DispatchersSpec.scala:230-260, fila AC-DSP).
- `uid` es un `Long` aleatorio fijado al construir (ActorSystem.scala:838).

**Errores.** Ninguno propio. `printTree` puede lanzar si la jerarquía cambia mientras se recorre (no es contractual: el Scaladoc advierte de OOM/inestabilidad, ActorSystem.scala:801-806).

**Orden / concurrencia / ciclo de vida.** `printTree` no es atómico (instantánea no consistente). `threadFactory` se usa durante la construcción de dispatchers y scheduler (antes de `start`).

**Dependencias.** Filas: AC-SYS-010 ; AC-SYS-018 ; AC-SYS-023 ; AC-DSP-001 ; AC-DSP-015 ; AC-EXT-001.2. Externas: ninguna.

**Equivalente Rust propuesto.** Trait de extensión `ExtendedActorSystem` (feature `unstable`) con `provider()`, `guardian()`, `system_guardian()`, `system_actor_of(..)`, `log_filter()`, `uid()`, `print_tree() -> String`. `thread_factory`/`dynamic_access` no tienen equivalente directo (Q-SYS-6): la creación de hilos pasa por el runtime elegido (Q-X-4) y la reflexión por el registro de fábricas (Q-X-2). Viabilidad: alta para todo salvo los dos últimos.

**Diferencias observables.** `print_tree()` tiene formato propio (no se compara textualmente con Pekko: ningún test lo afirma). Sin `threadFactory`/`dynamicAccess`.

**Aserciones.**
- A1: `printTree` no falla sobre un sistema vivo y su salida se usa sólo como diagnóstico del fallo `"System didn't terminate within 5 seconds"` — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:319-323 (se ejecuta únicamente en la rama de fallo; no hay aserción sobre el texto).
- A2: `printTree` se invoca en SupervisorHierarchySpec en los diagnósticos de fallo — actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorHierarchySpec.scala:565,639 (sin aserción sobre el texto).
- A3: el texto de `printTree` empieza con `-> ` para la raíz y marca cada hijo con `⌊-> ` — derivada de ActorSystem.scala:1269, *no afirmada por spec*. Caracterización: el propio formato de rakka (nombre, tipo, nº de hijos), no la cadena de Pekko.
- A4: `uid` es distinto entre sistemas — derivada de `ThreadLocalRandom` (ActorSystem.scala:838), *no afirmada por spec*.

**Validación prevista.** A1-A2: no portables como aserción (sólo diagnósticos); el equivalente es un test de humo `print_tree()` no entra en pánico con un árbol de 3 niveles. A3-A4: caracterización propia. Sin oráculo: el texto de `printTree` depende de clases JVM (`getClass`, `currentStatus`), justificado.

**Decisiones.** Q-SYS-6 (efecto: `threadFactory` y `dynamicAccess` se excluyen de la superficie pública).

## AC-SYS-014 — close() (AutoCloseable): terminate + espera hasta close-actor-system-timeout

*Unidad SYS · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual] Bloquea hasta whenTerminated o vence pekko.coordinated-shutdown.close-actor-system-timeout (Await.result lanza TimeoutException al vencer). `Using(ActorSystem()){..}` termina el sistema al salir del bloque; close() deja whenTerminated completo.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1137-1141
- reference.conf:1257 (close-actor-system-timeout)

**Condiciones.**
- Timeout vencido: Await.result lanza java.util.concurrent.TimeoutException (derivado de scala.concurrent.Await).

**Orden / concurrencia / ciclo de vida.** terminate() primero, luego bloqueo en whenTerminated.

**Dependencias.** Filas: AC-SYS-004 ; AC-CS-009. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** método explícito `close_blocking(&self)`; sin `Drop` que termine (Q-SYS-1)

**Diferencias observables.** Rust: `close_blocking` o `async fn close`. Diferencia: AutoCloseable no existe; el equivalente idiomático es una guarda `SystemGuard` opcional (Q-SYS-1).

**Aserciones.**
- A1: tras `close()`, registerOnTermination lanza RejectedExecutionException — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:266-273.
- A2: `Using(ActorSystem()) { s => .. }` deja `whenTerminated.isCompleted==true` — actor-tests/src/test/scala-2.13+/org/apache/pekko/actor/ActorSystemScala213PlusSpec.scala:29-35.

**Oráculo.** Jar oficial: `sys-terminate.jsh` (close en vez de terminate).

**Decisiones.** Q-SYS-1

## AC-SYS-015 — Errores fatales no capturados: jvm-exit-on-fatal-error → System.exit(-1); si no, terminate()

*Unidad SYS · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual JVM] El UncaughtExceptionHandler de los hilos del sistema: errores NonFatal/InterruptedException/NotImplementedError/ControlThrowable se loguean ('Uncaught error from thread [{}]'); cualquier otro es fatal: imprime a stderr, loguea con marker SECURITY y o System.exit(-1) (si jvm-exit-on-fatal-error) o terminate(). Un hilo del sistema que muere con error fatal apaga el proceso (o el sistema).

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:875-923
- reference.conf:90 (jvm-exit-on-fatal-error = on)

**Condiciones.**
- No fatal: `log.error(cause, "Uncaught error from thread [{}]", thread.getName)` (actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:879).
- Fatal: stderr `Uncaught error from thread [<t>]: <msg>, <message> ActorSystem[<name>]` + stacktrace, luego log con LogMarker.Security (actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:900-923); `message` = `shutting down JVM since 'pekko.jvm-exit-on-fatal-error' is enabled for` o `shutting down`.
- Si IncompatibleClassChangeError con mensaje que empieza con org.apache.pekko: aviso a stderr sobre versiones mezcladas (actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:882-890).

**Orden / concurrencia / ciclo de vida.** Corre en el hilo que murió.

**Dependencias.** Filas: AC-CFG-007 ; AC-EVT-015. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** hook de panic en hilos propios + política configurable

**Diferencias observables.** Rust: `panic` en tarea de actor ya es 'fallo' (Q-X-5); 'fatal' = abort/OOM no capturable; propuesta: política `FatalErrorPolicy::{ExitProcess, TerminateSystem}` aplicada a panics fuera de actores y a un handler de señal; diferencia observable: sin IncompatibleClassChangeError.

**Aserciones.**
- A1: un error NonFatal en un hilo del sistema sólo se loguea — derivada de actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:879-880, *no afirmada por spec*.
- A2: con jvm-exit-on-fatal-error=on el proceso sale con código -1 — derivada de actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:894-896, *no afirmada por spec*.
- A3: con jvm-exit-on-fatal-error=off se llama terminate() — derivada de actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:897-899, *no afirmada por spec*.

**Oráculo.** Parcial: jar oficial con un hilo del threadFactory del sistema que lanza OutOfMemoryError simulado y `jvm-exit-on-fatal-error=off`; escenario propuesto `sys-fatal-error.jsh`.

**Decisiones.** Q-SYS-2, Q-X-5

## AC-SYS-016 — LocalActorRefProvider.actorOf: merge de Deploy, ruta router vs no-router, lookup de dispatcher y mailbox, LocalActorRef vs RepointableActorRef vs RoutedActorRef

*Unidad SYS · visibilidad internal-required · analysis `extracted`*

**Contrato observable.** [contractual por spec] `provider.actorOf(system, props, supervisor, path, systemService, deploy, lookupDeploy, async)` (create-only) resuelve `Props` + `Deploy` (código vs config `pekko.actor.deployment`) a un `InternalActorRef`: sin router, el `Deploy` de config (si `lookupDeploy`) o el pasado (si no) pisa dispatcher y mailbox de los `Props` y `DispatcherSameAsParent` hereda el dispatcher del padre; con router se fusionan los `routerConfig` (`router :: deploy :: lookup` reducidos con `withFallback`) y se crea un `RoutedActorRef`. Los errores de configuración de dispatcher o mailbox se convierten en `ConfigurationException`. Con `async` se crea un `RepointableActorRef` (inicialización asíncrona, `UnstartedCell`); si no, un `LocalActorRef`. Dos creadores concurrentes del mismo nombre: exactamente uno gana y el otro recibe `InvalidActorNameException`.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:666-764 (`actorOf`), 123-139 (contrato de `systemService`/`deploy`/`lookupDeploy`)
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:941-942 (llamada con `systemService`)
- reference.conf: `pekko.actor.deployment`, `pekko.actor.debug.router-misconfiguration`, `pekko.actor.default-dispatcher`, `pekko.actor.default-mailbox`
- specs: actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:131-197; actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorSpec.scala:109-127,510-546

**Precondiciones.** `props.deploy.routerConfig` define la rama (`NoRouter` ↔ no router). `systemService = true` omite el `Deployer` (documentado en ActorRefProvider.scala:123-129; ver AC-SYS-010).

**Comportamiento.**
- Rama `NoRouter`: si `debug.router-misconfiguration`, avisa cuando la config declara un router y el código no (`log.warning("Configuration says that [{}] should be a router, but code disagrees. …")`: ActorRefProvider.scala:670-677). `props2` se calcula con la tabla `(d.dispatcher, d.mailbox)` de ActorRefProvider.scala:684-702: `NoDispatcherGiven`/`NoMailboxGiven` mantienen `props`; `DispatcherSameAsParent` usa el dispatcher del padre (`parentDispatcher`: ActorRefProvider.scala:679-682); sin deploy y con `DispatcherSameAsParent` en los props también hereda.
- Se verifica `dispatchers.hasDispatcher(props2.dispatcher)` (ActorRefProvider.scala:704-705); se obtiene dispatcher y `MailboxType` (ActorRefProvider.scala:707-709); `async` ⇒ `RepointableActorRef(..).initialize(async)`, si no ⇒ `LocalActorRef` (ActorRefProvider.scala:711-713).
- Rama router: el `routerConfig` final es `(router :: deploy.routerConfig :: lookup.routerConfig).reduce((a, b) => b.withFallback(a))` (ActorRefProvider.scala:721-725); se validan dispatcher de rutees y del router (727-730); `routerProps` usa `RoutedActorCell.RouterActorCreator` y se verifican de antemano los mailboxes de router y rutees (732-745); se construye `RoutedActorRef(..).initialize(async)` (747-748).

**Errores.**
- Dispatcher inexistente (no router): `ConfigurationException("Dispatcher [<id>] not configured for path <path>")` (ActorRefProvider.scala:704-705).
- Dispatcher de rutees/router inexistente: `ConfigurationException("Dispatcher [..] not configured for routees of <path>")` / `"… for router of <path>"` (ActorRefProvider.scala:727-730).
- Cualquier `NonFatal` al crear dispatcher, mailbox o la ref: `ConfigurationException("configuration problem while creating [<path>] with dispatcher [<d>] and mailbox [<m>]", cause)`; en el caso router el texto incluye "router dispatcher/mailbox and routee dispatcher/mailbox" (ActorRefProvider.scala:714-718,750-754). Nota: la causa puede ser un `AssertionError` (es `NonFatal` en Scala).
- Nombre mal formado o duplicado: `InvalidActorNameException` (mensajes fijados por AC-PATH).

**Orden / concurrencia / ciclo de vida.** La reserva atómica del nombre (`reserveChild`) hace que, entre N creadores concurrentes del mismo nombre, uno tenga éxito y los demás `InvalidActorNameException` (LocalActorRefProviderSpec.scala:134-154). Dentro de un mismo mensaje, el segundo `actorOf` con el mismo nombre falla (LocalActorRefProviderSpec.scala:156-166).

**Dependencias.** Filas: AC-DSP-007 ; AC-MBX-013 ; AC-RT-003.1 ; AC-REF-009.1 ; AC-PATH-008 ; AC-SUP-014. Externas: Typesafe Config.

**Equivalente Rust propuesto.** `provider::local::spawn(system, props, supervisor, path, SpawnFlags{system_service, async_init, lookup_deploy}, deploy_override) -> Result<ActorRef, SpawnError>`; errores `SpawnError::{InvalidName, Configuration{path, dispatcher, mailbox, source}}`. Los routers se crean por la misma función con la fusión de `RouterConfig` (contrato de routers: AC-RT). Viabilidad alta; la reserva atómica de nombre se implementa con un `DashMap`/`Mutex<HashMap>` en el contenedor de hijos.

**Diferencias observables.** `Err(SpawnError)` en lugar de excepciones (Q-X-5); `Dispatcher`/`Mailbox` por registro de fábricas, no por FQCN (Q-X-2); la causa `AssertionError` del test se sustituye por un error cualquiera devuelto por la fábrica o un pánico capturado en la creación.

**Aserciones.**
- A1: 4 creaciones concurrentes de `new-actorN` con el mismo nombre producen exactamente {éxito, `InvalidActorNameException`} (conjunto de resultados `Set(1, 2)`), repetido 100 veces — actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:133-153.
- A2: dos `actorOf(.., "duplicate")` en el mismo mensaje producen exactamente 1 `InvalidActorNameException` — actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:155-165.
- A3: nombres mal formados: `null`→mensaje con "null", `""`→"empty", `"$hallo"`→"not start with `$`", `"a%"`/`"%3"`/`"%xx"`/`"%0G"`/`"%gg"`/`"%"`/`"%1t"`/`"a?"`→"Invalid actor path element", `"üß"`→"include only ASCII", `he"llo`→`["] at position: 2`, `$hello`→`[$] at position: 0`, `hell>o`→`[>] at position: 4` — actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:167-197.
- A4: un `MailboxType` cuyo `create` lanza hace que un actor hijo (no top-level) falle en `context.actorOf` con `ConfigurationException` cuya `getCause` es la excepción original — actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorSpec.scala:518-527 (el mailbox `error-mailbox` se define en SupervisorSpec.scala:115-125 y `creator(.., fail = true)` lo aplica en :108-111).
- A5: lo mismo con un router hijo (`creator(..).withRouter(RoundRobinPool(1))`): `ConfigurationException` con `getCause == failure` — actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorSpec.scala:535-544.
- A6: para un actor top-level con la misma falla, el error se manifiesta como `ActorInitializationException` logueado una vez y el actor termina (`expectTerminated`) tanto sin router como con `RoundRobinPool(1)` — actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorSpec.scala:512-516 y 527-531 (casos "creating a top-level actor" / "creating a top-level router").
- A7: dispatcher inexistente en los `Props` falla con `ConfigurationException("Dispatcher [x] not configured for path <path>")` — derivada de ActorRefProvider.scala:704-705, *no afirmada por spec* (ActorModelSpec/DispatchersSpec cubren la parte de dispatchers: AC-DSP). Caracterización propuesta.
- A8: `debug.router-misconfiguration = on` avisa cuando la config declara un router y el código no — derivada de ActorRefProvider.scala:670-677, *no afirmada por spec*. Caracterización propuesta con `EventFilter.warning`.
- A9: la tabla de fusión `(dispatcher, mailbox)` del `Deploy` produce, p. ej., `(SameAsParent, NoMailboxGiven)` ⇒ `props.withDispatcher(parentDispatcher)` — derivada de ActorRefProvider.scala:689-694, *no afirmada por spec* (los casos de `SameAsParent` se prueban indirectamente en DispatchersSpec).

**Validación prevista.** A1-A6: tests portados 1:1 (A4-A6 con un `MailboxType` de prueba que falla en `create`). A7-A9: caracterización propia con las seis filas de la tabla de fusión; diferencial con el jar oficial para los mensajes de A3 y A7 (escenario `tools/oracle/sys-actorof-errors.jsh`): los textos son observables y en parte contractuales (`include(..)` en los specs).

**Decisiones.** Q-X-2 (fábricas de dispatcher/mailbox por nombre), Q-X-5 (Err vs excepción).

## AC-SYS-017 — Actores temporales: tempPath, tempContainer, registerTempActor/unregisterTempActor

*Unidad SYS · visibilidad internal-required · analysis `extracted`*

**Contrato observable.** [contractual interno] El proveedor local genera rutas únicas bajo `/temp` para refs de corta vida (cada `ask` crea una `PromiseActorRef` registrada ahí): `tempPath(prefix) = rootPath/temp/<prefix>$<base64(contador)>` con un `AtomicLong` creciente (primer nombre `$a`, luego `$b`…). `tempContainer` es un `VirtualPathContainer` (hijo virtual de la raíz) que mantiene un mapa concurrente nombre→ref y es resoluble por path mientras viva el ref. `registerTempActor` y `unregisterTempActor` exigen (assert) que el padre del path sea `/temp`.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:97-121 (contrato en el trait), 425-442 (`tempNumber`, `tempNode`, `tempPath`), 601-611 (`tempContainer`, register/unregister)
- actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:735-828 (`VirtualPathContainer`)
- specs: actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:248-266,268-289

**Precondiciones.** Los paths pasados a `register/unregisterTempActor` DEBEN venir de `tempPath()` (documentado; se comprueba con `assert(path.parent eq tempNode, ..)`: ActorRefProvider.scala:604,609).

**Comportamiento.**
- `tempPath(prefix)`: `builder = prefix?; "$" + Helpers.base64(tempNumber.getAndIncrement())` (ActorRefProvider.scala:434-442). El alfabeto base64 es el de AC-UTIL-002.1.
- `VirtualPathContainer`: mapa `ConcurrentHashMap[String, InternalActorRef]` (ActorRef.scala:770). `addChild(name, ref)`: si reemplaza un valor previo, loguea `debug("{} replacing child {} ({} -> {})")` y detiene el antiguo (ActorRef.scala:807-816). `removeChild(name)`: si no existía, `warning("{} trying to remove non-child {}")` (ActorRef.scala:818-819). `getChild(Iterator)`: iterador vacío o nombre vacío ⇒ el propio contenedor; nombre ausente ⇒ `Nobody`; si quedan más elementos delega en `child.getChild` (ActorRef.scala:835-848). `hasChildren`/`foreachChild` leen una vista débilmente consistente (ActorRef.scala:850-855).
- Mensajes `ActorSelectionMessage` enviados al contenedor: sólo se admite `SelectChildName` en el primer elemento; si el hijo existe y es el último elemento, se le envía el mensaje; si no existe o quedan elementos o no es `SelectChildName`, se reenvía a un `EmptyLocalActorRef` salvo `wildcardFanOut` (entonces se descarta) (ActorRef.scala:776-805). Otros mensajes: `super.!` (ruta a dead letters).
- Derivación de nombre en ask: el nombre del ref temporal incluye el nombre del destino (`myName…`) o, para un `ActorSelection`, `_user_myName…` (AskSpec.scala:248-266); ver AC-ASK-002.

**Errores.** `assert` fallido (`AssertionError`, "cannot registerTempActor() with anything not obtained from tempPath()") si el path no es de `/temp`. `require(elements.nonEmpty)` en el manejo de `ActorSelectionMessage`.

**Orden / concurrencia / ciclo de vida.** El contador es atómico: nombres únicos entre hilos. `addChild`/`removeChild` son atómicos por clave. Un ref desregistrado sigue siendo válido pero deja de resolverse por path (cae a `EmptyLocalActorRef`/dead letters).

**Dependencias.** Filas: AC-ASK-002 ; AC-UTIL-002.1 ; AC-PATH-015 ; AC-REF-011. Externas: ninguna.

**Equivalente Rust propuesto.** `TempContainer { children: DashMap<Arc<str>, ActorRef>, counter: AtomicU64 }` con `temp_path(prefix) -> ActorPath`, `register_temp(ref, path)`, `unregister_temp(path)`; `debug_assert!` en lugar de `assert`. Viabilidad alta.

**Diferencias observables.** `AssertionError` → `debug_assert!`/`Err` (Q-X-5); el resto igual.

**Aserciones.**
- A1: el nombre del ref temporal de un `ask` a un actor de nombre `myName` empieza por `myName` y, para un `ask` a `actorSelection("/user/myName")`, por `_user_myName` — actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:248-266.
- A2: cuando el actor responde al ref temporal, el `ask` completa con `"complete"` y el path del ref temporal (ya terminado) sigue conteniendo el nombre del destino (`pathPrefix`) — actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:268-289 (test "proper path when promise actor terminated"; aserciones en :286 y :288).
- A3: `tempPath()` genera nombres únicos `$..` bajo `/temp`, empezando en `$a` — derivada de ActorRefProvider.scala:428-442 y de AC-UTIL-002.1, *no afirmada por spec directa*. Caracterización: tres llamadas consecutivas producen `/temp/$a`, `/temp/$b`, `/temp/$c` en un sistema nuevo.
- A4: un ref temporal registrado es resoluble por path (`resolveActorRef(path)` / `ActorSelection`) y deja de serlo tras `unregisterTempActor` — derivada de ActorRefProvider.scala:603-611 y ActorRef.scala:807-848, *no afirmada por spec*.
- A5: `registerTempActor` con un path que no es de `/temp` falla el assert — derivada de ActorRefProvider.scala:604, *no afirmada por spec*.
- A6: `addChild` sobre un nombre existente detiene el ref anterior — derivada de ActorRef.scala:807-816, *no afirmada por spec*.

**Validación prevista.** A1-A2: tests portados dentro de la unidad ASK. A3-A6: caracterización propia; diferencial con el jar oficial para A3 (secuencia de `tempPath`) en `tools/oracle/sys-temp-paths.jsh`, justificado porque los nombres temporales aparecen en logs y dead letters.

**Decisiones.** ninguna específica.

## AC-SYS-018 — SPI ActorRefProvider (@DoNotInherit): rootGuardian, guardian, systemGuardian, deadLetters, ignoreRef, rootPath, deployer, resolveActorRef, terminationFuture, getExternalAddressFor, getDefaultAddress, serializationInformation, addressString

*Unidad SYS · visibilidad internal-required · analysis `extracted`*

**Contrato observable.** [contractual para remote/cluster; `@DoNotInherit`] Interfaz del proveedor de refs: `rootGuardian`, `rootGuardianAt(address)`, `guardian` (/user), `systemGuardian` (/system), `deadLetters`, `ignoreRef`, `rootPath`, `settings`, `init(system)`, `deployer`, `tempPath*`/`tempContainer`/`register/unregisterTempActor` (AC-SYS-017), `actorOf` (AC-SYS-016), `resolveActorRef` (AC-SYS-024), `terminationFuture` (AC-SYS-019), `getExternalAddressFor`, `getDefaultAddress`, `serializationInformation`, `addressString`. En el proveedor local: `getExternalAddressFor(addr) = Some(addr)` sólo si `addr == rootPath.address`, si no `None`; `getDefaultAddress = rootPath.address`; `rootGuardianAt(address)` = `rootGuardian` si coincide con la dirección raíz, si no `deadLetters`; `addressString` es `getDefaultAddress.toString` cacheado. `ProviderSelection` (local/remote/cluster/custom) decide qué proveedor se instancia.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:41-188 (trait), 554-556 (`rootGuardianAt`), 759-761 (`getExternalAddressFor`, `getDefaultAddress`), 763-778 (`serializationInformation`), 780-791 (`addressString`)
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:102-144 (`ProviderSelection`), 408-420 (selección en `Settings`)
- actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:513-534 (`IgnoreActorRef.path` = `pekko://local/ignore`)
- reference.conf: `pekko.actor.provider`
- specs: actor-tests/src/test/scala/org/apache/pekko/actor/ProviderSelectionSpec.scala:38-67

**Precondiciones.** El proveedor se instancia por FQCN con `(systemName, settings, eventStream, dynamicAccess)` (ActorSystem.scala:967-975) y se inicializa con `init(system)` (AC-SYS-002). Antes de `init`, `serializationInformation` lanza `IllegalStateException("Too early access of serializationInformation")` si aún no hay `system` (ActorRefProvider.scala:770-771).

**Comportamiento.**
- `ProviderSelection.apply(String)`: `"local"` → `Local`; `"remote"` o FQCN de `RemoteActorRefProvider` → `Remote`; `"cluster"` o FQCN de `ClusterActorRefProvider` → `Cluster`; cualquier otro → `Custom(fqcn)` (ActorSystem.scala:137-144). `hasCluster` sólo es `true` para `Cluster`. `Settings.ProviderClass` es el `fqcn` seleccionado (ActorSystem.scala:416-418).
- `rootGuardianAt`, `getExternalAddressFor` y `getDefaultAddress` locales según arriba.
- `ignoreRef`: ref con path `pekko://local/ignore` (nombre de sistema ficticio `local`), que descarta todo (ActorRef.scala:541-548; comportamiento en AC-REF).
- `serializationInformation` se cachea una vez con `Serialization.Information(rootPath.address, system)` (ActorRefProvider.scala:764-777).

**Errores.** `IllegalStateException` en `serializationInformation` demasiado pronto. Un FQCN que no se pueda instanciar hace fallar la construcción del sistema (`Try(stopScheduler())` y relanza: ActorSystem.scala:976-979).

**Orden / concurrencia / ciclo de vida.** `init` ocurre una vez desde `start`. Los campos `system` y `extraNames` son `@volatile`; los guardianes son `lazy val`.

**Dependencias.** Filas: AC-CFG-008 ; AC-CFG-015 ; AC-SYS-016 ; AC-SYS-017 ; AC-SYS-019 ; AC-SYS-024 ; AC-SYS-025 ; AC-SER-001.3. Externas: ninguna.

**Equivalente Rust propuesto.** Trait sellado `ActorRefProvider` con una implementación `LocalActorRefProvider` y un punto de extensión interno (`pub(crate)` o feature `provider-spi`) para futuros proveedores remotos; `enum ProviderSelection { Local, Remote, Cluster, Custom(String) }` con el mismo mapeo de strings. `Custom(name)` se resuelve contra un registro de proveedores (Q-X-2). Viabilidad alta para el proveedor local; remote/cluster fuera de alcance con frontera explícita (el trait debe poder alojarlos).

**Diferencias observables.** El FQCN de clase JVM de `Local/Remote/Cluster` se reemplaza por identificadores propios; `ProviderSelection("remote")` y el FQCN siguen siendo equivalentes mediante una tabla de alias. `@DoNotInherit` se sustituye por un trait sellado.

**Aserciones.**
- A1: `ProviderSelection.Local.fqcn` es el nombre de clase de `LocalActorRefProvider`, `hasCluster` es `false` y `Settings` con `provider = "local"` expone el mismo `ProviderClass` — actor-tests/src/test/scala/org/apache/pekko/actor/ProviderSelectionSpec.scala:38-43.
- A2: `ProviderSelection.Remote.fqcn` es `org.apache.pekko.remote.RemoteActorRefProvider`, `hasCluster` es `false`, `ProviderSelection("remote") == ProviderSelection(RemoteActorRefProvider)` y `Settings` lo refleja — actor-tests/src/test/scala/org/apache/pekko/actor/ProviderSelectionSpec.scala:45-51.
- A3: `ProviderSelection.Cluster` análogo con `hasCluster == true` — actor-tests/src/test/scala/org/apache/pekko/actor/ProviderSelectionSpec.scala:53-59.
- A4: `ProviderSelection.Custom(other)` conserva el FQCN, `hasCluster == false` y `Settings.ProviderClass` lo devuelve — actor-tests/src/test/scala/org/apache/pekko/actor/ProviderSelectionSpec.scala:61-67.
- A5: `getExternalAddressFor(addr)` local devuelve `Some(addr)` sólo para la dirección raíz — derivada de ActorRefProvider.scala:759, *no afirmada por spec*. Caracterización propuesta.
- A6: `rootGuardianAt(address)` devuelve el guardián raíz sólo para la dirección propia y `deadLetters` en otro caso — derivada de ActorRefProvider.scala:554-556, *no afirmada por spec*.
- A7: `getDefaultAddress == rootPath.address` y `addressString == getDefaultAddress.toString` — derivada de ActorRefProvider.scala:761,783-790, *no afirmada por spec*.
- A8: `ignoreRef.path == "pekko://local/ignore"` — derivada de ActorRef.scala:543-546, *no afirmada por spec aquí* (AC-PATH-015 y AC-REF).

**Validación prevista.** A1-A4: tests portados 1:1 (sin FQCN reales; con los identificadores de rakka). A5-A8: caracterización propia; diferencial con el jar oficial para A5-A7 en `tools/oracle/sys-provider-addresses.jsh` (justificado: sin spec y observable por remote/cluster y `serializationInformation`).

**Decisiones.** Q-X-2 (proveedores por registro), Q-SYS-6 (efecto: `DynamicAccess` no forma parte de la firma del constructor del proveedor).

## AC-SYS-019 — terminationFuture del proveedor y 'bubble-walker' (theOneWhoWalksTheBubblesOfSpaceTime)

*Unidad SYS · visibilidad internal-required · analysis `extracted`*

**Contrato observable.** [contractual interno] El padre del guardián raíz es un ref mínimo en /bubble-walker: recibe Failed de la raíz (loguea 'guardian <child> failed, shutting down!' y la detiene), ignora Supervise, y ante DeathWatchNotification se detiene y completa terminationFuture con Terminated(rootGuardian)(true,true). Es el 'último' ref: su stop() marca la terminación.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:428-430
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:455-494

**Condiciones.**
- Mensaje normal mientras camina: `log.error(s"$this received unexpected message [$message]")`; null -> InvalidMessageException("Message is null") (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:476-481).
- SystemMessage inesperado: error `$this received unexpected system message [$message]` (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:491).

**Orden / concurrencia / ciclo de vida.** stop() idempotente (trySuccess).

**Dependencias.** Filas: AC-SYS-005. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** oneshot interno + ref mínimo

**Diferencias observables.** Rust: igual.

**Aserciones.**
- A1: Terminated devuelto por terminate() tiene actor=rootGuardian y ambos flags true — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:260-262.
- A2: Failed del guardián raíz -> error 'guardian .. failed, shutting down!' y stop del hijo — derivada de actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:485-488, *no afirmada por spec*.
- A3: stop() es idempotente — derivada de actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:464-468.

**Oráculo.** Jar oficial: `sys-terminate.jsh`.

**Decisiones.** ninguna específica

## AC-SYS-020 — ManifestInfo: chequeo de versiones mezcladas de módulos Pekko

*Unidad SYS · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec para `checkSameVersion`; JVM-específico para la lectura de MANIFEST] `ManifestInfo` es una extensión que lee `META-INF/MANIFEST.MF` de todos los jars del classpath y registra la versión (`Implementation-Version`, o `Bundle-Version`) de los artefactos con vendor `org.apache.pekko` (`Implementation-Vendor-Id`/`Bundle-Vendor`), indexada por título (`Implementation-Title`/`Bundle-Name`). `checkSameVersion(productName, dependencies, versions)` devuelve `Some(mensaje)` si las versiones de los módulos listados no son idénticas y `None` si coinciden; el arranque del sistema lo invoca con `logWarning = true` y, si `pekko.fail-mixed-versions = on`, lanza `IllegalStateException(message)`.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/util/ManifestInfo.scala:33-98 (objeto: constantes, `Version`, `checkSameVersion` puro), 110-194 (extensión: lectura de manifests y `checkSameVersion` con log/throw)
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1044-1076 (lista `allModules`), 1071 (invocación en `start`), 472 (`FailMixedVersions`)
- reference.conf: `pekko.fail-mixed-versions`
- specs: actor-tests/src/test/scala/org/apache/pekko/util/ManifestInfoSpec.scala:19-47

**Precondiciones.** Sólo se consideran versiones cuyo vendor está en `knownVendors = Set("org.apache.pekko")` y que tengan título, versión y vendor no nulos (ManifestInfo.scala:140-144).

**Comportamiento.**
- `Version(version: String)` envuelve `pekko.util.Version` (comparable, `equals`, `hashCode`, `toString`: ManifestInfo.scala:58-74; su orden se especifica en AC-UTIL).
- `checkSameVersion` puro: filtra `versions` a las claves en `dependencies`; si el conjunto de valores distintos tiene más de uno, calcula `highestVersion = values.max`, `toBeUpdated` = módulos con versión distinta de la mayor (en orden del mapa) y `groupedByVersion` = pares `(versión, [módulos ordenados])` ordenados por versión, y devuelve el mensaje fijo `"You are using version <max> of <product>, but it appears you (perhaps indirectly) also depend on older versions of related artifacts. You can solve this by adding an explicit dependency on version <max> of the [<toBeUpdated>] artifacts to your project. Here's a complete collection of detected artifacts: <grouped>. See also: https://pekko.apache.org/docs/pekko/current/common/binary-compatibility-rules.html#mixed-versioning-is-not-allowed"` (ManifestInfo.scala:77-102).
- Versión "dynver": `1.0.2+10-abababef` > `1.0.2` (ManifestInfoSpec.scala:34-44).
- Extensión: `checkSameVersion(product, deps, logWarning)` usa `throwException = settings.FailMixedVersions`; con `logWarning` loguea `warning(message)`; con `throwException` lanza `IllegalStateException(message)`; devuelve `true` sólo si no hay discrepancia (ManifestInfo.scala:165-193).
- La lectura de manifests tolera `IOException` con `warning("Could not read manifest information. {}", ioe)` (ManifestInfo.scala:150-153).

**Errores.** `IllegalStateException(message)` si `throwException` y hay discrepancia. Sin discrepancia no hay error ni log.

**Orden / concurrencia / ciclo de vida.** Se ejecuta una vez en `start` (después del `startUnsubscriber`, antes de `loadExtensions`). Es una extensión: se instancia por sistema. La lectura de `versions` es un `val` calculado al crear la extensión.

**Dependencias.** Filas: AC-CFG-007 ; AC-UTIL-005.2 ; AC-UTIL-016.9 ; AC-EXT-001.2 ; AC-SYS-002. Externas: classpath JVM.

**Equivalente Rust propuesto.** Función pura `check_same_version(product, dependencies, versions: &BTreeMap<String, Version>) -> Option<String>` con el mismo texto, más un registro de versiones de crates conocidos (`ManifestInfo::versions()` poblado en compilación a partir de `env!("CARGO_PKG_VERSION")` de cada crate rakka; Q-SYS-3(a)). El arranque llama `check_same_version` y respeta `fail-mixed-versions`. Viabilidad alta para la parte pura; la lectura de manifests no existe en Rust.

**Diferencias observables.** No hay MANIFEST ni classpath: las versiones vienen de constantes de compilación (en un workspace de crates con versión única el chequeo es trivial y casi siempre `None`; el caso de versiones mezcladas sólo es observable inyectando el mapa). Los nombres de módulo de `allModules` se reducen a los crates rakka equivalentes.

**Aserciones.**
- A1: con `pekko-actor 1.0.2`, `pekko-persistence 1.0.1`, `pekko-cluster 1.0.1` y `unrelated 1.0.1`, y `allModules = [pekko-actor, pekko-persistence, pekko-cluster]`, el resultado es exactamente el mensaje con "version 1.0.2 of Pekko", "[pekko-persistence, pekko-cluster]" y "(1.0.1, [pekko-cluster, pekko-persistence]), (1.0.2, [pekko-actor])" — actor-tests/src/test/scala/org/apache/pekko/util/ManifestInfoSpec.scala:21-33.
- A2: con versiones `1.0.2` y `1.0.2+10-abababef` (dynver) la versión mayor es la dynver, el módulo a actualizar es `pekko-actor` y el agrupado es `(1.0.2, [pekko-actor]), (1.0.2+10-abababef, [pekko-persistence])` — actor-tests/src/test/scala/org/apache/pekko/util/ManifestInfoSpec.scala:35-45.
- A3: módulos no listados en `dependencies` (`unrelated`) se ignoran — actor-tests/src/test/scala/org/apache/pekko/util/ManifestInfoSpec.scala:22-28 (el caso de A1 incluye `unrelated 1.0.1` y no aparece en el mensaje).
- A4: si todas las versiones coinciden, `checkSameVersion` devuelve `None` — derivada de ManifestInfo.scala:84,101, *no afirmada por spec*. Caracterización propuesta.
- A5: con `fail-mixed-versions = on` y discrepancia, el arranque lanza `IllegalStateException` con el mismo mensaje; con `off`, sólo un `warning` — derivada de ManifestInfo.scala:165-193 y ActorSystem.scala:1071, *no afirmada por spec*. Caracterización propuesta inyectando el mapa de versiones.
- A6: sólo cuentan artefactos con vendor `org.apache.pekko` — derivada de ManifestInfo.scala:46,140-144, *no afirmada por spec*; sin equivalente en Rust (no hay manifests).

**Validación prevista.** A1-A3: tests portados 1:1 (función pura). A4-A5: caracterización propia con mapa inyectado. A6: no aplicable en Rust (se documenta). Sin oráculo: A1-A2 son strings fijos ya capturados por el spec; justificado.

**Decisiones.** Q-SYS-3 (efecto: se implementa la parte pura y un registro de versiones de compilación; la lectura de MANIFEST queda sin equivalente, que es una exclusión menor sujeta a la decisión del Owner).

## AC-SYS-021 — Main (arranca un sistema con un actor y termina al morir)

*Unidad SYS · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual, deprecado desde Akka 2.6.0] `org.apache.pekko.Main.main(args)` exige exactamente un argumento (FQCN de la clase del actor "supervisor de aplicación"); con otro número imprime por stdout `you need to provide exactly one argument: the class of the application supervisor actor` y no hace nada más. Con un argumento: crea `ActorSystem("Main")`, carga la clase vía `dynamicAccess.getClassFor[Actor]`, crea el actor `app` y un `Terminator` (`app-terminator`) que vigila (`watch`) al actor; cuando `app` termina, el `Terminator` loguea `info("application supervisor has terminated, shutting down")` y llama `context.system.terminate()`. Si algo falla (`NonFatal`) al cargar/crear, termina el sistema y relanza la excepción.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/Main.scala:27-62 (1.7.1)
- specs: ninguno (búsqueda de `pekko.Main`/`Main.main` en actor-tests, testkit y actor/src/test sin resultados)

**Precondiciones.** El argumento debe ser el FQCN de una clase `Actor` instanciable sin argumentos.

**Comportamiento.** Ver contrato. Orden: `ActorSystem("Main")` → `getClassFor` → `actorOf(Props(appClass), "app")` → `actorOf(Props(classOf[Terminator], app), "app-terminator")` (Main.scala:42-46). `Terminator.preStart`-equivalente: `context.watch(app)` en el constructor (Main.scala:54); `receive { case Terminated(_) => log.info(..); context.system.terminate() }` (Main.scala:55-59).

**Errores.** Aridad incorrecta: mensaje por stdout (sin excepción ni código de salida). Clase inexistente o no-actor: `getClassFor(..).get` lanza (`ClassNotFoundException`/`ClassCastException` envueltos), el `catch NonFatal` hace `system.terminate()` y relanza (Main.scala:47-49).

**Orden / concurrencia / ciclo de vida.** `main` retorna tras crear los actores; el proceso sigue vivo mientras el sistema no termine (hilos no daemon según `pekko.daemonic`).

**Dependencias.** Filas: AC-SYS-001.2 ; AC-SYS-004 ; AC-DW-001 ; AC-EVT-013.2. Externas: ninguna.

**Equivalente Rust propuesto.** Función utilitaria `rakka::run_main(app_factory_name: &str) -> Result<(), SystemError>` tras `#[cfg(feature = "deprecated")]` (Q-SYS-3), con el nombre de la clase sustituido por un nombre registrado en el registro de fábricas (Q-X-2): crea el sistema `"Main"`, el actor `app`, el actor `app-terminator` que lo observa y termina el sistema cuando `app` muere; aridad incorrecta ⇒ mensaje por stdout. Viabilidad alta.

**Diferencias observables.** El argumento es un nombre registrado, no un FQCN; el binario `Main` de la JVM no tiene equivalente (`fn main` del usuario); el mensaje de uso se conserva.

**Aserciones.** Ninguna está afirmada por spec en el repositorio de Pekko; todas las siguientes son *no afirmada por spec* y se proponen como caracterización:
- A1: `main(&[])` y `main(&["a","b"])` imprimen el mensaje de uso y no crean sistema — derivada de Main.scala:39-41.
- A2: con un actor `app` que se detiene a sí mismo, el sistema `Main` termina (`when_terminated` se completa) — derivada de Main.scala:53-60.
- A3: con una clase inexistente, el sistema termina y el error se propaga — derivada de Main.scala:47-49.
- A4: los actores se llaman `/user/app` y `/user/app-terminator` — derivada de Main.scala:45-46.

**Validación prevista.** Caracterización propia (A1-A4) con captura de stdout y `when_terminated`; sin oráculo (deprecado, sin spec, comportamiento trivial) — justificado.

**Decisiones.** Q-SYS-3 (efecto: la fila existe pero queda detrás de la feature `deprecated`), Q-X-2 (nombre registrado en vez de FQCN).

## AC-SYS-022 — Settings.toString con redacción de datos sensibles (renderWithRedactions)

*Unidad SYS · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] settings.toString renderiza el Config sin valores de variables de entorno y reemplazando el nombre de usuario del proceso por `<username>`; no aparece el home del usuario. El Config se renderiza con setShowEnvVariableValues(false) y se reemplaza `user.name` por `<username>`.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:487-490
- actor/src/main/scala/org/apache/pekko/util/Helpers.scala:182-194

**Condiciones.**
- El reemplazo es por subcadena (String.replace) del valor de la propiedad `user.name` (Helpers.scala:191): si el nombre de usuario es una subcadena común, se reemplaza en todo el texto.

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** Filas: AC-CFG-001. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `Settings::to_redacted_string()` y `Display`

**Diferencias observables.** Rust: usar el usuario del proceso (`USER`/`getuid`) y no mostrar valores de variables de entorno. Diferencia: el 'home' del test se pasa como propiedad Java `${user.home}` en HOCON; en Rust el equivalente de sustitución es `${HOME}` (Q-X-1).

**Aserciones.**
- A1: `settings.toString` no contiene `user.name` ni `user.home` — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:421-425.
- A2: `settings.toString` contiene `<username>` — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:426.

**Oráculo.** Jar oficial: escenario propuesto `sys-settings-redaction.jsh`.

**Decisiones.** Q-SYS-5, Q-X-1

## AC-SYS-023 — assertInitialized: guardia contra uso del sistema durante la inicialización

*Unidad SYS · visibilidad internal-required · analysis `extracted`*

**Contrato observable.** [contractual interno] Lanza IllegalStateException con texto fijo mientras el sistema no completó provider.init. Las extensiones que requieren sistema inicializado llaman a esta guardia.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1081-1088

**Condiciones.**
- IllegalStateException("The calling code expected that the ActorSystem was initialized but it wasn't yet. This is probably a bug in the ActorSystem initialization sequence often related to initialization of extensions. Please report at https://github.com/apache/pekko/issues.") (actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1086-1088; texto parcial, verificar contra el archivo al implementar).

**Orden / concurrencia / ciclo de vida.** _initialized se fija tras provider.init (actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1097).

**Dependencias.** Filas: AC-SYS-002 ; AC-EXT-003. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `system.assert_initialized() -> Result<(), InitializationError>`

**Diferencias observables.** Rust: `Err`. Diferencia: tipo de error.

**Aserciones.**
- A1: antes de provider.init, assertInitialized falla — derivada de actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1086-1088, *no afirmada por spec*.

**Oráculo.** No aplica: sólo alcanzable con extensiones que llaman en construcción.

**Decisiones.** Q-X-5

## AC-SYS-024 — LocalActorRefProvider.resolveActorRef (String y ActorPath): paths ajenos o inexistentes → deadLetters / EmptyLocalActorRef

*Unidad SYS · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual interno] Path con dirección distinta de la raíz o path vacío -> deadLetters (con log Debug `Resolve (deserialization) ..`); path no existente -> EmptyLocalActorRef bajo el mismo path. Resuelve refs por path para deserialización, ActorSelection y envío por path.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:629-664

**Condiciones.**
- String no parseable o dirección ajena: `deadLetters` + Debug `Resolve (deserialization) of unknown (invalid) path [{}], using deadLetters.` (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:631-633).
- ActorPath de otro root: Debug `Resolve (deserialization) of foreign path [{}] doesn't match root path [{}], using deadLetters.` (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:638-643).
- Elementos vacíos: deadLetters (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:651-653). No encontrado: `EmptyLocalActorRef(system.provider, ref.path / pathElements, eventStream)` (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:662).

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** Filas: AC-PATH-007 ; AC-REF-012. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `provider.resolve_actor_ref(path) -> ActorRef`

**Diferencias observables.** Rust: igual; `ActorSelection` (AC-REF) reutiliza este método.

**Aserciones.**
- A1: un hijo con nombre `pekko%3A%2F%2FClusterSystem%40127.0.0.1%3A7355` se resuelve y conserva el nombre — actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:48-64.
- A2: path de otro sistema devuelve deadLetters — derivada de actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:636-644, *no afirmada por spec*.
- A3: path inexistente devuelve un ref terminado (EmptyLocalActorRef) con el mismo path — derivada de actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:655-663.

**Oráculo.** Jar oficial: escenario propuesto `sys-resolve-actor-ref.jsh`.

**Decisiones.** ninguna específica

## AC-SYS-025 — Hijos especiales del guardián raíz: temp, deadLetters y extraNames (registerExtraNames)

*Unidad SYS · visibilidad internal-required · analysis `extracted`*

**Contrato observable.** [contractual interno] rootGuardian.getSingleChild: 'temp' -> tempContainer, 'deadLetters' -> deadLetters, otros nombres registrados con registerExtraNames, resto -> hijo real. Permite que /deadLetters y /temp sean resolubles por path como hijos del root.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:506-515,553-558

**Condiciones.**
- getParent del root = root (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:553).

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** Filas: AC-SYS-017 ; AC-REF-010. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `RootGuardian::child(name)` con tabla de nombres especiales

**Diferencias observables.** Rust: igual.

**Aserciones.**
- A1: `system.actorSelection("/deadLetters")` y `/temp` resuelven a esos refs — derivada de actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:554-557, *no afirmada por spec leída*.

**Oráculo.** Jar oficial: `sys-resolve-actor-ref.jsh`.

**Decisiones.** ninguna específica

## AC-EVT-001 — Contrato EventBus: subscribe/unsubscribe/publish con resultado booleano

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] subscribe devuelve false si ya estaba suscrito al canal; unsubscribe false si no lo estaba; publish entrega a todos los suscriptores del clasificador y a ningún otro; publish sin suscriptores no falla. Bus genérico con tres tipos asociados; las implementaciones concretas (Lookup/Subchannel/Scanning/ManagedActor) comparten estas garantías.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/EventBus.scala:32-89
- documentación de usuario `docs/src/main/paradox/event-bus.md` queda fuera de los módulos del alcance (no se usa como fuente de contrato)

**Condiciones.**
- subscribe(sub, c) -> true la primera vez, false si ya estaba (actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:75-79).
- unsubscribe(sub, c) -> true si estaba, false si no (actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:65-73,81-85).
- publish(e) sin suscriptores: sin efecto ni error (actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:99-101).

**Orden / concurrencia / ciclo de vida.** Cada suscriptor recibe un evento por publish (sin duplicados); no hay garantía de orden entre suscriptores.

**Dependencias.** Filas: -. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** trait `EventBus { type Event; type Classifier; type Subscriber; }`

**Diferencias observables.** Rust: trait con tipos asociados; Subscriber es `ActorRef` o closure. Diferencia: ninguna observable.

**Aserciones.**
- A1: subscribe -> true — actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:61-63.
- A2: unsubscribe de suscriptor existente -> true — actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:65-67.
- A3: unsubscribe de un suscriptor inexistente -> false — actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:69-73.
- A4: subscribe dos veces: true luego false; unsubscribe luego true — actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:75-79.
- A5: unsubscribe dos veces: true luego false — actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:81-85.
- A6: 10 suscriptores con clasificadores distintos: todos subscribe=true y unsubscribe=true — actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:87-97.
- A7: publish con un único suscriptor entrega 1 vez (y varias veces si se publica varias) — actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:103-121.
- A8: publish a 10 suscriptores del mismo clasificador entrega a los 10 — actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:123-136.
- A9: no se entrega a suscriptores de otro clasificador ni a ex-suscriptores — actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:138-155.

**Oráculo.** Jar oficial: escenario propuesto `evt-eventbus-contract.jsh` (clase LookupClassification mínima; sólo Scala-from-jshell es incómodo: usar `EventStream` con clases como clasificador).

**Decisiones.** ninguna específica

## AC-EVT-002 — LookupClassification

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual] Clasificación por igualdad de clasificador (Index<Classifier, Subscriber> con comparador de suscriptores). subscribe(sub, c) = subscribers.put(c, sub); publish(e) itera los valores del clasificador classify(e).

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/EventBus.scala:92-127

**Condiciones.**
- compareSubscribers define la identidad de suscriptor (orden total); mapSize es sólo una pista de capacidad (EventBus.scala:95-103).

**Orden / concurrencia / ciclo de vida.** Itera el índice sin bloqueo global (Index concurrente, AC-UTIL-009).

**Dependencias.** Filas: AC-EVT-001 ; AC-UTIL-009. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `HashMap<Classifier, BTreeSet<Subscriber>>`

**Diferencias observables.** Rust: `RwLock<HashMap<..>>` o estructura concurrente; sin diferencia observable.

**Aserciones.**
- A1: los tests abstractos de EventBus pasan con classify = evento.toString — actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:346-360.
- A2: unsubscribe(subscriber) sin clasificador quita todas las suscripciones — derivada de actor/src/main/scala/org/apache/pekko/event/EventBus.scala:121, *no afirmada por spec*.
- A3: publish entrega en el hilo del publicador (publish(event, subscriber) síncrono) — derivada de actor/src/main/scala/org/apache/pekko/event/EventBus.scala:123-126.

**Oráculo.** Jar oficial: ver AC-EVT-001.

**Decisiones.** ninguna específica

## AC-EVT-003 — SubchannelClassification (jerarquía de clasificadores)

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] Suscribirse a un supertipo recibe eventos de subtipos (clase o interfaz); correcto al agregar/quitar suscripciones en cualquier orden; resultado booleano de subscribe/unsubscribe por (suscriptor, canal) exacto. Cache Map[Classifier, Set[Subscriber]] volátil reconstruido bajo lock (subscriptions.synchronized) y consultado sin lock en publish.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/EventBus.scala:133-208

**Condiciones.**
- subscribe -> diff.nonEmpty (true si cambió la visibilidad efectiva) (EventBus.scala:159-163).
- publish con clasificador desconocido: lo añade a la caché bajo lock (EventBus.scala:177-190).
- hasSubscriptions(sub) es costoso: recorre la caché (EventBus.scala:196-199).

**Orden / concurrencia / ciclo de vida.** Escrituras serializadas por lock; lecturas lock-free sobre caché inmutable.

**Dependencias.** Filas: AC-UTIL-008 ; AC-EVT-001. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** índice de subclasificación sobre `EventClass` (ver Q-X-6)

**Diferencias observables.** Rust: `ArcSwap<HashMap<..>>` + Mutex para escritura. Diferencia: la relación de subtipo debe proveerse (Q-X-6).

**Aserciones.**
- A1: subscribe(B3) true; publish(C) sin entrega, publish(B3) entrega; subscribe(A) true; C y B2 entregan; unsubscribe(B2) true; luego C y B3 sin duplicar, A entrega — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:172-196.
- A2: clases e interfaces (AT, BT, CC, CCATBT): suscripción a interfaz recibe subtipos — update on subscribe actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:198-219.
- A3: update on unsubscribe (a3 deja de recibir, otros no) — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:221-241.
- A4: unsubscribe(sub) de todos los canales — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:243-263.
- A5: update on publish (la caché se completa al publicar) — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:265-279.
- A6: unsubscribe de una clase usada con trait: a2 pierde CC pero conserva BT — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:281-301.
- A7: subscribe después de publish añade el canal sin perder los anteriores — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:303-319.

**Oráculo.** Jar oficial: escenario propuesto `evt-subchannel.jsh` usando `EventStream` con clases Java/Scala de prueba compiladas en jshell.

**Decisiones.** Q-X-6

## AC-EVT-004 — ScanningClassification

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual] Recorre todos los pares (clasificador, suscriptor) evaluando `matches(classifier, event)`; orden total por compareClassifiers/compareSubscribers. Un mismo suscriptor con dos clasificadores que coinciden recibe el evento dos veces (itera pares).

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/EventBus.scala:216-264

**Condiciones.**
- unsubscribe(subscriber) elimina todos los pares con compareSubscribers==0 (EventBus.scala:253-259).

**Orden / concurrencia / ciclo de vida.** Iteración en orden total de los pares.

**Dependencias.** Filas: AC-EVT-001. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `Vec`/`BTreeSet<(Classifier, Subscriber)>` + predicado

**Diferencias observables.** Rust: `BTreeSet` bajo lock o skiplist concurrente.

**Aserciones.**
- A1: los tests abstractos de EventBus pasan con matches(c,e)=e.toString==c — actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:315-329.
- A2: un suscriptor con dos clasificadores coincidentes recibe dos veces — derivada de actor/src/main/scala/org/apache/pekko/event/EventBus.scala:256-263, *no afirmada por spec*.

**Oráculo.** Jar oficial: ver AC-EVT-001.

**Decisiones.** ninguna específica

## AC-EVT-005 — ManagedActorClassification (desuscripción automática al morir el suscriptor) y ActorClassificationUnsubscriber

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] El bus vigila (watch) a los suscriptores mediante un unsubscriber y los quita al recibir Terminated; la muerte del actor clasificador (monitored) no quita suscripciones; un suscriptor sólo se des-registra del unsubscriber cuando deja su última suscripción. seqNr secuencia las altas/bajas hacia el unsubscriber (Register/Unregister con seq == atSeq+1; si no, stash) para que watch/unwatch lleguen en el mismo orden que las mutaciones del mapa.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/EventBus.scala:272-431
- actor/src/main/scala/org/apache/pekko/event/ActorClassificationUnsubscriber.scala:29-105
- reference.conf:752 (pekko.actor.debug.event-stream)

**Condiciones.**
- subscribe/unsubscribe con suscriptor o clasificador null: IllegalArgumentException("Subscriber is null") / ("Classifier is null") (EventBus.scala:407-415).
- Texto Debug: `actor <a> has terminated, unsubscribing it from <bus>` (actor/src/main/scala/org/apache/pekko/event/ActorClassificationUnsubscriber.scala:69), `unregistered watch of <a> in <bus>`, `registered watch for <a> in <bus>`, `will monitor <bus>`.

**Orden / concurrencia / ciclo de vida.** Terminated(actor) -> unsubscribe(actor) -> otro Unregister que se procesa inofensivamente (comentario actor/src/main/scala/org/apache/pekko/event/ActorClassificationUnsubscriber.scala:70-72).

**Dependencias.** Filas: AC-EVT-001 ; AC-DW-001 ; AC-SYS-010. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** actor unsubscriber interno + mapa `ActorRef -> BTreeSet<ActorRef>`

**Diferencias observables.** Rust: ídem, usando el watch de AC-DW. Diferencia: `this.toString` del bus en el texto Debug depende de Display del bus.

**Aserciones.**
- A1: al morir el suscriptor, deja de recibir y el unsubscriber lo publica en Debug `actor <subs> has terminated, unsubscribing it from <bus>` — actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:201-222,282-288.
- A2: la muerte del actor clasificador (a1) no quita la suscripción; sigue entregando m(2) — actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:224-246.
- A3: tras unsubscribe de a1 sigue recibiendo de a2; sólo al quitar a2 se publica `unregistered watch of <subs> in <bus>` — actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:248-280,290-296.
- A4: los tests abstractos de EventBus pasan con ActorEventBus — actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:179-197.
- A5: null en subscribe/unsubscribe lanza IllegalArgumentException — derivada de actor/src/main/scala/org/apache/pekko/event/EventBus.scala:402-414, *no afirmada por spec*.

**Oráculo.** Jar oficial: escenario propuesto `evt-managed-classification.jsh`.

**Decisiones.** Q-X-4

## AC-EVT-006 — EventStream: subscribe por clase, unsubscribe, publish; null → IllegalArgumentException("subscriber is null"); modo debug event-stream

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] Bus del sistema con SubchannelClassification por clase del evento (`event.getClass`); entrega con tell al ActorRef suscriptor; null como suscriptor lanza IllegalArgumentException con mensaje exacto `subscriber is null`. Con debug=true publica Debug en cada (un)subscribe: `subscribing <ref> to channel <cls>`, `unsubscribing <ref> from channel <cls>`, `unsubscribing <ref> from all channels`, `initialized unsubscriber to: <u>, registering <n> initial subscribers with it`, `not using unsubscriber <u>, because already initialized with <p>`.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/EventStream.scala:34-172
- reference.conf:752 (pekko.actor.debug.event-stream)

**Condiciones.**
- subscribe/unsubscribe(null,..) -> IllegalArgumentException("subscriber is null") (actor/src/main/scala/org/apache/pekko/event/EventStream.scala:61,72,85).
- publish a suscriptor terminado cuando sys==null: lo desuscribe (actor/src/main/scala/org/apache/pekko/event/EventStream.scala:51-54).

**Orden / concurrencia / ciclo de vida.** subscribe: (debug Debug) -> registerWithUnsubscriber -> super.subscribe. unsubscribe: super -> unregisterIfNoMoreSubscribedChannels -> (debug Debug).

**Dependencias.** Filas: AC-EVT-003 ; AC-EVT-007. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `EventStream` con suscripciones por `EventClass`/`TypeId` (Q-X-6)

**Diferencias observables.** Rust: ver Q-X-6 para la clasificación de eventos; Debug de event-stream se conserva textual.

**Aserciones.**
- A1: subscribe(testActor, M); publish(M(42)) entrega; unsubscribe(testActor); publish(M(13)) no entrega — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:108-122.
- A2: subscribe(null, ..) lanza IllegalArgumentException con mensaje `subscriber is null` — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:124-128.
- A3: unsubscribe(null, c) y unsubscribe(null) lanzan IllegalArgumentException `subscriber is null` — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:130-135.
- A4: suscribir dos veces el mismo ref/clase devuelve true y luego false — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:333-334.
- A5: el mensaje Debug `unsubscribing <ref> from all channels` se publica en modo debug — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:337.
- A6: el mensaje Debug `unsubscribing <ref> from channel class <cls>` usa `channel.toString` de java.lang.Class (`class pkg.A` / `interface pkg.T`) — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:417-445 (dependiente de JVM).

**Oráculo.** Jar oficial: escenario propuesto `evt-eventstream-debug.jsh` para comparar textos Debug; el texto con `class`/`interface` es JVM y se mapeará (Q-EVT-1).

**Decisiones.** Q-X-6, Q-EVT-1

## AC-EVT-007 — EventStreamUnsubscriber: quita suscriptores al terminar

*Unidad EVT · visibilidad internal-required · analysis `extracted`*

**Contrato observable.** [contractual por spec] Actor de sistema `eventStreamUnsubscriber-N` que vigila a los suscriptores (watch); también cuando el suscriptor ya está muerto al suscribirse; no se inicializa dos veces; unwatch cuando el suscriptor ya no tiene ningún canal. Mensajes internos Register(actor) y UnregisterIfNoMoreSubscribedChannels(actor); el unsubscriber consulta hasSubscriptions (costoso) en su propio hilo.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/EventStreamUnsubscriber.scala:30-97
- actor/src/main/scala/org/apache/pekko/event/EventStream.scala:103-172

**Condiciones.**
- Nombre del actor: `eventStreamUnsubscriber-<contador global>` (actor/src/main/scala/org/apache/pekko/event/EventStreamUnsubscriber.scala:90).
- initUnsubscriber devuelve false si ya existe (actor/src/main/scala/org/apache/pekko/event/EventStream.scala:127-134).

**Orden / concurrencia / ciclo de vida.** Terminated(actor) -> eventStream.unsubscribe(actor) -> `unsubscribing <a> from all channels` (Debug) -> UnregisterIfNoMoreSubscribedChannels -> `unwatching <a>, since has no subscriptions`.

**Dependencias.** Filas: AC-EVT-006 ; AC-DW-001 ; AC-DW-002 ; AC-SYS-010. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** actor interno `EventStreamUnsubscriber`

**Diferencias observables.** Rust: ídem. Diferencia: ninguna observable; los textos Debug se conservan.

**Aserciones.**
- A1: al terminar el suscriptor se publican `unsubscribing <target> from all channels` y `unwatching <target>` y deja de recibir — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:320-347.
- A2: un suscriptor ya terminado al suscribirse es desuscrito inmediatamente y puede re-suscribirse (true) — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:349-378.
- A3: initUnsubscriber(p.ref) sobre un sistema ya iniciado devuelve false — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:380-395.
- A4: unsubscribe(sub) publica `unwatching <sub>` — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:397-415.
- A5: el unsubscriber hace watch por cada subscribe (idempotente): dos `watching <ref>`; unsubscribe(canal) mantiene el watch mientras queden canales; al quitar el último publica `unwatching <ref>, since has no subscriptions` — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:417-445.
- A6: unsubscribe de un canal sin suscripción devuelve false — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:442.

**Oráculo.** Jar oficial: escenario propuesto `evt-eventstream-unsubscriber.jsh`.

**Decisiones.** Q-X-4

## AC-EVT-008 — Tipos de dead letters: DeadLetter, SuppressedDeadLetter, Dropped, AllDeadLetters, DeadLetterSuppression, WrappedMessage.unwrap, y su publicación por DeadLetterActorRef/EmptyLocalActorRef

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual] DeadLetter(message, sender, recipient) exige sender y recipient no nulos; DeadLetterSuppression -> se publica SuppressedDeadLetter (no DeadLetter); Dropped(message, reason, sender, recipient); WrappedMessage.unwrap desenvuelve recursivamente. A un ref muerto (EmptyLocalActorRef) o a deadLetters, un mensaje normal publica DeadLetter con sender = deadLetters si no hay sender. AllDeadLetters es el supertipo de los tres; DeadLetterListener se suscribe a DeadLetter, Dropped y UnhandledMessage.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:551-555 (AllDeadLetters)
- actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:563-567 (DeadLetter)
- actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:581-586 (SuppressedDeadLetter)
- actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:594-603 (Dropped)
- actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:605-625 (WrappedMessage)
- actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:643-728 (EmptyLocalActorRef, DeadLetterActorRef)

**Condiciones.**
- DeadLetter/SuppressedDeadLetter con sender o recipient null: IllegalArgumentException por `require` con mensajes `DeadLetter sender may not be null` / `DeadLetter recipient may not be null` (AR:593-594, 612-613).
- EmptyLocalActorRef.! con null: InvalidMessageException("Message is null") (AR:685).
- Un DeadLetter reenviado a un EmptyLocalActorRef no forma bucle: se procesa sólo con specialHandle (AR:686-687).
- Watch a un ref muerto: DeathWatchNotification(existenceConfirmed=false, addressTerminated=false) (AR:694-698); Identify -> ActorIdentity(id, None) (AR:700-702).

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** Filas: AC-REF-016 ; AC-EVT-006. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** structs `DeadLetter`, `SuppressedDeadLetter`, `Dropped`; trait marker `DeadLetterSuppression`; trait `WrappedMessage`

**Diferencias observables.** Rust: structs; `message: Box<dyn Any + Send>` (Q-X-3). Diferencia: igualdad de DeadLetter (== de case class) requiere PartialEq sobre `Box<dyn Any>` -> los tests deben comparar por campos.

**Aserciones.**
- A1: a un actor muerto, SuppressedMsg produce SuppressedDeadLetter(SuppressedMsg, testActor, system.deadLetters) y NormalMsg produce DeadLetter(NormalMsg, testActor, deadActor) — actor-tests/src/test/scala/org/apache/pekko/actor/DeadLetterSupressionSpec.scala:41-62.
- A2: el listener de AllDeadLetters recibe ambos en ese orden — DeadLetterSupressionSpec.scala:60-62.
- A3: vía ActorSelection se desenvuelve el mensaje de ActorSelectionMessage (sólo se comprueba el contenido) — DeadLetterSupressionSpec.scala:64-71.
- A4: a system.deadLetters: DeadLetter(NormalMsg, testActor, system.deadLetters) y SuppressedDeadLetter equivalente — DeadLetterSupressionSpec.scala:74-97; por ActorSelection a deadLetters también — :99-105.
- A5: sender/recipient null en DeadLetter lanzan IllegalArgumentException — derivada de actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:565-566, *no afirmada por spec*.
- A6: WrappedMessage.unwrap desenvuelve recursivamente — derivada de actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:610-615, *no afirmada por spec*.

**Oráculo.** Jar oficial: escenario propuesto `evt-deadletters.jsh` (suscribe probes a DeadLetter/SuppressedDeadLetter/AllDeadLetters).

**Decisiones.** Q-X-3, Q-EVT-1

## AC-EVT-009 — DeadLetterListener: log de dead letters con límite (log-dead-letters), suspensión y reinicio

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] Cuenta DeadLetter, Dropped y UnhandledMessage; loguea Info 'Message [<clase>] ...' hasta log-dead-letters; al alcanzar el límite añade `, no more dead letters will be logged in next [<dur>]` (o `, no more dead letters will be logged` y se detiene si suspend=infinite); tras suspend-duration, el siguiente mensaje se loguea con `, of which N were not logged. The counter will be reset now` y reinicia el contador; no loguea mensajes envueltos de DeadLetterSuppression ni durante el apagado salvo log-dead-letters-during-shutdown. Cuatro modos de recibir: siempre (log-dead-letters=on => Int.MaxValue), hasta N con parada, hasta N con suspensión temporal, y suspendido.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/DeadLetterListener.scala:34-206
- reference.conf:58 (log-dead-letters=10), :63 (log-dead-letters-during-shutdown=off), :68 (suspend-duration=5 minutes)
- ActorSystem.scala:441-455 (parseo: off/false->0; on/true->Int.MaxValue; entero; suspend `infinite`)

**Condiciones.**
- Texto DeadLetter: `Message [<cls>] from <sender> to <recipient> was not delivered. [<n>] dead letters encountered<doneMsg>. If this is not an expected behavior then <recipient> may have terminated unexpectedly. This logging can be turned off or adjusted with configuration settings 'pekko.log-dead-letters' and 'pekko.log-dead-letters-during-shutdown'.` (actor/src/main/scala/org/apache/pekko/event/DeadLetterListener.scala:151-162). `from <sender>` sólo si sender es real (no noSender ni deadLetters) (actor/src/main/scala/org/apache/pekko/event/DeadLetterListener.scala:138,167-169).
- Texto Dropped: `Message [<cls>] from <s> to <r> was dropped. <reason>. [<n>] dead letters encountered<doneMsg>. ` (actor/src/main/scala/org/apache/pekko/event/DeadLetterListener.scala:143-146); Unhandled: `... was unhandled. ...` (actor/src/main/scala/org/apache/pekko/event/DeadLetterListener.scala:147-150). Marker: ActorLogMarker.deadLetter(messageStr) (actor/src/main/scala/org/apache/pekko/event/DeadLetterListener.scala:164).
- Mensaje envuelto: ` wrapped in [<wrapperCls>]` (actor/src/main/scala/org/apache/pekko/event/DeadLetterListener.scala:141).
- El evento Info tiene logSource = recipient.path.toString y logClass = recipient.getClass (actor/src/main/scala/org/apache/pekko/event/DeadLetterListener.scala:158-159).

**Orden / concurrencia / ciclo de vida.** preStart suscribe DeadLetter, Dropped y UnhandledMessage; preRestart/postRestart/postStop especiales (no re-suscribe, no pierde suscripción en restart; postStop desuscribe) (actor/src/main/scala/org/apache/pekko/event/DeadLetterListener.scala:57-70).

**Dependencias.** Filas: AC-CFG-005 ; AC-EVT-008 ; AC-EVT-010 ; AC-TK-005.2. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** actor de sistema `DeadLetterListener`

**Diferencias observables.** Rust: textos conservados; `<cls>` = nombre de tipo Rust (Q-EVT-1).

**Aserciones.**
- A1: con log-dead-letters=4 y suspend=2s: los 4 primeros mensajes (DeadLetter, Dropped, Unhandled, DeadLetter) se loguean con contadores [1][2][3][4], el 4º con `, no more dead letters will be logged in next` — actor-tests/src/test/scala/org/apache/pekko/actor/DeadLetterSuspensionSpec.scala:70-84.
- A2: los mensajes 5 y 6 no se loguean — DeadLetterSuspensionSpec.scala:85-86.
- A3: tras 2050 ms el 7º se loguea con `, of which 2 were not logged` — DeadLetterSuspensionSpec.scala:88-96.
- A4: el 8º reinicia el contador: `[1] dead letters encountered` — DeadLetterSuspensionSpec.scala:98-101.
- A5: con loglevel=INFO, un mensaje a un actor muerto produce Info `from Actor[pekko://LogDeadLetters/system/testProbe.. not delivered` y (por TestEventListener) un Warning `received dead letter from Actor[pekko://LogDeadLetters/system/testProbe..` — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:167-189.
- A6: sin sender: Info `.*not delivered` y Warning `received dead letter: boom` — ActorSystemSpec.scala:188-210.
- A7: mensajes DeadLetterSuppression envueltos no se cuentan ni loguean — derivada de actor/src/main/scala/org/apache/pekko/event/DeadLetterListener.scala:199-204, *no afirmada por spec*.

**Oráculo.** Jar oficial: escenario propuesto `evt-deadletter-listener.jsh` (captura Info con EventFilter equivalente: suscripción a LogEvent y volcado de mensajes).

**Decisiones.** Q-EVT-1, Q-X-7

## AC-EVT-010 — UnhandledMessage (publicación y log con debug.unhandled)

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual] UnhandledMessage(message, sender, recipient) se publica en el EventStream; con pekko.actor.debug.unhandled=on un actor interno `UnhandledMessageForwarder` lo convierte en Debug `unhandled message from <sender>: <msg>` con logSource=recipient.path y logClass=recipient.getClass. El forwarder se crea como /system/UnhandledMessageForwarder dentro de startDefaultLoggers.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:164-176
- actor/src/main/scala/org/apache/pekko/actor/Actor.scala:293-303 (`UnhandledMessage extends WrappedMessage with AllDeadLetters`), 545-550 y 656-661 (`aroundReceive` -> `unhandled(msg)`: `Terminated(dead)` => `DeathPactException(dead)`, otro => `eventStream.publish(UnhandledMessage(message, sender(), self))`; leído en la auditoría 2026-10-31)
- reference.conf:755 (debug.unhandled)

**Condiciones.**
- Si el nombre ya está en uso (InvalidActorNameException) se ignora (actor/src/main/scala/org/apache/pekko/event/Logging.scala:174-176).
- Los tests de LocalActorRefProviderSpec ven el log como Warning por TestEventListener (`unhandled message from Actor[..]: msg`, TestEventListener.scala:582-584).

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** Filas: AC-CELL-004 ; AC-EVT-006. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `struct UnhandledMessage` + forwarder interno

**Diferencias observables.** Rust: igual.

**Aserciones.**
- A1: publicar UnhandledMessage(42, deadLetters, deadLetters) con debug.unhandled=on entrega al suscriptor el mensaje y además un Debug(logSource=deadLetters.path, logClass=deadLetters.getClass, `unhandled message from <deadLetters>: 42`) — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:137-153.
- A2: sin debug.unhandled no se crea el forwarder — derivada de actor/src/main/scala/org/apache/pekko/event/Logging.scala:164-165, *no afirmada por spec*.
- A3: el warning de TestEventListener por mensaje no manejado coincide con LocalActorRefProviderSpec — actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:69-106.

**Oráculo.** Jar oficial: escenario propuesto `evt-unhandled.jsh`.

**Decisiones.** Q-EVT-1

## AC-EVT-011 — LoggingBus: niveles, setLogLevel, logger stdout de arranque, startDefaultLoggers (handshake InitializeLogger/LoggerInitialized con logger-startup-timeout), stopDefaultLoggers

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual] Antes de start se loguea por StandardOutLogger (stdout-loglevel); luego los loggers configurados (pekko.loggers) como actores bajo /system, cada uno debe responder LoggerInitialized dentro de logger-startup-timeout o se publica Warning y el arranque falla con LoggerInitializationException si responde otra cosa. setLogLevel suscribe/desuscribe cada logger a las clases Error/Warning/Info/Debug según el nivel; los eventos por debajo del nivel nunca llegan al actor.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:44-241
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:528-565 (levelFor/classFor)
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:974-1002 (InitializeLogger/LoggerInitialized)
- reference.conf:19,28,31,36,41,46 (loggers, logging-filter, loggers-dispatcher, logger-startup-timeout, loglevel, stdout-loglevel)

**Condiciones.**
- pekko.loggers vacío -> DefaultLogger (actor/src/main/scala/org/apache/pekko/event/Logging.scala:133-136). StandardOutLogger en la lista se ignora al crear actores y se mantiene suscrito (actor/src/main/scala/org/apache/pekko/event/Logging.scala:140,178-180).
- Logger no cargable: ConfigurationException(`Logger specified in config can't be loaded [<name>] due to [<e>]`, e) (actor/src/main/scala/org/apache/pekko/event/Logging.scala:149-152).
- Timeout de arranque del logger: Warning `Logger <name> did not respond within <timeout> to InitializeLogger(bus)` y respuesta '[TIMEOUT]' que luego falla con LoggerInitializationException(`Logger <name> did not respond with LoggerInitialized, sent instead [TIMEOUT]`) (actor/src/main/scala/org/apache/pekko/event/Logging.scala:226-238).
- Cualquier Exception al arrancar: stderr `error while starting up loggers`, stacktrace y ConfigurationException(`Could not start logger due to [<e>]`) (actor/src/main/scala/org/apache/pekko/event/Logging.scala:181-185).
- Nombre del actor logger: `log<N>-<SimpleName>` con N de LogExt (actor/src/main/scala/org/apache/pekko/event/Logging.scala:221); dispatcher = pekko.loggers-dispatcher (actor/src/main/scala/org/apache/pekko/event/Logging.scala:222).
- stopDefaultLoggers: desuscribe y detiene cada logger; si StandardOutLogger no estaba, lo activa antes y publica Debug `shutting down: StandardOutLogger`; al final Debug `all default loggers stopped` (actor/src/main/scala/org/apache/pekko/event/Logging.scala:192-211).

**Orden / concurrencia / ciclo de vida.** Debug publicados: `StandardOutLogger started`, `Default Loggers started`, `logger <name> started`. stopDefaultLoggers corre cuando /system termina (AC-SYS-003.2).

**Dependencias.** Filas: AC-CFG-004 ; AC-EVT-006 ; AC-SYS-010. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** trait `Logger` + `LoggingBus`; Q-X-7 decide actor vs backend

**Diferencias observables.** Rust: ver Q-X-7. Diferencia: nombres `log1-DefaultLogger` se conservan sólo si los loggers son actores.

**Aserciones.**
- A1: manage log levels: tras setLogLevel(Warning/Debug/Error) sólo llegan eventos con level <= configurado — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:155-170,456-463.
- A2: un sistema con pekko.loglevel=DEBUG y un logger configurado imprime a stdout — actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:183-188.
- A3: con loglevel=OFF y stdout-loglevel=OFF no se escribe nada en stdout — actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:210-216.
- A4: con varios loggers cada uno recibe el evento — actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:218-239.
- A5: stdout-loglevel/loglevel desconocidos usan ErrorLevel — derivada de actor/src/main/scala/org/apache/pekko/event/Logging.scala:93-102,126-131, *no afirmada por spec*.
- A6: timeout de logger: Warning y LoggerInitializationException — derivada de actor/src/main/scala/org/apache/pekko/event/Logging.scala:226-238, *no afirmada por spec*.
- A7: logger no cargable -> ConfigurationException — derivada de actor/src/main/scala/org/apache/pekko/event/Logging.scala:147-153, *no afirmada por spec*.
- A8: StandardOutLogger.! con null lanza InvalidMessageException("Message is null") — derivada de actor/src/main/scala/org/apache/pekko/event/Logging.scala:1131, *no afirmada por spec*.

**Oráculo.** Jar oficial: escenario propuesto `evt-loggingbus.jsh` (loggers configurados con clase que falla, loglevel desconocido).

**Decisiones.** Q-X-7, Q-EVT-2

## AC-EVT-012 — Familia LogEvent (Error/Warning/Info/Debug y variantes con MDC/marker: Error2/Error3, ...), LogLevel, campos timestamp/thread/logSource/logClass/message/mdc

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual] Eventos inmutables publicados en el EventStream y entregados a los loggers; thread = hilo creador, timestamp = currentTimeMillis al crear; Error lleva cause (Error.NoCause por defecto); variantes con MDC (Error2..) y con marker (Error3..). LogEvent.level, logSource (String), logClass, message (Any), mdc (Map[String, Any] vacío por defecto).

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:497-565
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:721-972

**Condiciones.**
- `LogEvent.apply(level, logSource, logClass, message[, mdc[, marker]])` con un nivel que no es Error/Warning/Info/Debug (p. ej. `OffLevel`) lanza `IllegalArgumentException("Unsupported log level [<level>]")` (actor/src/main/scala/org/apache/pekko/event/Logging.scala:783-812; auditoría 2026-10-31, sin spec que lo afirme).
- Variantes (Logging.scala:804-960): `Error(cause, src, cls, msg)` (case class; `Error.NoCause` por defecto, objeto `NoStackTrace`), `Error2` (+mdc), `Error3` (+mdc, marker); `Warning`, `Warning2` (+mdc), `Warning3` (+mdc, marker), `Warning4` (+mdc, marker, cause; `Warning.apply(cause, .., mdc)` pasa `marker = null`); `Info`/`Info2`/`Info3` y `Debug`/`Debug2`/`Debug3` análogos. Las subclases heredan de la anterior (`Error3 <: Error2 <: Error`) y los `apply` de los objetos compañeros eligen la variante según los argumentos (auditoría 2026-10-31).
- Serialización: LogEvent está ligado a `bytes` en el test 3165 y no debe causar StackOverflowError al serializar (actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:348-357).

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** Filas: AC-EVT-011. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `enum LogEvent { Error(..), Warning(..), Info(..), Debug(..) }` con `LogLevel`

**Diferencias observables.** Rust: enum con campos; `thread` = nombre del hilo; mdc = `HashMap<String, Value>`.

**Aserciones.**
- A1: un Warning con MDC {requestId:1} llega al logger con mdc.size==1 y mdc("requestId")==1 — actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:241-260.
- A2: MDC con mensaje actual: size 3 con requestId, currentMsg y currentMsgLength==22 — actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:262-269.
- A3: MDC del mensaje se limpia en el siguiente (size 1) — actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:271-274.
- A4: serializar un LogEvent con serialize-messages=on no desborda la pila — actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:348-357.

**Oráculo.** Jar oficial: `evt-logevent-shape.jsh`.

**Decisiones.** ninguna específica

## AC-EVT-013.1 — LoggingAdapter: formato de plantillas "{}" (hasta 4 args o Array), "WARNING arguments left", isXEnabled

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] Reemplaza cada `{}` en orden por arg.toString; un único argumento Array (primitivo o de objetos) se expande como varargs; si sobran argumentos se añade ` WARNING arguments left: N`; si faltan, los `{}` restantes quedan literales; no formatea si el nivel está deshabilitado. Las sobrecargas `error(cause, template, arg1..arg4)` y `error(template, arg1..arg4)` y las equivalentes de warning/info/debug.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:1204-1313
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:1533-1570 (format1/format/formatImpl)

**Condiciones.**
- formatImpl: mientras queden argumentos busca `{}`; si no hay más `{}` añade el resto de la plantilla + ` WARNING arguments left: <n>` y termina (actor/src/main/scala/org/apache/pekko/event/Logging.scala:1546-1561).
- format1 con Array de primitivos: se boxean (actor/src/main/scala/org/apache/pekko/event/Logging.scala:1533-1537).
- notifyLog con nivel no soportado: IllegalArgumentException(`Unsupported log level [<l>]`) (actor/src/main/scala/org/apache/pekko/event/Logging.scala:1521-1527).

**Orden / concurrencia / ciclo de vida.** Se evalúa isXEnabled antes de formatear (actor/src/main/scala/org/apache/pekko/event/Logging.scala:1238).

**Dependencias.** Filas: AC-EVT-012. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `LoggingAdapter::{error,warning,info,debug}(template, &[&dyn Display])` + macros

**Diferencias observables.** Rust: sin varargs: slice de `&dyn Display`; el caso Array del único argumento se modela con un método explícito `info_all(template, args)`.

**Aserciones.**
- A1: `error("One arg message: {}", "the arg")` -> mensaje `One arg message: the arg` — LoggingAdapterTest.java:61-62.
- A2: `error(cause, "Two args message: {}, {}", ...)` -> `Two args message: an arg, another arg` con cause — LoggingAdapterTest.java:64-72.
- A3: Array de primitivos int[]{10,20,30} -> `Args as array of primitives: 10, 20, 30` — LoggingAdapterTest.java:74-76.
- A4: Array de objetos -> expandido con toString — LoggingAdapterTest.java:78-84.
- A5: cuatro argumentos -> `Four args message: 1, 2, 3, 4` — LoggingAdapterTest.java:86-87.
- A6: sobran argumentos -> sufijo ` WARNING arguments left: N` — derivada de actor/src/main/scala/org/apache/pekko/event/Logging.scala:1549-1553, *no afirmada por spec*.

**Oráculo.** Jar oficial: escenario propuesto `evt-logging-format.jsh` (casos A1-A6; incluye más args que `{}` y `{}` sin args).

**Decisiones.** Q-EVT-3

## AC-EVT-013.2 — BusLogging y NoLogging (publicación al bus y adaptador nulo)

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual] BusLogging publica Error/Warning/Info/Debug(logSource, logClass, message, mdc) en el bus si el LoggingFilter lo permite; NoLogging no hace nada y reporta todos los niveles deshabilitados. BusLogging.isXEnabled delega en el LoggingFilter con (logClass, logSource).

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:2026-2055
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:2058-2138

**Condiciones.**
- NoLogging: isErrorEnabled=false (actor/src/main/scala/org/apache/pekko/event/Logging.scala:2066) y todos los métodos no hacen nada.

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** Filas: AC-EVT-013.1 ; AC-EVT-016. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `BusLogging` y `NoLogging`

**Diferencias observables.** Rust: ídem.

**Aserciones.**
- A1: `system.log.error("Danger! Danger!")` llega a los loggers configurados — actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:158-181.
- A2: NoLogging no publica — derivada de actor/src/main/scala/org/apache/pekko/event/Logging.scala:2058-2090, *no afirmada por spec*.

**Oráculo.** Jar oficial: `evt-logging-format.jsh`.

**Decisiones.** ninguna específica

## AC-EVT-013.3 — LogSource y fábricas Logging.apply/getLogger/withMarker (obtención de adaptadores; logSource derivado de path/clase)

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual] `LogSource[T]` define cómo un objeto origen se convierte en el par `(logSource: String, logClass: Class)` de los `LogEvent`: `genString(t)` (variante sin sistema), `genString(t, system)` (variante que incluye la dirección del sistema) y `getClazz(t)` (por defecto `t.getClass`). Reglas predefinidas (objeto `LogSource`): `String` → la propia cadena (con sistema: `s + "(" + system + ")"`) y clase `DummyClassForStringSources`; `Actor` y `ActorRef` → `path.toString` (con sistema: `path.toStringWithAddress(provider.getDefaultAddress)`, con respaldo a `path.toString` si falla); `Class` → `Logging.simpleName(c)` (con sistema: `simpleName + "(" + system + ")"`) y la propia clase como `logClass`. Las fábricas `Logging(system|bus, src)`, `Logging.withMarker(system|bus, src)`, `Logging(actor)`, `Logging.withMarker(actor)` y `Logging.getLogger(system|bus|actor, ..)` aplican esas reglas y devuelven un `BusLogging`/`MarkerLoggingAdapter`/adaptador diagnóstico sobre el EventStream (o bus) dado.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:246-294 (`trait LogSource`), 289-296 (`ActorWithLogClass`, `DummyClassForStringSources`), 317-403 (`object LogSource`: reglas, `apply`, `fromAnyRef`)
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:448-475 (`simpleName`, `messageClassName`)
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:583-715 (fábricas `apply`, `withMarker`, `getLogger`)
- actor/src/main/scala/org/apache/pekko/actor/Actor.scala:342-352 (uso desde `ActorLogging`)
- specs: actor-tests/src/test/scala/org/apache/pekko/event/LoggingReceiveSpec.scala:75-95,97-129,165-181; actor-tests/src/test/scala/org/apache/pekko/event/jul/JavaLoggerSpec.scala:66-94 (1.7.1); actor-tests/src/test/java/org/apache/pekko/event/LoggingAdapterTest.java:51-87

**Precondiciones.** Para `apply(system, src)`/`withMarker(system, src)`/`getLogger(system, ..)` el sistema debe ser `ExtendedActorSystem` (usa `system.logFilter`, Logging.scala:575). Para el bus no hay filtro de sistema: se usa `DefaultLoggingFilter` (Logging.scala:613-615). `apply(T)` exige un `LogSource[T]` implícito (error de compilación si no existe).

**Comportamiento.**
- `Logging(system, src)`: `(str, clazz) = LogSource(src, system)`; `new BusLogging(system.eventStream, str, clazz, system.logFilter)` (Logging.scala:573-576). `withMarker(system, src)` igual con `MarkerLoggingAdapter` (598-601).
- `Logging(bus, src)`: `(str, clazz) = LogSource(src)` (sin dirección); `new BusLogging(bus, str, clazz)` (617-620); `withMarker(bus, src)` (636-639).
- `Logging(actor)` / `getLogger(actor)`: adaptador con MDC (`BusLogging with DiagnosticLoggingAdapter`) sobre `actor.context.system.eventStream` y su `logFilter` (645-649,705); `withMarker(actor)`: `DiagnosticMarkerBusLoggingAdapter` (655-659).
- `getLogger(system, AnyRef)`/`getLogger(bus, AnyRef)`: usan `LogSource.fromAnyRef`, que despacha por tipo en tiempo de ejecución: `Class` → `apply(c)`, `Actor` → `apply(a)`, `ActorRef` → `apply(a)`, `String` → `apply(s)`; cualquier otro objeto → `(simpleName(x) [+ "(" + system + ")"], x.getClass)` (Logging.scala:381-402). **Detalle no obvio:** la variante con sistema de `fromAnyRef` llama a `apply(c|a|s)` SIN sistema para `Class`/`Actor`/`ActorRef`/`String` (Logging.scala:397-400), de modo que `getLogger(system, "x")` produce `logSource = "x"` mientras `Logging(system, "x")` produce `"x(pekko://sys)"`.
- `Logging.simpleName(clazz)`: subcadena tras el último `.` del nombre de clase (sin tratar `$`) (Logging.scala:450-454). `messageClassName(msg)`: `"null"` para null, `ActorSelectionMessage(<clase del mensaje envuelto>)` para selecciones y `m.getClass.getName` en otro caso (461-465).

**Errores.** No hay excepciones propias; un objeto sin `LogSource` es error de compilación. `genString(actorRef, system)` captura `NonFatal` y devuelve `path.toString` si el sistema/remoting no terminó de arrancar (Logging.scala:332-337).

**Orden / concurrencia / ciclo de vida.** Los adaptadores son baratos y sin estado compartido salvo el MDC del adaptador diagnóstico (no thread-safe: sólo dentro del actor). El `logSource` se calcula una vez al crear el adaptador.

**Dependencias.** Filas: AC-EVT-013.2 ; AC-EVT-014 ; AC-EVT-015 ; AC-EVT-018 ; AC-PATH-015 ; AC-REF-001.3. Externas: reflexión JVM (`getClass`).

**Equivalente Rust propuesto.** `trait LogSource { fn gen_string(&self) -> String; fn gen_string_with_system(&self, system: &ActorSystem) -> String; fn log_class(&self) -> LogClass; }` con implementaciones para `&str`/`String` (clase marcadora `LogClass::StringSource`), `ActorRef` y el contexto/actor (path con dirección; clase = tipo del actor) y para tipos (`LogClass::of::<T>()` con `simple_name`). Fábricas `Logging::new(&system, &src)`, `Logging::on_bus(&bus, &src)`, `Logging::with_marker(..)`, `Logging::diagnostic(&ctx)`; `get_logger` se puede unificar con `new` mediante `&dyn LogSource`, preservando la discrepancia sobre la dirección del sistema sólo si el Owner decide mantenerla (ver Decisiones). `LogClass` es un token (`TypeId` + nombre) para filtrar y para el campo `logClass` (Q-EVT-1). Viabilidad alta.

**Diferencias observables.**
- `Class[_]` JVM → `LogClass` por `TypeId`/`type_name`; `simpleName` pasa a ser el último segmento de `type_name` (Q-EVT-1).
- `LogSource[T]` implícito → trait implementado explícitamente; el despacho por tipo de `fromAnyRef` se hace con `&dyn LogSource`/`dyn Any`.
- La asimetría `apply` vs `getLogger` (sistema incluido o no en el string) es un accidente del código Pekko: se propone conservarla en `getLogger`-equivalente sólo si algún spec la exige (ninguno lo hace); si se unifica, es una divergencia a registrar (Q-EVT-6).

**Aserciones.**
- A1: un `LoggingReceive` con origen `Some("funky")` publica un `Logging.Debug` cuyo `logSource` es `"funky"` y cuyo `logClass` es `classOf[DummyClassForStringSources]` (regla `String` de `fromAnyRef`) — actor-tests/src/test/scala/org/apache/pekko/event/LoggingReceiveSpec.scala:81-92.
- A2: el origen por defecto de `LoggingReceive` (el actor) produce `logSource == actor.path.toString` (sin dirección del sistema) y `logClass == actor.underlyingActor.getClass` (regla `Actor` de `fromAnyRef`) — actor-tests/src/test/scala/org/apache/pekko/event/LoggingReceiveSpec.scala:117-119 (y 140-142; con nivel Info: 174-176).
- A3: un actor con `ActorLogging` emite eventos cuyo `logClass` es la clase del actor — actor-tests/src/test/scala/org/apache/pekko/event/jul/JavaLoggerSpec.scala:77,92 (1.7.1) (`record.getSourceClassName == classOf[LogProducer].getName`, derivado de `logClass.getName` en JavaLogger.scala:53).
- A4: `Logging.getLogger(system, this)` devuelve un adaptador utilizable con formato `{}` (los eventos llegan al EventStream) — actor-tests/src/test/java/org/apache/pekko/event/LoggingAdapterTest.java:51-87 (no afirma el `logSource`).
- A5: `LogSource(String)` con sistema es `s + "(" + system + ")"` y sin sistema `s` — derivada de Logging.scala:318-322, *no afirmada por spec*. Caracterización propuesta.
- A6: `LogSource(Actor|ActorRef)` con sistema es `path.toStringWithAddress(defaultAddress)` (ej. `pekko://sys/user/a`) — derivada de Logging.scala:329-337, *no afirmada por spec* (A2 sólo cubre la variante sin sistema). Caracterización propuesta.
- A7: `LogSource(Class)` es `simpleName` (+ `"(" + system + ")"`) y `logClass` la propia clase — derivada de Logging.scala:351-356, *no afirmada por spec*.
- A8: `fromAnyRef` para un objeto arbitrario devuelve `(simpleName(x) [+ "(system)"], x.getClass)` — derivada de Logging.scala:387,401, *no afirmada por spec*.
- A9: `Logging.simpleName` no elimina el prefijo de clase externa (`Outer$Inner` se conserva) — derivada de Logging.scala:450-454, *no afirmada por spec*.
- A10: `messageClassName(ActorSelectionMessage(m, ..))` = `ActorSelectionMessage(<clase de m>)` y `null` → `"null"` — derivada de Logging.scala:461-465; el texto lo consume DeadLetterListener (AC-EVT-009), *no afirmada por spec aquí*.

**Validación prevista.** A1-A3: tests portados (se obtienen como aserciones secundarias de los specs de LoggingReceive y JavaLogger). A4: portado (propiedad del formato). A5-A10: caracterización propia (tabla de `LogSource` por tipo) y diferencial con el jar oficial (`tools/oracle/evt-logsource.jsh`: `LogSource.fromAnyRef` y `Logging.simpleName` sobre una tabla de objetos) justificado porque son cadenas visibles en los logs.

**Decisiones.** Q-EVT-1 (nombre de clase/`simpleName` en Rust), Q-EVT-6 (asimetría `getLogger` vs `apply`).

## AC-EVT-014 — DiagnosticLoggingAdapter (MDC) y DiagnosticActorLogging

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] mdc(currentMessage) se invoca por cada mensaje del actor y su resultado se adjunta a cada LogEvent emitido mientras procesa ese mensaje; luego se limpia. DiagnosticLoggingAdapter mantiene `_mdc` mutable; mdc(null) -> emptyMDC; clearMDC().

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:1636-1673

**Condiciones.**
- mdc(null) se normaliza a emptyMDC; setMDC(java null) -> emptyMDC (actor/src/main/scala/org/apache/pekko/event/Logging.scala:1662-1690).

**Orden / concurrencia / ciclo de vida.** mdc() se llama antes de cada mensaje (reqId incrementa por mensaje: 1,2,3,4).

**Dependencias.** Filas: AC-EVT-013.1. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** campos estructurados de `tracing` o `HashMap` en el adaptador

**Diferencias observables.** Rust: ídem.

**Aserciones.**
- A1: reqId crece 1,2,3,4 y MDC por mensaje — actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:241-274.
- A2: ErrorLevel MDC `{messageLength=8}`, Warning `{messageLength=9}`, Info `{messageLength=9}`, 4ª llamada Debug sin MDC y 5ª con `{messageLength=29}` — LoggingAdapterTest.java:92-112.
- A3: un actor cuyo `mdc` devuelve `null` publica el evento con MDC vacío (`{}`) — actor-tests/src/test/java/org/apache/pekko/event/LoggingAdapterTest.java:111-122 (auditoría 2026-10-31: el test `mustSupportMdcNull` envía `Log(InfoLevel, "Null MDC")` y espera `expectLog(InfoLevel(), "Null MDC", "{}")`).
- A4: `DiagnosticActorLogging.aroundReceive` llama `log.mdc(mdc(msg))` antes de procesar cada mensaje y `log.clearMDC()` en `finally`; `mdc` por defecto es `emptyMDC` — actor/src/main/scala/org/apache/pekko/actor/Actor.scala:372-384 (leído en la auditoría; el comportamiento por mensaje lo afirman A1-A2), *no afirmada por spec* de forma directa para el `finally`.

**Oráculo.** Jar oficial: `evt-logging-format.jsh` con actor DiagnosticActorLogging.

**Decisiones.** ninguna específica

## AC-EVT-015 — MarkerLoggingAdapter, LogMarker, ActorLogMarker

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] Un marker (nombre + propiedades) se adjunta al LogEvent (Error3/Warning3/Info3/Debug3); LogMarker.Security existe con nombre `SECURITY`. `ActorLogMarker.deadLetter(messageClass)` lo usa el DeadLetterListener.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:1700-1746
- actor/src/main/scala/org/apache/pekko/actor/ActorLogMarker.scala:24-34 (`ActorLogMarker.deadLetter`)
- actor/src/main/scala/org/apache/pekko/event/DeadLetterListener.scala:164 (único uso)

**Condiciones.**
- LogMarker.toString = `LogMarker(<name>,<properties>)` (actor/src/main/scala/org/apache/pekko/event/Logging.scala:1711; corregido en la auditoría 2026-10-31, antes 1699-1700).
- `ActorLogMarker.deadLetter(messageClass)` = `LogMarker("pekkoDeadLetter", Map("pekkoMessageClass" -> messageClass))` (ActorLogMarker.scala:33-34; la clave es `LogMarker.Properties.MessageClass`, Logging.scala:1731-1735). Los nombres de marcador y propiedades son API pública según el Scaladoc (ActorLogMarker.scala:20-24). `LogMarker.Security` es `LogMarker("SECURITY")` (Logging.scala:1726).

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** Filas: AC-EVT-013.1. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `struct LogMarker { name, properties }` y métodos con marker

**Diferencias observables.** Rust: ídem.

**Aserciones.**
- A1: markerLogging.info(LogMarker.Security, msg) publica Info3 con marker.name == "SECURITY" y message intacto — actor-tests/src/test/scala/org/apache/pekko/event/MarkerLoggingSpec.scala:28-36.
- A2: markerLogging.error(Security, cause, msg) publica Error con cause.getMessage == "Security Exception" — actor-tests/src/test/scala/org/apache/pekko/event/MarkerLoggingSpec.scala:38-46.
- A3: el DeadLetterListener publica el log de dead letters con el marcador `pekkoDeadLetter` y la propiedad `pekkoMessageClass` con la clase del mensaje — derivada de ActorLogMarker.scala:33-34 y DeadLetterListener.scala:147, *no afirmada por spec leída* (ver AC-EVT-009).

**Oráculo.** Jar oficial: `evt-logging-format.jsh` con marcadores.

**Decisiones.** ninguna específica

## AC-EVT-016 — LoggingFilter / DefaultLoggingFilter / LoggingFilterWithMarker

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual] DefaultLoggingFilter habilita un nivel si eventStream.logLevel >= nivel; pluggable por FQCN con pekko.logging-filter; LoggingFilterWithMarker añade marker. Con DefaultLoggingFilter(() => InfoLevel) isDebugEnabled=false.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:1573-1633
- reference.conf:28

**Condiciones.**
- LoggingFilterWithMarkerWrapper adapta un LoggingFilter sin marker (actor/src/main/scala/org/apache/pekko/event/Logging.scala:1605-1619).

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** Filas: AC-CFG-004. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** trait `LoggingFilter` + `DefaultLoggingFilter`

**Diferencias observables.** Rust: ídem.

**Aserciones.**
- A1: con loglevel=OFF el adaptador no loguea (isErrorEnabled=false) — actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:210-216.
- A2: DefaultLoggingFilter(()=>InfoLevel) permite info del MarkerLoggingAdapter — actor-tests/src/test/scala/org/apache/pekko/event/MarkerLoggingSpec.scala:22-36.

**Oráculo.** Jar oficial: ver AC-EVT-013.1.

**Decisiones.** Q-X-2

## AC-EVT-017 — DefaultLogger / StdOutLogger / StandardOutLogger: formato de salida, timestamps, MDC, marker, causa

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] Línea Error `[ERROR] [<ts>] [<thread>] [<source>]<mdc> <msg><stack>`; Warning/Info/Debug `[WARN|INFO|DEBUG] [<ts>] [<thread>] [<source>]<mdc> <msg>`; con marker se inserta `[<markerName>]` pegado a continuación del nivel (`[LEVEL] [<marker>][<ts>] ...`); timestamp `MM/dd/yyyy HH:mm:ss.SSS` en zona horaria del sistema; la causa se incluye aun sin stacktrace. El StandardOutLogger es un ActorRef mínimo síncrono; DefaultLogger lo envuelve como actor con LoggerMailbox.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:1005-1117 (StdOutLogger y formatos)
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:1126-1148 (StandardOutLogger)
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:1155-1160 (DefaultLogger)
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:1165-1174 (stackTraceFor)
- actor/src/main/scala/org/apache/pekko/util/Helpers.scala:110-115 (timestamp)

**Condiciones.**
- Formatos: ErrorFormat `[ERROR] [%s] [%s] [%s]%s %s%s`; ErrorFormatWithMarker `[ERROR] [%s][%s] [%s] [%s]%s %s%s`; sin causa `[ERROR] [%s] [%s] [%s]%s %s` (actor/src/main/scala/org/apache/pekko/event/Logging.scala:1101-1105); `[WARN] [%s] [%s] [%s]%s %s`, `[INFO] ...`, `[DEBUG] ...` y variantes con marker (actor/src/main/scala/org/apache/pekko/event/Logging.scala:1107-1114).
- formatMDC: vacío -> ''; uno -> `[k:v]`; varios -> `[k1:v1][k2:v2]` (actor/src/main/scala/org/apache/pekko/event/Logging.scala:1092-1097) — sin separador antes: el %s del formato va pegado a `[source]`.
- stackTraceFor: null/NoCause -> ''; NoStackTrace -> ` (<claseDeLaCausa>: <mensaje>)`; otro -> '\n' + printStackTrace (actor/src/main/scala/org/apache/pekko/event/Logging.scala:1165-1174).
- Evento inesperado: Warning `received unexpected event of class <cls>: <e>` (actor/src/main/scala/org/apache/pekko/event/Logging.scala:1016-1017).
- StandardOutLogger.! con null: InvalidMessageException("Message is null") (actor/src/main/scala/org/apache/pekko/event/Logging.scala:1131).

**Orden / concurrencia / ciclo de vida.** print por println síncrono; DefaultLogger procesa en su hilo (loggers-dispatcher).

**Dependencias.** Filas: AC-EVT-011 ; AC-EVT-012. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `StdOutFormatter` + `StandardOutLogger`

**Diferencias observables.** Rust: conservar bytes exactos si Q-EVT-2 lo acepta; thread = nombre del hilo del runtime.

**Aserciones.**
- A1: log.timestamp(event) == formato `MM/dd/yyyy HH:mm:ss.SSS` en zona del sistema — actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:320-328.
- A2: error con causa NoStackTrace incluye el mensaje de la causa en la salida — actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:330-344.
- A3: currentTimeMillisToUTCString devuelve `HH:mm:ss.SSSUTC` con ceros a la izquierda — actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:283-318 (Helpers, AC-UTIL).
- A4: la salida de un logger de sistema real aparece en stdout — actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:185-188.
- A5: los eventos no se pierden al cerrar: ver AC-EVT-020 — actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:190-206.
- A6: formato exacto de cada nivel con/sin marker y MDC — derivada de actor/src/main/scala/org/apache/pekko/event/Logging.scala:1020-1117, *no afirmada por spec*.
- A7: formatMDC para 1 y N entradas — derivada de actor/src/main/scala/org/apache/pekko/event/Logging.scala:1092-1097, *no afirmada por spec*.

**Oráculo.** Jar oficial: escenario propuesto `evt-stdout-format.jsh` con zona horaria fijada (-Duser.timezone=UTC) y thread name controlado; es el candidato principal de oráculo (bytes de stdout).

**Decisiones.** Q-EVT-2, Q-X-7

## AC-EVT-018 — ActorLogging (log perezoso con logSource = path del actor)

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual] El trait `ActorLogging` aporta `log: LoggingAdapter` creado al primer acceso con `pekko.event.Logging(context.system, this)` (origen = el actor, sistema = el de su contexto) y cacheado en un campo no volátil (sólo seguro dentro del actor). `DiagnosticActorLogging` (AC-EVT-014) es el equivalente con MDC y creación eager.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/Actor.scala:330-352 (`ActorLogging`)
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:334-348,583-586 (regla `Actor` de `LogSource` y fábrica)
- specs: actor-tests/src/test/scala/org/apache/pekko/event/jul/JavaLoggerSpec.scala:36-43,62-94 (1.7.1); (usos sin aserción propia: actor-tests/src/test/scala/org/apache/pekko/routing/BalancingSpec.scala:35, actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerStressSpec.scala:35)

**Precondiciones.** El trait sólo se mezcla en un `Actor` (autotipo `this: Actor =>`). El primer acceso a `log` debe ocurrir con `context` disponible (no en el constructor antes de que exista el contexto... en la práctica `context` se inicializa antes del constructor del actor: Actor.scala, ver AC-CELL).

**Comportamiento.** `log`: `if (_log eq null) _log = Logging(context.system, this); _log` (Actor.scala:343-348). El adaptador resultante es un `BusLogging` sobre `system.eventStream` con `logSource = path.toStringWithAddress(defaultAddress)` y `logClass = this.getClass` (AC-EVT-013.3). Los métodos del adaptador (`error/warning/info/debug` con `{}`) se especifican en AC-EVT-013.1. El nivel efectivo lo decide el `LoggingFilter` del sistema (AC-EVT-016).

**Errores.** Ninguno propio.

**Orden / concurrencia / ciclo de vida.** Creación perezosa única por instancia de actor; tras un reinicio se crea un `_log` nuevo (nueva instancia de actor). No es thread-safe fuera del actor (comentario del código: "only used in Actor, i.e. thread safe").

**Dependencias.** Filas: AC-EVT-013.1 ; AC-EVT-013.3 ; AC-EVT-016 ; AC-CELL-001. Externas: ninguna.

**Equivalente Rust propuesto.** `ctx.log()` (o `self.log(ctx)`) en el contexto del actor, con `OnceCell<LoggingAdapter>` en el estado interno del contexto; origen = el actor. Viabilidad alta: el adaptador se obtiene del `ActorContext` y no necesita campo en el actor.

**Diferencias observables.** Rust: `ctx.log()` en lugar de un mixin; la fuente incluye el nombre de tipo del actor (Q-EVT-1) en lugar del nombre de clase JVM.

**Aserciones.**
- A1: el mensaje con `{}` se formatea con el argumento: `log.info(s, x)` con `("{} is the magic number", 3)` produce el mensaje `"3 is the magic number"` en el evento y `log.error(e, e.getMessage)` produce `"Simulated error"` con la causa `e` — actor-tests/src/test/scala/org/apache/pekko/event/jul/JavaLoggerSpec.scala:66-94 (1.7.1) (aserciones sobre el `LogRecord` del `JavaLogger`: nivel, mensaje, `thrown`).
- A2: los eventos de un actor con `ActorLogging` llevan `logClass` igual a la clase del actor — actor-tests/src/test/scala/org/apache/pekko/event/jul/JavaLoggerSpec.scala:77,92 (1.7.1).
- A3: `log` crea el adaptador en el primer acceso y devuelve la misma instancia después — derivada de Actor.scala:343-348, *no afirmada por spec*. Caracterización propuesta: dos llamadas a `ctx.log()` devuelven adaptadores con el mismo `logSource`.
- A4: el `logSource` es el path del actor con la dirección del sistema (`pekko://sys/user/x`) — derivada de Actor.scala:346 + Logging.scala:329-337, *no afirmada por spec*. Caracterización propuesta con un `EventStream` suscrito.
- A5: tras un reinicio el actor nuevo obtiene un `log` nuevo con el mismo `logSource` — derivada de Actor.scala:341, *no afirmada por spec*.

**Validación prevista.** A1-A2: portados desde JavaLoggerSpec (reescritos contra un backend de prueba o el EventStream directamente). A3-A5: caracterización propia. Diferencial con el jar oficial para A4 (`tools/oracle/evt-actorlogging-source.jsh`: imprimir `logSource` y `logClass` de un evento) por tratarse de texto visible.

**Decisiones.** Q-EVT-1 (nombre del tipo en `logClass`).

## AC-EVT-019 — LoggingReceive (decora receive con log Debug; activable por debug.receive)

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] `LoggingReceive(r)` (y `withLabel`, `apply(level)`, `create` Java) envuelve un `Receive` en un `LoggingReceive` SÓLO si `pekko.actor.debug.receive = on` (`Settings.AddLoggingReceive`); si no, devuelve `r` sin cambios. Si `r` ya es un `LoggingReceive`, no lo envuelve otra vez. El decorador registra en `isDefinedAt(o)` (es decir, ANTES de ejecutar el cuerpo) un evento al nivel configurado (Debug por defecto) con el texto `received <handled|unhandled> message <o> from <context.sender()>` (+ `" in state <label>"` si hay etiqueta) cuando `eventStream.logLevel >= nivel`. El origen del evento es el actor del contexto o la fuente explícita; si ese origen es un `DiagnosticActorLogging`, el evento incluye su MDC (`LogEvent` con MDC → `Debug2`/`Info2`…). `apply(o)` delega en `r(o)` sin registrar.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/LoggingReceive.scala:28-108
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:454 (`AddLoggingReceive`)
- reference.conf: `pekko.actor.debug.receive`
- specs: actor-tests/src/test/scala/org/apache/pekko/event/LoggingReceiveSpec.scala:37-71,73-183

**Precondiciones.** Se necesita un `ActorContext` implícito (el contexto del actor que lo construye). Los specs usan el sistema `appLogging` con `pekko.actor.debug.receive = true` y `pekko.loglevel = DEBUG` (LoggingReceiveSpec.scala:39-43) y silencian Debug/Info ajenos con `TestEvent.Mute` (51-58).

**Comportamiento.**
- `withLabel(label, level)(r)`: `r match { case _: LoggingReceive => r; case _ => if (AddLoggingReceive) new LoggingReceive(None, r, Option(label), level) else r }` (LoggingReceive.scala:64-67). `apply(r)` ≡ `withLabel(null)(r)` con nivel `DebugLevel`; `apply(level)(r)` fija el nivel.
- `isDefinedAt(o)`: `handled = r.isDefinedAt(o)`; si `eventStream.logLevel >= logLevel`: `src = source.getOrElse(context.actor)`; `(str, clazz) = LogSource.fromAnyRef(src)`; mensaje según arriba; evento `LogEvent(logLevel, str, clazz, message[, mdc])`; `eventStream.publish(event)`; devuelve `handled` (LoggingReceive.scala:87-105).
- Efecto sobre composición: al ser `isDefinedAt` el punto de registro, un `orElse` que atiende antes el mensaje NO registra (el `LoggingReceive` no llega a consultarse) — ver A2.

**Errores.** Ninguno propio. Un nivel no soportado por `LogEvent.apply` (p. ej. `OffLevel`) lanza `IllegalArgumentException("Unsupported log level [..]")` al construir el evento (Logging.scala:772-777).

**Orden / concurrencia / ciclo de vida.** El evento se publica ANTES de ejecutar el cuerpo del handler (el orden `Debug` → respuesta `"x"` es observable y lo afirma el spec). Se ejecuta en el hilo del actor; `context.sender()` es el del mensaje actual.

**Dependencias.** Filas: AC-CFG-011 ; AC-EVT-012 ; AC-EVT-013.3 ; AC-EVT-014 ; AC-CELL-004 ; AC-FSM-002. Externas: ninguna.

**Equivalente Rust propuesto.** `logging_receive(ctx, receive)` / `logging_receive_at(ctx, level, receive)` / `with_label(ctx, label, level, receive)` que devuelven el mismo `Receive` si `debug.receive` está apagado y un envoltorio si está encendido. **Restricción de diseño:** para conservar "registrar antes de ejecutar" y el texto `handled/unhandled`, el `Receive` de rakka debe exponer `is_defined_at(&msg) -> bool` separado de `apply` (tabla de casos tipados con guardas) o una operación equivalente `try_handle` que informe de "no definido" sin efectos; un handler que sólo se conoce al ejecutarlo obligaría a registrar DESPUÉS del handler, rompiendo el orden observable (Q-EVT-7). Viabilidad: alta con `Receive` basado en tabla; limitada con closures opacas.

**Diferencias observables.**
- `from <context.sender()>` y `deadLetters` usan el `toString` de `ActorRef` (`Actor[pekko://sys/..]`, Q-EVT-1).
- El `message` impreso usa `o.toString` del mensaje (Debug/Display del tipo Rust; un mensaje sin `Display` necesita `Debug`).
- Mensajes unhandled posteriores (`UnhandledMessage`) los publica la celda, no el decorador (AC-EVT-010).

**Aserciones.**
- A1: un `LoggingReceive(Some("funky"), { case null => })` que recibe `"hallo"` publica exactamente `Logging.Debug("funky", classOf[DummyClassForStringSources], "received unhandled message hallo from " + system.deadLetters)` y después un `UnhandledMessage` — actor-tests/src/test/scala/org/apache/pekko/event/LoggingReceiveSpec.scala:75-95.
- A2: con `debug.receive = on`, `switch.orElse(LoggingReceive { case _ => sender() ! "x" })` registra `Debug(actor.path.toString, actor.underlyingActor.getClass, "received handled message buh from " + self)` y luego entrega `"x"`; tras `context.become(r, false)` con un `Receive` sin decorador, un mensaje `"bah"` produce un `UnhandledMessage("bah", _, actor)` y ningún Debug de LoggingReceive; el mensaje `"becomenull"` (atendido por `switch`, antes del decorador en el `orElse`) no produce Debug — actor-tests/src/test/scala/org/apache/pekko/event/LoggingReceiveSpec.scala:97-129 (la ausencia de Debug para `"becomenull"` se deduce de que `expectMsgPF` en la línea 124 recibe `UnhandledMessage` como primer mensaje).
- A3: `LoggingReceive(LoggingReceive { .. })` no duplica el registro: tras `"buh"` llega un único `Debug` y después `"x"` — actor-tests/src/test/scala/org/apache/pekko/event/LoggingReceiveSpec.scala:131-146.
- A4: con un actor `DiagnosticActorLogging` cuyo `mdc` devuelve `Map("hello" -> "mdc")`, el evento es un `Logging.Debug2` con `m.mdc == myMDC` — actor-tests/src/test/scala/org/apache/pekko/event/LoggingReceiveSpec.scala:148-163.
- A5: `LoggingReceive(Logging.InfoLevel) { .. }` publica un `Logging.Info` con `src == actor.path.toString` y mensaje `"received handled message buh from " + self` (suscripción a `Logging.Info`), seguido de `"x"` — actor-tests/src/test/scala/org/apache/pekko/event/LoggingReceiveSpec.scala:165-181.
- A6: con `debug.receive = off`, `LoggingReceive(r)` devuelve `r` y no se publica nada — derivada de LoggingReceive.scala:66, *no afirmada por spec*. Caracterización propuesta.
- A7: sólo se registra si `eventStream.logLevel >= logLevel` (con `loglevel = INFO` y nivel Debug no hay evento) — derivada de LoggingReceive.scala:89, *no afirmada por spec*. Caracterización propuesta.
- A8: `withLabel("L")` añade `" in state L"` al final del mensaje — derivada de LoggingReceive.scala:94-97, *no afirmada por spec aquí* (lo ejerce FSM con debug.fsm: AC-FSM).
- A9: `LoggingReceive.apply(o)` delega en el `Receive` sin registrar — derivada de LoggingReceive.scala:106, *no afirmada por spec*.

**Validación prevista.** A1-A5: tests portados 1:1 (usando el harness de TestActorRef de AC-TK-006.1 o un actor normal con probe; el orden Debug-antes-de-respuesta exige el `Receive` con `is_defined_at`, ver Equivalente). A6-A9: caracterización propia. Diferencial con el jar oficial para el texto exacto (`tools/oracle/evt-loggingreceive.jsh`: capturar el `Debug` de un actor con `debug.receive=on`), justificado porque el formato del mensaje y del `logSource` es contractual por spec.

**Decisiones.** Q-EVT-1 (representación textual de refs y tipos), Q-EVT-7 (el `Receive` debe exponer `is_defined_at`; arquitectura), Q-X-3 (mensajes dinámicos).

## AC-EVT-020 — LoggerMailbox (vacía la cola de logs al cerrar)

*Unidad EVT · visibilidad internal-required · analysis `extracted`*

**Contrato observable.** [contractual por spec] Al limpiar el mailbox del logger (cleanUp), los LogEvent pendientes con level <= logLevel se imprimen por StandardOutLogger para no perderlos; si el nivel es OFF no se imprime. LoggerMailboxType exige owner y system (`no mailbox owner or system given`).

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/LoggerMailbox.scala:29-75
- reference.conf:731 (logger-queue)

**Condiciones.**
- LoggerMailboxType.create sin owner/system: IllegalArgumentException("no mailbox owner or system given") (actor/src/main/scala/org/apache/pekko/event/LoggerMailbox.scala:41).

**Orden / concurrencia / ciclo de vida.** cleanUp ocurre tras cambiar el mailbox por el de dead letters (comentario actor/src/main/scala/org/apache/pekko/event/LoggerMailbox.scala:57-58).

**Dependencias.** Filas: AC-MBX-004. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** mailbox del logger con drenaje final

**Diferencias observables.** Rust: ídem si hay logger-actor.

**Aserciones.**
- A1: con un logger lento (500 ms en msg1), tres errores seguidos y shutdownActorSystem(verifySystemShutdown=true): stdout tiene 3 líneas, la primera con msg1 y la última con msg3 — actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:190-206.
- A2: eventos con level > logLevel se descartan en cleanUp — derivada de actor/src/main/scala/org/apache/pekko/event/LoggerMailbox.scala:64, *no afirmada por spec*.

**Oráculo.** Jar oficial: `evt-logger-drain.jsh` (SlowLogger de LoggerSpec).

**Decisiones.** Q-X-7

## AC-EVT-021 — JavaLogger / JavaLoggingFilter (java.util.logging)

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual, deprecado desde Akka 2.6.0] `JavaLogger` es un actor logger que traduce cada `LogEvent` del EventStream a un `java.util.logging.LogRecord` y lo entrega al `logging.Logger` correspondiente; `JavaLoggingFilter` filtra en origen según el nivel del backend JUL. Fábrica `jul.Logger`: nombre del logger = `logSource` si `logClass == DummyClassForStringSources`, si no `logClass.getName`. Mapa de niveles: Info→INFO, Debug→CONFIG, Warning→WARNING, Error→SEVERE, cualquier otro→FINE. El trait `JavaLogging` aporta un `lazy val log` JUL con el nombre de la clase.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/jul/JavaLogger.scala:29-121 (1.7.1)
- reference.conf: `pekko.loggers`, `pekko.logging-filter` (se activan por FQCN)
- specs: actor-tests/src/test/scala/org/apache/pekko/event/jul/JavaLoggerSpec.scala:26-97 (1.7.1)

**Precondiciones.** Se activa con `pekko.loggers = ["org.apache.pekko.event.jul.JavaLogger"]` y `pekko.logging-filter = "org.apache.pekko.event.jul.JavaLoggingFilter"` (JavaLoggerSpec.scala:29-34).

**Comportamiento.**
- `receive`: `Error(cause, ..)` → `log(SEVERE, cause, event)`; `Warning`/`Info`/`Debug` → `log(level, null, event)`; `InitializeLogger(_)` → loguea en `Logger(getClass.getName)` el aviso `"<clase> has been deprecated since Akka 2.6.0. Use SLF4J instead."` y responde `LoggerInitialized` (JavaLogger.scala:36-45).
- `log(level, cause, event)`: `logger = Logger(event.logClass, event.logSource)`; `LogRecord(level, String.valueOf(event.message))`; `setLoggerName(logger.getName)`; `setThrown(cause)`; `setThreadID(event.thread.getId.toInt)`; `setSourceClassName(event.logClass.getName)`; `setSourceMethodName(null)`; `logger.log(record)` (JavaLogger.scala:47-56).
- `JavaLoggingFilter.isXEnabled(logClass, logSource)` = `eventStream.logLevel >= X && Logger(logClass, logSource).isLoggable(mapLevel(X))` (JavaLogger.scala:113-120).
- El mensaje que recibe el logger ya está formateado (`{}` resuelto por el adaptador, AC-EVT-013.1).

**Errores.** Ninguno propio; un fallo del handler JUL se propaga al actor logger (supervisado por /system).

**Orden / concurrencia / ciclo de vida.** Corre como actor de `/system` con `RequiresMessageQueue[LoggerMessageQueueSemantics]` (AC-EVT-020); el orden de registros respeta el orden de publicación por emisor.

**Dependencias.** Filas: AC-EVT-011 ; AC-EVT-012 ; AC-EVT-013.3 ; AC-EVT-016 ; AC-EVT-020. Externas: `java.util.logging` (JVM).

**Equivalente Rust propuesto.** Puente opcional (feature `log-bridge`) a la fachada `log`: `LogBridgeActor` que convierte cada `LogEvent` en un `log::Record` con `target` = `logSource` (para orígenes string) o el nombre del tipo, nivel Error→`Error`, Warning→`Warn`, Info→`Info`, Debug→`Debug` (JUL CONFIG no tiene equivalente), otros→`Trace`; la causa del `Error` y el hilo viajan como pares clave-valor (`cause`, `thread_id`); `LogBridgeFilter` implementa `isXEnabled` con `log::log_enabled!`. Viabilidad media: el contenido (nivel, mensaje, causa, hilo, clase origen) se conserva pero `LogRecord.getMillis/getSourceMethodName` no existen en `log`; para los tests se define un `RecordingLogger: log::Log` que captura una estructura `JulLikeRecord { level, message, thrown, thread_id, source_class, source_method: None, millis }`.

**Diferencias observables.** Niveles JUL → niveles `log` (CONFIG pasa a Debug, FINE a Trace); `thrown` es un valor en claves; `LogRecord.sourceMethodName` siempre ausente (igual que Pekko: "lost forever").

**Aserciones.**
- A1: un `Error` con causa produce un registro con `millis != 0`, `threadID != 0`, nivel `SEVERE`, mensaje `"Simulated error"`, `thrown` de la clase de la causa, `sourceClassName` = clase del productor y `sourceMethodName == null` — actor-tests/src/test/scala/org/apache/pekko/event/jul/JavaLoggerSpec.scala:66-79 (1.7.1).
- A2: un `Info` con plantilla `"{} is the magic number"` y argumento `3` produce un registro con `millis != 0`, `threadID != 0`, nivel `INFO`, mensaje `"3 is the magic number"`, `thrown == null`, `sourceClassName` = clase del productor y `sourceMethodName == null` — actor-tests/src/test/scala/org/apache/pekko/event/jul/JavaLoggerSpec.scala:81-94 (1.7.1).
- A3: `mapLevel`: Info→INFO, Debug→CONFIG, Warning→WARNING, Error→SEVERE, otros→FINE — derivada de JavaLogger.scala:95-101, *no afirmada por spec*. Caracterización propuesta (tabla).
- A4: `Logger(classOf[DummyClassForStringSources], "name")` resuelve el logger `"name"`; con otra clase, el logger se llama como la clase — derivada de JavaLogger.scala:85-88, *no afirmada por spec*.
- A5: `InitializeLogger` responde `LoggerInitialized` y emite el aviso de deprecación — derivada de JavaLogger.scala:41-44, *no afirmada por spec*.
- A6: `JavaLoggingFilter.isDebugEnabled` es `true` sólo si el nivel del EventStream permite Debug y el backend permite `CONFIG` — derivada de JavaLogger.scala:119-120, *no afirmada por spec*.

**Validación prevista.** A1-A2: tests portados con el `RecordingLogger` (los campos `millis`/`thread id` se comprueban no nulos). A3-A6: caracterización propia. Sin oráculo para A3-A6 (mapa trivial y dependiente del backend JUL), justificado.

**Decisiones.** Q-EVT-4 (puente opcional a la fachada `log`; efecto: la fila se implementa detrás de una feature y los campos exclusivos de JUL se representan como pares clave-valor).

## AC-EVT-022 — API Java del EventBus (event/japi/EventBusJavaAPI.scala): clases abstractas LookupEventBus, SubchannelEventBus, ScanningEventBus, ManagedActorEventBus

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual, API Java] Fachada de composición sobre los traits Scala de AC-EVT-002..005: el trait `japi.EventBus[E, S, C]` (`subscribe`, `unsubscribe(sub, from)`, `unsubscribe(sub)`, `publish`) y cuatro clases abstractas que internamente construyen un bus Scala anónimo y delegan 1:1: `LookupEventBus` (usuario implementa `mapSize`, `compareSubscribers`, `classify`, `publish(event, subscriber)`), `SubchannelEventBus` (`subclassification`, `classify`, `publish(event, subscriber)`), `ScanningEventBus` (`compareClassifiers`, `compareSubscribers`, `matches`, `publish(event, subscriber)`) y `ManagedActorEventBus[E](system)` (`mapSize`, `classify`; suscriptores y clasificadores son `ActorRef`). La semántica de resultado booleano, desuscripción y publicación es la de AC-EVT-001 y de cada clasificación; las fachadas no añaden comportamiento.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/japi/EventBusJavaAPI.scala:20-230
- actor/src/main/scala/org/apache/pekko/event/EventBus.scala (traits subyacentes: AC-EVT-001..005)
- specs: no hay en pekko-actor; el uso está en el módulo de documentación (fuera de los cuatro módulos del alcance de archivos): docs/src/test/java/jdocs/event/EventBusDocTest.java:277-350 (se cita sólo como evidencia de uso)

**Precondiciones.** El usuario hereda de la clase abstracta e implementa los métodos protegidos/públicos listados. `ManagedActorEventBus` exige el `ActorSystem` en el constructor (EventBusJavaAPI.scala:203).

**Comportamiento.** Cada clase mantiene un `private val bus = new pekko.event.EventBus with <Classification> { type Event = E; type Subscriber = S; type Classifier = C; … }` cuyos métodos abstractos reenvían a los de la clase abstracta (EventBusJavaAPI.scala:58-73,109-122,153-169,204-214) y sus `subscribe/unsubscribe/publish` reenvían a `bus` (EventBusJavaAPI.scala:95-98,139-142,191-194,226-229). Nota: `ManagedActorEventBus` usa `ActorEventBus with ManagedActorClassification with ActorClassifier`, por lo que hereda la desuscripción automática al morir un suscriptor/clasificador (AC-EVT-005).

**Errores.** Los de las clasificaciones subyacentes (p. ej. `compareSubscribers` inconsistente → comportamiento indefinido del conjunto ordenado). Las fachadas no capturan nada.

**Orden / concurrencia / ciclo de vida.** Igual que cada clasificación: la publicación entrega en el hilo del publicador; Lookup/Scanning usan estructuras concurrentes (ver AC-EVT-002/004).

**Dependencias.** Filas: AC-EVT-001 ; AC-EVT-002 ; AC-EVT-003 ; AC-EVT-004 ; AC-EVT-005 ; AC-JAPI-001. Externas: ninguna.

**Equivalente Rust propuesto.** No existe la distinción Java/Scala: las clasificaciones de AC-EVT-002..005 se exponen una sola vez como tipos genéricos parametrizados por un trait de personalización (`LookupBus<B: LookupBusSpec>`, `SubchannelBus<..>`, `ScanningBus<..>`, `ManagedActorBus<..>`) con los mismos puntos de personalización (`classify`, `compare_subscribers`, `map_size`, `publish_to`, `subclassification`, `compare_classifiers`, `matches`). Esta fila queda cubierta por AC-EVT-002..005 y no genera código adicional; lo que sí se exige es que el conjunto de puntos de personalización sea el mismo (paridad de API). Viabilidad alta.

**Diferencias observables.** Ninguna de comportamiento. Diferencia de API: una sola superficie en lugar de dos (Q-EVT-5).

**Aserciones.** Los specs del repositorio de Pekko para estas clases viven en el módulo de documentación; aquí se citan como evidencia y se reasignan a los tests de las clasificaciones:
- A1: `LookupEventBus` con clasificador de `MsgEnvelope.topic`: tras `subscribe(testActor, "greetings")`, publicar `("time", ..)` y `("greetings", "hello")` entrega sólo `"hello"` — docs/src/test/java/jdocs/event/EventBusDocTest.java:277-290.
- A2: `SubchannelEventBus` con subclasificación "empieza por": con suscripción `"abc"`, `("xyzabc")` y `("bcdef")` no se entregan y `("abc")` y `("abcdef")` sí — docs/src/test/java/jdocs/event/EventBusDocTest.java:292-309.
- A3: `ScanningEventBus` con `matches = event.length <= classifier`: con suscripción `3`, `"xyzabc"` no se entrega y `"ab"` y `"abc"` sí — docs/src/test/java/jdocs/event/EventBusDocTest.java:310-326.
- A4: `ManagedActorEventBus`: el suscriptor 1 recibe la notificación del observador 1; el suscriptor 2, suscrito a los observadores 1 y 2, recibe ambas; el suscriptor 1 no recibe la del observador 2 (`expectNoMessage(500 ms)`) — docs/src/test/java/jdocs/event/EventBusDocTest.java:327-348.
- A5: delegación 1:1: el resultado booleano de `subscribe/unsubscribe` de la fachada coincide con el del bus subyacente — derivada de EventBusJavaAPI.scala:95-97, *no afirmada por spec*. En Rust se cubre por AC-EVT-001 A1-A3.

**Validación prevista.** A1-A4 se cubren portando EventBusSpec (AC-EVT-001..005) con las cuatro clasificaciones y, adicionalmente, un test de paridad de API que instancia cada bus con los puntos de personalización del doc test (sin crates adicionales). No hay oráculo: la fila no tiene lógica propia (fachada), justificado.

**Decisiones.** Q-EVT-5 (efecto: una sola API en Rust; la fila no genera módulo propio y queda verificada por AC-EVT-002..005).

## AC-TK-001.1 — TestKitBase/TestKit: testActor (cola interna, actor de sistema sobre CallingThreadDispatcher), lastSender, msgAvailable, ignoreMsg/ignoreNoMsg, watch/unwatch, setAutoPilot

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] testActor es un actor creado con systemActorOf bajo /system con nombre `<testActorName>-<N>` (N global creciente), ejecutado en CallingThreadDispatcher; encola en una cola bloqueante (LinkedBlockingDeque) cada mensaje (mensaje, sender); el resto de aserciones consumen esa cola. El TestKit es el receptor de las respuestas de los actores bajo prueba; `testActor`, `lastSender`, `ignoreMsg`, `watch`, `setAutoPilot` son mensajes internos (SetIgnore, Watch, UnWatch, SetAutoPilot, Spawn) procesados por TestActor.

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:36-150 (TestActor)
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:167-198 (TestKitBase, testActor)
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:212-240
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:285
- testkit/src/main/resources/reference.conf:31-33 (calling-thread-dispatcher)

**Condiciones.**
- testActor lazy: `systemActorOf(TestActor.props(queue).withDispatcher(CallingThreadDispatcher.Id), "%s-%d".format(testActorName, testActorId.incrementAndGet))` y espera a `isStarted` si es RepointableRef (hasta 3 s dilatados, polling 10 ms) (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:188-198).
- lastMessage = NullMessage hasta que alguna aserción desencola; `lastSender`/`lastMessage.msg` sobre NullMessage lanzan IllegalActorStateException("last receive did not dequeue a message") (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:70-73).
- ignoreMsg(f): los mensajes para los que f devuelve true no se encolan (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:140-141); ignoreNoMsg() lo borra.
- Mensajes no AnyRef (primitivos) llegan boxed: `case x: AnyRef` (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:135).

**Orden / concurrencia / ciclo de vida.** El testActor procesa en el hilo del emisor (CallingThreadDispatcher): cuando `tell` retorna, el mensaje ya está en la cola o descartado. watch/unwatch/setAutoPilot/ignoreMsg se envían como mensajes al testActor y por tanto también son síncronos.

**Dependencias.** Filas: AC-SYS-010 ; AC-DSP-001 ; AC-TK-006.2 ; AC-DW-001. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `TestKit::new(&system)` con canal propio; `test_actor() -> ActorRef`

**Diferencias observables.** Rust: canal (`crossbeam`/`tokio::mpsc`) + actor de test con dispatcher inline (Q-TK-1). Diferencia: ninguna observable si la entrega es síncrona; si no, `expectMsg(0.millis)` de los specs portados deja de ser válido.

**Aserciones.**
- A1: `tk.ref ? "hello"` y `tk.expectMsg(0.millis, "hello")` inmediato; luego `lastMessage.sender ! "world"` completa el future de forma síncrona (`future.isCompleted == true`) — testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:32-39.
- A2: `tk1.ref.!("hello")(tk2.ref)`; `tk1.expectMsg(0.millis,..)`; `lastMessage.sender ! "world"` -> `tk2.expectMsg(0.millis, "world")` — testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:41-48.
- A3: ignoreMsg{case 42 => true}: 42 se descarta y "pigdog" llega — testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:193-198.
- A4: probe.watch(target muerto) con cola no vacía: llega "hello" y luego Terminated(target)(false,false) — testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:200-211.
- A5: nombre por defecto empieza por `testProbe`; con nombre `worker` empieza por `worker` — testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:213-221.
- A6: nombres de actor de sistema `testActor-N`/`testProbe-N` aparecen como `Actor[pekko://<sys>/system/testProbe..` en logs — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:177,181.
- A7: al detenerse el testActor, los mensajes pendientes se reenvían a deadLetters como DeadLetter(m, sender, self) — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:144-149, *no afirmada por spec*.

**Oráculo.** Jar oficial (pekko-testkit 1.7.1) vía jshell: escenario propuesto `tk-testactor.jsh` (path y nombre, orden síncrono). Procedencia: jars oficiales (requiere fetch-jars con pekko-testkit).

**Decisiones.** Q-TK-1, Q-TK-4

## AC-TK-001.2 — TestKit expect*: expectMsg, expectMsgPF, expectMsgType/Class, expectMsgAnyOf/AnyClassOf, expectMsgAllOf/AllClassOf/AllConformingOf, expectTerminated

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] Cada expect* toma el siguiente mensaje de la cola con timeout dilatado (por defecto remainingOrDefault) y falla con AssertionError cuyo mensaje contiene el hint si vence o no coincide. El timeout de las variantes con duración se dilata (`max.dilated`); `receiveOne` NO dilata.

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:447-473 (expectMsg)
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:485-491 (expectMsgPF)
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:502-505 (expectTerminated)
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:551-565 (expectMsgType)
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:631-690 (expectMsgAllOf/AllClassOf)

**Condiciones.**
- expectMsg vence: AssertionError `timeout (<max>) during expectMsg while waiting for <obj>` + `: <hint>` (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:470).
- expectMsg distinto: AssertionError `expected <obj>, found <o>` + `: <hint>` (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:471).
- expectMsgPF sin match: `expected: <hint> but got unexpected message <o>` (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:489); timeout: `timeout (<max>) during expectMsg: <hint>` (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:488).
- expectMsgAllOf: recibe n=obj.size mensajes; error `not found [..] found unexpected [..]` (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:648-663).
- expectMsgAllClassOf compara clases exactas con BoxedType (primitivos boxed); AllConformingOf compara por conformidad (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:684-690).

**Orden / concurrencia / ciclo de vida.** Consume exactamente un mensaje por expect (excepto *All*, n mensajes). Mensajes no coincidentes se pierden (se consumieron).

**Dependencias.** Filas: AC-TK-001.1 ; AC-TK-001.5 ; AC-TK-004. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `expect_msg(&v)`, `expect_msg_type::<T>()`, `expect_msg_pf(f)`, `expect_msg_all_of(&[..])`, `expect_terminated(&ref)`

**Diferencias observables.** Rust: aserciones por `panic!`/`assert!` o `Result`; el AssertionError JVM pasa a `TestFailure` (Q-X-5). Mensajes de fallo textuales: conservar si algún spec porteado los afirma (sólo el hint, testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:93-124).

**Aserciones.**
- A1: `expectMsg(0.millis, hint, "hello")` sin mensajes -> AssertionError cuyo mensaje contiene `some hint` — testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:107-114.
- A2: `expectMsg(0.millis, hint, "bye")` con "hello" en cola -> AssertionError que contiene el hint — testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:116-124.
- A3: expectMsgType[Int] / expectMsgAnyClassOf(classOf[Int]) / expectMsgAllClassOf(classOf[Int]) / expectMsgAllConformingOf(classOf[Int]) (con y sin duración) aceptan el primitivo 42 boxed — testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:151-160.
- A4: expectTerminated(target) devuelve el Terminated si el testActor vigila a target — testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:200-211 (usa expectMsg con Terminated).
- A5: `expectMsg(max, obj)` dilata `max` por timefactor — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:456, *no afirmada por spec leída*.
- A6: expectMsgAllOf informa `not found`/`found unexpected` — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:648-663, *no afirmada por spec*.
- A7: expectMsgPF devuelve f(o) — testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:485-491, *no afirmada por spec leída*.
- A8: expectMsgAnyOf/AnyClassOf/AllConformingOf — pendiente de leer cuerpos (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:588-717).

**Oráculo.** Jar oficial pekko-testkit: `tk-expect.jsh` comparando textos de AssertionError (A1-A2, A6).

**Decisiones.** Q-TK-4, Q-X-5

## AC-TK-001.3 — TestKit expectNoMessage / expectNoMsg

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] expectNoMessage() espera pekko.test.expect-no-message-default (100 ms) dilatado; expectNoMessage(max) NO dilata; si llega un mensaje lo extrae de la cola y lanza AssertionError `assertion failed: received unexpected message <m> after <n> millis`; expectNoMsg(max) (deprecado) sí dilata. Polling con espera geométrica (max 100 ms) sobre peekFirst; sólo hace pop del mensaje ofensor.

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:717-765
- testkit reference.conf:26 (expect-no-message-default = 100ms)

**Condiciones.**
- Error: `java.lang.AssertionError: assertion failed: received unexpected message <elem> after <ms> millis` (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:760-761). El elem es el RealMessage(msg, sender), no sólo el msg (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:754-760: `elem` es de la cola).

**Orden / concurrencia / ciclo de vida.** El mensaje que provoca el fallo se saca de la cola para que expectNoMessage posteriores no lo vean (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:757).

**Dependencias.** Filas: AC-TK-004. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `expect_no_message()` y `expect_no_message_for(dur)`

**Diferencias observables.** Rust: ídem; mensaje de error conservado si un spec lo afirma (ninguno lo hace).

**Aserciones.**
- A1: dos mensajes en cola, `eventually(expectNoMessage(100.millis))` falla las primeras veces (extrayendo uno cada vez) y luego pasa; después un "tak" nuevo se recibe — testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:223-233.
- A2: expectNoMessage tras within no hace fallar el within por tiempo — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:432,763, *no afirmada por spec*.
- A3: formato del mensaje de error — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:760-761, *no afirmada por spec*.

**Oráculo.** Jar oficial: `tk-expect.jsh`.

**Decisiones.** ninguna específica

## AC-TK-001.4 — TestKit receiveWhile, receiveN, receiveOne, fishForMessage, fishForSpecificMessage

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] receiveN(n) recibe n mensajes antes del deadline (AssertionError `timeout (<max>) while expecting <n> messages (got <k>)`); receiveWhile consume mientras el PF acepte y devuelve la lista (no es error alcanzar max); fishForMessage descarta mensajes hasta que el PF devuelve true y falla si el PF no está definido; fishForSpecificMessage ignora los no definidos. receiveOne(max): max==0 -> pollFirst; finito -> pollFirst(timeout); infinito -> takeFirst; actualiza lastMessage (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:844-863).

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:515-546
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:786-815
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:821-863

**Condiciones.**
- fishForMessage: timeout `timeout (<max>) during fishForMessage, hint: <hint>`; PF no definido: `fishForMessage(<hint>) found unexpected message <o>` (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:521-522).
- fishForSpecificMessage: timeout `timeout (<max>) during fishForSpecificMessage, hint: <hint>` (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:542).
- receiveWhile: si el mensaje no coincide se devuelve a la cabeza de la cola (offerFirst) y se restaura lastMessage (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:803-806); activa lastWasNoMsg (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:813).

**Orden / concurrencia / ciclo de vida.** receiveWhile con `idle` corta si pasa ese tiempo sin mensajes.

**Dependencias.** Filas: AC-TK-001.1. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `receive_n(n)`, `receive_while(f)`, `fish_for_message(f)`, `fish_for_specific_message(f)`

**Diferencias observables.** Rust: ídem.

**Aserciones.**
- A1: fishForMessage con PF `case "fishForMe" => true; case _ => false` consume "hallo", "welt", "fishForMe" y deja "done" — testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:162-175.
- A2: fishForSpecificMessage devuelve "fishForMe" y deja "done" — testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:177-191.
- A3: receiveN(n) con menos mensajes -> AssertionError `timeout .. while expecting n messages (got k)` — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:833, *no afirmada por spec*.
- A4: receiveWhile devuelve los resultados en orden y re-encola el primer no coincidente — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:792-808, *no afirmada por spec*.
- A5: fishForMessage con PF no definido para un mensaje falla — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:522, *no afirmada por spec*.
- A6: receiveOne(0) no bloquea — testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:845-847.

**Oráculo.** Jar oficial: `tk-receive.jsh`.

**Decisiones.** ninguna específica

## AC-TK-001.5 — TestKit within / remaining / remainingOr / remainingOrDefault / awaitCond / awaitAssert / assertForDuration

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] within(min,max) acota el tiempo de un bloque (max dilatado; min no); anidados toman el menor; remaining fuera de within lanza AssertionError; awaitCond vence con AssertionError `timeout <max> expired: <message>`; awaitAssert relanza la última excepción al vencer. El estado `end` (Duration.Undefined fuera de within) lo comparte todo el TestKit (campo var, no thread-safe).

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:252-280 (remaining*)
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:297-358 (awaitCond/awaitAssert)
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:371-396
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:414-442 (within)

**Condiciones.**
- remaining fuera de within: AssertionError `` `remaining` may not be called outside of `within` `` (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:261).
- remainingOr con end infinito: IllegalArgumentException("`end` cannot be infinite") (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:270,272); max infinito: "max duration cannot be infinite" (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:277,279).
- within: AssertionError `required min time <min> not possible, only <rem> left` (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:418); `block took <d>, should at least have been <min>` (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:431); `block took <d>, exceeding <max>` (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:433), este último no se comprueba si la última aserción fue expectNoMessage/receiveWhile (lastWasNoMsg).
- awaitCond: `assert(now < stop, s"timeout ${_max} expired: $message")` (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:308); polling cada `interval` (100 ms por defecto) (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:306-314).
- awaitAssert: la excepción se relanza si `(now + t) >= stop` (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:346); sólo captura NonFatal.

**Orden / concurrencia / ciclo de vida.** within restaura `end` en finally (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:426-428).

**Dependencias.** Filas: AC-TK-004. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `within(min, max, || ..)`, `remaining()`, `await_cond(..)`, `await_assert(..)`

**Diferencias observables.** Rust: ídem; el estado `end` puede ser un campo con `Cell`/`Mutex`. Q-TK-4 (API bloqueante/async).

**Aserciones.**
- A1: con timefactor 2.0, `1.second.dilated.toNanos == 1e9 * timefactor` — testkit/src/test/scala/org/apache/pekko/testkit/TestTimeSpec.scala:24-25.
- A2: `probe.awaitCond(false, 1.second)` lanza AssertionError y tarda >= 1 s * timefactor y < +1 s — testkit/src/test/scala/org/apache/pekko/testkit/TestTimeSpec.scala:27-33.
- A3: `awaitAssert("foo" === "foo")` pasa; `awaitAssert("foo" === "bar", 500.millis, 300.millis)` dentro de within(300ms,2s) lanza TestFailedException — testkit/src/test/scala/org/apache/pekko/testkit/TestTimeSpec.scala:36-43.
- A4: `remaining` fuera de `within` lanza AssertionError — testkit/src/test/scala/org/apache/pekko/testkit/TestTimeSpec.scala:45-49.
- A5: within fallido por exceder max — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:433, *no afirmada por spec*.
- A6: within anidado toma el menor tiempo restante — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:422-424, *no afirmada por spec*.
- A7: `within(3.seconds){ expectMsg(..); ..; expectNoMessage() }` — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:116-121.

**Oráculo.** Jar oficial: `tk-within.jsh` (mensajes de AssertionError).

**Decisiones.** Q-TK-4

## AC-TK-002.1 — TestProbe: ref, send, forward, sender, reply, nombres (testProbe-N / nombre de usuario)

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] TestProbe es un TestKit con su propio testActor; `send(actor,msg)` usa testActor como sender; `reply(msg)` envía al sender del último mensaje con el probe como emisor; `forward(actor, msg=último)` conserva el sender original; el nombre del actor empieza por el indicado (por defecto `testProbe`). TestProbe.apply()(implicit system) y apply(name).

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:1037-1075

**Condiciones.**
- reply sobre NullMessage lanza IllegalActorStateException (por lastMessage.sender, testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:71-73).

**Orden / concurrencia / ciclo de vida.** Síncrono (ver AC-TK-001.1).

**Dependencias.** Filas: AC-TK-001.1. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `TestProbe::new(&system)`, `TestProbe::named(&system, name)`

**Diferencias observables.** Rust: ídem.

**Aserciones.**
- A1: probe1.send(probe2.ref, "hello"); probe2.expectMsg(0.millis,"hello"); probe2.lastMessage.sender ! "world"; probe1.expectMsg(0.millis, "some hint here", "world") — testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:50-57.
- A2: TestProbe("worker").ref.path.name empieza por `worker` — testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:213-216.
- A3: `new TestProbe(system).ref.path.name` empieza por `testProbe` — testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:218-221.
- A4: forward y reply — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:1058-1068, *no afirmada por spec leída*.

**Oráculo.** Jar oficial: `tk-testactor.jsh`.

**Decisiones.** Q-TK-1

## AC-TK-002.2 — AutoPilot (TestActor.AutoPilot, NoAutoPilot, KeepRunning) y ejecución antes de encolar

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] El AutoPilot corre por cada mensaje recibido por el testActor (antes de decidir ignorar y encolar); su retorno es el siguiente pilot (KeepRunning conserva el actual). El pilot recibe `sender()` y el mensaje; puede reenviar a otro testActor.

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:39-51
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:135-142

**Condiciones.**
- autopilot corre siempre (incluso con ignoreMsg que luego descarta el mensaje) (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:136-141).

**Orden / concurrencia / ciclo de vida.** autopilot.run -> ignore-check -> offerLast.

**Dependencias.** Filas: AC-TK-001.1. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** trait `AutoPilot { fn run(&mut self, sender, msg) -> AutoPilotNext }`

**Diferencias observables.** Rust: trait objeto; la devolución `KeepRunning` es un enum.

**Aserciones.**
- A1: pilot que reenvía todo a testActor salvo "stop" (-> NoAutoPilot): testActor recibe hallo, welt; el probe también; tras "stop" nada pasa a testActor — testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:126-149.
- A2: el mensaje procesado por el pilot también se encola en el probe — testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:142-146.

**Oráculo.** Jar oficial: `tk-autopilot.jsh`.

**Decisiones.** ninguna específica

## AC-TK-002.3 — childActorOf (Spawn) y DelegatingSupervisorStrategy del testActor

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] childActorOf(props[, name][, strategy]) crea un hijo del testActor y lo devuelve; sin estrategia usa stoppingStrategy; con estrategia, el testActor delega a ella los fallos de ese hijo. DelegatingSupervisorStrategy olvida la delegación al terminar el hijo (handleChildTerminated).

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:78-111
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:878-916

**Condiciones.**
- Delegado ausente -> stoppingStrategy (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:83).

**Orden / concurrencia / ciclo de vida.** childActorOf usa `testActor ! Spawn` y `expectMsgType[ActorRef]`, por lo que consume un mensaje de la cola.

**Dependencias.** Filas: AC-TK-001.1 ; AC-SUP-003. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `probe.child_actor_of(props, name, strategy)`

**Diferencias observables.** Rust: ídem.

**Aserciones.**
- A1: `child.path.parent == probe.ref.path` y nombre dado respetado — testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:59-66.
- A2: con SupervisorStrategy.defaultStrategy un hijo que lanza RuntimeException se reinicia (postRestart llamado > 1 vez) — testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:68-91.
- A3: sin estrategia, el hijo que falla se detiene — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:83, *no afirmada por spec*.

**Oráculo.** Jar oficial: `tk-childactorof.jsh`.

**Decisiones.** Q-X-5

## AC-TK-003 — ImplicitSender y DefaultTimeout

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] ImplicitSender hace que `self` sea testActor (sender implícito); DefaultTimeout expone `timeout` = pekko.test.default-timeout. Mixins; nada más.

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:1077-1083

**Condiciones.**
- n/a

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** Filas: AC-TK-001.1 ; AC-TK-004. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `TestKit::tell_from_probe`/sender explícito; `default_timeout()`

**Diferencias observables.** Diferencia observable de API: `self` implícito no existe en Rust.

**Aserciones.**
- A1: `self === testActor` — testkit/src/test/scala/org/apache/pekko/testkit/ImplicitSenderSpec.scala:29-33.
- A2: `timeout === testKitSettings.DefaultTimeout` — testkit/src/test/scala/org/apache/pekko/testkit/DefaultTimeoutSpec.scala:29-33.

**Oráculo.** No aplica (es sintaxis Scala).

**Decisiones.** Q-X-3

## AC-TK-004 — Dilatación de tiempo y settings (pekko.test.timefactor, single-expect-default, expect-no-message-default, filter-leeway, default-timeout)

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] timefactor debe ser positivo y finito (si no, excepción al crear TestKitSettings); `dilated` multiplica duraciones por timefactor con redondeo (+0.5); TestKitExtension es una extensión por sistema. TestKitSettings: TestTimeFactor, SingleExpectDefaultTimeout (3s), ExpectNoMessageDefaultTimeout (100ms), TestEventFilterLeeway (3s), DefaultTimeout (5s).

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/TestKitExtension.scala:25-42
- testkit/src/main/scala/org/apache/pekko/testkit/package.scala:64-67
- testkit/src/main/resources/reference.conf:10-34

**Condiciones.**
- Error: `pekko.test.timefactor must be positive finite double` (TestKitExtension.scala:37) — texto del código.

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** Filas: AC-CFG-001 ; AC-EXT-001. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `TestKitSettings` leído de config + `Dilate` trait

**Diferencias observables.** Rust: ídem; TestKitSettings se obtiene de la config del sistema.

**Aserciones.**
- A1: `1.second.dilated.toNanos == 1e9 * timefactor.toLong` con timefactor=2.0 — testkit/src/test/scala/org/apache/pekko/testkit/TestTimeSpec.scala:24-25.
- A2: la dilatación afecta a awaitCond (dura >= 2 s para 1 s) — testkit/src/test/scala/org/apache/pekko/testkit/TestTimeSpec.scala:27-33.
- A3: timefactor <= 0 o infinito produce error al crear el TestKitSettings — derivada de TestKitExtension.scala:35-37, *no afirmada por spec*.
- A4: valores por defecto 1.0, 3s, 100ms, 3s, 5s — derivada de testkit/src/main/resources/reference.conf:14-29, *no afirmada por spec*.

**Oráculo.** Jar oficial: `tk-settings.jsh` (leer TestKitSettings con config por defecto).

**Decisiones.** Q-TK-5

## AC-TK-005.1 — EventFilter: fábricas error/warning/info/debug/custom/apply[E], DeadLettersFilter y reglas de coincidencia

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] Un filtro decide si un LogEvent coincide: tipo (Error con causa asignable al tipo dado; Warning/Info/Debug), fuente exacta (opcional), mensaje (igualdad completa si `message`, prefijo `start` o regex `pattern`); EventFilter.error() sin tipo coincide con cualquier Error (con o sin causa); EventFilter[T]() sólo con causa asignable; EventFilter[NoCause.type]() coincide con cualquier error. El mensaje se convierte a String (`null` -> "null"); la fuente es la cadena logSource.

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:89-158 (EventFilter base)
- testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:177-309 (fábricas)
- testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:325-376 (ErrorFilter)
- testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:386-511 (Warning/Info/DebugFilter)
- testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:518-543 (Custom, DeadLettersFilter)

**Condiciones.**
- ErrorFilter: coincide si (throwable es NoCause.getClass o throwable.isInstance(cause)) y [(msg==null && cause.getMessage==null && stackTrace vacío) || doMatch(src,msg) || doMatch(src, cause.getMessage)] (testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:332-337).
- doMatch: `(source vacío o igual) && (complete ? msg == s : msg.startsWith(s)) | regex.findFirstIn(msg).isDefined)` (testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:150-156).
- DeadLettersFilter coincide con Warning cuyo `msg` sea instancia (boxed) de la clase dada (testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:536-540).
- Con occurrences=Int.MaxValue no hay conteo (testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:87,102).

**Orden / concurrencia / ciclo de vida.** apply(event) decrementa `todo` al coincidir.

**Dependencias.** Filas: AC-EVT-012. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `EventFilter::error::<E>()`, `::warning()`, `::info()`, `::debug()`, `::custom(f)`

**Diferencias observables.** Rust: `EventFilter` con enum de nivel; el `Error(cause)` requiere `TypeId` del error (Q-TK-3).

**Aserciones.**
- A1: EventFilter.error() coincide con error sin causa — testkit/src/test/scala/org/apache/pekko/testkit/TestEventListenerSpec.scala:23-27.
- A2: EventFilter.error() coincide con error con causa — testkit/src/test/scala/org/apache/pekko/testkit/TestEventListenerSpec.scala:29-33.
- A3: EventFilter[AnError]() no coincide con error sin causa — testkit/src/test/scala/org/apache/pekko/testkit/TestEventListenerSpec.scala:36-39.
- A4: EventFilter[AnError]() no coincide con causa de otro tipo — testkit/src/test/scala/org/apache/pekko/testkit/TestEventListenerSpec.scala:41-45.
- A5: EventFilter[AnError]() coincide con causa del tipo — testkit/src/test/scala/org/apache/pekko/testkit/TestEventListenerSpec.scala:47-50.
- A6: EventFilter[Error.NoCause.type]() coincide con error sin causa y con cualquier causa — testkit/src/test/scala/org/apache/pekko/testkit/TestEventListenerSpec.scala:53-64.
- A7: EventFilter.warning() coincide con warning sin y con causa — testkit/src/test/scala/org/apache/pekko/testkit/TestEventListenerSpec.scala:66-75.
- A8: filtros por `start`, `pattern`, `message`, `source` — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:150-156, *no afirmada por spec*.
- A9: `EventFilter.warning(pattern = "received dead letter from Actor\[pekko://LogDeadLetters/system/testProbe.*")` — uso en ActorSystemSpec.scala:174-181.

**Oráculo.** Jar oficial pekko-testkit: `tk-eventfilter.jsh` con una tabla (filtro, evento) -> bool.

**Decisiones.** Q-TK-3

## AC-TK-005.2 — EventFilter.intercept, filterEvents, filterException, awaitDone/assertDone y occurrences

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] intercept publica Mute(this), ejecuta el bloque, espera hasta filter-leeway (dilatado) a que se alcancen exactamente `occurrences` y publica UnMute en finally; si faltan: AssertionError `timeout (<leeway>) waiting for <n> messages on <filter>`; si sobran: `received <k> excess messages on <filter>`. filterEvents(filters)(block): Mute(filters); bloque; espera con leeway todos; UnMute en finally; falla con `Filter completion error:\n Timeout (<leeway>) waiting for <f>`.

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:107-138
- testkit/src/main/scala/org/apache/pekko/testkit/package.scala:25-51
- testkit reference.conf:18 (filter-leeway = 3s)

**Condiciones.**
- filterEvents falla con AssertionError("Filter completion error:\n" + lista de `Timeout (<leeway>) waiting for <filtro>`) (package.scala:41-42).
- intercept y filterEvents no comprueban excesos hasta el final; `awaitDone` devuelve true si todo==MaxValue o todo==0 (testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:107-110).

**Orden / concurrencia / ciclo de vida.** Mute/UnMute son mensajes al EventStream procesados por el TestEventListener (asíncrono respecto del bloque; por eso la espera con leeway).

**Dependencias.** Filas: AC-TK-005.3 ; AC-TK-004. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `filter.intercept(&system, || ..)`, `filter_events(&system, [..], || ..)`

**Diferencias observables.** Rust: ídem; la espera usa poll con dilatación.

**Aserciones.**
- A1: `EventFilter.warning(pattern="received dead letter from Actor\[pekko://LogDeadLetters/system/testProbe.*", occurrences=1).intercept{ a.tell("boom", probe.ref) }(sys)` pasa con exactamente 1 ocurrencia — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:167-189.
- A2: intercept anidado (info `.*not delivered` y warning `received dead letter: boom`) — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:191-213.
- A3: `filterEvents(EventFilter.warning(s"unhandled message from Actor[$deadLettersPath]: $message", occurrences = 1)) { rootGuardian ! message }` — actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:76-78.
- A4: falta/exceso de ocurrencias produce AssertionError con los textos fijos — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:116-120,131-135, *no afirmada por spec*.
- A5: `filterException[T](block)` = `EventFilter[T]().intercept(block)` — package.scala:53-54.

**Oráculo.** Jar oficial: `tk-intercept.jsh` (fallos por falta y por exceso).

**Decisiones.** Q-TK-4, Q-X-7

## AC-TK-005.3 — TestEventListener: Mute/UnMute, conversión de DeadLetter/UnhandledMessage/Dropped en Warning, impresión de lo no filtrado

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] Logger (DefaultLogger) que suscribe Mute, UnMute, DeadLetter, UnhandledMessage y Dropped; los LogEvent filtrados no se imprimen; DeadLetter -> Warning `received dead letter[ from <sender>]: <msg>` (o `received dead system message`), UnhandledMessage -> Warning `unhandled message from <sender>: <msg>`, Dropped -> Warning `dropped message from <sender>. <reason>: <msg>`; Terminate (DeadLetter de Terminate) se ignora. Las filtros se guardan en lista (cabeza = último); removeFilter quita el primero igual (por igualdad de case class, que ignora `occurrences` al estar en la segunda lista de parámetros).

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:557-611

**Condiciones.**
- filter(event): `filters.exists(f => try f(event) catch { case _: Exception => false })` (testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:593-597).
- Mensaje DeadLetter: `if snd eq deadLetters then "received dead letter" else "received dead letter from " + snd`, luego `: <msg>` (testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:574-578). Primero se evalúa un filtro contra Warning sin prefijo (testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:572-573) — permite DeadLettersFilter.

**Orden / concurrencia / ciclo de vida.** Mute/UnMute se aplican en el orden de llegada al logger (actor).

**Dependencias.** Filas: AC-EVT-008 ; AC-EVT-011 ; AC-EVT-017. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** logger de test que captura `LogEvent`

**Diferencias observables.** Rust: ídem; el logger de test puede implementarse sin actor si Q-X-7=backend.

**Aserciones.**
- A1: un DeadLetter sin filtro imprime el Warning `received dead letter ..` (observado indirectamente por filtros) — ActorSystemSpec.scala:177-181.
- A2: `received dead letter: boom` cuando el sender es noSender (deadLetters) — ActorSystemSpec.scala:203.
- A3: UnhandledMessage -> `unhandled message from Actor[<deadLetters>]: <msg>` — LocalActorRefProviderSpec.scala:77.
- A4: DeadLettersFilter coincide con el Warning interno (prefijo no incluido) — testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:536-540,572-573, *no afirmada por spec leída*.
- A5: Dropped -> Warning `dropped message from <sender>. <reason>: <msg>` — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:585-588.
- A6: Terminate se ignora — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:571.

**Oráculo.** Jar oficial pekko-testkit: `tk-testeventlistener.jsh`.

**Decisiones.** Q-X-7

## AC-TK-006.1 — TestActorRef (underlyingActor, receive síncrono)

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] `TestActorRef[T <: Actor]` es un `LocalActorRef` que (1) se crea y arranca de forma síncrona (el actor existe y su constructor y `preStart` ya corrieron al volver de `TestActorRef(..)`), (2) usa por defecto el `CallingThreadDispatcher` (AC-TK-006.2) salvo que los `Props` indiquen otro dispatcher, de modo que cada `!`/`?` ejecuta el `receive` del actor en el hilo del emisor antes de volver, (3) deja el `receiveTimeout` sin definir (`Duration.Undefined`), (4) expone el actor subyacente (`underlyingActor`), (5) permite inyectar mensajes directamente (`receive(o[, sender])`) sin pasar por el buzón, propagando al llamador las excepciones del `receive`, (6) ofrece `watch`/`unwatch` en nombre del actor, (7) se puede crear bajo cualquier supervisor (guardián por defecto, otro `TestActorRef`, una probe, o `TestProbe.childActorOf`) y con nombre aleatorio `$<base64>` si no se da.

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/TestActorRef.scala:27-155 (clase), 158-263 (companion: `apply`/`create`, `randomName`, `InternalGetActor`)
- testkit/src/main/scala/org/apache/pekko/testkit/CallingThreadDispatcher.scala:136-138 (`Id`)
- testkit/src/main/resources/reference.conf:31-33
- specs: testkit/src/test/scala/org/apache/pekko/testkit/TestActorRefSpec.scala:30-118 (fixtures), 120-359 (casos); usos en otros specs: actor-tests/src/test/scala/org/apache/pekko/event/LoggingReceiveSpec.scala:108-128, actor-tests/src/test/scala/org/apache/pekko/dispatch/ExecutionContextSpec.scala:216-233, actor-tests/src/test/scala/org/apache/pekko/io/TcpListenerSpec.scala:157-177, actor-tests/src/test/scala/org/apache/pekko/io/TcpConnectionSpec.scala:105,114,966-1004, actor-tests/src/test/scala/org/apache/pekko/io/InetAddressDnsResolverSpec.scala:103-108,152-157 (1.7.1), actor-tests/src/test/scala/org/apache/pekko/actor/FSMActorSpec.scala:172-183,272-289,316

**Precondiciones.** Un `ActorSystem` implícito (en realidad un `ActorSystemImpl`, por el cast). El supervisor debe ser una ref local (`LocalActorRef`) o una `RepointableActorRef` ya iniciada; con un top-level aún no iniciado lanza `IllegalStateException("cannot attach a TestActor to an unstarted top-level actor, ensure that it is started by sending a message and observing the reply")` (TestActorRef.scala:38-43). Con supervisor de otro tipo sólo se loguea un error (50-54).

**Comportamiento.**
- Construcción: reserva el nombre en el supervisor (`reserveChild(name)`), arma dispatcher/mailbox con `props.withDispatcher(CallingThreadDispatcher.Id)` cuando `props.deploy.dispatcher == NoDispatcherGiven`, y llama a `underlying.start()` (TestActorRef.scala:57-82). Como el dispatcher es síncrono, `Create` se procesa dentro de `start()`.
- `receive(o, sender)`: fija `underlying.currentMessage = Envelope(o, sender|deadLetters)`, llama `underlying.receiveMessage(o)` y limpia `currentMessage` en `finally` (TestActorRef.scala:113-118): ejecuta el `Receive` del actor con soporte de `become`/`unbecome`, sin mailbox; cualquier excepción llega al llamador.
- `underlyingActor`: si `isTerminated` lanza `IllegalActorStateException("underlying actor is terminated")`; si `underlying.actor` es `null` (creación asíncrona) envía `InternalGetActor` y espera (`Await`) hasta `DefaultTimeout` (TestActorRef.scala:125-134). Tras un reinicio devuelve la instancia nueva (el Scaladoc advierte que la antigua se descarta).
- `newActorCell` intercepta `InternalGetActor` (autorecibido) y responde `actor` (TestActorRef.scala:86-99).
- `toString = "TestActor[" + path + "]"` (154).
- Fábricas: `apply[T](factory)`, `(factory, name)`, `(props)`, `(props, name)`, `(props, supervisor)`, `(props, supervisor, name)`, `apply[T: ClassTag]`, `(name)`, `(supervisor)`, `(supervisor, name)` y las `create` Java (TestActorRef.scala:168-262). Las variantes por `ClassTag` crean la instancia por reflexión y, si falla, `ActorInitializationException("Could not instantiate Actor …")` (192-201).

**Errores.** `IllegalStateException` (supervisor top-level sin iniciar); `IllegalActorStateException` (`underlyingActor` sobre actor terminado); `ActorInitializationException` (instanciación reflexiva fallida); `TimeoutException`/`AskTimeout` si la obtención asíncrona del actor tarda más que `DefaultTimeout`; las excepciones del `receive` inyectado se propagan tal cual.

**Orden / concurrencia / ciclo de vida.** Con el dispatcher por defecto todo es síncrono y determinista dentro de un hilo (ver AC-TK-006.2): `ref ! m` retorna después de que `receive(m)` y de las reentradas encoladas hayan terminado; `ref ? m` devuelve un futuro ya completo si el actor respondió dentro del `receive` (TestActorRefSpec.scala:224-230). Con un dispatcher explícito (p. ej. `disp1` de tipo `Dispatcher`) el comportamiento pasa a ser el normal asíncrono (TestActorRefSpec.scala:264-267).

**Dependencias.** Filas: AC-TK-006.2 ; AC-TK-001.1 ; AC-REF-002 ; AC-REF-009.2 ; AC-CELL-001 ; AC-CELL-010 ; AC-SUP-014. Externas: ninguna.

**Equivalente Rust propuesto.** **Choque con la propiedad de Rust.** En la JVM `underlyingActor` devuelve una referencia compartida y mutable al objeto del actor, que el mismo test lee mientras el actor lo muta en sus `receive` (no hay sincronización). En Rust el estado del actor (`A`) lo posee la celda y se accede con `&mut A` sólo dentro de la invocación del `receive`. La representación propuesta conserva el comportamiento observable así:
1. `TestActorRef<A>` guarda el estado en `Arc<ActorSlot<A>>` donde `ActorSlot<A> { lock: ReentrantLock, state: UnsafeCell/Mutex<Option<A>> }`; el mismo candado reentrante que usa el `CallingThreadDispatcher` (AC-TK-006.2) protege la ejecución de cada invocación. `A: Send + 'static` es suficiente (el acceso está serializado por el candado, no hace falta `Sync`).
2. Acceso al subyacente: `with_actor(|a: &mut A| …) -> R` toma el candado, ejecuta el cierre y lo libera (préstamo con alcance: compila sólo si el cierre no retiene la referencia); `underlying_actor() -> ActorGuard<'_, A>` devuelve una guarda (`Deref/DerefMut`, `!Send`) para el patrón `ref.underlyingActor.field` de los specs (TcpListenerSpec.scala:177, TcpConnectionSpec.scala:104). Ambas devuelven `Err(TestActorRefError::Terminated)` si el actor terminó (equivalente a `IllegalActorStateException`) y reevalúan la instancia en cada llamada (tras un reinicio devuelven la nueva).
3. Diferencia deliberada y beneficiosa: si otro hilo está ejecutando el actor, `with_actor` espera a que termine esa invocación (Pekko lo lee sin sincronizar, con carrera). En un único hilo (el caso de todos los specs) el resultado es idéntico. Una guarda mantenida mientras se envía un mensaje al mismo actor desde el mismo hilo NO provoca interbloqueo (el candado es reentrante) y el mensaje se ejecuta de inmediato, igual que en Pekko (donde el hilo de test no es el que sostiene el candado de la CTD): por eso la guarda debe soltarse antes de enviar si se quiere que el `receive` vea el estado ya modificado.
4. `receive(msg, sender)`: toma el candado, fija el mensaje/emisor actuales del contexto y llama al manejador del actor sin pasar por el buzón (conserva `become`/`unbecome`); devuelve `Result<(), ActorFailure>` con el fallo del manejador (o relanza el pánico original con `resume_unwind`, según Q-X-5) en lugar de lanzar una excepción.
5. La creación es síncrona: `TestActorRef::new(&system, props)` devuelve cuando `pre_start` ya corrió; los nombres aleatorios `$<base64>` reutilizan `Helpers.base64` (AC-UTIL-002.1); `with_supervisor`, `named` y `with_dispatcher` sustituyen las sobrecargas por un builder.
6. Las variantes por `ClassTag` se sustituyen por `TestActorRef::of::<A: Default>()`/fábrica explícita (sin reflexión).
Viabilidad: alta para todo lo observable por los specs listados; límite: el actor debe poder ejecutarse en el hilo del emisor (requisito sync del manejador, ver AC-TK-006.2 y Q-TK-8).

**Diferencias observables.**
- `underlyingActor` devuelve guarda/cierre en vez de la referencia compartida (Q-TK-2); es más estricto que Pekko cuando hay concurrencia real.
- Excepciones → `Result` (Q-X-5); `IllegalActorStateException` → `TestActorRefError::Terminated`.
- No hay reflexión por `ClassTag` (Q-X-2).
- `receiveTimeout` sin definir: el contexto expone `receive_timeout() -> Option<Duration> == None` (equivalente a `Duration.Undefined`).

**Aserciones.**
- A1: creación anidada: un `TestActorRef` creado dentro del `Actor` de otro `TestActorRef`, y un hijo creado con `context.actorOf`, devuelven refs no nulas y distintas al ser consultadas con `?` — testkit/src/test/scala/org/apache/pekko/testkit/TestActorRefSpec.scala:130-154.
- A2: respuesta vía `sender()`: tras `clientRef ! "complex"` y tres `"simple"` el contador baja de 4 a 0 de forma síncrona (antes de que el `!` vuelva), repetido con `"complex2"`, y todo corre en el hilo del test (`assertThread`) — testkit/src/test/scala/org/apache/pekko/testkit/TestActorRefSpec.scala:156-179.
- A3: `PoisonPill` detiene el `TestActorRef`, un vigilante recibe `Terminated(a)` y `a.isTerminated == true`, sin salir del hilo del test — testkit/src/test/scala/org/apache/pekko/testkit/TestActorRefSpec.scala:181-198.
- A4: `Kill` reinicia al hijo bajo una estrategia `OneForOne(5, 1s)(ActorKilledException)`: `preRestart` y `postRestart` corren (contador 2 → 0) de forma síncrona — testkit/src/test/scala/org/apache/pekko/testkit/TestActorRefSpec.scala:200-222.
- A5: `a ? "work"` devuelve un futuro ya completo (`f.isCompleted == true`) con `"workDone"` — testkit/src/test/scala/org/apache/pekko/testkit/TestActorRefSpec.scala:224-230.
- A6: el `receiveTimeout` de un actor con `TestActorRef` funciona (el actor recibe `ReceiveTimeout` tras 1 s y envía `"timeout"`) — testkit/src/test/scala/org/apache/pekko/testkit/TestActorRefSpec.scala:232-235.
- A7: `underlyingActor` da acceso al estado interno: tras `ref ! "hallo"`, `actor.s == "hallo"` — testkit/src/test/scala/org/apache/pekko/testkit/TestActorRefSpec.scala:241-252.
- A8: `underlyingActor.context.receiveTimeout` es exactamente `Duration.Undefined` — testkit/src/test/scala/org/apache/pekko/testkit/TestActorRefSpec.scala:254-257.
- A9: el dispatcher por defecto es `CallingThreadDispatcher` — testkit/src/test/scala/org/apache/pekko/testkit/TestActorRefSpec.scala:259-262.
- A10: un dispatcher explícito (`withDispatcher("disp1")`) se respeta (clase `Dispatcher`) — testkit/src/test/scala/org/apache/pekko/testkit/TestActorRefSpec.scala:264-267.
- A11: `ref.receive("work")` ejecuta el manejador y el actor se detiene (`isTerminated`) — testkit/src/test/scala/org/apache/pekko/testkit/TestActorRefSpec.scala:269-273; con `sender` explícito (`testActor`) además llega `"workDone"` a ese emisor — :275-280.
- A12: un `TestActorRef` creado con un supervisor `TestProbe.ref` o con `parent.childActorOf(..)` no lanza al crearse y registra la excepción del hijo (`EventFilter[RuntimeException](message = "expected")`) — testkit/src/test/scala/org/apache/pekko/testkit/TestActorRefSpec.scala:282-307.
- A13: el companion crea con supervisor por defecto (el guardián `/user`), con nombre dado, con supervisor dado y con ambos, y con `Props` en las mismas combinaciones; los atributos `supervisor` y `name` del actor reflejan lo pedido — testkit/src/test/scala/org/apache/pekko/testkit/TestActorRefSpec.scala:311-358.
- A14: con un `TestActorRef` anidado y `p.send(b, "hello")` hacia un actor que reenvía a otro con `blocking`, `p.expectMsg(0.seconds, "hello")` pasa (entrega totalmente síncrona) — actor-tests/src/test/scala/org/apache/pekko/dispatch/ExecutionContextSpec.scala:216-233.
- A15: `underlyingActor` se usa para leer campos internos de actores de I/O (`channel`, `listener`) — actor-tests/src/test/scala/org/apache/pekko/io/TcpListenerSpec.scala:177 y TcpConnectionSpec.scala:104,113,957 (aserciones propias de AC-IO; aquí fijan el patrón de acceso de sólo lectura).
- A16: un supervisor top-level aún no iniciado provoca `IllegalStateException` con el mensaje citado — derivada de TestActorRef.scala:38-43, *no afirmada por spec*. Caracterización propuesta.
- A17: `underlyingActor` sobre un actor terminado lanza `IllegalActorStateException("underlying actor is terminated")` — derivada de TestActorRef.scala:127, *no afirmada por spec*. Caracterización propuesta: `Err(Terminated)`.
- A18: los nombres generados son `$` + base64 del contador y `toString` es `TestActor[<path>]` — derivada de TestActorRef.scala:154,163-166, *no afirmada por spec*.

**Validación prevista.** A1-A13: tests portados 1:1 (TestActorRefSpec completo) sobre la CTD de Rust; el test A2/A4 exige además la verificación de que `Thread::current().id()` no cambia (`assertThread`). A14 se porta con ExecutionContextSpec. A15 se valida en la unidad IO. A16-A18: caracterización propia. Sin oráculo diferencial: el contrato es de comportamiento síncrono local y los specs lo fijan; sólo A18 podría compararse (formato de `toString`), por lo que se añade a `tools/oracle/tk-testactorref-tostring.jsh`, justificado por ser texto visible en logs.

**Decisiones.** Q-TK-2 (representación de `underlyingActor`: guarda/cierre sobre el candado reentrante; propuesta basada en la lectura completa de la clase y su spec), Q-TK-1 (entrega síncrona), Q-TK-8 (manejador ejecutable en el hilo del emisor), Q-X-5.

## AC-TK-006.2 — CallingThreadDispatcher (ejecución síncrona en el hilo del emisor; configurador en reference.conf del testkit)

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] Dispatcher de prueba (`pekko.test.calling-thread-dispatcher`, `type = …CallingThreadDispatcherConfigurator`, singleton por configurador) que no crea hilos: ejecuta cada invocación de un actor en el hilo que envía el mensaje. Estrategia: se ejecuta en el hilo actual salvo que el actor esté suspendido (`suspendSwitch`) o ya esté ejecutándose en el hilo actual (reentrada), en cuyo caso el mensaje se encola en una cola local al hilo para ser ejecutado cuando la invocación activa más arriba en la pila termine; si el actor se está ejecutando en otro hilo, el hilo actual espera a que acabe (candado con `tryLock(50 ms)` en bucle). Con un único hilo, el orden de ejecución es completamente determinista. Los mensajes de sistema (`Create`, `Terminate`, `Supervise`, `Watch`, `Failed`…) no pasan por colas por-hilo: se encolan en la cola de sistema del actor y se procesan de inmediato en el mismo hilo. `suspend`/`resume` son globales por actor: al reanudar, los mensajes encolados por TODOS los hilos se recogen en la cola del hilo que reanuda y se ejecutan allí.

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/CallingThreadDispatcher.scala:51-66 (reglas de bloqueo), 68-134 (`CallingThreadDispatcherQueues`), 136-138 (`Id`), 140-339 (`CallingThreadDispatcher`), 341-347 (configurador), 349-395 (`CallingThreadMailbox`)
- testkit/src/main/resources/reference.conf:31-33 (`pekko.test.calling-thread-dispatcher`)
- actor/src/main/resources/reference.conf:142-145 (`unstarted-push-timeout` "relevante con el CallingThreadDispatcher para un actor top-level")
- actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala:366 (comentario: "needed for CallingThreadDispatcher, which never calls Mailbox.run()")
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:188-198 (el `testActor` usa la CTD)
- specs: actor-tests/src/test/scala/org/apache/pekko/testkit/CallingThreadDispatcherModelSpec.scala:26-71; actor-tests/src/test/scala/org/apache/pekko/actor/dispatch/ActorModelSpec.scala:279-509; testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:30-57; actor-tests/src/test/scala/org/apache/pekko/dispatch/ExecutionContextSpec.scala:216-256; testkit/src/test/scala/org/apache/pekko/testkit/TestActorRefSpec.scala:156-230

**Precondiciones.** Seleccionable por id `pekko.test.calling-thread-dispatcher` en `Props`, o implícito en `TestActorRef`/`testActor`. El `LocalActorRef` debe usar `CallingThreadMailbox` (si no: `ActorInitializationException("expected CallingThreadMailbox, got …")`, CallingThreadDispatcher.scala:189).

**Comportamiento.**
- Parámetros: `throughput = 0`, `throughputDeadlineTime = 0`, `registerForExecution = false` (nunca usa un executor), `shutdownTimeout = 1 s`, `shutdown()` vacío, `id = "pekko.test.calling-thread-dispatcher"` (CallingThreadDispatcher.scala:167-181). `executeTask(invocation)` ejecuta `invocation.run()` en el hilo actual (248): por eso es válido como `ExecutionContext` síncrono (ExecutionContextSpec.scala:187-208).
- `register(actor)`: tras `super.register`, ejecuta `runQueue` de inmediato → `Create` se procesa durante `actorOf` (183-191).
- `dispatch(receiver, handle)`: encola el sobre en la cola del hilo actual dentro de `suspendSwitch.fold(<suspendido: encola, no ejecuta>)(<no suspendido: encola y ejecuta>)` y, si no está suspendido, llama `runQueue` (232-246).
- `systemDispatch`: `systemEnqueue` + `runQueue` (223-230).
- `runQueue(mbox, queue)`: si el candado reentrante `ctdLock` ya es del hilo actual no hace nada (la invocación exterior drenará la cola); si no, `tryLock(50 ms)`; con el candado ejecuta `process`: bucle `processAllSystemMessages()` → `dequeue()` (si no suspendido ni cerrado) → `actor.invoke(handle)` hasta que no queden sobres; las `NonFatal` se loguean (`"Error during message processing"`) y se sigue; sin candado y con mensajes pendientes, reintenta (250-338).
- `suspend`: `suspendSwitch.switchOn; mbox.suspend()`. `resume`: bajo `suspendSwitch.switchOff { gatherFromAllOtherQueues(mbox, queue); mbox.resume() }` y, si cambió el estado, `runQueue` en el hilo actual (202-221).
- `unregister`: `super.unregister` y `unregisterQueues(mbox)` (193-200). `cleanUp` del buzón: recoge todas las colas de hilos y `qq.cleanUp(self, deadLetterMailbox.messageQueue)` (los mensajes sin leer pasan a dead letters) y `q.remove()` (381-394).
- Colas: una `MessageQueue` por (buzón, hilo) en un `ThreadLocal`, registradas con `WeakReference` en `CallingThreadDispatcherQueues` (GC cada 1 s) (68-134, 356-368). `gatherFromAllOtherQueues` vacía las colas ajenas dentro de la cola propia (117-133).
- Interrupciones: `InterruptedException` durante el procesamiento se captura, se loguea y se relanza al final de `runQueue` (`throwInterruptionIfExistsOrSet`) (258-338).

**Errores.** Interrupción de hilo ⇒ `InterruptedException` al llamador tras procesar el resto; `NonFatal` del `invoke` ⇒ log y continuar; si un mensaje se encola cuando el candado lo tiene otro hilo y se pierde la carrera tras el `cleanUp`, no hay garantía de entrega a dead letters (comentario del código en 382-386).

**Orden / concurrencia / ciclo de vida.** Reglas de bloqueo (51-66): los mensajes normales siempre se encolan localmente al hilo; procesar una cola comprueba `suspendSwitch` antes de cada invocación; reanudar mueve atómicamente los mensajes de todos los hilos a la cola del hilo que reanuda; todo acceso a la cola ocurre bajo el candado del switch; los mensajes de sistema van directo a la cola de sistema (no por hilo). Orden: FIFO por (hilo, buzón); reentradas procesadas en FIFO tras la invocación activa. Un actor que se envía un mensaje a sí mismo desde su `receive` NO se ejecuta recursivamente: se encola y corre al terminar la invocación actual (observable en TestActorRefSpec.scala:156-179).

**Dependencias.** Filas: AC-DSP-001 ; AC-DSP-002 ; AC-MBX-013 ; AC-TK-006.1 ; AC-TK-001.1 ; AC-CELL-004. Externas: hilos y candados de la JVM (`ReentrantLock`, `ThreadLocal`, `Thread.interrupt`).

**Equivalente Rust propuesto.** Módulo `rakka_testkit::calling_thread` con `CallingThreadDispatcher` (implementa el trait `MessageDispatcher` de AC-DSP) y `CallingThreadMailbox`. **Choque con la propiedad de Rust y representación:**
1. *Exclusión mutua reentrante.* `ctdLock` → `parking_lot::ReentrantMutex<()>` (o `std::sync::Mutex` + `owner: AtomicU64` con el `ThreadId` y un contador de reentrada). Es necesario que "el hilo actual ya posee el candado" sea consultable sin bloquear: `lock.is_owned_by_current_thread()`. El estado del actor (`&mut A`) vive en una celda que sólo se toca con el candado tomado, por lo que la reentrada nunca crea dos `&mut A` simultáneos: la llamada reentrante sólo encola y retorna, y es la invocación exterior la que, al volver su `receive`, extrae el siguiente sobre.
2. *Colas por hilo.* `ThreadLocal[MessageQueue]` con `WeakReference` → `Mutex<HashMap<ThreadId, VecDeque<Envelope>>>` por buzón (misma semántica FIFO por hilo; la recogida en `resume`/`cleanUp` es un `drain` de todas las entradas hacia la cola propia; sin `WeakReference`, las entradas de hilos muertos se purgan en `resume`/`cleanUp` y cada 1 s como el original, o al desregistrar).
3. *Interruptor de suspensión.* `Switch` → `Mutex<bool>` con operaciones `fold(on, off)` y `switch_on/off(|| …)` ejecutadas bajo el mismo `Mutex` que protege la cola (replica "todo acceso a la cola ocurre bajo el candado del switch").
4. *Mensajes de sistema.* Cola de sistema por actor (no por hilo) procesada con `process_all_system_messages` antes de cada sobre normal.
5. *Ejecución del manejador en el hilo del emisor.* La invocación (`actor.invoke(handle)`) se hace en el hilo que llama a `tell`. Si el manejador del actor es `async fn`, se conduce con un `block_on` mínimo sin reactor (p. ej. `futures::executor::block_on` o `pollster`) en ese hilo; los manejadores que requieran el reactor de Tokio dentro de un contexto de CTD deben fallar con un error explícito en lugar de bloquear el runtime (Q-TK-8). Con manejadores `fn` síncronos (los de los specs de testkit) no hay restricción.
6. *Interrupciones.* `Thread.interrupt` no tiene equivalente en Rust; los dos tests de `ActorModelSpec` que lo ejercen no son portables y se excluyen con justificación (Q-TK-7). El resto de la semántica (registro y continuación tras `NonFatal` ⇒ captura de pánico con `catch_unwind` y log `Error during message processing`) sí.
7. *Dead letters al limpiar.* `cleanUp` mueve los sobres pendientes de todas las colas al buzón de dead letters del sistema.
Viabilidad: alta para el modelo de un hilo (todos los specs de testkit) y media-alta para concurrencia multi-hilo; límite: sin interrupciones.

**Diferencias observables.**
- Sin `Thread.interrupt` (Q-TK-7). Sin `WeakReference`: la recolección de colas de hilos terminados es explícita.
- Manejadores `async` requieren `block_on` sin reactor (Q-TK-8).
- `PanicPolicy`: un pánico en el manejador se trata como `NonFatal`: se loguea y se continúa (consistente con Q-X-5).

**Aserciones.**
- A1: `ActorModelSpec` aplicado a la CTD ("A Calling Thread Dispatcher") — actor-tests/src/test/scala/org/apache/pekko/testkit/CallingThreadDispatcherModelSpec.scala:58-71 (hereda los casos de ActorModelSpec con un dispatcher interceptado; ver A2-A9).
- A2: "dynamically handle its own life cycle": registra/desregistra y el dispatcher cuenta `stops` (0→1→2→3) al detener actores y completar futuros — actor-tests/src/test/scala/org/apache/pekko/actor/dispatch/ActorModelSpec.scala:284-310.
- A3: "process messages one at a time": el segundo mensaje (Wait 1000) no se solapa con el siguiente; `msgsReceived == msgsProcessed == 3` — actor-tests/src/test/scala/org/apache/pekko/actor/dispatch/ActorModelSpec.scala:312-330.
- A4: "handle queueing from multiple threads": 10 hilos × 20 mensajes ⇒ 200 recibidos y procesados — actor-tests/src/test/scala/org/apache/pekko/actor/dispatch/ActorModelSpec.scala:332-348.
- A5: "not process messages for a suspended actor": suspendido no procesa (`assertNoCountDown` 1000 ms); al reanudar, procesa el mensaje encolado; contadores `suspensions = resumes = 1` — actor-tests/src/test/scala/org/apache/pekko/actor/dispatch/ActorModelSpec.scala:360-382.
- A6: "handle waves of actors": 3 olas de 10 000 actores con un `Terminated` por actor y `stops = run` — actor-tests/src/test/scala/org/apache/pekko/actor/dispatch/ActorModelSpec.scala:384-421 (el comentario de las líneas 399-401 documenta que con la CTD el hilo "keep-alive" debe estar en otro hilo para no interbloquearse).
- A7: "continue to process messages when exception is thrown": tras `IndexOutOfBoundsException` y `RemoteException` en el manejador siguen procesándose los mensajes posteriores (`Reply("foo2")`, `Reply("bar2")`), mientras los futuros de los mensajes fallidos quedan sin completar (`f3.value.isEmpty`) — actor-tests/src/test/scala/org/apache/pekko/actor/dispatch/ActorModelSpec.scala:480-498.
- A8: "not double-deregister": `DoubleStop` deja `registers == 1` y `unregisters == 1` — actor-tests/src/test/scala/org/apache/pekko/actor/dispatch/ActorModelSpec.scala:500-507.
- A9: dos casos con interrupciones (`ActorInterruptedException` con causa `"Ping!"` y la variante sin excepción) — actor-tests/src/test/scala/org/apache/pekko/actor/dispatch/ActorModelSpec.scala:423-478: **no portables** (Q-TK-7).
- A10: entrega síncrona del `testActor`: `tk.ref ? "hello"; tk.expectMsg(0.millis, "hello")`, `lastMessage.sender ! "world"` ⇒ `future.isCompleted == true` — testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:32-39; análogos en :41-57.
- A11: como `ExecutionContext` síncrono, `ec.execute(batch)` ejecuta la tarea y las encoladas dentro de ella antes de volver (`x == 1`) — actor-tests/src/test/scala/org/apache/pekko/dispatch/ExecutionContextSpec.scala:235-256.
- A12: orden determinista y reentrada: mensajes enviados desde dentro de un `receive` a otro actor (que a su vez responde al primero) no se anidan en la pila y el contador final es exacto; todo en el hilo del test — testkit/src/test/scala/org/apache/pekko/testkit/TestActorRefSpec.scala:156-179.
- A13: los mensajes sin leer de un buzón CTD terminado pasan a dead letters — derivada de CallingThreadDispatcher.scala:381-394, observable en testkit/src/test/scala/org/apache/pekko/testkit/PekkoSpecSpec.scala:75-113 ("enqueue unread messages from testActor to deadLetters": `locker should contain(DeadLetter(42, davyJones, probe.ref))`).
- A14: los mensajes de sistema se procesan inmediatamente en el hilo del emisor, aunque haya mensajes normales encolados — derivada de CallingThreadDispatcher.scala:223-230 y 282, *no afirmada por spec*. Caracterización propuesta: `watch` sobre un actor ocupado entrega `Terminated` sin esperar.
- A15: en reanudación, los mensajes encolados por otros hilos se ejecutan en el hilo que reanuda — derivada de CallingThreadDispatcher.scala:209-221 y 117-133, *no afirmada por spec*. Caracterización propuesta con dos hilos.
- A16: si el actor se ejecuta en otro hilo, el emisor espera al candado (reintentos de 50 ms) y no pierde el mensaje — derivada de CallingThreadDispatcher.scala:310-337, *no afirmada por spec*. Caracterización propuesta con un `receive` bloqueado por un latch.
- A17: `register` procesa `Create` dentro de `actorOf` (el `preStart` ya corrió al volver) — derivada de CallingThreadDispatcher.scala:183-191, observable en TestActorRefSpec.scala:132-141.

**Validación prevista.** A1-A8 y A10-A12: portados 1:1 (ActorModelSpec parametrizado por el dispatcher bajo prueba; los contadores del interceptor se reproducen con un `DispatcherInterceptor` de prueba). A9: excluido (Q-TK-7). A13: portado con PekkoSpecSpec. A14-A17: caracterización propia (tests con dos hilos y latches). No se usa oráculo diferencial porque el comportamiento es de ordenación en hilos y los specs ya lo fijan; si el Owner exige confirmación, un escenario jshell (`tools/oracle/tk-ctd-order.jsh`) puede registrar el orden de ejecución de una secuencia reentrante.

**Decisiones.** Q-TK-1 (entrega síncrona del testActor: la lectura completa confirma que depende de este dispatcher), Q-TK-7 (interrupciones), Q-TK-8 (manejadores async), Q-X-4, Q-X-5.

## AC-TK-007 — TestFSMRef

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] `TestFSMRef[S, D, T <: Actor](system, props, supervisor, name)(implicit ev: T <:< FSM[S, D])` es un `TestActorRef[T]` con acceso a estado y timers del FSM subyacente: `stateName`, `stateData`, `setState(stateName, stateData, timeout, stopReason)` (cada argumento omitido conserva el valor actual; `timeout` por defecto `null` ≡ sin timeout), y los proxies de timers `startTimerWithFixedDelay`, `startTimerAtFixedRate`, `startSingleTimer`, `setTimer` (deprecado), `cancelTimer`, `isTimerActive(name)`, `isStateTimerActive`. `setState` equivale a una transición iniciada desde dentro del FSM, incluidos el timeout y la parada.

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/TestFSMRef.scala:22-136 (clase y companion)
- specs: testkit/src/test/scala/org/apache/pekko/testkit/TestFSMRefSpec.scala:21-106; actor-tests/src/test/scala/org/apache/pekko/actor/FSMActorSpec.scala:224-263 (uso con timers y terminación)

**Precondiciones.** Hereda las de `TestActorRef`. El actor debe ser un FSM (`T <:< FSM[S, D]`, comprobado en compilación). Los companions crean con supervisor por defecto = guardián y nombre aleatorio si no se dan (TestFSMRef.scala:119-146).

**Comportamiento.**
- `fsm` = `underlyingActor` (TestFSMRef.scala:48): cada consulta reevalúa la instancia (tras reinicio, la nueva).
- `setState(..)`: `fsm.applyState(FSM.State(stateName, stateData, Option(timeout), stopReason))` (66-72): ejecuta la transición (callbacks `onTransition`, timers de estado, `onTermination` si `stopReason`) en el hilo del llamador, fuera de cualquier invocación del `receive`.
- Los proxies de timers delegan en el FSM (`startTimerWithFixedDelay`… `cancelTimer`, `isTimerActive`, `isStateTimerActive`: 75-116).
- Los timers que disparan lo hacen desde el scheduler y envían el mensaje al actor; con la CTD (AC-TK-006.2) el `receive` corre en el hilo del scheduler, por lo que el test que consulta `stateName` desde otro hilo ve la transición cuando termina.

**Errores.** Los de `underlyingActor` (actor terminado ⇒ `IllegalActorStateException`). `setState` con un estado inexistente: el FSM lo trata como transición a un estado sin handler (log `Next state … does not exist`, AC-FSM).

**Orden / concurrencia / ciclo de vida.** `setState` y los accesos no están sincronizados con los timers en Pekko; en el spec de `timeout`, el test sondea con `awaitCond` dentro de `within(80 ms, 500 ms)` (TestFSMRefSpec.scala:46-49).

**Dependencias.** Filas: AC-TK-006.1 ; AC-TK-006.2 ; AC-FSM-001.1 ; AC-FSM-002 ; AC-FSM-004. Externas: ninguna.

**Equivalente Rust propuesto.** `TestFsmRef<S, D, A>` (con `A: Fsm<S, D>`) construido sobre `TestActorRef<A>`:
- `state_name() -> S` y `state_data() -> D` (requieren `Clone`, devuelven copias para no retener la guarda); `with_fsm(|fsm: &mut A| …)` para acceso completo.
- `set_state(StateUpdate::new().name(s).data(d).timeout(t).stop_reason(r))` (el builder sustituye a los argumentos por defecto de Scala: lo no indicado conserva el valor actual) que toma el candado de `TestActorRef` y llama a `fsm.apply_state(State{..})`.
- `start_timer_with_fixed_delay/at_fixed_rate/single_timer/cancel_timer/is_timer_active/is_state_timer_active` como delegaciones dentro de `with_fsm`.
Los timers usan el scheduler del sistema; con la CTD el `receive` disparado por el timer toma el candado del actor, de modo que `state_name()` desde otro hilo espera a que acabe una transición en curso (más estricto que Pekko, ver AC-TK-006.1). Viabilidad: alta, condicionada a que el trait `Fsm` de rakka exponga `apply_state` y los timers (unidad FSM).

**Diferencias observables.** `S`/`D` clonables en vez de referencias; el argumento `timeout: FiniteDuration = null` se modela con `Option<Duration>`; `setTimer` deprecado puede omitirse si AC-FSM-004 decide no portarlo (queda sujeto a esa decisión).

**Aserciones.**
- A1: acceso a estado: tras `startWith(1, "")`, `stateName == 1` y `stateData == ""`; tras `fsm ! "go"` pasan a `2` y `"go"` (transición síncrona vía CTD) — testkit/src/test/scala/org/apache/pekko/testkit/TestFSMRefSpec.scala:25-40.
- A2: `setState(stateName = 1)` cambia sólo el nombre (los datos siguen `"go"`); `setState(stateData = "buh")` cambia sólo los datos — testkit/src/test/scala/org/apache/pekko/testkit/TestFSMRefSpec.scala:41-46.
- A3: `setState(timeout = 100.millis)` arma el timeout de estado: dentro de `within(80.millis, 500.millis)` el FSM pasa a estado `2` con datos `"timeout"` — testkit/src/test/scala/org/apache/pekko/testkit/TestFSMRefSpec.scala:47-50.
- A4: timers: `isTimerActive("test")` es `false`; tras `startTimerWithFixedDelay("test", 12, 10.millis)` es `true`; tras `cancelTimer("test")` es `false` — testkit/src/test/scala/org/apache/pekko/testkit/TestFSMRefSpec.scala:53-65.
- A5: companion: supervisor por defecto = guardián (`fsm.underlyingActor.supervisor == guardian`); nombre dado; supervisor dado; supervisor y nombre — testkit/src/test/scala/org/apache/pekko/testkit/TestFSMRefSpec.scala:68-104.
- A6: `TestFSMRef` se usa para comprobar que todos los timers se cancelan al terminar el FSM (`startSingleTimer` por transición, `onTermination`) — actor-tests/src/test/scala/org/apache/pekko/actor/FSMActorSpec.scala:224-263 (aserciones de AC-FSM; aquí fijan el uso de `isTimerActive` desde el test).
- A7: `setState(stopReason = Some(Normal))` ejecuta `onTermination` y detiene el actor — derivada de TestFSMRef.scala:66-72 y del FSM, *no afirmada por spec*. Caracterización propuesta.
- A8: `isStateTimerActive` refleja el timeout de estado armado por `setState(timeout = …)` — derivada de TestFSMRef.scala:116, *no afirmada por spec*. Caracterización propuesta.

**Validación prevista.** A1-A5: tests portados 1:1 sobre `TestFsmRef` (A3 con reloj real porque usa el scheduler del sistema; la ventana 80-500 ms se escala con `timefactor`). A6: se valida en AC-FSM. A7-A8: caracterización propia. Sin oráculo: los specs fijan los valores.

**Decisiones.** Q-TK-2 (acceso al FSM subyacente por clones/cierre), Q-TK-1, Q-TK-8; depende de AC-FSM-001/002/004 para la API `Fsm` de rakka.

## AC-TK-008 — ExplicitlyTriggeredScheduler (tiempo manual)

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] timePasses(d) avanza el reloj lógico, ejecuta en el hilo llamante todas las tareas vencidas en orden de tiempo (incluidas las que se programan durante la ejecución) y respeta la cancelación; tareas con delay<=0 se ejecutan al programarlas; las periódicas se reprograman sólo si la ejecución no falló. currentTime en ms (AtomicLong); `scheduled` ConcurrentHashMap Item->tiempo; las tareas se ejecutan ordenadas por tiempo, una a una (re-evalúa el conjunto tras cada ejecución).

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/ExplicitlyTriggeredScheduler.scala:44-142
- reference.conf (actor): pekko.scheduler.implementation para inyectarlo (AC-SCH-010)

**Condiciones.**
- El runnable que lanza excepción no se reprograma y no propaga el error (Try) (testkit/src/main/scala/org/apache/pekko/testkit/ExplicitlyTriggeredScheduler.scala:98-102).
- cancel(): true si el size del mapa bajó; isCancelled se marca siempre (testkit/src/main/scala/org/apache/pekko/testkit/ExplicitlyTriggeredScheduler.scala:125-130).
- timePasses convierte `amount.toMillis`; no dilata (ETS doc 63-65).

**Orden / concurrencia / ciclo de vida.** Ejecución en el hilo del test, no en el ExecutionContext dado (testkit/src/main/scala/org/apache/pekko/testkit/ExplicitlyTriggeredScheduler.scala:38-42).

**Dependencias.** Filas: AC-SCH-010. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** scheduler manual inyectable (`ManualScheduler`)

**Diferencias observables.** Rust: ídem; `time_passes(Duration)` sin el sleep de 100 ms o con espera configurable (Q-TK-4).

**Aserciones.**
- A1: schedule(0s, 5s) y timePasses(12s) -> 3 ejecuciones — testkit/src/test/scala/org/apache/pekko/testkit/ExplicitlyTriggeredSchedulerSpec.scala:24-30.
- A2: schedule(5s,5s) y timePasses(12s) -> 2 ejecuciones — testkit/src/test/scala/org/apache/pekko/testkit/ExplicitlyTriggeredSchedulerSpec.scala:32-38.
- A3: scheduleOnce(5s) y timePasses(100s) -> 1 — testkit/src/test/scala/org/apache/pekko/testkit/ExplicitlyTriggeredSchedulerSpec.scala:40-46.
- A4: tres tareas idénticas a 1 s, timePasses(2s) -> 3 — testkit/src/test/scala/org/apache/pekko/testkit/ExplicitlyTriggeredSchedulerSpec.scala:48-56.
- A5: cancelar tras 7 s devuelve true y no ejecuta más (1 a los 11 s) — testkit/src/test/scala/org/apache/pekko/testkit/ExplicitlyTriggeredSchedulerSpec.scala:58-69.
- A6: cancelar una de tres: 3 a los 7 s y 5 a los 11 s — testkit/src/test/scala/org/apache/pekko/testkit/ExplicitlyTriggeredSchedulerSpec.scala:71-84.
- A7: cancelada de inmediato no ejecuta (0) — testkit/src/test/scala/org/apache/pekko/testkit/ExplicitlyTriggeredSchedulerSpec.scala:86-93.
- A8: avanzar el tiempo varias veces: 1 a los 7 s, 1 a los 8 s, 3 a los 18 s — testkit/src/test/scala/org/apache/pekko/testkit/ExplicitlyTriggeredSchedulerSpec.scala:95-106.

**Oráculo.** Jar oficial pekko-testkit: `tk-manual-scheduler.jsh` (misma tabla de tiempos). Procedencia: jars oficiales.

**Decisiones.** Q-TK-4

## AC-TK-009 — Utilidades de sincronización de test: TestLatch, TestBarrier y TestException

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual] Primitivas para tests: `TestLatch(count = 1)(implicit system)` (envoltorio de `CountDownLatch` que es `Awaitable[Unit]`; `countDown`, `isOpen`, `open`, `reset`, `ready/result(atMost)` con espera dilatada por `timefactor` y `TimeoutException("Timeout of <atMost> with time factor of <factor>")` al agotarse; `Duration.Inf`/no finito ⇒ `IllegalArgumentException("TestLatch does not support waiting for <d>")`; `DefaultTimeout = 5 s`); `TestBarrier(count)` (envoltorio de `CyclicBarrier`; `await()` y `await(timeout)` dilatados; al agotarse lanza `TestBarrierTimeoutException("Timeout of <t> and time factor of <factor>")`; `reset()`; `DefaultTimeout = 5 s`); `TestException(message)` (excepción `RuntimeException` sin traza, case class con igualdad por mensaje).

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/TestLatch.scala:32-62
- testkit/src/main/scala/org/apache/pekko/testkit/TestBarrier.scala:24-54
- testkit/src/main/scala/org/apache/pekko/testkit/TestException.scala:21
- testkit/src/main/scala/org/apache/pekko/testkit/package.scala (`dilated`: AC-TK-013) y TestKitExtension (`TestTimeFactor`: AC-TK-004)
- usos como evidencia: actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:219-230; testkit/src/test/scala/org/apache/pekko/testkit/PekkoSpecSpec.scala:67-72,101-104; actor-tests/src/test/scala/org/apache/pekko/actor/ActorWithStashSpec.scala:103,117,125; actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorSpec.scala:471

**Precondiciones.** Un `ActorSystem` implícito (para leer el `timefactor`). `TestBarrier.await()` sin timeout usa `DefaultTimeout`.

**Comportamiento.**
- `TestLatch.ready(atMost)`: `atMost` debe ser `FiniteDuration`; espera `atMost.dilated.toNanos`; si no se abrió, `TimeoutException`; devuelve `this` (TestLatch.scala:45-56). `result(atMost)` = `ready` (57-60). `open()` hace `countDown` hasta `isOpen`. `reset()` crea un `CountDownLatch` nuevo con el `count` original.
- `TestBarrier.await(timeout)`: `barrier.await(timeout.dilated.toNanos, NANOSECONDS)`; en `TimeoutException` lanza `TestBarrierTimeoutException` (TestBarrier.scala:42-50); `reset()` delega.
- `TestException`: `final case class TestException(message: String) extends RuntimeException(message) with NoStackTrace` (TestException.scala:21).

**Errores.** `TimeoutException` (latch); `TestBarrierTimeoutException` (barrier; otras excepciones de `CyclicBarrier` como `BrokenBarrierException` se propagan); `IllegalArgumentException` (latch con duración infinita).

**Orden / concurrencia / ciclo de vida.** Seguras entre hilos (`CountDownLatch`/`CyclicBarrier`). `TestLatch.reset()` no es atómico respecto de esperas en curso (reemplaza el campo `latch` no volátil).

**Dependencias.** Filas: AC-TK-004 ; AC-TK-013 ; AC-TK-001.1. Externas: `java.util.concurrent`.

**Equivalente Rust propuesto.** `testkit::util::{TestLatch, TestBarrier, TestException}`: `TestLatch::new(count, &system)` (con `std::sync::{Mutex, Condvar}`), `count_down()`, `is_open()`, `open()`, `reset()`, `ready(at_most: Duration) -> Result<(), TimeoutError>` con `at_most.dilated(&system)`; versión `async` opcional (`ready_async`) para tests sobre Tokio (Q-TK-4). `TestBarrier::new(n)` con `std::sync::Barrier` no admite timeout ni `reset`: se implementa sobre `Mutex<Condvar>` con generación para soportar `await_timeout` y `reset`. `TestException` como `struct TestException(String)` con `thiserror`, sin traza, `PartialEq`. Viabilidad alta.

**Diferencias observables.** `Awaitable[Unit]` → método propio (`ready`/`result`); mensajes de error idénticos; `Duration.Inf` se modela con `Option<Duration>` o con un tipo `FiniteDuration` que impide el caso (diferencia: error en compilación en lugar de `IllegalArgumentException`).

**Aserciones.**
- A1: `TestLatch(count)`: `countDown` `count` veces abre el latch y `Await.ready(latch, 5.seconds)` retorna; el orden de ejecución de los callbacks de terminación se observa a través del latch — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:215-235.
- A2: un `TestLatch` abierto desde `registerOnTermination` se completa al terminar el sistema (`Await.ready(latch, 2.seconds)`) — testkit/src/test/scala/org/apache/pekko/testkit/PekkoSpecSpec.scala:64-73.
- A3: `TestBarrier(2)` sincroniza el hilo del test con el actor: `state.finished.await()` (timeout por defecto) vuelve cuando el actor llama a `await`, y `beforeEach` hace `reset()` — actor-tests/src/test/scala/org/apache/pekko/actor/ActorWithStashSpec.scala:103,117,125.
- A4: `ready` con duración infinita lanza `IllegalArgumentException("TestLatch does not support waiting for Duration.Inf")` — derivada de TestLatch.scala:48-50, *no afirmada por spec*. Caracterización propuesta.
- A5: al agotarse el timeout, `TestLatch.ready` lanza `TimeoutException` con el mensaje `Timeout of <atMost> with time factor of <factor>` y `TestBarrier.await` lanza `TestBarrierTimeoutException` con `Timeout of <t> and time factor of <factor>` — derivada de TestLatch.scala:51-54 y TestBarrier.scala:44-48, *no afirmada por spec*.
- A6: la espera real es `atMost × timefactor` — derivada de `dilated`, *no afirmada por spec aquí* (ver AC-TK-004/013).
- A7: `reset()` deja el latch/barrera reutilizable con el contador original — derivada de TestLatch.scala:43 y TestBarrier.scala:52, *no afirmada por spec directa* (A3 usa `reset` en `beforeEach`).
- A8: `TestException("x")` tiene `getMessage == "x"`, sin traza (`getStackTrace.isEmpty`) e igualdad por mensaje — derivada de TestException.scala:21, *no afirmada por spec*.

**Validación prevista.** A1-A3: se validan junto a los specs de las unidades que los usan (ActorSystemSpec, PekkoSpecSpec, ActorWithStashSpec). A4-A8: tests de caracterización propios (con `timefactor` inyectado). Sin oráculo: lógica trivial sobre primitivas estándar.

**Decisiones.** Q-TK-4 (API bloqueante vs. asíncrona), Q-TK-5 (origen del `timefactor`).

## AC-TK-010 — TestKit.shutdownActorSystem / shutdown (verifySystemShutdown)

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual] Termina el sistema y espera hasta `duration` dilatado (por defecto min(10s dilatado, 10s)); si vence imprime (o lanza RuntimeException si verifySystemShutdown) `Failed to stop [<name>] within [<d>] \n<printTree>`. duration indefinida -> `10.seconds.dilated.min(10.seconds)` (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:1018).

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:871-876
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:1012-1031

**Condiciones.**
- Timeout: mensaje `Failed to stop [%s] within [%s] \n%s` (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:1024-1027); println si no verify, RuntimeException si verify.

**Orden / concurrencia / ciclo de vida.** terminate() y luego Await.ready(whenTerminated, d).

**Dependencias.** Filas: AC-SYS-004 ; AC-SYS-013 ; AC-TK-004. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `shutdown_actor_system(&system, timeout, verify)`

**Diferencias observables.** Rust: `Result<(), ShutdownTimeout>`.

**Aserciones.**
- A1: `shutdownActorSystem(sys, verifySystemShutdown = true)` vuelve sin error cuando el sistema termina (drena logs) — actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:190-206.
- A2: tope de 10 s para duración indefinida — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:1018, *no afirmada por spec*.
- A3: mensaje de fallo con printTree — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:1024-1029, *no afirmada por spec*.

**Oráculo.** Jar oficial: `tk-shutdown.jsh`.

**Decisiones.** ninguna específica

## AC-TK-011 — javadsl TestKit / EventFilter

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual, API Java] `javadsl.TestKit(system)` es una fachada que delega TODA su API en un `TestProbe` interno (`tp = new TestProbe(system)`): `getTestActor`/`getRef` (el actor es el de la probe, nombre `testProbe-N`, no `testActor-N`), `getSystem`, `msgAvailable`, `getLastSender`, `send`, `forward`, `reply`, `watch`/`unwatch`, `ignoreMsg`/`ignoreNoMsg`, `setAutoPilot`, `remaining*`, `within`, `awaitCond`, `awaitAssert`, `expectMsg*`/`expectMsgPF`/`expectMsgClass`/`expectMsgAnyOf`/`expectMsgAllOf*`/`expectMsgAnyClassOf`, `expectNoMessage`/`expectNoMsg`, `expectTerminated`, `fishForMessage`/`fishForSpecificMessage`, `receiveN`/`receiveOne`/`receiveWhile`, `childActorOf`, `dilated`, y el companion `shutdownActorSystem(..)` (10 s, sin verificar por defecto). Las funciones de emparejamiento Java (`JFunction`) son TOTALES: se envuelven en `CachingPartialFunction` que usa `JavaPartialFunction.NoMatch` como "no definido". `javadsl.EventFilter(clazz, system)` acepta una clase `Throwable` (filtro de `Error` con esa causa) o una clase de `LogEvent`; configuración encadenada `message` (exacto: `complete = true`), `startsWith` (`complete = false`), `matches` (patrón regex), `from` (fuente), `occurrences`; `intercept(Supplier)` ejecuta el bloque con el filtro equivalente (`ErrorFilter`/`WarningFilter`/`InfoFilter`/`DebugFilter`); `unknown LogLevel` ⇒ `IllegalArgumentException`.

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/javadsl/TestKit.scala:29-545 (clase), 863-937 (companion), 953-966 (`CachingPartialFunction`)
- testkit/src/main/scala/org/apache/pekko/testkit/javadsl/EventFilter.scala:23-92
- specs: testkit/src/test/java/org/apache/pekko/testkit/javadsl/TestKitApiTest.java:24-182 (compila toda la API; "actual impl is tested elsewhere"); actor-tests/src/test/java/org/apache/pekko/event/LoggingAdapterTest.java:51-123,137-175 (uso real de `expectMsgPF` Java con `Duration` y `getRef`); docs/src/test/java/jdocs/event/EventBusDocTest.java:277-349 (fuera de los módulos del alcance, sólo evidencia)

**Precondiciones.** Un `ActorSystem` ya creado; el sistema debe apagarse por el usuario. La clase no es thread-safe (una cola, una pila de `within`).

**Comportamiento.**
- Delegación: `expectMsg(max, obj, hint)` ⇒ `tp.expectMsg(max, hint, obj)` (nótese el orden de argumentos distinto: TestKit.scala:484); `expectMsgEquals` ≡ `expectMsg`; `expectNoMsg()` ≡ `expectNoMessage()`; `within(min, max, f)`/`within(max, f)` con `Supplier`; `awaitCond(max, interval, message, p)`; `awaitAssert(max, interval, a)`.
- Sobrecargas `java.time.Duration` convierten con `asScala`; `dilated` aplica `timefactor`.
- `expectMsgPF/fishForMessage/fishForSpecificMessage/receiveWhile/ignoreMsg` envuelven `JFunction` en `CachingPartialFunction`: `isDefinedAt(x)` evalúa `match(x)` y cachea el resultado; `NoMatch` ⇒ no definido (TestKit.scala:953-966).
- `javadsl.EventFilter`: `message(msg)` fija `pattern = false, complete = true`; `startsWith(msg)` ⇒ `complete = false`; `matches(regex)` ⇒ `pattern = true` (sin tocar `complete`); `from(source)`; `occurrences(n)` (por defecto `Integer.MAX_VALUE`); en `intercept`, si es filtro de error y no hay `exceptionType`, usa `Logging.noCause.getClass` (equivale a "cualquier `Error`") (EventFilter.scala:47-61).

**Errores.** `require` del constructor de `EventFilter`: `IllegalArgumentException("supplied class must either be LogEvent or Throwable")`; `intercept` con clase de `LogEvent` desconocida ⇒ `IllegalArgumentException("unknown LogLevel <clazz>")`. Los `expect*` fallan con `AssertionError` como en la probe (AC-TK-001.2).

**Orden / concurrencia / ciclo de vida.** Igual que `TestProbe` (AC-TK-001.x/002.x).

**Dependencias.** Filas: AC-TK-001.1 ; AC-TK-001.2 ; AC-TK-001.3 ; AC-TK-001.4 ; AC-TK-001.5 ; AC-TK-002.2 ; AC-TK-002.3 ; AC-TK-005.1 ; AC-TK-005.2 ; AC-TK-010 ; AC-JAPI-001. Externas: `java.util.function`.

**Equivalente Rust propuesto.** Ninguna distinción Java/Scala (Q-EVT-5): la API única `TestProbe`/`TestKit` de AC-TK-001/002 y `EventFilter` de AC-TK-005 cubre esta fila. Los puntos de paridad que exigen cuidado: (1) conservar los nombres alternativos (`expect_msg_equals`, `expect_no_msg`, `get_ref` ⇒ alias o simplemente `ref_()`), (2) el orden de argumentos de `expect_msg(max, obj, hint)` debe quedar documentado, (3) `EventFilter` con `message`/`starts_with`/`matches`/`from`/`occurrences` ya existe en la API fluida de AC-TK-005.1 y debe aceptar tipos de error (`TypeId`) o niveles (Q-TK-3). Viabilidad alta; la fila queda verificada por la suite de AC-TK-001/002/005.

**Diferencias observables.** Una sola API en lugar de dos; las funciones Java totales con `NoMatch` se sustituyen por `Option` o por `FnMut(&Msg) -> Option<R>` (la semántica "no definido" se conserva como `None`).

**Aserciones.**
- A1: `TestKitApiTest` compila todas las firmas Java de `TestKit` (awaitAssert/awaitCond, childActorOf, expectMsg*, fishFor*, receive*, ignoreMsg, expectMsgAllOf/AnyOf/AnyClassOf, getters, forward/send/watch/unwatch, setAutoPilot, `shutdownActorSystem(system, Duration)`) — testkit/src/test/java/org/apache/pekko/testkit/javadsl/TestKitApiTest.java:26-181 (sin ejecutar: el método se declara `compileOnlyTestCase`, con `testKit` nulo).
- A2: los eventos de log llegan a `getRef()` y `expectMsgPF(Duration, hint, f)` con una función total aplicada a un `LogEvent` verifica mensaje, nivel y MDC en 3 s — actor-tests/src/test/java/org/apache/pekko/event/LoggingAdapterTest.java:137-175.
- A3: `javadsl.TestKit.getRef()` entrega mensajes a una cola que `expectMsg*` examina (uso de la API en un test real) — actor-tests/src/test/java/org/apache/pekko/event/LoggingAdapterTest.java:51-87.
- A4: `javadsl.EventFilter` no tiene tests en el repositorio de Pekko (ni en testkit ni en actor-tests): su comportamiento es *no afirmado por spec*. Caracterización propuesta a partir de EventFilter.scala:47-91: `new EventFilter(classOf[RuntimeException], sys).message("expected").occurrences(1).intercept(..)` falla si no hay exactamente 1 `Error` con causa `RuntimeException` y mensaje exacto.
- A5: la construcción con una clase que no es `Throwable` ni `LogEvent` lanza `IllegalArgumentException("supplied class must either be LogEvent or Throwable")` — derivada de EventFilter.scala:25-27, *no afirmada por spec*.
- A6: `expectMsg(max, obj, hint)` pasa `hint` y `obj` a la probe en el orden `(max, hint, obj)` — derivada de TestKit.scala:484, *no afirmada por spec*.
- A7: `getRef`/`getTestActor` es el actor de una `TestProbe` (`testProbe-N`), no un `testActor-N` — derivada de TestKit.scala:56,62, *no afirmada por spec*.

**Validación prevista.** A1: test de compilación (que la API Rust expone las mismas operaciones; se genera una lista de operaciones esperadas y se comprueba por reflexión de macros o doc-test). A2-A3: portados dentro de LoggingAdapterTest (unidad EVT). A4-A7: caracterización propia. Sin oráculo.

**Decisiones.** Q-EVT-5 (API única; efecto: sin módulo `javadsl`), Q-TK-3 (tipos de causa en `EventFilter`).

## AC-TK-012.1 — PekkoSpec: configuración base de specs (testConf), nombre del sistema por la pila de llamadas, muteDeadLetters, ciclo before/after

*Unidad TK · visibilidad internal-required · analysis `extracted`*

**Contrato observable.** [contractual para portar specs] PekkoSpec crea un ActorSystem cuyo nombre sale del nombre de la clase del spec (TestKitUtils.testNameFromCallStack: sin paquete, sin `$..`, caracteres no [a-zA-Z_0-9-] -> `_`), con testConf: loggers=[TestEventListener], loglevel=WARNING, stdout-loglevel=WARNING, default-dispatcher fork-join (parallelism 8); apaga el sistema en afterAll. PekkoSpec mezcla TestKit con ScalaTest; en Rust el harness deberá ofrecer: sistema por test con nombre derivado, config base, `expect*`, `EventFilter`, apagado al final.

**Fuente.**
- testkit/src/test/scala/org/apache/pekko/testkit/PekkoSpec.scala:41-151
- testkit/src/main/scala/org/apache/pekko/testkit/TestKitUtils.scala:22-79

**Condiciones.**
- testNameFromCallStack sin clase en la pila: IllegalArgumentException(`Couldn't find [<cls>] in call stack`) (TestKitUtils.scala:53).

**Orden / concurrencia / ciclo de vida.** beforeAll: startCoroner + atStartup; afterAll: beforeTermination, shutdown(), afterTermination, stopCoroner (PekkoSpec.scala:107-117).

**Dependencias.** Filas: AC-TK-001.1 ; AC-TK-005.3 ; AC-TK-010. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `tests/common::TestSystem` (harness de tests de rakka)

**Diferencias observables.** Rust: harness de tests (Q-TK-6). Diferencia: sin Coroner/ScalaTest.

**Aserciones.**
- A1: testConf = loggers TestEventListener, loglevel WARNING, stdout-loglevel WARNING, fork-join 8 hilos — testkit/src/test/scala/org/apache/pekko/testkit/PekkoSpec.scala:42-58.
- A2: scrubActorSystemName: `a.b.C$$anon$1` -> `C` y no alfanuméricos -> `_` — derivada de TestKitUtils.scala:64-70, *no afirmada por spec leída*.
- A3: muteDeadLetters publica Mute(DeadLettersFilter(clazz)(MaxValue)) sólo si el log no está en debug — testkit/src/test/scala/org/apache/pekko/testkit/PekkoSpec.scala:133-139.
- A4: afterAll apaga el sistema — testkit/src/test/scala/org/apache/pekko/testkit/PekkoSpec.scala:115-120.

**Oráculo.** No aplica (infra de test).

**Decisiones.** Q-TK-6

## AC-TK-012.2 — Infra restante de specs: Coroner, TestTags, WithLogCapturing, AbstractSpec, PekkoJUnitActorSystemResource, TestActorRefJavaCompile, metrics/*

*Unidad TK · visibilidad internal-required · analysis `extracted`*

**Contrato observable.** [contractual para portar specs] Infraestructura de la suite de Pekko: (1) `Coroner`: `watch(duration, title, out, startAndStopDuration = 1 s, displayThreadCounts = false)` arranca un hilo "Coroner" que espera `duration`; si no se cancela, imprime `Coroner not cancelled after <ms>ms. Looking for signs of foul play...` y un informe (`Coroner's Report`, con volcado de hilos y secciones de bloqueos mutuos "Deadlocks found for monitors and ownable synchronizers" o "…for monitors, but NOT ownable synchronizers"); `WatchHandle.cancel()` evita el informe y espera hasta `startAndStopDuration` a que termine; `printReport(title, out)` imprime el informe en el acto; `WatchedByCoroner` lo conecta al ciclo `startCoroner`/`stopCoroner` de `PekkoSpec` con `expectedTestDuration.dilated`. (2) `TestTags`: etiquetas `timing`, `long-running`, `performance`, `gh-exclude`, `gh-exclude-aeron`. (3) `WithLogCapturing`: mezcla que, en cada test, recoge todos los `LogEvent` del sistema y los imprime con prefijo `| ` entre `--> [test] Start of log messages of test that [<resultado>]` y `<-- … End …` SÓLO si el test falla; `DebugLogSilencingTestEventListener` (no imprime Debug) y `SilenceAllTestEventListener` (no imprime nada). (4) `AbstractSpec` = `AnyWordSpecLike with Matchers with BeforeAndAfterEach`. (5) `PekkoJUnitActorSystemResource(name[, config])`: regla JUnit que crea el sistema en `before()` (config por defecto `PekkoSpec.testConf`; si falla imprime la traza y deja `null`) y lo apaga con `TestKit.shutdownActorSystem` en `after()`. (6) `TestActorRefJavaCompile`: comprobación de compilación de `TestActorRef.create` desde Java. (7) `metrics/*`: kit de benchmarking (Codahale metrics + HdrHistogram) usado por pruebas de rendimiento.

**Fuente.**
- testkit/src/test/scala/org/apache/pekko/testkit/Coroner.scala:37-254 (objeto `Coroner`: `watch` 92-134, `printReport` 139-254), 267-292 (`WatchedByCoroner`)
- testkit/src/test/scala/org/apache/pekko/testkit/TestTags.scala:18-23; testkit/src/test/scala/org/apache/pekko/testkit/WithLogCapturing.scala:25-97; testkit/src/test/scala/org/apache/pekko/testkit/AbstractSpec.scala:20-21
- testkit/src/test/java/org/apache/pekko/testkit/PekkoJUnitActorSystemResource.java:45-79; testkit/src/test/java/org/apache/pekko/testkit/TestActorRefJavaCompile.java:21-33
- testkit/src/test/scala/org/apache/pekko/testkit/metrics/ (MetricsKit.scala, MetricsKitOps.scala, HdrHistogram.scala, AveragingGauge.scala, KnownOpsInTimespanTimer.scala, FileDescriptorMetricSet.scala, MemoryUsageSnapshotting.scala, MetricKeyDSL.scala, reporter/PekkoConsoleReporter.scala)
- specs: testkit/src/test/scala/org/apache/pekko/testkit/CoronerSpec.scala:28-154; testkit/src/test/scala/org/apache/pekko/testkit/metrics/MetricsKitSpec.scala:38-87

**Precondiciones.** `Coroner` usa `ManagementFactory.getThreadMXBean` (JVM). `WithLogCapturing` exige un `implicit def system` y el listener adecuado en `pekko.loggers` (p. ej. `SilenceAllTestEventListener`).

**Comportamiento.**
- `Coroner.watch`: crea un `WatchHandleImpl`; el hilo "Coroner" cuenta hilos si `displayThreadCounts` (`Coroner Thread Count starts at <n> in <title>`), marca `started()`, espera `Await.result(handle, duration)`; si el resultado es `false` (expiró) llama `expired()`, imprime el aviso y `printReport`; `finished()` al acabar; `cancel()` completa la promesa con `true` y espera `finishedLatch` hasta `startAndStopDuration`. Al terminar (`finally`), con `displayThreadCounts` imprime `Coroner Thread Count started at <n>, ended at <m>, peaked at <p> in <title>` (Coroner.scala:121-126), y siempre hace `out.flush()` y `finished()`.
- `printReport(title, out)`: cabecera `#Coroner's Report: <title>` con arquitectura de SO, procesadores, carga, hora de arranque y uptime de la VM y uso de heap/non-heap (Coroner.scala:139-154); espera 300 ms para evitar una lectura de hilos incoherente (156-159); imprime `All threads:` con el volcado (`dumpAllThreads`) y `Deadlocks found <descripción>:` con los hilos bloqueados (`findDeadlockedThreads`, que usa sincronizadores de propiedad cuando la JVM lo soporta) (249-254).
- `WithLogCapturing.withFixture`: un `EventFilter(Int.MaxValue)` que acumula todos los eventos (`matches` siempre `true`), envuelve `test()` y lo registra con mensajes Debug `Logging started/finished for test [name]`; si el resultado no es éxito ni pendiente, imprime el log capturado con el `StdOutLogger`.
- `PekkoJUnitActorSystemResource`: ver arriba.
- `metrics/*`: `MetricsKit` es `private[pekko]` (su Scaladoc advierte que NO es una utilidad de micro-benchmarking, MetricsKit.scala:25-32); registra métricas Codahale (`counter`, `averageGauge`, `timedWithKnownOps`, `hdrHistogram`, `measureMemory/Gc/FileDescriptors`, MetricsKitOps.scala:28-107) y las reporta por consola (`PekkoConsoleReporter`). Sólo lo consume `ActorCreationPerfSpec` (actor-tests/src/test/scala/org/apache/pekko/actor/ActorCreationPerfSpec.scala:133), un spec de rendimiento sin contrato funcional. `HdrHistogram.update` lanza `IllegalArgumentException` con texto "can not be stored in this histogram" cuando el valor excede el máximo (HdrHistogram.scala:36-56).

**Errores.** `Coroner.printReport` captura `NonFatal` al imprimir el informe y escribe `Error displaying Coroner's Report` con la traza. Los demás componentes no definen errores propios.

**Orden / concurrencia / ciclo de vida.** El Coroner corre en su propio hilo; `cancel()` es síncrono con timeout `startAndStopDuration`. `WithLogCapturing` asume que el filtro se usa desde un único actor.

**Dependencias.** Filas: AC-TK-012.1 ; AC-TK-005.2 ; AC-TK-010 ; AC-EVT-017. Externas: JMX (`ThreadMXBean`), Codahale Metrics, HdrHistogram, JUnit, ScalaTest.

**Equivalente Rust propuesto.** Sólo lo que un spec portado necesite, en `tests/common` o en `rakka-testkit`:
- `Coroner`: `Coroner::watch(duration, title, out) -> WatchHandle` con hilo propio que, al expirar, imprime un informe de `std::thread` del proceso (nombres de hilos vivos; sin volcado de pilas porque `std` no lo permite: se puede usar `backtrace`/`tokio-console` opcionalmente) y la detección de deadlock con `parking_lot::deadlock` (feature `deadlock_detection`). Viabilidad media: el volcado de pilas de otros hilos y la detección de deadlocks de sincronizadores de la JVM no tienen equivalente directo.
- `TestTags`: atributos `#[ignore]`/features (`timing`, `long-running`, `performance`) o variables de entorno de `cargo-nextest`; sin equivalente semántico exacto.
- `WithLogCapturing`: guarda RAII `LogCapture` que registra los `LogEvent` del `EventStream` durante el test e imprime los eventos con prefijo `| ` sólo si el test falla (`std::thread::panicking()` en `Drop`); más los listeners `SilenceAllTestEventListener`/`DebugLogSilencingTestEventListener`. Viabilidad alta.
- `AbstractSpec`: innecesaria (Rust no tiene mixins de ScalaTest).
- `PekkoJUnitActorSystemResource`: guarda RAII `TestSystem` (AC-TK-012.1) que crea el sistema con `testConf` y llama `shutdown_actor_system` en `Drop`.
- `TestActorRefJavaCompile`: sin equivalente (no hay interop Java).
- `metrics/*`: no se porta (infraestructura de benchmark); si se necesitaran benchmarks, se usa `criterion`/`divan` fuera del contrato de comportamiento.

**Diferencias observables.** Sin interop Java ni JMX; el informe del Coroner no tendrá pilas de hilos (limitación de `std`); las etiquetas de ScalaTest se sustituyen por mecanismos de `cargo`.

**Aserciones.**
- A1: `Coroner.watch(100 ms, "XXXX", out)` genera un informe que contiene `"Coroner's Report"` y `"XXXX"` si no se cancela (`Await.ready(coroner, 5.seconds)`) — testkit/src/test/scala/org/apache/pekko/testkit/CoronerSpec.scala:39-47.
- A2: cancelar a tiempo no genera salida (`report === ""`) — testkit/src/test/scala/org/apache/pekko/testkit/CoronerSpec.scala:49-56.
- A3: con `displayThreadCounts = true` el informe contiene `"Coroner Thread Count starts at "`, `"Coroner Thread Count started at "` y el título, pero no `"Coroner's Report"` — testkit/src/test/scala/org/apache/pekko/testkit/CoronerSpec.scala:58-68.
- A4: `printReport` con dos hilos en deadlock (candados `ReentrantLock` en orden inverso) incluye `"Coroner's Report"` y, si la JVM soporta sincronizadores propios, la sección `"Deadlocks found for monitors and ownable synchronizers"` con los nombres `deadlock-thread-a` y `deadlock-thread-b`; si no, la sección `"…but NOT ownable synchronizers"` con `None` — testkit/src/test/scala/org/apache/pekko/testkit/CoronerSpec.scala:70-151.
- A5: `MetricsKit` mide descriptores de fichero, tiempo sobre un número conocido de operaciones, promedio con `Gauge`, histogramas, y falla con `"can not be stored in this histogram"` al desbordar — testkit/src/test/scala/org/apache/pekko/testkit/metrics/MetricsKitSpec.scala:40-85 (benchmarks de la suite JVM; ver Diferencias).
- A6: `WithLogCapturing` imprime el log capturado sólo si el test falla y con prefijo `| ` — derivada de WithLogCapturing.scala:31-76, *no afirmada por spec* (no hay spec de WithLogCapturing). Caracterización propuesta.
- A7: `PekkoJUnitActorSystemResource` crea el sistema en `before()` (nombre y config dados, `testConf` por defecto) y lo apaga en `after()` — derivada de PekkoJUnitActorSystemResource.java:51-85, *no afirmada por spec directa* (la usa LoggingAdapterTest.java:44-53 como infraestructura).

**Validación prevista.** A1-A3: portados (con un hilo "coroner" y `Vec<u8>` como `out`). A4: portado sólo si `parking_lot::deadlock` está disponible; el texto de las secciones es específico de JMX, por lo que la aserción se adapta a las cabeceras que defina rakka (diferencia documentada). A5: no se porta (JVM). A6-A7: caracterización propia. Sin oráculo: infraestructura de test.

**Decisiones.** Q-TK-6 (harness: `TestSystem`, `LogCapture`, `Coroner`), Q-TK-9 (límites del Coroner y exclusión de `metrics/*`).

## AC-TK-013 — Funciones de paquete: filterEvents(Iterable|varargs), filterException[T], TestDuration.dilated

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] ver AC-TK-005.2 (filterEvents/filterException) y AC-TK-004 (dilated). Ver AC-TK-005.2.

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/package.scala:25-67

**Condiciones.**
- Ver AC-TK-005.2.

**Orden / concurrencia / ciclo de vida.** Ver AC-TK-005.2.

**Dependencias.** Filas: AC-TK-005.1 ; AC-TK-004. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** funciones libres `filter_events`, `filter_exception`, trait `Dilate`

**Diferencias observables.** Ver AC-TK-005.2.

**Aserciones.**
- A1: filterEvents(EventFilter.warning(.., occurrences = 1)) { .. } — actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:76-78,89-90,102-103.

**Oráculo.** Jar oficial: `tk-intercept.jsh`.

**Decisiones.** Q-TK-4

## AC-TK-014 — TestKitUtils.testNameFromCallStack / scrubActorSystemName

*Unidad TK · visibilidad internal-required · analysis `extracted`*

**Contrato observable.** [contractual interno] Deriva un nombre de ActorSystem válido (AC-SYS-001.1) a partir de la pila de llamadas. En Rust no hay pila de clases; el nombre sale del nombre del test.

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/TestKitUtils.scala:22-79

**Condiciones.**
- scrubActorSystemName: quita paquete (`^.*\.`), anónimos (`\$\$?\w+`), reemplaza `[^a-zA-Z_0-9-]` por `_` y elimina `MultiJvmNode\d+` (TestKitUtils.scala:64-70).

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** Filas: AC-SYS-001.1 ; AC-TK-012.1. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** función de harness que usa `std::any::type_name`/nombre de test

**Diferencias observables.** Q-TK-6.

**Aserciones.**
- A1: el resultado siempre es un nombre válido de ActorSystem salvo cadena vacía — derivada de TestKitUtils.scala:64-70, *no afirmada por spec*.
- A2: sin clase en la pila lanza IllegalArgumentException — TestKitUtils.scala (cuerpo).

**Oráculo.** Jar oficial: no aplica (depende de pila JVM).

**Decisiones.** Q-TK-6

## AC-TK-015 — JavaSerializable y TestJavaSerializer (serializador `java-test`, id 23, binding del testkit)

*Unidad TK · visibilidad internal-required · analysis `extracted`*

**Contrato observable.** [contractual interno] El `reference.conf` del testkit registra el serializador `java-test` (clase `TestJavaSerializer`), su identificador numérico `23` y el binding `"org.apache.pekko.testkit.JavaSerializable" = java-test`. Un mensaje de test que mezcla el marcador `JavaSerializable` (que extiende `java.io.Serializable`) se serializa con `TestJavaSerializer`: `toBinary` escribe el objeto con `ObjectOutputStream` mientras `JavaSerializer.currentSystem` está fijado al sistema; `fromBinary` lo lee con `ClassLoaderObjectInputStream` sobre el `classLoader` del sistema; `includeManifest = false`. Sirve para mensajes ad hoc entre sistemas (remote/cluster) sin habilitar la serialización Java general (`allow-java-serialization = off`).

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/JavaSerializable.scala:17-22; testkit/src/main/scala/org/apache/pekko/testkit/TestJavaSerializer.scala:30-52
- testkit/src/main/resources/reference.conf:36-49
- specs: no hay spec propio en testkit; evidencia relacionada: actor-tests/src/test/scala/org/apache/pekko/serialization/SerializeSpec.scala:101,231 (`BothTestSerializableAndJavaSerializable`, otro marcador de test, no este)

**Precondiciones.** Un `ExtendedActorSystem` (constructor de `TestJavaSerializer(system)`); la entrada del `serialization-bindings` debe cargarse (viene en el `reference.conf` del testkit, o explícita como indica el Scaladoc: "needs to be explicitly enabled in the config").

**Comportamiento.** `TestJavaSerializer extends BaseSerializer` (identificador desde `serialization-identifiers`: 23). `toBinary(o)`: `ByteArrayOutputStream` + `ObjectOutputStream`, `JavaSerializer.currentSystem.withValue(system) { out.writeObject(o) }` (TestJavaSerializer.scala:38-44). `fromBinary(bytes, clazz)`: `ClassLoaderObjectInputStream(system.dynamicAccess.classLoader, UnsynchronizedByteArrayInputStream(bytes))`, `readObject` bajo `currentSystem.withValue(system)` y cierre en `finally` (45-51).

**Errores.** `NotSerializableException`/`ClassNotFoundException` propagados por `ObjectOutputStream`/`ObjectInputStream`.

**Orden / concurrencia / ciclo de vida.** Sin estado; la fijación de `currentSystem` es por hilo (`DynamicVariable`).

**Dependencias.** Filas: AC-SER-002 ; AC-SER-003.5 ; AC-SER-004.1 ; AC-SER-004.2 ; AC-SER-013 ; AC-CFG-001. Externas: serialización de objetos Java (JVM).

**Equivalente Rust propuesto.** La serialización de objetos Java no existe en Rust. Se propone un serializador de pruebas `TestSerializer` registrado en el registro de serializadores (Q-X-2) con identificador `23` y binding por marcador (`trait TestSerializable: serde::Serialize + DeserializeOwned`), implementado con un formato binario estándar (bincode/postcard) y `include_manifest = false`; el identificador `23` y el nombre `java-test` se conservan por paridad de configuración (`pekko.actor.serializers.java-test`). Viabilidad media: se preserva el contrato "mensajes de test ad hoc pasan por un serializador de id 23 sin habilitar el serializador general" pero no el formato de bytes (JVM) — un diferencial byte a byte con Pekko no es posible ni necesario.

**Diferencias observables.** Formato de bytes distinto (no interoperable con Pekko JVM); marcador como trait en lugar de interfaz `Serializable`; `JavaSerializer.currentSystem` sustituido por un contexto de serialización explícito (AC-SER-007.2).

**Aserciones.** No hay spec del serializador en el repositorio de Pekko; todo es *no afirmada por spec* y se propone como caracterización:
- A1: con la config del testkit, `serialization.serializerFor(clase JavaSerializable)` es `TestJavaSerializer` y su `identifier` es `23` — derivada de testkit/src/main/resources/reference.conf:38-48.
- A2: `fromBinary(toBinary(x), None) == x` para un mensaje `JavaSerializable` — derivada de TestJavaSerializer.scala:38-51.
- A3: `includeManifest == false` — derivada de TestJavaSerializer.scala:37.
- A4: con `allow-java-serialization = off` el binding sigue funcionando para mensajes marcados — derivada de reference.conf:46-48 y del propósito en el Scaladoc (TestJavaSerializer.scala:30-36).

**Validación prevista.** Caracterización propia (A1-A4) en la unidad SER/TK con el serializador de pruebas; diferencial no aplicable (formato JVM). Justificado: sin spec ni valor de comparación.

**Decisiones.** Q-X-2 (registro de serializadores por nombre), Q-TK-10 (serializador de pruebas sin formato Java).

## AC-TK-016 — TestActors: EchoActor, BlackholeActor y ForwardActor

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] Actores de uso común en tests: `EchoActor` responde a `sender()` el mismo mensaje sin modificar; `BlackholeActor` ignora todo; `ForwardActor(ref)` reenvía cada mensaje con `ref.forward(message)` (conserva el emisor original). Props: `echoActorProps`, `blackholeProps`, `forwardActorProps(ref)`.

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/TestActors.scala:22-57
- specs: testkit/src/test/scala/org/apache/pekko/testkit/TestActorsSpec.scala:16-41; usados como utilería en actor-tests (DeadLetterSupressionSpec, DeadLetterSuspensionSpec, ActorSystemDispatcherSpec.scala:88)

**Precondiciones.** Ninguna; `ForwardActor` exige un `ActorRef` destino.

**Comportamiento.** Ver contrato. `EchoActor.receive = { case message => sender() ! message }`; `BlackholeActor.receive = { case _ => }`; `ForwardActor.receive = { case message => ref.forward(message) }`. Los mensajes no se transforman ni se filtran; el echo usa el `sender()` del mensaje (si es `deadLetters` la respuesta va a dead letters).

**Errores.** Ninguno propio.

**Orden / concurrencia / ciclo de vida.** Los de un actor normal (mailbox FIFO por emisor).

**Dependencias.** Filas: AC-TK-001.1 ; AC-REF-002 ; AC-CELL-001. Externas: ninguna.

**Equivalente Rust propuesto.** `testkit::actors::{echo_props(), blackhole_props(), forward_props(target)}` y los tipos `EchoActor`, `BlackholeActor`, `ForwardActor`; `forward` conserva el emisor. Viabilidad alta.

**Diferencias observables.** Mensajes dinámicos (`Any`) bajo Q-X-3; igual comportamiento.

**Aserciones.**
- A1: `EchoActor` devuelve el mensaje unchanged (`"hello world"`) al `ImplicitSender` — testkit/src/test/scala/org/apache/pekko/testkit/TestActorsSpec.scala:20-29.
- A2: `ForwardActor(testActor)` reenvía `"forward me"` al `testActor` — testkit/src/test/scala/org/apache/pekko/testkit/TestActorsSpec.scala:31-40.
- A3: `BlackholeActor` no responde ni falla ante cualquier mensaje — derivada de TestActors.scala:35-39, *no afirmada por spec*. Caracterización propuesta: `expect_no_message`.
- A4: `ForwardActor` conserva el emisor original (el destino ve `sender == emisor`) — derivada de `ref.forward` (TestActors.scala:46-50), *no afirmada por spec*. Caracterización propuesta.

**Validación prevista.** A1-A2 portados con TestActorsSpec; A3-A4 caracterización propia. Sin oráculo.

**Decisiones.** Q-X-3.

## AC-TK-017 — SocketUtil (direcciones y puertos libres para tests)

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual] Utilidad de `object SocketUtil` para obtener direcciones TCP/UDP libres en `localhost`/loopback: `temporaryLocalPort(udp)`, `temporaryLocalPort(protocol: Tcp|Udp|Both)` (con `Both`, hasta 5 intentos para un puerto libre en TCP y UDP, y `RuntimeException("Unable to find a port that is free for tcp and udp")` si no lo logra), `temporaryServerAddress(address = RANDOM_LOOPBACK_ADDRESS, udp)`, `temporaryServerAddresses(n, hostname, udp)`, `temporaryServerHostnameAndPort`, `temporaryUdpIpv6Port(iface)`, `notBoundServerAddress([address])` (puerto 0). Para `RANDOM_LOOPBACK_ADDRESS` elige `127.20.<0-255>.<0-254>` si el sistema permite enlazar `127.20.0.0`, y `127.0.0.1` si no. Cada dirección se obtiene enlazando un socket al puerto 0 y cerrándolo (hay una ventana de carrera hasta que el test lo reutilice). Fallo al enlazar ⇒ `RuntimeException("Binding to <addr> failed with <msg>", cause)`.

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/SocketUtil.scala:27-139
- specs: no hay spec propio; usado por actor-tests/src/test/scala/org/apache/pekko/io/UdpIntegrationSpec.scala:24,77, actor-tests/src/test/scala/org/apache/pekko/io/CapacityLimitSpec.scala:20, actor-tests/src/test/scala/org/apache/pekko/io/TcpListenerSpec.scala:152, entre otros (unidad IO)

**Precondiciones.** Permiso para enlazar sockets locales. `temporaryUdpIpv6Port` exige una interfaz con dirección IPv6.

**Comportamiento.** `temporaryServerAddresses` genera `n` direcciones: por cada una elige host (según arriba), crea `ServerSocketChannel` (TCP) o `DatagramChannel` (UDP), `bind(new InetSocketAddress(host, 0))`, lee el puerto efectivo, cierra el socket y devuelve `InetSocketAddress(host, puerto)` (SocketUtil.scala:84-117). `canBindOnAlternativeLoopbackAddresses` se calcula una vez al cargar el objeto (31-38). `findBoth(tries)` repite `temporaryLocalPort(udp=false)` y comprueba que el puerto también esté libre en UDP (52-73).

**Errores.** `RuntimeException` descrito arriba (dos casos). Otros errores de E/S se propagan.

**Orden / concurrencia / ciclo de vida.** Sin estado compartido salvo la constante calculada al inicio; seguro entre hilos. No reserva el puerto: otro proceso puede tomarlo (inherente).

**Dependencias.** Filas: AC-IO-001 ; AC-TK-009. Externas: pila de red del SO.

**Equivalente Rust propuesto.** `testkit::net::{temporary_local_port(Protocol), temporary_server_address(host, udp), temporary_server_addresses(n, host, udp), notbound_server_address(..)}` con `std::net::{TcpListener, UdpSocket}`: enlazar a `127.20.x.y:0` (Linux) o `127.0.0.1:0` (macOS/Windows, donde sólo `127.0.0.1` está activo por defecto), leer `local_addr()` y soltar. La detección de `canBindOnAlternativeLoopbackAddresses` se hace una vez con `OnceLock`. Viabilidad alta.

**Diferencias observables.** La disponibilidad de `127.20.x.y` depende del SO; la fila ya contempla el respaldo a `127.0.0.1`. Los errores son `io::Error`/tipo propio (Q-X-5).

**Aserciones.** Sin spec propio: *no afirmada por spec*; caracterización propuesta derivada del código:
- A1: `temporary_server_address(RANDOM_LOOPBACK_ADDRESS)` devuelve una IP en `127.20.0.0/16` si el SO lo permite, si no `127.0.0.1`, con puerto > 0 — derivada de SocketUtil.scala:31-38,91-96.
- A2: tras obtener una dirección libre se puede enlazar a ella inmediatamente — derivada de SocketUtil.scala:101-116.
- A3: `temporary_local_port(Both)` devuelve un puerto libre tanto en TCP como en UDP — derivada de SocketUtil.scala:52-73.
- A4: `temporary_server_addresses(n, ..)` devuelve `n` direcciones distintas (en el mismo host se garantiza por puerto distinto mientras no se cierren) — derivada de SocketUtil.scala:84-117, *con la salvedad de que los sockets se cierran tras cada elección, por lo que la unicidad no está garantizada*; la caracterización sólo exige `len == n`.
- A5: `not_bound_server_address()` es `127.0.0.1:0` — derivada de SocketUtil.scala:132-134.

**Validación prevista.** Caracterización propia (A1-A5); el uso real se valida en la unidad IO. Sin oráculo (dependiente del SO).

**Decisiones.** Q-X-5 (errores).

## API mínima de testkit por spec portada

Qué necesita cada spec leído de `TestKit`/`TestProbe`/`EventFilter` (todos leídos con cuerpo). Las filas AC-TK dan el contrato; esta tabla da la dependencia inversa.

| Spec (path relativo a upstream/pekko) | API del testkit usada | Filas AC-TK |
|---|---|---|
| actor-tests/.../actor/ActorSystemSpec.scala | `PekkoSpec(config)` + `PekkoSpec.testConf`; `ImplicitSender`; `TestProbe()(sys)`, `probe.watch`, `probe.expectTerminated`; `EventFilter.info/warning(pattern=…, occurrences=1).intercept{}(sys)` anidado; `EventFilter[E](msg, occurrences).intercept` con `Await` dentro; `TestLatch` + `latch.countDown`/`Await.ready`; `.dilated`; `shutdown(sys)`; `TestKit.awaitCond`; `expectMsgType[ActorIdentity]`; `testActor` | 001.1, 001.5, 002.1, 005.1, 005.2, 005.3, 009, 010, 012.1 |
| actor-tests/.../actor/LocalActorRefProviderSpec.scala | `filterEvents(EventFilter.warning(…, occurrences=1)){…}`; `EventFilter[InvalidActorNameException](occurrences=1).intercept`; `watch`, `expectTerminated`, `awaitAssert(…, 1.second)`; `expectMsgType[ActorRef]`; `testActor` | 001.1, 001.5, 005.2, 013 |
| actor-tests/.../actor/DeadLetterSupressionSpec.scala | `ImplicitSender`; `TestActors.echoActorProps`; `watch`/`expectTerminated`; `TestProbe()` ×3 con `expectMsg(d, msg)`, `expectMsgType[…]`, `expectNoMessage()` y `expectNoMessage(Duration.Zero)` | 001.1, 001.2, 001.3, 002.1, 009 |
| actor-tests/.../actor/DeadLetterSuspensionSpec.scala | `PekkoSpec(String config)`; `ImplicitSender`; `EventFilter.info(start=…, occurrences=1).intercept{}`; `TestActors.echoActorProps`; `watch`/`expectTerminated` | 005.1, 005.2, 009, 012.1 |
| actor-tests/.../event/EventStreamSpec.scala | `within(3.seconds){…}`; `expectMsg`, `expectNoMessage()`, `expectMsgAllOf`; `testActor`; `TestProbe()` (`a1,a2 = TestProbe()`), `expectMsgType[AT]`, `expectNoMessage(1.second)`, `fishForMessage(hint=…)`; `watch`/`expectTerminated`; `awaitAssert`; `shutdown(sys)` | 001.1–001.5, 002.1, 012.1 |
| actor-tests/.../event/EventBusSpec.scala | `PekkoSpec(conf)`; `TestProbe()`, `fishForMessage(hint=…)`; `expectMsg`, `expectNoMessage()`; `watch`/`expectTerminated`; `testActor` | 001.1–001.4, 002.1 |
| actor-tests/.../event/LoggerSpec.scala | `TestProbe()`, `expectMsg("OK")`, `fishForMessage()`, `expectMsgPF(max=3.seconds)`, `expectNoMessage(0.5.seconds.dilated)`; `TestKit.shutdownActorSystem(sys[, verifySystemShutdown])`; captura de `Console.out`; `ActorSystem(name, config.withFallback(PekkoSpec.testConf))` | 001.2, 001.3, 001.4, 004, 010, 012.1 |
| actor-tests/.../event/MarkerLoggingSpec.scala | `PekkoSpec`, `ImplicitSender`, `TestEvent.Mute(EventFilter.info()/EventFilter[E](msg))`, `expectMsgType[Info3/Error]` | 003, 005.1, 005.3 |
| actor-tests/src/test/java/.../event/LoggingAdapterTest.java | `javadsl.TestKit` (`expectMsgClass`, `getRef`), `PekkoJUnitActorSystemResource` | 011, 012.2 |
| testkit/src/test/.../TestProbeSpec.scala | contrato propio de TestProbe (ver AC-TK-001.x/002.x) | 001.x, 002.x |

Resumen de la API mínima transversal que un port 1:1 debe ofrecer antes de portar cualquier spec de otra unidad: (1) `TestKit`/`TestProbe` con `ref`, `send`, `reply`, `watch`/`unwatch`, `expectMsg`, `expectMsgType`, `expectMsgPF`, `expectMsgAllOf`, `expectNoMessage`, `expectTerminated`, `fishForMessage`, `receiveN`, `ignoreMsg`, `within`, `awaitCond`, `awaitAssert`, `lastSender`; (2) `EventFilter` (`error/warning/info/debug/custom`, `[E]`, `start`/`pattern`/`message`/`source`/`occurrences`, `intercept`) y `filterEvents`/`filterException`, junto con `TestEventListener` y `TestEvent.Mute/UnMute`; (3) `PekkoSpec.testConf` (loggers=TestEventListener, loglevel/stdout-loglevel=WARNING); (4) `TestKit.shutdownActorSystem`; (5) `TestActors.echoActorProps` (AC-TK-016), `TestLatch` (AC-TK-009); (6) `ExplicitlyTriggeredScheduler` (AC-TK-008) para los specs de scheduler; (7) entrega síncrona del testActor (Q-TK-1, blocking).

## Descomposiciones

- AC-SYS-001 -> AC-SYS-001.1 (validación del nombre) y AC-SYS-001.2 (constructores/sobrecargas/nombre por defecto).
- AC-SYS-003 -> AC-SYS-003.1 (jerarquía de guardianes y mensajes ajenos) y AC-SYS-003.2 (protocolo de hooks de terminación: ningún test lo afirma).
- AC-EVT-013 -> AC-EVT-013.1 (formato `{}`/args), AC-EVT-013.2 (BusLogging/NoLogging) y AC-EVT-013.3 (LogSource y fábricas de adaptadores).
- AC-TK-001 -> AC-TK-001.1 (testActor/cola), .2 (expect*), .3 (expectNoMessage), .4 (receive*/fish*), .5 (within/remaining/await*).
- AC-TK-002 -> AC-TK-002.1 (API de probe), .2 (AutoPilot), .3 (childActorOf/DelegatingSupervisorStrategy).
- AC-TK-005 -> AC-TK-005.1 (reglas de coincidencia de EventFilter), .2 (intercept/filterEvents/occurrences), .3 (TestEventListener/Mute/UnMute).
- AC-TK-006 -> AC-TK-006.1 (TestActorRef) y AC-TK-006.2 (CallingThreadDispatcher).
- AC-TK-009 (cierre 2026-10-31) -> se conserva como AC-TK-009 (TestLatch, TestBarrier, TestException) y se añaden AC-TK-016 (TestActors) y AC-TK-017 (SocketUtil), porque son comportamientos independientes con y sin spec propio. No se usa el esquema padre/hijos porque el padre sigue teniendo contrato propio.
- AC-TK-012 -> AC-TK-012.1 (PekkoSpec/TestKitUtils) y AC-TK-012.2 (Coroner, TestTags, WithLogCapturing, metrics).

Los padres desaparecen; sus `impl`/`validation` eran `pending`, de modo que los hijos heredan `pending`.

## Omisiones agregadas

| ID | Evidencia (archivo/clave de Pekko no cubierto) |
|---|---|
| AC-SYS-022 | `ActorSystem.Settings.toString` -> `renderWithRedactions` (ActorSystem.scala:488-491; util/Helpers.scala:180-192); afirmado por ActorSystemSpec.scala:395-411 |
| AC-SYS-023 | `ActorSystemImpl.assertInitialized` (ActorSystem.scala:1053-1058) |
| AC-SYS-024 | `LocalActorRefProvider.resolveActorRef` (ActorRefProvider.scala:622-657) |
| AC-SYS-025 | `rootGuardian.getSingleChild` / `registerExtraNames` (ActorRefProvider.scala:499-508,546-551) |
| AC-EVT-022 | `event/japi/EventBusJavaAPI.scala` sin ninguna cita en el inventario |
| AC-TK-013 | `testkit/package.scala` (`filterEvents`, `filterException`) |
| AC-TK-014 | `testkit/TestKitUtils.scala` (uncited-testkit-main.txt) |
| AC-TK-016 | `testkit/TestActors.scala` (spec propio `TestActorsSpec`; separado de AC-TK-009 en el cierre 2026-10-31) |
| AC-TK-017 | `testkit/SocketUtil.scala` (sin spec propio; separado de AC-TK-009 en el cierre 2026-10-31) |
| AC-TK-015 | `testkit/JavaSerializable.scala`, `testkit/TestJavaSerializer.scala` y claves `pekko.actor.serializers.java-test` / `serialization-identifiers` / `serialization-bindings` del reference.conf del testkit (uncited-testkit-main.txt) |

Además, se separaron como filas propias comportamientos que el inventario mezclaba: textos de `Logging.scala` para arranque de loggers (dentro de AC-EVT-011), `UnhandledMessageForwarder` (AC-EVT-010) y `DeadLetterActorRef`/`EmptyLocalActorRef` (AC-EVT-008).

## Contradicciones halladas y corregidas

- AC-SYS-003: el inventario citaba LocalActorRefProviderSpec:71,84,97 para el protocolo RegisterTerminationHook; esos tests sólo afirman el `UnhandledMessage` de mensajes enviados a los guardianes (LocalActorRefProviderSpec.scala:70-107). Ningún test del repo afirma el protocolo de hooks (sólo remote lo usa). Separado en AC-SYS-003.1/.2.
- AC-SYS-007: el mensaje de Children.scala se cita como 312-315; el `throw` está en la línea 314.
- AC-SYS-009: el inventario no distinguía los dos mensajes de UnsupportedOperationException (con y sin nombre, ActorSystem.scala:930-931 y 936-937).
- AC-EVT-017: el formato descrito (`[LEVEL] [timestamp] [thread] [source] message`) omitía que el MDC (si existe) se imprime pegado a `[source]` con `formatMDC`, y las variantes con marker (`[LEVEL] [marker][ts] …`). Ver Logging.scala:1080-1102.
- AC-EVT-011: el inventario decía que cada logger 'debe responder LoggerInitialized dentro del timeout o el arranque falla'; en el código el timeout publica un Warning y devuelve la cadena `[TIMEOUT]`, y el fallo (LoggerInitializationException) ocurre porque esa respuesta no es LoggerInitialized (Logging.scala:214-228).
- AC-TK-004: el inventario hablaba de 'env var' para timefactor; en el código sólo existe la clave `pekko.test.timefactor` (TestKitExtension.scala:35-37).
- AC-TK-005: el inventario atribuía a TestEventListenerSpec la aserción de que `intercept` falla por ocurrencias; ese spec sólo prueba la coincidencia de filtros (TestEventListenerSpec.scala:23-83). El comportamiento de intercept/leeway sale del código (TestEventListener.scala:106-137) y de usos en ActorSystemSpec/LocalActorRefProviderSpec.
- AC-TK-005: el Scaladoc de `EventFilter` indica que `source` acepta `Class[_]`; el código sólo acepta `String` (TestEventListener.scala:142,193-198).
- AC-TK-010: el inventario decía 'sin test'; LoggerSpec.scala:189-205 usa `shutdownActorSystem(sys, verifySystemShutdown = true)`.
- AC-EVT-005: el inventario citaba ActorClassificationUnsubscriber.scala:81 (el archivo tiene 104 líneas; el mensaje `has terminated` está en la línea 68).
- AC-SYS-017: AskSpec está en `actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala` (no en el paquete actor).

## Filas que siguen `listed`

Tras el cierre documental (2026-10-31) **ninguna fila de este grupo queda `listed`**. Las filas extraídas en esta pasada conservan decisiones abiertas que afectan a `impl`/`validation`, no al contrato extraído: AC-SYS-001.2 (Q-X-4, Q-SYS-6), AC-SYS-018 (Q-X-2, Q-SYS-6), AC-SYS-020 y AC-SYS-021 (Q-SYS-3), AC-EVT-019 (Q-EVT-7), AC-EVT-021 (Q-EVT-4), AC-EVT-013.3 (Q-EVT-1, Q-EVT-6), AC-TK-006.1 (Q-TK-2, Q-TK-8), AC-TK-006.2 (Q-TK-1 bloqueante, Q-TK-7, Q-TK-8), AC-TK-007 (Q-TK-2), AC-TK-011 (Q-EVT-5, Q-TK-3), AC-TK-012.2 (Q-TK-6, Q-TK-9) y AC-TK-015 (Q-TK-10). Cuando se implementen, su `impl` queda `partial` hasta que el Owner responda.

## Cobertura

Archivos de Pekko del área y filas que los cubren (rutas relativas a `upstream/pekko`).

**actor/src/main/scala/org/apache/pekko/actor**
- ActorSystem.scala: AC-SYS-001.1/.2, -002, -004 a -015, -022, -023 (Settings también toca AC-CFG; ver nota en AC-SYS-022).
- ActorRefProvider.scala: AC-SYS-003.1/.2, -008, -016 a -019, -024, -025.
- ActorRef.scala (partes de dead letters, líneas 573-756): AC-EVT-008; el resto del archivo pertenece a AC-REF.
- Actor.scala (UnhandledMessage 293-303, ActorLogging/DiagnosticActorLogging 328-382, aroundReceive/unhandled 545-550 y 656-661): AC-EVT-010, -014, -018 (leído en el cierre 2026-10-31).
- ActorLogMarker.scala: AC-EVT-015 (leído en el cierre 2026-10-31).
- dungeon/Children.scala:310-316: AC-SYS-007. dungeon/FaultHandling.scala:186-196: AC-SYS-011 (leído en el cierre 2026-10-31).
- Main.scala: AC-SYS-021. util/ManifestInfo.scala: AC-SYS-020.

**actor/src/main/scala/org/apache/pekko/event**
- ActorClassificationUnsubscriber.scala: AC-EVT-005. AddressTerminatedTopic.scala: AC-DW-012 (fuera de este grupo; es el tópico de AddressTerminated para remote, sin test en classic). DeadLetterListener.scala: AC-EVT-009. EventBus.scala: AC-EVT-001 a -005. EventStream.scala: AC-EVT-006. EventStreamUnsubscriber.scala: AC-EVT-007. LoggerMailbox.scala: AC-EVT-020. Logging.scala: AC-EVT-011 a -017 (incl. .1/.2/.3). LoggingReceive.scala: AC-EVT-019. japi/EventBusJavaAPI.scala: AC-EVT-022. jul/JavaLogger.scala: AC-EVT-021.

**testkit/src/main/scala/org/apache/pekko/testkit**
- CallingThreadDispatcher.scala: AC-TK-006.2. ExplicitlyTriggeredScheduler.scala: AC-TK-008. JavaSerializable.scala, TestJavaSerializer.scala: AC-TK-015. TestActors.scala: AC-TK-016. SocketUtil.scala: AC-TK-017. TestBarrier.scala, TestException.scala, TestLatch.scala: AC-TK-009. TestActorRef.scala: AC-TK-006.1. TestEventListener.scala: AC-TK-005.1/.2/.3. TestFSMRef.scala: AC-TK-007. TestKit.scala: AC-TK-001.x, -002.x, -003, -010. TestKitExtension.scala: AC-TK-004. TestKitUtils.scala: AC-TK-014. package.scala: AC-TK-013, -004. javadsl/*: AC-TK-011.
- resources/reference.conf del testkit: AC-TK-004 (test.*), AC-TK-006.2 (calling-thread-dispatcher), AC-TK-015 (serializers).

**Claves de `actor/src/main/resources/reference.conf` del área (sólo las de este grupo)**: `pekko.loggers`, `logging-filter`, `loggers-dispatcher`, `logger-startup-timeout`, `loglevel`, `stdout-loglevel` (AC-EVT-011/016); `log-config-on-start` (AC-SYS-002); `log-dead-letters`, `log-dead-letters-during-shutdown`, `log-dead-letters-suspend-duration` (AC-EVT-009); `jvm-exit-on-fatal-error` (AC-SYS-015); `fail-mixed-versions` (AC-SYS-020); `pekko.actor.guardian-supervisor-strategy` (AC-SYS-008); `pekko.actor.debug.event-stream`/`unhandled`/`receive` (AC-EVT-006/010/019); `pekko.actor.mailbox.logger-queue` (AC-EVT-020); `pekko.coordinated-shutdown.close-actor-system-timeout` (AC-SYS-014). No se recorrió todo el reference.conf: las demás secciones (dispatchers, mailboxes, serialization, deployment, `creation-timeout`, `unstarted-push-timeout`, `daemonic`, `home`, `debug.autoreceive/lifecycle/fsm`) pertenecen a otras unidades (CFG, DSP, MBX, SER, CELL, FSM).

**Archivos de tests sin comportamiento observable propio (justificación)**
- testkit/src/test/scala/org/apache/pekko/testkit/metrics/* (MetricsKit, HdrHistogram, AveragingGauge, etc.): infraestructura de benchmarking de la suite JVM, leída en el cierre 2026-10-31; sólo la consume ActorCreationPerfSpec y no define comportamiento de actores (AC-TK-012.2, Q-TK-9).
- testkit/src/test/java/.../PekkoJUnitActorSystemResource.java, TestActorRefJavaCompile.java: boilerplate de compilación/JUnit (TestActorRefJavaCompile sólo verifica que compile la API Java); descritos en AC-TK-012.2.
- actor-tests/src/test/java/org/apache/pekko/event/ActorWithMDC.java: actor auxiliar de LoggingAdapterTest (cubierto por AC-EVT-014).
- actor-tests/src/test/java/org/apache/pekko/actor/setup/ActorSystemSetupTest.java: cubierto por AC-CFG-015 (sólo verifica que la API sea usable desde Java).
- actor-tests/src/test/scala-2.13+/org/apache/pekko/actor/ActorSystemScala213PlusSpec.scala: cubierto por AC-SYS-001.2 y AC-SYS-014 (`Using` termina el sistema).
- actor-tests/src/test/scala/org/apache/pekko/testkit/CallingThreadDispatcherModelSpec.scala: AC-TK-006.2 (leído en el cierre 2026-10-31).

## Revisión de exhaustividad (cierre documental 2026-10-31)

Se contrastó el árbol de fuentes del grupo (`actor/.../actor/ActorSystem.scala`, `ActorRefProvider.scala`, `ActorRef.scala` parcial, `Actor.scala` parcial, `ActorLogMarker.scala`, `event/**`, `util/ManifestInfo.scala`, `pekko/Main.scala`; `testkit/src/main/**`, `testkit/src/test/**` y las specs de `actor-tests` que usan estas APIs). Clasificación: **C** = cubierto por contratos concretos, **D** = duplicado o fachada equivalente, **I** = detalle interno sin contrato independiente (justificado), **F** = fuera de estos módulos con frontera explícita, **P** = pendiente de análisis (nunca se convierte en "fuera de alcance").

| Elemento revisado | Clase | Fila(s) o justificación |
|---|---|---|
| `ActorSystem.apply/create` (todas las sobrecargas), `BootstrapSetup`, `ProviderSelection` | C | AC-SYS-001.2, AC-SYS-018, AC-CFG-015 |
| Validación del nombre, `uid`, `Version` | C | AC-SYS-001.1, AC-SYS-013, AC-CFG-003 |
| `ActorSystemImpl.start` / `_initialized` / `assertInitialized` | C | AC-SYS-002, AC-SYS-023 |
| `terminate` / `finalTerminate` / `isTerminating` / `close` / `abort` | C | AC-SYS-004, AC-SYS-014, AC-SYS-011 |
| `registerOnTermination` + `TerminationCallbacks` | C | AC-SYS-006 |
| Accesores públicos (`name`, `settings`, `startTime`, `uptime`, `eventStream`, `log`, `deadLetters`, `scheduler`, `dispatchers`, `mailboxes`, `dispatcher`, `/`, `child`, `descendant`, `logConfiguration`, `toString`) y getters Java (`getEventStream`, `getScheduler`, `getDispatcher`, `getWhenTerminated`) | C / D | AC-SYS-012 y AC-SYS-005; los getters Java son fachada equivalente (Q-EVT-5) |
| `ExtendedActorSystem` (`provider`, `guardian`, `systemGuardian`, `systemActorOf`, `logFilter`, `printTree`) | C | AC-SYS-013, AC-SYS-010 |
| `threadFactory` / `MonitorableThreadFactory`, `dynamicAccess` / `ReflectiveDynamicAccess` | F | AC-DSP-015 cubre el factory de hilos; `dynamicAccess` se sustituye por el registro de fábricas (Q-X-2, Q-SYS-6). Frontera: ninguna API pública de rakka expone estos dos |
| `uncaughtExceptionHandler` | C | AC-SYS-015 (Q-SYS-2) |
| `ActorSystem.Settings` (claves) y `toString` redactado | F / C | claves: AC-CFG-* (otro grupo); `toString`: AC-SYS-022 |
| `Settings.amendSlf4jConfig` | F | AC-CFG-018 (otro grupo; ConfigSpec.scala:186-204); depende de `pekko-slf4j`, fuera de los módulos |
| Registro de extensiones (`registerExtension`, `extension`, `hasExtension`, `findExtension` con `creation-timeout`) | F | AC-EXT-001.2 / AC-EXT-002 (otro grupo); el orden de carga está en AC-SYS-002 |
| `createScheduler` / `stopScheduler` | F | el contrato del scheduler y de `close()` está en AC-SCH-* (otro grupo); el enganche `registerOnTermination(stopScheduler)` está en AC-SYS-002 |
| `ActorSystem.stop(actor)` (enrutado a `StopChild` de los guardianes) | D | AC-CELL-010 lo cubre (system.stop); el actor guardián está en AC-SYS-003.1 |
| `actorOf` con `guardianProps` (UnsupportedOperationException) | C | AC-SYS-009 |
| `ActorRefProvider` (trait), `rootGuardianAt`, `getExternalAddressFor`, `getDefaultAddress`, `serializationInformation`, `addressString` | C | AC-SYS-018 |
| `LocalActorRefProvider.actorOf` | C | AC-SYS-016 |
| `Guardian` / `SystemGuardian` (protocolo de hooks) | C | AC-SYS-003.1, AC-SYS-003.2, AC-SUP-014 |
| `tempPath`, `tempContainer`, `VirtualPathContainer` | C | AC-SYS-017 |
| `resolveActorRef`, hijos especiales (`temp`, `deadLetters`, `extraNames`), `bubble-walker` | C | AC-SYS-024, AC-SYS-025, AC-SYS-019 |
| `IgnoreActorRef` | F | AC-REF / AC-PATH-015 (otro grupo); la dirección `pekko://local/ignore` se afirma en AC-SYS-018 A8 |
| `ManifestInfo`, `Main` | C | AC-SYS-020, AC-SYS-021 (solape con AC-UTIL-016.9: el contrato completo vive en AC-SYS-020) |
| `EventBus` y clasificaciones, `EventStream`, unsubscribers, `DeadLetterListener`, `UnhandledMessage`, `LoggingBus`, `LogEvent`, `LoggingAdapter`, `BusLogging`/`NoLogging`, `LogSource`, `Diagnostic*`, `Marker*`, `LoggingFilter`, `StdOutLogger`, `LoggerMailbox` | C | AC-EVT-001..017, AC-EVT-020 |
| `ActorLogging`, `DiagnosticActorLogging` | C | AC-EVT-018, AC-EVT-014 |
| `LoggingReceive` | C | AC-EVT-019 |
| `JavaLogger` / `JavaLoggingFilter` / `jul.Logger` / `JavaLogging` | C | AC-EVT-021 (Q-EVT-4) |
| `japi.EventBus`, `LookupEventBus`, `SubchannelEventBus`, `ScanningEventBus`, `ManagedActorEventBus` | D | AC-EVT-022: fachada 1:1 de AC-EVT-002..005 (Q-EVT-5) |
| `Logging.LoggerException`, `Logging.LogEventException` | I | `LoggerException` es la causa artificial de `Error` sin causa (AC-EVT-012); `LogEventException` sólo lo consume `AbstractDispatcher.reportFailure` (AC-DSP, frontera) para republicar el evento original |
| `AddressTerminatedTopic` | F | AC-DW-012 (otro grupo; sin test en classic) |
| `TestKit` / `TestKitBase` / `TestProbe` / `TestActor` / `AutoPilot` | C | AC-TK-001.x, AC-TK-002.x, AC-TK-003 |
| `TestKitExtension`, `package.scala`, `TestKitUtils` | C | AC-TK-004, AC-TK-013, AC-TK-014 |
| `EventFilter`, `TestEventListener`, `TestEvent.Mute/UnMute` | C | AC-TK-005.x |
| `ExplicitlyTriggeredScheduler` | C | AC-TK-008 |
| `TestKit.shutdownActorSystem` | C | AC-TK-010 |
| `TestActorRef` | C | AC-TK-006.1 |
| `CallingThreadDispatcher` / `CallingThreadMailbox` / `CallingThreadDispatcherQueues` | C | AC-TK-006.2 |
| `TestFSMRef` | C | AC-TK-007 |
| `TestLatch`, `TestBarrier`, `TestException` | C | AC-TK-009 |
| `TestActors` | C | AC-TK-016 (nueva) |
| `SocketUtil` | C | AC-TK-017 (nueva) |
| `javadsl.TestKit`, `javadsl.EventFilter` | D | AC-TK-011: delegan 1:1 en `TestProbe`/`EventFilter` (Q-EVT-5) |
| `JavaSerializable`, `TestJavaSerializer` y claves `serializers`/`serialization-identifiers`/`serialization-bindings` del testkit | C | AC-TK-015 (Q-TK-10) |
| `pekko.test.*` (`timefactor`, `filter-leeway`, `single-expect-default`, `expect-no-message-default`, `default-timeout`, `calling-thread-dispatcher`) | C | AC-TK-004, AC-TK-006.2 |
| `PekkoSpec`, `PekkoSpecSpec` | C | AC-TK-012.1 |
| `Coroner`, `WatchedByCoroner`, `TestTags`, `WithLogCapturing`, `AbstractSpec`, `PekkoJUnitActorSystemResource`, `TestActorRefJavaCompile` | C | AC-TK-012.2 |
| `testkit/src/test/.../metrics/*` (MetricsKit, MetricsKitOps, HdrHistogram, AveragingGauge, KnownOpsInTimespanTimer, FileDescriptorMetricSet, MemoryUsageSnapshotting, MetricKeyDSL, PekkoConsoleReporter) | I | AC-TK-012.2 / Q-TK-9: infraestructura de benchmarking (Codahale + HdrHistogram) de la suite JVM, `private[pekko]`, consumida únicamente por `ActorCreationPerfSpec` (rendimiento, sin contrato funcional). Se leyó `MetricsKit`, `MetricsKitOps`, `HdrHistogram` y su spec; no hay comportamiento observable de actores |
| Specs `ImplicitSenderSpec`, `DefaultTimeoutSpec`, `TestTimeSpec`, `ExplicitlyTriggeredSchedulerSpec`, `TestEventListenerSpec`, `TestProbeSpec` | C | AC-TK-003, AC-TK-004/013, AC-TK-008, AC-TK-005.x, AC-TK-001/002 |
| Interrupciones (`Thread.interrupt`) en `ActorModelSpec:422-477` | P | Q-TK-7: no portable a Rust; requiere decisión del Owner (exclusión de dos casos), no es "fuera de alcance" por defecto |
| Handlers asíncronos ejecutados en el hilo del emisor (CTD) | P | Q-TK-8: depende del modelo de runtime/Receive (Q-X-3, Q-X-4) |
| Re-lectura contra fuente de los bloques `extracted` previos con aserciones "derivada del código / no afirmada por spec" no incluidos en la auditoría puntual de este cierre (AC-SYS-003.2, -005..-008, -015, -019, -022..-025, AC-EVT-002..-005, -008, -009, -011, -013.1, -013.2, -017, -020, AC-TK-001.1..001.5, -002.x, -004, -005.x, -010, -012.1, -013, -014) | P | Se corrigieron los bloques con textos explícitos de "no leído" (AC-SYS-004, -009; AC-EVT-001, -010, -012, -014, -015) y se re-leyeron CoordinatedShutdownSpec:535-582, ActorSystemDispatcherSpec:105-168, LoggingAdapterTest.java:114-125, Actor.scala:293-303,328-382,545-550,656-661 y ActorLogMarker.scala. El resto sigue marcado con sus leyendas originales; deben re-leerse en la ejecución de su unidad |

### Hallazgos de la lectura completa

- **Contradicción (AC-SYS-016):** el antiguo A4 afirmaba que `SupervisorSpec:510-540` probaba "dispatcher/mailbox inválido"; el spec prueba un `MailboxType` cuyo `create` lanza (`ConfigurationException` con causa para hijos y routers hijos; `ActorInitializationException` para top-level).
- **Contradicción (AC-SYS-010):** `ActorSelectionSpec:162,245` no usa `systemActorOf`; selecciona `/`, `/user` y `/system`.
- **Contradicción (AC-SYS-004):** `CoordinatedShutdownSpec:546,557` sí afirma el comportamiento de `terminate()` frente a `CoordinatedShutdown`; se añadieron A6-A9.
- **Contradicción (AC-SYS-009):** `ActorSystemDispatcherSpec:156` pertenece a la clase `ActorSystemDispatchersSpec` y afirma que el guardián `/user` con `guardianProps` usa el default-dispatcher.
- **Omisión (AC-EVT-012):** `LogEvent.apply` lanza `IllegalArgumentException("Unsupported log level [..]")` para niveles distintos de los cuatro soportados (Logging.scala:771-800).
- **Comportamiento no obvio (AC-EVT-013.3):** `LogSource.fromAnyRef(o, system)` no añade la dirección del sistema para `Class`/`Actor`/`ActorRef`/`String`, a diferencia de `Logging(system, src)` (Logging.scala:395-402 vs 573-576).
- **Comportamiento no obvio (AC-EVT-019):** `LoggingReceive` registra en `isDefinedAt`, antes de ejecutar el cuerpo; el orden Debug → respuesta es observable y obliga a que el `Receive` de rakka exponga `is_defined_at` (Q-EVT-7).
- **Comportamiento no obvio (AC-TK-011):** `javadsl.TestKit` usa un `TestProbe` (no el `TestKit` Scala), por lo que su `getRef` es `testProbe-N`.
- **Solape (AC-SYS-020 / AC-UTIL-016.9):** ambas filas citan `ManifestInfo`; el contrato completo vive en AC-SYS-020 y AC-UTIL-016.9 debe limitarse a `util.Version`/referencias.

### Representación propuesta de TestActorRef / CallingThreadDispatcher / TestFSMRef (resumen)

Choque: Pekko devuelve de `underlyingActor` una referencia compartida y mutable al actor, que el test lee mientras la CTD ejecuta `receive` en su propio hilo sin sincronización; Rust exige un único `&mut A` y propiedad clara. Representación (detalle en AC-TK-006.1, AC-TK-006.2 y AC-TK-007): (1) candado reentrante por actor (`ReentrantMutex`) que además es el `ctdLock` de la CTD; (2) estado en una celda protegida por ese candado, con acceso por cierre `with_actor(|a| ..)` o guarda `!Send` (`underlying_actor()`); (3) la reentrada (un actor enviándose mensajes directa o indirectamente) nunca crea un segundo `&mut A`: sólo encola en la cola del hilo y retorna, y la invocación exterior drena la cola en FIFO; (4) colas por (buzón, hilo) en `Mutex<HashMap<ThreadId, VecDeque<_>>>` con `resume`/`cleanUp` que las vacían hacia el hilo actual / dead letters; (5) suspensión con `Mutex<bool>` bajo el mismo candado que la cola; (6) mensajes de sistema por una cola propia procesada de inmediato; (7) el manejador se ejecuta en el hilo del emisor (manejadores `async` con `block_on` sin reactor). Se preservan: ejecución síncrona (`!` retorna tras `receive`), orden determinista con un hilo, reentrada diferida, `ask` con futuro ya completo, `Kill`/reinicio síncronos. Diferencias: acceso al actor serializado (más estricto), sin `Thread.interrupt` (Q-TK-7), handlers async acotados (Q-TK-8).
