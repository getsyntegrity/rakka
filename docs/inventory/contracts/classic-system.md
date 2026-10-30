# Contratos verificables: grupo `classic-system` (SYS, EVT, TK)

Fuente: Apache Pekko 1.7.1 (`upstream/pekko`, commit `9a54d129…`). Todas las rutas son relativas a `upstream/pekko`. Las citas `path:line` fueron generadas desde lecturas con números de línea y validadas mecánicamente (existencia de archivo y rango, más un conjunto de anclas de texto en las líneas clave). `analysis=extracted` sólo si se leyó la implementación y el cuerpo de los tests citados; si no, `listed` con el motivo en `notes`. Las aserciones marcadas *no afirmada por spec* salen del código, no de un test.

Convención de `impl`/`validation`: todas las filas del grupo están `pending` (ninguna implementación existe aún). Decisiones transversales referenciadas sin redefinir: Q-X-1 (HOCON), Q-X-2 (registro de fábricas), Q-X-3 (mensajes dinámicos), Q-X-4 (runtime), Q-X-5 (fallos), Q-X-6 (jerarquía de eventos), Q-X-7 (loggers actor vs backend), Q-X-8 (typed sobre classic).

## AC-SYS-001.1 — Validación del nombre de ActorSystem (regex y mensaje)

*Unidad SYS · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] El nombre debe cumplir ^[a-zA-Z0-9][a-zA-Z0-9-_]*$ (match completo); si no, IllegalArgumentException con mensaje fijo. Crear un sistema con un nombre inválido falla antes de arrancar nada. Nombres válidos: ASCII alfanumérico inicial, luego alfanuméricos, '-' o '_'. Los nombres inválidos del test incluyen prefijo '-' o '_', espacio, '*', '@', '#', '$', '%', '/'.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:840-843 (validación en el cuerpo de ActorSystemImpl)
- Scaladoc/doc pública: n/a (el patrón aparece sólo en el código y en el spec)
- reference.conf: n/a

**Condiciones.**
- Normal: 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_' es válido (actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:160-162).
- Error: IllegalArgumentException con mensaje exacto `invalid ActorSystem name [<name>], must contain only word characters (i.e. [a-zA-Z0-9] plus non-leading '-' or '_')` (actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:840-843; el spec sólo verifica el tipo, el texto es derivado del código).
- Límite: cadena vacía no matchea (primer carácter obligatorio); el spec actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:152-158 lo cubre por propiedad (cualquier String que no matchee el regex).

**Orden / concurrencia / ciclo de vida.** Se valida en el constructor de ActorSystemImpl, antes de crear settings/proveedor/hilos; no hay efectos secundarios si falla.

**Dependencias.** Filas: -. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `ActorSystemName::parse(&str) -> Result<ActorSystemName, InvalidSystemName>`

**Diferencias observables.** Rust: tipo `ActorSystemName` validado en construcción. Diferencia observable: el error es `Err`, no excepción (Q-X-5); el regex se aplica sobre Unicode como ASCII estricto (igual que Java: `[a-zA-Z0-9]` no admite letras no ASCII).

**Aserciones.**
- A1: los nombres '-hallowelt', '_hallowelt', 'hallo welt', 'hallo*welt', 'hallo@welt', 'hallo#welt', 'hallo$welt', 'hallo%welt', 'hallo/welt' lanzan IllegalArgumentException — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:135-150.
- A2: todo String que no matchee `^[a-zA-Z0-9][a-zA-Z0-9-_]*$` lanza IllegalArgumentException (propiedad) — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:152-158.
- A3: el nombre con los 64 caracteres válidos (a-z, A-Z, 0-9, '-', '_') se acepta — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:160-162.
- A4: el mensaje de error es el literal de actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:840-843 — derivada del código, *no afirmada por spec*.

**Oráculo.** Jar oficial pekko-actor 1.7.1: escenario propuesto `tools/oracle/sys-name-validation.jsh` (no existe) que llama `ActorSystem.create(n)` y vuelca excepción+mensaje para una tabla de nombres; procedencia: jars oficiales.

**Decisiones.** Q-X-5

## AC-SYS-001.2 — Constructores de ActorSystem: apply/create y sus sobrecargas, nombre por defecto "default"

*Unidad SYS · visibilidad public · analysis `listed`*

**Contrato observable.** [contractual] `ActorSystem()` usa el nombre "default"; las sobrecargas componen (name, config, classLoader, defaultExecutionContext, setup/BootstrapSetup). Un único punto de creación con varias formas; `ActorSystem()` produce un sistema llamado "default" (actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:263).

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:193-330 (create/apply)
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:263 (`apply("default")`)
- reference.conf: n/a

**Condiciones.**
- Normal: `ActorSystem()` -> nombre "default" (actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:251 lo usa sin nombre).
- Error: nombre inválido -> ver AC-SYS-001.1.
- Pendiente de leer: orden de precedencia config/BootstrapSetup/ActorSystemSetup.

**Orden / concurrencia / ciclo de vida.** n/a: la construcción es síncrona; start() corre dentro de apply (ver AC-SYS-002).

**Dependencias.** Filas: AC-SYS-001.1 ; AC-CFG-001 ; AC-CFG-015. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `ActorSystem::builder(name).config(..).execution_context(..).build()` y `ActorSystem::default()`

**Diferencias observables.** Rust: builder; sin ClassLoader. Diferencia observable: no hay sobrecarga por ClassLoader (Q-X-1/Q-X-2 para config y fábricas).

**Aserciones.**
- A1: `ActorSystem()` crea un sistema utilizable cuyo `whenTerminated` no está completo hasta terminar — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:250-261.
- A2: `Using(ActorSystem()) {..}` termina el sistema al salir (AutoCloseable) — actor-tests/src/test/scala-2.13+/org/apache/pekko/actor/ActorSystemScala213PlusSpec.scala:29-35.

**Oráculo.** No aplica hasta leer el cuerpo de las sobrecargas; después, jar oficial.

**Decisiones.** Q-X-1, Q-X-4

## AC-SYS-002 — Secuencia de arranque (start): provider.init (guardián raíz, cadena de Watch /user→/system→/, loggers por defecto), deadLetterListener, unsubscriber, chequeo de módulos, extensiones, log-config-on-start

*Unidad SYS · visibilidad internal-required · analysis `listed`*

**Contrato observable.** [contractual interno] Orden fijo: registrar stopScheduler en terminación; provider.init; marcar inicializado; deadLetterListener si log-dead-letters>0; startUnsubscriber; ManifestInfo; loadExtensions si no está terminando; logConfiguration si log-config-on-start. Si algo falla con NonFatal: terminate() y se relanza el error. Al volver de la construcción el sistema ya tiene guardianes, loggers por defecto, listener de dead letters (si procede), unsubscriber del EventStream y extensiones cargadas.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1059-1083
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:613-620
- reference.conf: pekko.log-config-on-start (reference.conf:50), pekko.log-dead-letters (reference.conf:58), pekko.library-extensions/extensions (reference.conf:78,83)

**Condiciones.**
- Normal: init -> deadLetterListener (nombre `deadLetterListener` bajo /system) -> startUnsubscriber -> checkSameVersion -> loadExtensions -> logConfiguration (actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1065-1074).
- Error: `NonFatal(e)` -> `terminate()` (o `stopScheduler` si terminate falla) y `throw e` (actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1075-1081).
- Si `terminating` ya es true no se cargan extensiones (actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1072-1073).

**Orden / concurrencia / ciclo de vida.** provider.init: rootGuardian.start; systemGuardian.sendSystemMessage(Watch(guardian, systemGuardian)); rootGuardian.sendSystemMessage(Watch(systemGuardian, rootGuardian)); luego startDefaultLoggers (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:613-620). `_initialized=true` antes del deadLetterListener (habilita assertInitialized).

**Dependencias.** Filas: AC-SYS-003.1 ; AC-EXT-003 ; AC-EVT-007 ; AC-EVT-009 ; AC-EVT-011. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `ActorSystemBuilder::build` ejecuta la secuencia y devuelve `Err` tras terminar el sistema

**Diferencias observables.** Rust: `build()` síncrono; los loggers como actores dependen de Q-X-7. Diferencia observable: nombre de los actores de sistema (`log1-DefaultLogger`, `deadLetterListener`, `eventStreamUnsubscriber-1`) se conserva si se mantiene la jerarquía /system.

**Aserciones.**
- A1: el listener se crea como `deadLetterListener` hijo de /system sólo si `log-dead-letters`>0 — derivada del código actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1068-1069, *no afirmada por spec*.
- A2: si loadExtensions falla, el sistema queda terminado y el error se propaga — derivada del código actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1075-1081 (ExtensionSpec:99,153 sin leer).
- A3: la cadena de Watch /user -> /system -> / está establecida antes de cualquier actor de usuario — derivada del código actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:617-618.
- A4: `assertInitialized()` lanza IllegalStateException mientras `_initialized` sea false — ver AC-SYS-023.

**Oráculo.** Parcial: jar oficial permite comparar el árbol de /system tras arrancar (`printTree`), escenario propuesto `sys-start-tree.jsh`; procedencia jars oficiales.

**Decisiones.** Q-X-7, Q-SYS-4

## AC-SYS-003.1 — Jerarquía de guardianes "/", "/user", "/system" y su comportamiento ante mensajes ajenos

*Unidad SYS · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] Los tres guardianes existen; un mensaje cualquiera enviado a ellos se publica como UnhandledMessage (log Warning 'unhandled message from Actor[<deadLetters path>]: <msg>'), no como DeadLetter. / (root), /user y /system son actores; rootGuardian es su propio padre; el padre de rootGuardian es el 'bubble walker'.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:315-326 (Guardian)
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:538-552 (rootGuardian)
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:558-584 (guardian /user)
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:586-599 (systemGuardian /system)
- docs: n/a (el link a supervision.md del inventario original no se verificó)

**Condiciones.**
- Guardian: `Terminated(_) => context.stop(self)`; `StopChild(child) => context.stop(child)` (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:319-322).
- Los guardianes no pierden hijos en reinicio: `preRestart` vacío (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:325,373).
- rootGuardian.getSingleChild: 'temp' -> tempContainer, 'deadLetters' -> deadLetters, otros -> extraNames o hijo real (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:546-551).
- Con guardianProps (AC-SYS-009) /user usa esos Props y su dispatcher (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:562-577).
- Error: root guardian usa estrategia que loguea 'guardian failed, shutting down system' y Stop (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:518-522).

**Orden / concurrencia / ciclo de vida.** Primero root, luego /user (reserveChild + initChild + start), luego /system. Los tres están en `lazy val`: se materializan en provider.init.

**Dependencias.** Filas: AC-SUP-014 ; AC-CELL-004 ; AC-EVT-010. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `RootGuardian`, `UserGuardian`, `SystemGuardian` como actores internos

**Diferencias observables.** Rust: tres actores internos con mailbox sin límite (UnboundedMessageQueueSemantics, actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:317,333). Diferencia: ninguna observable si el path y el warning se conservan.

**Aserciones.**
- A1: `rootGuardian ! "Hello, Mr. Root Guardian"` produce exactamente 1 warning `unhandled message from Actor[<path de deadLetters>]: Hello, Mr. Root Guardian` — actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:70-81.
- A2: ídem para /user — actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:83-94.
- A3: ídem para /system — actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:96-107.
- A4: el hijo de un actor se resuelve por path con nombre URL-encoded `pekko%3A%2F%2FClusterSystem%40127.0.0.1%3A7355` y mantiene `path.name` — actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:49-65.

**Oráculo.** Jar oficial: escenario propuesto `sys-guardians-unhandled.jsh` (envía a /, /user, /system y captura eventos); procedencia jars oficiales.

**Decisiones.** Q-X-3

## AC-SYS-003.2 — Protocolo de hooks de terminación del guardián de sistema (RegisterTerminationHook / TerminationHook / TerminationHookDone)

*Unidad SYS · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual docs] /system registra hooks (sender de RegisterTerminationHook, vigilado con watch); cuando /user termina notifica TerminationHook a todos, espera TerminationHookDone o Terminated de cada uno, detiene los loggers por defecto y se detiene; la raíz para al último. Mecanismo interno por el que remote/cluster (y extensiones) retrasan el apagado de /system. En local sin remote nadie lo usa.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:295-308 (mensajes)
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:331-374 (SystemGuardian)
- Scaladoc/doc pública: n/a
- reference.conf: n/a

**Condiciones.**
- `RegisterTerminationHook` sólo se acepta si `sender() != context.system.deadLetters` (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:351).
- Antes de Terminated(guardian): un hook que muere se elimina del set (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:346-349).
- Terminated(guardian): `context.become(terminating)`, enviar TerminationHook a todos, y comprobar si ya está vacío (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:339-345).
- En `terminating`: `Terminated(a)` o `TerminationHookDone` (de sender()) quitan el hook y, si el set queda vacío, `eventStream.stopDefaultLoggers(system)` y `context.stop(self)` (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:356-370).

**Orden / concurrencia / ciclo de vida.** /system vigila a /user (Watch en init). Los loggers se detienen después de que todos los hooks terminaron, de modo que los hooks aún pueden loguear.

**Dependencias.** Filas: AC-SYS-003.1 ; AC-DW-001 ; AC-EVT-011. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `SystemGuardian` con `HashSet<ActorRef>` de hooks; mensajes internos `RegisterTerminationHook/TerminationHook/TerminationHookDone`

**Diferencias observables.** Rust: mismos mensajes como tipos internos. Diferencia: ninguna observable local; sin remote es código sin consumidor (Q-SYS-4).

**Aserciones.**
- A1: un RegisterTerminationHook cuyo sender es deadLetters se ignora — derivada del código actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:351, *no afirmada por spec*.
- A2: al terminar /user, cada hook registrado recibe TerminationHook — actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:344, *no afirmada por spec*.
- A3: /system no se detiene hasta que cada hook responde TerminationHookDone o termina — actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:357-358,366-370, *no afirmada por spec*.
- A4: stopDefaultLoggers corre inmediatamente antes de context.stop(self) del guardián de sistema — actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:367-369, *no afirmada por spec*.
- A5: un hook que termina antes de Terminated(guardian) se retira del set — actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:346-349, *no afirmada por spec*.

**Oráculo.** Parcial: con jar oficial se puede registrar un actor-hook vía `system.systemActorOf` + `RegisterTerminationHook` (clase private[pekko]; requiere jshell en package pekko.actor). Escenario propuesto `sys-termination-hook.jsh`; procedencia jars oficiales.

**Decisiones.** Q-SYS-4

## AC-SYS-004 — terminate(): idempotente; corre CoordinatedShutdown si run-by-actor-system-terminate, si no finalTerminate (stop de /user en cascada)

*Unidad SYS · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual] Devuelve whenTerminated; múltiples llamadas no fallan; al completar, whenTerminated está completo con Terminated(rootGuardian) con existenceConfirmed=true y addressTerminated=true y es la misma instancia que devuelve terminate(). terminate pone `terminating=true`; si CoordinatedShutdown está habilitado delega en `CoordinatedShutdown.run(ActorSystemTerminateReason)` (que luego llama finalTerminate); si no, `finalTerminate()`.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1089-1104 (terminate)
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1112-1117 (finalTerminate: detiene deadLetterListener si !log-dead-letters-during-shutdown; guardian.stop())
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:448-487 (bubble walker completa terminationPromise)
- reference.conf: pekko.coordinated-shutdown.run-by-actor-system-terminate / terminate-actor-system (reference.conf, sección coordinated-shutdown)

**Condiciones.**
- Combinación run-by-actor-system-terminate=on y terminate-actor-system=off: ConfigurationException en Settings (actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:478-481, ver AC-SYS-022).
- `aborting` evita CoordinatedShutdown (actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1091).
- Idempotente: finalTerminate dice 'these actions are idempotent' (actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1113).

**Orden / concurrencia / ciclo de vida.** `terminating=true` se fija antes de cualquier otra acción (impide nuevas extensiones y rechaza actorOf, AC-SYS-007). El resultado del Future se completa cuando el bubble-walker recibe DeathWatchNotification de /.

**Dependencias.** Filas: AC-CS-004 ; AC-CS-006 ; AC-SYS-005. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `async fn terminate(&self) -> Terminated` y `fn terminate_nowait(&self)`

**Diferencias observables.** Rust: `terminate()` devuelve un future compartido (Q-X-4); `Terminated{actor: root, existence_confirmed: true, address_terminated: true}`.

**Aserciones.**
- A1: `whenTerminated` no está completo antes de terminate y lo está tras `Await` — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:250-256.
- A2: `terminated.actor == provider.rootGuardian`, `addressTerminated==true`, `existenceConfirmed==true` — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:257-259.
- A3: el valor de `whenTerminated` es la misma instancia que devuelve `terminate()` (theSameInstanceAs) — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:260.
- A4: llamar `terminate()` y luego `registerOnTermination` sólo ejecuta el callback una vez o lanza Rejected (exactamente 1 de las dos) — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:272-286.
- A5: terminate() es idempotente (múltiples llamadas no fallan) — derivada de actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1112-1117, *no afirmada por spec*.

**Oráculo.** Jar oficial: escenario propuesto `sys-terminate.jsh` (terminate dos veces y volcado de Terminated). Procedencia: jars oficiales.

**Decisiones.** Q-X-4

## AC-SYS-005 — whenTerminated / getWhenTerminated

*Unidad SYS · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual] Future[Terminated] compartido; se completa después de que corrieron todos los callbacks registrados con registerOnTermination. Await sobre whenTerminated sólo vuelve cuando todos los callbacks terminaron.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1000-1003
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1303-1333 (TerminationCallbacks.terminationFuture = done.future)
- reference.conf: n/a

**Condiciones.**
- El `done` Promise se completa vía cadena: cada add() encadena su ejecución antes del anterior (ver AC-SYS-006).

**Orden / concurrencia / ciclo de vida.** Todos los callbacks terminan antes de que whenTerminated se complete (cada callback corre con `andThen` y completa su promesa siguiente).

**Dependencias.** Filas: AC-SYS-006 ; AC-UTIL-003. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `system.when_terminated() -> impl Future<Output=Terminated> + Clone` (Shared)

