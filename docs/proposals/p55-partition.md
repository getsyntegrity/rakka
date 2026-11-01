# Propuesta de partición de P55 (T-SUPERVISION)

**Estado (2026-11-01): PROPUESTA para revisión. No aplicada.** No cambia `docs/unit-rows.tsv`, estados `impl`/`validation`, decisiones del Owner ni código. No aprueba el plan para implementar ni responde ninguna decisión. Las recomendaciones del registro siguen sin aprobar.

**Revisión 2 (tras la revisión del PR #6, base `8178fa4`):** corrige (1) la lectura de Q-X-8(b), (2) el análisis de la costura P49→P55 (AT-ADP-006.2/.3/.4) y (3) el alcance de C6 y el estado de AT-BEH-010. Sigue siendo provisional y no aplicada.

**Revisión 3 (2026-11-01, base `ae96f56b138f9c3cdabc313f31c161880e6760f2`):** resuelve con evidencia C10 (hooks del `ActorAdapter` desde un padre Classic), la arista `AT-CTX-018>AT-SUP-016` y la posición de `FailedBehavior`, y revisa S1–S6 (ciclos, atomicidad, costuras). Los resultados están en el **§12**; las afirmaciones de la rev. 2 que la evidencia refuta se corrigieron en su lugar y llevan la marca **[Rev. 3]**. Sigue siendo una propuesta: no se tocó `docs/unit-rows.tsv`, ni estados `impl`/`validation`, ni respuestas del Owner, ni contratos o TSV (los deltas están descritos, no aplicados). **Propuesta documental y evidencia: no aplica asignaciones ni autoriza implementación.** Los resultados del oráculo son observaciones de escenarios concretos (JDK 11), no garantías generales.

**Base revisada:** rama `docs/p55-partition-proposal` desde `origin/develop` = `590893c3946e9532d5422125c758f4a8bd5508d6` (squash del PR #5; `develop` no avanzó desde entonces). Upstream: `upstream/pekko` en `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (v1.7.1, verificado). P55 tiene **31 filas** (igual que el antecedente).

**Base de la rev. 3:** rama local `docs/p55-partition-review` creada desde `origin/develop` = `ae96f56b138f9c3cdabc313f31c161880e6760f2` (squash del PR #6; el árbol de `4ae938b`, último commit de `docs/p55-partition-proposal`, es idéntico). Upstream sin cambios: `git -C upstream/pekko rev-parse HEAD` = `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`.

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
| AT-SUP-003 | `stop`: log «Supervisor StopSupervisor saw failure: …», devuelve `Failed`, padre recibe `ChildFailed` | `Supervision.scala:159-167`, `SupervisionSpec:397-418`, `WatchSpec:130-162` | S1 | A1-A3 E. **[Rev. 3]** depende en la implementación de AT-SUP-010 (S4 en rev. 2): `StopSupervisor` llama a `log` (`Supervision.scala:97-118,159-167`); ver §12.4 |
| AT-SUP-004 | `resume`: `same` (o Behavior de `UnstashException`); falla de arranque con resume detiene el actor | `Supervision.scala:169-178` | S1 | La parte de `UnstashException` se cierra con S2 (ver §6) |
| AT-BEH-024 | `Failed(cause)` no vivo; el adaptador lanza `TypedActorFailedException` o hace `stop` | `ActorAdapter.scala:164-172`, `BehaviorImpl.scala:71,94-96` | S1 | Productor (`StopSupervisor`) y validación en S1; la reacción del adaptador es AT-ADP-006.2 (P49). **[Rev. 3]** el valor `Failed` y sus invariantes ya están contratados en P48 (AT-BEH-001/011/012); A1 (sin `supervise`) no ejerce `FailedBehavior` (§12.3) |
| AT-SIG-005 | `ChildFailed(ref, cause)` solo para hijo directo observado; `toString` con nombre de clase | `MessageAndSignals.scala:96-119`, `ActorAdapter.scala:94-100,232-273` | S1 | Cierra con `supervise(stop)` + watch; API pública = Q-T-SIG-1 |
| AT-SIG-006 | `Terminated` no manejado → `DeathPactException("death pact with <ref> was triggered")` dentro de `interpretSignal`, supervisable | `Behavior.scala:270-277`, `MessageAndSignals.scala:18-27` | S1 | A1,A2,A3 E (A3 usa restart: ver SUP-014) |
| AT-SIG-002 | `PreRestart` al Behavior que falló; excepción del manejador se loguea y no impide el restart; sin `PostStop` | `Supervision.scala:332-351` | S2 | A1-A4 E |
| AT-SUP-005 | `restart` ilimitado: `PreRestart`, hijos parados, setup de nuevo; falla de arranque con restart ilimitado relanza | `Supervision.scala:294-311,313-330,332-419` | S2 | Q-T-BEH-1 (bloqueante) y Q-T-SUP-4 |
| AT-SUP-006 | `withLimit(n, ventana)`: contador/deadline, `0` detiene, evalúa antes de `PreRestart` | `Supervision.scala:215-218,332-338,430-441` | S2 | Validación temporal real: Q-T-TMR-4/Q-T-SUP-8 |
| AT-SUP-008 | `withStopChildren`: espera `Terminated`; revoca `watchWith` previo; setups que fallan | `Supervision.scala:220-275,353-376,421-428` | S2 | A1-A7 E (largo, 7 escenarios) |
| AT-SUP-009 | Stash de restart (`restart-stash-capacity`=1000 o `withStashCapacity`); lleno → `Dropped("Stash is full in [RestartSupervisor]")` | `Supervision.scala:120-124,220-292,365-376`, `reference.conf:22-27` | S2 | Aplica a Restart **y** Backoff; Q-T-STASH-1, Q-X-6 |
| AT-SUP-014 | `DeathPactException` supervisable con `restart` | `Behavior.scala:270-277` | S2 | Su única aserción usa `restart`; implementación depende solo de SIG-006 |
| AT-SUP-016 | `cancelAllTimers` en falla/restart; descarte de `TimerMsg` de encarnaciones previas | `Supervision.scala:297-299,332-335,394`, `TimerSchedulerImpl.scala:169-196` | S2 | Depende de TMR-006 (P53). **[Rev. 3]** consume `cancelAllTimers` de AT-CTX-018 (P49); la fuente cita los hooks de ADP-006.3 pero sus aserciones solo ejercen el restart tipado (§12.1, §12.2) |
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
| **006.2** `next` | `Unhandled`→`unhandled` clásico; `Stopped`→`ComposedStoppingBehavior` + `stop` (PostStop al Behavior anterior y luego `stopped(cb)`); resto→`canonicalize` (`ActorAdapter.scala:164-179,339-362`). Dependen de BEH-003/004 (P52/P48), no de P55. | Solo la rama `Failed`: `rethrowTypedFailure` ⇒ `TypedActorFailedException(cause)`, si no ⇒ `context.stop(self)` (`:168-172`); `next` no reemplaza `behavior` por el `Failed`, de modo que `postStop` entrega `PostStop` al Behavior vigente (`:325-335`; sección «Orden» de BEH-024). | **[Rev. 3 — corrige rev. 2]** P49 consume un *valor* que P48 ya contrata: variante `Failed` (AT-BEH-001, `typed-core.md:11,21`), «Failed devuelve sí mismo» al interpretar (AT-BEH-011, `Behavior.scala:303-304`) e `isAlive` falso (AT-BEH-012, `Behavior.scala:248-250`). Los *productores* están en P55: `StopSupervisor` (S1, `Supervision.scala:159-167`) y Backoff con `maxRestarts` agotado (S3, `:338-340`); Restart con límite agotado **relanza** (`:337`). | `ActorAdapter.scala:164-172`; `BehaviorImpl.scala:40,71,94-96`; contratos BEH-001/011/012/024 | **Sin dependencia de implementación P49→P55:** P49 prueba la rama con el valor real de P48, no con un doble. La integración (`WatchSpec:130-162`, `ClassicSupervisingTypedSpec:74-93` con `supervise(stop)`) se hace en S1. Ninguna aserción de P48 cubre hoy `Failed` (BEH-004 A1 es solo `Stopped`; BEH-012 A1-A4 no lo usan): se proponen aserciones N (§12.3). La arista `ADP-006.2>BEH-024` es referencia documental. |
| **006.3** hooks | `preStart` (`start`+`validateAsInitial`), `preRestart` (cancela timers, `PreRestart`, `behavior=stopped`), `postRestart`, `postStop` (sin iniciar `Deferred`), `around*` y MDC (`:275-335`). | Contrato lista `AT-SUP-005`; sus A1/A2 citan `ClassicSupervisingTypedSpec:94-103` y `TimerSpec:174-200`. | **[Rev. 3 — C10 resuelta, §12.1]** Referencia documental y solape de validación, **no** dependencia de implementación: los hooks usan `Behavior.start/interpretSignal/validateAsInitial` (BEH-007/011/012) y `cancelAllTimers` (CTX-018), no `RestartSupervisor`. A1/A2 ejercen el supervisor tipado (oráculo S-A: `PreRestart` entregado desde `RestartSupervisor` sin consultar al padre). Los hooks **sí** son alcanzables por API pública (padre Classic con estrategia `Restart` + excepción que sale del Behavior, p. ej. `Kill`: oráculo S-C/S-E/S-F) y ningún test de Pekko los ejerce. | `ActorAdapter.scala:309-323`; `tools/oracle/adapter-classic-restart.jsh`; `docs/proposals/evidence/p55-c10-run1.txt` | Sin interfaz desde P55. ADP-006.3 necesita aserciones N propias (§12.1); A1/A2 se reetiquetan como validación compartida con SUP-005/SIG-002/SUP-016. |
| **006.4** estrategia del padre | `OneForOneStrategy(loggingEnabled=false)`; `TypedActorFailedException`⇒registrar causa (si se observa al hijo) y `Stop` sin log; otra excepción⇒registrar, log ERROR con el texto de la causa y `Stop` (hijo typed) o decisor clásico (hijo clásico) (`:232-273`). | (i) `SUP-001`: su ruta sin `supervise` y su A4 son este código; (ii) `SUP-006`: cuando el límite se agota la excepción llega aquí y se detiene con log; (iii) `SIG-005`/`BEH-024`: el padre produce `ChildFailed` desde `failures` (006.1); (iv) `SUP-021`/`ADP-005`: hijos clásicos. | **El decisor lo implementa P49; S1/S2 lo consumen y validan.** La costura declarada `006.4>SUP-001` apunta en sentido inverso al de la implementación: es **referencia documental** (SUP-001 describe el default Stop que 006.4 implementa). Para los hijos clásicos (iv) la obligación pertenece a S6. | `ActorAdapter.scala:232-273`; `SupervisionSpec:224-237` (límite agotado), `WatchSpec:100-162` | P55 no necesita interfaz hacia 006.4 porque P49 es anterior. Integración posterior: S1 (`WatchSpec`, ChildFailed con y sin `supervise(stop)`) y S2 (SUP-006 A1/A4). `SUP-001`, `SIG-005`, `BEH-024` quedan `partial` hasta ejecutarlas. |

**Efecto en la propuesta [Rev. 3]:** (1) **no hay dependencia de implementación de P49 hacia código de P55**: `FailedBehavior` ya forma parte de P48 (§12.3), 006.3 no depende de SUP-005 (§12.1) y 006.4 es referencia documental; (2) S1 no «implementa» la ruta sin `supervise`: la valida; implementa `Stop`/`Resume`, el log del fallo y el productor de `Failed`; (3) el **orden entre S1–S6** queda condicionado por los deltas del §12, no aplicados. La rev. 2 dejaba pendiente de decisión «mover la parte de datos de BEH-024 a P48»: la evidencia de la rev. 3 la sustituye por la recomendación de §12.3.

### 4.4 Ciclos
**[Rev. 3 — corrige rev. 2]** La rev. 2 afirmaba no encontrar ciclos de implementación «en las relaciones que leí» y dejaba otras sin verificar. Calculando las aristas `depends_on` fila→fila con la asignación de §3 aparecen **dos ciclos de dos unidades**: S1↔S2 (`AT-SIG-006>AT-SUP-014` y `AT-SUP-014>AT-SIG-006`) y S1↔S6 (`AT-BEH-024>AT-SUP-022` y `AT-SUP-020/022>AT-SUP-001/003/BEH-024`), más la arista S1→S4 `AT-SUP-003>AT-SUP-010` (`docs/proposals/evidence/p55-unit-edges.out`). Su clasificación y resolución están en §12.4. Quedan **sin verificar**: los bloques AT-ADP-006.1/.5-.8, `AT-BEH-007>AT-SUP-001` (P48→P55) y `AT-BEH-003>AT-SIG-006` (P52→P55) (§12.4, C13).

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
| P49 (ADP-006.2) → P48 (BEH-001/011/012: valor `Failed`) | implementación, **ya dentro de P48** [Rev. 3]; la arista `006.2>BEH-024` es documental | `BehaviorImpl.scala:40,71,94-96`; `Behavior.scala:248-250,303-304` |
| P49 (ADP-006.3) → AT-SUP-005 | referencia documental / solape de validación **[Rev. 3: verificado, §12.1]**; los hooks son alcanzables y requieren aserciones propias | `tools/oracle/adapter-classic-restart.jsh` |
| P49 (ADP-006.4) ↔ S1/S2 | P49 implementa el decisor; S1/S2 lo consumen y validan; la costura `006.4>SUP-001` es documental | `ActorAdapter.scala:232-273` |
| P61 (CTX-017, JAPI-001/002/003/004/006) → S1/S5/S6 | consumidores | `AT-CTX-017>AT-SUP-021`, etc. |
| S2 (SUP-016) → P49 (CTX-018 `cancelAllTimers`) | implementación [Rev. 3]; la arista declarada `CTX-018>SUP-016` va en sentido inverso | `Supervision.scala:298,334,394`; `ActorContextImpl.scala:128-131` |
| S1 (SUP-003) → SUP-010 (S4 en rev. 2) | implementación [Rev. 3], no registrada en rev. 2: `StopSupervisor` llama a `log` | `Supervision.scala:97-118,159-167` |
| S1 ↔ S2 (SIG-006 ↔ SUP-014); S1 ↔ S6 (BEH-024 ↔ SUP-022) | ciclos de aristas; V/documental (§12.4) | `p55-unit-edges.out` |

**Interfaz temporal (costura P49→P55) [Rev. 3]:** ya **no está justificada** por la evidencia: el valor `Failed` es parte de P48 (§12.3), 006.3 y 006.4 no dependen de código de P55 y `CTX-018>SUP-016` va en sentido inverso. No se define ninguna interfaz temporal ni doble; `AT-ADP-006.2` y `AT-BEH-024` quedan `partial` hasta la integración con `supervise(stop)` en S1 (`WatchSpec:130-162`).

**[Rev. 3]** Con la asignación de §3 el grafo de aristas fila→fila **no es acíclico** (§4.4, §12.4); lo es solo tras los deltas propuestos en §12, que no están aplicados. Un grafo acíclico tampoco demuestra atomicidad ni implementabilidad: solo describe aristas declaradas; la justificación de cada unidad (§12.4) es un juicio propio, no una prueba.

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
- **C10 — AT-ADP-006.3 A1/A2 (RESUELTA en la rev. 3, §12.1):** citan `ClassicSupervisingTypedSpec:94-103` y `TimerSpec:174-200`, que ejercen el restart *tipado* (`RestartSupervisor`), no `ActorAdapter.preRestart/postRestart` (oráculo S-A; causa `RuntimeException` desde el handler, estrategia `supervise(restart)` interna). Los hooks son alcanzables por API pública (padre Classic con `Restart` y excepción que sale del Behavior: S-C, S-E, S-F) y ninguna aserción portada los ejerce. Delta propuesto (no aplicado): reetiquetar A1/A2 como validación compartida con AT-SUP-005/AT-SIG-002/AT-SUP-016 y añadir aserciones N (PreRestart sin PostStop y nuevo `setup`; composición con `supervise(restart)`; `TimerMsg` obsoleto tras restart clásico).
- **C11 — Definición de `FailedBehavior` (reformulada en la rev. 3, §12.3):** el valor ya está contratado en P48 (AT-BEH-001 variante `Failed`, AT-BEH-011 «Failed devuelve sí mismo», AT-BEH-012 `isAlive` falso) pero ninguna aserción lo cubre; AT-BEH-024 describe producción y notificación y mezcla la reacción del adaptador (AT-ADP-006.2). Delta propuesto: §12.3.
- **C12 — Arista `AT-CTX-018>AT-SUP-016` (rev. 3, §12.2):** invertir para `cancelAllTimers` (SUP-016 consume a CTX-018); ningún otro comportamiento de CTX-018 se relaciona con SUP-016. Delta: quitar `AT-SUP-016` de `depends_on` de AT-CTX-018 y añadir `AT-CTX-018` a AT-SUP-016.
- **C13 — `DeathPactException` (rev. 3, sin resolver):** AT-BEH-011 (P48) contrata que `interpretSignal` lance `DeathPactException`, cuyo tipo y texto solo define AT-SIG-006 (S1, `MessageAndSignals.scala:18-27`); AT-BEH-011 no declara esa dependencia y AT-BEH-003 (P52) sí (`AT-BEH-003>AT-SIG-006`). Es el mismo patrón que `FailedBehavior`; no se analizó con la misma profundidad (§12.4).
- **C14 — Otras aristas (rev. 3, §12.4):** añadir `AT-SUP-013>AT-SUP-002` (el rechazo vive en `Supervisor.apply`, `Supervision.scala:44-51`); retirar `AT-BEH-024>AT-SUP-022` (documental) y `AT-ADP-006.2>AT-BEH-024` (sustituida por BEH-001/011/012); marcar `AT-SIG-006>AT-SUP-014` como validación; BEH-024 A1 es compartida con SUP-001/SIG-005 A2 (no ejerce `FailedBehavior`).
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
7. **[Rev. 3] Deltas del §12** (aserciones N para `Failed` en P48 en lugar de mover BEH-024; SUP-010 a S1; SUP-018 y STASH-006 a S4; S6 en S6a/S6b; inversión de `CTX-018>SUP-016`; aristas de C14): ¿se autorizan? Sustituye a la pregunta anterior sobre mover la parte de datos de BEH-024 a P48. Ver también D-3 en §12.5 (reinicio del adaptador por un padre Classic).
8. **Autorización de reasignación:** aplicar esta partición en `docs/unit-rows.tsv`, `unit-plan.md` y derivados. No se aplicó nada.

## 11. Límites de lo efectivamente revisado

- **Confirmado por fuente (leído en cuerpo):** los 31 bloques contractuales de P55 y los de AT-ADP-006.2/.3/.4; `Supervision.scala:40-445`; `InterceptorImpl.scala:85-125`; `ActorAdapter.scala:41-66,90-100,150-180,228-362`; `BehaviorImpl.scala:60-100`; `scaladsl/adapter/PropsAdapter.scala:25-40`; `scaladsl/AbstractBehavior.scala:15-105`; `SupervisorStrategy.scala` (Scaladoc de backoff y `withMaxRestarts`); `reference.conf:22-27`.
- **Afirmado por spec (leído):** `SupervisionSpec` (índice y fragmentos `:876-960,1032-1076,1270-1307,1441-1462,1527-1573,1661-1692`); `ActorContextSpec:255-279`; `BehaviorSpec:215-228`; `WatchSpec:165-204`; `ClassicSupervisingTypedSpec:60-165` (nombres y expectativas); `TypedSupervisingClassicSpec:55-75`; `TimerSpec:174-200`; `AdapterSpec:60-110,317-330`; `AdaptationFailureSpec:66-127`.
- **No leí (rev. 2; sigue vigente salvo lo indicado en la rev. 3):** `StashSpec` (rev. 3: solo `:630-675`), `ActorThreadSpec`, `TerminatedSpec`, el resto de `AdapterSpec` y `TimerSpec` (rev. 3: `AdapterSpec:112-200,300-375`, `TimerSpec:160-235,322-410`); `StashBufferImpl.scala`, `TimerSchedulerImpl.scala` (rev. 3: `:34-75,160-196`), `ActorContextAdapter.scala` completo (rev. 3: `:54,125-150`); los bloques AT-ADP-006.1/.5-.8 y AT-JAPI-* (solo su relación declarada). Las aserciones que dependen de ellos se toman del contrato.
- **Pendiente / no verificado (rev. 3):** C10, la costura `CTX-018>SUP-016` y la posición de `FailedBehavior` quedaron resueltas en §12; siguen pendientes la costura `DeathPactException` (C13), `AT-BEH-007>AT-SUP-001`, `AT-BEH-003>AT-SIG-006`, las dependencias de la capa de interoperabilidad con Q-X-8 (b) y el oráculo de Q-T-SUP-3 y de las aserciones N de P55 (solo se ejecutó el escenario de §12.1, con JDK 11).
- No revisé P16, las nueve filas pendientes de Q-X-9 ni el resto de las unidades. La atomicidad de S1-S6 no está acreditada: es una propuesta, no un plan ejecutable. Los controles mecánicos (31 filas, `check.py`, `unit_plan.py --check`) no demuestran la validez semántica de la propuesta.

## 12. Revisión 3: resultados de las verificaciones pendientes

Clases de evidencia: **[código]** lectura de `upstream/pekko` @ `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`; **[spec]** lo que afirma un test de Pekko; **[oráculo]** salida ejecutada de los jars oficiales 1.7.1; **[propuesta]** juicio propio. Ningún delta de este apartado está aplicado.

### 12.1 C10: reinicio del `ActorAdapter` desde un padre Classic

Leídos completos: AT-ADP-006.3 (`typed-extended.md:480`) y AT-SUP-005 (`typed-core.md:2339`); `ActorAdapter.scala:36-365` (en particular `next` `:164-179`, estrategia `:232-273`, hooks `:275-335`); `ClassicSupervisingTypedSpec.scala` entero; `TimerSpec.scala:160-235,322-410`; `AdapterSpec.scala:112-200,300-375`; `AdapterTest.java:190-260,320-365`.

**Dos reinicios distintos [código]:**

| | Reinicio por `Behaviors.supervise(...).onFailure(restart)` | Reinicio del actor Classic que aloja el Behavior |
|---|---|---|
| Disparo | excepción `NonFatal` dentro del Behavior; `RestartSupervisor` la captura (`Supervision.scala:332-351`) | excepción que **sale** del Behavior y un padre Classic cuya estrategia devuelve `Restart` |
| Mecanismo | `cancelAllTimers` (`:334`), `signalRestart` → `PreRestart` (`:344`), `restartCompleted` (`:394`) sobre la **misma** instancia de `ActorAdapter`, el mismo `ActorContextAdapter` y el mismo `TimerSchedulerImpl` | la celda crea una instancia **nueva** del adaptador (`FaultHandling.scala:92`, `faultRecreate`), llama `ActorAdapter.preRestart` (`:309-315`) en la vieja y `postRestart` (`:317-323`) en la nueva |
| Hooks `preRestart`/`postRestart` | no se invocan: el actor no falla para la celda | se invocan |

**Qué excepciones llegan al padre Classic [código]:** todo spawn desde Classic envuelve el Behavior en `supervise(stop)` con `rethrowTypedFailure = false` (`scaladsl/adapter/package.scala:60,74,114,127`; `javadsl/Adapter.scala:161-163`; `scaladsl/adapter/PropsAdapter.scala:33-36`), que captura toda `NonFatal` y produce `Failed` → `context.stop(self)` (`ActorAdapter.scala:168-172`): la estrategia del padre **no se consulta** para fallos del handler. Sí llegan las que ocurren fuera del Behavior, p. ej. `Kill` (`ActorCell.scala:575` lanza `ActorKilledException` antes de interpretar el mensaje). El decisor por defecto del padre Classic detiene ante `ActorKilledException`/`ActorInitializationException` (`actor/FaultHandling.scala:219-220`); un padre typed (`ActorAdapter`) decide siempre `Stop` para hijos typed (`:232-273`). Por tanto solo un padre Classic con estrategia propia que devuelva `Restart` alcanza los hooks.

**Tests citados, causa y estrategia [spec + oráculo]:**

| Aserción | Test | Causa | Estrategia | Ruta ejercida |
|---|---|---|---|---|
| ADP-006.3 A1 | `ClassicSupervisingTypedSpec:94-103` (y variante `context.spawn`, `:117-128`) | `TestException("oh dear")` lanzada por el handler `"throw"` | `supervise(restart)` del usuario dentro del `supervise(stop)` del spawn | `RestartSupervisor` (oráculo S-A: `via=[typed:RestartSupervisor …]`) |
| ADP-006.3 A2 | `TimerSpec:174-200`, `:202-232` | `Exc` tras `SlowThenThrow` | `supervise(restart)` | `RestartSupervisor`; mismo `TimerSchedulerImpl` (oráculo S-A: `timerActive=false` en `PreRestart`) |
| — | `AdapterSpec:130-141`, `:159-165` (`Classic2`, «TODO Restart will not really restart the behavior») | `ThrowIt3` | decisor Classic `Restart` | **ninguna**: `supervise-restart` solo se envía al padre typed `typed1` (`:327`; `AdapterTest.java:349`), nunca a `Classic2` |

Búsqueda: `preRestart|postRestart` solo aparece en `ActorAdapter.scala:309,317` (incluyendo `actor-typed-tests`, `actor-testkit-typed`, `persistence-typed`, `cluster-typed`, `cluster-sharding-typed`); `Kill|ActorKilledException|OneForOneStrategy|SupervisorStrategy.Restart` en `actor-typed-tests`, `actor-testkit-typed/src` y `actor-typed/src/test` solo aparece en `AdapterSpec:130-141` y `AdapterTest.java:206,231`. La ausencia de llamadas explícitas no se tomó como prueba: se ejecutó el escenario siguiente.

**Oráculo [oráculo].** Escenario `tools/oracle/adapter-classic-restart.jsh` (nuevo). Comando: `JAVA_HOME=~/.sdkman/candidates/java/11.0.32-tem PATH=$JAVA_HOME/bin:$PATH ORACLE=jars tools/oracle/run.sh tools/oracle/adapter-classic-restart.jsh`. Jars verificados por `tools/oracle/fetch-jars.sh` (SHA-1 fijados). Dos corridas con salida idéntica (`diff` sin diferencias): `docs/proposals/evidence/p55-c10-run1.txt`, `p55-c10-run2.txt`. Acreditan que ese resultado se repitió en esas dos ejecuciones con esas temporizaciones; **no** acreditan otros escenarios, causas ni interleavings. Los archivos de evidencia no se editaron. La señal registra su propia pila (`via=`) para observar quién la entrega. Padre Classic con estrategia «`Restart` a todo» salvo S-D.

| Escenario | Causa | Estrategia del Behavior | Observado |
|---|---|---|---|
| S-A | `RuntimeException` del handler | `supervise(restart)` | `PreRestart` vía `internal.RestartSupervisor`; `setup#2`; el decisor del padre **no** se invoca; vivo |
| S-B | `RuntimeException` del handler | ninguna (envoltura `stop`) | `PostStop` vía `ActorAdapter.postStop`; el padre ve `Terminated`; el decisor **no** se invoca |
| S-C | `Kill` | ninguna | decisor del padre invocado (`ActorKilledException` → `Restart`); `PreRestart` vía `classic-hook:ActorAdapter.preRestart`; **sin** `PostStop`; `setup#2`; `timerActive=false`; vivo |
| S-D | `Kill` | ninguna; padre con estrategia por defecto | `PostStop`; `Terminated` (control: sin `Restart` del padre no hay reinicio) |
| S-E | `Kill`, hijo creado con `actorOf(Adapter.props(..))` | ninguna | igual que S-C |
| S-F | `Kill` | `supervise(restart)` | igual que S-C; la pila muestra `RestartSupervisor` **y** `ActorAdapter.preRestart` (la señal del hook atraviesa el supervisor tipado) |
| S-G | `Kill` con un `TimerMsg` ya encolado; la nueva incarnación registra un timer con la misma clave | ninguna | el `TimerMsg` obsoleto **no** llega al Behavior (descartado por `interceptTimerMsg`, `TimerSchedulerImpl.scala:176-179`; inferido: el oráculo solo muestra que no se entrega) |
| S-H | Secuencia exacta: `armBlock` arranca un timer de 50 ms y bloquea el handler 150 ms; ~20 ms después se encola `Kill`; el `TimerMsg` queda en el buzón detrás de `Kill`; el padre decide `Restart`; la nueva incarnación **no** registra timers | ninguna | En las 2 corridas con JDK 11 la incarnación #2 registró `received org.apache.pekko.actor.typed.internal.TimerSchedulerImpl$TimerMsg` (línea `S-H … incarnation#2 (no timer) received …`). La explicación por código (`hasTimer` falso en el contexto nuevo; `ActorAdapter.scala:128` solo intercepta con timer) es **inferida de la lectura**, no observada por separado. Solo vale para esta secuencia y estas temporizaciones |

**Veredicto sobre la hipótesis C10:**
- **Confirmada (parte):** A1 y A2 de ADP-006.3 no ejercen los hooks del adaptador; ejercen el supervisor tipado. Son evidencia de AT-SUP-005/AT-SIG-002/AT-SUP-016 (S2).
- **Descartada (parte):** «no se conoce ruta pública». Existe: padre Classic con `Restart` + excepción que sale del Behavior (S-C, S-E, S-F). Los hooks son comportamiento observable sin ninguna aserción portada.
- **Dirección:** ADP-006.3 → AT-SUP-005 no es dependencia de implementación (los hooks usan BEH-007/011/012 y `cancelAllTimers`, no `RestartSupervisor`): referencia documental y solape de validación.
- **Resultado observado (S-H), sin generalizar:** en el escenario S-H (secuencia descrita en la tabla) un `TimerMsg` de la incarnación anterior llegó al Behavior de usuario de la incarnación #2, que no había creado timers. No se establece como garantía general ni se probó variando temporizaciones, causas ni número de timers. `TimerSpec:327-370` (SUP-016 A3) solo ejerce el restart tipado, donde el `TimerSchedulerImpl` es el mismo (`ActorContextImpl.scala:110-122`). No se afirma que sea un defecto ni se decide si debe conservarse: lo determinará el contrato de la fila si el Owner lo confirma como comportamiento observable en alcance (D-3).
- **Límites:**
  - **JDK:** las corridas se hicieron con **JDK 11** (`11.0.32-tem`); el repositorio requiere **Java 21** (`AGENTS.md`) y esa validación **sigue pendiente**. Hasta repetirla con Java 21 el resultado vale solo para JDK 11.
  - **`sleep` en `tools/oracle/adapter-classic-restart.jsh`** (el escenario no se declara determinista; no se demostró que lo sea). Para *provocar* una condición temporal: `Thread.sleep(150)` dentro del handler y el timer de 50 ms (mantienen el handler ocupado para que el `TimerMsg` quede encolado tras `Kill`), `Thread.sleep(20)` entre `armBlock` y `Kill` (coloca `Kill` antes de que venza el timer; si el planificador demora, el orden puede cambiar) y el timer de 30 s de S-G (inerte en la ventana). Para *sincronizar*: `Thread.sleep(300)` tras crear el hijo, `Thread.sleep(700)`/`Thread.sleep(900)` tras el estímulo (esperar el efecto) y `Thread.sleep(500)` tras `system.terminate()`. Los de sincronización suponen que 300/700/900 ms bastan; una máquina lenta podría dar otra salida sin que cambie el comportamiento de Pekko.
  - **Cobertura de escenarios:** causa de escape probada solo con `Kill`; no se probó un fallo en `preStart`/`postRestart` (`ActorInitializationException`, contrato ADP-006.3 «Errores»), ni un padre `OneForOne` con límites, ni más de un timer, ni otras temporizaciones. El descarte en S-G es inferido (el oráculo solo muestra que el `TimerMsg` no se entrega).
- **Delta propuesto (no aplicado):** reetiquetar A1/A2 de ADP-006.3 como validación compartida y añadir aserciones **N**: (N1) restart clásico → `PreRestart` sin `PostStop`, nuevo `setup`, timers inactivos (S-C/S-E); (N2) composición con `supervise(restart)` (S-F); (N3) `TimerMsg` obsoleto (S-G/S-H). Quitar o marcar como referencia `AT-SUP-005` en `depends_on` de ADP-006.3.

### 12.2 CTX-018 → SUP-016

Leídos completos AT-CTX-018 (`typed-core.md:1567`) y AT-SUP-016 (`:2735`); `ActorContext.scala:236-266,334-385`; `ActorContextImpl.scala:98-131,188-232,288-304`; `ActorContextAdapter.scala:54,125-150`; `Supervision.scala:290-300,330-336,390-396`; `TimerSchedulerImpl.scala:34-75,160-196`; `ActorAdapter.scala:121-135,305-335`; `StashBufferImpl.scala:204-236`.

| Comportamiento de CTX-018 | Implementa [código] | Lo usa [código] | Relación con SUP-016 | Clase |
|---|---|---|---|---|
| `cancelAllTimers` | `ActorContextImpl.scala:128-131` → `TimerSchedulerImpl.cancelAll` (`:162-167`) | `Supervision.scala:298,334,394`; `ActorAdapter.scala:311,319,327` | SUP-016 (S2) **lo consume** | **implementación: SUP-016 → CTX-018** (sentido inverso al declarado) |
| `hasTimer` | `ActorContextImpl.scala:126` | `ActorAdapter.scala:128` (decide interceptar `TimerMsg`) | SUP-016 A3/A4 se apoyan en esa rama, pero la consume el adaptador (ADP-006.1 A9) | consumo por P49; validación cruzada |
| descarte de `TimerMsg` obsoleto | AT-TMR-006 (`TimerSchedulerImpl.scala:169-196`) | `ActorAdapter.scala:128-133` | SUP-016 lo cita y depende de TMR-006 | referencia a TMR-006, no a CTX-018 |
| `spawnMessageAdapter` | `ActorContextImpl.scala:294-304`; `ActorContextAdapter.scala:135-141` | `ReplicatorBehavior.scala:146` (cluster-typed), tests | ninguna | — |
| `onUnhandled` | `ActorContextAdapter.scala:146` | `StashBufferImpl.scala:212`; `ActorContextImpl.scala:223` | ninguna (es AT-STASH-005) | — |
| `currentBehavior` | `ActorContextAdapter.scala:54` | `StashBufferImpl.scala:234`; `persistence-typed` | ninguna | — |
| `clearMdc` | `ActorContextImpl.scala:192-201` | `ActorAdapter.scala:121,306,314,322,334` | ninguna | — |

Aserciones: A1-A3 de CTX-018 (`StashSpec:636-671`, `ActorContextSpec:639-660`, `MessageAdapterSpec:94-114`) no usan supervisión; las de SUP-016 (`TimerSpec:174-232,327-404`) usan `supervise(restart)` y ejercen `cancelAllTimers` solo de forma indirecta. La frase «los usan stash, supervision y otros modulos» de CTX-018 enumera consumidores: referencia documental.

**Conclusión:** la arista no se retira entera: `cancelAllTimers` justifica la relación en el sentido **SUP-016 → CTX-018**. Ningún otro comportamiento de CTX-018 la justifica en ningún sentido. Delta (no aplicado): quitar `AT-SUP-016` de `depends_on` de AT-CTX-018 y añadir `AT-CTX-018` a AT-SUP-016 (la costura P49→P55 pasa de 4 a 3 aristas, ya documentales: §12.1, §12.3). Además la `Fuente` de SUP-016 cita los hooks de ADP-006.3 (`ActorAdapter.scala:309-323`) pero sus aserciones solo ejercen el restart tipado; los hooks pertenecen a ADP-006.3.

### 12.3 Resolución propuesta de `FailedBehavior`

Contrastados AT-BEH-024 (`typed-core.md:789`), AT-ADP-006.2 (`typed-extended.md:464`), AT-BEH-001/011/012 (P48) y AT-SUP-003/001, AT-SIG-005; `BehaviorImpl.scala:40,71,94-96`, `Behavior.scala:235-259,296-310`, `Supervision.scala:145-167,336-342`, `ActorAdapter.scala:94-100,164-172,232-273,325-335`, `ActorRefFactoryAdapter.scala:41-56`, `WatchSpec.scala:96-206`.

| Parte | Contenido [código] | Dónde está contratada hoy |
|---|---|---|
| **1. Valor e invariantes** | etiqueta `FailedBehavior = 7` (`BehaviorImpl.scala:40`); `failed(cause)` y clase con `cause` (`:71`, `:94-96`, `toString` `Failed(<cause>)`); `isAlive` falso (`Behavior.scala:248-250`); interpretar devuelve el mismo Behavior (`:303-304`); `validateAsInitial` solo rechaza `same`/`unhandled`, así que acepta `Failed` (`:235-241`) | **P48**: AT-BEH-001 (`typed-core.md:11,21`: una de las 9 etiquetas; variante `Failed(Error)`), AT-BEH-011 («Failed devuelve sí mismo», Efectos), AT-BEH-012 («`isAlive` es falso para Stopped y Failed»). **Ninguna aserción** lo cubre (BEH-004 A1 es solo `Stopped`; BEH-012 A1-A4 no lo usan) |
| **2. Interpretación por el adaptador** | `next`: `rethrowTypedFailure` ⇒ `TypedActorFailedException(cause)`, si no ⇒ `context.stop(self)` (`ActorAdapter.scala:168-172`); `next` no sustituye `behavior`, así `postStop` entrega `PostStop` al Behavior vigente (`:325-335`); la estrategia del padre registra la causa (`:232-240`) | **P49**: AT-ADP-006.2 (rama) y AT-ADP-006.4. La frase «no sustituye `behavior`» solo está en la sección «Orden» de BEH-024 |
| **3. Producción, notificación y validación** | productores: `StopSupervisor` (`Supervision.scala:159-167`); Backoff con `maxRestarts` agotado (`:338-340`); notificación: `failures`/`recordChildFailure` → `ChildFailed` (`ActorAdapter.scala:94-100,232-240,268-272`); validación: `WatchSpec:130-162` | **P55**: AT-BEH-024 (A2), AT-SUP-003 (A3 es el mismo test), AT-SIG-005 (A3) |

**Dos correcciones a la rev. 2 [código + contratos]:** (i) «solo AT-BEH-024 define `FailedBehavior`» es falso: lo contrata P48 (parte 1); (ii) **A1 de BEH-024 (`WatchSpec:100-128`, sin `supervise`) no ejerce `FailedBehavior`**: `ctx.spawn` no envuelve el Behavior en `supervise` (`ActorRefFactoryAdapter.scala:41-56`), la excepción sale del Behavior y la resuelve la rama `case _` de la estrategia del padre (`ActorAdapter.scala:240-262`) — es AT-ADP-006.4/AT-SUP-001. Solo A2 (con `supervise(stop)`) pasa por `Failed`. (No ejecutado en el oráculo.)

**Recomendación [propuesta; no registrada como decisión aceptada] — R-FB:** no mover ni dividir AT-BEH-024.
1. El valor y sus invariantes se implementan y verifican en P48 con aserciones **N** nuevas en filas existentes: `isAlive(Failed) = false` (BEH-012), interpretar `Failed` con mensaje y con señal —incluido `PostStop`— devuelve el mismo valor (BEH-011), el valor conserva su causa (BEH-001), `validateAsInitial(Failed)` lo acepta (BEH-012). Procedencia: **D** (código), sin spec.
2. P49 implementa la rama de AT-ADP-006.2 con ese valor real (sin doble) y añade al contrato de ADP-006.2 la frase «`next` no sustituye `behavior`».
3. AT-BEH-024 queda en S1 como producción y notificación; A1 se marca compartida con SUP-001/SIG-005 (A2), igual que C4.

| Fila | Cambio propuesto (TSV/contrato; no aplicado) |
|---|---|
| AT-ADP-006.2 | `depends_on`: quitar `AT-BEH-024`; añadir `AT-BEH-001`, `AT-BEH-011`, `AT-BEH-012`. Contrato: incorporar «no sustituye `behavior`» |
| AT-BEH-024 | `depends_on`: quitar `AT-SUP-022` (el flag es de la rama del adaptador); añadir `AT-ADP-006.2` como validación (A2). A1 marcada compartida |
| AT-BEH-001, -011, -012 | aserciones N de la lista anterior |
| Unidades | sin cambio de asignación; la costura P49→P55 de `FailedBehavior` desaparece |

Interoperabilidad: **nada se excluye**. `TypedActorFailedException`, `rethrowTypedFailure` y el registro de causa siguen siendo obligaciones de ADP-006.2/006.4 y SUP-022 (S6a). Con Q-X-8 (b) cambia el mecanismo; se mantiene el observable (la causa llega en `ChildFailed`, `PostStop` al Behavior que falló, un solo log).

| Alternativa | A favor | En contra |
|---|---|---|
| **A. R-FB** (recomendada) | no duplica contratos; elimina la costura; P49 prueba con el valor real | requiere aserciones N en tres filas de P48 y tres deltas de dependencias |
| B. (rev. 2) mover la parte de datos de BEH-024 a P48 | valor y su prueba juntos | duplica BEH-001/011/012; no explica por qué falta la aserción hoy |
| C. dividir BEH-024 en valor / producción | el valor tiene fila y aserciones propias | split de fila (renumeración en TSV, `contract-map`, `unit-rows`, planes) y duplicación con BEH-001/011/012 |
| D. doble temporal en P49 | no toca contratos | innecesario si P48 ya lo contrata; un doble no demuestra el valor real |

Abierto, no inventado: la representación de la causa común a `Failed` (BEH-001: `Failed(Error)`) y `ChildFailed` (SIG-005: `Arc<dyn Error + Send + Sync>`) no la fija la evidencia; depende de Q-X-5 y Q-T-BEH-1.

### 12.4 S1–S6: ciclos, atomicidad y costuras

Método: el script `docs/proposals/evidence/p55-unit-edges.py` (salida `p55-unit-edges.out`; no es un control del repositorio) lee `depends_on` de `docs/inventory/actor-typed.tsv`, extrae los IDs de fila y conserva solo las aristas cuyo origen y destino están entre las 31 filas de P55 y en unidades distintas. Comprueba, por tanto, **aristas declaradas** en el inventario, no dependencias leídas del código: no las clasifica (implementación / validación / documental; esa clasificación es manual y está en la tabla siguiente). Las asignaciones `rev3` y `rev3+deltas` son entradas del script (los deltas se codifican a mano: dos aristas retiradas, `SUP-013>SUP-002` añadida; `SUP-016>CTX-018` no aparece porque `CTX-018` no es de P55). Verifica también que cada asignación tenga 31 filas sin repetir. Sobre la asignación de §3 aparecen: S1→S2, S2→S1, S1→S4, S1→S6, S6→S1, S6→S2, S3→S2, con dos ciclos S1↔S2 y S1↔S6.

| Arista | Clase [código/contrato] | Resolución propuesta |
|---|---|---|
| `SIG-006>SUP-014` / `SUP-014>SIG-006` (S1↔S2) | `SUP-014>SIG-006`: implementación (la `DeathPactException` se lanza dentro de `interpretSignal`, `Behavior.scala:270-277`); `SIG-006>SUP-014`: validación (A3 usa `restart`, `SupervisionSpec:1408-1439`) | S1 antes que S2; SIG-006 queda `partial` hasta ejecutar A3 en S2. Se marca la arista como validación |
| `BEH-024>SUP-022` / `SUP-020,022>BEH-024,SUP-001,SUP-003` (S1↔S6) | `BEH-024>SUP-022`: documental (§12.3, el flag es de la rama del adaptador); `SUP-020,022>…`: implementación/validación hacia S1 | retirar `BEH-024>SUP-022`; S6a después de S1 y S2 |
| `SUP-003>SUP-010` (S1→S4) | **implementación**: `StopSupervisor.handleException` llama a `log` (`Supervision.scala:97-118,159-167`) y A1/A2 de SUP-003 afirman el log | mover SUP-010 a S1 (O2) |
| `SUP-013` sin arista a `SUP-002` | falta: el rechazo de `AbstractBehavior` está en `Supervisor.apply` (`Supervision.scala:44-51`) | añadir `SUP-013>SUP-002`; S5 → S1 |
| `CTX-018>SUP-016` (P49→P55) | §12.2 | invertir |
| `AT-BEH-007>AT-SUP-001` (P48→P55), `AT-BEH-003>AT-SIG-006` (P52→P55) | **no analizadas** a fondo; `DeathPactException` (C13) sigue el patrón de `FailedBehavior` | pendiente |

Con los deltas del §12 **el script no encuentra ciclos** entre las unidades de P55 (`p55-unit-edges.out`, caso `rev3+deltas`: S2→S1, S3→S2, S5→S1, S6a→S1, S6a→S2). Ese resultado depende de retirar aristas por clasificación manual; no demuestra atomicidad de las unidades ni que sean implementables en ese orden, y no cubre aristas hacia otras unidades (P48/P49/P52…). Sin aristas declaradas S4→S1–S3: su dependencia es de validación y debe registrarse como orden, no como arista.

**Organización propuesta (sin aplicar), 31 filas, cada una una vez:**

| Unidad | Filas | n |
|---|---|---|
| S1 | AT-SUP-001, AT-SUP-002, AT-SUP-003, AT-SUP-004, AT-SUP-010, AT-BEH-024, AT-SIG-005, AT-SIG-006 | 8 |
| S2 | AT-SIG-002, AT-SUP-005, AT-SUP-006, AT-SUP-008, AT-SUP-009, AT-SUP-014, AT-SUP-016 | 7 |
| S3 | AT-SUP-007.1, AT-SUP-007.2, AT-SUP-007.3, AT-SUP-017 | 4 |
| S4 | AT-SUP-011, AT-SUP-012, AT-SUP-019, AT-SUP-018, AT-STASH-006 | 5 |
| S5 | AT-BEH-010, AT-SUP-013 | 2 |
| S6a | AT-ADP-002, AT-SUP-020, AT-SUP-022 | 3 |
| S6b | AT-ADP-005, AT-SUP-021 | 2 |

Cambios respecto de §3, con su motivo:
- **O2 — SUP-010 de S4 a S1.** El log de un fallo es parte del resultado observable de `Stop` (SUP-003 A1/A2) y la función `log` es la misma para todas las estrategias. A3-A5 de SUP-010 necesitan Backoff y stash (S2/S3): la fila queda `partial` hasta entonces, como SIG-006.
- **O3 — SUP-018 y STASH-006 de S2 a S4.** Sus aserciones usan las cuatro estrategias (A1 stop, A2/A5 restart, A3 backoff, A4 resume: `StashSpec:460-610`, `SupervisionSpec:1195-1230`); en S2 no pueden cerrar. El código de `UnstashException` en `RestartSupervisor` (`Supervision.scala:345-349`) se escribe en S2 igualmente.
- **O4 — S6 en S6a y S6b.** Las filas forman dos componentes sin aristas entre sí: {ADP-002, SUP-020, SUP-022} (typed lanzado desde el mundo Classic: `supervise(stop)`, `rethrowTypedFailure`) y {ADP-005, SUP-021} (hijo Classic bajo padre typed: decisor Classic). S6b no necesita S2 (SUP-021 no usa `restart`); S6a sí (SUP-020 A2).

**Revisión por unidad (organización propuesta).** «Resultado» = observable único; «cierre» = pruebas concretas. Las decisiones salen de §8.

- **S1 — fallo sin reinicio.** *Resultado:* una excepción bajo `Stop`/`Resume` se registra con el formato y nivel configurados, detiene o no el actor y el padre observa `ChildFailed(causa)`; un `Terminated` sin manejar hace fallar al actor con `DeathPactException`. *Puede comenzar con:* P48 (BEH-004/007/011/012 incluida la variante `Failed`), P53 (BEH-013), P52 (LOG-001), P54 (SIG-004); la integración con la celda exige P49 (ADP-006.1/.2/.4). *Decisiones:* no impiden comenzar las de §8 salvo la forma de `Behavior` (Q-T-BEH-1) heredada de P48; impiden cerrar Q-X-5, Q-T-SUP-1, Q-T-SIG-1, Q-T-SUP-5, Q-X-7. *Cierre:* `SupervisionSpec:129-147,214-222,387-418,1441-1461,1497-1525`, `WatchSpec:130-204`, `TerminatedSpec`, `ActorContextSpec:483-533`; aserciones N de `Failed` (P48); oráculo jshell `Stop`/`Resume` con fallo en mensaje y en setup. *Atomicidad:* aceptable con reservas: SIG-006 no aporta código propio (el lanzamiento es de `interpretSignal`, P48) y SIG-006 A3 / SUP-010 A3-A5 cierran en S2/S3; se mantiene en S1 porque su A1 (`WatchSpec:165-204`) necesita la cadena `ChildFailed` de SIG-005. Si el Owner prefiere no tener filas `partial` en S1, SIG-006 pasaría a S2 junto con SUP-014 (alternativa, no recomendada: SIG-006 A1 no usa `restart`).
- **S2 — Restart.** *Resultado:* `Restart` con y sin límite/ventana: `PreRestart`, timers cancelados, hijos detenidos y espera de `Terminated`, mensajes apilados con capacidad y `Dropped`, nuevo `setup`. *Comienza con:* S1; P53 (STASH-001, TMR-006); P52 (EVT-004); P50 (SYS-006); P54 (CTX-005/006); P49 (CTX-018 `cancelAllTimers`). *Decisiones:* Q-T-BEH-1 (bloqueante, SUP-005), Q-T-STASH-1 y Q-X-6 (SUP-009) impiden comenzar según ficha; cerrar: Q-T-SUP-4, Q-T-SUP-7, Q-T-SUP-8/Q-T-TMR-4. *Cierre:* `SupervisionSpec:149-174,224-281,512-847,1078-1105`, `TimerSpec:174-232,327-404`, `ActorContextSpec:255-279`; oráculo con 2 hijos de parada lenta. *Atomicidad:* sí; el stash no se separa de la espera de hijos (SUP-008 A1). SUP-014 no aporta código (su única aserción usa `restart`).
- **S3 — Backoff.** *Resultado:* `restartWithBackoff` con retardo exponencial y jitter, `ScheduledRestart`/`ResetRestartCount`, `withMaxRestarts`. *Comienza con:* S2 (misma clase `RestartSupervisor`, `SUP-007.1>SUP-009`) y P53 (TMR-001). *Decisiones:* cerrar: Q-T-SUP-3 (bloqueante, SUP-007.3), Q-T-SUP-6, Q-T-SUP-8/Q-T-TMR-4 (reloj/azar inyectables). *Cierre:* `SupervisionSpec:882-1076,1117-1176`, oráculo para Q-T-SUP-3 (no ejecutado), N para la fórmula. *Atomicidad:* sí; las señales de SUP-017 solo las producen los timers de Backoff (`Supervision.scala:198-203`).
- **S4 — supervisión compuesta.** *Resultado:* la supervisión conserva su semántica al anidarse (deduplicación `isSame`), al cambiar de Behavior (re-wrap), al detenerse bajo cualquier estrategia y al fallar un `unstashAll`. *Comienza con:* S1-S3 completos (validación), P53 (BEH-015, STASH), P48 (BEH-004). *Decisiones:* cerrar: Q-T-SUP-1 (SUP-011), Q-T-STASH-1. *Cierre:* `SupervisionSpec:176-212,448-490,1195-1230,1270-1406,1661-1692`, `StashSpec:460-610`. *Atomicidad:* **la más débil**: sin SUP-010 es una unidad de composición con una sola idea (interceptor + estrategias), pero buena parte de su cierre es validación de S1-S3; si el Owner no quiere una unidad de validación, cada fila cerraría en la última unidad que necesite (SUP-011/012/019 y SUP-018/STASH-006 en S3).
- **S5 — AbstractBehavior.** *Resultado:* el estilo orientado a objetos y su rechazo bajo `supervise`. *Comienza con:* P48 (BEH-007), S1 (`Supervisor.apply`) y P54. *Decisiones:* Q-T-BEH-1 y Q-T-BEH-5 impiden cerrar SUP-013 y BEH-010 A1/A2/A4/A5; Q-T-CTX-2 (A3). *Cierre:* `ActorThreadSpec:188-277`, `SupervisionSpec:1527-1573`, `AdaptationFailureSpec:66-127`. *Atomicidad:* se sostiene por el test compartido (A4 = SUP-013 A1/A2) y por el tipo; el rechazo vive en S1.
- **S6a — typed lanzado desde Classic.** *Resultado:* stop por defecto, `supervise(restart)` respetado y `rethrowTypedFailure` según el origen (guardián, `systemActorOf`, spawns desde Classic, `ctx.spawn`). *Comienza con:* S1, S2 (SUP-020 A2), P49 (ADP-003, PropsAdapter), P50 (SYS-008). *Decisiones:* Q-X-8, Q-T-ADP-3 (mecanismo; las obligaciones no se excluyen: §9). *Cierre:* `ClassicSupervisingTypedSpec:74-160`, `WatchSpec:100-162`. *Atomicidad:* sí por el flag compartido; ADP-002 A1-A4 = SUP-020 A1-A3 (C4).
- **S6b — hijo Classic bajo padre typed.** *Resultado:* el padre typed supervisa al hijo Classic con el decisor Classic por defecto (log ERROR) y `watch`/`unwatch`/`stop`/`actorOf` cruzados. *Comienza con:* P49 (ADP-006.4), P21 (decisor Classic), P61 (CTX-017) como consumidor. *Decisiones:* Q-X-8, Q-T-SUP-7, Q-T-CTX-4. *Cierre:* `AdapterSpec:300-343`, `TypedSupervisingClassicSpec:55-75`, `AdapterTest.java:338-352`. *Atomicidad:* sí; no comparte aserciones con S6a.

### 12.5 Resumen de la revisión

**A. Correcciones demostradas por la evidencia**
1. **C10:** A1/A2 de ADP-006.3 ejercen el supervisor tipado, no los hooks (oráculo S-A, código, spec). Los hooks son alcanzables por API pública y no tienen aserción portada (oráculo S-C/E/F; búsqueda en los tests).
2. La rev. 2 no sabía si existía una ruta pública de restart clásico: existe (padre Classic con `Restart` + excepción que sale del Behavior).
3. `AT-CTX-018>AT-SUP-016` está invertida para `cancelAllTimers`; ningún otro comportamiento de CTX-018 la justifica (código).
4. `FailedBehavior` no la define solo BEH-024: el valor ya está en AT-BEH-001/011/012 (P48), sin aserción propia (contratos + código).
5. A1 de BEH-024 no ejerce `FailedBehavior` (código + spec).
6. §4.4 de la rev. 2 era imprecisa: con la asignación de §3 hay ciclos S1↔S2 y S1↔S6 a nivel de aristas, y una arista S1→S4 (SUP-003→SUP-010) no registrada.
7. La `Fuente` de SUP-016 incluye los hooks de ADP-006.3 pero sus aserciones solo ejercen el restart tipado.
8. `SUP-013` no declara su dependencia de `SUP-002` aunque el rechazo vive en `Supervisor.apply`.
9. Tras un restart clásico un `TimerMsg` obsoleto puede llegar al Behavior (S-H): hecho observado de Pekko, no cubierto por ninguna aserción.

**B. Propuestas de organización** (sin aplicar): R-FB (§12.3); O2 SUP-010 → S1; O3 SUP-018/STASH-006 → S4; O4 S6 → S6a/S6b; deltas de aristas C12 y C14; aserciones N en BEH-001/011/012 y ADP-006.3; reetiquetado de A1/A2 de ADP-006.3 y de la `Fuente` de SUP-016.

**C. Decisiones que necesita el Owner**
- **D-1 (propuesta, sin aplicar).** ¿Se autorizan los deltas de B (reasignación en `docs/unit-rows.tsv`, edición de `depends_on` y de contratos) y, con ellos, S1–S6b? Publicar esta revisión **no** autoriza reasignar filas ni modificar contratos. Sustituye a las preguntas 7 y 8 de §10 en lo que se refiere a `FailedBehavior` y a la reasignación.
- **D-2 (propuesta, sin aplicar).** Si prefiere B/C de §12.3 (mover o dividir BEH-024) en lugar de R-FB, que sigue siendo una recomendación no registrada como decisión.
- **D-3 (arquitectura, no una obligación nueva).** El objetivo vigente del proyecto es preservar los comportamientos observables de Pekko que estén dentro del alcance. Por eso el reinicio del actor que aloja un Behavior typed cuando un padre Classic decide `Restart` (S-C…S-H) es un comportamiento observable de Pekko que ya está bajo esa regla; la paridad no está pendiente de aprobación. Lo que falta decidir es **cómo se implementa**: depende de Q-X-8 y Q-T-ADP-2/3 (con (b) la capa de interoperabilidad no queda exenta por defecto). Excluir cualquier parte (p. ej. la entrega de un `TimerMsg` obsoleto en S-H) exigiría una **decisión explícita de divergencia** por aserción.
- **Dudas abiertas que esta revisión no resuelve:** (i) **S4** es la unidad de atomicidad más débil (§12.4): su cierre es en buena parte validación de S1–S3 y podría no ser una unidad propia; (ii) **`DeathPactException`** (C13): la costura BEH-011 (P48) / SIG-006 (S1) / BEH-003 (P52) no se analizó con la profundidad de `FailedBehavior`, y las aristas `BEH-007>SUP-001` y `BEH-003>SIG-006` siguen sin verificar.
- **Obligaciones de paridad ya vigentes (no son decisiones pendientes):** conservar los comportamientos observables de Pekko dentro del alcance, incluida la interoperabilidad (S6, §9) y el reinicio clásico del adaptador (§12.1). Las decisiones pendientes son D-1, D-2, la arquitectura de D-3 y las de §8/§10.
- Siguen sin respuesta las preguntas 1-6 de §10 (Q-X-8, Q-X-5, Q-T-BEH-5, Q-T-SIG-1, Q-T-SUP-1, Q-T-SUP-3) y los bloqueantes Q-T-BEH-1 y Q-T-SUP-3.

**¿Se puede aprobar P55 documentalmente?** Como propuesta de organización, sí una vez el Owner responda D-1 (y D-2 si objeta R-FB). **No** está lista como plan ejecutable. Impedimentos concretos: (1) los deltas no están aplicados y los ciclos S1↔S2/S1↔S6 solo desaparecen con ellos; (2) O2/O3/O4 son reasignaciones que exigen autorización; (3) la costura `DeathPactException` (C13) y las aristas `BEH-007>SUP-001`, `BEH-003>SIG-006` no se analizaron; (4) el oráculo se ejecutó con JDK 11 (Java 21 pendiente), con sleeps como sincronización, y solo cubre C10; Q-T-SUP-3 y las aserciones N de P55 siguen sin medir; (5) Q-T-BEH-1 y Q-T-SUP-3 (bloqueantes) impiden cerrar S2, S3 y S5.
