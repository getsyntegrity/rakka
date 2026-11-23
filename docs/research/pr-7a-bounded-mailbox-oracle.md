# PR 7a: oráculo y contrato de `BoundedMailbox` (Pekko 2.0.0)

Investigación **sin código de producción**: contratos verificados contra Apache Pekko 2.0.0 (`9fbd2283b1cf701c90a1223bf621942c191db61f`, JDK 17), escenarios JVM nuevos, golden y estabilidad. Rama `research/bounded-mailbox-7a`, base `1df22f2`. Cubre ADR-0001 §6 PR 7 (la parte `BoundedMailbox`; `NonBlockingBoundedMailbox`, AC-MBX-008, queda para 7b/otro PR) y las filas AC-MBX-007, AC-MBX-014 (`bounded-capacity:N`), AC-MBX-013/015 (selección y semántica de cola) en lo que toca a `BoundedMailbox`.

Reutiliza, sin duplicar: el oráculo del PR 1 (`docs/evidence/mailbox-oracle.md`, 47 ids `MBX-*`: estado, orden sistema/usuario, cierre, `CC-05` varado, rechazo de executor `RJ-*`, `UC-*`/`IF-*`/`AC-01` de `UnstartedCell`) y `BoundedBlockingQueue` de `src/util` (AC-UTIL-012, ya validado contra el oráculo en P03: no es esta cola; `BoundedMailbox.MessageQueue` es un `LinkedBlockingQueue`, ver C2 abajo).

## 1. Contratos verificados (Pekko 2.0.0, `path:line`)

Rutas relativas a `upstream/pekko/actor/src/main/scala/org/apache/pekko/` salvo indicación.