**Diferencias observables.** Rust: `Shared<..>` o canal watch. Diferencia: ninguna observable.

**Aserciones.**
- A1: un callback de 50 ms (dilatado) registrado con registerOnTermination ya corrió cuando `Await.ready(whenTerminated)` vuelve — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:234-248.
- A2: `whenTerminated.isCompleted` es true tras terminar y false antes — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:250-256.

**Oráculo.** Jar oficial: escenario propuesto `sys-terminate.jsh` (comparte con AC-SYS-004).

**Decisiones.** Q-X-4

## AC-SYS-006 — registerOnTermination: callbacks en orden inverso al registro (encadenados); tras terminar → RejectedExecutionException

*Unidad SYS · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] Ejecución en orden inverso al registro; registrar después de terminado lanza RejectedExecutionException("ActorSystem already terminated."). Si se registra durante la terminación: o se ejecuta una vez o se rechaza (nunca ambas ni ninguna). El Scaladoc de TerminationCallbacks.add dice 'callbacks are executed in reverse order of insertion'. Implementación: lista enlazada de promesas (CAS sobre AtomicReference).

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1084-1085
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1303-1333 (TerminationCallbacks)
- reference.conf: n/a

**Condiciones.**
- `add` tras completar upstream: `RejectedExecutionException("ActorSystem already terminated.")` (actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1321).
- Los callbacks corren en el ExecutionContext del dispatcher del sistema (actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1000 usa `(dispatcher)`).

**Orden / concurrencia / ciclo de vida.** Callback N+1 corre antes que N (cada nuevo callback se ejecuta cuando completa `p.future` y el anterior espera a ése: `some.completeWith(p.future.andThen{r.run()})`). Serie, no paralelo.

**Dependencias.** Filas: AC-SYS-005. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** Pila de closures `Mutex<Option<Vec<Callback>>>`

**Diferencias observables.** Rust: pila LIFO ejecutada secuencialmente; el hilo/tarea que ejecuta es del runtime (Q-X-4).

**Aserciones.**
- A1: 10 callbacks registrados en orden 1..10 se ejecutan en orden 10..1 — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:212-232.
- A2: tras `system.close()`, `registerOnTermination` lanza RejectedExecutionException con mensaje exactamente `ActorSystem already terminated.` — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:263-270.
- A3: `terminate()` seguido de `registerOnTermination`: el contador (callback o rechazo) vale 1 — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:272-286.
- A4: registrar callbacks antes de terminar no los ejecuta hasta la terminación — derivada de actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1311-1327, *no afirmada por spec*.

**Oráculo.** Jar oficial: escenario propuesto `sys-termination-callbacks.jsh`.

**Decisiones.** Q-X-4

## AC-SYS-007 — Creación de actores durante el apagado se rechaza

*Unidad SYS · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] actorOf externo con el guardián terminando → IllegalStateException("cannot create children while terminating or terminated"); una vez que falla, falla siempre; los actores creados antes de fallar no quedan huérfanos. Mientras /user está en estado terminating (childrenRefs.isTerminating) los attachChild externos lanzan; los hijos creados antes están todos terminados o aún en UnstartedCell.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/dungeon/Children.scala:313-314 (throw)
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:927-937

**Condiciones.**
- Error: IllegalStateException con mensaje exacto `cannot create children while terminating or terminated` (Children.scala:314).
- Una vez que `failing==true` crear no puede volver a tener éxito (actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:309).

**Orden / concurrencia / ciclo de vida.** Condición de carrera intencionada: `terminate()` desde un scheduler a 100 ms mientras otro hilo crea actores sin parar; el invariante es monotonía (éxitos, luego fallos).

**Dependencias.** Filas: AC-CELL-015 ; AC-SYS-004. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `Err(SpawnError::SystemTerminating)` desde `actor_of`

**Diferencias observables.** Rust: el estado 'terminating' del contenedor de hijos del guardián debe ser atómico con respecto a reserveChild. Diferencia: error tipado.

**Aserciones.**
- A1: una vez que actorOf lanza IllegalStateException no vuelve a tener éxito (`failing should not be true`) — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:309.
- A2: tras terminar el sistema, ningún actor creado queda no terminado salvo UnstartedCell — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:323-325.
- A3: el sistema termina en < 10 s de uptime bajo creación continua — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:316-320.

**Oráculo.** Jar oficial: escenario propuesto `sys-create-during-terminate.jsh` (no determinista en tiempo; comparar sólo las invariantes A1-A2).

**Decisiones.** Q-X-4, Q-X-5

## AC-SYS-008 — Fallo o escalamiento de /user apaga el sistema; guardian-supervisor-strategy configurable

*Unidad SYS · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] Kill sobre /user (ActorKilledException) apaga el sistema; con guardian-supervisor-strategy=StoppingSupervisorStrategy un hijo de /user que falla se detiene (Terminated con existenceConfirmed=true, addressTerminated=false); con una estrategia que escala, el fallo apaga el sistema. El root guardian usa una estrategia que loguea 'guardian failed, shutting down system' y hace Stop de /user. /user usa la estrategia configurada (por defecto DefaultSupervisorStrategy).

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:510-527
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:422 (SupervisorStrategyClass)
- reference.conf:123 (pekko.actor.guardian-supervisor-strategy)

**Condiciones.**
- Kill a /user -> EventFilter[ActorKilledException] observa el error y whenTerminated se completa (actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:330-333).
- Estrategia que lanza `Exception("hello")` en un hijo con StoppingSupervisorStrategy: el hijo muere, el sistema sigue (actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:343-356).
- Estrategia `Escalate` -> el sistema termina (actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:359-375).

**Orden / concurrencia / ciclo de vida.** Fallo de /user -> reportado al root guardian -> `SupervisorStrategy.Stop` sobre /user -> /system (que vigila a /user) ejecuta hooks -> raíz para.

**Dependencias.** Filas: AC-SUP-014 ; AC-SYS-003.1 ; AC-CFG-005. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `GuardianSupervisorStrategy` cargado por nombre (Q-X-2)

**Diferencias observables.** Rust: estrategia por tipo registrado. Diferencia: FQCN de config pasa a clave de registro (Q-X-2).

**Aserciones.**
- A1: `system.actorSelection("/user") ! Kill` genera un ActorKilledException y el sistema termina — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:328-334.
- A2: con StoppingSupervisorStrategy, un `Exception("hello")` en un hijo de /user: 1 evento de error, y el probe recibe `Terminated(a)(existenceConfirmed=true, addressTerminated=false)` — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:336-357.
- A3: con una estrategia que escala, la excepción de un hijo provoca el apagado del sistema — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:359-375.

**Oráculo.** Jar oficial: escenario propuesto `sys-guardian-strategy.jsh`.

**Decisiones.** Q-X-2, Q-X-5

## AC-SYS-009 — Guardián de usuario custom (guardianProps; lo usa actor-typed)

*Unidad SYS · visibilidad internal-required · analysis `extracted`*

**Contrato observable.** [contractual por spec] Con guardianProps, actorOf top-level desde afuera lanza UnsupportedOperationException (con o sin nombre); /user usa el dispatcher indicado por los Props (default-dispatcher si SameAsParent). Con guardian custom el sistema no permite crear actores desde afuera: sólo el guardián (typed) puede crearlos.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:927-937
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:562-577

**Condiciones.**
- UnsupportedOperationException con nombre: `cannot create top-level actor [<name>] from the outside on ActorSystem with custom user guardian` (actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:930-931); sin nombre: `cannot create top-level actor from the outside on ActorSystem with custom user guardian` (actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:936-937).
- Dispatcher: `Deploy.DispatcherSameAsParent` -> DefaultDispatcherId; si no, `props.dispatcher` (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:566-568).

**Orden / concurrencia / ciclo de vida.** n/a: construcción.

**Dependencias.** Filas: AC-SYS-003.1 ; AC-DSP-006. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** parámetro interno `guardian_props: Option<Props>` del builder

**Diferencias observables.** Rust: `actor_of` devuelve `Err(Unsupported)`. El guardián custom lo usa actor-typed (Q-X-8).

**Aserciones.**
- A1: `sys.actorOf(Props.empty)` y `sys.actorOf(Props.empty, "empty")` lanzan UnsupportedOperationException — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:386-392.
- A2: el /user con guardianProps usa el dispatcher indicado — derivada de actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:562-577 (ActorSystemDispatcherSpec:156 sin leer).

**Oráculo.** Jar oficial: se puede reproducir con `new ActorSystemImpl(..., Some(Props.empty), ...)` desde jshell en el paquete pekko.actor. Escenario propuesto `sys-custom-guardian.jsh`.

**Decisiones.** Q-X-8, Q-X-5

## AC-SYS-010 — systemActorOf (actores bajo /system)

*Unidad SYS · visibilidad public · analysis `listed`*

**Contrato observable.** [contractual] Crea hijos del guardián de sistema con systemService=true; lo usan extensiones, loggers, deadLetterListener, unsubscribers y el testActor del TestKit. Los actores de sistema cuelgan de /system y se detienen después de /user.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:779,924-925
- reference.conf: n/a

**Condiciones.**
- Usa `systemGuardian.underlying.attachChild(props, name, systemService = true)` (actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:924-925).
- El TestKit crea `testActor-N`/`testProbe-N` con este método (TestKit.scala:189-199).

**Orden / concurrencia / ciclo de vida.** Se puede llamar durante start (loggers, deadLetterListener).

**Dependencias.** Filas: AC-SYS-003.1. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `system.system_actor_of(props, name) -> Result<ActorRef>`

**Diferencias observables.** Rust: método público (en Pekko es ExtendedActorSystem) marcado `#[doc(hidden)]` o trait de extensión.

**Aserciones.**
- A1: el path de un actor creado con systemActorOf es /system/<name> — derivada de actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:924-925, *no afirmada por spec leída*.

**Oráculo.** Jar oficial: escenario propuesto `sys-system-actor-of.jsh`.

**Decisiones.** ninguna específica

## AC-SYS-011 — abort(): apagado forzado sin CoordinatedShutdown

*Unidad SYS · visibilidad internal-required · analysis `listed`*

**Contrato observable.** [contractual Scaladoc] Marca aborting y llama terminate(), que entonces salta CoordinatedShutdown; no espera hijos remotos. aborting=true y terminate(): como CoordinatedShutdown se omite (`!aborting`), se ejecuta finalTerminate directo.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1123-1134
- Scaladoc: actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1124-1130

**Condiciones.**
- abort -> `aborting=true; terminate()` (actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1131-1134).

**Orden / concurrencia / ciclo de vida.** Ver AC-SYS-004.

**Dependencias.** Filas: AC-SYS-004. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `fn abort(&self)`

**Diferencias observables.** Rust: igual.

**Aserciones.**
- A1: tras abort(), CoordinatedShutdown no corre y el sistema termina — derivada de actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1091,1132-1135, *no afirmada por spec*.

**Oráculo.** Jar oficial: escenario propuesto `sys-abort.jsh` (registrar tarea de CoordinatedShutdown y comprobar que no corre).

**Decisiones.** ninguna específica

## AC-SYS-012 — Accesores: name, settings, startTime, uptime (s), eventStream, log, deadLetters, scheduler, dispatchers, mailboxes, dispatcher; toString = address raíz

*Unidad SYS · visibilidad public · analysis `listed`*

**Contrato observable.** [contractual] uptime en segundos desde startTime (ms epoch); toString = `lookupRoot.path.root.address.toString`. Handle clonable con acceso a servicios del sistema.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:530-650 (sin leer en detalle)
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1262 (toString)

**Condiciones.**
- toString del sistema = dirección raíz (p. ej. `pekko://name`).

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** Filas: AC-SYS-022. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** métodos del handle `ActorSystem` (Clone, Arc interno)

**Diferencias observables.** Rust: handle `Arc`. toString -> `Display` de la dirección raíz.

**Aserciones.**
- A1: `system.settings.toString` no contiene el nombre de usuario ni el home y contiene `<username>` — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:395-411 (ver AC-SYS-022).

**Oráculo.** Jar oficial para toString y uptime; escenario propuesto `sys-accessors.jsh`.

**Decisiones.** ninguna específica

## AC-SYS-013 — ExtendedActorSystem: provider, guardian, systemGuardian, systemActorOf, threadFactory, dynamicAccess, printTree

*Unidad SYS · visibilidad public · analysis `listed`*

**Contrato observable.** [contractual] API para extensiones e integraciones. printTree imprime el árbol con formato `-> name SimpleName ActorClass status=.. N children` (depuración). Superficie para extensiones; printTree se imprime cuando falla un apagado.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:755-806
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1264-1300 (printTree)

**Condiciones.**
- printTree formato: `-> <name> <simpleName(ref)> <actor class|null> status=<mailbox status> <Terminating(..)|Terminated|EmptyChildren|N children>` con hijos ordenados e indentados.

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** Filas: AC-SYS-018. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** trait o métodos `pub` marcados unstable

**Diferencias observables.** Rust: `print_tree()` con formato propio; la comparación textual con Pekko no es necesaria salvo que un test la afirme (ninguno lo hace).

**Aserciones.**
- A1: `printTree` no falla sobre un sistema vivo — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:318 (sólo usado en mensaje de diagnóstico).

**Oráculo.** No aplica: el texto de printTree depende de clases JVM (getClass).

**Decisiones.** ninguna específica

## AC-SYS-014 — close() (AutoCloseable): terminate + espera hasta close-actor-system-timeout

*Unidad SYS · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual] Bloquea hasta whenTerminated o vence pekko.coordinated-shutdown.close-actor-system-timeout (Await.result lanza TimeoutException al vencer). `Using(ActorSystem()){..}` termina el sistema al salir del bloque; close() deja whenTerminated completo.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1106-1110
- reference.conf:1257 (close-actor-system-timeout)

**Condiciones.**
- Timeout vencido: Await.result lanza java.util.concurrent.TimeoutException (derivado de scala.concurrent.Await).

**Orden / concurrencia / ciclo de vida.** terminate() primero, luego bloqueo en whenTerminated.

**Dependencias.** Filas: AC-SYS-004 ; AC-CS-009. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** método explícito `close_blocking(&self)`; sin `Drop` que termine (Q-SYS-1)

**Diferencias observables.** Rust: `close_blocking` o `async fn close`. Diferencia: AutoCloseable no existe; el equivalente idiomático es una guarda `SystemGuard` opcional (Q-SYS-1).

**Aserciones.**
- A1: tras `close()`, registerOnTermination lanza RejectedExecutionException — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:263-270.
- A2: `Using(ActorSystem()) { s => .. }` deja `whenTerminated.isCompleted==true` — actor-tests/src/test/scala-2.13+/org/apache/pekko/actor/ActorSystemScala213PlusSpec.scala:29-35.

**Oráculo.** Jar oficial: `sys-terminate.jsh` (close en vez de terminate).

**Decisiones.** Q-SYS-1

## AC-SYS-015 — Errores fatales no capturados: jvm-exit-on-fatal-error → System.exit(-1); si no, terminate()

*Unidad SYS · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual JVM] El UncaughtExceptionHandler de los hilos del sistema: errores NonFatal/InterruptedException/NotImplementedError/ControlThrowable se loguean ('Uncaught error from thread [{}]'); cualquier otro es fatal: imprime a stderr, loguea con marker SECURITY y o System.exit(-1) (si jvm-exit-on-fatal-error) o terminate(). Un hilo del sistema que muere con error fatal apaga el proceso (o el sistema).

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:858-906
- reference.conf:90 (jvm-exit-on-fatal-error = on)

**Condiciones.**
- No fatal: `log.error(cause, "Uncaught error from thread [{}]", thread.getName)` (actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:862).
- Fatal: stderr `Uncaught error from thread [<t>]: <msg>, <message> ActorSystem[<name>]` + stacktrace, luego log con LogMarker.Security (actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:883-906); `message` = `shutting down JVM since 'pekko.jvm-exit-on-fatal-error' is enabled for` o `shutting down`.
- Si IncompatibleClassChangeError con mensaje que empieza con org.apache.pekko: aviso a stderr sobre versiones mezcladas (actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:865-873).

**Orden / concurrencia / ciclo de vida.** Corre en el hilo que murió.

**Dependencias.** Filas: AC-CFG-007 ; AC-EVT-015. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** hook de panic en hilos propios + política configurable

**Diferencias observables.** Rust: `panic` en tarea de actor ya es 'fallo' (Q-X-5); 'fatal' = abort/OOM no capturable; propuesta: política `FatalErrorPolicy::{ExitProcess, TerminateSystem}` aplicada a panics fuera de actores y a un handler de señal; diferencia observable: sin IncompatibleClassChangeError.

**Aserciones.**
- A1: un error NonFatal en un hilo del sistema sólo se loguea — derivada de actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:862-863, *no afirmada por spec*.
- A2: con jvm-exit-on-fatal-error=on el proceso sale con código -1 — derivada de actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:877-879, *no afirmada por spec*.
- A3: con jvm-exit-on-fatal-error=off se llama terminate() — derivada de actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:880-882, *no afirmada por spec*.

**Oráculo.** Parcial: jar oficial con un hilo del threadFactory del sistema que lanza OutOfMemoryError simulado y `jvm-exit-on-fatal-error=off`; escenario propuesto `sys-fatal-error.jsh`.

**Decisiones.** Q-SYS-2, Q-X-5

## AC-SYS-016 — LocalActorRefProvider.actorOf: merge de Deploy, ruta router vs no-router, lookup de dispatcher y mailbox, LocalActorRef vs RepointableActorRef vs RoutedActorRef

*Unidad SYS · visibilidad internal-required · analysis `listed`*

**Contrato observable.** [contractual por spec] Errores de config de dispatcher/mailbox se convierten en ConfigurationException en la creación; dos creadores concurrentes del mismo nombre: exactamente uno gana (el otro InvalidActorNameException). Resuelve Props+Deploy (config `pekko.actor.deployment`) a un ref concreto.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:659-757
- reference.conf: pekko.actor.deployment (reference.conf sección deployment)

