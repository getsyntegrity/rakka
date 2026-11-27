# Contención de `ThreadPool` (Q-MBX-16, opción (c) del Owner; el cambio de producción está en #456)
Carga: ping-pong de `MailboxConfigSpec.scala:279-329` (2.000.000 de mensajes, `throughput = 1`, 8 hilos). Una máquina compartida: Apple M4 Max (12 P + 4 E), 64 GiB, Darwin 27.2.0, rustc 1.100.0, perfil `test`. **Reproducir:** en la base y en la rama, `cargo test -p rakka-actor --test classic_single_consumer_mailbox --no-run` (instrumentado: `cargo llvm-cov --no-report -p rakka-actor --test classic_single_consumer_mailbox -- --list`); después `caffeinate -i python3.12 -I tools/perf/pingpong_stats.py --binary before=A --binary after=B --runs 30 --log runs.tsv` (gate: instrumentados y `--together`; contención: además `--background --burners 16`). Línea base previa (`2054d3bb`, n = 6, carga 10-26), mín/mediana/máx: sin límite 8,8/12,1/15,9 s; acotado 8,7/11,8/16,4 s.
## Mecanismo
`sample` (4 s): cada worker pasaba ~1100 de 1313 muestras en `__psynch_mutexwait` del `Mutex<Receiver>`; eso muestra dónde esperan los ociosos, no la causa. Barrido en una copia temporal instrumentada, test sin límite, sólo dos pasadas por celda (s):

| Variante \ hilos | 1 | 2 | 4 | 8 | 16 |
|---|---|---|---|---|---|
| antes (`mpsc` + `Mutex`) | 2,1/2,1 | 5,4/4,5 | 15,3/7,4 | 8,5/8,6 | 8,0/9,7 |
| `Condvar` simple | 2,0/2,0 | 3,0/3,2 | 6,5/9,2 | 9,4/11,9 | 11,8/7,9 |
| un despertar en vuelo (adoptada) | 2,8/2,9 | 4,8/3,2 | 5,3/3,7 | 5,5/4,1 | 4,9/6,1 |
| quien envía se queda el trabajo (descartada) | 1,9/2,5 | 2,1/1,8 | 2,6/1,7 | 2,5/1,8 | 2,0/1,8 |