| # | Contrato | Cita |
|---|---|---|
| C1 | `BoundedMailbox(capacity: Int, pushTimeOut: FiniteDuration)` es una `case class` final. El constructor valida **sólo** `capacity < 0` (IAE `The capacity for BoundedMailbox can not be negative`) y `pushTimeOut eq null` (IAE `...push time-out ... can not be null`). **Capacidad 0 pasa** la validación. | `dispatch/Mailbox.scala:729-742` |
| C2 | `create` devuelve `BoundedMailbox.MessageQueue(capacity, pushTimeOut)`, que **extiende `java.util.concurrent.LinkedBlockingQueue[Envelope](capacity)`**: con capacidad 0 el `create` lanza `IllegalArgumentException` (mensaje `null`) del JDK, no el tipo de mailbox. Capacidad `Int.MaxValue` es válida (`remainingCapacity == Int.MaxValue`). | `Mailbox.scala:740-749`; oráculo `BM-CF-05`, `BM-CF-06` |
| C3 | Constructor desde configuración: `new BoundedMailbox(settings, config)` lee `mailbox-capacity` (`getInt`) y `mailbox-push-timeout-time` (`getNanosDuration`, es decir `FiniteDuration` en nanosegundos). | `Mailbox.scala:733-736` |
| C4 | Valores por defecto: `default-mailbox.mailbox-capacity = 1000`, `mailbox-push-timeout-time = 10s`, `mailbox-type = UnboundedMailbox` (la capacidad y el timeout existen aunque el tipo por defecto sea ilimitado); "negative values signify infinite timeout". Binding de requisito `BoundedMessageQueueSemantics` → `pekko.actor.mailbox.bounded-queue-based` → `BoundedMailbox`. | `resources/reference.conf:678-700,713-715,736-740` |
| C5 | **Enqueue** (`BoundedQueueBasedMessageQueue.enqueue`): si `pushTimeOut.length >= 0` → `queue.offer(handle, length, unit)` y, si devuelve `false`, `receiver.provider.deadLetters.tell(DeadLetter(handle.message, handle.sender, receiver), handle.sender)`; si `length < 0` → `queue.put(handle)` (espera **sin límite**). El signo se mira en `length` (no en la unidad). Timeout 0 ⇒ `offer(0)`: descarte inmediato. `dequeue` es `queue.poll()` (no bloquea). | `Mailbox.scala:581-593` |
| C6 | `dispatch`: `mbox.enqueue(receiver.self, invocation)` **antes** de `registerForExecution(mbox, hasMessageHint = true, ...)`; el bloqueo ocurre, pues, en el hilo del emisor y antes de intentar programar el turno. `systemDispatch` usa `systemEnqueue`, **no acotado**, y no mira la capacidad. | `dispatch/Dispatcher.scala:80-96` |
| C7 | `ActorCell.sendMessage` envuelve `dispatcher.dispatch` en `try ... catch handleException`; `InterruptedException` ⇒ Error `"interrupted during message send"` en el event stream y `Thread.currentThread().interrupt()` (el emisor interrumpido **no** ve la excepción). Otras `NonFatal` ⇒ Error `"swallowing exception during message send"`. | `actor/dungeon/Dispatch.scala:136-148,172-179` |
| C8 | Mailbox cerrado: `unregister` hace `swapMailbox(deadLetterMailbox)` → `becomeClosed()` → `cleanUp()`; `cleanUp` drena la cola de usuario (`queue.poll` repetido) hacia la cola del `deadLetterMailbox`, que publica `DeadLetter`. Un emisor que lee el mailbox **después** del swap encola en el `deadLetterMailbox` y **no bloquea nunca**, sea cual sea el `pushTimeOut`. | `dispatch/AbstractDispatcher.scala:279-285`, `Mailbox.scala:354-368,540-551`, `dispatch/Mailboxes.scala:51-72` |
| C9 | Mensajes de sistema: lista enlazada de sistema aparte (`systemEnqueue`), procesada antes que los de usuario y tras cada mensaje; no consume capacidad. | `Mailbox.scala:262-309,454-527`; oráculo `BM-AC-06` |
| C10 | El mensaje en proceso ya salió de la cola: la capacidad cuenta sólo los pendientes. | oráculo `BM-AC-02` |
| C11 | `Mailboxes.lookup(id)`: `"unbounded"` → `UnboundedMailbox`; `"bounded"` → `new BoundedMailbox(settings, config("bounded"))` (exige que exista la ruta `bounded`; en una config normal es `ConfigException.Missing`); prefijo `bounded-capacity:` → `BoundedMailbox(N, Duration.Zero)` (`id.split(':')(1).toInt`: sin número `ArrayIndexOutOfBounds`, no numérico `NumberFormatException`, `bounded-capacity:5:6` ⇒ 5); id configurado → instancia por FQCN con `(Settings, Config)`; fallo ⇒ IAE `Cannot instantiate MailboxType [...]` con causa. | `Mailboxes.scala:43,211-262` |
| C12 | **Aviso de timeout no cero**: un mailbox `ProducesPushTimeoutSemanticsMailbox` con `pushTimeOut.toNanos > 0` publica **una sola vez por sistema** (bandera `mailboxNonZeroPushTimeoutWarningIssued`, sin sincronizar: "don't care if this happens twice") un `Warning("mailboxes", ...)` con el texto exacto; cero y negativo no avisan ni gastan el aviso; `bounded-capacity:N` y `bounded` no pasan por esa rama. | `Mailboxes.scala:130,245-262`, `Mailbox.scala:574-579` |
| C13 | `UnstartedCell` (actor de nivel superior aún no iniciado) tiene cola propia **no acotada** y toma su lock con `tryLock(UnstartedPushTimeout)`; `replaceWith` mantiene el lock mientras entrega cada mensaje con `cell.sendMessage`, de modo que una cola acotada y llena **retiene el lock** hasta `pushTimeOut` y los emisores que esperan el lock caen a dead letters con un `Warning` (`...due to lock timeout`, texto con `"type" + class` sin espacio, tal cual). `reference.conf` lo dice: "only relevant if using a bounded mailbox". | `actor/RepointableActorRef.scala:212-300`, `reference.conf:141-145` |
| C14 | Specs de Pekko que fijan lo observable de la cola: `MailboxConfigSpec` ("create a bounded mailbox with 10 capacity and with push timeout", `testEnqueueDequeue` con timeouts −1/0/100 ms y 4 consumidores), `ActorMailboxSpec` (selección por dispatcher/mailbox/requisito, `pushTimeOut === Duration.Zero` en `default-bounded-mailbox-with-zero-pushtimeout`). | `actor-tests/.../dispatch/MailboxConfigSpec.scala:56-92,146-190`; `actor-tests/.../actor/ActorMailboxSpec.scala:332-344` |