**Condiciones.**
- Dispatcher inexistente: ConfigurationException(`Dispatcher [<id>] not configured for path <path>`) (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:704-705).
- Cualquier NonFatal al crear dispatcher/mailbox: ConfigurationException(`configuration problem while creating [<path>] with dispatcher [<d>] and mailbox [<m>]`, cause) (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:714-718).
- Router: `Dispatcher [..] not configured for routees of <path>` / `for router of <path>` (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:727-730).

**Orden / concurrencia / ciclo de vida.** Deploy de código vs config: dispatcher/mailbox definidos en el deploy pisan los de Props (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:684-702). DispatcherSameAsParent usa el dispatcher del padre.

**Dependencias.** Filas: AC-DSP-007 ; AC-MBX-013 ; AC-RT-003 ; AC-PATH-005. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** fábrica en `provider::local::spawn`

**Diferencias observables.** Rust: misma lógica en `spawn`. Diferencia: errores como `Err(ConfigError)`.

**Aserciones.**
- A1: 4 creaciones concurrentes de `new-actorN` con el mismo nombre -> conjunto de resultados {éxito, InvalidActorNameException} (1 y 2) — actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:134-154.
- A2: dos `actorOf(.., "duplicate")` en el mismo mensaje -> 1 InvalidActorNameException — actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:156-166.
- A3: nombres mal formados: null->'null', ''->'empty', '$hallo'->'not start with `$`', 'a%'/'%3'/'%xx'/.. ->'Invalid actor path element', 'üß'->'include only ASCII', `he"llo` -> `["] at position: 2` — actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:168-198.
- A4: ConfigurationException ante dispatcher/mailbox inválido — SupervisorSpec:510-540 (sin leer), derivada de actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:704-718.
- A5: `debug.router-misconfiguration` avisa de config router vs código — derivada de actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:670-676, *no afirmada por spec*.

**Oráculo.** Jar oficial para A3 (mensajes de nombres) y A4: escenario propuesto `sys-actorof-errors.jsh`.

**Decisiones.** Q-X-2, Q-X-5

## AC-SYS-017 — Actores temporales: tempPath, tempContainer, registerTempActor/unregisterTempActor

*Unidad SYS · visibilidad internal-required · analysis `listed`*

**Contrato observable.** [contractual interno] Refs bajo /temp registrados en un VirtualPathContainer, resolubles por path mientras viven; nombres `$` + base64 del contador. Cada ask crea un ref temporal bajo /temp.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:428-446 (tempPath)
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:601-611 (register/unregister)

**Condiciones.**
- `tempPath(prefix)` = rootPath/temp/<prefix>$<base64(contador)> (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:434-442).
- registerTempActor exige que el padre sea tempNode (assert) (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:604).

**Orden / concurrencia / ciclo de vida.** Contador atómico creciente.

**Dependencias.** Filas: AC-ASK-002 ; AC-UTIL-009. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** mapa concurrente nombre→ref con contador atómico

**Diferencias observables.** Rust: igual.

**Aserciones.**
- A1: `tempPath()` genera nombres únicos `$..` bajo /temp — derivada de actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:432-442, *no afirmada por spec leída*.
- A2: un ref temporal registrado es resoluble por path — derivada de actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:601-611.

**Oráculo.** Jar oficial: escenario propuesto `sys-temp-paths.jsh` (secuencia de tempPath).

**Decisiones.** ninguna específica

## AC-SYS-018 — SPI ActorRefProvider (@DoNotInherit): rootGuardian, guardian, systemGuardian, deadLetters, ignoreRef, rootPath, deployer, resolveActorRef, terminationFuture, getExternalAddressFor, getDefaultAddress, serializationInformation, addressString

*Unidad SYS · visibilidad internal-required · analysis `listed`*

**Contrato observable.** [contractual para remote/cluster] Punto de enganche de RemoteActorRefProvider/ClusterActorRefProvider; en local getExternalAddressFor(addr) = Some(addr) sólo si coincide con rootPath.address. Interfaz del proveedor.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:40-187
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:759 (getExternalAddressFor local)

**Condiciones.**
- rootGuardianAt(address) = rootGuardian si address==rootPath.address, si no deadLetters (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:554-556).

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** Filas: AC-CFG-008. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** trait sellado `ActorRefProvider` con impl local

**Diferencias observables.** Rust: trait sellado.

**Aserciones.**
- A1: `getExternalAddressFor(addr)` local devuelve Some(addr) sólo para la dirección raíz — actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:759, *no afirmada por spec leída*.

**Oráculo.** Jar oficial para A1.

**Decisiones.** ninguna específica

## AC-SYS-019 — terminationFuture del proveedor y 'bubble-walker' (theOneWhoWalksTheBubblesOfSpaceTime)

*Unidad SYS · visibilidad internal-required · analysis `extracted`*

**Contrato observable.** [contractual interno] El padre del guardián raíz es un ref mínimo en /bubble-walker: recibe Failed de la raíz (loguea 'guardian <child> failed, shutting down!' y la detiene), ignora Supervise, y ante DeathWatchNotification se detiene y completa terminationFuture con Terminated(rootGuardian)(true,true). Es el 'último' ref: su stop() marca la terminación.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:421-423
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:448-487

**Condiciones.**
- Mensaje normal mientras camina: `log.error(s"$this received unexpected message [$message]")`; null -> InvalidMessageException("Message is null") (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:469-474).
- SystemMessage inesperado: error `$this received unexpected system message [$message]` (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:484).

**Orden / concurrencia / ciclo de vida.** stop() idempotente (trySuccess).

**Dependencias.** Filas: AC-SYS-005. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** oneshot interno + ref mínimo

**Diferencias observables.** Rust: igual.

**Aserciones.**
- A1: Terminated devuelto por terminate() tiene actor=rootGuardian y ambos flags true — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:257-259.
- A2: Failed del guardián raíz -> error 'guardian .. failed, shutting down!' y stop del hijo — derivada de actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:478-481, *no afirmada por spec*.
- A3: stop() es idempotente — derivada de actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:457-461.

**Oráculo.** Jar oficial: `sys-terminate.jsh`.

**Decisiones.** ninguna específica

## AC-SYS-020 — ManifestInfo: chequeo de versiones mezcladas de módulos Pekko

*Unidad SYS · visibilidad internal-accidental · analysis `listed`*

**Contrato observable.** [contractual JVM] Lee MANIFEST de los jars y advierte/falla con fail-mixed-versions. Avisa si hay versiones distintas de módulos Pekko en el classpath.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1013-1045,1069

**Condiciones.**
- fail-mixed-versions=on (reference.conf:103) hace fallar el arranque.

**Orden / concurrencia / ciclo de vida.** Se llama en start tras startUnsubscriber (actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1069).

**Dependencias.** Filas: AC-CFG-007. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** verificación en tiempo de compilación (cargo resuelve una versión por crate) + `Version` en runtime

**Diferencias observables.** Rust: comprobación de que `rakka-actor` y `rakka-actor-typed` comparten versión (const en compilación); diferencia observable: no hay warning de runtime.

**Aserciones.**
- Pendiente de lectura.

**Oráculo.** No aplica: depende de manifests JAR.

**Decisiones.** Q-SYS-3

## AC-SYS-021 — Main (arranca un sistema con un actor y termina al morir)

*Unidad SYS · visibilidad deprecated · analysis `listed`*

**Contrato observable.** [contractual] Utilidad deprecada. Pendiente de lectura.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/Main.scala:32-60

**Condiciones.**
- Pendiente.

**Orden / concurrencia / ciclo de vida.** Pendiente.

**Dependencias.** Filas: AC-SYS-001.2. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `rakka::Main::run::<A>()` (feature `deprecated`)

**Diferencias observables.** Rust: función utilitaria detrás de feature; sin diferencias conocidas.

**Aserciones.**
- Pendiente de lectura.

**Oráculo.** Jar oficial: `java org.apache.pekko.Main <clase>`.

**Decisiones.** Q-SYS-3

## AC-SYS-022 — Settings.toString con redacción de datos sensibles (renderWithRedactions)

*Unidad SYS · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] settings.toString renderiza el Config sin valores de variables de entorno y reemplazando el nombre de usuario del proceso por `<username>`; no aparece el home del usuario. El Config se renderiza con setShowEnvVariableValues(false) y se reemplaza `user.name` por `<username>`.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:488-491
- actor/src/main/scala/org/apache/pekko/util/Helpers.scala:180-192

**Condiciones.**
- El reemplazo es por subcadena (String.replace) del valor de la propiedad `user.name` (Helpers.scala:191): si el nombre de usuario es una subcadena común, se reemplaza en todo el texto.

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** Filas: AC-CFG-001. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `Settings::to_redacted_string()` y `Display`

**Diferencias observables.** Rust: usar el usuario del proceso (`USER`/`getuid`) y no mostrar valores de variables de entorno. Diferencia: el 'home' del test se pasa como propiedad Java `${user.home}` en HOCON; en Rust el equivalente de sustitución es `${HOME}` (Q-X-1).

**Aserciones.**
- A1: `settings.toString` no contiene `user.name` ni `user.home` — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:404-408.
- A2: `settings.toString` contiene `<username>` — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:409.

**Oráculo.** Jar oficial: escenario propuesto `sys-settings-redaction.jsh`.

**Decisiones.** Q-SYS-5, Q-X-1

## AC-SYS-023 — assertInitialized: guardia contra uso del sistema durante la inicialización

*Unidad SYS · visibilidad internal-required · analysis `extracted`*

**Contrato observable.** [contractual interno] Lanza IllegalStateException con texto fijo mientras el sistema no completó provider.init. Las extensiones que requieren sistema inicializado llaman a esta guardia.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1050-1057

**Condiciones.**
- IllegalStateException("The calling code expected that the ActorSystem was initialized but it wasn't yet. This is probably a bug in the ActorSystem initialization sequence often related to initialization of extensions. Please report at https://github.com/apache/pekko/issues.") (actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1055-1057; texto parcial, verificar contra el archivo al implementar).

**Orden / concurrencia / ciclo de vida.** _initialized se fija tras provider.init (actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1066).

**Dependencias.** Filas: AC-SYS-002 ; AC-EXT-003. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `system.assert_initialized() -> Result<(), InitializationError>`

**Diferencias observables.** Rust: `Err`. Diferencia: tipo de error.

**Aserciones.**
- A1: antes de provider.init, assertInitialized falla — derivada de actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1055-1057, *no afirmada por spec*.

**Oráculo.** No aplica: sólo alcanzable con extensiones que llaman en construcción.

**Decisiones.** Q-X-5

## AC-SYS-024 — LocalActorRefProvider.resolveActorRef (String y ActorPath): paths ajenos o inexistentes → deadLetters / EmptyLocalActorRef

*Unidad SYS · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual interno] Path con dirección distinta de la raíz o path vacío -> deadLetters (con log Debug `Resolve (deserialization) ..`); path no existente -> EmptyLocalActorRef bajo el mismo path. Resuelve refs por path para deserialización, ActorSelection y envío por path.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:622-657

**Condiciones.**
- String no parseable o dirección ajena: `deadLetters` + Debug `Resolve (deserialization) of unknown (invalid) path [{}], using deadLetters.` (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:624-626).
- ActorPath de otro root: Debug `Resolve (deserialization) of foreign path [{}] doesn't match root path [{}], using deadLetters.` (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:631-636).
- Elementos vacíos: deadLetters (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:644-646). No encontrado: `EmptyLocalActorRef(system.provider, ref.path / pathElements, eventStream)` (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:655).

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** Filas: AC-PATH-007 ; AC-REF-012. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `provider.resolve_actor_ref(path) -> ActorRef`

**Diferencias observables.** Rust: igual; `ActorSelection` (AC-REF) reutiliza este método.

**Aserciones.**
- A1: un hijo con nombre `pekko%3A%2F%2FClusterSystem%40127.0.0.1%3A7355` se resuelve y conserva el nombre — actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:49-65.
- A2: path de otro sistema devuelve deadLetters — derivada de actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:629-637, *no afirmada por spec*.
- A3: path inexistente devuelve un ref terminado (EmptyLocalActorRef) con el mismo path — derivada de actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:648-656.

**Oráculo.** Jar oficial: escenario propuesto `sys-resolve-actor-ref.jsh`.

**Decisiones.** ninguna específica

## AC-SYS-025 — Hijos especiales del guardián raíz: temp, deadLetters y extraNames (registerExtraNames)

*Unidad SYS · visibilidad internal-required · analysis `extracted`*

**Contrato observable.** [contractual interno] rootGuardian.getSingleChild: 'temp' -> tempContainer, 'deadLetters' -> deadLetters, otros nombres registrados con registerExtraNames, resto -> hijo real. Permite que /deadLetters y /temp sean resolubles por path como hijos del root.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:499-508,546-551

**Condiciones.**
- getParent del root = root (actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:546).

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** Filas: AC-SYS-017 ; AC-REF-010. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `RootGuardian::child(name)` con tabla de nombres especiales

**Diferencias observables.** Rust: igual.

**Aserciones.**
- A1: `system.actorSelection("/deadLetters")` y `/temp` resuelven a esos refs — derivada de actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:547-550, *no afirmada por spec leída*.

**Oráculo.** Jar oficial: `sys-resolve-actor-ref.jsh`.

**Decisiones.** ninguna específica

## AC-EVT-001 — Contrato EventBus: subscribe/unsubscribe/publish con resultado booleano

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] subscribe devuelve false si ya estaba suscrito al canal; unsubscribe false si no lo estaba; publish entrega a todos los suscriptores del clasificador y a ningún otro; publish sin suscriptores no falla. Bus genérico con tres tipos asociados; las implementaciones concretas (Lookup/Subchannel/Scanning/ManagedActor) comparten estas garantías.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/EventBus.scala:33-90
- docs: sección event-bus (docs/src/main/paradox/event-bus.md; no se leyó)

**Condiciones.**
- subscribe(sub, c) -> true la primera vez, false si ya estaba (actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:76-80).
- unsubscribe(sub, c) -> true si estaba, false si no (actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:66-74,82-86).
- publish(e) sin suscriptores: sin efecto ni error (actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:100-102).

**Orden / concurrencia / ciclo de vida.** Cada suscriptor recibe un evento por publish (sin duplicados); no hay garantía de orden entre suscriptores.

**Dependencias.** Filas: -. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** trait `EventBus { type Event; type Classifier; type Subscriber; }`

**Diferencias observables.** Rust: trait con tipos asociados; Subscriber es `ActorRef` o closure. Diferencia: ninguna observable.

**Aserciones.**
- A1: subscribe -> true — actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:62-64.
- A2: unsubscribe de suscriptor existente -> true — actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:66-68.
- A3: unsubscribe de un suscriptor inexistente -> false — actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:70-74.
- A4: subscribe dos veces: true luego false; unsubscribe luego true — actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:76-80.
- A5: unsubscribe dos veces: true luego false — actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:82-86.
- A6: 10 suscriptores con clasificadores distintos: todos subscribe=true y unsubscribe=true — actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:88-98.
- A7: publish con un único suscriptor entrega 1 vez (y varias veces si se publica varias) — actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:104-122.
- A8: publish a 10 suscriptores del mismo clasificador entrega a los 10 — actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:124-137.
- A9: no se entrega a suscriptores de otro clasificador ni a ex-suscriptores — actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:139-156.

**Oráculo.** Jar oficial: escenario propuesto `evt-eventbus-contract.jsh` (clase LookupClassification mínima; sólo Scala-from-jshell es incómodo: usar `EventStream` con clases como clasificador).

**Decisiones.** ninguna específica

## AC-EVT-002 — LookupClassification

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual] Clasificación por igualdad de clasificador (Index<Classifier, Subscriber> con comparador de suscriptores). subscribe(sub, c) = subscribers.put(c, sub); publish(e) itera los valores del clasificador classify(e).

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/EventBus.scala:93-130

**Condiciones.**
- compareSubscribers define la identidad de suscriptor (orden total); mapSize es sólo una pista de capacidad (EventBus.scala:95-103).

**Orden / concurrencia / ciclo de vida.** Itera el índice sin bloqueo global (Index concurrente, AC-UTIL-009).

**Dependencias.** Filas: AC-EVT-001 ; AC-UTIL-009. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `HashMap<Classifier, BTreeSet<Subscriber>>`

**Diferencias observables.** Rust: `RwLock<HashMap<..>>` o estructura concurrente; sin diferencia observable.

**Aserciones.**
- A1: los tests abstractos de EventBus pasan con classify = evento.toString — actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:353-369.
- A2: unsubscribe(subscriber) sin clasificador quita todas las suscripciones — derivada de actor/src/main/scala/org/apache/pekko/event/EventBus.scala:124, *no afirmada por spec*.
- A3: publish entrega en el hilo del publicador (publish(event, subscriber) síncrono) — derivada de actor/src/main/scala/org/apache/pekko/event/EventBus.scala:126-129.

**Oráculo.** Jar oficial: ver AC-EVT-001.

**Decisiones.** ninguna específica

## AC-EVT-003 — SubchannelClassification (jerarquía de clasificadores)

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] Suscribirse a un supertipo recibe eventos de subtipos (clase o interfaz); correcto al agregar/quitar suscripciones en cualquier orden; resultado booleano de subscribe/unsubscribe por (suscriptor, canal) exacto. Cache Map[Classifier, Set[Subscriber]] volátil reconstruido bajo lock (subscriptions.synchronized) y consultado sin lock en publish.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/EventBus.scala:136-211

**Condiciones.**
- subscribe -> diff.nonEmpty (true si cambió la visibilidad efectiva) (EventBus.scala:159-163).
- publish con clasificador desconocido: lo añade a la caché bajo lock (EventBus.scala:177-190).
- hasSubscriptions(sub) es costoso: recorre la caché (EventBus.scala:196-199).

**Orden / concurrencia / ciclo de vida.** Escrituras serializadas por lock; lecturas lock-free sobre caché inmutable.

**Dependencias.** Filas: AC-UTIL-008 ; AC-EVT-001. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** índice de subclasificación sobre `EventClass` (ver Q-X-6)

**Diferencias observables.** Rust: `ArcSwap<HashMap<..>>` + Mutex para escritura. Diferencia: la relación de subtipo debe proveerse (Q-X-6).

