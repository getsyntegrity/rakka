# Propuesta de partición de P55 (T-SUPERVISION)

**Estado (2026-11-01): PROPUESTA para revisión. No aplicada.** No cambia `docs/unit-rows.tsv`, estados `impl`/`validation`, decisiones del Owner ni código. No aprueba el plan para implementar ni responde ninguna decisión. Las recomendaciones del registro siguen sin aprobar.

**Revisión 2 (tras la revisión del PR #6, base `8178fa4`):** corrige (1) la lectura de Q-X-8(b), (2) el análisis de la costura P49→P55 (AT-ADP-006.2/.3/.4) y (3) el alcance de C6 y el estado de AT-BEH-010. Sigue siendo provisional y no aplicada.

**Base revisada:** rama `docs/p55-partition-proposal` desde `origin/develop` = `590893c3946e9532d5422125c758f4a8bd5508d6` (squash del PR #5; `develop` no avanzó desde entonces). Upstream: `upstream/pekko` en `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (v1.7.1, verificado). P55 tiene **31 filas** (igual que el antecedente).

## 1. Resultado

Seis unidades en lugar de una (7 + 9 + 4 + 4 + 2 + 5 = 31). De las cuatro hipótesis anteriores se sostienen tres (supervisión base, AbstractBehavior, interoperabilidad) y se corrige una (backoff y stash):

| Hipótesis anterior | Veredicto | Evidencia |
|---|---|---|
| Supervisión base | **Se sostiene, acotada** a Stop/Resume + clasificación del fallo (S1). Restart deja de ser «base». | `Supervision.scala:44-63` (`Supervisor.apply` elige Restart/Resume/Stop); `SimpleSupervisor` (Stop y Resume) no tiene estado, `RestartSupervisor` sí (`:206-445`). |
| Backoff y stash de restart como un bloque inseparable | **Se corrige.** El stash NO es del backoff: es del restart (también del `Restart` sin backoff). Backoff es una extensión separable de Restart (S3). | `RestartSupervisor` es una sola clase para `Restart` y `Backoff` (`:206`); `restartingInProgress = (StashBuffer, hijos)` se crea en `prepareRestart` para ambos (`:366-373`); con `Restart` y hijos espera `Terminated` mientras apila (`:252-270`, `:388`). Backoff solo añade `calculateDelay`, `ScheduledRestart`, `ResetRestartCount` (`:185-203`, `:380-383`, `:394-400`). |
| AbstractBehavior y su relación con supervisión | **Se sostiene como unidad propia pequeña** (S5), pero la «dependencia mutua» no es real (ver §4). | §4.2 |
| Interoperabilidad Classic/Typed | **Se sostiene** (S6) y permanece **en alcance** con cualquier alternativa de Q-X-8; sus «ciclos» eran referencias entre fachada y contrato, y ADP-002/ADP-005 duplican aserciones de SUP-020/021. | §4.1, §9 |

Se añade una unidad que la hipótesis no tenía: **S4 composición y log** (SUP-010/011/012/019), cuyas aserciones solo pueden verificarse con varias estrategias a la vez.

## 2. Filas originales de P55 (31)

`AT-BEH-010, AT-BEH-024, AT-SIG-002, AT-SIG-005, AT-SIG-006, AT-SUP-001, AT-SUP-002, AT-SUP-003, AT-SUP-004, AT-SUP-005, AT-SUP-006, AT-SUP-007.1, AT-SUP-007.2, AT-SUP-007.3, AT-SUP-008, AT-SUP-009, AT-SUP-010, AT-SUP-011, AT-SUP-012, AT-SUP-013, AT-SUP-014, AT-SUP-016, AT-SUP-017, AT-SUP-018, AT-SUP-019, AT-SUP-020, AT-SUP-021, AT-SUP-022, AT-STASH-006, AT-ADP-002, AT-ADP-005`

(AT-SUP-015 no pertenece a P55 y no se revisó.)

## 3. Tabla fila → unidad propuesta

Los nombres S1..S6 son provisionales (los IDs `Pnn` los asigna el Owner al aplicar). «Aserciones»: **E** = afirmada por spec, **D** = derivada de la implementación, **N** = caracterización nueva propuesta.

| Fila | Comportamiento (leído en el cuerpo) | Fuente principal | Unidad | Nota de la revisión |
|---|---|---|---|---|
| AT-SUP-001 | Sin `supervise` un fallo detiene el actor; `PostStop`; padre observador recibe `ChildFailed`; sin doble log | `ActorAdapter.scala:232-273`, `SupervisionSpec:387-396`, `WatchSpec:100-128` | S1 | A1-A3 E, A4 D. La ruta sin `supervise` la implementa el decisor del adaptador (AT-ADP-006.4, P49); S1 la valida extremo a extremo (§4.3) |
| AT-SUP-002 | `supervise(b).onFailure[T](estrategia)`: selector por tipo, solo NonFatal, valida Behavior inicial, rechaza AbstractBehavior | `Supervision.scala:44-83`, `SupervisionSpec:139-147,214-222,1497-1525` | S1 | Selector = Q-T-SUP-1; la identidad del selector condiciona S4 (SUP-011) |
| AT-SUP-003 | `stop`: log «Supervisor StopSupervisor saw failure: …», devuelve `Failed`, padre recibe `ChildFailed` | `Supervision.scala:159-167`, `SupervisionSpec:397-418`, `WatchSpec:130-162` | S1 | A1-A3 E |
| AT-SUP-004 | `resume`: `same` (o Behavior de `UnstashException`); falla de arranque con resume detiene el actor | `Supervision.scala:169-178` | S1 | La parte de `UnstashException` se cierra con S2 (ver §6) |
| AT-BEH-024 | `Failed(cause)` no vivo; el adaptador lanza `TypedActorFailedException` o hace `stop` | `ActorAdapter.scala:164-172`, `BehaviorImpl.scala:71,94-96` | S1 | Productor (`StopSupervisor`) y validación en S1; la reacción del adaptador es AT-ADP-006.2 (P49). **Esta fila es la única que define `FailedBehavior`** (§4.3) |
| AT-SIG-005 | `ChildFailed(ref, cause)` solo para hijo directo observado; `toString` con nombre de clase | `MessageAndSignals.scala:96-119`, `ActorAdapter.scala:94-100,232-273` | S1 | Cierra con `supervise(stop)` + watch; API pública = Q-T-SIG-1 |
| AT-SIG-006 | `Terminated` no manejado → `DeathPactException("death pact with <ref> was triggered")` dentro de `interpretSignal`, supervisable | `Behavior.scala:270-277`, `MessageAndSignals.scala:18-27` | S1 | A1,A2,A3 E (A3 usa restart: ver SUP-014) |
| AT-SIG-002 | `PreRestart` al Behavior que falló; excepción del manejador se loguea y no impide el restart; sin `PostStop` | `Supervision.scala:332-351` | S2 | A1-A4 E |
| AT-SUP-005 | `restart` ilimitado: `PreRestart`, hijos parados, setup de nuevo; falla de arranque con restart ilimitado relanza | `Supervision.scala:294-311,313-330,332-419` | S2 | Q-T-BEH-1 (bloqueante) y Q-T-SUP-4 |
| AT-SUP-006 | `withLimit(n, ventana)`: contador/deadline, `0` detiene, evalúa antes de `PreRestart` | `Supervision.scala:215-218,332-338,430-441` | S2 | Validación temporal real: Q-T-TMR-4/Q-T-SUP-8 |
| AT-SUP-008 | `withStopChildren`: espera `Terminated`; revoca `watchWith` previo; setups que fallan | `Supervision.scala:220-275,353-376,421-428` | S2 | A1-A7 E (largo, 7 escenarios) |
| AT-SUP-009 | Stash de restart (`restart-stash-capacity`=1000 o `withStashCapacity`); lleno → `Dropped("Stash is full in [RestartSupervisor]")` | `Supervision.scala:120-124,220-292,365-376`, `reference.conf:22-27` | S2 | Aplica a Restart **y** Backoff; Q-T-STASH-1, Q-X-6 |
| AT-SUP-014 | `DeathPactException` supervisable con `restart` | `Behavior.scala:270-277` | S2 | Su única aserción usa `restart`; implementación depende solo de SIG-006 |
| AT-SUP-016 | `cancelAllTimers` en falla/restart; descarte de `TimerMsg` de encarnaciones previas | `Supervision.scala:297-299,332-335,394`, `TimerSchedulerImpl.scala:169-196` | S2 | Depende de TMR-006 (P53) |
| AT-SUP-018 | Falla durante unstash: `UnstashException(cause, behavior)`; `PreRestart/PostStop` al último Behavior desapilado; resto del stash se conserva | `StashBufferImpl.scala:187-244,268-287`, `ActorAdapter.scala:152-162` | S2 | Cubre Stop/Resume/Restart/Backoff; se cierra por completo con S3 (A3) |
| AT-STASH-006 | Texto y `unwrap` de `UnstashException` | `StashBufferImpl.scala:268-287` | S2 | Q-T-STASH-1 |
| AT-SUP-007.1 | `restartWithBackoff`: `min(max, min·2^n)·(1+rnd·factor)`; el factor aleatorio se aplica después del tope | `Supervision.scala:185-196,380-387` | S3 | Fórmula verificada contra el código |
| AT-SUP-007.2 | `resetBackoffAfter` (por defecto `(min+max)/2`) con `ResetRestartCount(current, owner)` | `Supervision.scala:198-203,241-250,392-402` | S3 | Default afirmado por spec `:1117-1120` |
| AT-SUP-007.3 | `withMaxRestarts`: solo `-1` es ilimitado; `0` y negativos distintos de `-1` fallan en la primera excepción (el Scaladoc dice lo contrario) | `SupervisorStrategy.scala:151`, `Supervision.scala:335`, `SupervisorStrategy.scala:329-333` | S3 | Q-T-SUP-3 (bloqueante) |
| AT-SUP-017 | `ScheduledRestart`/`ResetRestartCount` pasan al siguiente supervisor y al Behavior de usuario como señal opaca; `ScheduledRestart` sin restart → `IllegalStateException` | `Supervision.scala:198-203,220-250` | S3 | Q-T-SUP-6; ver §6 sobre el nombre de clase |
| AT-SUP-010 | Log `Supervisor <Clase> saw failure[ [n]]: msg`; niveles; `criticalLogLevel`; `loggingEnabled` | `Supervision.scala:97-118,353-356` | S4 | Aserciones de ≥3 estrategias; **corrección en §7 (C1)** |
| AT-SUP-011 | Anidado/plano; el más interno que coincide maneja; deduplicación `isSame` por clase de excepción | `Supervision.scala:70-83`, `InterceptorImpl.scala:106-123` | S4 | `isSame` depende de la identidad del selector (Q-T-SUP-1) |
| AT-SUP-012 | La supervisión persiste al cambiar de Behavior (re-wrap por el interceptor) | `InterceptorImpl.scala:90-123` | S4 | **Aserciones mal citadas: C2** |
| AT-SUP-019 | Un Behavior supervisado que es/queda `stopped` deja el actor parado con cualquier estrategia | `InterceptorImpl.scala:106-111` | S4 | A1-A3 recorren las cinco estrategias |
| AT-BEH-010 | `AbstractBehavior`: `onMessage` abstracto; `onSignal` con comportamiento por defecto, `this`≡`same`, contexto equivocado → `IllegalStateException`, contexto `null` → `IllegalArgumentException` | `AbstractBehavior.scala:21-105` | S5 | `onMessage` es abstracto (el usuario lo implementa); `onSignal` tiene default. A4 es la aserción de SUP-013. **Fila `partial` mientras Q-T-BEH-5 condicione A1-A5** (§5 S5, C6) |
| AT-SUP-013 | `supervise(AbstractBehavior)` → `IllegalArgumentException` con texto exacto | `Supervision.scala:44-51` | S5 | Dependencia de implementación unidireccional |
| AT-SUP-020 | Typed spawneado desde clásico: `supervise(stop)` y `rethrowTypedFailure=false` | `adapter/package.scala:49-137`, `ClassicSupervisingTypedSpec:74-160` | S6 | En alcance con Q-X-8 (a) y (b); cambia el mecanismo, no la obligación (§9) |
| AT-SUP-021 | Hijo clásico bajo padre typed: `defaultDecider` clásico, log ERROR | `ActorAdapter.scala:55-58,232-266` | S6 | En alcance con Q-X-8 (a) y (b) (§9) |
| AT-SUP-022 | `rethrowTypedFailure`: true para hijos de `ctx.spawn`, false para guardián/`systemActorOf`/spawns desde clásico | `ActorAdapter.scala:65,164-172,232-240` | S6 | El flag `rethrowTypedFailure` es el mecanismo de (a); el observable (causa en `ChildFailed`, guardián/`systemActorOf` terminan sin padre typed, un solo log) es obligatorio en ambas alternativas |
| AT-ADP-002 | `spawn/spawnAnonymous` desde clásico (envoltura Stop) | `adapter/package.scala:49-137` | S6 | A1-A4 repiten SUP-020 A1-A3 (C4) |
| AT-ADP-005 | `actorOf` de hijos clásicos desde typed; `watch/stop` cruzados | `adapter/package.scala:100-152` | S6 | A5,A6,A8 repiten SUP-021 (C4) |

## 4. Ciclos: cuáles eran reales

### 4.1 AT-ADP-002↔AT-SUP-020 y AT-ADP-005↔AT-SUP-021: **referencias entre fachada y contrato, no ciclos de implementación**
`ClassicSupervisingTypedSpec:74-160` es la única fuente de las aserciones de ambos pares: ADP-002 A1-A4 son SUP-020 A1-A3 con los mismos números de línea; ADP-005 A5/A6/A8 son SUP-021 A2/A3/A1 (`AdapterSpec:317-343`, `TypedSupervisingClassicSpec:60-75`). La dirección real de implementación es una sola: la envoltura `spawn` de ADP-002 **consume** `supervise(stop)` (S1) y `PropsAdapter` (P49). Misma unidad (S6), sin interfaz temporal; las aserciones se ejecutan una sola vez. Como `SUP-020 A2` usa `supervise(restart)`, S6 cierra después de S2.

### 4.2 AT-BEH-010↔AT-SUP-013: **dependencia de implementación unidireccional más una aserción compartida**
SUP-013 necesita que exista el tipo `AbstractBehavior` (`Supervision.scala:45-46` hace `isInstanceOf`); BEH-010 no necesita nada de supervisión. Su «A4» es literalmente el test que ya es SUP-013 A1/A2 (`SupervisionSpec:1527-1548`). BEH-010 A5 (`:1550-1573`) sí es solo de AbstractBehavior. Se mantienen juntas (S5) porque comparten ese test, no por un ciclo.

### 4.3 Costura P49→P55: AT-ADP-006.2, .3 y .4 analizados por separado
Leídos los tres bloques contractuales completos, `ActorAdapter.scala:41-66,90-100,150-180,228-362`, `BehaviorImpl.scala:60-100`, `scaladsl/adapter/PropsAdapter.scala:25-40` y los specs citados. **Conclusión: el marcador `Failed(cause)` solo explica una parte de 006.2; 006.3 y 006.4 se relacionan con P55 de otro modo.** Las costuras declaradas (`ADP-006.2>BEH-024`, `006.3>SUP-005`, `006.4>SUP-001`) no son las tres del mismo tipo.

| Fila P49 | Obligaciones propias (no relacionadas con P55) | Relación con P55 | Dirección y tipo | Evidencia | Interfaz temporal / integración / `partial` |
|---|---|---|---|---|---|
| **006.2** `next` | `Unhandled`→`unhandled` clásico; `Stopped`→`ComposedStoppingBehavior` + `stop` (PostStop al Behavior anterior y luego `stopped(cb)`); resto→`canonicalize` (`ActorAdapter.scala:164-179,339-362`). Dependen de BEH-003/004 (P52/P48), no de P55. | Solo la rama `Failed`: `rethrowTypedFailure` ⇒ `TypedActorFailedException(cause)`, si no ⇒ `context.stop(self)` (`:168-172`). | P49 **consume** el tipo `FailedBehavior` (implementación) que hoy define únicamente AT-BEH-024 (`typed-core.md:3415,3526`; `BehaviorImpl.scala:71,94-96`; `Behavior.scala:303-304`). Los *productores* están en P55: `StopSupervisor` (S1, `Supervision.scala:159-167`) y Backoff con `maxRestarts` agotado (S3, `:338-340`); Restart con límite agotado **relanza** (`:337`), no produce `Failed`. | `ActorAdapter.scala:164-172`; contrato BEH-024 | **Dependencia de implementación P49→(definición de `FailedBehavior`)**. Contrato mínimo que la evidencia permite fijar: valor construible desde una causa, **no vivo**, accesible su causa, e interpretarlo devuelve el mismo valor. P49 prueba la rama con un doble que lo construye; la integración (`WatchSpec:100-162` con `supervise(stop)`) se hace en S1. Sin esa definición P49 no puede compilar la rama. **Opción a decidir (no aplicada, requiere autorización de reasignación):** mover la parte de datos de BEH-024 al núcleo del Behavior (P48, donde viven `stopped`/`unhandled`); si no, el orden P49→P55 no puede cumplirse para esta rama. `006.2`/`BEH-024` `partial` hasta la integración. |
| **006.3** hooks | `preStart` (`start`+`validateAsInitial`), `preRestart` (cancela timers, `PreRestart`, `behavior=stopped`), `postRestart`, `postStop` (sin iniciar `Deferred`), `around*` y MDC (`:275-335`). | Contrato lista `AT-SUP-005` y sus A1/A2 citan `ClassicSupervisingTypedSpec:94-103` y `TimerSpec:174-200`. | **Hipótesis (C10): referencia documental y solape de validación, no dependencia de implementación; pendiente de verificar la ruta pública de reinicio.** Esos tests usan `Behaviors.supervise(...restart)`, es decir el `RestartSupervisor` tipado (`signalRestart`, `cancelAllTimers`: `Supervision.scala:298,334,394`), **no** los hooks del adaptador. Los hooks son el reinicio *clásico*; una búsqueda en `actor-typed`, `actor-typed-tests` y `actor-testkit-typed` no encuentra ningún test que los invoque explícitamente (lo que no prueba que no se ejecuten de forma indirecta). | `ActorAdapter.scala:309-323`; `grep` de `preRestart|postRestart` sin resultados en tests | Ninguna interfaz necesaria desde P55. **Incertidumbre registrada:** no se verificó si existe una ruta de API pública que haga reiniciar al `ActorAdapter` por un padre clásico (los spawns desde clásico envuelven en `supervise(stop)` con `rethrowTypedFailure=false`, `scaladsl/adapter/PropsAdapter.scala:33-36`). Hasta verificarlo, no se afirma que 006.3 A1/A2 cubran o no los hooks (hipótesis C10). |
| **006.4** estrategia del padre | `OneForOneStrategy(loggingEnabled=false)`; `TypedActorFailedException`⇒registrar causa (si se observa al hijo) y `Stop` sin log; otra excepción⇒registrar, log ERROR con el texto de la causa y `Stop` (hijo typed) o decisor clásico (hijo clásico) (`:232-273`). | (i) `SUP-001`: su ruta sin `supervise` y su A4 son este código; (ii) `SUP-006`: cuando el límite se agota la excepción llega aquí y se detiene con log; (iii) `SIG-005`/`BEH-024`: el padre produce `ChildFailed` desde `failures` (006.1); (iv) `SUP-021`/`ADP-005`: hijos clásicos. | **El decisor lo implementa P49; S1/S2 lo consumen y validan.** La costura declarada `006.4>SUP-001` apunta en sentido inverso al de la implementación: es **referencia documental** (SUP-001 describe el default Stop que 006.4 implementa). Para los hijos clásicos (iv) la obligación pertenece a S6. | `ActorAdapter.scala:232-273`; `SupervisionSpec:224-237` (límite agotado), `WatchSpec:100-162` | P55 no necesita interfaz hacia 006.4 porque P49 es anterior. Integración posterior: S1 (`WatchSpec`, ChildFailed con y sin `supervise(stop)`) y S2 (SUP-006 A1/A4). `SUP-001`, `SIG-005`, `BEH-024` quedan `partial` hasta ejecutarlas. |

**Efecto en la propuesta:** (1) las únicas obligaciones de P49 que dependen de código de P55 son la **definición del tipo `FailedBehavior`** (006.2) y, en sentido inverso, el decisor que S1/S2 consumen; (2) S1 no «implementa» la ruta sin `supervise`: la valida; implementa `Stop`/`Resume` y el productor de `Failed`; (3) el **orden entre S1–S6 no cambia**, pero queda condicionado: la definición de `FailedBehavior` debe existir antes de P49, y hoy solo la fija una fila de P55. Esa condición queda como **pendiente de decisión del Owner** (opción de mover la parte de datos de BEH-024) y no se aplica aquí.

### 4.4 Ciclos reales
**Entre S1–S6 no encontré ningún ciclo de implementación en las relaciones que leí.** Quedan **sin verificar**: `AT-ADP-006.3>AT-SUP-005` más allá de lo anotado arriba, `AT-CTX-018>AT-SUP-016` (P49→P55) y los bloques AT-ADP-006.1/.5-.8; no puedo afirmar su ausencia. Hay acoplamiento de **validación** entre S1 y S2 (SUP-014, SUP-004 con `UnstashException`) y entre S2/S3 y S4 (aserciones multi-estrategia), resuelto por el orden S1→S2→S3→S4. Hay un posible ciclo **P49↔P55 por `FailedBehavior`** si no se resuelve la definición del tipo (§4.3).

## 5. Unidades: resultado verificable, alcance, aserciones y validación

### S1 — T-SUPERVISION-FALLO-BASE (7 filas)
SUP-001, SUP-002, SUP-003, SUP-004, BEH-024, SIG-005, SIG-006.
- **Resultado:** `supervise(...).onFailure` aplica **Stop** y **Resume** por tipo de fallo (solo no fatal) y produce `Failed(causa)`; un `Terminated` sin manejar produce `DeathPactException`; y queda validado extremo a extremo que un fallo detiene el actor (`PostStop`, `ChildFailed` con la causa, un solo log). La ruta *sin* `supervise` y el registro de causas los implementa P49 (AT-ADP-006.4/006.1); aquí solo se validan (§4.3).
- **Exclusiones:** Restart/Backoff (S2/S3); log configurable (S4); mecanismo clásico (S6); reacción del adaptador (P49).
- **Validación:** tests portados `SupervisionSpec:129-147,214-222,387-418,1497-1525`, `WatchSpec:100-204`, `TerminatedSpec`, `ActorContextSpec:483-533`; oráculo (jshell) para `Stop`/`Resume` con fallo en mensaje y en setup. N propuestas: toString de `ChildFailed` (SIG-005 A4, D).

### S2 — T-SUPERVISION-REINICIO (9 filas)
SIG-002, SUP-005, SUP-006, SUP-008, SUP-009, SUP-014, SUP-016, SUP-018, STASH-006.
- **Resultado:** `Restart` con y sin límite/ventana: `PreRestart`, cancelación de timers, parada de hijos y espera de `Terminated`, apilado de mensajes y señales con capacidad y `Dropped`, nuevo setup y desapilado, incluida la falla durante el desapilado.
- **Por qué no se divide más:** el stash es parte de la espera de hijos de cualquier restart (SUP-008 A1: `GetState` queda apilado y se responde tras el restart), de modo que un restart sin stash no es equivalente a Pekko; dividir exigiría una interfaz temporal de stash sin contrato propio.
- **Exclusiones:** retardo/backoff (S3); aserciones multi-estrategia (S4).
- **Validación:** `SupervisionSpec:149-174,224-281,512-832,834-847,1090-1241`, `StashSpec:460-610`, `TimerSpec:174-230,327-404`; oráculo nuevo con 2 hijos de parada lenta.

### S3 — T-SUPERVISION-BACKOFF (4 filas)
SUP-007.1, SUP-007.2, SUP-007.3, SUP-017.
- **Resultado:** `restartWithBackoff`: retardo exponencial con jitter, `ScheduledRestart`, reinicio del contador (`ResetRestartCount`), tope `withMaxRestarts`, y paso de señales a supervisores anidados y al Behavior de usuario.
- **Validación:** `SupervisionSpec:882-1076,1117-1176`. Reloj/azar inyectables (Q-T-SUP-8/Q-T-TMR-4) para evitar tiempo real; una aserción D para la fórmula (`calculateDelay`).

### S4 — T-SUPERVISION-COMPOSICION-LOG (4 filas)
SUP-010, SUP-011, SUP-012, SUP-019.
- **Resultado:** anidado/plano con deduplicación, persistencia de la supervisión al cambiar de Behavior, `stopped` bajo cualquier estrategia, y log configurable (nivel, habilitación, nivel crítico tras N errores).
- **Motivo de separarla:** ninguna de sus aserciones puede ejecutarse con una sola estrategia (`SupervisionSpec:176-212,448-490,1270-1406,1441-1495,1661-1692`). Su mecanismo se implementa en S1-S3; aquí se cierra la composición.
- **Condicionada por:** Q-T-SUP-1 (si el selector es un predicado, `isSame` por clase no se puede reproducir y la deduplicación de SUP-011 cambia).

### S5 — T-ABSTRACTBEHAVIOR (2 filas)
BEH-010, SUP-013. Después de S1 (usa `supervise`) y de P54 (`MessageAdaptionFailure`).
Contrastado con `scaladsl/AbstractBehavior.scala:15-105`: `onMessage` es **abstracto** (lo implementa el usuario, no tiene default); `onSignal` tiene default (`PartialFunction.empty`) y `receiveSignal` lo completa con «`MessageAdaptionFailure(ex)` se relanza, el resto `unhandled`» (`:97-103`); devolver `this` o `same` conserva el Behavior (Scaladoc `:59,74`; ningún spec lo afirma por separado: **D/Scaladoc**); el constructor rechaza contexto `null` (`:48-51`) y `receive`/`receiveSignal` comprueban el contexto (`:82-88`).

| Comportamiento de AT-BEH-010 | Procedencia | Depende de Q-T-BEH-5 |
|---|---|---|
| Default de `onSignal`: `MessageAdaptionFailure` se relanza y el resto es `unhandled`; sin override el actor se detiene ante un fallo de adaptador, y sobrevive si lo maneja (A6, `AdaptationFailureSpec:66-127`) | E | **No** en lo observable. Su realización en Rust se apoya en la forma del trait, que es lo que Q-T-BEH-5 propone fijar; no se declara cerrado sin esa forma. |
| `onMessage` abstracto: el usuario debe implementarlo | D (`AbstractBehavior.scala:64`) | No |
| Devolver `this` ≡ `same` | D/Scaladoc | No |
| Contexto de otro actor → `IllegalStateException` con texto exacto (A1, A2, A5), incluida la repetición en `PostStop` | E | **Sí** |
| Contexto `null` → `IllegalArgumentException` (A4 segunda parte) | E | **Sí** |
| `supervise(AbstractBehavior)` rechazado (A4, = SUP-013 A1/A2) | E | **Sí** |
| Uso del contexto del padre en el constructor → `UnsupportedOperationException` (A3, `ActorThreadSpec:217-241`) | E | No: es **Q-T-CTX-2** (chequeo de hilo) |

- **Estado de las filas:** **AT-BEH-010 sigue `partial` mientras conserve comportamientos condicionados por Q-T-BEH-5 (A1, A2, A4, A5) o por Q-T-CTX-2 (A3)**; implementar y verificar A6 y las reglas D no la cierran. **AT-SUP-013 sigue `partial`** (su única conducta observable depende de Q-T-BEH-5). Esta propuesta no responde Q-T-BEH-5: si el Owner acepta la divergencia, es una exclusión que debe registrar por aserción.
- **Validación:** `ActorThreadSpec:188-277`, `SupervisionSpec:1527-1573`, `AdaptationFailureSpec:66-127`.

### S6 — T-SUPERVISION-INTEROP-CLASICA (5 filas)
ADP-002, ADP-005, SUP-020, SUP-021, SUP-022. Después de S2.
- **Resultado:** default Stop del typed spawneado desde clásico (override con `restart` respetado), `rethrowTypedFailure` según el origen, y `defaultDecider` clásico para hijos clásicos de padre typed.
- **Alcance:** las cinco filas siguen en alcance con Q-X-8 (a) **y** (b). Cambia el mecanismo que implementa los contratos, no su obligatoriedad: ver §9.

## 6. Dependencias y orden propuesto

Orden: **S1 → S2 → S3 → S4**; **S5** tras S1; **S6** tras S2. S5 y S6 son independientes entre sí y de S3/S4.

| Consumidor → proveedor | Tipo | Evidencia |
|---|---|---|
| S1 → P48 (BEH-004/007/011, SIG-001), P53 (BEH-013 `intercept`), P54 (SIG-004), P49 (adaptador), P52 (LOG-001) | implementación | `Supervisor.apply` se construye con `Behaviors.intercept` (`Supervision.scala:55-61`) |
| S2 → S1, P53 (STASH-001, TMR-006), P50/P52 (`Dropped` por EventStream) | implementación | `Supervision.scala:120-124`, `:366-373` |
| S3 → S2 | implementación | misma clase `RestartSupervisor` |
| S3 → P53 (TMR-001 `scheduleOnce`) | implementación | `Supervision.scala:385-387,398-399` |
| S4 → S1, S2, S3 | **validación** (mecanismo en S1-S3) | aserciones multi-estrategia |
| S5 → S1 | validación (SUP-013) y tipo `AbstractBehavior` | `Supervision.scala:45-46` |
| S6 → S1, S2, P49 (PropsAdapter, ADP-003), P21 (supervisión clásica) | implementación con Q-X-8 (a); con (b) las dependencias de la capa de interoperabilidad están **pendientes de definir** (§9) | `adapter/package.scala:49-137` |
| P49 (ADP-006.2) → definición de `FailedBehavior` (hoy solo en AT-BEH-024, S1) | implementación; **orden no resuelto** (§4.3) | `BehaviorImpl.scala:71,94-96` |
| P49 (ADP-006.3) → AT-SUP-005 | referencia documental / solape de validación; **no verificado** si hay ruta pública que ejerza los hooks | §4.3 |
| P49 (ADP-006.4) ↔ S1/S2 | P49 implementa el decisor; S1/S2 lo consumen y validan; la costura `006.4>SUP-001` es documental | `ActorAdapter.scala:232-273` |
| P61 (CTX-017, JAPI-001/002/003/004/006) → S1/S5/S6 | consumidores | `AT-CTX-017>AT-SUP-021`, etc. |

**Interfaz temporal (costura P49→P55):** solo está justificada por la evidencia para la rama `Failed` de AT-ADP-006.2: valor construible desde una causa, no vivo, que conserva la causa y cuya interpretación devuelve el mismo valor (`BehaviorImpl.scala:71,94-96`; `Behavior.scala:303-304`). No se define ninguna interfaz para 006.3 ni 006.4 porque la evidencia no muestra una dependencia de implementación desde P49 hacia P55. Prueba en P49 con un doble; integración en S1 (`WatchSpec:100-162`); `AT-ADP-006.2` y `AT-BEH-024` `partial` hasta entonces.

Que el grafo entre S1–S6 sea acíclico no demuestra que la partición sea implementable: la propuesta lo respalda con las flechas citadas, pero deja registradas como **no verificadas** las relaciones de §4.4, y como **abierta** la posición del tipo `FailedBehavior` respecto de P49.

## 7. Correcciones contractuales propuestas (no aplicadas)

- **C1 — AT-SUP-010, «Orden»:** dice que el log ocurre «antes del PreRestart en Restart». En `Supervision.scala:332-356` el log se emite dentro de `prepareRestart`, que se llama **después** de `signalRestart` (`:344`) para Restart **y** Backoff. Excepciones: Restart que alcanza el límite relanza sin loguear (`:337`); Backoff que alcanza `maxRestarts` loguea y devuelve `Failed` (`:338-340`). Propuesta: reescribir el orden y añadir N: «en `PreRestart` el log del fallo aún no se emitió».
- **C2 — AT-SUP-012, aserciones:** A1 cita `SupervisionSpec:1270-1307`, que verifica deduplicación (SUP-011 A5), y A2 cita `ActorContextSpec:255-279`, test que no cambia de Behavior (reinicia un contador). Ninguna prueba el re-wrap. La evidencia real, E: tests que hacen `IncrementState` (devuelve un Behavior nuevo) y luego `Throw` y siguen reiniciando: `SupervisionSpec:1032-1050`, `:912-958`. Re-wrap además D en `InterceptorImpl.scala:90-123`.
- **C3 — AT-BEH-010 A4:** es la aserción de AT-SUP-013 (A1/A2). Marcar como compartida y ejecutarla una vez.
- **C4 — AT-ADP-002 A1-A4 y AT-ADP-005 A5/A6/A8:** repiten SUP-020/021. Marcar como compartidas y ejecutarlas una vez en S6.
- **C5 — Decisiones de filas S3:** SUP-007.2 enlaza Q-T-SUP-3, pero sus cuatro aserciones están afirmadas por spec (`:1117-1120,991-1076,1463-1495`); Q-T-SUP-3 solo condiciona SUP-007.3. SUP-007.1 enlaza Q-T-SUP-2, cuya discrepancia ya está resuelta por lectura. Propuesta: anotar «parte resuelta» (no cambia estados).
- **C6 — Q-T-BEH-5 y AT-BEH-010 (precisado):** el registro dice que Q-T-BEH-5 «bloquea AT-BEH-010». Contrastado con `AbstractBehavior.scala`: solo los comportamientos que dependen de la comprobación de contexto (A1, A2, A4, A5) están condicionados por Q-T-BEH-5, A3 lo está por Q-T-CTX-2, y el default de `onSignal` (A6), `onMessage` abstracto y `this`≡`same` son observables bajo cualquier respuesta (ver tabla de S5). Eso **no** cierra la fila: AT-BEH-010 sigue `partial` mientras conserve esos comportamientos pendientes de decisión. La propuesta no responde Q-T-BEH-5.
- **C7 — AT-SUP-017 A1 usa el nombre de clase** (`getClass.getName should endWith("ResetRestartCount")`, `SupervisionSpec:1052`): en Rust no existe. Reformular la aserción observable como «el Behavior de usuario recibe exactamente una señal opaca interna cuando ningún supervisor es el dueño, y el actor sigue vivo»; sigue siendo Q-T-SUP-6.
- **C8 — Tres discrepancias Scaladoc↔código** (todas resueltas a favor del código, nada que decidir salvo ratificar): backoff «descarta mensajes» vs stash con `Dropped` (`SupervisorStrategy.scala:64,100` vs `Supervision.scala:120-124,266-270,282-286`); `resetBackoffAfter` por defecto «igual a minBackoff» (`SupervisorStrategy.scala:317-326`) vs `(min+max)/2` (afirmado por `SupervisionSpec:1117-1120`); `withMaxRestarts` «cero o negativo desactiva» (`:329-333`) vs solo `-1` (`SupervisorStrategy.scala:151`, `Supervision.scala:335`).

- **C9 — Cita de `unlimitedRestarts`:** el contrato de AT-SUP-007.3 y Q-T-SUP-3 citan `Supervision.scala:145-152`; esa función está en `SupervisorStrategy.scala:151` (`maxRestarts == -1`). Corregir la cita; el comportamiento descrito es correcto.
- **C10 — AT-ADP-006.3 A1/A2 (HIPÓTESIS PENDIENTE DE VERIFICAR):** citan `ClassicSupervisingTypedSpec:94-103` y `TimerSpec:174-200`, que usan `Behaviors.supervise(...restart)` (restart tipado). Que ningún test llame explícitamente a `preRestart`/`postRestart` del adaptador **no demuestra** que esos hooks no se ejecuten de forma indirecta. Hasta verificar la ruta pública de reinicio desde un padre clásico, no se corrige ni se descarta la procedencia de A1/A2: es una hipótesis, no un defecto confirmado (§4.3).
- **C11 — Definición de `FailedBehavior`:** solo AT-BEH-024 (P55) la cubre, pero P49 (AT-ADP-006.2) la consume (§4.3).
- **Fuera de alcance, registrado sin tarea:** no se auditó el resto de afirmaciones de AT-ADP-006.1 ni los bloques AT-ADP-006.5-.8.

## 8. Decisiones: qué condiciona inicio y cierre

Estado de todas: **sin respuesta** (Q-T-BEH-1 y Q-T-SUP-3 son bloqueantes; el resto, propuesta). «Puede avanzar» solo si la ficha lo afirma o el código lo demuestra.

| Decisión | Unidades | Impide comenzar | Impide cerrar | Se resuelve leyendo Pekko | Requiere al Owner |
|---|---|---|---|---|---|
| Q-X-5 (qué es un fallo supervisable) | S1, S2, S6 (SUP-001, 002, 022, BEH-024) | no (puede avanzar con (A) provisional) | sí: SUP-002 A2/A4 (solo NonFatal) | Pekko: solo NonFatal; fatales no se capturan | Qué fallos de Rust cuentan como fatales (`panic`, `Err`) |
| Q-T-SUP-1 (selector) | S1, S4 | SUP-002 (ficha) | SUP-002, SUP-011 | `isSame` compara clase (`:78-83`) | Forma del selector; si es predicado, qué pasa con la deduplicación |
| Q-T-SIG-1 (forma de `Signal`) | S1 | API pública de ChildFailed | SIG-005 | `ChildFailed extends Terminated`: un `case Terminated` coincide también con ChildFailed y `interpretSignal` lanza DeathPact para ambos | Escenario que cambia: handler que solo mira `Terminated` y recibe `ChildFailed` |
| Q-T-SUP-5 (resume + falla de arranque) | S1 | no | SUP-004 (forma del error) | texto y parada: `SupervisionSpec:1107-1115` | Forma del error (envoltura `ActorInitializationException`) |
| Q-T-BEH-1 (**bloqueante**) | S2, S5 | SUP-005 y SUP-013 (ficha) | SUP-005, SUP-013 | — | Forma de `Behavior` con setup re-ejecutable |
| Q-T-STASH-1 | S2 | SUP-009, SUP-018, STASH-006 (ficha) | las mismas | orden relativo de mensajes y señales apilados | Tipo interno |
| Q-T-SUP-4 | S2 | no (solo rama ilimitada de SUP-005) | rama ilimitada | `SupervisionSpec:779-832` afirma el reintento con límite | Solo si se quiere divergir (b) |
| Q-T-SUP-7, Q-T-CTX-4 | S2 (SUP-006), S6 | SUP-021 (ficha) | rama clásica | — | Depende de Q-X-8 |
| Q-T-SUP-2 | S2, S3 | no | no (registro) | **Resuelta** (código + `SupervisionSpec:882-936`) | Solo ratificar el registro |
| Q-T-SUP-3 (**bloqueante**) | S3 | SUP-007.3 | SUP-007.3 | Código: `0`/negativo≠-1 fallan en la 1.ª excepción. Falta confirmar con oráculo (no ejecutado) | Ratificar «seguir el código, no el Scaladoc» tras la medición |
| Q-T-SUP-6 | S3 | SUP-017 (ficha) | SUP-017 | El spec afirma que la señal llega al usuario | Nada si se sigue a Pekko; solo la representación opaca |
| Q-T-SUP-8, Q-T-TMR-4 | S2, S3 | no | validación determinista | — | Reloj/azar inyectables (arquitectura) |
| Q-X-7 | S4 | no | SUP-010 (backend de log) | — | Backend de logging |
| Q-X-6 | S2 | no | SUP-009 | `Dropped` por EventStream | Suscripción por tipo |
| Q-T-BEH-5 | S5 | BEH-010 A1, A2, A4, A5; SUP-013 | las mismas (la fila BEH-010 sigue `partial`) | — | Aceptar o no la divergencia (ver §10) |
| Q-T-CTX-2 | S5 | no | BEH-010 A1-A3 | — | Chequeo de hilo como compilación |
| Q-X-8, Q-T-ADP-3 | S6 | cómo se implementa la interoperabilidad (no si es obligatoria) | S6 completa | — | Alternativa (a) o (b); **una exclusión funcional sería una decisión adicional**, no una consecuencia de (b) (§9) |

## 9. Cómo cambia la propuesta según Q-X-8

La alternativa (b) registrada es «**runtime tipado nativo con interop aparte**» (`tools/package/generate.py`, Q-X-8; (a) es «replicar las capas de Pekko»). Elegir (b) **no autoriza exclusiones**: los contratos de interoperabilidad siguen siendo obligatorios.

- **S1–S5:** no cambian de alcance. Con (b) cambia el mecanismo con el que `ChildFailed` lleva la causa (hoy `TypedActorFailedException` hacia la estrategia del padre clásico: `ActorAdapter.scala:41-49,232-273`); el observable es el mismo.
- **S6 permanece en alcance con (a) y con (b)** (ADP-002, ADP-005, SUP-020, SUP-021, SUP-022). Cambia *qué* implementa cada contrato:
  - Con (a): el adaptador clásico (`ClassicActorSystemOps/ContextOps.spawn`, `PropsAdapter`, `actorOf` desde typed, `rethrowTypedFailure`).
  - Con (b): una **capa de interoperabilidad aparte** debe seguir satisfaciendo estas obligaciones observables, tomadas de los contratos:
    1. Un actor typed lanzado desde el mundo clásico (sistema, contexto, `PropsAdapter`) **se detiene por defecto** ante un fallo no supervisado (recibe `PostStop`, no `PreRestart`), un observador clásico recibe `Terminated`, y la estrategia clásica del padre no interviene; un `supervise(restart)` propio se respeta (SUP-020 A1-A5, `ClassicSupervisingTypedSpec:74-160`).
    2. Un padre typed supervisa a un hijo clásico con el decisor clásico por defecto (reinicia salvo `ActorInitializationException`, `ActorKilled`, `DeathPactException`) y registra el fallo en ERROR (SUP-021 A1-A3, ADP-005 A5/A6/A8).
    3. `watch`/`unwatch`/`stop` cruzados en ambos sentidos y `actorOf` de hijos clásicos desde un contexto typed (ADP-005 A1-A4, A7).
    4. El guardián y `systemActorOf` terminan sin notificar a un padre typed; un hijo de `ctx.spawn` informa la causa a su padre sin duplicar el log (SUP-022 A1-A3).
    5. Textos de error propios de estas rutas (p. ej. despliegue remoto, AT-ADP-007) siguen las reglas de sus filas.
  - **Dependencias de esa capa: pendientes de definir.** No se diseña aquí el runtime nativo ni se propone una alternativa aprobada; falta decidir sobre qué componentes (sistema clásico incluido) se apoyaría y, por tanto, qué unidades previas necesitaría.
- **Exclusión funcional:** si alguna de estas obligaciones debiera dejar de cumplirse, sería **una decisión adicional del Owner por fila y por aserción** (con el escenario concreto que cambia), nunca una consecuencia automática de Q-X-8(b). Mientras no exista, las cinco filas se mantienen en alcance y `partial`.

## 10. Preguntas concretas para el Owner

1. **Q-X-8:** (a) replicar las capas de Pekko o (b) runtime nativo con interop aparte. En ambos casos S6 sigue en alcance. Si con (b) se pretendiera que algún comportamiento de interoperabilidad deje de cumplirse, ¿cuál, con qué escenario concreto (p. ej. un actor typed lanzado desde un padre clásico con la estrategia clásica por defecto), y se registra como exclusión explícita?
2. **Q-X-5:** ¿qué fallos son «no fatales» (supervisables) y cuáles no se capturan? Escenario: `panic!` dentro de un handler bajo `restart`.
3. **Q-T-BEH-5:** ¿se acepta eliminar el error de contexto equivocado, el de contexto `null` y el rechazo de `supervise(AbstractBehavior)`? Escenario: pasar a un `AbstractBehavior` el contexto de otro actor. Mientras no se responda, AT-BEH-010 y AT-SUP-013 siguen `partial`.
4. **Q-T-SIG-1:** si `ChildFailed` y `Terminated` son variantes separadas, ¿un handler que solo trata `Terminated` debe también recibir `ChildFailed` y, si no lo trata, lanzar `DeathPactException`?
5. **Q-T-SUP-1:** forma del selector y qué ocurre con la deduplicación de SUP-011 si es un predicado.
6. **Q-T-SUP-3:** tras medir con el oráculo, ¿se ratifica seguir el código en `withMaxRestarts(0 | negativo)`?
7. **Posición de `FailedBehavior`:** ¿se autoriza mover la parte de datos de AT-BEH-024 al núcleo del Behavior (P48) para que P49 pueda consumirla, o se acepta otra solución? (§4.3).
8. **Autorización de reasignación:** aplicar esta partición en `docs/unit-rows.tsv`, `unit-plan.md` y derivados. No se aplicó nada.

## 11. Límites de lo efectivamente revisado

- **Confirmado por fuente (leído en cuerpo):** los 31 bloques contractuales de P55 y los de AT-ADP-006.2/.3/.4; `Supervision.scala:40-445`; `InterceptorImpl.scala:85-125`; `ActorAdapter.scala:41-66,90-100,150-180,228-362`; `BehaviorImpl.scala:60-100`; `scaladsl/adapter/PropsAdapter.scala:25-40`; `scaladsl/AbstractBehavior.scala:15-105`; `SupervisorStrategy.scala` (Scaladoc de backoff y `withMaxRestarts`); `reference.conf:22-27`.
- **Afirmado por spec (leído):** `SupervisionSpec` (índice y fragmentos `:876-960,1032-1076,1270-1307,1441-1462,1527-1573,1661-1692`); `ActorContextSpec:255-279`; `BehaviorSpec:215-228`; `WatchSpec:165-204`; `ClassicSupervisingTypedSpec:60-165` (nombres y expectativas); `TypedSupervisingClassicSpec:55-75`; `TimerSpec:174-200`; `AdapterSpec:60-110,317-330`; `AdaptationFailureSpec:66-127`.
- **No leí:** `StashSpec`, `ActorThreadSpec`, `TerminatedSpec`, el resto de `AdapterSpec` y `TimerSpec`; `StashBufferImpl.scala`, `TimerSchedulerImpl.scala`, `ActorContextAdapter.scala` completo; los bloques AT-ADP-006.1/.5-.8, AT-CTX-017/018 y AT-JAPI-* (solo su relación declarada). Las aserciones que dependen de ellos se toman del contrato.
- **Pendiente / no verificado:** si existe alguna ruta de API pública que haga reiniciar al `ActorAdapter` por un padre clásico (los hooks de 006.3); la costura `CTX-018>SUP-016`; la posición del tipo `FailedBehavior` respecto de P49 (decisión). No ejecuté el oráculo ni escenas `.jsh` (Q-T-SUP-3 y las aserciones N sin medir). Las dependencias de la capa de interoperabilidad con Q-X-8 (b) no están definidas.
- No revisé P16, las nueve filas pendientes de Q-X-9 ni el resto de las unidades. La atomicidad de S1-S6 no está acreditada: es una propuesta, no un plan ejecutable. Los controles mecánicos (31 filas, `check.py`, `unit_plan.py --check`) no demuestran la validez semántica de la propuesta.