### Hallazgos que contradicen o precisan lo supuesto

1. **Capacidad 0**: la acepta el tipo y la configuración (C1), la rechaza la cola (C2). Con `Props.withMailbox` de ese id, `actorOf` **vuelve** y el actor queda terminado sin arrancar (`BM-AC-13`: es el mismo camino "celda que no se puede hacer" del slice C1 de Q-MBX-11). Con capacidad negativa en configuración, `actorOf` **lanza** `ConfigurationException` con causa `IllegalArgumentException("Cannot instantiate MailboxType ...")` (`BM-AC-14`).
2. **Duración**: `getNanosDuration` trunca a nanosegundos; `5` sin unidad son 5 ms, `-1` son −1 ms, `1.5s` vale 1 500 000 000 ns, `infinite`, `abc`, `""` y `null` son `ConfigException` (`BM-CF-02`). `mailbox-capacity = 1.5` **se trunca a 1** (`getInt`), `2147483648` es `WrongType`, `"7"` (cadena) vale 7 (`BM-CF-03`).
3. `toString` del `BoundedMailbox` por prefijo es `BoundedMailbox(5,0 days)` (`Duration.Zero`), mientras que desde config un 0 es `0 nanoseconds`: una distinción de representación sin valor observable; el golden normaliza a nanosegundos.
4. **Emisor interrumpido** durante la espera: no ve la excepción (C7); el mensaje no se encola ni se envía a dead letters (`BM-AC-16`, `BM-QU-08`). No tiene análogo en Rust: ver §5.
5. **Ventana de cierre con emisor parado en la cola llena** (`BM-AC-11`): el `cleanUp` libera espacio mientras un emisor espera; el mensaje del emisor o va a dead letters (si entra antes del fin del drenaje) o queda **varado** en la cola vieja (si entra después). Es la misma carrera que Q-MBX-8 (aceptada: B+C), con la ventana ensanchada por el bloqueo. El caso del golden fija sólo el invariante (`procesado = [block]`, `dead + varado = 3`, los dos primeros en orden); no el reparto, que es no determinista.
6. **Starvation del pool** (`BM-AC-12`, `BM-UC-02`): en un pool de un hilo, un emisor que corre en ese mismo pool y llena la cola de otro actor del mismo pool no puede ser relevado: sólo el `pushTimeOut` termina la espera (paridad con el riesgo de ADR-0001 §3 B).

## 2. Valores límite cubiertos

| Variable | Casos | Resultado observado |
|---|---|---|
| capacity | 0, 1, 3, 1000, `Int.MaxValue`, −1, `Int.MinValue`, `2147483648`, `1.5`, `"7"`, `abc`, `null`, ausente | 0: ctor y `lookup` bien, `create` IAE `null`; −1/MinValue: IAE de Pekko; MAX: `remaining = MAX`; `1.5` → 1; resto: `ConfigException` |
| pushTimeOut | `0`, `0s`, `1ns`, `250ms`, `5`, `10s`, `2h`, `-1`, `-1ms`, `-2s`, `1.5s`, `infinite`, `abc`, `"10 parsecs"`, `""`, `null`, ausente; en ejecución: 0, 100 ms, 60 s, −1 ms | `>=0` ⇒ `offer` (0 inmediato); `<0` ⇒ `put` sin límite (estado `WAITING`, no `TIMED_WAITING`); no numéricos ⇒ `ConfigException.BadValue/Null/Missing` |
| `bounded-capacity:` | `5`, `1`, `0`, `Int.MaxValue`, `-1`, vacío, `x`, `5:6`, ` 7` | ver C11 |
| ids | `nope`, `bounded`, clase que no es mailbox, capacidad negativa en config, `mailbox-type = ""` | `ConfigurationException`, `ConfigException.Missing`, IAE+causa `ClassCastException`, IAE+causa IAE, `ConfigurationException` |