**Aserciones.**
- A1: subscribe(B3) true; publish(C) sin entrega, publish(B3) entrega; subscribe(A) true; C y B2 entregan; unsubscribe(B2) true; luego C y B3 sin duplicar, A entrega — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:150-174.
- A2: clases e interfaces (AT, BT, CC, CCATBT): suscripción a interfaz recibe subtipos — update on subscribe actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:176-197.
- A3: update on unsubscribe (a3 deja de recibir, otros no) — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:199-219.
- A4: unsubscribe(sub) de todos los canales — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:221-241.
- A5: update on publish (la caché se completa al publicar) — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:243-257.
- A6: unsubscribe de una clase usada con trait: a2 pierde CC pero conserva BT — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:259-279.
- A7: subscribe después de publish añade el canal sin perder los anteriores — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:281-297.

**Oráculo.** Jar oficial: escenario propuesto `evt-subchannel.jsh` usando `EventStream` con clases Java/Scala de prueba compiladas en jshell.

**Decisiones.** Q-X-6

## AC-EVT-004 — ScanningClassification

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual] Recorre todos los pares (clasificador, suscriptor) evaluando `matches(classifier, event)`; orden total por compareClassifiers/compareSubscribers. Un mismo suscriptor con dos clasificadores que coinciden recibe el evento dos veces (itera pares).

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/EventBus.scala:219-269

**Condiciones.**
- unsubscribe(subscriber) elimina todos los pares con compareSubscribers==0 (EventBus.scala:253-259).

**Orden / concurrencia / ciclo de vida.** Iteración en orden total de los pares.

**Dependencias.** Filas: AC-EVT-001. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `Vec`/`BTreeSet<(Classifier, Subscriber)>` + predicado

**Diferencias observables.** Rust: `BTreeSet` bajo lock o skiplist concurrente.

**Aserciones.**
- A1: los tests abstractos de EventBus pasan con matches(c,e)=e.toString==c — actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:317-333.
- A2: un suscriptor con dos clasificadores coincidentes recibe dos veces — derivada de actor/src/main/scala/org/apache/pekko/event/EventBus.scala:261-268, *no afirmada por spec*.

**Oráculo.** Jar oficial: ver AC-EVT-001.

**Decisiones.** ninguna específica

## AC-EVT-005 — ManagedActorClassification (desuscripción automática al morir el suscriptor) y ActorClassificationUnsubscriber

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] El bus vigila (watch) a los suscriptores mediante un unsubscriber y los quita al recibir Terminated; la muerte del actor clasificador (monitored) no quita suscripciones; un suscriptor sólo se des-registra del unsubscriber cuando deja su última suscripción. seqNr secuencia las altas/bajas hacia el unsubscriber (Register/Unregister con seq == atSeq+1; si no, stash) para que watch/unwatch lleguen en el mismo orden que las mutaciones del mapa.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/EventBus.scala:277-436
- actor/src/main/scala/org/apache/pekko/event/ActorClassificationUnsubscriber.scala:28-104
- reference.conf:752 (pekko.actor.debug.event-stream)

**Condiciones.**
- subscribe/unsubscribe con suscriptor o clasificador null: IllegalArgumentException("Subscriber is null") / ("Classifier is null") (EventBus.scala:407-415).
- Texto Debug: `actor <a> has terminated, unsubscribing it from <bus>` (actor/src/main/scala/org/apache/pekko/event/ActorClassificationUnsubscriber.scala:68), `unregistered watch of <a> in <bus>`, `registered watch for <a> in <bus>`, `will monitor <bus>`.

**Orden / concurrencia / ciclo de vida.** Terminated(actor) -> unsubscribe(actor) -> otro Unregister que se procesa inofensivamente (comentario actor/src/main/scala/org/apache/pekko/event/ActorClassificationUnsubscriber.scala:69-71).

**Dependencias.** Filas: AC-EVT-001 ; AC-DW-001 ; AC-SYS-010. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** actor unsubscriber interno + mapa `ActorRef -> BTreeSet<ActorRef>`

**Diferencias observables.** Rust: ídem, usando el watch de AC-DW. Diferencia: `this.toString` del bus en el texto Debug depende de Display del bus.

**Aserciones.**
- A1: al morir el suscriptor, deja de recibir y el unsubscriber lo publica en Debug `actor <subs> has terminated, unsubscribing it from <bus>` — actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:202-223,283-289.
- A2: la muerte del actor clasificador (a1) no quita la suscripción; sigue entregando m(2) — actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:225-247.
- A3: tras unsubscribe de a1 sigue recibiendo de a2; sólo al quitar a2 se publica `unregistered watch of <subs> in <bus>` — actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:249-281,291-297.
- A4: los tests abstractos de EventBus pasan con ActorEventBus — actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:180-198.
- A5: null en subscribe/unsubscribe lanza IllegalArgumentException — derivada de actor/src/main/scala/org/apache/pekko/event/EventBus.scala:407-419, *no afirmada por spec*.

**Oráculo.** Jar oficial: escenario propuesto `evt-managed-classification.jsh`.

**Decisiones.** Q-X-4

## AC-EVT-006 — EventStream: subscribe por clase, unsubscribe, publish; null → IllegalArgumentException("subscriber is null"); modo debug event-stream

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] Bus del sistema con SubchannelClassification por clase del evento (`event.getClass`); entrega con tell al ActorRef suscriptor; null como suscriptor lanza IllegalArgumentException con mensaje exacto `subscriber is null`. Con debug=true publica Debug en cada (un)subscribe: `subscribing <ref> to channel <cls>`, `unsubscribing <ref> from channel <cls>`, `unsubscribing <ref> from all channels`, `initialized unsubscriber to: <u>, registering <n> initial subscribers with it`, `not using unsubscriber <u>, because already initialized with <p>`.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/EventStream.scala:34-161
- reference.conf:752 (pekko.actor.debug.event-stream)

**Condiciones.**
- subscribe/unsubscribe(null,..) -> IllegalArgumentException("subscriber is null") (actor/src/main/scala/org/apache/pekko/event/EventStream.scala:57,65,75).
- publish a suscriptor terminado cuando sys==null: lo desuscribe (actor/src/main/scala/org/apache/pekko/event/EventStream.scala:51-54).

**Orden / concurrencia / ciclo de vida.** subscribe: (debug Debug) -> registerWithUnsubscriber -> super.subscribe. unsubscribe: super -> unregisterIfNoMoreSubscribedChannels -> (debug Debug).

**Dependencias.** Filas: AC-EVT-003 ; AC-EVT-007. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `EventStream` con suscripciones por `EventClass`/`TypeId` (Q-X-6)

**Diferencias observables.** Rust: ver Q-X-6 para la clasificación de eventos; Debug de event-stream se conserva textual.

**Aserciones.**
- A1: subscribe(testActor, M); publish(M(42)) entrega; unsubscribe(testActor); publish(M(13)) no entrega — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:86-100.
- A2: subscribe(null, ..) lanza IllegalArgumentException con mensaje `subscriber is null` — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:102-106.
- A3: unsubscribe(null, c) y unsubscribe(null) lanzan IllegalArgumentException `subscriber is null` — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:108-113.
- A4: suscribir dos veces el mismo ref/clase devuelve true y luego false — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:311-312.
- A5: el mensaje Debug `unsubscribing <ref> from all channels` se publica en modo debug — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:315.
- A6: el mensaje Debug `unsubscribing <ref> from channel class <cls>` usa `channel.toString` de java.lang.Class (`class pkg.A` / `interface pkg.T`) — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:395-423 (dependiente de JVM).

**Oráculo.** Jar oficial: escenario propuesto `evt-eventstream-debug.jsh` para comparar textos Debug; el texto con `class`/`interface` es JVM y se mapeará (Q-EVT-1).

**Decisiones.** Q-X-6, Q-EVT-1

## AC-EVT-007 — EventStreamUnsubscriber: quita suscriptores al terminar

*Unidad EVT · visibilidad internal-required · analysis `extracted`*

**Contrato observable.** [contractual por spec] Actor de sistema `eventStreamUnsubscriber-N` que vigila a los suscriptores (watch); también cuando el suscriptor ya está muerto al suscribirse; no se inicializa dos veces; unwatch cuando el suscriptor ya no tiene ningún canal. Mensajes internos Register(actor) y UnregisterIfNoMoreSubscribedChannels(actor); el unsubscriber consulta hasSubscriptions (costoso) en su propio hilo.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/EventStreamUnsubscriber.scala:30-97
- actor/src/main/scala/org/apache/pekko/event/EventStream.scala:93-161

**Condiciones.**
- Nombre del actor: `eventStreamUnsubscriber-<contador global>` (actor/src/main/scala/org/apache/pekko/event/EventStreamUnsubscriber.scala:90).
- initUnsubscriber devuelve false si ya existe (actor/src/main/scala/org/apache/pekko/event/EventStream.scala:116-123).

**Orden / concurrencia / ciclo de vida.** Terminated(actor) -> eventStream.unsubscribe(actor) -> `unsubscribing <a> from all channels` (Debug) -> UnregisterIfNoMoreSubscribedChannels -> `unwatching <a>, since has no subscriptions`.

**Dependencias.** Filas: AC-EVT-006 ; AC-DW-001 ; AC-DW-002 ; AC-SYS-010. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** actor interno `EventStreamUnsubscriber`

**Diferencias observables.** Rust: ídem. Diferencia: ninguna observable; los textos Debug se conservan.

**Aserciones.**
- A1: al terminar el suscriptor se publican `unsubscribing <target> from all channels` y `unwatching <target>` y deja de recibir — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:298-325.
- A2: un suscriptor ya terminado al suscribirse es desuscrito inmediatamente y puede re-suscribirse (true) — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:327-356.
- A3: initUnsubscriber(p.ref) sobre un sistema ya iniciado devuelve false — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:358-373.
- A4: unsubscribe(sub) publica `unwatching <sub>` — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:375-393.
- A5: el unsubscriber hace watch por cada subscribe (idempotente): dos `watching <ref>`; unsubscribe(canal) mantiene el watch mientras queden canales; al quitar el último publica `unwatching <ref>, since has no subscriptions` — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:395-423.
- A6: unsubscribe de un canal sin suscripción devuelve false — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:420.

**Oráculo.** Jar oficial: escenario propuesto `evt-eventstream-unsubscriber.jsh`.

**Decisiones.** Q-X-4

## AC-EVT-008 — Tipos de dead letters: DeadLetter, SuppressedDeadLetter, Dropped, AllDeadLetters, DeadLetterSuppression, WrappedMessage.unwrap, y su publicación por DeadLetterActorRef/EmptyLocalActorRef

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual] DeadLetter(message, sender, recipient) exige sender y recipient no nulos; DeadLetterSuppression -> se publica SuppressedDeadLetter (no DeadLetter); Dropped(message, reason, sender, recipient); WrappedMessage.unwrap desenvuelve recursivamente. A un ref muerto (EmptyLocalActorRef) o a deadLetters, un mensaje normal publica DeadLetter con sender = deadLetters si no hay sender. AllDeadLetters es el supertipo de los tres; DeadLetterListener se suscribe a DeadLetter, Dropped y UnhandledMessage.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:579-583 (AllDeadLetters)
- actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:591-595 (DeadLetter)
- actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:609-614 (SuppressedDeadLetter)
- actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:622-631 (Dropped)
- actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:633-653 (WrappedMessage)
- actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:671-756 (EmptyLocalActorRef, DeadLetterActorRef)

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
- A5: sender/recipient null en DeadLetter lanzan IllegalArgumentException — derivada de actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:593-594, *no afirmada por spec*.
- A6: WrappedMessage.unwrap desenvuelve recursivamente — derivada de actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:638-643, *no afirmada por spec*.

**Oráculo.** Jar oficial: escenario propuesto `evt-deadletters.jsh` (suscribe probes a DeadLetter/SuppressedDeadLetter/AllDeadLetters).

**Decisiones.** Q-X-3, Q-EVT-1

## AC-EVT-009 — DeadLetterListener: log de dead letters con límite (log-dead-letters), suspensión y reinicio

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] Cuenta DeadLetter, Dropped y UnhandledMessage; loguea Info 'Message [<clase>] ...' hasta log-dead-letters; al alcanzar el límite añade `, no more dead letters will be logged in next [<dur>]` (o `, no more dead letters will be logged` y se detiene si suspend=infinite); tras suspend-duration, el siguiente mensaje se loguea con `, of which N were not logged. The counter will be reset now` y reinicia el contador; no loguea mensajes envueltos de DeadLetterSuppression ni durante el apagado salvo log-dead-letters-during-shutdown. Cuatro modos de recibir: siempre (log-dead-letters=on => Int.MaxValue), hasta N con parada, hasta N con suspensión temporal, y suspendido.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/DeadLetterListener.scala:33-161
- reference.conf:58 (log-dead-letters=10), :63 (log-dead-letters-during-shutdown=off), :68 (suspend-duration=5 minutes)
- ActorSystem.scala:441-455 (parseo: off/false->0; on/true->Int.MaxValue; entero; suspend `infinite`)

**Condiciones.**
- Texto DeadLetter: `Message [<cls>] from <sender> to <recipient> was not delivered. [<n>] dead letters encountered<doneMsg>. If this is not an expected behavior then <recipient> may have terminated unexpectedly. This logging can be turned off or adjusted with configuration settings 'pekko.log-dead-letters' and 'pekko.log-dead-letters-during-shutdown'.` (actor/src/main/scala/org/apache/pekko/event/DeadLetterListener.scala:134-145). `from <sender>` sólo si sender es real (no noSender ni deadLetters) (actor/src/main/scala/org/apache/pekko/event/DeadLetterListener.scala:121,150-152).
- Texto Dropped: `Message [<cls>] from <s> to <r> was dropped. <reason>. [<n>] dead letters encountered<doneMsg>. ` (actor/src/main/scala/org/apache/pekko/event/DeadLetterListener.scala:126-129); Unhandled: `... was unhandled. ...` (actor/src/main/scala/org/apache/pekko/event/DeadLetterListener.scala:130-133). Marker: ActorLogMarker.deadLetter(messageStr) (actor/src/main/scala/org/apache/pekko/event/DeadLetterListener.scala:147).
- Mensaje envuelto: ` wrapped in [<wrapperCls>]` (actor/src/main/scala/org/apache/pekko/event/DeadLetterListener.scala:124).
- El evento Info tiene logSource = recipient.path.toString y logClass = recipient.getClass (actor/src/main/scala/org/apache/pekko/event/DeadLetterListener.scala:141-142).

**Orden / concurrencia / ciclo de vida.** preStart suscribe DeadLetter, Dropped y UnhandledMessage; preRestart/postRestart/postStop especiales (no re-suscribe, no pierde suscripción en restart; postStop desuscribe) (actor/src/main/scala/org/apache/pekko/event/DeadLetterListener.scala:40-53).

**Dependencias.** Filas: AC-CFG-005 ; AC-EVT-008 ; AC-EVT-010 ; AC-TK-005.2. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** actor de sistema `DeadLetterListener`

**Diferencias observables.** Rust: textos conservados; `<cls>` = nombre de tipo Rust (Q-EVT-1).

**Aserciones.**
- A1: con log-dead-letters=4 y suspend=2s: los 4 primeros mensajes (DeadLetter, Dropped, Unhandled, DeadLetter) se loguean con contadores [1][2][3][4], el 4º con `, no more dead letters will be logged in next` — actor-tests/src/test/scala/org/apache/pekko/actor/DeadLetterSuspensionSpec.scala:70-84.
- A2: los mensajes 5 y 6 no se loguean — DeadLetterSuspensionSpec.scala:85-86.
- A3: tras 2050 ms el 7º se loguea con `, of which 2 were not logged` — DeadLetterSuspensionSpec.scala:88-96.
- A4: el 8º reinicia el contador: `[1] dead letters encountered` — DeadLetterSuspensionSpec.scala:98-101.
- A5: con loglevel=INFO, un mensaje a un actor muerto produce Info `from Actor[pekko://LogDeadLetters/system/testProbe.. not delivered` y (por TestEventListener) un Warning `received dead letter from Actor[pekko://LogDeadLetters/system/testProbe..` — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:164-186.
- A6: sin sender: Info `.*not delivered` y Warning `received dead letter: boom` — ActorSystemSpec.scala:188-210.
- A7: mensajes DeadLetterSuppression envueltos no se cuentan ni loguean — derivada de actor/src/main/scala/org/apache/pekko/event/DeadLetterListener.scala:154-159, *no afirmada por spec*.

**Oráculo.** Jar oficial: escenario propuesto `evt-deadletter-listener.jsh` (captura Info con EventFilter equivalente: suscripción a LogEvent y volcado de mensajes).

**Decisiones.** Q-EVT-1, Q-X-7

## AC-EVT-010 — UnhandledMessage (publicación y log con debug.unhandled)

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual] UnhandledMessage(message, sender, recipient) se publica en el EventStream; con pekko.actor.debug.unhandled=on un actor interno `UnhandledMessageForwarder` lo convierte en Debug `unhandled message from <sender>: <msg>` con logSource=recipient.path y logClass=recipient.getClass. El forwarder se crea como /system/UnhandledMessageForwarder dentro de startDefaultLoggers.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:154-166
- reference.conf:755 (debug.unhandled)

**Condiciones.**
- Si el nombre ya está en uso (InvalidActorNameException) se ignora (actor/src/main/scala/org/apache/pekko/event/Logging.scala:164-166).
- Los tests de LocalActorRefProviderSpec ven el log como Warning por TestEventListener (`unhandled message from Actor[..]: msg`, TestEventListener.scala:582-584).

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** Filas: AC-CELL-004 ; AC-EVT-006. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `struct UnhandledMessage` + forwarder interno

**Diferencias observables.** Rust: igual.

**Aserciones.**
- A1: publicar UnhandledMessage(42, deadLetters, deadLetters) con debug.unhandled=on entrega al suscriptor el mensaje y además un Debug(logSource=deadLetters.path, logClass=deadLetters.getClass, `unhandled message from <deadLetters>: 42`) — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:115-131.
- A2: sin debug.unhandled no se crea el forwarder — derivada de actor/src/main/scala/org/apache/pekko/event/Logging.scala:154-155, *no afirmada por spec*.
- A3: el warning de TestEventListener por mensaje no manejado coincide con LocalActorRefProviderSpec — actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:70-107.