Con 8 hilos y ~2,0 M de trabajos, antes → adoptada → descartada: sueños de un worker 743 k → 457 k → 18; cambios de worker 925 k → 212 k → 8; cambios de contexto involuntarios 2,49 M → 0,64 M → 2 k; `sys` 10,5 → 5,4 → 0,03 s (`Condvar` simple: 1,43 M, 972 k, 1,74 M, 14,1 s). **El coste dominante es el relevo entre hilos:** una parte grande de los envíos despierta a un worker dormido aunque el emisor vaya a quedar libre. Sin relevos el test tarda ~2 s: ni el bit `Scheduled`, ni la cola, ni `throughput = 1` en sí explican el resto. El cerrojo retenido en `recv` **no parece la causa por sí solo**: `Condvar` simple no mejora con 8 ni con 16 hilos; con 4 la comparación es débil (dos pasadas muy dispersas) y en ningún caso bastan dos pasadas para una cifra. La variante descartada es la más rápida, pero un turno bloqueado retendría lo que envió; haría falta robo de trabajo (unidad propia). Girar antes de dormir no se midió. Carga paralela (3 pasadas): 4 parejas 5,6-5,9 s antes y 4,3-5,0 s después; 64 parejas 3,7-4,0 y 3,1-3,6 s.
## Corrección y razonamiento
`RunQueue` (estado bajo un `Mutex`) + `Condvar`, sin dependencias nuevas. Se despierta sólo si hay trabajo, alguien duerme y **no hay otro despertar en vuelo**.
- Sin wakeups perdidos: al soltar el cerrojo, o la cola está vacía, o nadie duerme, o hay un despertar en vuelo. Sólo se duerme con la cola vacía; quien despierta borra la marca, mira la cola y despierta al siguiente **antes** de ejecutar su trabajo (las ráfagas despiertan en cadena). Un despertar espurio sólo añade un `notify`. Todo el estado va bajo el mutex (sin atómicos) y ningún cerrojo se mantiene al ejecutar o soltar un trabajo.
- Apagado: `closed` + `notify_all`; lo encolado se ejecuta y lo posterior se rechaza; `Drop` equivale a `shutdown`; el pool se construye antes de crear los hilos, así que un `spawn` fallido cierra a los ya creados. Una sola cola FIFO, como antes. Panics (Q-MBX-17 sigue abierta, sin cambios): el turno se atrapa en `cell.rs`, las tareas en el pool; si aun así muriesen todos los hilos, el pool rechaza y suelta lo encolado, como antes.
## TDD
Antes del cambio, 5 tests de guarda del pool en verde. `cargo test -p rakka-actor --lib thread_pool_tests` (RED 1-3 sobre `2054d3bb`):
- RED 1: `error[E0432]: unresolved import crate::dispatch::dispatcher::RunQueue`.
- RED 2 (despertar si alguien duerme): `no_second_worker_is_notified_while_one_is_waking`, `left: Ok(true)`, `right: Ok(false)`; 9 passed, 1 failed.
- RED 3 (sin `Drop`): `dropping_the_pool_ends_its_worker` (hoy `drop_and_shutdown_wake_every_sleeping_worker`), `left: Err(Timeout)`, `right: Err(Disconnected)`.
- RED 4 (revisión de #456): `a_failed_spawn_ends_the_workers_already_started`, `left: Err(Timeout)`, `right: Ok(())`; 9 passed, 1 failed. Verde al construir el pool antes de los hilos.
- **Caracterización, no RED** (escritos después del código): que `shutdown` y `Drop` despiertan a todos los workers dormidos, y que un pool que pierde su único worker (payload de panic cuyo `Drop` entra en panic) suelta lo encolado y rechaza.
- Script (`tools/gates/test_pingpong_stats.py`): RED `AssertionError: SystemExit not raised` con una ejecución que no corrió ningún test; verde al exigir al menos uno.
- Integración por API pública, también caracterización: `tests/classic_dispatcher_threads_lost.rs` (si muere el único hilo del dispatcher, lo encolado se suelta y lo posterior se rechaza); vale para la base y para #456.
- Sin tests de tiempo. Mutante `pool-notifies-while-a-worker-is-already-waking` registrado, **no ejecutado**.
## Antes y después
Antes = `c4338dd7`, después = `138b9044` (mismo algoritmo que #456); intercaladas, `caffeinate -i`, ningún otro build mío, **con batería** (sin modo de bajo consumo). Escenarios **concurrentes** (planificación no determinista): una ejecución no prueba nada. p95 por rango más próximo: el 29.º valor con n = 30, el 30.º con n = 31 (idle «después»), el 19.º con n = 20; con n = 19 y 18 coincide con el máximo. Sin afirmaciones sobre p99. Segundos: mín / mediana / p95 / máx. Las ejecuciones crudas están en `dispatcher-queue-contention-runs/` (columnas: versión, test, ronda, segundos, pasó, suspendida, carga a 1 min). Excluidas: las líneas 40-41 de `idle.tsv` (dos tandas mías solapadas; se añadió una ronda) y 3 de `contention.tsv` marcadas como suspendidas (cierre de la tapa).

| Condición (carga 1 min) | n | antes | después | fallos de 30 s |
|---|---|---|---|---|
| idle, test sin límite (3,5-5,8) | 30 / 31 | 7,60 / 7,69 / 7,81 / 7,88 | 2,98 / 3,03 / 3,09 / 3,10 | 0 |
| idle, test acotado (3,5-5,8) | 30 / 31 | 7,63 / 7,72 / 7,83 / 7,92 | 2,98 / 3,01 / 3,08 / 3,09 | 0 |
| gate: `llvm-cov`, ambos tests a la vez (4-7) | 30 | 11,70 / 11,84 / 12,00 / 12,32 | 6,63 / 6,71 / 6,79 / 6,83 | 0 |
| gate + 16 quemadores de CPU (23-31) | 20 | 9,33 / 10,07 / 10,82 / 10,95 | 6,97 / 7,50 / 7,80 / 7,89 | 0 |
| gate + 16 quemadores + QoS background (6-78); censurado: el test aborta a los 30 s | 19 / 18 | 30,03 / 30,11 / 32,15 / 32,15 | 30,07 / 30,14 / 31,02 / 31,02 | todas: 19/19 y 18/18 |

**Escenario original de #443** (`llvm-cov`, ambos tests a la vez, QoS background, sin quemadores), antes `c6d2eade` y después `fa783172` (#456), intercaladas, n = 10 por versión (basta para ver fallos y diferencias grandes; sin p95), con batería, carga 2,6-7,2; mín / mediana / máx: antes 17,13 / 17,58 / 18,15, después 11,43 / 11,52 / 12,92; 0 fallos. #443 falló con carga 44-50, que **no** se reprodujo aquí: no muestra qué pasaría con esa carga. Una sola máquina; no se generaliza.
**No medido:** con builds concurrentes (la carga de un gate compartido), en Linux, en máquinas de pocos núcleos ni en hardware de CI.
## Reincorporar los 30 s al gate (opción (b) de Q-MBX-16): propuesta Q-MBX-18 (`proposed`; decide el Owner)
**Nada de esta sección está autorizado ni en vigor:** no hay política general de rendimiento ni cambio de ningún límite de tiempo.
- 7,89 s es el máximo observado **fuera de QoS background**, en una sola máquina (margen ~3,8x; gate ~4,4x, idle ~10x; antes el gate tenía 2,4x, y #443 observó más de 30 s con carga ajena). En QoS background sin quemadores y con carga baja el máximo fue **12,92 s** (margen ~2,3x).
- Con QoS background **y** la CPU sobresuscrita (16 quemadores) ambas versiones fallan siempre. Sin quemadores y con carga baja ambas versiones pasan (arriba); con la carga que tenía #443 no se volvió a medir.
- Recomendación: **(a) de Q-MBX-18**, que es la opción (b) de Q-MBX-16: mantener el `within(30.seconds)` original en el gate habitual (sin fusionar su exclusión de #443), sin subirlo ni reintentar, y sin ejecutar el gate en QoS background.
- Si se aceptara, mediciones aparte: en todo PR que toque `dispatch/` o la planificación de `actor/cell.rs`, en cada MILESTONE y antes de un release; las ejecuta el autor o quien corra el gate; carga a 1 min < 8, `caffeinate -i`, este script antes/después si cambia el ejecutor; fila en este documento (fecha, SHA, máquina, carga, toolchain, perfil, tiempos, resultado) citada en el PR.
- Si se aceptara, una medición fallida bloquearía: que ese PR salga de borrador o se fusione, que el MILESTONE pase y que se publique el release, hasta corregirlo o hasta una decisión del Owner. No se reintenta hasta que pase.