## 3. Escenarios JVM (`oracle/run.sh bounded-mailbox-all`, 44 casos)

Código: `oracle/src/main/java/pilot/BoundedMailboxOracle.java` (clase **nueva**; `Cli.java` sólo añade 3 `case`: `bounded-mailbox-ids|all|case`). Golden: `cases/oracle-only/bounded-mailbox.golden`; manifiesto (capa, filas, cita, instrumento por caso): `cases/oracle-only/bounded-mailbox-cases.tsv`; verificador: `tools/parity/check_bounded_mailbox_oracle.py` (`--runs N`, `--per-case`, `--record`, `--selftest`, 8 mutantes, debe salir con 1) con sus pruebas `tools/parity/test_check_bounded_mailbox_oracle.py`.

| Familia | Ids | Qué fija |
|---|---|---|
| `CF` (unitaria) | 01-07 | claves por defecto, tabla de timeouts, tabla de capacidades, claves ausentes, ctor de la case class, `create` con 0/1/3/MAX, tipo de la cola (`BlockingQueue`, `BoundedMessageQueueSemantics`) |
| `MB` (integración: `Mailboxes`) | 01-06 | aviso de timeout no cero (una vez, texto exacto), cero/negativo no avisan, `bounded-capacity:N`, errores de `lookup`, selección por requisito `BoundedMessageQueueSemantics`, `ProducesMessageQueue` |
| `QU` (cola sola) | 01-10 | llenado y rechazo con timeout 0, liberación de hueco y FIFO, capacidad 1, contenido del `DeadLetter` (mensaje, emisor del sobre, receptor; `noSender` ⇒ `deadLetters`; un `DeadLetter` rebosado **no** se descarta), espera ≥ timeout, desbloqueo por `dequeue` (`TIMED_WAITING`/`WAITING`), interrupción, `cleanUp` de cola llena, `cleanUp` con emisor parado |
| `QC` (productores concurrentes) | 01-02 | 4×50 con timeout 0 y nadie consume: aceptados = capacidad, el resto dead letters, orden por productor; con consumidor y timeout largo: sin pérdida, orden por productor, nunca por encima de la capacidad |
| `AC` (actores reales) | 01-16 | desborde con consumidor retenido; el mensaje en proceso no ocupa hueco; **backpressure**: emisor aparcado `TIMED_WAITING` y entrega completa en orden; 4 productores con timeout 0 y con timeout largo (un turno a la vez, `numberOfMessages <= capacidad` visto desde el actor); **mensajes de sistema** no acotados y adelantan a los de usuario; actor suspendido con cola llena; espera con timeout > 0 en actor suspendido; `stop` con cola llena (los pendientes a dead letters en orden); actor detenido con timeout −1: el emisor no bloquea; emisor parado cuando el actor se detiene (invariante); starvation del pool; capacidad 0 y negativa vía `Props.withMailbox`; selección por `mailbox-type` del dispatcher; emisor interrumpido |
| `UC` (`UnstartedCell`) | 01-03 | la cola del `UnstartedCell` se vuelca en la acotada al arrancar (`capacity` primeros, el resto dead letters en orden); el volcado corre en el hilo del actor que adjunta; un emisor que choca con el lock durante el volcado espera `unstarted-push-timeout` y su mensaje es dead letter con `Warning` |

Capa por caso (política de testing §1): `CF`/`QU`/`QC` son unitarias en el contrato (el JVM usa un sistema real sólo para tener `deadLetters`; el gemelo Rust usa un sumidero de dead letters declarado como frontera sustituida); `MB`/`AC`/`UC` son de integración.

### Ids existentes que ya cubren contratos (no se repiten)