**Oráculo.** Jar oficial: escenario propuesto `evt-unhandled.jsh`.

**Decisiones.** Q-EVT-1

## AC-EVT-011 — LoggingBus: niveles, setLogLevel, logger stdout de arranque, startDefaultLoggers (handshake InitializeLogger/LoggerInitialized con logger-startup-timeout), stopDefaultLoggers

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual] Antes de start se loguea por StandardOutLogger (stdout-loglevel); luego los loggers configurados (pekko.loggers) como actores bajo /system, cada uno debe responder LoggerInitialized dentro de logger-startup-timeout o se publica Warning y el arranque falla con LoggerInitializationException si responde otra cosa. setLogLevel suscribe/desuscribe cada logger a las clases Error/Warning/Info/Debug según el nivel; los eventos por debajo del nivel nunca llegan al actor.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:44-231
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:518-555 (levelFor/classFor)
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:962-990 (InitializeLogger/LoggerInitialized)
- reference.conf:19,28,31,36,41,46 (loggers, logging-filter, loggers-dispatcher, logger-startup-timeout, loglevel, stdout-loglevel)

**Condiciones.**
- pekko.loggers vacío -> DefaultLogger (actor/src/main/scala/org/apache/pekko/event/Logging.scala:126-129). StandardOutLogger en la lista se ignora al crear actores y se mantiene suscrito (actor/src/main/scala/org/apache/pekko/event/Logging.scala:133,168-170).
- Logger no cargable: ConfigurationException(`Logger specified in config can't be loaded [<name>] due to [<e>]`, e) (actor/src/main/scala/org/apache/pekko/event/Logging.scala:142-145).
- Timeout de arranque del logger: Warning `Logger <name> did not respond within <timeout> to InitializeLogger(bus)` y respuesta '[TIMEOUT]' que luego falla con LoggerInitializationException(`Logger <name> did not respond with LoggerInitialized, sent instead [TIMEOUT]`) (actor/src/main/scala/org/apache/pekko/event/Logging.scala:216-228).
- Cualquier Exception al arrancar: stderr `error while starting up loggers`, stacktrace y ConfigurationException(`Could not start logger due to [<e>]`) (actor/src/main/scala/org/apache/pekko/event/Logging.scala:171-175).
- Nombre del actor logger: `log<N>-<SimpleName>` con N de LogExt (actor/src/main/scala/org/apache/pekko/event/Logging.scala:211); dispatcher = pekko.loggers-dispatcher (actor/src/main/scala/org/apache/pekko/event/Logging.scala:212).
- stopDefaultLoggers: desuscribe y detiene cada logger; si StandardOutLogger no estaba, lo activa antes y publica Debug `shutting down: StandardOutLogger`; al final Debug `all default loggers stopped` (actor/src/main/scala/org/apache/pekko/event/Logging.scala:182-201).

**Orden / concurrencia / ciclo de vida.** Debug publicados: `StandardOutLogger started`, `Default Loggers started`, `logger <name> started`. stopDefaultLoggers corre cuando /system termina (AC-SYS-003.2).

**Dependencias.** Filas: AC-CFG-004 ; AC-EVT-006 ; AC-SYS-010. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** trait `Logger` + `LoggingBus`; Q-X-7 decide actor vs backend

**Diferencias observables.** Rust: ver Q-X-7. Diferencia: nombres `log1-DefaultLogger` se conservan sólo si los loggers son actores.

**Aserciones.**
- A1: manage log levels: tras setLogLevel(Warning/Debug/Error) sólo llegan eventos con level <= configurado — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:133-148,434-441.
- A2: un sistema con pekko.loglevel=DEBUG y un logger configurado imprime a stdout — actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:182-187.
- A3: con loglevel=OFF y stdout-loglevel=OFF no se escribe nada en stdout — actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:209-215.
- A4: con varios loggers cada uno recibe el evento — actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:217-238.
- A5: stdout-loglevel/loglevel desconocidos usan ErrorLevel — derivada de actor/src/main/scala/org/apache/pekko/event/Logging.scala:89-98,119-124, *no afirmada por spec*.
- A6: timeout de logger: Warning y LoggerInitializationException — derivada de actor/src/main/scala/org/apache/pekko/event/Logging.scala:216-228, *no afirmada por spec*.
- A7: logger no cargable -> ConfigurationException — derivada de actor/src/main/scala/org/apache/pekko/event/Logging.scala:140-146, *no afirmada por spec*.
- A8: StandardOutLogger.! con null lanza InvalidMessageException("Message is null") — derivada de actor/src/main/scala/org/apache/pekko/event/Logging.scala:1119, *no afirmada por spec*.

**Oráculo.** Jar oficial: escenario propuesto `evt-loggingbus.jsh` (loggers configurados con clase que falla, loglevel desconocido).

**Decisiones.** Q-X-7, Q-EVT-2

## AC-EVT-012 — Familia LogEvent (Error/Warning/Info/Debug y variantes con MDC/marker: Error2/Error3, ...), LogLevel, campos timestamp/thread/logSource/logClass/message/mdc

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual] Eventos inmutables publicados en el EventStream y entregados a los loggers; thread = hilo creador, timestamp = currentTimeMillis al crear; Error lleva cause (Error.NoCause por defecto); variantes con MDC (Error2..) y con marker (Error3..). LogEvent.level, logSource (String), logClass, message (Any), mdc (Map[String, Any] vacío por defecto).

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:487-555
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:711-960

**Condiciones.**
- LogEvent.apply(level, ..) para Off o niveles desconocidos: no se leyó la rama de error (actor/src/main/scala/org/apache/pekko/event/Logging.scala:772-800).
- Serialización: LogEvent está ligado a `bytes` en el test 3165 y no debe causar StackOverflowError al serializar (actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:347-356).

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** Filas: AC-EVT-011. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `enum LogEvent { Error(..), Warning(..), Info(..), Debug(..) }` con `LogLevel`

**Diferencias observables.** Rust: enum con campos; `thread` = nombre del hilo; mdc = `HashMap<String, Value>`.

**Aserciones.**
- A1: un Warning con MDC {requestId:1} llega al logger con mdc.size==1 y mdc("requestId")==1 — actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:240-259.
- A2: MDC con mensaje actual: size 3 con requestId, currentMsg y currentMsgLength==22 — actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:261-268.
- A3: MDC del mensaje se limpia en el siguiente (size 1) — actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:270-273.
- A4: serializar un LogEvent con serialize-messages=on no desborda la pila — actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:347-356.

**Oráculo.** Jar oficial: `evt-logevent-shape.jsh`.

**Decisiones.** ninguna específica

## AC-EVT-013.1 — LoggingAdapter: formato de plantillas "{}" (hasta 4 args o Array), "WARNING arguments left", isXEnabled

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] Reemplaza cada `{}` en orden por arg.toString; un único argumento Array (primitivo o de objetos) se expande como varargs; si sobran argumentos se añade ` WARNING arguments left: N`; si faltan, los `{}` restantes quedan literales; no formatea si el nivel está deshabilitado. Las sobrecargas `error(cause, template, arg1..arg4)` y `error(template, arg1..arg4)` y las equivalentes de warning/info/debug.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:1192-1300
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:1520-1557 (format1/format/formatImpl)

**Condiciones.**
- formatImpl: mientras queden argumentos busca `{}`; si no hay más `{}` añade el resto de la plantilla + ` WARNING arguments left: <n>` y termina (actor/src/main/scala/org/apache/pekko/event/Logging.scala:1533-1548).
- format1 con Array de primitivos: se boxean (actor/src/main/scala/org/apache/pekko/event/Logging.scala:1520-1524).
- notifyLog con nivel no soportado: IllegalArgumentException(`Unsupported log level [<l>]`) (actor/src/main/scala/org/apache/pekko/event/Logging.scala:1508-1514).

**Orden / concurrencia / ciclo de vida.** Se evalúa isXEnabled antes de formatear (actor/src/main/scala/org/apache/pekko/event/Logging.scala:1225).

**Dependencias.** Filas: AC-EVT-012. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `LoggingAdapter::{error,warning,info,debug}(template, &[&dyn Display])` + macros

**Diferencias observables.** Rust: sin varargs: slice de `&dyn Display`; el caso Array del único argumento se modela con un método explícito `info_all(template, args)`.

**Aserciones.**
- A1: `error("One arg message: {}", "the arg")` -> mensaje `One arg message: the arg` — LoggingAdapterTest.java:61-62.
- A2: `error(cause, "Two args message: {}, {}", ...)` -> `Two args message: an arg, another arg` con cause — LoggingAdapterTest.java:64-72.
- A3: Array de primitivos int[]{10,20,30} -> `Args as array of primitives: 10, 20, 30` — LoggingAdapterTest.java:74-76.
- A4: Array de objetos -> expandido con toString — LoggingAdapterTest.java:78-84.
- A5: cuatro argumentos -> `Four args message: 1, 2, 3, 4` — LoggingAdapterTest.java:86-87.
- A6: sobran argumentos -> sufijo ` WARNING arguments left: N` — derivada de actor/src/main/scala/org/apache/pekko/event/Logging.scala:1536-1540, *no afirmada por spec*.

**Oráculo.** Jar oficial: escenario propuesto `evt-logging-format.jsh` (casos A1-A6; incluye más args que `{}` y `{}` sin args).

**Decisiones.** Q-EVT-3

## AC-EVT-013.2 — BusLogging y NoLogging (publicación al bus y adaptador nulo)

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual] BusLogging publica Error/Warning/Info/Debug(logSource, logClass, message, mdc) en el bus si el LoggingFilter lo permite; NoLogging no hace nada y reporta todos los niveles deshabilitados. BusLogging.isXEnabled delega en el LoggingFilter con (logClass, logSource).

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:2017-2046
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:2049-2129

**Condiciones.**
- NoLogging: isErrorEnabled=false (actor/src/main/scala/org/apache/pekko/event/Logging.scala:2057) y todos los métodos no hacen nada.

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** Filas: AC-EVT-013.1 ; AC-EVT-016. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `BusLogging` y `NoLogging`

**Diferencias observables.** Rust: ídem.

**Aserciones.**
- A1: `system.log.error("Danger! Danger!")` llega a los loggers configurados — actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:157-180.
- A2: NoLogging no publica — derivada de actor/src/main/scala/org/apache/pekko/event/Logging.scala:2049-2081, *no afirmada por spec*.

**Oráculo.** Jar oficial: `evt-logging-format.jsh`.

**Decisiones.** ninguna específica

## AC-EVT-013.3 — LogSource y fábricas Logging.apply/getLogger/withMarker (obtención de adaptadores; logSource derivado de path/clase)

*Unidad EVT · visibilidad public · analysis `listed`*

**Contrato observable.** [contractual] Logging(system, obj) y Logging(bus, obj) crean un adaptador con (logSource, logClass) derivados del LogSource implícito del objeto (ActorRef -> path, String, Class, Actor, ActorSystem). Pendiente de lectura.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:233-480
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:573-705

**Condiciones.**
- Pendiente. Se vio que `Logging.apply(logSource: Actor)` crea un DiagnosticLoggingAdapter con el logFilter del sistema (actor/src/main/scala/org/apache/pekko/event/Logging.scala:645-649).

**Orden / concurrencia / ciclo de vida.** Pendiente.

**Dependencias.** Filas: AC-EVT-013.2. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** trait `LogSource` + `Logging::new(system, &source)`

**Diferencias observables.** Pendiente.

**Aserciones.**
- Pendiente de lectura.

**Oráculo.** Jar oficial para logSource de cada tipo (propuesto `evt-logsource.jsh`).

**Decisiones.** ninguna específica

## AC-EVT-014 — DiagnosticLoggingAdapter (MDC) y DiagnosticActorLogging

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] mdc(currentMessage) se invoca por cada mensaje del actor y su resultado se adjunta a cada LogEvent emitido mientras procesa ese mensaje; luego se limpia. DiagnosticLoggingAdapter mantiene `_mdc` mutable; mdc(null) -> emptyMDC; clearMDC().

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:1623-1660

**Condiciones.**
- mdc(null) se normaliza a emptyMDC; setMDC(java null) -> emptyMDC (actor/src/main/scala/org/apache/pekko/event/Logging.scala:1649-1677).

**Orden / concurrencia / ciclo de vida.** mdc() se llama antes de cada mensaje (reqId incrementa por mensaje: 1,2,3,4).

**Dependencias.** Filas: AC-EVT-013.1. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** campos estructurados de `tracing` o `HashMap` en el adaptador

**Diferencias observables.** Rust: ídem.

**Aserciones.**
- A1: reqId crece 1,2,3,4 y MDC por mensaje — actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:240-273.
- A2: ErrorLevel MDC `{messageLength=8}`, Warning `{messageLength=9}`, Info `{messageLength=9}`, 4ª llamada Debug sin MDC y 5ª con `{messageLength=29}` — LoggingAdapterTest.java:92-112.
- A3: mdc null soportado — LoggingAdapterTest.java:114- (no se leyó el cuerpo completo).

**Oráculo.** Jar oficial: `evt-logging-format.jsh` con actor DiagnosticActorLogging.

**Decisiones.** ninguna específica

## AC-EVT-015 — MarkerLoggingAdapter, LogMarker, ActorLogMarker

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] Un marker (nombre + propiedades) se adjunta al LogEvent (Error3/Warning3/Info3/Debug3); LogMarker.Security existe con nombre `SECURITY`. `ActorLogMarker.deadLetter(messageClass)` lo usa el DeadLetterListener.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:1687-1740
- actor/src/main/scala/org/apache/pekko/actor/ActorLogMarker.scala:26

**Condiciones.**
- LogMarker.toString = `LogMarker(<name>,<properties>)` (actor/src/main/scala/org/apache/pekko/event/Logging.scala:1699-1700).

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** Filas: AC-EVT-013.1. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `struct LogMarker { name, properties }` y métodos con marker

**Diferencias observables.** Rust: ídem.

**Aserciones.**
- A1: markerLogging.info(LogMarker.Security, msg) publica Info3 con marker.name == "SECURITY" y message intacto — actor-tests/src/test/scala/org/apache/pekko/event/MarkerLoggingSpec.scala:28-36.
- A2: markerLogging.error(Security, cause, msg) publica Error con cause.getMessage == "Security Exception" — actor-tests/src/test/scala/org/apache/pekko/event/MarkerLoggingSpec.scala:38-46.

**Oráculo.** Jar oficial: `evt-logging-format.jsh` con marcadores.

**Decisiones.** ninguna específica

## AC-EVT-016 — LoggingFilter / DefaultLoggingFilter / LoggingFilterWithMarker

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual] DefaultLoggingFilter habilita un nivel si eventStream.logLevel >= nivel; pluggable por FQCN con pekko.logging-filter; LoggingFilterWithMarker añade marker. Con DefaultLoggingFilter(() => InfoLevel) isDebugEnabled=false.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:1560-1620
- reference.conf:28

**Condiciones.**
- LoggingFilterWithMarkerWrapper adapta un LoggingFilter sin marker (actor/src/main/scala/org/apache/pekko/event/Logging.scala:1592-1606).

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** Filas: AC-CFG-004. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** trait `LoggingFilter` + `DefaultLoggingFilter`

**Diferencias observables.** Rust: ídem.

**Aserciones.**
- A1: con loglevel=OFF el adaptador no loguea (isErrorEnabled=false) — actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:209-215.
- A2: DefaultLoggingFilter(()=>InfoLevel) permite info del MarkerLoggingAdapter — actor-tests/src/test/scala/org/apache/pekko/event/MarkerLoggingSpec.scala:22-36.

**Oráculo.** Jar oficial: ver AC-EVT-013.1.

**Decisiones.** Q-X-2

## AC-EVT-017 — DefaultLogger / StdOutLogger / StandardOutLogger: formato de salida, timestamps, MDC, marker, causa

*Unidad EVT · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] Línea Error `[ERROR] [<ts>] [<thread>] [<source>]<mdc> <msg><stack>`; Warning/Info/Debug `[WARN|INFO|DEBUG] [<ts>] [<thread>] [<source>]<mdc> <msg>`; con marker se inserta `[<markerName>]` pegado a continuación del nivel (`[LEVEL] [<marker>][<ts>] ...`); timestamp `MM/dd/yyyy HH:mm:ss.SSS` en zona horaria del sistema; la causa se incluye aun sin stacktrace. El StandardOutLogger es un ActorRef mínimo síncrono; DefaultLogger lo envuelve como actor con LoggerMailbox.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:993-1105 (StdOutLogger y formatos)
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:1114-1136 (StandardOutLogger)
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:1143-1148 (DefaultLogger)
- actor/src/main/scala/org/apache/pekko/event/Logging.scala:1153-1162 (stackTraceFor)
- actor/src/main/scala/org/apache/pekko/util/Helpers.scala:108-113 (timestamp)

**Condiciones.**
- Formatos: ErrorFormat `[ERROR] [%s] [%s] [%s]%s %s%s`; ErrorFormatWithMarker `[ERROR] [%s][%s] [%s] [%s]%s %s%s`; sin causa `[ERROR] [%s] [%s] [%s]%s %s` (actor/src/main/scala/org/apache/pekko/event/Logging.scala:1089-1093); `[WARN] [%s] [%s] [%s]%s %s`, `[INFO] ...`, `[DEBUG] ...` y variantes con marker (actor/src/main/scala/org/apache/pekko/event/Logging.scala:1095-1102).
- formatMDC: vacío -> ''; uno -> `[k:v]`; varios -> `[k1:v1][k2:v2]` (actor/src/main/scala/org/apache/pekko/event/Logging.scala:1080-1085) — sin separador antes: el %s del formato va pegado a `[source]`.
- stackTraceFor: null/NoCause -> ''; NoStackTrace -> ` (<claseDeLaCausa>: <mensaje>)`; otro -> '\n' + printStackTrace (actor/src/main/scala/org/apache/pekko/event/Logging.scala:1153-1162).
- Evento inesperado: Warning `received unexpected event of class <cls>: <e>` (actor/src/main/scala/org/apache/pekko/event/Logging.scala:1004-1005).
- StandardOutLogger.! con null: InvalidMessageException("Message is null") (actor/src/main/scala/org/apache/pekko/event/Logging.scala:1119).