| Contrato | Id existente |
|---|---|
| FIFO por emisor, sin pérdida, ejecución serial (cola ilimitada) | `MBX-CC-01`, `MBX-OR-01` |
| Sistema antes que usuario, suspensión | `MBX-OR-02`, `MBX-OR-03`, `MBX-ST-*` |
| Cierre, dead letters al detener, `deadLetterMailbox` | `MBX-OR-04`, `MBX-OR-05`, `MBX-CL-01`, `MBX-CC-04`, `MBX-DL-01` |
| Ventana de cierre (varado) | `MBX-CC-05`, `MBX-CC-05a`, `mailbox-stress` (los casos `BM-AC-11`/`BM-QU-10` sólo añaden la variante con cola llena) |
| Rechazo del executor | `MBX-RJ-01..07` |
| `UnstartedCell`, `attachChild` | `MBX-UC-01..04`, `MBX-IF-01..05`, `MBX-AC-01` |
| Cola `BoundedBlockingQueue` (otra clase) | P03 AC-UTIL-012 |
| `unstarted-push-timeout` con cola ilimitada | C3: `MBX-PT-01..06` (rama `slice/unstarted-push-timeout-c3`, **sin integrar**; no se ha leído su contenido más allá del título de los casos) |

## 4. Estabilidad y observables descartados

Estabilidad de la versión final del golden (44 casos): ver §8 (resultados exactos). Reglas de determinismo aplicadas:

- Sin `sleep`: sólo esperas por condición observable (`Thread.getState` del hilo bajo prueba, contador, mailbox ocioso, `fence` en el recolector de dead letters) con techo de 10 s.
- El reloj se observa sólo como "esperó al menos el timeout" (`offer` con `awaitNanos` garantiza `>=`); las cotas superiores son techos holgados (< 10 s o < 1,5 s con margen 10x), nunca valores.
- El consumidor que no debe correr se retiene con un mensaje `Block` en su turno: lo aceptado es función sólo de la capacidad.
- Concurrencia: sólo totales, orden por productor, cotas (`<= capacidad`), contabilidad "cada mensaje exactamente una vez".
- Los textos de `ConfigException` (origen, línea, rutas de jar de la máquina) se **eliden** (sólo la clase): son del dominio HOCON, ya cubierto por P07/P08, y dependen de la máquina.

Descartado por no determinista o no portable (no está en el golden):

| Observable | Motivo |
|---|---|
| Reparto exacto dead letter / varado de un emisor que despierta durante `cleanUp` (`BM-AC-11`, `BM-QU-10`) | depende de la carrera entre el bucle de drenaje y el emisor que despierta; el golden fija el invariante (`dead + varado = enviados`, ninguno procesado) |
| Orden relativo entre dead letters de productores distintos | intercalado |
| Duraciones medidas (más allá de `>= timeout`) | reloj |
| Qué productor ve su mensaje aceptado cuando compiten por los últimos huecos | intercalado; sólo el total |
| Orden de publicación `late` vs `m2` (`BM-UC-03`) | está en el golden porque la diferencia es 150 ms contra 2 s (margen >13x, 49 corridas, incluida carga de 6 JVM); si Rust lo reproduce con relojes distintos conviene que la comparación sea como conjunto (ver riesgos) |
| Frecuencia natural de varados con cola llena | `mailbox-stress` del PR 1 ya la caracteriza (~1 cada 2,8·10^7 envíos); no se repite |

## 5. Gemelo Rust: especificación (no hay tests en RED)

**No se prepararon tests Rust en RED.** Un twin necesita el tipo `BoundedMailbox` y una variante de `MessageQueue::enqueue` con receptor y salida a dead letters (la actual `enqueue(&self, envelope)` no tiene ni receptor ni camino a `deadLetters`), es decir, decidir API de producción; un archivo de test nuevo no compilaría sin tocar `crates/*/src`. Un RED por "no resuelve el import" sería evidencia de forma, no de comportamiento, y obligaría a inventar la API que debe decidir 7b. Queda como especificación (la comparación es contra el golden, **no** contra valores copiados a mano):