**Orden / concurrencia / ciclo de vida.** print por println síncrono; DefaultLogger procesa en su hilo (loggers-dispatcher).

**Dependencias.** Filas: AC-EVT-011 ; AC-EVT-012. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `StdOutFormatter` + `StandardOutLogger`

**Diferencias observables.** Rust: conservar bytes exactos si Q-EVT-2 lo acepta; thread = nombre del hilo del runtime.

**Aserciones.**
- A1: log.timestamp(event) == formato `MM/dd/yyyy HH:mm:ss.SSS` en zona del sistema — actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:319-327.
- A2: error con causa NoStackTrace incluye el mensaje de la causa en la salida — actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:329-343.
- A3: currentTimeMillisToUTCString devuelve `HH:mm:ss.SSSUTC` con ceros a la izquierda — actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:282-317 (Helpers, AC-UTIL).
- A4: la salida de un logger de sistema real aparece en stdout — actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:184-187.
- A5: los eventos no se pierden al cerrar: ver AC-EVT-020 — actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:189-205.
- A6: formato exacto de cada nivel con/sin marker y MDC — derivada de actor/src/main/scala/org/apache/pekko/event/Logging.scala:1008-1105, *no afirmada por spec*.
- A7: formatMDC para 1 y N entradas — derivada de actor/src/main/scala/org/apache/pekko/event/Logging.scala:1080-1085, *no afirmada por spec*.

**Oráculo.** Jar oficial: escenario propuesto `evt-stdout-format.jsh` con zona horaria fijada (-Duser.timezone=UTC) y thread name controlado; es el candidato principal de oráculo (bytes de stdout).

**Decisiones.** Q-EVT-2, Q-X-7

## AC-EVT-018 — ActorLogging (log perezoso con logSource = path del actor)

*Unidad EVT · visibilidad public · analysis `listed`*

**Contrato observable.** [contractual] `log` se crea al primer uso con el actor como fuente (Logging(context.system, this)). Pendiente de lectura.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/actor/Actor.scala:330-368

**Condiciones.**
- Pendiente.

**Orden / concurrencia / ciclo de vida.** Pendiente.

**Dependencias.** Filas: AC-EVT-013.3. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `ctx.log()`

**Diferencias observables.** Rust: ídem.

**Aserciones.**
- Pendiente de lectura.

**Oráculo.** Jar oficial: `evt-logsource.jsh`.

**Decisiones.** ninguna específica

## AC-EVT-019 — LoggingReceive (decora receive con log Debug; activable por debug.receive)

*Unidad EVT · visibilidad public · analysis `listed`*

**Contrato observable.** [contractual] Si pekko.actor.debug.receive=on, LoggingReceive envuelve el Receive y antes de cada match publica en el EventStream `received handled|unhandled message <msg> from <sender>[ in state <label>]` al nivel configurado (Debug por defecto) si eventStream.logLevel >= nivel. withLabel(label)(r): si r ya es LoggingReceive se devuelve igual; si no y AddLoggingReceive, new LoggingReceive; si no, r intacto.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/LoggingReceive.scala:28-107
- reference.conf:740 (debug.receive)

**Condiciones.**
- El evento incluye MDC si la fuente es DiagnosticActorLogging (actor/src/main/scala/org/apache/pekko/event/LoggingReceive.scala:98-101).

**Orden / concurrencia / ciclo de vida.** Se evalúa en isDefinedAt (no en apply), antes de aplicar.

**Dependencias.** Filas: AC-CFG-011 ; AC-EVT-012. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** wrapper de behavior

**Diferencias observables.** Rust: dependiente de Q-X-3 (comportamiento dinámico).

**Aserciones.**
- A1: sin debug.receive el Receive no se modifica — derivada de actor/src/main/scala/org/apache/pekko/event/LoggingReceive.scala:64-67, *no afirmada por spec leída*.
- A2: con debug.receive=on se publica un evento por mensaje con `received handled|unhandled message <m> from <sender>` — derivada de actor/src/main/scala/org/apache/pekko/event/LoggingReceive.scala:87-103.
- A3: etiqueta `in state <l>` — actor/src/main/scala/org/apache/pekko/event/LoggingReceive.scala:94-97.
- A4: MDC adjunto con DiagnosticActorLogging — actor/src/main/scala/org/apache/pekko/event/LoggingReceive.scala:98-101.

**Oráculo.** Jar oficial: escenario propuesto `evt-loggingreceive.jsh`.

**Decisiones.** Q-X-3

## AC-EVT-020 — LoggerMailbox (vacía la cola de logs al cerrar)

*Unidad EVT · visibilidad internal-required · analysis `extracted`*

**Contrato observable.** [contractual por spec] Al limpiar el mailbox del logger (cleanUp), los LogEvent pendientes con level <= logLevel se imprimen por StandardOutLogger para no perderlos; si el nivel es OFF no se imprime. LoggerMailboxType exige owner y system (`no mailbox owner or system given`).

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/LoggerMailbox.scala:28-73
- reference.conf:731 (logger-queue)

**Condiciones.**
- LoggerMailboxType.create sin owner/system: IllegalArgumentException("no mailbox owner or system given") (actor/src/main/scala/org/apache/pekko/event/LoggerMailbox.scala:39).

**Orden / concurrencia / ciclo de vida.** cleanUp ocurre tras cambiar el mailbox por el de dead letters (comentario actor/src/main/scala/org/apache/pekko/event/LoggerMailbox.scala:55-56).

**Dependencias.** Filas: AC-MBX-004. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** mailbox del logger con drenaje final

**Diferencias observables.** Rust: ídem si hay logger-actor.

**Aserciones.**
- A1: con un logger lento (500 ms en msg1), tres errores seguidos y shutdownActorSystem(verifySystemShutdown=true): stdout tiene 3 líneas, la primera con msg1 y la última con msg3 — actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:189-205.
- A2: eventos con level > logLevel se descartan en cleanUp — derivada de actor/src/main/scala/org/apache/pekko/event/LoggerMailbox.scala:62, *no afirmada por spec*.

**Oráculo.** Jar oficial: `evt-logger-drain.jsh` (SlowLogger de LoggerSpec).

**Decisiones.** Q-X-7

## AC-EVT-021 — JavaLogger / JavaLoggingFilter (java.util.logging)

*Unidad EVT · visibilidad deprecated · analysis `listed`*

**Contrato observable.** [contractual] Logger hacia JUL, deprecado. Pendiente de lectura.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/jul/JavaLogger.scala:32-120

**Condiciones.**
- Pendiente.

**Orden / concurrencia / ciclo de vida.** Pendiente.

**Dependencias.** Filas: AC-EVT-011. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** bridge a la fachada `log`/`tracing` (feature opcional)

**Diferencias observables.** Rust: bridge opcional.

**Aserciones.**
- Pendiente de lectura.

**Oráculo.** No aplica: depende de java.util.logging.

**Decisiones.** Q-EVT-4

## AC-EVT-022 — API Java del EventBus (event/japi/EventBusJavaAPI.scala): clases abstractas LookupEventBus, SubchannelEventBus, ScanningEventBus, ManagedActorEventBus

*Unidad EVT · visibilidad public · analysis `listed`*

**Contrato observable.** [contractual] Adaptadores para implementar los buses desde Java sin tipos abstractos. Pendiente de lectura.

**Fuente.**
- actor/src/main/scala/org/apache/pekko/event/japi/EventBusJavaAPI.scala

**Condiciones.**
- Pendiente.

**Orden / concurrencia / ciclo de vida.** Pendiente.

**Dependencias.** Filas: AC-EVT-002 ; AC-EVT-003 ; AC-EVT-004 ; AC-EVT-005 ; AC-JAPI-001. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** mismas implementaciones; sin distinción Java/Scala

**Diferencias observables.** Q-EVT-5.

**Aserciones.**
- Pendiente de lectura.

**Oráculo.** No aplica.

**Decisiones.** Q-EVT-5

## AC-TK-001.1 — TestKitBase/TestKit: testActor (cola interna, actor de sistema sobre CallingThreadDispatcher), lastSender, msgAvailable, ignoreMsg/ignoreNoMsg, watch/unwatch, setAutoPilot

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] testActor es un actor creado con systemActorOf bajo /system con nombre `<testActorName>-<N>` (N global creciente), ejecutado en CallingThreadDispatcher; encola en una cola bloqueante (LinkedBlockingDeque) cada mensaje (mensaje, sender); el resto de aserciones consumen esa cola. El TestKit es el receptor de las respuestas de los actores bajo prueba; `testActor`, `lastSender`, `ignoreMsg`, `watch`, `setAutoPilot` son mensajes internos (SetIgnore, Watch, UnWatch, SetAutoPilot, Spawn) procesados por TestActor.

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:37-151 (TestActor)
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:168-199 (TestKitBase, testActor)
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:213-241
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:286
- testkit/src/main/resources/reference.conf:31-33 (calling-thread-dispatcher)

**Condiciones.**
- testActor lazy: `systemActorOf(TestActor.props(queue).withDispatcher(CallingThreadDispatcher.Id), "%s-%d".format(testActorName, testActorId.incrementAndGet))` y espera a `isStarted` si es RepointableRef (hasta 3 s dilatados, polling 10 ms) (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:189-199).
- lastMessage = NullMessage hasta que alguna aserción desencola; `lastSender`/`lastMessage.msg` sobre NullMessage lanzan IllegalActorStateException("last receive did not dequeue a message") (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:71-74).
- ignoreMsg(f): los mensajes para los que f devuelve true no se encolan (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:141-142); ignoreNoMsg() lo borra.
- Mensajes no AnyRef (primitivos) llegan boxed: `case x: AnyRef` (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:136).

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
- A6: nombres de actor de sistema `testActor-N`/`testProbe-N` aparecen como `Actor[pekko://<sys>/system/testProbe..` en logs — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:174,178.
- A7: al detenerse el testActor, los mensajes pendientes se reenvían a deadLetters como DeadLetter(m, sender, self) — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:145-150, *no afirmada por spec*.

**Oráculo.** Jar oficial (pekko-testkit 1.7.1) vía jshell: escenario propuesto `tk-testactor.jsh` (path y nombre, orden síncrono). Procedencia: jars oficiales (requiere fetch-jars con pekko-testkit).

**Decisiones.** Q-TK-1, Q-TK-4

## AC-TK-001.2 — TestKit expect*: expectMsg, expectMsgPF, expectMsgType/Class, expectMsgAnyOf/AnyClassOf, expectMsgAllOf/AllClassOf/AllConformingOf, expectTerminated

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] Cada expect* toma el siguiente mensaje de la cola con timeout dilatado (por defecto remainingOrDefault) y falla con AssertionError cuyo mensaje contiene el hint si vence o no coincide. El timeout de las variantes con duración se dilata (`max.dilated`); `receiveOne` NO dilata.

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:448-474 (expectMsg)
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:486-492 (expectMsgPF)
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:503-506 (expectTerminated)
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:552-566 (expectMsgType)
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:632-691 (expectMsgAllOf/AllClassOf)

**Condiciones.**
- expectMsg vence: AssertionError `timeout (<max>) during expectMsg while waiting for <obj>` + `: <hint>` (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:471).
- expectMsg distinto: AssertionError `expected <obj>, found <o>` + `: <hint>` (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:472).
- expectMsgPF sin match: `expected: <hint> but got unexpected message <o>` (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:490); timeout: `timeout (<max>) during expectMsg: <hint>` (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:489).
- expectMsgAllOf: recibe n=obj.size mensajes; error `not found [..] found unexpected [..]` (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:649-664).
- expectMsgAllClassOf compara clases exactas con BoxedType (primitivos boxed); AllConformingOf compara por conformidad (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:685-691).

**Orden / concurrencia / ciclo de vida.** Consume exactamente un mensaje por expect (excepto *All*, n mensajes). Mensajes no coincidentes se pierden (se consumieron).

**Dependencias.** Filas: AC-TK-001.1 ; AC-TK-001.5 ; AC-TK-004. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `expect_msg(&v)`, `expect_msg_type::<T>()`, `expect_msg_pf(f)`, `expect_msg_all_of(&[..])`, `expect_terminated(&ref)`

**Diferencias observables.** Rust: aserciones por `panic!`/`assert!` o `Result`; el AssertionError JVM pasa a `TestFailure` (Q-X-5). Mensajes de fallo textuales: conservar si algún spec porteado los afirma (sólo el hint, testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:93-124).

**Aserciones.**
- A1: `expectMsg(0.millis, hint, "hello")` sin mensajes -> AssertionError cuyo mensaje contiene `some hint` — testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:107-114.
- A2: `expectMsg(0.millis, hint, "bye")` con "hello" en cola -> AssertionError que contiene el hint — testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:116-124.
- A3: expectMsgType[Int] / expectMsgAnyClassOf(classOf[Int]) / expectMsgAllClassOf(classOf[Int]) / expectMsgAllConformingOf(classOf[Int]) (con y sin duración) aceptan el primitivo 42 boxed — testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:151-160.
- A4: expectTerminated(target) devuelve el Terminated si el testActor vigila a target — testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:200-211 (usa expectMsg con Terminated).
- A5: `expectMsg(max, obj)` dilata `max` por timefactor — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:457, *no afirmada por spec leída*.
- A6: expectMsgAllOf informa `not found`/`found unexpected` — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:649-664, *no afirmada por spec*.
- A7: expectMsgPF devuelve f(o) — testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:486-492, *no afirmada por spec leída*.
- A8: expectMsgAnyOf/AnyClassOf/AllConformingOf — pendiente de leer cuerpos (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:589-725).

**Oráculo.** Jar oficial pekko-testkit: `tk-expect.jsh` comparando textos de AssertionError (A1-A2, A6).

**Decisiones.** Q-TK-4, Q-X-5

## AC-TK-001.3 — TestKit expectNoMessage / expectNoMsg

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] expectNoMessage() espera pekko.test.expect-no-message-default (100 ms) dilatado; expectNoMessage(max) NO dilata; si llega un mensaje lo extrae de la cola y lanza AssertionError `assertion failed: received unexpected message <m> after <n> millis`; expectNoMsg(max) (deprecado) sí dilata. Polling con espera geométrica (max 100 ms) sobre peekFirst; sólo hace pop del mensaje ofensor.

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:727-783
- testkit reference.conf:26 (expect-no-message-default = 100ms)

**Condiciones.**
- Error: `java.lang.AssertionError: assertion failed: received unexpected message <elem> after <ms> millis` (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:778-779). El elem es el RealMessage(msg, sender), no sólo el msg (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:772-778: `elem` es de la cola).

**Orden / concurrencia / ciclo de vida.** El mensaje que provoca el fallo se saca de la cola para que expectNoMessage posteriores no lo vean (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:775).

**Dependencias.** Filas: AC-TK-004. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `expect_no_message()` y `expect_no_message_for(dur)`

**Diferencias observables.** Rust: ídem; mensaje de error conservado si un spec lo afirma (ninguno lo hace).

**Aserciones.**
- A1: dos mensajes en cola, `eventually(expectNoMessage(100.millis))` falla las primeras veces (extrayendo uno cada vez) y luego pasa; después un "tak" nuevo se recibe — testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:223-233.
- A2: expectNoMessage tras within no hace fallar el within por tiempo — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:433,781, *no afirmada por spec*.
- A3: formato del mensaje de error — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:778-779, *no afirmada por spec*.

**Oráculo.** Jar oficial: `tk-expect.jsh`.

**Decisiones.** ninguna específica

## AC-TK-001.4 — TestKit receiveWhile, receiveN, receiveOne, fishForMessage, fishForSpecificMessage

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] receiveN(n) recibe n mensajes antes del deadline (AssertionError `timeout (<max>) while expecting <n> messages (got <k>)`); receiveWhile consume mientras el PF acepte y devuelve la lista (no es error alcanzar max); fishForMessage descarta mensajes hasta que el PF devuelve true y falla si el PF no está definido; fishForSpecificMessage ignora los no definidos. receiveOne(max): max==0 -> pollFirst; finito -> pollFirst(timeout); infinito -> takeFirst; actualiza lastMessage (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:862-881).

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:516-547
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:804-833
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:839-881

**Condiciones.**
- fishForMessage: timeout `timeout (<max>) during fishForMessage, hint: <hint>`; PF no definido: `fishForMessage(<hint>) found unexpected message <o>` (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:522-523).
- fishForSpecificMessage: timeout `timeout (<max>) during fishForSpecificMessage, hint: <hint>` (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:543).
- receiveWhile: si el mensaje no coincide se devuelve a la cabeza de la cola (offerFirst) y se restaura lastMessage (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:821-824); activa lastWasNoMsg (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:831).

**Orden / concurrencia / ciclo de vida.** receiveWhile con `idle` corta si pasa ese tiempo sin mensajes.

**Dependencias.** Filas: AC-TK-001.1. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `receive_n(n)`, `receive_while(f)`, `fish_for_message(f)`, `fish_for_specific_message(f)`

**Diferencias observables.** Rust: ídem.

**Aserciones.**
- A1: fishForMessage con PF `case "fishForMe" => true; case _ => false` consume "hallo", "welt", "fishForMe" y deja "done" — testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:162-175.
- A2: fishForSpecificMessage devuelve "fishForMe" y deja "done" — testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:177-191.
- A3: receiveN(n) con menos mensajes -> AssertionError `timeout .. while expecting n messages (got k)` — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:851, *no afirmada por spec*.
- A4: receiveWhile devuelve los resultados en orden y re-encola el primer no coincidente — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:810-826, *no afirmada por spec*.
- A5: fishForMessage con PF no definido para un mensaje falla — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:523, *no afirmada por spec*.
- A6: receiveOne(0) no bloquea — testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:863-865.

**Oráculo.** Jar oficial: `tk-receive.jsh`.

**Decisiones.** ninguna específica

## AC-TK-001.5 — TestKit within / remaining / remainingOr / remainingOrDefault / awaitCond / awaitAssert / assertForDuration

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] within(min,max) acota el tiempo de un bloque (max dilatado; min no); anidados toman el menor; remaining fuera de within lanza AssertionError; awaitCond vence con AssertionError `timeout <max> expired: <message>`; awaitAssert relanza la última excepción al vencer. El estado `end` (Duration.Undefined fuera de within) lo comparte todo el TestKit (campo var, no thread-safe).

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:253-281 (remaining*)
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:298-359 (awaitCond/awaitAssert)
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:372-397
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:415-443 (within)

**Condiciones.**
- remaining fuera de within: AssertionError `` `remaining` may not be called outside of `within` `` (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:262).
- remainingOr con end infinito: IllegalArgumentException("`end` cannot be infinite") (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:271,273); max infinito: "max duration cannot be infinite" (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:278,280).
- within: AssertionError `required min time <min> not possible, only <rem> left` (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:419); `block took <d>, should at least have been <min>` (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:432); `block took <d>, exceeding <max>` (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:434), este último no se comprueba si la última aserción fue expectNoMessage/receiveWhile (lastWasNoMsg).
- awaitCond: `assert(now < stop, s"timeout ${_max} expired: $message")` (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:309); polling cada `interval` (100 ms por defecto) (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:307-315).
- awaitAssert: la excepción se relanza si `(now + t) >= stop` (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:347); sólo captura NonFatal.

**Orden / concurrencia / ciclo de vida.** within restaura `end` en finally (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:427-429).

**Dependencias.** Filas: AC-TK-004. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `within(min, max, || ..)`, `remaining()`, `await_cond(..)`, `await_assert(..)`

**Diferencias observables.** Rust: ídem; el estado `end` puede ser un campo con `Cell`/`Mutex`. Q-TK-4 (API bloqueante/async).

**Aserciones.**
- A1: con timefactor 2.0, `1.second.dilated.toNanos == 1e9 * timefactor` — testkit/src/test/scala/org/apache/pekko/testkit/TestTimeSpec.scala:24-25.
- A2: `probe.awaitCond(false, 1.second)` lanza AssertionError y tarda >= 1 s * timefactor y < +1 s — testkit/src/test/scala/org/apache/pekko/testkit/TestTimeSpec.scala:27-33.
- A3: `awaitAssert("foo" === "foo")` pasa; `awaitAssert("foo" === "bar", 500.millis, 300.millis)` dentro de within(300ms,2s) lanza TestFailedException — testkit/src/test/scala/org/apache/pekko/testkit/TestTimeSpec.scala:36-43.
- A4: `remaining` fuera de `within` lanza AssertionError — testkit/src/test/scala/org/apache/pekko/testkit/TestTimeSpec.scala:45-49.
- A5: within fallido por exceder max — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:434, *no afirmada por spec*.
- A6: within anidado toma el menor tiempo restante — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:423-425, *no afirmada por spec*.
- A7: `within(3.seconds){ expectMsg(..); ..; expectNoMessage() }` — actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:94-99.

**Oráculo.** Jar oficial: `tk-within.jsh` (mensajes de AssertionError).

**Decisiones.** Q-TK-4

## AC-TK-002.1 — TestProbe: ref, send, forward, sender, reply, nombres (testProbe-N / nombre de usuario)

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] TestProbe es un TestKit con su propio testActor; `send(actor,msg)` usa testActor como sender; `reply(msg)` envía al sender del último mensaje con el probe como emisor; `forward(actor, msg=último)` conserva el sender original; el nombre del actor empieza por el indicado (por defecto `testProbe`). TestProbe.apply()(implicit system) y apply(name).

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:1055-1093

**Condiciones.**
- reply sobre NullMessage lanza IllegalActorStateException (por lastMessage.sender, testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:72-74).

**Orden / concurrencia / ciclo de vida.** Síncrono (ver AC-TK-001.1).

**Dependencias.** Filas: AC-TK-001.1. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `TestProbe::new(&system)`, `TestProbe::named(&system, name)`

**Diferencias observables.** Rust: ídem.

**Aserciones.**
- A1: probe1.send(probe2.ref, "hello"); probe2.expectMsg(0.millis,"hello"); probe2.lastMessage.sender ! "world"; probe1.expectMsg(0.millis, "some hint here", "world") — testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:50-57.
- A2: TestProbe("worker").ref.path.name empieza por `worker` — testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:213-216.
- A3: `new TestProbe(system).ref.path.name` empieza por `testProbe` — testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:218-221.
- A4: forward y reply — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:1076-1086, *no afirmada por spec leída*.

**Oráculo.** Jar oficial: `tk-testactor.jsh`.

**Decisiones.** Q-TK-1

## AC-TK-002.2 — AutoPilot (TestActor.AutoPilot, NoAutoPilot, KeepRunning) y ejecución antes de encolar

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] El AutoPilot corre por cada mensaje recibido por el testActor (antes de decidir ignorar y encolar); su retorno es el siguiente pilot (KeepRunning conserva el actual). El pilot recibe `sender()` y el mensaje; puede reenviar a otro testActor.

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:40-52
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:136-143

**Condiciones.**
- autopilot corre siempre (incluso con ignoreMsg que luego descarta el mensaje) (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:137-142).

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
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:79-112
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:896-934

**Condiciones.**
- Delegado ausente -> stoppingStrategy (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:84).

**Orden / concurrencia / ciclo de vida.** childActorOf usa `testActor ! Spawn` y `expectMsgType[ActorRef]`, por lo que consume un mensaje de la cola.

**Dependencias.** Filas: AC-TK-001.1 ; AC-SUP-003. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `probe.child_actor_of(props, name, strategy)`

**Diferencias observables.** Rust: ídem.

**Aserciones.**
- A1: `child.path.parent == probe.ref.path` y nombre dado respetado — testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:59-66.
- A2: con SupervisorStrategy.defaultStrategy un hijo que lanza RuntimeException se reinicia (postRestart llamado > 1 vez) — testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:68-91.
- A3: sin estrategia, el hijo que falla se detiene — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:84, *no afirmada por spec*.

**Oráculo.** Jar oficial: `tk-childactorof.jsh`.

**Decisiones.** Q-X-5

## AC-TK-003 — ImplicitSender y DefaultTimeout

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] ImplicitSender hace que `self` sea testActor (sender implícito); DefaultTimeout expone `timeout` = pekko.test.default-timeout. Mixins; nada más.

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:1095-1101

**Condiciones.**
- n/a

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** Filas: AC-TK-001.1 ; AC-TK-004. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `TestKit::tell_from_probe`/sender explícito; `default_timeout()`

**Diferencias observables.** Diferencia observable de API: `self` implícito no existe en Rust.

**Aserciones.**
- A1: `self === testActor` — testkit/src/test/scala/org/apache/pekko/testkit/ImplicitSenderSpec.scala:28-32.
- A2: `timeout === testKitSettings.DefaultTimeout` — testkit/src/test/scala/org/apache/pekko/testkit/DefaultTimeoutSpec.scala:28-32.

**Oráculo.** No aplica (es sintaxis Scala).

**Decisiones.** Q-X-3

## AC-TK-004 — Dilatación de tiempo y settings (pekko.test.timefactor, single-expect-default, expect-no-message-default, filter-leeway, default-timeout)

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] timefactor debe ser positivo y finito (si no, excepción al crear TestKitSettings); `dilated` multiplica duraciones por timefactor con redondeo (+0.5); TestKitExtension es una extensión por sistema. TestKitSettings: TestTimeFactor, SingleExpectDefaultTimeout (3s), ExpectNoMessageDefaultTimeout (100ms), TestEventFilterLeeway (3s), DefaultTimeout (5s).

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/TestKitExtension.scala:25-42
- testkit/src/main/scala/org/apache/pekko/testkit/package.scala:67-70
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
- testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:88-157 (EventFilter base)
- testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:176-308 (fábricas)
- testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:324-375 (ErrorFilter)
- testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:385-510 (Warning/Info/DebugFilter)
- testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:517-542 (Custom, DeadLettersFilter)

**Condiciones.**
- ErrorFilter: coincide si (throwable es NoCause.getClass o throwable.isInstance(cause)) y [(msg==null && cause.getMessage==null && stackTrace vacío) || doMatch(src,msg) || doMatch(src, cause.getMessage)] (testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:331-336).
- doMatch: `(source vacío o igual) && (complete ? msg == s : msg.startsWith(s)) | regex.findFirstIn(msg).isDefined)` (testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:149-155).
- DeadLettersFilter coincide con Warning cuyo `msg` sea instancia (boxed) de la clase dada (testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:535-539).
- Con occurrences=Int.MaxValue no hay conteo (testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:86,101).

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
- A8: filtros por `start`, `pattern`, `message`, `source` — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:149-155, *no afirmada por spec*.
- A9: `EventFilter.warning(pattern = "received dead letter from Actor\[pekko://LogDeadLetters/system/testProbe.*")` — uso en ActorSystemSpec.scala:174-181.

**Oráculo.** Jar oficial pekko-testkit: `tk-eventfilter.jsh` con una tabla (filtro, evento) -> bool.

**Decisiones.** Q-TK-3

## AC-TK-005.2 — EventFilter.intercept, filterEvents, filterException, awaitDone/assertDone y occurrences

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] intercept publica Mute(this), ejecuta el bloque, espera hasta filter-leeway (dilatado) a que se alcancen exactamente `occurrences` y publica UnMute en finally; si faltan: AssertionError `timeout (<leeway>) waiting for <n> messages on <filter>`; si sobran: `received <k> excess messages on <filter>`. filterEvents(filters)(block): Mute(filters); bloque; espera con leeway todos; UnMute en finally; falla con `Filter completion error:\n Timeout (<leeway>) waiting for <f>`.

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:106-137
- testkit/src/main/scala/org/apache/pekko/testkit/package.scala:28-54
- testkit reference.conf:18 (filter-leeway = 3s)

**Condiciones.**
- filterEvents falla con AssertionError("Filter completion error:\n" + lista de `Timeout (<leeway>) waiting for <filtro>`) (package.scala:41-42).
- intercept y filterEvents no comprueban excesos hasta el final; `awaitDone` devuelve true si todo==MaxValue o todo==0 (testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:106-109).

**Orden / concurrencia / ciclo de vida.** Mute/UnMute son mensajes al EventStream procesados por el TestEventListener (asíncrono respecto del bloque; por eso la espera con leeway).

**Dependencias.** Filas: AC-TK-005.3 ; AC-TK-004. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `filter.intercept(&system, || ..)`, `filter_events(&system, [..], || ..)`

**Diferencias observables.** Rust: ídem; la espera usa poll con dilatación.

**Aserciones.**
- A1: `EventFilter.warning(pattern="received dead letter from Actor\[pekko://LogDeadLetters/system/testProbe.*", occurrences=1).intercept{ a.tell("boom", probe.ref) }(sys)` pasa con exactamente 1 ocurrencia — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:164-186.
- A2: intercept anidado (info `.*not delivered` y warning `received dead letter: boom`) — actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:188-210.
- A3: `filterEvents(EventFilter.warning(s"unhandled message from Actor[$deadLettersPath]: $message", occurrences = 1)) { rootGuardian ! message }` — actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:77-79.
- A4: falta/exceso de ocurrencias produce AssertionError con los textos fijos — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:115-119,130-134, *no afirmada por spec*.
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
- A4: DeadLettersFilter coincide con el Warning interno (prefijo no incluido) — testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:535-539,572-573, *no afirmada por spec leída*.
- A5: Dropped -> Warning `dropped message from <sender>. <reason>: <msg>` — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:585-588.
- A6: Terminate se ignora — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:571.

**Oráculo.** Jar oficial pekko-testkit: `tk-testeventlistener.jsh`.

**Decisiones.** Q-X-7

## AC-TK-006.1 — TestActorRef (underlyingActor, receive síncrono)

*Unidad TK · visibilidad public · analysis `listed`*

**Contrato observable.** [contractual por spec] Ref a un actor que procesa mensajes en el hilo del emisor y expone el actor subyacente. Pendiente de lectura.

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/TestActorRef.scala:34-263

**Condiciones.**
- Pendiente.

**Orden / concurrencia / ciclo de vida.** Pendiente.

**Dependencias.** Filas: AC-TK-006.2 ; AC-REF-002. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `TestActorRef<A>` con `underlying_actor()` (Q-TK-2)

**Diferencias observables.** Q-TK-2.

**Aserciones.**
- Pendiente de lectura.

**Oráculo.** Jar oficial pekko-testkit tras leer.

**Decisiones.** Q-TK-2

## AC-TK-006.2 — CallingThreadDispatcher (ejecución síncrona en el hilo del emisor; configurador en reference.conf del testkit)

*Unidad TK · visibilidad public · analysis `listed`*

**Contrato observable.** [contractual por spec] Dispatcher `pekko.test.calling-thread-dispatcher` (type CallingThreadDispatcherConfigurator) que procesa cada mensaje en el hilo que lo envía; lo usan testActor/TestProbe (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:192). Pendiente de lectura; sólo se verificó el uso por TestKit y su declaración de config.

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/CallingThreadDispatcher.scala:68-395
- testkit/src/main/resources/reference.conf:31-33

**Condiciones.**
- Pendiente.

**Orden / concurrencia / ciclo de vida.** Pendiente (reentrancia, orden entre hilos, suspensión).

**Dependencias.** Filas: AC-DSP-001. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** dispatcher de test síncrono (inline)

**Diferencias observables.** Q-TK-1.

**Aserciones.**
- Pendiente de lectura.

**Oráculo.** Jar oficial tras leer.

**Decisiones.** Q-TK-1

## AC-TK-007 — TestFSMRef

*Unidad TK · visibilidad public · analysis `listed`*

**Contrato observable.** [contractual por spec] Acceso a estado, datos y timers de un FSM. Pendiente.

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/TestFSMRef.scala:44-147

**Condiciones.**
- Pendiente.

**Orden / concurrencia / ciclo de vida.** Pendiente.

**Dependencias.** Filas: AC-TK-006.1 ; AC-FSM-001. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** -

**Diferencias observables.** Depende de AC-FSM.

**Aserciones.**
- Pendiente de lectura.

**Oráculo.** Jar oficial tras leer.

**Decisiones.** Q-TK-2

## AC-TK-008 — ExplicitlyTriggeredScheduler (tiempo manual)

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] timePasses(d) avanza el reloj lógico, ejecuta en el hilo llamante todas las tareas vencidas en orden de tiempo (incluidas las que se programan durante la ejecución) y respeta la cancelación; tareas con delay<=0 se ejecutan al programarlas; las periódicas se reprograman sólo si la ejecución no falló. currentTime en ms (AtomicLong); `scheduled` ConcurrentHashMap Item->tiempo; las tareas se ejecutan ordenadas por tiempo, una a una (re-evalúa el conjunto tras cada ejecución).

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/ExplicitlyTriggeredScheduler.scala:44-141
- reference.conf (actor): pekko.scheduler.implementation para inyectarlo (AC-SCH-010)

**Condiciones.**
- El runnable que lanza excepción no se reprograma y no propaga el error (Try) (testkit/src/main/scala/org/apache/pekko/testkit/ExplicitlyTriggeredScheduler.scala:97-101).
- cancel(): true si el size del mapa bajó; isCancelled se marca siempre (testkit/src/main/scala/org/apache/pekko/testkit/ExplicitlyTriggeredScheduler.scala:124-129).
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

## AC-TK-009 — Utilidades: TestLatch, TestBarrier, TestActors (Echo, Blackhole, Forward), TestException, SocketUtil

*Unidad TK · visibilidad public · analysis `listed`*

**Contrato observable.** [contractual] Helpers usados por las specs (p. ej. TestLatch en ActorSystemSpec; TestActors.echoActorProps en DeadLetterSupressionSpec). Pendiente.

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/TestLatch.scala:31-61
- testkit/src/main/scala/org/apache/pekko/testkit/TestActors.scala:21-56

**Condiciones.**
- Pendiente.

**Orden / concurrencia / ciclo de vida.** Pendiente.

**Dependencias.** Filas: AC-TK-001.1. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** módulo `testkit::util`

**Diferencias observables.** Rust: equivalentes con `Barrier`/`Notify`.

**Aserciones.**
- Pendiente de lectura.

**Oráculo.** No aplica.

**Decisiones.** ninguna específica

## AC-TK-010 — TestKit.shutdownActorSystem / shutdown (verifySystemShutdown)

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual] Termina el sistema y espera hasta `duration` dilatado (por defecto min(10s dilatado, 10s)); si vence imprime (o lanza RuntimeException si verifySystemShutdown) `Failed to stop [<name>] within [<d>] \n<printTree>`. duration indefinida -> `10.seconds.dilated.min(10.seconds)` (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:1036).

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:889-894
- testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:1030-1049

**Condiciones.**
- Timeout: mensaje `Failed to stop [%s] within [%s] \n%s` (testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:1042-1045); println si no verify, RuntimeException si verify.

**Orden / concurrencia / ciclo de vida.** terminate() y luego Await.ready(whenTerminated, d).

**Dependencias.** Filas: AC-SYS-004 ; AC-SYS-013 ; AC-TK-004. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `shutdown_actor_system(&system, timeout, verify)`

**Diferencias observables.** Rust: `Result<(), ShutdownTimeout>`.

**Aserciones.**
- A1: `shutdownActorSystem(sys, verifySystemShutdown = true)` vuelve sin error cuando el sistema termina (drena logs) — actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:189-205.
- A2: tope de 10 s para duración indefinida — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:1036, *no afirmada por spec*.
- A3: mensaje de fallo con printTree — derivada de testkit/src/main/scala/org/apache/pekko/testkit/TestKit.scala:1042-1047, *no afirmada por spec*.

**Oráculo.** Jar oficial: `tk-shutdown.jsh`.

**Decisiones.** ninguna específica

## AC-TK-011 — javadsl TestKit / EventFilter

*Unidad TK · visibilidad public · analysis `listed`*