1. Unitarios (sin `ActorSystem`; sumidero de dead letters como frontera sustituida): `CF-02/03/04/05/06`, `QU-01..07`, `QU-09`, `QC-01/02` (hilos de prueba con barrera, sin sleeps; esperar por estado de un `Condvar`/contador).
2. Integración (actores reales): `MB-*`, `AC-01..15`, `UC-01..03`.
3. `QU-08` y `AC-16` (interrupción de hilo) y los estados `WAITING`/`TIMED_WAITING` no tienen análogo exacto en Rust: proponer `NOT_APPLICABLE` con motivo (Q-X-17), conservando la parte observable (el mensaje no se encola ni va a dead letters) mediante una API cancelable si Q-MBX-1 (c) elige `try_tell` awaitable.
4. Registro: los ids entran en `cases/oracle-ids/bounded-mailbox.txt` y en `FAMILIES` de `tools/parity/make_oracle_cases.py` (`bounded-mailbox`: `bm`) sólo cuando Rust implemente un caso (convención del PR 1: sólo se graban en `rust/app-cases` los ids que Rust produce; los demás quedan en la lista de pendientes). **No se grabaron casos `rust-anything app record` en este PR**: grabar un caso que Rust no produce haría divergir el replay completo. El golden `oracle-only` es la evidencia JVM (mismo mecanismo que `mailbox.golden`).

## 6. Dependencias

- **C3 (`UnstartedPushTimeout`, `slice/unstarted-push-timeout-c3`)**: `BM-UC-03` y C13 dependen de que `UnstartedCell` use `tryLock(unstarted-push-timeout)` y publique el `Warning` de `lock timeout`. C3 ya añade `MBX-PT-01..06` en `MailboxOracle.java`/`mailbox.golden` con cola ilimitada; `BM-UC-03` es la variante con el **lock retenido por una cola acotada** y no la duplica. El gemelo de `BM-UC-*` no puede validarse antes de que C3 esté integrado. `BM-UC-01/02` sólo exigen Q-MBX-11 A-C2 (ya integrados: #322, #323, #326, #327).
- **C4 (`research/unstarted-termination-c4`)**: sin commits propios al momento de escribir esto, no se ha podido leer. Contrato afectado, a coordinar: `stop` de un actor no iniciado cuyo volcado está bloqueado en una cola acotada llena (el `Terminate` va a la cola de sistema del `UnstartedCell`, se aplica tras el volcado y los mensajes pendientes del mailbox acotado terminan como dead letters en orden, como `BM-AC-09`). **No hay caso JVM** para eso en 7a; si C4 lo define, debe correr también con `bm` acotado.
- **Q-MBX-1 (proposed)**: bloquea la implementación Rust de AC-MBX-007; el oráculo no la necesita.
- **Q-MBX-8 (aceptada: B+C)**: aplica a `BM-AC-11`/`BM-QU-10`; el protocolo de cierre de 4a (`enqueue_unless_closed`) debe seguir cubriendo el emisor despertado.
- **PR 6a (throughput/deadline) y 6b/6c (rechazo de executor)**: ya integrados; 7a no los modifica.
- **PR 10 (`Mailboxes`)**: `MB-*` caen en su dominio (AC-MBX-013/014). Si 7b no puede llevar `Mailboxes.lookup`, `MB-*` se pasan a PR 10 sin cambiar el golden.

## 7. Decisiones nuevas propuestas (no registradas)

No se editaron `docs/owner-batch-02.md` ni los `*.decisions.tsv` (archivos compartidos con otros agentes) y no se tocó `owner-decisions.tsv`. Texto listo para registrar como `proposed`:

| Id | Pregunta | Recomendación |
|---|---|---|
| `Q-MBX-12` | Capacidad 0 en `BoundedMailbox`: Pekko la acepta en tipo y configuración y falla al **crear la cola** (IAE del JDK, `null`) | (a) paridad: el tipo y `lookup` aceptan 0, `create` falla con un error de configuración equivalente y `actorOf` termina el actor sin arrancar (camino C1 de Q-MBX-11); (b) rechazar 0 ya en el constructor (más claro, diverge). Recomienda (a). |
| `Q-MBX-13` | `getInt` trunca `mailbox-capacity = 1.5` a 1 y acepta `"7"` | sigue a la capa Config (ya portada, P08): no decidir en 7a; documentar como heredado. |
| `Q-MBX-14` | Interrupción del emisor bloqueado (C7, `AC-16`/`QU-08`): sin análogo en Rust | `NOT_APPLICABLE` por JVM-only salvo que Q-MBX-1 (c) cree una API cancelable. |

## 8. Resultados de estabilidad

Golden final (44 casos, JDK 17 `openjdk@17`, Mac de 16 núcleos con la carga normal de la sesión): **49 corridas de `bounded-mailbox-all` con salida byte a byte idéntica** (md5 único, 44 líneas cada una, stderr vacío):

| Condición | Corridas | Idénticas |
|---|---|---|
| secuenciales, una JVM | 25 | 25 |
| 3 JVM concurrentes × 6 rondas | 18 | 18 |
| 6 JVM concurrentes | 6 | 6 |
| `--per-case` (cada caso en su propia JVM) y `--runs 3` del verificador | 44 + 3 | todas iguales al golden |

Historial honesto: la primera versión (40 casos) y la segunda (43, con `UC-*`) también dieron 43 y 49 corridas idénticas antes de añadir `AC-16`; la primera versión imprimía mensajes de `ConfigException` con rutas de jar de la máquina y un `NullPointerException` en `QU-04` (sobre sin sistema); ambos se corrigieron antes de grabar (eliden el texto de `ConfigException`; el sobre usa el sistema del caso). No hubo ningún caso con salida distinta entre corridas tras esas correcciones: no se necesitó descartar ningún caso por inestabilidad, sólo se evitó diseñar los observables de §4.

## 9. Riesgos y huecos

1. `BM-UC-03` mide ventanas de 150 ms y 2 s sobre un `parkedInDrain()` que lee el stack de hilos de la JVM (diagnóstico de `Thread`, sin reflexión sobre Pekko): es la única observación que depende del nombre de método `replaceWith` de Pekko; si cambia en una actualización, el caso falla de forma ruidosa (`condition not reached`), no silenciosa.
2. Los estados `WAITING`/`TIMED_WAITING` son de la JVM 17 (`LinkedBlockingQueue` con `Condition`); en JDK 21 se espera igual pero **no se verificó** (la excepción JDK 21 de Q-X-16 es sólo NUM-RENDER/NUM-SHORT).
3. El mensaje de la causa `ClassCastException` de `BM-MB-04` y `Index 1 out of bounds for length 1` de `BM-MB-03` son textos de Pekko/JDK; el segundo es un texto del JDK (`ArrayIndexOutOfBoundsException`) que un twin Rust no reproducirá: usar `NOT_APPLICABLE` o comparar sólo la clase lógica.
4. El mecanismo de grabación `rust-anything app record` no se ejecutó (ver §5.4): el golden es `oracle-only`. Si el Owner exige casos grabados desde ya, habrá que registrar los ids como pendientes en `recorded_pekko.rs` (cambio de producción de pruebas, fuera de esta investigación).
5. `tools/package/check.py` falla en este worktree sólo por el submódulo `upstream/pekko` vacío (citas de atribución de `crates/`); no es consecuencia de estos cambios.
6. Sin ejecutar: `NonBlockingBoundedMailbox`, `BoundedDequeBased*`, `BoundedControlAware*`, `BoundedPriority*`, el stash acotado (AC-MBX-016): quedan para 7b y PR 8-9, con las mismas reglas.

## 10. Reproducción

```
mvn -f oracle/pom.xml package -DskipTests          # JDK 17, oracle/target no se versiona
oracle/run.sh bounded-mailbox-ids | wc -l            # 44
python3.12 tools/parity/check_bounded_mailbox_oracle.py --runs 3
python3.12 tools/parity/check_bounded_mailbox_oracle.py --per-case
python3.12 tools/parity/check_bounded_mailbox_oracle.py --selftest   # debe salir con 1
python3.12 -m unittest test_check_bounded_mailbox_oracle             # desde tools/parity
```

Archivos compartidos tocados (diff mínimo): `oracle/src/main/java/pilot/Cli.java` (+3 líneas, antes de `case "path-address-ids"`). No se tocaron `MailboxOracle.java`, `cases/oracle-case-ids.tsv`, `make_oracle_cases.py`, `recorded_pekko.rs` ni `docs/decisions*`.