**Contrato observable.** [contractual] Variante Java de TestKit y EventFilter. Pendiente.

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/javadsl/TestKit.scala
- testkit/src/main/scala/org/apache/pekko/testkit/javadsl/EventFilter.scala

**Condiciones.**
- Pendiente.

**Orden / concurrencia / ciclo de vida.** Pendiente.

**Dependencias.** Filas: AC-TK-001.1 ; AC-TK-005.1. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** N/A (misma API; sin distinción Java)

**Diferencias observables.** Q-EVT-5.

**Aserciones.**
- Pendiente de lectura.

**Oráculo.** No aplica.

**Decisiones.** Q-EVT-5

## AC-TK-012.1 — PekkoSpec: configuración base de specs (testConf), nombre del sistema por la pila de llamadas, muteDeadLetters, ciclo before/after

*Unidad TK · visibilidad internal-required · analysis `extracted`*

**Contrato observable.** [contractual para portar specs] PekkoSpec crea un ActorSystem cuyo nombre sale del nombre de la clase del spec (TestKitUtils.testNameFromCallStack: sin paquete, sin `$..`, caracteres no [a-zA-Z_0-9-] -> `_`), con testConf: loggers=[TestEventListener], loglevel=WARNING, stdout-loglevel=WARNING, default-dispatcher fork-join (parallelism 8); apaga el sistema en afterAll. PekkoSpec mezcla TestKit con ScalaTest; en Rust el harness deberá ofrecer: sistema por test con nombre derivado, config base, `expect*`, `EventFilter`, apagado al final.

**Fuente.**
- testkit/src/test/scala/org/apache/pekko/testkit/PekkoSpec.scala:38-148
- testkit/src/main/scala/org/apache/pekko/testkit/TestKitUtils.scala:20-71

**Condiciones.**
- testNameFromCallStack sin clase en la pila: IllegalArgumentException(`Couldn't find [<cls>] in call stack`) (TestKitUtils.scala:53).

**Orden / concurrencia / ciclo de vida.** beforeAll: startCoroner + atStartup; afterAll: beforeTermination, shutdown(), afterTermination, stopCoroner (PekkoSpec.scala:107-117).

**Dependencias.** Filas: AC-TK-001.1 ; AC-TK-005.3 ; AC-TK-010. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** `tests/common::TestSystem` (harness de tests de rakka)

**Diferencias observables.** Rust: harness de tests (Q-TK-6). Diferencia: sin Coroner/ScalaTest.

**Aserciones.**
- A1: testConf = loggers TestEventListener, loglevel WARNING, stdout-loglevel WARNING, fork-join 8 hilos — testkit/src/test/scala/org/apache/pekko/testkit/PekkoSpec.scala:39-55.
- A2: scrubActorSystemName: `a.b.C$$anon$1` -> `C` y no alfanuméricos -> `_` — derivada de TestKitUtils.scala:64-70, *no afirmada por spec leída*.
- A3: muteDeadLetters publica Mute(DeadLettersFilter(clazz)(MaxValue)) sólo si el log no está en debug — testkit/src/test/scala/org/apache/pekko/testkit/PekkoSpec.scala:130-136.
- A4: afterAll apaga el sistema — testkit/src/test/scala/org/apache/pekko/testkit/PekkoSpec.scala:112-117.

**Oráculo.** No aplica (infra de test).

**Decisiones.** Q-TK-6

## AC-TK-012.2 — Infra restante de specs: Coroner, TestTags, WithLogCapturing, AbstractSpec, PekkoJUnitActorSystemResource, TestActorRefJavaCompile, metrics/*

*Unidad TK · visibilidad internal-required · analysis `listed`*

**Contrato observable.** [contractual para portar specs] Detección de deadlocks (Coroner), tags de tests (TimingTest), captura de logs, y MetricsKit para benchmarks internos. Pendiente.

**Fuente.**
- testkit/src/test/scala/org/apache/pekko/testkit/Coroner.scala
- testkit/src/test/scala/org/apache/pekko/testkit/WithLogCapturing.scala

**Condiciones.**
- Pendiente.

**Orden / concurrencia / ciclo de vida.** Pendiente.

**Dependencias.** Filas: AC-TK-012.1. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** helpers de `tests/common` (sólo los que un spec portado use)

**Diferencias observables.** Pendiente.

**Aserciones.**
- Pendiente de lectura.

**Oráculo.** No aplica.

**Decisiones.** Q-TK-6

## AC-TK-013 — Funciones de paquete: filterEvents(Iterable|varargs), filterException[T], TestDuration.dilated

*Unidad TK · visibilidad public · analysis `extracted`*

**Contrato observable.** [contractual por spec] ver AC-TK-005.2 (filterEvents/filterException) y AC-TK-004 (dilated). Ver AC-TK-005.2.

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/package.scala:28-70

**Condiciones.**
- Ver AC-TK-005.2.

**Orden / concurrencia / ciclo de vida.** Ver AC-TK-005.2.

**Dependencias.** Filas: AC-TK-005.1 ; AC-TK-004. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** funciones libres `filter_events`, `filter_exception`, trait `Dilate`

**Diferencias observables.** Ver AC-TK-005.2.

**Aserciones.**
- A1: filterEvents(EventFilter.warning(.., occurrences = 1)) { .. } — actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:77-79,90-91,103-104.

**Oráculo.** Jar oficial: `tk-intercept.jsh`.

**Decisiones.** Q-TK-4

## AC-TK-014 — TestKitUtils.testNameFromCallStack / scrubActorSystemName

*Unidad TK · visibilidad internal-required · analysis `extracted`*

**Contrato observable.** [contractual interno] Deriva un nombre de ActorSystem válido (AC-SYS-001.1) a partir de la pila de llamadas. En Rust no hay pila de clases; el nombre sale del nombre del test.

**Fuente.**
- testkit/src/main/scala/org/apache/pekko/testkit/TestKitUtils.scala:20-71

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

*Unidad TK · visibilidad internal-required · analysis `listed`*

**Contrato observable.** [contractual interno] reference.conf del testkit registra pekko.actor.serializers.java-test = TestJavaSerializer, serialization-identifiers id 23 y binding JavaSerializable -> java-test. Pendiente.

**Fuente.**
- testkit/src/main/resources/reference.conf:36-49

**Condiciones.**
- Pendiente.

**Orden / concurrencia / ciclo de vida.** Pendiente.

**Dependencias.** Filas: AC-SER-001. Externas: Typesafe Config / Scala stdlib / JVM según la fila (ver Fuente).

**Equivalente Rust propuesto.** serializador de test registrado por nombre (Q-X-2)

**Diferencias observables.** Pendiente.

**Aserciones.**
- Pendiente de lectura.

**Oráculo.** No aplica.

**Decisiones.** Q-X-2


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

Resumen de la API mínima transversal que un port 1:1 debe ofrecer antes de portar cualquier spec de otra unidad: (1) `TestKit`/`TestProbe` con `ref`, `send`, `reply`, `watch`/`unwatch`, `expectMsg`, `expectMsgType`, `expectMsgPF`, `expectMsgAllOf`, `expectNoMessage`, `expectTerminated`, `fishForMessage`, `receiveN`, `ignoreMsg`, `within`, `awaitCond`, `awaitAssert`, `lastSender`; (2) `EventFilter` (`error/warning/info/debug/custom`, `[E]`, `start`/`pattern`/`message`/`source`/`occurrences`, `intercept`) y `filterEvents`/`filterException`, junto con `TestEventListener` y `TestEvent.Mute/UnMute`; (3) `PekkoSpec.testConf` (loggers=TestEventListener, loglevel/stdout-loglevel=WARNING); (4) `TestKit.shutdownActorSystem`; (5) `TestActors.echoActorProps`, `TestLatch` (AC-TK-009, sin leer); (6) `ExplicitlyTriggeredScheduler` (AC-TK-008) para los specs de scheduler; (7) entrega síncrona del testActor (Q-TK-1, blocking).

## Descomposiciones

- AC-SYS-001 -> AC-SYS-001.1 (validación del nombre) y AC-SYS-001.2 (constructores/sobrecargas/nombre por defecto).
- AC-SYS-003 -> AC-SYS-003.1 (jerarquía de guardianes y mensajes ajenos) y AC-SYS-003.2 (protocolo de hooks de terminación: ningún test lo afirma).
- AC-EVT-013 -> AC-EVT-013.1 (formato `{}`/args), AC-EVT-013.2 (BusLogging/NoLogging) y AC-EVT-013.3 (LogSource y fábricas de adaptadores).
- AC-TK-001 -> AC-TK-001.1 (testActor/cola), .2 (expect*), .3 (expectNoMessage), .4 (receive*/fish*), .5 (within/remaining/await*).
- AC-TK-002 -> AC-TK-002.1 (API de probe), .2 (AutoPilot), .3 (childActorOf/DelegatingSupervisorStrategy).
- AC-TK-005 -> AC-TK-005.1 (reglas de coincidencia de EventFilter), .2 (intercept/filterEvents/occurrences), .3 (TestEventListener/Mute/UnMute).
- AC-TK-006 -> AC-TK-006.1 (TestActorRef) y AC-TK-006.2 (CallingThreadDispatcher).
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

- AC-SYS-001.2: Sobrecargas con ClassLoader no tienen equivalente directo (Q-X-4). Sólo se leyó el encabezado de las sobrecargas (ASY:193-330); falta leer el cuerpo de apply(name, bootstrapSetup/setup) y la fusión de config.
- AC-SYS-002: Se leyó la implementación; el cuerpo de ExtensionSpec:99,153 no se leyó, por lo que las aserciones son derivadas del código. Pasar a extracted tras leerlo.
- AC-SYS-010: No se leyó ActorSelectionSpec:162,245. Dependencia fuerte de TK-001.1 (testActor-N es systemActorOf).
- AC-SYS-011: No se leyó FaultHandling.scala:189-195 (efecto en hijos remotos). Sin spec.
- AC-SYS-012: No se leyó ASY:530-650 completo (accesores) ni uptime. Sólo se leyó toString (ASY:1262 es toString del ActorSystemImpl) y el test de redacción (ver AC-SYS-022).
- AC-SYS-013: printTree es depuración (accidental) pero la usan TestKit.shutdownActorSystem y ASS:318; formato leído en ASY:1264-1300. Falta leer 755-806.
- AC-SYS-016: Se leyó la implementación y LocalActorRefProviderSpec:132-198; SupervisorSpec:510-540 (ConfigurationException) no se leyó. Mensajes de InvalidActorNameException los fija ActorPath (AC-PATH).
- AC-SYS-017: No se leyó AskSpec:248,268 ni VirtualPathContainer (ActorRef.scala:763-860). La ruta del test es actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala.
- AC-SYS-018: No se leyó ARP:40-187 ni ProviderSelectionSpec. Hook principal hacia remote (fuera de alcance, pero el trait debe poder alojarlo).
- AC-SYS-020: Q-SYS-3: propuesta de equivalente (no exclusión). No se leyó ManifestInfo.scala ni su spec.
- AC-SYS-021: Q-SYS-3: propuesta de equivalente; no se leyó Main.scala.
- AC-EVT-013.3: Omisión derivada de dividir AC-EVT-013. No se leyó LOG:233-480 (LogSource) ni 573-705; sin spec asociado.
- AC-EVT-018: No se leyó Actor.scala:330-368.
- AC-EVT-019: Implementación leída (completa); LoggingReceiveSpec no se leyó, por lo que las aserciones son derivadas del código. Pasar a extracted tras leerlo.
- AC-EVT-021: Q-EVT-4: propuesta de equivalente, la fila sigue en alcance. No se leyó JavaLogger.scala ni su spec.
- AC-EVT-022: Omisión detectada: archivo sin ninguna cita en el inventario (uncited-classic-main.txt no lo listó pero el árbol event/japi sí lo contiene). Q-EVT-5. No se leyó.
- AC-TK-006.1: No se leyó TestActorRef.scala ni TestActorRefSpec. Q-TK-2: choque con el modelo de propiedad de Rust. Citas del inventario sin verificar.
- AC-TK-006.2: No se leyó CallingThreadDispatcher.scala (395 líneas) ni CallingThreadDispatcherModelSpec (uncited por el orquestador). Q-TK-1 (blocking) depende de esta fila.
- AC-TK-007: No se leyó (TestFSMRef.scala:147 líneas).
- AC-TK-009: No se leyeron. JavaSerializable y TestJavaSerializer pasan a AC-TK-015.
- AC-TK-011: No se leyó. Q-EVT-5 aplica por analogía (API Java sin equivalente propio).
- AC-TK-012.2: No se leyeron. metrics/* (MetricsKit, HdrHistogram, etc.) es infraestructura de benchmarks de la suite JVM: se justifica en Cobertura; no es comportamiento observable de actores.
- AC-TK-015: Omisión detectada (uncited-testkit-main.txt: JavaSerializable.scala, TestJavaSerializer.scala). No se leyeron. Depende de la política de serialización Java (AC-SER-004).

## Cobertura

Archivos de Pekko del área y filas que los cubren (rutas relativas a `upstream/pekko`).

**actor/src/main/scala/org/apache/pekko/actor**
- ActorSystem.scala: AC-SYS-001.1/.2, -002, -004 a -015, -022, -023 (Settings también toca AC-CFG; ver nota en AC-SYS-022).
- ActorRefProvider.scala: AC-SYS-003.1/.2, -008, -016 a -019, -024, -025.
- ActorRef.scala (partes de dead letters, líneas 573-756): AC-EVT-008; el resto del archivo pertenece a AC-REF.
- Actor.scala (ActorLogging/Unhandled/DiagnosticActorLogging, líneas 297-382): AC-EVT-010, -014, -018 (sin leer).
- ActorLogMarker.scala: AC-EVT-015 (sin leer).
- dungeon/Children.scala:310-316: AC-SYS-007. dungeon/FaultHandling.scala:189-195: AC-SYS-011 (sin leer).
- Main.scala: AC-SYS-021. util/ManifestInfo.scala: AC-SYS-020.

**actor/src/main/scala/org/apache/pekko/event**
- ActorClassificationUnsubscriber.scala: AC-EVT-005. AddressTerminatedTopic.scala: AC-DW-012 (fuera de este grupo; es el tópico de AddressTerminated para remote, sin test en classic). DeadLetterListener.scala: AC-EVT-009. EventBus.scala: AC-EVT-001 a -005. EventStream.scala: AC-EVT-006. EventStreamUnsubscriber.scala: AC-EVT-007. LoggerMailbox.scala: AC-EVT-020. Logging.scala: AC-EVT-011 a -017 (incl. .1/.2/.3). LoggingReceive.scala: AC-EVT-019. japi/EventBusJavaAPI.scala: AC-EVT-022. jul/JavaLogger.scala: AC-EVT-021.

**testkit/src/main/scala/org/apache/pekko/testkit**
- CallingThreadDispatcher.scala: AC-TK-006.2. ExplicitlyTriggeredScheduler.scala: AC-TK-008. JavaSerializable.scala, TestJavaSerializer.scala: AC-TK-015. SocketUtil.scala, TestActors.scala, TestBarrier.scala, TestException.scala, TestLatch.scala: AC-TK-009. TestActorRef.scala: AC-TK-006.1. TestEventListener.scala: AC-TK-005.1/.2/.3. TestFSMRef.scala: AC-TK-007. TestKit.scala: AC-TK-001.x, -002.x, -003, -010. TestKitExtension.scala: AC-TK-004. TestKitUtils.scala: AC-TK-014. package.scala: AC-TK-013, -004. javadsl/*: AC-TK-011.
- resources/reference.conf del testkit: AC-TK-004 (test.*), AC-TK-006.2 (calling-thread-dispatcher), AC-TK-015 (serializers).

**Claves de `actor/src/main/resources/reference.conf` del área (sólo las de este grupo)**: `pekko.loggers`, `logging-filter`, `loggers-dispatcher`, `logger-startup-timeout`, `loglevel`, `stdout-loglevel` (AC-EVT-011/016); `log-config-on-start` (AC-SYS-002); `log-dead-letters`, `log-dead-letters-during-shutdown`, `log-dead-letters-suspend-duration` (AC-EVT-009); `jvm-exit-on-fatal-error` (AC-SYS-015); `fail-mixed-versions` (AC-SYS-020); `pekko.actor.guardian-supervisor-strategy` (AC-SYS-008); `pekko.actor.debug.event-stream`/`unhandled`/`receive` (AC-EVT-006/010/019); `pekko.actor.mailbox.logger-queue` (AC-EVT-020); `pekko.coordinated-shutdown.close-actor-system-timeout` (AC-SYS-014). No se recorrió todo el reference.conf: las demás secciones (dispatchers, mailboxes, serialization, deployment, `creation-timeout`, `unstarted-push-timeout`, `daemonic`, `home`, `debug.autoreceive/lifecycle/fsm`) pertenecen a otras unidades (CFG, DSP, MBX, SER, CELL, FSM).

**Archivos de tests sin comportamiento observable propio (justificación)**
- testkit/src/test/scala/org/apache/pekko/testkit/metrics/* (MetricsKit, HdrHistogram, AveragingGauge, etc.): infraestructura de benchmarking de la suite JVM; no define comportamiento de actores (AC-TK-012.2).
- testkit/src/test/java/.../PekkoJUnitActorSystemResource.java, TestActorRefJavaCompile.java: boilerplate de compilación/JUnit (TestActorRefJavaCompile sólo verifica que compile la API Java).
- actor-tests/src/test/java/org/apache/pekko/event/ActorWithMDC.java: actor auxiliar de LoggingAdapterTest (cubierto por AC-EVT-014).
- actor-tests/src/test/java/org/apache/pekko/actor/setup/ActorSystemSetupTest.java: cubierto por AC-CFG-015 (sólo verifica que la API sea usable desde Java).
- actor-tests/src/test/scala-2.13+/org/apache/pekko/actor/ActorSystemScala213PlusSpec.scala: cubierto por AC-SYS-001.2 y AC-SYS-014 (`Using` termina el sistema).
- actor-tests/src/test/scala/org/apache/pekko/testkit/CallingThreadDispatcherModelSpec.scala: AC-TK-006.2 (sin leer).
