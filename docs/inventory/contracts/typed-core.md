# Contratos verificables: typed-core (BEH, SIG, CTX, REF, SYS, SUP, TMR, STASH)

Fuente de verdad: `upstream/pekko` en 9a54d1294c3ac7821bbed3c940769c3d5027d1ff (v1.7.1). Todas las citas `path:linea` son relativas a `upstream/pekko`. Filas en `docs/inventory/rows/typed-core.tsv`, decisiones en `docs/inventory/contracts/typed-core.decisions.tsv`. Las decisiones `Q-X-*` son las transversales del orquestador (HOCON, registro de fabricas, mensajes dinamicos, runtime, fallo, EventStream, loggers, typed sobre classic, Done/Timeout). Marcador de oraculo: ningun escenario tipado existe aun en `tools/oracle/`; los oraculos propuestos son escenarios `.jsh` por escribir sobre los jars oficiales.

Convencion de aserciones: `A<n>` es un comportamiento esperado + cita del cuerpo del test de Pekko que lo afirma; cuando ningun test lo afirma se marca *derivada del codigo; no afirmada por spec*.

## AT-BEH-001 — Behavior[T]: tipo base sellado (@DoNotInherit) con tags internos y ExtensibleBehavior (receive/receiveSignal) como punto de extension

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Un Behavior[T] es un valor inmutable que define como reacciona el actor al proximo mensaje T o Signal y cual es el Behavior siguiente. El usuario no extiende Behavior (@DoNotInherit): extiende ExtensibleBehavior o usa los DSL. Un Behavior que cierra sobre un ActorContext no se puede mover a otro contexto. Hay 9 tipos internos (tags 1..9): Extensible, Empty, Ignore, Unhandled, Deferred, Same, Failed, Stopped, Supervise.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:30-51; actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:116-151 (ExtensibleBehavior.receive/receiveSignal); actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/BehaviorImpl.scala:29-44 (BehaviorTags) | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: receive/receiveSignal devuelven el Behavior siguiente o uno de los especiales (stopped, same, unhandled); el resultado de un ExtensibleBehavior se arranca (Behavior.start) si es Deferred. Limites: Behavior.narrow es un cast sin costo (Behavior.scala:56). Errores: ExtensibleBehavior.receive/receiveSignal pueden lanzar Exception (@throws), que lo trata la supervision (AT-SUP-*). Efectos: ninguno al construir el valor; el estado mutable debe vivir en setup (AT-BEH-023).

**Orden / concurrencia / ciclo de vida.** El actor procesa un mensaje o senal a la vez; el Behavior devuelto reemplaza al actual antes del siguiente (ActorAdapter.next, ActorAdapter.scala:164-180). Los Behavior no tienen hooks propios de ciclo de vida: reciben PreRestart/PostStop como Signal (AT-SIG-002/003).

**Dependencias.** AT-BEH-011; AC-CELL (lo ejecuta ActorAdapter)

**Equivalente Rust propuesto.** enum Behavior<M> { Same, Unhandled, Stopped(Option<PostStopFn>), Empty, Ignore, Deferred(Arc<dyn Fn(&mut Ctx<M>)->Behavior<M> + Send + Sync>), Extensible(Box<dyn ExtensibleBehavior<M>>), Intercept(..), Failed(Error) } con constructores en un modulo behaviors; el tag numerico de Scala pasa a ser la variante del enum. **Reutilizacion (propuesta, no aceptada: Q-T-BEH-1):** `Behavior<M>: Clone` (las variantes que llevan cierres los comparten por `Arc`), porque el mismo valor se arranca en actores independientes y se vuelve a arrancar en cada restart; un `FnOnce` no puede cumplirlo (ver AT-BEH-007). **Diferencias observables:** Enum cerrado en vez de clase sellada; ExtensibleBehavior pasa a trait publico con receive/receive_signal. Sin subtipado, Behavior<T> no es covariante/contravariante (ver AT-BEH-022).

**Aserciones.**

- A1: Un Behavior funcional cambia de estado devolviendo otro Behavior (StateA a StateB y de vuelta) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/BehaviorSpec.scala:286-292

- A2: El mismo valor Behavior sirve para dos actores/arranques independientes (cada BehaviorTestKit parte de StateA); el spec lo prueba sobre Behaviors funcionales y sus envoltorios (transformMessages, intercept, supervise), no sobre un `Deferred` directo (BehaviorSpec.scala:505, 528, 554) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/BehaviorSpec.scala:334-338

- A3: ExtensibleBehavior es la base de AbstractBehavior, ReceiveBehavior e InterceptorImpl (los DSL se implementan sobre el), derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/AbstractBehavior.scala:46; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/BehaviorImpl.scala:135-139; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/InterceptorImpl.scala:51-54

- A4: Existen exactamente 9 tags de Behavior (1..9), derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/BehaviorImpl.scala:34-42

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): comparar secuencias de estado de un Behavior receive sobre un actor real

**Decisiones.** Q-T-BEH-1, Q-T-BEH-4

## AT-BEH-002 — Behaviors.same

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Behaviors.same: devolverlo desde un handler mantiene el Behavior actual (sin reallocar). No es valido como Behavior inicial (IllegalArgumentException) ni se puede ejecutar directamente en el interprete.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Behaviors.scala:52-58; actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:193-202 (canonicalize); actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:250-253 (validateAsInitial); actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:299-300; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/BehaviorImpl.scala:58, 90-92 (toString "Same") | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: canonicalize(same, current) = current. Errores: validateAsInitial(same) lanza IllegalArgumentException("cannot use Same as initial behavior"); interpretar same/unhandled lanza IllegalArgumentException("cannot execute with [Same] as behavior"); setup(setup(same)) detiene el actor al arrancar con ActorInitializationException (error logueado). Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** Se resuelve en Behavior.canonicalize despues de cada handler (ActorAdapter.next, ActorAdapter.scala:177-178). Dentro de unstash se resuelve contra el Behavior en curso del unstash, no el del actor (StashBufferImpl.scala:187-244).

**Dependencias.** AT-BEH-011; AT-BEH-012

**Equivalente Rust propuesto.** Variante Behavior::Same; validate_as_initial devuelve Err(InvalidInitialBehavior). **Diferencias observables:** Ninguna observable.

**Aserciones.**

- A1: Un handler que devuelve same permite recibir otro mensaje con el mismo Behavior (Ignore luego Ping) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/BehaviorSpec.scala:251-253

- A2: setup que devuelve same como Behavior inicial termina el actor y se loguea ActorInitializationException — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/DeferredSpec.scala:147-158

- A3: unstashAll(Behaviors.same) usa el Behavior en curso (inicial y tras un mensaje intermedio) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashSpec.scala:331-347, 349-369

- A4: validateAsInitial(same) lanza IllegalArgumentException con mensaje "cannot use Same as initial behavior" (texto exacto por toString "Same"); no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:250-253

- A5: interpretar same directamente lanza IllegalArgumentException("cannot execute with [Same] as behavior"); no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:299-300

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): llamar Behavior.validateAsInitial/interpretMessage por jshell con Behaviors.same y capturar mensaje de excepcion

**Decisiones.** Q-T-BEH-1

## AT-BEH-003 — Behaviors.unhandled

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Behaviors.unhandled: el mensaje/senal se considera no manejado; se mantiene el Behavior actual y el ActorAdapter lo deriva a Actor.unhandled clasico, que publica UnhandledMessage en el event stream. Un Terminated no manejado lanza DeathPactException (AT-SIG-006). Invalido como Behavior inicial.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Behaviors.scala:60-66; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/BehaviorImpl.scala:60, 86-88; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:168-171; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:224-234; actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:301-302 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: handler devuelve unhandled, el actor sigue vivo con el Behavior actual y procesa el siguiente mensaje. Errores: Terminated no manejado en interpretSignal lanza DeathPactException(ref); como inicial: IllegalArgumentException("cannot use Unhandled as initial behavior"). Efectos: UnhandledMessage(msg, sender, recipient) al event stream para mensajes; para otras Signal no se publica nada (ActorAdapter.unhandled, caso `_: Signal`, ActorAdapter.scala:226-227).

**Orden / concurrencia / ciclo de vida.** La publicacion ocurre en el hilo del actor tras interpretar; en unstash se invoca ctx.onUnhandled (StashBufferImpl.scala:211-213), que llama ActorAdapter.unhandled.

**Dependencias.** AT-EVT-003; AT-SIG-006; AC-CELL; AC-EVT

**Equivalente Rust propuesto.** Behavior::Unhandled; el runtime publica UnhandledMessage{msg, sender, recipient} en el EventStream. **Diferencias observables:** Necesita un EventStream (Q-X-6).

**Aserciones.**

- A1: Behavior.isUnhandled(interpretMessage(b, ctx, Miss)) es true y el monitor ve Missed — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/BehaviorSpec.scala:259-265

- A2: Un mensaje unhandled se publica como UnhandledMessage y el actor sigue procesando (Ping despues de Miss responde Pong) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:126-133

- A3: delegate que devuelve unhandled publica UnhandledMessage(UnPingable,_,_) y el actor pasa al delegador — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorContextDelegateSpec.scala:83-100

- A4: unhandled como Behavior inicial lanza IllegalArgumentException("cannot use Unhandled as initial behavior"); no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:250-253

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): suscribir un listener al eventStream clasico y enviar un mensaje a un Behavior que devuelve unhandled

**Decisiones.** Q-X-6

## AT-BEH-004 — Behaviors.stopped y stopped(postStop)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Behaviors.stopped y stopped(postStop): el actor termina voluntariamente; sus hijos se detienen; el Behavior previo recibe PostStop y despues corre el callback postStop (ComposedStoppingBehavior). Es valido como Behavior inicial (el actor para enseguida). Un Deferred no se arranca al parar.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Behaviors.scala:68-91; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/BehaviorImpl.scala:62-65, 98-121 (StoppedBehavior); actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:177-180; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:329-340; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:346-366; actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:309-312 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: stopped(cb) => el Behavior previo (y sus interceptores) recibe PostStop, luego se invoca cb. Limites: si el Behavior previo es Deferred no se arranca (ActorAdapter.scala:354-356). Errores: ComposedStoppingBehavior lanza IllegalStateException("Stopping, should never receive a message") si recibe un mensaje y IllegalArgumentException("The ComposedStoppingBehavior should only ever receive a PostStop signal, but received ...") si recibe otra senal (ActorAdapter.scala:346-351). Efectos: context.stop(self) y el Behavior pasa a Stopped; todos los hijos paran antes de que se complete la terminacion.

**Orden / concurrencia / ciclo de vida.** Orden observable: PostStop del Behavior (y del interceptor) -> callback stopped(cb) -> Terminated a watchers (tras terminar los hijos). Con stopped como resultado de setup inicial, el callback corre al arrancar (StopSpec 33-41, 95-102).

**Dependencias.** AT-SIG-003; AC-CELL (context.stop)

**Equivalente Rust propuesto.** Behavior::Stopped(Option<Arc<dyn Fn(&mut Ctx<M>) + Send + Sync>>) (propuesta, no aceptada: Q-T-BEH-1). **Diferencias observables:** Ninguna observable. **Nota de coherencia (derivada del codigo, no afirmada por un spec):** `StoppedBehavior` es un valor inmutable con un callback (BehaviorImpl.scala:64-65); si el valor se reutiliza, el callback se invoca una vez por actor que se detiene, igual que el cierre de `Deferred`.

**Aserciones.**

- A1: Un Behavior stopped deja de estar vivo (isAlive false) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/BehaviorSpec.scala:271-275

- A2: stopped(cb) devuelto desde setup ejecuta cb — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StopSpec.scala:33-41

- A3: stopped(cb) tras un mensaje ejecuta cb — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StopSpec.scala:43-53

- A4: Primero llega PostStop al Behavior y despues corre el callback ("signal" luego "callback") — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StopSpec.scala:55-73

- A5: stopped(cb) envuelto en un interceptor tambien ejecuta cb — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StopSpec.scala:76-92

- A6: stopped(cb) como Behavior inicial del ActorSystem/spawn ejecuta cb enseguida — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StopSpec.scala:95-102

- A7: El interceptor ve PostStop antes que el callback, ambos en orden — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/InterceptSpec.scala:400-427

- A8: Los hijos reciben PostStop antes que el callback del padre ("child-done","child-done","parent-done") — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/GracefulStopSpec.scala:30-58

- A9: Si el hijo se crea con setup que devuelve stopped, el padre recibe Terminated — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/DeferredSpec.scala:84-97

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): spawn de un actor con stopped(cb) y receiveSignal(PostStop); registrar el orden de eventos

**Decisiones.** ninguna (sin decision abierta)

## AT-BEH-005 — Behaviors.empty

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Behaviors.empty: trata todo mensaje como unhandled (el interprete devuelve unhandled); las senales no se manejan (Terminated no manejado produce DeathPactException como cualquier Behavior).

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Behaviors.scala:93-96; actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:315-316; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/BehaviorImpl.scala:67, 78-80 (toString "Empty") | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: cada mensaje => Unhandled => UnhandledMessage publicado. Senales: interpret devuelve unhandled; MessageAdaptionFailure no mata el actor (el default relanza solo en receive*/AbstractBehavior; empty no lo relanza). Errores: Terminated => DeathPactException (Behavior.scala:274). Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** Sin estado.

**Dependencias.** AT-BEH-003

**Equivalente Rust propuesto.** Behavior::Empty. **Diferencias observables:** Ninguna.

**Aserciones.**

- A1: Un actor con empty NO se detiene cuando falla un message adapter (MessageAdaptionFailure se ignora) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/AdaptationFailureSpec.scala:103-127

- A2: interpret de EmptyBehavior devuelve unhandled; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:315-316

- A3: Terminated recibido por empty lanza DeathPactException; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:280-287

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): enviar mensaje y Terminated a un actor con Behaviors.empty

**Decisiones.** ninguna (sin decision abierta)

## AT-BEH-006 — Behaviors.ignore

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Behaviors.ignore: ignora todos los mensajes y senales y devuelve same sin publicar UnhandledMessage. Tambien traga Terminated (no hay DeathPact).

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Behaviors.scala:98-101; actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:307-308; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/BehaviorImpl.scala:69, 82-84 (toString "Ignore") | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: devuelve same siempre (incluso para PostStop/PreRestart). Errores: ninguno (interpretSignal no lanza DeathPact porque result != Unhandled). Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** Sin estado.

**Dependencias.** AT-BEH-011

**Equivalente Rust propuesto.** Behavior::Ignore. **Diferencias observables:** Ninguna.

**Aserciones.**

- A1: Un actor con ignore sobrevive al MessageAdaptionFailure — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/AdaptationFailureSpec.scala:103-127

- A2: ignore tambien devuelve same para Terminated, por lo que no hay DeathPactException; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:280-287, 297-298

- A3: ignore no publica UnhandledMessage; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:307-308

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): actor con ignore y watcher: parar al observado y comprobar que no hay fallo

**Decisiones.** ninguna (sin decision abierta)

## AT-BEH-007 — Behaviors.setup (DeferredBehavior)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Behaviors.setup(factory): la fabrica corre al arrancar el actor (no al primer mensaje) y en cada restart, con el ActorContext; setups anidados se arrancan hasta un Behavior concreto; setup(same)/setup(unhandled) se rechaza; una excepcion en la fabrica detiene el actor (ActorInitializationException) salvo supervision; un Deferred no se arranca al parar.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Behaviors.scala:29-41; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/BehaviorImpl.scala:123-133; actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:193-221 (canonicalize, start); actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:303-311; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:321-327 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: factory(ctx) devuelve el Behavior concreto; start desenvuelve Deferred anidados, incluso dentro de interceptores y SuperviseBehavior. Limites: PostStop no arranca Deferred (ActorAdapter.scala:329-331). Errores: excepcion de la fabrica => ActorInitializationException logueada y el actor termina; setup(same) => idem; en BehaviorTestKit la excepcion se propaga tal cual al llamador (DeferredSpec 177-188). Efectos: la fabrica puede hacer spawn/watch/timers.

**Orden / concurrencia / ciclo de vida.** preStart (ActorAdapter.scala:299-307): Behavior.validateAsInitial(Behavior.start(behavior, ctx)). Cuando se devuelve desde un handler se arranca inmediatamente (el siguiente mensaje lo procesa el Behavior arrancado).

**Dependencias.** AT-BEH-011; AT-SUP-001

**Equivalente Rust propuesto.** **Requisitos observables (contrato):** (R1) la misma fabrica se invoca una vez por cada arranque del mismo valor `Behavior`, tambien en actores independientes: `DeferredBehavior` es un objeto inmutable cuyo `apply(ctx)` se invoca en cada `Behavior.start`/`canonicalize` (BehaviorImpl.scala:123-133; Behavior.scala:183-202), y `start` no lo consume; (R2) en cada restart la fabrica se vuelve a ejecutar: `RestartSupervisor.restartCompleted` llama `Behavior.start(initial, ctx)` sobre el mismo `initial` (Supervision.scala:404; SupervisionSpec.scala:1145-1153 y 1673-1691); (R3) un `Deferred` sin arrancar no se arranca al parar (ActorAdapter.scala:329-331, 355-356). **Representacion Rust propuesta (NO aceptada; pendiente de la decision Q-T-BEH-1 del Owner):** `Behavior::Deferred(Arc<dyn Fn(&mut ActorContext<M>) -> Behavior<M> + Send + Sync>)` con `Behavior<M>: Clone`; el estado que la fabrica comparte entre invocaciones (p. ej. el `AtomicBoolean` de SupervisionSpec.scala:1674-1684) pasa a mutabilidad interior (`Arc<AtomicBool>`), no a `FnMut`. **Alternativas aun abiertas:** (b) `Box<dyn FnOnce>` prohibiendo restart y reutilizacion de esos Behaviors, que contradice R1 y R2 y por eso no puede cerrar AT-BEH-007; (c) trait object con `clone_box`. Ninguna esta aceptada. **Diferencias observables:** ninguna si se adopta (a) o (c); con (b) se pierden R1 y R2, lo que exigiria autorizacion explicita del Owner (A-4, A-5).

**Aserciones.**

- A1: La fabrica se ejecuta al arrancar el actor, antes del primer mensaje — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/DeferredSpec.scala:51-61

- A2: Excepcion en la fabrica de un hijo detiene al hijo; el padre que lo observa recibe Terminated y se loguea ActorInitializationException — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/DeferredSpec.scala:63-82

- A3: setup que devuelve stopped detiene el actor — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/DeferredSpec.scala:84-97

- A4: Setups anidados se arrancan hasta el Behavior concreto — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/DeferredSpec.scala:99-109

- A5: setup envuelto en transformMessages y en monitor se arranca de inmediato y luego procesa mensajes — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/DeferredSpec.scala:111-145

- A6: setup(setup(same)) termina el actor con ActorInitializationException — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/DeferredSpec.scala:147-158

- A7: En BehaviorTestKit la fabrica corre inmediatamente y la excepcion se propaga al llamador — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/DeferredSpec.scala:166-188

- A8: En un restart la fabrica de `setup` se ejecuta de nuevo: con `restart.withLimit` y una fabrica que falla una vez se observa StartFailed y luego Started — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:355-377; actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1152-1160

- A9: La segunda ejecucion de la fabrica tras un restart ve el estado externo dejado por la primera y puede devolver `stopped` (el actor termina) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1680-1698

- A10: Un `Deferred` sin arrancar no se arranca al parar el actor (sin efectos de creacion), derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:333-335; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:359-360

- A8: Intercept con setup anidado y recursivo reinicia el estado de la fabrica solo cuando se vuelve a crear el setup — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/InterceptSpec.scala:236-291

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): spawn de un setup que registra eventos en un probe externo; con y sin supervise(restart)

**Decisiones.** Q-T-BEH-1, Q-X-5

**Actualización 2026-11-18.** El Owner aceptó (a) en Q-T-BEH-1: la forma `Behavior` de Rust deja de estar «NO aceptada»; la caché de Props de 2.0.0 no se replica (optimización interna sin efecto observable).

## AT-BEH-008.1 — Behaviors.receive / receiveMessage: Receive[T] con y sin contexto

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** receive(onMessage: (ctx, msg) => Behavior) y receiveMessage(onMessage: msg => Behavior) crean un Behavior (Receive[T]) que procesa mensajes con o sin ActorContext. El manejador de senales por defecto relanza MessageAdaptionFailure y trata el resto como unhandled.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Behaviors.scala:103-135; actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Behaviors.scala:330-349 (Receive, ReceiveImpl, ReceiveMessageImpl); actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/BehaviorImpl.scala:73-76, 135-172 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: onMessage devuelve el siguiente Behavior; receive con ctx permite spawn/watch; receiveMessage evita envolver la funcion. Errores: excepcion en onMessage => supervision; default de senales: MessageAdaptionFailure(ex) => throw ex (BehaviorImpl.scala:74). Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** receive y receiveMessage son ExtensibleBehavior; se evaluan en el hilo del actor.

**Dependencias.** AT-BEH-001; AT-SIG-007

**Equivalente Rust propuesto.** behaviors::receive(|ctx, msg| ...) y behaviors::receive_message(|msg| ...) devuelven un constructor de Receive<M> con .on_signal(..). **Diferencias observables:** Ninguna observable.

**Aserciones.**

- A1: Un Behavior receive procesa mensajes, cambia de estado y reacciona a PostStop/PreRestart/Terminated por receiveSignal — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/BehaviorSpec.scala:350-382

- A2: receive con firma (context, message) y receiveSignal se comporta igual — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/BehaviorSpec.scala:400-434

- A3: receive sin receiveSignal: el default relanza MessageAdaptionFailure y el actor se detiene — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/AdaptationFailureSpec.scala:65-67, 89-101

- A4: receiveMessage (sin ctx) tambien relanza MessageAdaptionFailure por defecto — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/AdaptationFailureSpec.scala:66, 89-101

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): secuencia de mensajes sobre un receive/receiveMessage equivalente

**Decisiones.** Q-T-BEH-2

## AT-BEH-008.2 — Behaviors.receivePartial / receiveMessagePartial: funciones parciales con unhandled

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** receivePartial y receiveMessagePartial construyen un Receive a partir de una PartialFunction; un mensaje para el que la funcion no esta definida se trata como unhandled (applyOrElse a Behaviors.unhandled) y se publica UnhandledMessage.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Behaviors.scala:158-172 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: caso definido => su Behavior. Limites: caso no definido => Behaviors.unhandled. Errores: receivePartial(PartialFunction.empty) no maneja MessageAdaptionFailure (relanza). Efectos: UnhandledMessage para no definidos.

**Orden / concurrencia / ciclo de vida.** Igual que receive.

**Dependencias.** AT-BEH-008.1; AT-BEH-003

**Equivalente Rust propuesto.** No hay PartialFunction: se modela con receive cuyo match devuelve Behavior::Unhandled en el brazo _ (AT-BEH-008.1). **Diferencias observables:** El compilador de Rust fuerza la exhaustividad del match; la variante *_partial desaparece como API separada.

**Aserciones.**

- A1: receiveMessagePartial ignora (unhandled) Command1 y procesa Command2 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ReceivePartialSpec.scala:31-46

- A2: receivePartial (con ctx) ignora Command1 y procesa Command2 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ReceivePartialSpec.scala:48-63

- A3: receiveMessagePartial con un solo caso ("stop") hace que el resto sea unhandled y PostStop llegue al receiveSignal — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/OnSignalSpec.scala:45-61

- A4: receivePartial(PartialFunction.empty) se detiene por MessageAdaptionFailure (default relanza) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/AdaptationFailureSpec.scala:67, 89-101

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): enviar mensajes definidos y no definidos; observar UnhandledMessage

**Decisiones.** Q-T-BEH-2

## AT-BEH-008.3 — Behaviors.receiveMessageWithSame (desde 1.1.0)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** receiveMessageWithSame(onMessage: T => Unit) procesa el mensaje y siempre devuelve same; no tiene ctx ni cambia de Behavior.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Behaviors.scala:137-156 (@since 1.1.0) | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: onMessage(msg) y luego same. Errores: excepcion => supervision. Efectos: los del callback.

**Orden / concurrencia / ciclo de vida.** Igual que receiveMessage; el cuerpo es onMessage.andThen(_ => same) (Behaviors.scala:154).

**Dependencias.** AT-BEH-008.1; AT-BEH-002

**Equivalente Rust propuesto.** behaviors::receive_message_with_same(|msg| ...). **Diferencias observables:** Ninguna.

**Aserciones.**

- A1: ReceiveMessageWithSameBehaviorSpec pasa los casos de Messages: dos Ping, Miss luego Ping, Ignore luego Ping — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/BehaviorSpec.scala:244-253, 383-397

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): secuencia Ping/Miss/Ignore

**Decisiones.** Q-T-BEH-2

## AT-BEH-009 — receiveSignal y Receive.receiveSignal (manejo de senales)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** receiveSignal(partial) agrega un manejador parcial de Signal a un Receive o crea un Behavior que solo maneja senales (los mensajes quedan unhandled). Las senales no cubiertas siguen el default (MessageAdaptionFailure se relanza; el resto unhandled).

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Behaviors.scala:174-178; actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Behaviors.scala:330-349; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/BehaviorImpl.scala:73-76, 141-145, 165-169 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: el handler recibe (ctx, signal) y devuelve el siguiente Behavior. Limites: Behaviors.receiveSignal es receive((_,_) => same).receiveSignal(handler) (Behaviors.scala:176-177): los mensajes se aceptan y se ignoran con same, NO unhandled. Errores: signal handler que no cubre MessageAdaptionFailure relanza la causa. Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** Las senales se entregan en el hilo del actor; PostStop llega tras terminar los hijos (AT-SIG-003).

**Dependencias.** AT-SIG-001

**Equivalente Rust propuesto.** Metodo builder .on_signal(|ctx, sig| ...) sobre el Behavior devuelto por receive; Signal como enum no exhaustivo. **Diferencias observables:** Ninguna observable. Correccion al inventario viejo: Behaviors.receiveSignal solo-senales hace same en mensajes (no unhandled).

**Aserciones.**

- A1: receiveSignal instalado: Terminated(stoppedChild) llega y se maneja — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/OnSignalSpec.scala:29-43

- A2: PostStop llega al handler con 0 y 2 hijos — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/OnSignalSpec.scala:63-70

- A3: PostStop llega al guardian del ActorSystem — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/OnSignalSpec.scala:72-81

- A4: receiveSignal que no cubre MessageAdaptionFailure => el actor se detiene; cubriendolo, sobrevive — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/AdaptationFailureSpec.scala:69-72, 80-83, 89-127

- A5: Behaviors.receiveSignal consume mensajes devolviendo same; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Behaviors.scala:177-178

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): receiveSignal con PostStop/PreRestart/Terminated sobre un actor con hijos

**Decisiones.** ninguna (sin decision abierta)

## AT-BEH-010 — AbstractBehavior (scaladsl, estilo orientado a objetos)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** AbstractBehavior (estilo orientado a objetos): clase con onMessage y onSignal (default: PartialFunction.empty; MessageAdaptionFailure se relanza y el resto es unhandled). Debe crearse dentro de setup pasando su propio ActorContext; si se usa otro contexto, el primer receive/receiveSignal lanza IllegalStateException; devolver `this` equivale a same.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/AbstractBehavior.scala:21-105; actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/AbstractBehavior.scala:48-51 (context null); actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/AbstractBehavior.scala:82-88 (checkRightContext); docs/src/main/paradox/typed/fault-tolerance.md:76-82 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Errores (texto exacto): new AbstractBehavior(null) => IllegalArgumentException("context must not be null. Wrap in Behaviors.setup and pass the context to the constructor of AbstractBehavior."); contexto ajeno en receive/receiveSignal => IllegalStateException("Actor [<self>] of AbstractBehavior class [<clase>] was created with wrong ActorContext [<self ajeno>]. Wrap in Behaviors.setup and pass the context to the constructor of AbstractBehavior."). Normales: onSignal parcial; PreRestart/PostStop llegan por onSignal. Efectos: el chequeo se hace en cada receive y receiveSignal, por lo que tambien se loguea en PostStop (2 ocurrencias en los tests).

**Orden / concurrencia / ciclo de vida.** El chequeo es previo a onMessage/onSignal; el uso del ctx ajeno desde el constructor falla con UnsupportedOperationException (ActorThreadSpec 217-241).

**Dependencias.** AT-BEH-007; AT-SUP-013

**Equivalente Rust propuesto.** trait ActorBehavior<M> { fn on_message(&mut self, ctx, msg) -> Next<M>; fn on_signal(..) -> Next<M> {default} } con estado mutable en self; el chequeo de contexto equivocado se vuelve innecesario si el contexto se pasa por parametro y no se guarda. **Diferencias observables:** Si el contexto se pasa por parametro (no se guarda en la struct) la clase de error 'wrong ActorContext' no existe.

**Aserciones.**

- A1: AbstractBehavior con el contexto del padre falla al primer mensaje con IllegalStateException "was created with wrong ActorContext" (2 ocurrencias: mensaje y PostStop) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorThreadSpec.scala:189-216

- A2: Compartir la misma instancia entre dos actores se detecta como contexto equivocado (2 ocurrencias) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorThreadSpec.scala:244-278

- A3: Usar el contexto del padre en el constructor falla con UnsupportedOperationException "Unsupported access to ActorContext" — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorThreadSpec.scala:218-242

- A4: supervise(AbstractBehavior) lanza IllegalArgumentException (tambien con contexto null) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1534-1555

- A5: detect wrong context: error IllegalStateException con mensaje que contiene "created with wrong ActorContext" y el actor termina — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1557-1580

- A6: AbstractBehavior sin override de onSignal se detiene por MessageAdaptionFailure; si maneja solo PreRestart tambien; si maneja MessageAdaptionFailure sobrevive — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/AdaptationFailureSpec.scala:73-74, 84, 89-127

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): clase AbstractBehavior de prueba creada con contexto ajeno; capturar texto exacto del error

**Decisiones.** Q-T-BEH-5

## AT-BEH-011 — Interprete de Behavior: start, canonicalize, interpretMessage, interpretSignal, existsInStack

*Visibilidad:* internal-required · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Interprete de Behavior (Behavior.start, canonicalize, interpretMessage, interpretSignal, existsInStack). Es nucleo sin API de usuario pero BehaviorTestKit lo usa. Reglas: start arranca Deferred anidados (tambien dentro de interceptores y SuperviseBehavior); canonicalize resuelve same/unhandled contra el actual y desenvuelve Deferred; ejecutar same/unhandled/deferred/supervise directamente es IllegalArgumentException; Behavior null es InvalidMessageException; el resultado de ExtensibleBehavior se arranca.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:186-323; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/BehaviorImpl.scala:29-44 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Errores (texto exacto): behavior null => InvalidMessageException("[null] is not an allowed behavior"); same/unhandled => IllegalArgumentException("cannot execute with [<toString>] as behavior"); Deferred => IllegalArgumentException("deferred [<d>] should not be passed to interpreter"); Supervise => IllegalArgumentException("supervise [<s>] should not be passed to interpreter"); existsInStack con Deferred en la pila => IllegalArgumentException("Cannot verify behavior existence when there are deferred in the behavior stack, Behavior.start the stack first. This is probably a bug, please create an issue. <d>"). interpretSignal(Terminated) con resultado Unhandled => DeathPactException(ref). Efectos: PostStop sobre StoppedBehavior invoca su callback (Behavior.scala:299-302); Ignore devuelve same; Empty devuelve unhandled; Failed devuelve si mismo.

**Orden / concurrencia / ciclo de vida.** interpretSignal lanza DeathPact despues de interpretar (para permitir supervision). start no es tail-recursivo (StackOverflowError si la pila es muy profunda).

**Dependencias.** AT-BEH-001; AC-CELL

**Equivalente Rust propuesto.** Funcion interna interpret(behavior, ctx, input) -> Result<Behavior<M>, ActorError> en el modulo runtime; null no existe en Rust. **Diferencias observables:** Rust no tiene null; el interprete devuelve Result<Behavior, ActorError>.

**Aserciones.**

- A1: interpretMessage es utilizable desde dentro de un Behavior.receive (test de compilacion) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:91-98

- A2: canonicalize: Renew devuelve el mismo Behavior; Miss unhandled; Ping same, en las 5 variantes de decoracion — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:100-134, 692-721

- A3: start arranca setup anidado dentro de transformMessages y monitor — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/DeferredSpec.scala:99-145

- A4: interpretMessage devuelve unhandled para Miss — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/BehaviorSpec.scala:259-265

- A5: Mensajes de error exactos de interpret y existsInStack; derivadas del codigo; no afirmadas por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:236-238, 285-296

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): invocar Behavior.interpretMessage/interpretSignal por jshell con cada tag y capturar excepciones

**Decisiones.** Q-T-BEH-1

## AT-BEH-012 — Validacion de Behavior inicial (validateAsInitial, isAlive, isUnhandled, isDeferred)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Validacion de Behavior inicial: validateAsInitial rechaza same y unhandled (IllegalArgumentException); stopped es valido; isAlive es falso para Stopped y Failed; isUnhandled e isDeferred consultan el tag. ActorContext.spawn y ActorSystem.apply validan el Behavior inicial.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:245-269; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorRefFactoryAdapter.scala:58-70; actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorSystem.scala:305-306; actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:92-93 (supervise.onFailure valida el envuelto) | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Errores: IllegalArgumentException("cannot use <Same|Unhandled> as initial behavior"); ActorSystem.apply con Behavior no vivo => IllegalArgumentException (require, ActorSystem.scala:304, sin mensaje del usuario: "requirement failed"). unstash con unhandled inicial => IllegalArgumentException("Cannot unstash with unhandled as starting behavior"). supervise(b) valida b con validateAsInitial. Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** Ocurre antes de crear el actor (sistema) o al instanciarlo (spawn).

**Dependencias.** AT-BEH-011

**Equivalente Rust propuesto.** Funciones publicas en el modulo behavior: is_alive, is_unhandled; validate_as_initial devuelve Result. **Diferencias observables:** Ninguna salvo el tipo de resultado (Result).

**Aserciones.**

- A1: setup(setup(same)) detiene el actor — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/DeferredSpec.scala:147-158

- A2: unstashAll(Behaviors.unhandled) lanza IllegalArgumentException — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashBufferSpec.scala:261-265

- A3: stopped como Behavior inicial del sistema es rechazado ("note that an immediately stopped ActorSystem(Behaviors.stopped) is not allowed") — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/GuardianStartupSpec.scala:76-77

- A4: ActorSystem.apply valida el Behavior inicial y exige que este vivo; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorSystem.scala:305-306

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): ActorSystem.apply con same/unhandled/stopped; ctx.spawn con same: ver donde sale la excepcion

**Decisiones.** ninguna (sin decision abierta)

## AT-BEH-013 — BehaviorInterceptor + Behaviors.intercept (aroundStart/aroundReceive/aroundSignal, interceptMessageClass)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** BehaviorInterceptor + Behaviors.intercept: envuelve un Behavior; aroundStart/aroundReceive/aroundSignal pueden transformar, filtrar o delegar via target. Solo intercepta mensajes cuya clase es asignable a interceptMessageClass (el resto pasa directo al interno); la fabrica se invoca por cada uso (estado por actor).

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/BehaviorInterceptor.scala:25-94; actor-typed/src/main/scala/org/apache/pekko/actor/typed/BehaviorInterceptor.scala:96-139 (PreStartTarget, ReceiveTarget, SignalTarget); actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Behaviors.scala:180-192; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/InterceptorImpl.scala:34-104; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/BehaviorImpl.scala:174-182 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: aroundStart por defecto = target.start(ctx); aroundSignal por defecto = target(ctx, signal); aroundReceive es abstracto; devolver Behaviors.same sin invocar target filtra el mensaje. interceptMessageClass: constructor por defecto toma el ClassTag de Outer (con boxing de primitivos, BehaviorInterceptor.scala:52-56); null => ningun mensaje pasa por aroundReceive (todos van directo al target). Errores: excepcion en el interceptor => supervision. Efectos: aroundStart puede reemplazar el Behavior arrancado (devolver stopped evita arrancar el interno).

**Orden / concurrencia / ciclo de vida.** InterceptorImpl.apply es Deferred: crea el interceptor por actor y llama preStart(ctx) (InterceptorImpl.scala:37-42, 82-85). Las senales siempre pasan por aroundSignal (InterceptorImpl.scala:101-104). Cada resultado se re-envuelve/deduplica (AT-BEH-015).

**Dependencias.** AT-BEH-015; AT-BEH-011

**Equivalente Rust propuesto.** trait BehaviorInterceptor<O, I> { fn around_start; fn around_receive(&mut self, ctx, msg: O, target) -> Behavior<I>; fn around_signal; fn is_same(&self, other: &dyn Any) -> bool }; el filtro por clase se reemplaza por una funcion de proyeccion O -> Option<I> o por TypeId. **Diferencias observables:** El filtro por clase depende de herencia JVM; en Rust se modela con TypeId/proyeccion (Q-T-BEH-3).

**Aserciones.**

- A1: Un interceptor ve 'before', luego corre el Behavior interno y despues 'after' — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/InterceptSpec.scala:109-124

- A2: aroundStart puede devolver stopped y entonces el Behavior interno nunca se arranca — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/InterceptSpec.scala:209-234

- A3: Interceptor con setup anidado: aroundReceive ve cada mensaje y el estado de la fabrica se mantiene — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/InterceptSpec.scala:236-258

- A4: Interceptor sobre setup(same) rechaza el inicial y el actor termina — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/InterceptSpec.scala:293-304

- A5: Intercepta subclases del tipo Outer (A y B de Message) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/InterceptSpec.scala:365-398

- A6: Un interceptor de protocolo (Any->InternalProtocol) transforma Command y ExternalResponse y rechaza otros con RuntimeException — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/InterceptSpec.scala:457-479

- A7: Combinable con otro interceptor, con transformMessages, con withMdc y con PoisonPillInterceptor; los mensajes no del tipo exterior saltan al interno — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/InterceptSpec.scala:457-523

- A8: BehaviorSignalInterceptor no recibe mensajes: aroundReceive lanza IllegalStateException("Unexpected message in <clase>, it should only intercept signals.") si se invoca; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/BehaviorInterceptor.scala:163-169

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): interceptor de prueba sobre un receive; comparar orden before/after y filtrado por clase

**Decisiones.** Q-T-BEH-3

## AT-BEH-014 — BehaviorSignalInterceptor

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** BehaviorSignalInterceptor: interceptor que solo ve senales (aroundSignal obligatorio); los mensajes pasan sin tocarse porque interceptMessageClass es null.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/BehaviorInterceptor.scala:140-177; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/InterceptorImpl.scala:90-99 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: aroundSignal(ctx, signal, target) debe invocar target(ctx, signal) para que el interno vea la senal. Errores: aroundReceive final lanza IllegalStateException si se llega (no debe pasar). Efectos: ninguno propio.

**Orden / concurrencia / ciclo de vida.** Ve PostStop antes del callback de stopped(cb) (IS 400-427).

**Dependencias.** AT-BEH-013

**Equivalente Rust propuesto.** Implementacion por defecto de around_receive que delega. **Diferencias observables:** Ninguna.

**Aserciones.**

- A1: El interceptor de senales ve PostStop del Behavior que para con stopped(cb) y luego corre el callback — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/InterceptSpec.scala:400-427

- A2: PoisonPillInterceptor (BehaviorSignalInterceptor) detiene el actor ante la senal PoisonPill — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/InterceptSpec.scala:306-327

- A3: Los mensajes no pasan por aroundSignal; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/InterceptorImpl.scala:94-97 + BehaviorInterceptor.scala:157

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): interceptor de senales sobre actor con PostStop/PreRestart

**Decisiones.** Q-T-BEH-3

## AT-BEH-015 — Deduplicacion de interceptores (isSame)

*Visibilidad:* internal-required · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Deduplicacion de interceptores (isSame): si el Behavior devuelto por el interno ya contiene el mismo interceptor (isSame), se conserva solo el mas interno, para no crecer la pila en definiciones recursivas. Por defecto isSame es igualdad de instancia.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/InterceptorImpl.scala:106-123; actor-typed/src/main/scala/org/apache/pekko/actor/typed/BehaviorInterceptor.scala:87-92; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/BehaviorImpl.scala:174-182; actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:228-243 (existsInStack) | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: tras cada receive/receiveSignal/start, si el resultado esta vivo y no es same/unhandled, se busca en la pila (existsInStack) un InterceptorImpl con isSame; si existe se descarta el envoltorio externo (el nuevo resultado ya lo contiene). Limites: stopped/same/unhandled no se deduplican. Errores: existsInStack con Deferred sin arrancar lanza IllegalArgumentException (por eso se arranca antes, InterceptorImpl.scala:107). Efectos: aroundReceive se ejecuta una sola vez por mensaje.

**Orden / concurrencia / ciclo de vida.** Se evalua despues de cada interpretacion en el hilo del actor.

**Dependencias.** AT-BEH-013

**Equivalente Rust propuesto.** is_same por identidad de tipo o comparacion explicita; el runtime recorre la pila antes de re-envolver. **Diferencias observables:** isSame por igualdad de tipo/valor; sin identidad de instancia necesariamente.

**Aserciones.**

- A1: Dos intercept con la misma instancia anidados: solo se intercepta una vez (before/after una vez) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/InterceptSpec.scala:126-142

- A2: Intercept recursivo con la misma instancia: cada mensaje se intercepta una sola vez — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/InterceptSpec.scala:144-166

- A3: Instancias distintas (fabrica nueva cada vez): se conservan todas, inicial y recursivo — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/InterceptSpec.scala:168-207

- A4: La pila no crece al reanidar el mismo interceptor 20 veces (SameTypeInterceptor aparece como mucho una vez en el stack) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/InterceptSpec.scala:429-454

- A5: monitor al mismo ref anidado o recursivo notifica una vez por mensaje — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/MonitorSpec.scala:39-71

- A6: transformMessages con el mismo PartialFunction no se acumula (transformCount=1; =2 tras el segundo mensaje recursivo) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TransformMessagesSpec.scala:85-140

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): contar invocaciones de aroundReceive en una definicion recursiva

**Decisiones.** Q-T-BEH-3

## AT-BEH-016 — transformMessages (Behavior.BehaviorDecorators) / narrow

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** transformMessages (Behavior.BehaviorDecorators): embudo que acepta un tipo exterior y lo transforma al interior via funcion parcial; lo no definido queda unhandled; mensajes cuya clase no es la exterior pasan sin transformar (interceptMessageClass por ClassTag); las senales no se transforman; mezclar distintos transformMessages en la misma pila lanza IllegalStateException salvo que sea la misma funcion parcial.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:153-185; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/BehaviorImpl.scala:55-56; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/InterceptorImpl.scala:206-246 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Errores (texto exacto): IllegalStateException("transformMessages can only be used one time in the same behavior stack. One defined in <loc1>, and another in <loc2>") (InterceptorImpl.scala:232-234). Normales: matcher.applyOrElse => unhandled si no definido. Efectos: ninguno. Los mensajes de otra clase llegan al interno con un cast sin chequeo (InterceptorImpl.scala:97).

**Orden / concurrencia / ciclo de vida.** El filtrado ocurre antes del Behavior interno; funciona dentro y fuera de withTimers (AT-TMR-008).

**Dependencias.** AT-BEH-013; AT-BEH-015

**Equivalente Rust propuesto.** Combinador behavior.transform_messages(|outer: O| -> Option<I>) que produce Behavior<O>; el chequeo 'mixing' se puede hacer en compilacion si cada transformacion cambia el tipo. **Diferencias observables:** Rust no admite el cast sin chequeo: los mensajes externos de otro tipo requieren enum de protocolo exterior.

**Aserciones.**

- A1: Transforma Int a String (42 llega como "42") — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TransformMessagesSpec.scala:66-72

- A2: Filtra mensajes (13 no definido => descartado como unhandled) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TransformMessagesSpec.scala:74-83

- A3: El mismo PartialFunction usado varias veces no se acumula (inicial y recursivo) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TransformMessagesSpec.scala:85-140

- A4: Mezclar dos transformMessages distintos en la misma pila termina el actor con ActorInitializationException — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TransformMessagesSpec.scala:142-158

- A5: Funciona combinado con timers internos y externos — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TransformMessagesSpec.scala:160-202

- A6: Combinable con un interceptor de protocolo: ExternalResponse se salta el transformMessages — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/InterceptSpec.scala:481-491

- A7: El texto del IllegalStateException; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/InterceptorImpl.scala:232-234

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): transformMessages[Number] del Scaladoc con BigDecimal/BigInt/Integer

**Decisiones.** Q-T-BEH-2, Q-T-BEH-3, Q-X-3

## AT-BEH-017 — Behaviors.monitor

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Behaviors.monitor(monitor, behavior): reenvia una copia de cada mensaje de la clase T (ClassTag) al ref monitor antes de procesarlo; otros tipos pasan sin monitorear. Deduplicado por ref (isSame compara actorRef).

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Behaviors.scala:194-208; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/InterceptorImpl.scala:128-149 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: monitor ! msg y luego target(ctx, msg). Limites: senales no se monitorean. Errores: ninguno propio. Efectos: un tell extra por mensaje.

**Orden / concurrencia / ciclo de vida.** El mensaje llega al monitor antes de que lo procese el Behavior interno (MonitorInterceptor.aroundReceive, InterceptorImpl.scala:138-141).

**Dependencias.** AT-BEH-013

**Equivalente Rust propuesto.** behaviors::monitor(monitor_ref: ActorRef<M>, behavior) con M: Clone. **Diferencias observables:** Requiere M: Clone; el filtro por clase es un TypeId.

**Aserciones.**

- A1: El monitor recibe cada mensaje — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/MonitorSpec.scala:28-37

- A2: Monitor anidado al mismo ref: una sola copia por mensaje — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/MonitorSpec.scala:39-53

- A3: Monitor recursivo al mismo ref: una sola copia por mensaje — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/MonitorSpec.scala:55-71

- A4: monitor sobre un setup lo arranca al crear y reenvia luego — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/DeferredSpec.scala:129-145

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): monitor sobre un echo; contar copias

**Decisiones.** Q-T-BEH-3

## AT-BEH-018 — Behaviors.logMessages + LogOptions

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Behaviors.logMessages(+LogOptions): registra mensajes y senales recibidos con plantillas "actor [{}] received message: {}" y "actor [{}] received signal: {}" (ruta del actor y valor); nivel por defecto DEBUG, logger configurable, desactivable con LogOptions.withEnabled(false). Intercepta mensajes de cualquier tipo.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Behaviors.scala:210-224; actor-typed/src/main/scala/org/apache/pekko/actor/typed/LogOptions.scala:25-95; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/InterceptorImpl.scala:154-204 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: plantilla con selfPath y mensaje/senal; nivel segun LogOptions (ERROR/WARN/INFO/DEBUG/TRACE). Limites: enabled=false no loguea. isSame compara LogOptions (igualdad por valor, un caso class): dos logMessages con las mismas opciones se deduplican. Efectos: usa LoggerFactory.getLogger(LogMessagesInterceptor) si no hay logger (InterceptorImpl.scala:174).

**Orden / concurrencia / ciclo de vida.** El log sale antes de delegar al Behavior interno; PostStop se loguea porque pasa por aroundSignal.

**Dependencias.** AT-LOG-001; AT-BEH-013

**Equivalente Rust propuesto.** Interceptor que emite tracing::event! con nivel configurable. **Diferencias observables:** Salida por tracing; MDC tomado del span (AT-BEH-019).

**Aserciones.**

- A1: Se loguea "actor [<path>] received message: Hello" a DEBUG y "received signal: PostStop" al parar — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/LogMessagesSpec.scala:36-48

- A2: Con LogOptions().withLevel(INFO) sale a INFO — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/LogMessagesSpec.scala:50-63

- A3: Con withLogger(system.log) usa ese logger — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/LogMessagesSpec.scala:65-79

- A4: Con withEnabled(false) no se loguea (0 ocurrencias) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/LogMessagesSpec.scala:81-94

- A5: Se combina con withMdc: las entradas MDC aparecen en el log del mensaje y de la senal — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/LogMessagesSpec.scala:96-138

- A6: Loguea mensajes de cualquier tipo (13 en un Behavior[String]) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/LogMessagesSpec.scala:140-148

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): capturar la salida del logger de un actor con logMessages

**Decisiones.** Q-T-BEH-6, Q-X-7

## AT-BEH-019 — Behaviors.withMdc (estatico, por mensaje, combinado)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Behaviors.withMdc (estatico, por mensaje, combinado): agrega entradas MDC durante el procesamiento de cada mensaje y senal (las entradas por mensaje pisan a las estaticas con la misma clave); el MDC se limpia al terminar cada mensaje; un withMdc mas interno devuelto desde el Behavior reemplaza al externo; al declarar (inicial) gana el mas externo; mensajes de otra clase pasan sin MDC.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Behaviors.scala:272-323; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/WithMdcBehaviorInterceptor.scala:28-117 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: setMdcValues(static + dynamic) antes de cada aroundReceive; para aroundSignal y aroundStart solo el estatico (WithMdcBehaviorInterceptor.scala:77, 101). Limites: el aroundStart elimina interceptores MDC ya presentes en la pila (el externo gana, lineas 51-82); isSame es true para cualquier otro WithMdc (linea 85-88) => en ejecucion el ultimo gana. Efectos: MDC.clear() en finally tras cada start/receive/signal.

**Orden / concurrencia / ciclo de vida.** MDC es thread-local en SLF4J: valido solo durante el procesamiento del mensaje.

**Dependencias.** AT-LOG-002; AT-BEH-013

**Equivalente Rust propuesto.** Interceptor que abre un tracing::Span con campos (o usa log::kv) durante around_receive. **Diferencias observables:** MDC thread-local => span de tracing (Q-T-BEH-6).

**Aserciones.**

- A1: En setup el MDC dinamico esta vacio y el estatico presente; en el mensaje 1 hay txId y first; en el 2 no hay first (no se filtra entre mensajes) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.scala:342-385

- A2: withMdc anidado inicial: gana el externo ("outermost"), no aparece "innermost" — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.scala:402-422

- A3: El MDC se mantiene al cambiar de Behavior — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.scala:424-450

- A4: Si el nuevo Behavior esta envuelto en withMdc, reemplaza al anterior (mdc-version 1 a 2) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.scala:452-480

- A5: El usuario puede agregar entradas MDC propias dentro del handler ("inner-outer") — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.scala:482-499

- A6: Combinable con un interceptor de protocolo: mdcForMessage no se invoca para ExternalResponse — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/InterceptSpec.scala:493-511

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): log de un actor con withMdc capturado con un appender de prueba

**Decisiones.** Q-T-BEH-6, Q-X-7

## AT-BEH-020 — Behaviors.withTimers (fabrica de TimerScheduler)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Behaviors.withTimers(factory): entrega un TimerScheduler ligado al actor (setup + timer compartido del contexto); ver AT-TMR-*.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Behaviors.scala:263-271; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/TimerSchedulerImpl.scala:39-49; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorContextImpl.scala:116-133 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: withTimers[T] = setup(wrapWithTimers(factory)); varios withTimers anidados reciben el mismo scheduler (ctx.timer). Errores: IllegalArgumentException("timers not supported with [<clase del ctx>]") si el contexto no es ActorContextImpl (TimerSchedulerImpl.scala:47). Efectos: ver TMR.

**Orden / concurrencia / ciclo de vida.** Ver TimerSpec (AT-TMR-001).

**Dependencias.** AT-TMR-001

**Equivalente Rust propuesto.** behaviors::with_timers(|timers| ...). **Diferencias observables:** Ninguna observable (ver AT-TMR-001).

**Aserciones.**

- A1: Timers anidados (externo e interno) ambos disparan — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TimerSpec.scala:256-281

- A2: Los timers se mantienen al cambiar de Behavior — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TimerSpec.scala:283-306

- A3: El texto del IllegalArgumentException para contextos no soportados; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/TimerSchedulerImpl.scala:48

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): ver AT-TMR-001

**Decisiones.** Q-T-TMR-1

## AT-BEH-021 — Behaviors.withStash (fabrica de StashBuffer)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Behaviors.withStash(capacity)(factory): setup que entrega un StashBuffer de capacidad fija ligado al actor; ver AT-STASH-*.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Behaviors.scala:42-51; actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/StashBuffer.scala:24-35 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: withStash(c)(f) = setup(ctx => f(StashBuffer(ctx, c))). Limites: capacity no se valida en el constructor (negativo no lanza; isFull seria size==capacity). Efectos: ver STASH.

**Orden / concurrencia / ciclo de vida.** El buffer vive mientras viva el Behavior setup; sobrevive a los cambios de Behavior solo si se captura en el closure.

**Dependencias.** AT-STASH-001

**Equivalente Rust propuesto.** behaviors::with_stash(capacity, |stash| ...). **Diferencias observables:** Ninguna.

**Aserciones.**

- A1: withStash(10) permite stash/unstashAll dentro del actor (Immutable y Mutable) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashSpec.scala:221-262

- A2: capacity no validada en el constructor; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/StashBufferImpl.scala:44-46

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): ver AT-STASH-001

**Decisiones.** ninguna (sin decision abierta)

## AT-BEH-022 — Behavior.narrow (contravarianza simulada)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Behavior.narrow[U <: T]: estrechar el tipo de mensaje de un Behavior es siempre seguro (cast sin costo; mismo objeto). Simula la contravarianza.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:52-64 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: devuelve this. Limites: en JVM no hay cambio de identidad. Errores: ninguno. Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AT-REF-003

**Equivalente Rust propuesto.** Sin subtipado en Rust: narrow pasa a ser una conversion explicita via From/Into entre enums de protocolo (Behavior<Sub> a partir de Behavior<Super> con impl From<Sub> for Super). **Diferencias observables:** Cambia de 'cast gratis' a 'adaptacion explicita' con costo; ver Q-T-BEH-4.

**Aserciones.**

- A1: Un Behavior[Any] estrechado a Behavior[Command] se supervisa y recibe un String desde unsafeUpcast — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1504-1532

- A2: intercept(...).narrow convierte el interceptor Any-to-Msg en Behavior[Msg] — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/InterceptSpec.scala:329-363

**Oraculo.** no aplica: es un cast sin comportamiento runtime; el contrato es de tipos.

**Decisiones.** Q-T-BEH-4

## AT-BEH-023 — Reutilizacion de valores Behavior

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Reutilizacion de valores Behavior: un mismo valor Behavior puede usarse para varios actores/arranques (BehaviorSpec Reuse) y para el reinicio por supervision; el estado mutable debe vivir en setup.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:30-43; docs/src/main/paradox/typed/fault-tolerance.md:76-82 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: el valor no guarda estado entre usos. Errores: reutilizar una instancia AbstractBehavior en dos actores se detecta (AT-BEH-010). Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AT-BEH-007

**Equivalente Rust propuesto.** Behavior<M>: Clone (via Arc internamente) o fabricas Fn. **Diferencias observables:** Behavior debe ser Clone (Arc) o fabrica Fn.

**Aserciones.**

- A1: Dos BehaviorTestKit sobre el mismo valor Behavior parten del mismo estado inicial — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/BehaviorSpec.scala:334-338

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): spawn de dos actores con el mismo valor Behavior

**Decisiones.** Q-T-BEH-1

## AT-BEH-024 — Behavior fallido (FailedBehavior) y su tratamiento en el adaptador

*Visibilidad:* internal-required · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Cuando una supervision Stop (o un limite de restart agotado en backoff) captura una excepcion, devuelve BehaviorImpl.failed(cause) en lugar de relanzar. El Behavior fallido no esta vivo (isAlive false). El ActorAdapter lo trata en next(): si rethrowTypedFailure (hijo typed de un padre typed) lanza TypedActorFailedException(cause) para que el padre rellene ChildFailed; si no (guardian, systemActorOf) hace context.stop(self).

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/BehaviorImpl.scala:71, 94-96; actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:313-314; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:41-49, 168-172; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:139-151, 159-167 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: interpretar FailedBehavior devuelve el mismo Behavior (Behavior.scala:303-304). Errores: TypedActorFailedException(cause) es una RuntimeException interna que no se loguea de nuevo (ya lo logueo la supervision). Efectos: el actor termina; los hijos se paran y el padre observador recibe ChildFailed(ref, cause).

**Orden / concurrencia / ciclo de vida.** Se produce en el hilo del actor al finalizar el handler que lanzo. ActorAdapter.next no reemplaza `behavior` por el Failed: la variable conserva el Behavior previo (con la supervision), asi que ActorAdapter.postStop entrega PostStop a ese Behavior (ActorAdapter.scala:325-335); por eso el probe ve PostStop del Behavior que fallo.

**Dependencias.** AT-SUP-003; AT-SUP-001; AT-SIG-005; AT-SUP-022

**Equivalente Rust propuesto.** Behavior::Failed(ActorError) interno (no exportado); el runtime lo convierte en terminacion y ChildFailed. **Diferencias observables:** En Rust el equivalente es el resultado Err del handler con la causa, convertido por el runtime en terminacion y ChildFailed.

**Aserciones.**

- A1: Sin supervise, la excepcion detiene el actor, se entrega PostStop y el padre observador recibe ChildFailed con la causa — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/WatchSpec.scala:100-128

- A2: Con supervise(stop) explicito el padre tambien recibe ChildFailed con la misma causa — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/WatchSpec.scala:130-162

- A3: En BehaviorTestKit la excepcion no supervisada se propaga y el Behavior recibe PostStop — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:131-139

- A4: TypedActorFailedException solo se lanza si rethrowTypedFailure; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:172-176

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): padre que observa un hijo que lanza; comparar causa en ChildFailed

**Decisiones.** Q-X-5

## AT-SIG-001 — Signal (trait) y entrega confiable de senales

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Signal (trait abierto) es la base de las senales del sistema; se entregan al Behavior de forma confiable (a diferencia del at-most-once de los mensajes). Las senales publicas son PreRestart, PostStop, Terminated, ChildFailed y MessageAdaptionFailure; hay internas (PoisonPill, ScheduledRestart, ResetRestartCount).

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/MessageAndSignals.scala:28-35; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/PoisonPill.scala:30-37; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:202-207 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: toda senal llega al receiveSignal del Behavior vigente. Limites: las senales no pasan por el filtro interceptMessageClass; si no se manejan, el default las deja unhandled (salvo MessageAdaptionFailure que se relanza, y Terminated que produce DeathPactException). Errores: ver filas especificas. Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** Una senal se procesa en el mismo hilo y cola logica que los mensajes del actor; durante un restart con backoff las senales se acumulan en el stash del supervisor (AT-SUP-009).

**Dependencias.** AC-CELL; AC-DW

**Equivalente Rust propuesto.** enum Signal { PreRestart, PostStop, Terminated(ActorRefAny), ChildFailed{..}, MessageAdaptionFailure(Error), Custom(Box<dyn Any>) } #[non_exhaustive]. **Diferencias observables:** Trait abierto en Scala; enum con variante opaca (Custom/Internal) en Rust.

**Aserciones.**

- A1: Un Behavior recibe PostStop, PreRestart y Terminated tambien despues de procesar un mensaje, y un mensaje despues de Terminated — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/BehaviorSpec.scala:211-237

- A2: Las senales siguen el mismo despacho que los mensajes del adaptador (ActorAdapter.aroundReceive); derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:87-127

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): enviar cada senal a un Behavior con receiveSignal que registre

**Decisiones.** Q-T-SIG-1

## AT-SIG-002 — PreRestart

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** PreRestart: se entrega al Behavior que fallo antes de reemplazarlo en un restart; el Behavior devuelto se ignora; una excepcion del manejador se registra ("failure during PreRestart") y no impide el restart; no se emite PostStop en un restart de supervision.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/MessageAndSignals.scala:37-46; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:317-355; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:313-319; docs/src/main/paradox/typed/fault-tolerance.md:133-146 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: target.signalRestart(ctx) interpreta PreRestart sobre el Behavior (o el Behavior de la UnstashException). Errores: NonFatal en el manejador => ctx.log.error("failure during PreRestart", ex) y el restart sigue (Supervision.scala:344-347). Efectos: cancelAllTimers antes (Supervision.scala:334).

**Orden / concurrencia / ciclo de vida.** PreRestart -> (hijos parados si stopChildren) -> restart con nuevo Behavior inicial; PostStop NO se emite (docs fault-tolerance.md:145-146).

**Dependencias.** AT-SUP-005; AC-SUP

**Equivalente Rust propuesto.** Signal::PreRestart. **Diferencias observables:** Ninguna.

**Aserciones.**

- A1: restart: el probe recibe ReceivedSignal(PreRestart) y el estado vuelve a 0 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:518-532

- A2: PreRestart se entrega aunque el Behavior ya haya recibido mensajes — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/BehaviorSpec.scala:223-225

- A3: restart con limit: dos PreRestart y a la tercera falla PostStop — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:534-554

- A4: Tras falla al desapilar, PreRestart va al ultimo Behavior desapilado (pre-restart-2) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashSpec.scala:490-525

- A5: El texto "failure during PreRestart" al fallar el manejador; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:350

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): actor con supervise(restart) y receiveSignal que registra PreRestart/PostStop

**Decisiones.** ninguna (sin decision abierta)

## AT-SIG-003 — PostStop

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** PostStop: se emite cuando el actor y todos sus hijos (transitivamente) terminaron; Terminated llega a los watchers solo despues; no se emite en restart; un Deferred sin arrancar no recibe PostStop.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/MessageAndSignals.scala:48-63; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:329-340; docs/src/main/paradox/typed/actor-lifecycle.md:159-189 (leido: confirma que PostStop se emite al parar y que NO se emite en un restart; los ejemplos GracefulStopDocSpec/Test son de documentacion y no afirman mas) | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: postStop del adaptador cancela timers, interpreta PostStop sobre el Behavior vigente (no arranca Deferred) y lo reemplaza por Stopped. Efectos: ctx.clearMdc.

**Orden / concurrencia / ciclo de vida.** Hijos paran primero (classic), luego ActorAdapter.postStop, luego Terminated a watchers.

**Dependencias.** AC-CELL; AC-DW

**Equivalente Rust propuesto.** Signal::PostStop. **Diferencias observables:** Ninguna.

**Aserciones.**

- A1: PostStop llega tras el voluntario stopped — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:159-177

- A2: Con 0 y 2 hijos, el padre recibe PostStop — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/OnSignalSpec.scala:63-70

- A3: Los PostStop de los hijos preceden al callback del padre (stopped(cb)) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/GracefulStopSpec.scala:30-58

- A4: Un actor que para por excepcion sin supervision recibe PostStop — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:307-347

- A5: Terminated llega a los watchers despues de PostStop y de los hijos (Scaladoc); no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/MessageAndSignals.scala:48-53

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): padre con 2 hijos y watcher: ordenar eventos PostStop/Terminated

**Decisiones.** ninguna (sin decision abierta)

## AT-SIG-004 — Terminated (watch)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Terminated(ref): lifecycle signal al observar (context.watch) un actor que termino. DeathWatch es idempotente; la notificacion llega aunque el registro sea posterior a la terminacion; Terminated.equals/hashCode se basan en ref.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/MessageAndSignals.scala:65-96; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:94-101 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: classic.Terminated(ref) => Terminated(ActorRefAdapter(ref)) si no hay causa registrada. Limites: ChildFailed es un subtipo; Terminated.equals usa el patron Terminated(`ref`) (MessageAndSignals.scala:90-93) y por eso Terminated(r) == ChildFailed(r, c) es true mientras ChildFailed(r,c) == Terminated(r) es false (ChildFailed.equals exige ChildFailed con la misma causa, lineas 115-118). toString: "Terminated(<ref>)". Errores: ninguno. Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** Se entrega tras PostStop del observado y sus hijos; el adaptador entrega primero el Terminated a ctx en el hilo del actor observador.

**Dependencias.** AT-CTX-005; AC-DW

**Equivalente Rust propuesto.** Signal::Terminated(ActorRef<Untyped>) con PartialEq por path+uid. **Diferencias observables:** Rust: Signal::Terminated con PartialEq por path+uid; decidir si se conserva la asimetria (Q-T-SIG-1).

**Aserciones.**

- A1: watch de un hijo: al parar llega Terminated(ref) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:375-405

- A2: watch tras la terminacion tambien entrega Terminated — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:407-441

- A3: watch idempotente: dos watch y una sola terminacion — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/WatchSpec.scala:206-214

- A4: El observador recibe Terminated(terminator) con igualdad por ref — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/WatchSpec.scala:88-95

- A5: Terminated(r) hace match de un ChildFailed (pattern matching) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TerminatedSpec.scala:25-37

- A6: Terminated(r) == ChildFailed(r,c) es true pero ChildFailed(r,c) == Terminated(r) es false; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/MessageAndSignals.scala:92-95, 115-118

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): comparar igualdad Terminated/ChildFailed en ambos sentidos

**Decisiones.** Q-T-SIG-1

## AT-SIG-005 — ChildFailed

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** ChildFailed(ref, cause): subtipo de Terminated entregado solo cuando el hijo observado directo se detuvo por una falla; contiene la causa. Si el Behavior no distingue, se maneja como Terminated.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/MessageAndSignals.scala:98-121; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:94-100; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:236-277; docs/src/main/paradox/typed/fault-tolerance.md:149-162 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: failures[ref] = causa; en el Terminated posterior se emite ChildFailed(ref, ex) y se borra la entrada. Limites: solo hijos directos (supervisorStrategy del padre); si el padre no hizo watch, no hay causa y llega nada (no hay Terminated tampoco). toString: "ChildFailed(<ref>,<clase de la causa>)" (solo el nombre de clase). Errores: ninguno. Efectos: el padre typed decide Stop para hijos typed (ActorAdapter.scala:258-259).

**Orden / concurrencia / ciclo de vida.** La causa se registra en el hilo del padre cuando el hijo falla, antes del Terminated.

**Dependencias.** AT-SIG-004; AT-SUP-001; AC-SUP; AC-DW

**Equivalente Rust propuesto.** Signal::ChildFailed{ child, cause: Arc<dyn Error + Send + Sync> }; un helper as_terminated() para tratar ambos igual. **Diferencias observables:** Rust: variante con campos o helper as_terminated().

**Aserciones.**

- A1: ChildFailed hace match por Terminated(r) y por ChildFailed(ref,e) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TerminatedSpec.scala:25-44

- A2: Hijo sin supervise lanza: el padre recibe ChildFailed con la misma excepcion — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/WatchSpec.scala:100-128

- A3: Hijo con supervise(stop): el padre recibe ChildFailed con la excepcion original — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/WatchSpec.scala:130-162

- A4: toString de ChildFailed imprime solo el nombre de clase de la causa; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/MessageAndSignals.scala:113

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): padre que observa a hijo que falla; comparar causa y toString

**Decisiones.** Q-T-SIG-1

## AT-SIG-006 — DeathPactException por Terminated no manejado

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** DeathPactException por Terminated no manejado: si el Behavior devuelve unhandled para un Terminated, Behavior.interpretSignal lanza DeathPactException(ref) (que la supervision puede manejar); sin supervision el actor falla y para.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:280-287; actor-typed/src/main/scala/org/apache/pekko/actor/typed/MessageAndSignals.scala:18-27; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:224-234 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Errores (texto exacto): DeathPactException extends RuntimeException("death pact with <ref> was triggered") (MessageAndSignals.scala:22-23); ActorAdapter.unhandled tambien lanza DeathPactException para Terminated que llegue (linea 222-225), pero normalmente no se alcanza. Efectos: el actor termina; el padre que lo observa recibe Terminated/ChildFailed.

**Orden / concurrencia / ciclo de vida.** Se lanza dentro de interpretSignal, antes de que ActorAdapter.next vea el resultado, por eso es supervisable.

**Dependencias.** AT-SIG-004; AT-SUP-014

**Equivalente Rust propuesto.** Error tipado ActorError::DeathPact(ActorRefAny). **Diferencias observables:** ActorError::DeathPact(ref) con el mismo texto.

**Aserciones.**

- A1: Un actor que observa y no maneja Terminated falla con DeathPactException y su padre recibe Failed — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/WatchSpec.scala:165-204

- A2: receiveSignal que devuelve unhandled para Terminated: se loguea DeathPactException y el actor termina tras PostStop — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:483-533

- A3: supervise(...).onFailure[DeathPactException](restart): el actor sobrevive y responde — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1415-1446

- A4: El mensaje "death pact with <ref> was triggered"; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/MessageAndSignals.scala:22-23

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): padre-hijo sin manejo de Terminated; capturar mensaje exacto

**Decisiones.** ninguna (sin decision abierta)

## AT-SIG-007 — MessageAdaptionFailure

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** MessageAdaptionFailure(exception): si la funcion de un message adapter, pipeToSelf o ask lanza una excepcion NonFatal, se entrega esta senal al Behavior; el default la relanza (la supervision decide); el usuario puede manejarla para ignorar la falla.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/MessageAndSignals.scala:123-127; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:202-222; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/BehaviorImpl.scala:73-76; actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/AbstractBehavior.scala:97-104 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: withSafelyAdapted captura NonFatal => handleSignal(MessageAdaptionFailure(ex)). Limites: si el handler no relanza, el actor sigue y el mensaje se descarta; los demas Behaviors (empty, ignore, receiveSignal que lo captura, AbstractBehavior que lo captura) no mueren. Errores: adaptador que devuelve null => se ignora con warning (CTX-014). Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** La senal se procesa en el hilo del actor, en el orden en que llega el mensaje adaptado.

**Dependencias.** AT-CTX-011; AT-SUP-015

**Equivalente Rust propuesto.** Signal::MessageAdaptionFailure(Arc<dyn Error>); las funciones de adaptacion devuelven Result en lugar de lanzar (y panics se capturan con catch_unwind). **Diferencias observables:** Las funciones de adaptacion devuelven Result; panics => Q-X-5.

**Aserciones.**

- A1: receive, receiveMessage, receivePartial vacio, receiveSignal sin captura, AbstractBehavior vacio y con otras senales: el actor termina — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/AdaptationFailureSpec.scala:64-101

- A2: empty, ignore, receiveSignal que captura MessageAdaptionFailure y AbstractBehavior que la maneja: el actor sigue vivo — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/AdaptationFailureSpec.scala:77-85, 103-128

- A3: Excepcion en el adapter (tercer mensaje): el actor se detiene tras recibir los dos primeros — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/MessageAdapterSpec.scala:213-256

- A4: La falla de un adapter queda bajo la supervision del actor (nested y flatten) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1582-1655

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): message adapter que lanza, con cada Behavior de la lista

**Decisiones.** Q-X-5

## AT-SIG-008 — PoisonPill (senal interna) y PoisonPillInterceptor

*Visibilidad:* internal-required · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** PoisonPill (senal interna) y PoisonPillInterceptor: una senal que, si no la maneja el Behavior objetivo (resultado unhandled), detiene el actor via el interceptor; la usa cluster-sharding-typed para pasivar entidades (fuera de alcance de este modulo).

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/PoisonPill.scala:25-65 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: signal PoisonPill => target(ctx, p); si el resultado es unhandled => Behaviors.stopped; si no, el Behavior devuelto. Otras senales pasan. isSame: cualquier otro PoisonPillInterceptor (uno por pila). Efectos: ninguno propio.

**Orden / concurrencia / ciclo de vida.** Se evalua en aroundSignal; los mensajes no pasan por aqui.

**Dependencias.** AT-BEH-014

**Equivalente Rust propuesto.** Variante interna del enum Signal; no exportar. **Diferencias observables:** Variante interna del enum Signal; no exportar.

**Aserciones.**

- A1: ref.unsafeUpcast[Any] ! PoisonPill detiene un Behavior envuelto sin manejo explicito — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/InterceptSpec.scala:306-327

- A2: PoisonPillInterceptor se combina con un interceptor de protocolo y detiene el actor — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/InterceptSpec.scala:513-522

- A3: Si el Behavior objetivo maneja PoisonPill, su resultado se respeta; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/PoisonPill.scala:54-58

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): enviar PoisonPill por unsafeUpcast a un actor con el interceptor

**Decisiones.** ninguna (sin decision abierta)

## AT-CTX-001 — self y system (accesibles desde otros hilos)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** context.self es la identidad ligada a esta encarnacion del actor; context.system es el ActorSystem. Ambos son thread-safe (accesibles desde Futures y otros hilos).

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/ActorContext.scala:62-78; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorContextAdapter.scala:55-56 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: self y system son val finales del ActorContextAdapter. Limites: un actor recreado con el mismo nombre tiene otro uid y por lo tanto otro ref. Errores: ninguno. Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** Disponibles desde setup; seguros desde otros hilos.

**Dependencias.** AT-REF-001; AT-SYS-001; AC-CELL

**Equivalente Rust propuesto.** ctx.self_ref() -> ActorRef<M> (Clone + Send); ctx.system() -> ActorSystemHandle. **Diferencias observables:** Ninguna observable.

**Aserciones.**

- A1: context.system y context.self coinciden con el sistema y el actor externos — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:535-548

- A2: self/system son accesibles desde fuera del hilo del actor sin error; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorContextAdapter.scala:55-56

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): comparar self/system desde un Future

**Decisiones.** ninguna (sin decision abierta)

## AT-CTX-002 — spawn / spawnAnonymous con Props

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** spawn(behavior, name, props) / spawnAnonymous(behavior, props): crea un hijo con el Behavior y los Props dados. El nombre debe ser unico entre hermanos (el error de nombre lo da el clasico); el Behavior inicial se valida; el despliegue remoto esta prohibido (ConfigurationException "Remote deployment not allowed for typed actors"); no thread-safe.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/ActorContext.scala:127-142; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorContextAdapter.scala:75-83; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorRefFactoryAdapter.scala:27-71; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/PropsAdapter.scala:32-72 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: devuelve ActorRef[U]; el hijo queda supervisado por el ActorAdapter del padre con rethrowTypedFailure=true (hijos typed paran con Stop, AT-SUP-001). Errores: nombre duplicado/invalido => excepciones clasicas (InvalidActorNameException, AC-CELL); despliegue remoto => ConfigurationException("Remote deployment not allowed for typed actors") (ActorRefFactoryAdapter.scala:27, 36-37); fuera del hilo => UnsupportedOperationException (AT-CTX-016). Efectos: PropsAdapter aplica dispatcher, mailbox y ActorTags; siempre Deploy.local.

**Orden / concurrencia / ciclo de vida.** El Behavior del hijo arranca en su preStart (asincrono respecto del padre).

**Dependencias.** AT-DSP-001; AC-CELL; AC-PATH; AC-SUP

**Equivalente Rust propuesto.** ctx.spawn(behavior, name, props) -> Result<ActorRef<U>, SpawnError>; spawn_anonymous genera nombre $a, $b... **Diferencias observables:** Los errores de nombre y de despliegue se devuelven como SpawnError (Q-T-CTX-3).

**Aserciones.**

- A1: Un hijo con supervise(restart) reinicia (recibe PreRestart) y sigue respondiendo; context.stop(child) lo detiene — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:179-225

- A2: context.stop de un hijo anonimo lo detiene — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:227-253

- A3: context.child("B") devuelve el hijo creado con spawn(..., "B") y context.children lo incluye — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:550-576

- A4: PropsAdapter fuerza Deploy.local y traduce DispatcherSelector/MailboxSelector/ActorTags; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/PropsAdapter.scala:39-60

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): dos spawn con el mismo nombre; capturar excepcion y texto

**Decisiones.** Q-T-CTX-3

## AT-CTX-003 — children / child(name)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** children / child(name): lista los hijos vivos (sin orden particular) y busca por nombre; los refs son ActorRef[Nothing]; no thread-safe.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/ActorContext.scala:110-125; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorContextAdapter.scala:67-74 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: child(name) devuelve Option y children la coleccion de hijos clasicos adaptados (classicContext.children.map(ActorRefAdapter)). Errores: fuera del hilo del actor => UnsupportedOperationException (checkCurrentActorThread). Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** Refleja los hijos vivos en el momento de la llamada.

**Dependencias.** AT-CTX-002; AC-CELL

**Equivalente Rust propuesto.** ctx.children() -> impl Iterator<Item=ActorRef<Untyped>>; ctx.child(name) -> Option<..>. **Diferencias observables:** Tipo de ref de hijo opaco.

**Aserciones.**

- A1: child("A") antes de crear devuelve vacio, children devuelve el hijo B con nombre B — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:550-576

- A2: children/child llaman a checkCurrentActorThread; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorContextAdapter.scala:67-74

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): children tras spawn y tras stop del hijo

**Decisiones.** Q-X-3

## AT-CTX-004 — stop(child)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** stop(child): fuerza la terminacion de un hijo directo tras su mensaje actual; solo hijos directos; pasar self o un actor que no es hijo lanza IllegalArgumentException; un hijo ya parado se ignora; un message adapter (FunctionRef) se remueve.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/ActorContext.scala:154-163; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorContextAdapter.scala:85-112 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Errores (texto exacto): self => IllegalArgumentException("Only direct children of an actor can be stopped through the actor context, but you tried to stop [<self>] by passing its ActorRef to the `stop` method. Stopping self has to be expressed as explicitly returning a Stop Behavior with `Behaviors.stopped`."); no hijo => IllegalArgumentException("Only direct children of an actor can be stopped through the actor context, but [<ref>] is not a child of [<self>]. Stopping other actors has to be expressed as an explicit stop message that the actor accepts.") (CTXA 92-102). Limites: hijo ya detenido => sin efecto; FunctionRef => cell.removeFunctionRef. Efectos: classicContext.stop(c).

**Orden / concurrencia / ciclo de vida.** El hijo procesa su mensaje en curso y luego para (classic stop); PostStop del hijo, luego Terminated al padre si lo observa.

**Dependencias.** AT-CTX-002; AC-CELL

**Equivalente Rust propuesto.** ctx.stop(&child) -> Result<(), StopError::NotAChild>. **Diferencias observables:** NotAChild reemplaza a IllegalArgumentException conservando los textos (Q-T-CTX-3).

**Aserciones.**

- A1: context.stop(child) de un hijo lo detiene y el padre observa su terminacion — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:227-253

- A2: context.stop sobre un actor que no es hijo lanza IllegalArgumentException y el actor sigue funcionando — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:349-373

- A3: Parar un hijo ya parado no falla (se puede repetir) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:671-688

- A4: Los textos exactos de los dos IllegalArgumentException; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorContextAdapter.scala:100-110

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): context.stop con self y con un actor ajeno; capturar mensajes

**Decisiones.** Q-T-CTX-3

## AT-CTX-005 — watch(other)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** watch(other): registra la notificacion Terminated; idempotente si no se mezcla con watchWith; si el mismo sujeto se observo antes con watchWith lanza IllegalStateException.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/ActorContext.scala:165-178; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorContextAdapter.scala:114-117; actor/src/main/scala/org/apache/pekko/actor/dungeon/DeathWatch.scala:126-132 (checkWatchingSame) | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Errores (texto exacto, de classic): IllegalStateException("Watch(<self>, <ref>) termination message was not overwritten from [<previous>] to [<newMessage>]. If this was intended, unwatch first before using `watch` / `watchWith` with another message."). Limites: watch dos veces del mismo sujeto no falla y entrega un solo Terminated. Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** Si el sujeto ya termino, el Terminated llega igualmente (garantia de DeathWatch).

**Dependencias.** AT-SIG-004; AC-DW

**Equivalente Rust propuesto.** ctx.watch(&ref) -> Result<(), WatchError>. **Diferencias observables:** El conflicto watch/watchWith se devuelve como WatchError con el mismo texto (Q-T-CTX-3).

**Aserciones.**

- A1: watch de un hijo antes de su terminacion entrega Terminated — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:375-405

- A2: watch de un hijo ya terminado entrega Terminated — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:407-441

- A3: watch idempotente — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/WatchSpec.scala:206-214

- A4: watch tras watchWith sobre el mismo sujeto falla con IllegalStateException que contiene "termination message was not overwritten" y el supervisor para al actor — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/WatchSpec.scala:336-347

- A5: watchWith tras watch sobre el mismo sujeto falla igual — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/WatchSpec.scala:361-372

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): watch/watchWith/unwatch en las combinaciones del WatchSpec

**Decisiones.** Q-T-CTX-3

## AT-CTX-006 — watchWith(other, msg)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** watchWith(other, msg): en vez de Terminated entrega msg propio como mensaje; idempotente con el mismo msg; con otro msg o tras watch lanza IllegalStateException; para cambiarlo hay que hacer unwatch primero. La estrategia de restart revoca el watchWith previo sobre los hijos para esperar su terminacion.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/ActorContext.scala:180-193; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorContextAdapter.scala:118-121; docs/src/main/paradox/typed/actor-lifecycle.md:202-215 (leido: describe watchWith como alternativa a Terminated; sin reglas adicionales); actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:425-432 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Errores: ver AT-CTX-005 (mismo mensaje clasico). Limites: msg es del tipo T del actor. Efectos: al terminar el sujeto se encola msg (no una senal).

**Orden / concurrencia / ciclo de vida.** msg llega por la cola de mensajes del actor, no por receiveSignal.

**Dependencias.** AT-CTX-005; AC-DW

**Equivalente Rust propuesto.** ctx.watch_with(&ref, msg: M) con M: Clone? (se guarda hasta la terminacion). **Diferencias observables:** Requiere msg: Clone o almacenarlo hasta la terminacion.

**Aserciones.**

- A1: watchWith entrega CustomTerminationMessage al parar el sujeto — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/WatchSpec.scala:234-240

- A2: watchWith repetido con el mismo mensaje es idempotente — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/WatchSpec.scala:241-250

- A3: watch seguido de unwatch + watchWith cambia el mensaje (llega el custom) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/WatchSpec.scala:252-280

- A4: unwatch + watchWith redefine el mensaje (llega CustomTerminationMessage2) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/WatchSpec.scala:282-308

- A5: watchWith con otro mensaje sin unwatch falla con IllegalStateException — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/WatchSpec.scala:349-360

- A6: Tras restart el watchWith del padre sobre el hijo se revoca: no llega Ping(1) pero sigue Ping(0) de otro sujeto — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:624-666

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): secuencias del WatchSpec

**Decisiones.** Q-T-CTX-3

## AT-CTX-007 — unwatch(other)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** unwatch(other): revoca el registro de watch/watchWith; despues no llega Terminated ni el mensaje de watchWith.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/ActorContext.scala:195-202; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorContextAdapter.scala:122-125 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: classicContext.unwatch. Limites: unwatch sin watch previo no falla (comportamiento clasico). Efectos: libera el registro para otro mensaje en watchWith.

**Orden / concurrencia / ciclo de vida.** Si el Terminated ya estaba encolado antes del unwatch, el clasico lo descarta (AC-DW).

**Dependencias.** AC-DW

**Equivalente Rust propuesto.** ctx.unwatch(&ref). **Diferencias observables:** Ninguna.

**Aserciones.**

- A1: watch + unwatch y luego parar al hijo: el probe no recibe nada — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:443-481

- A2: unwatch permite redefinir el mensaje de terminacion — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/WatchSpec.scala:252-308

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): watch, unwatch y parar al sujeto

**Decisiones.** ninguna (sin decision abierta)

## AT-CTX-008 — setReceiveTimeout / cancelReceiveTimeout

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** setReceiveTimeout(duration, msg) / cancelReceiveTimeout(): si no llega ningun mensaje durante el periodo se entrega msg como mensaje (no senal); el plazo se reinicia con cada mensaje que influye (los NotInfluenceReceiveTimeout no); cancelReceiveTimeout lo apaga.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/ActorContext.scala:204-221; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorContextAdapter.scala:126-137; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:102-103 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: guarda receiveTimeoutMsg y llama classicContext.setReceiveTimeout(d). Limites: cancelReceiveTimeout => setReceiveTimeout(Duration.Undefined) y msg = null. Errores: ninguno propio (checkCurrentActorThread). Efectos: classic.ReceiveTimeout se convierte en handleMessage(receiveTimeoutMsg).

**Orden / concurrencia / ciclo de vida.** Se entrega como mensaje normal; un timeout de 1 nanosegundo dispara de inmediato.

**Dependencias.** AC-CELL (ReceiveTimeout clasico); AC-SCH

**Equivalente Rust propuesto.** ctx.set_receive_timeout(Duration, msg: M) con M: Clone. **Diferencias observables:** msg: Clone; ver Q-T-TMR-2.

**Aserciones.**

- A1: setReceiveTimeout(1.nano, msg) entrega el mensaje de timeout — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:578-595

- A2: setReceiveTimeout(1 minute) no interfiere con scheduleOnce ni con mensajes normales — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:597-623

- A3: La regla de reinicio del plazo (cada mensaje lo reinicia salvo los `NotInfluenceReceiveTimeout`; cambiar el timeout dentro del handler de un mensaje marcado surte efecto) la fija el `ActorCell` clasico y la afirma `ReceiveTimeoutSpec` clasico, no un spec typed — actor-tests/src/test/scala/org/apache/pekko/actor/ReceiveTimeoutSpec.scala:180-327 (detalle por prueba en AT-TMR-007.2, A1-A4). Los dos tests typed de A1/A2 no ejercitan el reinicio por trafico: A1 solo verifica entrega con 1 ns y A2 solo verifica que un plazo de 1 minuto no dispara. Caracterizacion propuesta (oraculo, tiempos con reloj inyectable): timeout de 50 ms con un mensaje normal cada 20 ms => no dispara; sin trafico => dispara y se vuelve a disparar periodicamente cada 50 ms (la propia entrega de `ReceiveTimeout` no es del marker, asi que el `ActorCell` reprograma: `ReceiveTimeout.scala:40-42`), hasta `cancelReceiveTimeout`.

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): timeout de 50 ms con trafico periodico de 20 ms y sin trafico

**Decisiones.** Q-T-TMR-2

## AT-CTX-009 — scheduleOnce(delay, target, msg)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** scheduleOnce(delay, target, msg): programa un envio unico a cualquier ActorRef y devuelve Cancellable; thread-safe.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/ActorContext.scala:223-232; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorContextAdapter.scala:139-141 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: classicContext.system.scheduler.scheduleOnce(delay, toClassic(target), msg) con el dispatcher del actor. Limites: target debe ser un ref adaptado (ActorRefAdapter.toClassic lanza UnsupportedOperationException("Only adapted classic ActorRefs permissible (<ref> of class <clase>)") en otro caso). Efectos: cancel antes del disparo evita el envio.

**Orden / concurrencia / ciclo de vida.** No pasa por el TimerScheduler (sin claves/generaciones).

**Dependencias.** AC-SCH

**Equivalente Rust propuesto.** ctx.schedule_once(Duration, &target, msg) -> Cancellable. **Diferencias observables:** Cancellable propio; la semantica de cancelar es la misma.

**Aserciones.**

- A1: scheduleOnce(1.nano) entrega el mensaje al probe — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:625-637

- A2: scheduleOnce(1.second) entrega "scheduled" mientras hay un receive timeout largo — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:597-623

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): scheduleOnce de 10 ms y cancelacion previa

**Decisiones.** ninguna (sin decision abierta)

## AT-CTX-010 — executionContext

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** executionContext: el ExecutionContext (dispatcher) del actor para tareas asincronas como Future; thread-safe.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/ActorContext.scala:234-241; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorContextAdapter.scala:138 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: classicContext.dispatcher. Efectos: las tareas corren en el dispatcher configurado en Props (p.ej. ping-pong-dispatcher).

**Orden / concurrencia / ciclo de vida.** Tareas en hilos del dispatcher, no del actor.

**Dependencias.** AT-DSP-002; AC-DSP

**Equivalente Rust propuesto.** ctx.executor() -> handle de runtime (p.ej. tokio::runtime::Handle) o spawner abstracto. **Diferencias observables:** Handle del runtime (Q-X-4).

**Aserciones.**

- A1: Un Future lanzado en context.executionContext corre fuera del hilo del actor y el acceso al contexto alli falla con UnsupportedOperationException — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorThreadSpec.scala:70-94

- A2: pipeToSelf procesa el resultado en el dispatcher del actor (nombre de hilo con prefijo del dispatcher) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorContextPipeToSelfSpec.scala:77-100

**Oraculo.** no aplica: es una referencia al dispatcher del runtime; ver Q-X-4.

**Decisiones.** Q-X-4

## AT-CTX-011 — messageAdapter[U](f)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** messageAdapter[U](f): crea (o reutiliza) un ref adaptador que convierte U al protocolo T del actor. Un adaptador por clase de mensaje (el nuevo reemplaza al anterior para la misma clase); se prueban en orden inverso de registro y matchean subclases; la funcion corre dentro del actor; tiene el mismo ciclo de vida que el actor; un tipo sin adaptador queda unhandled; tras la terminacion los mensajes van a dead letters.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/ActorContext.scala:268-294; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorContextImpl.scala:308-333; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:108-115, 182-196; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorContextAdapter.scala:142-148; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/InternalMessage.scala:17-40 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: internalMessageAdapter reemplaza la entrada de la misma clase (con boxing de primitivos) y devuelve siempre el mismo ref (un FunctionRef por actor, nombre "adapter"). Limites: tipo no registrado => unhandled (UnhandledMessage), no rompe. Errores: NonFatal del adaptador => MessageAdaptionFailure (AT-SIG-007). Efectos: AdaptMessage/AdaptWithRegisteredMessageAdapter son mensajes internos (InternalMessage.scala); AdaptMessage es WrappedMessage (aparece desenvuelto en logs y dead letters, ActorRef.scala:633-641).

**Orden / concurrencia / ciclo de vida.** La funcion adaptadora corre en el hilo del actor receptor (thread name del dispatcher del actor en MessageAdapterSpec 105-114); la respuesta se serializa con los demas mensajes.

**Dependencias.** AT-SIG-007; AC-CELL (FunctionRef); AC-REF

**Equivalente Rust propuesto.** ctx.message_adapter::<U>(|u: U| -> M) -> ActorRef<U>; clave por TypeId de U; el 'matchea subclases' no aplica (no hay herencia). **Diferencias observables:** Solo coincide el tipo exacto (TypeId); sin herencia.

**Aserciones.**

- A1: La funcion del adaptador corre dentro del actor (nombre y dispatcher del snitch), tambien con spawnMessageAdapter — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/MessageAdapterSpec.scala:77-117

- A2: Se usa el adaptador correcto por clase y el registrado antes para el mismo tipo se reemplaza; adaptador de Int funciona — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/MessageAdapterSpec.scala:119-166

- A3: Un tipo sin adaptador se descarta como unhandled sin romper el actor — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/MessageAdapterSpec.scala:168-211

- A4: Excepcion en el adaptador detiene el actor — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/MessageAdapterSpec.scala:213-256

- A5: Una excepcion posterior al adaptador (en el handler) no se atrapa como falla del adaptador; los mensajes pendientes van a dead letters — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/MessageAdapterSpec.scala:258-312

- A6: Tras terminar el actor, los mensajes al adaptador van a DeadLetter — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/MessageAdapterSpec.scala:316-336

- A7: spawnMessageAdapter con nombre incluye el nombre en el path del adaptador y entrega al actor — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:639-660

- A8: La funcion del adaptador puede acceder al contexto sin error de hilo — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorThreadSpec.scala:146-165

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): registrar adaptadores para Resp y Pong2; enviar tipos registrados, no registrados y subclases

**Decisiones.** Q-T-CTX-1, Q-X-3

## AT-CTX-012 — ask (desde el actor) con mapResponse

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** ask (desde el actor) con mapResponse: interaccion request-response con timeout; la respuesta o Failure(TimeoutException) se transforma al protocolo propio dentro del actor; las respuestas llegan en orden; si el mapeo lanza o no matchea, el actor falla como si fuera una excepcion normal.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/ActorContext.scala:296-326; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorContextImpl.scala:230-234 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: target.ask(createRequest)(timeout, scheduler) y pipeToSelf(mapResponse). Errores: timeout => Failure(TimeoutException) con mensaje que empieza "Ask timed out on" e incluye la ruta del destino, la clase del mensaje "[java.lang.String]" y el plazo "[10 ms]"; destino terminado => TimeoutException con "had already been terminated"; mapeo que lanza => el actor falla (stop con error logueado) (CAS 85-124). Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** Las respuestas se procesan en el orden de llegada (100 asks => orden preservado).

**Dependencias.** AT-ASK-001; AT-CTX-014

**Equivalente Rust propuesto.** ctx.ask(&target, |reply_to| Req, timeout, |res: Result<Res, AskError>| -> M). **Diferencias observables:** AskError::Timeout reemplaza a Failure(TimeoutException); el texto del mensaje debe conservarse.

**Aserciones.**

- A1: El mapeo corre en el actor (nombre del snitch y dispatcher) y devuelve Pong — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorContextAskSpec.scala:54-85

- A2: Si el mapeo lanza por una respuesta inesperada, el actor se detiene (NotImplementedError logueado) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorContextAskSpec.scala:87-126

- A3: Ask contra deadLetters: Failure(TimeoutException) con "had already been terminated" — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorContextAskSpec.scala:128-146

- A4: Ask a un actor que no responde en 10 ms: TimeoutException "Ask timed out on" con ruta, clase del mensaje y [10 ms] — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorContextAskSpec.scala:148-170

- A5: 100 asks responden en el mismo orden en que se enviaron — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorContextAskSpec.scala:172-201

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): ask a un actor que ignora y a deadLetters con timeout 10 ms; capturar mensaje exacto

**Decisiones.** Q-X-4, Q-X-9

## AT-CTX-013 — askWithStatus (StatusReply)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** askWithStatus: como ask pero desenvuelve StatusReply: Success(v) => Success(v); Error(why) => Failure(why) (normalmente StatusReply.ErrorMessage).

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/ActorContext.scala:328-329; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorContextImpl.scala:236-243 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: ver ask. Errores: StatusReply.Error("boho") => Failure(StatusReply.ErrorMessage("boho")) con getClass "org.apache.pekko.pattern.StatusReply$ErrorMessage"; Error(excepcion) conserva la excepcion. Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** Igual que ask.

**Dependencias.** AT-ASK-002; AC-ASK

**Equivalente Rust propuesto.** StatusReply<T> como Result<T, StatusError>; ctx.ask_with_status. **Diferencias observables:** StatusError con el mensaje reemplaza a StatusReply.ErrorMessage.

**Aserciones.**

- A1: StatusReply.Success(Pong) se mapea a Success(Pong) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorContextAskSpec.scala:203-224

- A2: StatusReply.Error("boho") llega como ErrorMessage con mensaje "boho" — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorContextAskSpec.scala:226-247

- A3: StatusReply.Error(TestException("boho")) conserva la excepcion original — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorContextAskSpec.scala:249-271

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): responder StatusReply.Success y Error con texto y con excepcion

**Decisiones.** ninguna (sin decision abierta)

## AT-CTX-014 — pipeToSelf(future)(mapResult)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** pipeToSelf(future)(mapResult): al completarse el Future el resultado se mapea y se envia a self (AdaptMessage), procesandose dentro del actor; si el mapeo devuelve null se ignora con un warning; las excepciones del mapeo van por MessageAdaptionFailure.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/ActorContext.scala:331-338; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorContextImpl.scala:280-294; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:202-222 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: future.onComplete(v => self ! AdaptMessage(v, mapResult)) con executor parasitico. Errores: mapResult null => warn "Adapter function returned null which is not valid as an actor message, ignoring. This can happen for example when using pipeToSelf and returning null from the adapt function. Null value is ignored and not passed on to actor."; mapeo que lanza => MessageAdaptionFailure. Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** El mapeo corre en el actor, no en el hilo del Future.

**Dependencias.** AT-SIG-007

**Equivalente Rust propuesto.** ctx.pipe_to_self(future, |Result<V, E>| -> M): spawnea la future en el runtime y envia el mensaje mapeado. **Diferencias observables:** No existe el caso null.

**Aserciones.**

- A1: Exito: "ok: hi" procesado en el dispatcher del actor con su nombre — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorContextPipeToSelfSpec.scala:46, 76-99

- A2: Fallo: "ko: <Fail>" — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorContextPipeToSelfSpec.scala:47

- A3: Mapeo que devuelve null: warning exacto y el mensaje no llega al actor — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorContextPipeToSelfSpec.scala:48-72

- A4: Excepcion en el mapeo bajo supervise(restart): PreRestart y nuevo arranque — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1582-1616

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): pipeToSelf de Future fallido y de null

**Decisiones.** Q-X-4

## AT-CTX-015 — delegate(delegator, msg)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** delegate(delegator, msg): arranca el Behavior delegador, interpreta msg (mensaje o senal) con el y devuelve el Behavior resultante: same se resuelve al delegador arrancado; unhandled se publica como UnhandledMessage (onUnhandled) y tambien se cambia al delegador.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/ActorContext.scala:144-152; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorContextImpl.scala:217-228 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: Behavior.start(delegator) + interpretMessage/Signal. Limites: same => started; unhandled => this.onUnhandled(msg) y started; cualquier otro => interpretado. Errores: excepciones del delegador se propagan. Efectos: UnhandledMessage en unhandled.

**Orden / concurrencia / ciclo de vida.** Sincrono dentro del handler.

**Dependencias.** AT-BEH-011; AT-EVT-003

**Equivalente Rust propuesto.** ctx.delegate(behavior, msg) -> Behavior<M>. **Diferencias observables:** Ninguna observable.

**Aserciones.**

- A1: delegate entre ping y pong: ForwardTo y luego ResponseFrom; el actor queda en el delegador — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorContextDelegateSpec.scala:67-81

- A2: Un mensaje no manejado por el delegador se publica como UnhandledMessage y el actor cambia al delegador — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorContextDelegateSpec.scala:83-100

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): delegate con same y con unhandled

**Decisiones.** Q-X-6

## AT-CTX-016 — Chequeo de hilo del ActorContext (no thread-safe)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Chequeo de hilo del ActorContext (no thread-safe): los metodos no thread-safe lanzan UnsupportedOperationException si se llaman fuera del procesamiento del mensaje o desde otro hilo; se permite desde message adapters y mappers de ask porque corren en el actor.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorContextImpl.scala:340-378; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:87-127; docs/src/main/paradox/typed/actor-lifecycle.md:63-70 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Errores (texto exacto): fuera de mensaje => UnsupportedOperationException("Unsupported access to ActorContext from the outside of <self>. No message is currently processed by the actor, but ActorContext was called from <hilo>."); otro hilo durante el mensaje => UnsupportedOperationException("Unsupported access to ActorContext operation from the outside of <self>. Current message is processed by <hilo>, but ActorContext was called from <hilo>."); doble entrada => IllegalStateException("Invalid access by thread from the outside of <self>. Current message is processed by <hilo>, but also accessed from <hilo>."). Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** setCurrentActorThread en aroundReceive y en aroundPreStart/PreRestart/PostRestart/PostStop; clearCurrentActorThread en finally. _currentActorThread no es volatil (puede no detectar siempre).

**Dependencias.** AC-CELL

**Equivalente Rust propuesto.** En Rust el borrow checker lo resuelve en compilacion: ActorContext<'a> solo existe como &mut durante el handler y no es Send/'static, por lo que no puede escapar a otra tarea. **Diferencias observables:** El borrow checker lo resuelve en compilacion; ActorThreadSpec pasa a tests trybuild.

**Aserciones.**

- A1: Acceso al contexto desde fuera tras la inicializacion lanza UnsupportedOperationException que incluye "Unsupported access to ActorContext" — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorThreadSpec.scala:51-68

- A2: Acceso desde otro hilo durante el mensaje lanza la misma excepcion — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorThreadSpec.scala:70-94

- A3: Acceso desde otro hilo tras terminar el mensaje lanza la misma excepcion — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorThreadSpec.scala:96-122

- A4: Acceso desde un hijo con el contexto del padre lanza la misma excepcion — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorThreadSpec.scala:124-144

- A5: Acceso permitido desde message adapter y desde mapper de ask — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorThreadSpec.scala:146-187

**Oraculo.** no aplica: depende de hilos de la JVM; se valida en Rust con tests de compilacion (Q-T-CTX-2).

**Decisiones.** Q-T-CTX-2

## AT-CTX-017 — TypedActorContext / asScala / asJava y ClassicActorContextProvider

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** `TypedActorContext[T]` es el contexto comun a ambos DSL (`asScala`, `asJava` devuelven la misma instancia: `ActorContextImpl` implementa los dos, `ActorContextImpl.scala:133-135`), y el puente al contexto clasico: `ctx.toClassic` / `ctx.actorOf(props[, name])` (extensiones `TypedActorContextOps`, `scaladsl/adapter/package.scala:142-152`) y `Adapter.toClassic(ctx)` / `Adapter.actorOf(ctx, ...)` (javadsl). El contexto clasico se obtiene solo si el contexto typed es un `ActorContextAdapter`; en caso contrario se lanza `UnsupportedOperationException`. `actorOf` crea un hijo CLASICO del actor typed (hijo directo del `ActorCell` subyacente, no de una capa typed). Las demas extensiones del mismo `package object` (spawn/watch/unwatch/stop sobre contexto clasico, toTyped/toClassic de refs, sistemas y schedulers) se contratan en AT-ADP-*; aqui solo se fija la frontera del contexto.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/TypedActorContext.scala:26-38; actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/adapter/package.scala:142-152; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorContextAdapter.scala:27-43, 47-58; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorContextImpl.scala:135-137; actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/ActorContext.scala:52-60 | reference.conf: n/a (sin clave asociada a esta fila)

**Precondiciones.** Cualquier `TypedActorContext` (el trait es `@DoNotInherit`, `TypedActorContext.scala:24`). Para `toClassic`/`actorOf` el contexto debe ser un `ActorContextAdapter` (el que el runtime entrega a los Behaviors de actores reales).

**Comportamiento.** `toClassic(ctx)` devuelve `adapter.classicContext` (`ActorContextAdapter.scala:30`). `ctx.actorOf(props)` y `ctx.actorOf(props, name)` delegan sin transformar en `classicContext.actorOf`, de modo que nombre, supervision y destino son los de un hijo clasico normal; la supervision del hijo la decide el `supervisorStrategy` del `ActorAdapter` padre (AT-SUP-021). `asScala`/`asJava` no copian: devuelven `this`.

**Errores.** `UnsupportedOperationException("Only adapted classic ActorContext permissible (<ctx> of class <clase>)")` si el contexto no es `ActorContextAdapter` (p. ej. los contextos de `BehaviorTestKit`, grupo TK), en el momento de la llamada (`ActorContextAdapter.scala:31-34`). Los errores de `actorOf` son los del clasico (AC-CELL / AC-PATH: nombre invalido o duplicado), no se transforman.

**Orden / concurrencia.** `toClassic`/`actorOf` no llaman a `checkCurrentActorThread` (a diferencia de `spawn`, `children`, `stop`): la comprobacion de hilo solo cubre los metodos del `ActorContextAdapter` (`ActorContextAdapter.scala:59-104`), no las extensiones del package object. Usar el `classic.ActorContext` obtenido fuera del hilo del actor es tan inseguro como en clasico (sin proteccion).

**Dependencias.** AT-ADP-004; AT-JAPI-005; AT-CTX-002; AT-SUP-021; Q-X-8

**Equivalente Rust propuesto.** Un unico `ActorContext<M>` (no hay dos DSL; `asScala`/`asJava` desaparecen: no hay diferencia observable, eran la identidad). La parte de puente (`to_classic()`, `actor_of`) existe solo si Q-X-8 mantiene una capa Classic; en ese caso `to_classic()` devuelve `Result<&ClassicContext, NotAdaptedContext>` (o `Option`) en lugar de lanzar. **Diferencias observables:** en un diseño nativo (Q-X-8 opcion b) `actorOf` clasico y `toClassic` quedan fuera de la superficie; la unica forma de contexto es la tipada. Sin Q-X-8 resuelta no se cierra el comportamiento del puente.

**Aserciones.**

- A1: Un actor typed crea un hijo clasico con `context.actorOf(props)` y le habla (ping/ok) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpec.scala:291-297 (el actor typed1 ejecuta `context.actorOf(classic1)` en AdapterSpec.scala:78-79; el test afirma `probe.expectMsg("ok")`).

- A2: `ctx.actorOf(props, name)` (sobrecarga con nombre) y `Adapter.actorOf(ctx, props, name)` (javadsl, actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/Adapter.scala:136-139) no estan afirmadas por spec (los specs solo usan la sobrecarga sin nombre: AdapterSpec.scala:79, 87, 93, 97, 153). Caracterizacion propuesta: el hijo se llama `name` y `context.child(name)` lo devuelve como `ActorRef[Nothing]`.

- A3: El texto de UnsupportedOperationException para contextos no adaptados; no afirmada por spec (ningun spec de actor-typed-tests invoca `toClassic` sobre un contexto no adaptado: busqueda de `Only adapted classic` solo encuentra la fuente) — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorContextAdapter.scala:33-35. Caracterizacion propuesta: oraculo que invoque `Adapter.toClassic` con un contexto de `StubbedActorContext`.

- A4: `asScala` y `asJava` devuelven el mismo objeto; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorContextImpl.scala:135-137.

**Validacion prevista.** A1: test portado (requiere capa clasica, Q-X-8); A2/A3: caracterizacion con oraculo (`tools/oracle/run.sh`, escenario nuevo que invoca `toClassic` con un contexto adaptado y con uno de testkit); A4 se vuelve estructural en Rust (no aplica).

**Oraculo.** Escenario .jsh nuevo (no existe aun): `Adapter.toClassic(ctx)` dentro de un actor real, y con `StubbedActorContext` para capturar el texto de la excepcion.

**Decisiones.** Q-X-8 y Q-T-CTX-4 (nueva: el puente solo existe con capa clasica; sin la respuesta del Owner el comportamiento A1-A3 queda no ejecutable pero el contrato esta cerrado). Q-T-CTX-2 no aplica (no hay chequeo de hilo en estas extensiones).

## AT-CTX-018 — spawnMessageAdapter (interno) y hooks internos onUnhandled/currentBehavior/cancelAllTimers/clearMdc

*Visibilidad:* internal-required · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** spawnMessageAdapter (interno) y hooks internos onUnhandled/currentBehavior/hasTimer/cancelAllTimers/clearMdc: spawnMessageAdapter crea adaptadores livianos con nombre $..-name; onUnhandled permite que el stash trate unhandled como el adaptador; los usan stash, supervision y otros modulos.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/ActorContext.scala:243-266, 338-381; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorContextAdapter.scala:142-154; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorContextImpl.scala:128-133, 192-201, 294-304 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: spawnMessageAdapter(f, name) y spawnMessageAdapter(f) (nombre vacio). onUnhandled(msg) = adapter.unhandled(msg). Efectos: clearMdc limpia el MDC solo si se uso (mdcUsed).

**Orden / concurrencia / ciclo de vida.** onUnhandled se invoca dentro de unstash para el mensaje que se desapila.

**Dependencias.** AT-STASH-005; AT-SUP-016

**Equivalente Rust propuesto.** Metodos pub(crate) del contexto. **Diferencias observables:** Visibilidad crate-privada; sin diferencia observable.

**Aserciones.**

- A1: Mensajes desapilados que devuelven unhandled se tratan como unhandled normales (UnhandledMessage) y el actor sigue — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashSpec.scala:636-671

- A2: spawnMessageAdapter(identity, name) devuelve un ref cuyo nombre incluye `name` y reenvia al actor — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:639-660

- A3: spawnMessageAdapter(f) tambien ejecuta f en el hilo del actor — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/MessageAdapterSpec.scala:96-116

**Oraculo.** no aplica: hooks internos observables solo via stash/supervision.

**Decisiones.** Q-T-CTX-1

## AT-REF-001 — ActorRef[-T]: tell / ! , contravariante, Comparable, Serializable

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** ActorRef[-T]: tell/! con semantica at-most-once; contravariante en T; Comparable (por path y uid); Serializable (writeReplace a SerializedActorRef). Enviar a un actor terminado descarta el mensaje y lo entrega a DeadLetter (best effort).

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorRef.scala:24-80; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorRefImpl.scala:25-52; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorRefAdapter.scala:28-55 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: tell(msg) delega a classicRef ! msg. Limites: ref terminado => DeadLetter. Errores: null => InvalidMessageException (AT-REF-002). Efectos: sendSystem traduce SystemMessage (Create no permitido: IllegalStateException("WAT? No, seriously."), Terminate, Watch, Unwatch, DeathWatchNotification) a mensajes de sistema clasicos (ActorRefAdapter.scala:70-81).

**Orden / concurrencia / ciclo de vida.** At-most-once; orden por par emisor-receptor heredado del clasico (AC-CELL/MBX).

**Dependencias.** AC-REF; AC-PATH; AT-SER-001; AT-SER-003

**Equivalente Rust propuesto.** struct ActorRef<M: Send + 'static> { inner: Arc<dyn RefInner> } con tell(&self, msg: M); Ord por path; la contravarianza se pierde (ver AT-REF-003). **Diferencias observables:** La contravarianza se pierde (Q-T-BEH-4).

**Aserciones.**

- A1: Un ActorRef typed envia a un actor clasico y viceversa — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpec.scala:250-273

- A2: Un mensaje a un actor terminado llega a DeadLetter — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/MessageAdapterSpec.scala:316-336

- A3: Ordenacion por path y uid (compareTo); derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorRefImpl.scala:33-40

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): compareTo y tell a un ref terminado

**Decisiones.** Q-T-BEH-4

## AT-REF-002 — Rechazo de mensajes null

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Rechazo de mensajes null: tell(null) lanza InvalidMessageException("[null] is not an allowed message") tanto en ActorRefAdapter como en ActorSystemAdapter.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorRefAdapter.scala:35-38; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorSystemAdapter.scala:70-73 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Errores (texto exacto): InvalidMessageException("[null] is not an allowed message"). Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** Se lanza en el hilo del emisor.

**Dependencias.** AC-REF

**Equivalente Rust propuesto.** No aplica: Rust no tiene null; el tipo M lo garantiza. **Diferencias observables:** No aplica en Rust: el tipo M lo garantiza.

**Aserciones.**

- A1: actor ! null lanza InvalidMessageException (con interceptores y transformMessages) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:662-669

- A2: sys ! null lanza InvalidMessageException — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.scala:172-178

- A3: typedRef ! null lanza InvalidMessageException — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpec.scala:258-265

**Oraculo.** no aplica: Rust no tiene null; el escenario del jar solo documenta el mensaje de error.

**Decisiones.** ninguna (sin decision abierta)

## AT-REF-003 — narrow / unsafeUpcast

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** ActorRef.narrow[U <: T] es seguro (devuelve el mismo objeto); unsafeUpcast[U >: T] es un cast sin chequeo que puede dar ClassCastException al enviar.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorRef.scala:44-55; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorRefImpl.scala:29-31 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: narrow/unsafeUpcast devuelven this (ActorRefImpl.scala:29-31) y por lo tanto el ref estrechado es == al original. Errores: unsafeUpcast + envio de un tipo equivocado => ClassCastException en el receptor. Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AT-BEH-022

**Equivalente Rust propuesto.** narrow: ActorRef<Super>::narrow::<Sub>() que envuelve con From<Sub> for Super (costo de un closure); unsafeUpcast no se porta (o solo interno con ActorRef<Box<dyn Any>>). **Diferencias observables:** Rust envuelve con From: el ref estrechado puede no ser == al original (Q-T-BEH-4).

**Aserciones.**

- A1: unsafeUpcast permite enviar un PoisonPill de tipo distinto al protocolo — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/InterceptSpec.scala:324

- A2: unsafeUpcast permite enviar un String a un actor de Command — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1526-1528

- A3: narrow devuelve this: igualdad con el original por path+uid; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorRefImpl.scala:29-31

**Oraculo.** no aplica: es un cast sin comportamiento runtime propio.

**Decisiones.** Q-T-BEH-4, Q-X-3

## AT-REF-004 — RecipientRef[-T]

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** RecipientRef[-T]: abstraccion minima con tell y !, implementada por ActorRef y, fuera de este modulo, EntityRef de sharding; no es serializable ni watchable (Scaladoc con FIXME).

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorRef.scala:120-145; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/InternalRecipientRef.scala:20-37 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: tell(msg). InternalRecipientRef agrega provider, isTerminated y refPrefix (por defecto toString). Limites: ask solo soporta InternalRecipientRef (AT-ASK-004). Errores: ninguno. Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AT-ASK-001

**Equivalente Rust propuesto.** trait RecipientRef<M> { fn tell(&self, msg: M); } **Diferencias observables:** Ninguna observable.

**Aserciones.**

- A1: RecipientRef solo define tell y !; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorRef.scala:126-145

- A2: InternalRecipientRef.isTerminated indica terminacion local conocida; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/InternalRecipientRef.scala:28-31

**Oraculo.** no aplica: es una abstraccion de tipo sin comportamiento runtime propio.

**Decisiones.** ninguna (sin decision abierta)

## AT-REF-005 — Igualdad, hash y path de ActorRef

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Igualdad, hash y path de ActorRef: path expone el ActorPath clasico; igualdad por path + uid del actor; hashCode = path.uid; compareTo por path y, a igual path, por uid; toString "Actor[<path>#<uid>]".

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorRef.scala:57-63; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorRefImpl.scala:33-52 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: equals(that: ActorRef[_]) = path.uid == other.path.uid && path == other.path. Limites: un actor nuevo con el mismo nombre tiene otro uid y por lo tanto otro ref. Errores: ninguno. Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AC-PATH; AC-REF

**Equivalente Rust propuesto.** impl PartialEq/Eq/Hash/Ord para ActorRef<M> delegando en path y uid. **Diferencias observables:** Igualdad entre ActorRef<A> y ActorRef<B> debe ser posible (por path+uid) aunque cambie el tipo (Q-T-BEH-4).

**Aserciones.**

- A1: context.system y context.self se pueden comparar con los del sistema/actor externos (igualdad) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:535-548

- A2: equals/hash/compareTo/toString segun path+uid; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorRefImpl.scala:33-52

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): comparar igualdad y toString de refs de actores recreados

**Decisiones.** Q-T-BEH-4

## AT-REF-006 — ignoreRef

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** ignoreRef: ActorRef que descarta todo mensaje; es singleton por sistema (la misma instancia para cualquier tipo); un ask contra el expira por timeout; es observable con watch sin lanzar; convertible a clasico y de vuelta.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorSystem.scala:176-179; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorSystemAdapter.scala:91-94; actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorRefResolver.scala:86-92 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: cachedIgnoreRef.unsafeUpcast. Limites: ActorRefResolver.resolveActorRef devuelve ignoreRef tipado si la ruta es la del ignoreRef (ActorRefResolver.scala:88-89). Errores: ninguno. Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AC-REF

**Equivalente Rust propuesto.** system.ignore_ref::<M>() -> ActorRef<M>. **Diferencias observables:** Ninguna observable.

**Aserciones.**

- A1: Los mensajes a ignoreRef se descartan; el contador de un actor aumenta igual — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorRefIgnoreSpec.scala:71-87

- A2: ask a ignoreRef hace expirar el Future con TimeoutException — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorRefIgnoreSpec.scala:90-104

- A3: context.ask con ignoreRef no entrega nada al probe — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorRefIgnoreSpec.scala:107-121

- A4: watch sobre ignoreRef no lanza y el actor sigue operativo — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorRefIgnoreSpec.scala:123-141

- A5: ignoreRef[String] y ignoreRef[Int] son la misma instancia — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorRefIgnoreSpec.scala:143-151

- A6: ignoreRef.toClassic.toTyped no falla — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorRefIgnoreSpec.scala:153-155

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): ask contra ignoreRef con timeout 100 ms

**Decisiones.** ninguna (sin decision abierta)

## AT-REF-007 — ActorRefResolver (extension) y ActorRefResolverSetup

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** ActorRefResolver (extension) y ActorRefResolverSetup: toSerializationFormat convierte un ActorRef en String (ruta completa con direccion local y uid como fragmento); resolveActorRef lo restaura; rechaza serializar un ref originado en otro ActorSystem; ActorRefResolverSetup permite reemplazar la extension (tests).

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorRefResolver.scala:20-107 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Errores (texto exacto): IllegalArgumentException("ActorRefResolver for ActorSystem [<dir sistema>] shouldn't be used for serialization of ActorRef that originates from another ActorSystem [<dir origen>]. Use the ActorRefResolver for that system instead.") (ActorRefResolver.scala:72-75). Limites: refs sin celda (RemoteActorRef, MinimalActorRef) no se pueden validar y se serializan con la direccion del resolver. Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AT-EXT-001; AC-SER; AC-PATH

**Equivalente Rust propuesto.** Servicio ActorRefResolver con to_serialization_format/resolve_actor_ref. **Diferencias observables:** Ninguna observable mientras el formato siga siendo pekko://sistema/ruta#uid (ver AT-SER-*).

**Aserciones.**

- A1: toSerializationFormat de un ref propio empieza con "pekko://sys1/" — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorRefResolverSpec.scala:34-36

- A2: Serializar un ref de sys1 con el resolver de sys2 lanza IllegalArgumentException — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorRefResolverSpec.scala:38-41

- A3: Un MinimalActorRef no se puede validar y usa la direccion del resolver ("pekko://sys2/") — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorRefResolverSpec.scala:43-52

- A4: resolveActorRef devuelve ignoreRef tipado para la ruta del ignoreRef; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorRefResolver.scala:86-92

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): dos sistemas sys1/sys2 en el mismo proceso

**Decisiones.** ninguna (sin decision abierta)

## AT-SYS-001 — ActorSystem.apply/create (guardian, nombre, config, guardianProps, setups)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** ActorSystem.apply/create(guardianBehavior, name[, config|setup|bootstrapSetup][, guardianProps]): crea un ActorSystem clasico (ActorSystemImpl) cuyo guardian de usuario ejecuta el Behavior typed; valida el Behavior inicial y exige que este vivo; carga la config con ConfigFactory.load(classLoader) salvo que se pase; los Props del guardian (mailbox, dispatcher) se aplican al guardian; envia GuardianStartupBehavior.Start al terminar de arrancar.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorSystem.scala:223-333; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorSystemAdapter.scala:144-171; docs/src/main/paradox/typed/actor-lifecycle.md:71-90 (leido: el actor guardian es el Behavior pasado al ActorSystem; al parar el guardian se para el sistema; terminate() usa Coordinated Shutdown) | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: createInternal => ActorSystemImpl con guardian PropsAdapter(rethrowTypedFailure=false) + system.start() + guardian ! Start + AdapterExtension(system).adapter. Errores: Behavior same/unhandled => IllegalArgumentException("cannot use Same as initial behavior"); Behavior no vivo (stopped) => IllegalArgumentException("requirement failed") por require sin mensaje (ActorSystem.scala:303-304). Efectos: arranca scheduler, dispatchers, logging (SLF4J), CoordinatedShutdown, extensiones (AT-EXT) y cargadores definidos por reference.conf.

**Orden / concurrencia / ciclo de vida.** El guardian recibe Start solo despues de system.start() (el contexto del actor ya es accesible). Mensajes enviados al sistema tras apply se encolan despues de Start.

**Dependencias.** AC-SYS; AC-CFG; AC-EXT; AT-ADP-006

**Equivalente Rust propuesto.** ActorSystem::<M>::new(guardian: Behavior<M>, name, config: Config) -> Result<ActorSystem<M>, ..>. **Diferencias observables:** ActorSystem::new devuelve Result; el guardian se arranca tras construir el runtime.

**Aserciones.**

- A1: Un guardian que recibe un mensaje y devuelve stopped termina el sistema, whenTerminated completa y CoordinatedShutdown reporta ActorSystemTerminateReason — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.scala:74-89

- A2: Guardian con setup recibe PostStop al terminar el sistema ("started" y luego "done") — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.scala:103-124

- A3: El nombre del sistema es el dado — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.scala:141-145

- A4: guardianProps con MailboxSelector.bounded(5) hace que el guardian use BoundedMailbox$MessageQueue — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.scala:187-208

- A5: Un mensaje enviado justo despues de crear el sistema llega al guardian; el guardian no arranca antes de que el sistema clasico este inicializado — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/GuardianStartupSpec.scala:34-71

- A6: ActorSystem.create(Behaviors.empty, name) crea un sistema cuyo getWhenTerminated no esta completo — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/ActorSystemTest.java:36-41

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): crear un sistema con guardian setup y observar el orden de eventos de arranque

**Decisiones.** Q-X-8, Q-X-1

## AT-SYS-002 — ActorSystem como ActorRef[T] del guardian

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** ActorSystem como ActorRef[T] del guardian: ActorSystem[T] extiende ActorRef[T]; tell envia al guardian de usuario; path es pekko://<name>/user; refPrefix "user"; tell(null) lanza InvalidMessageException.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorSystem.scala:45; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorSystemAdapter.scala:55-88, 137 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: system.guardian ! msg. Errores: InvalidMessageException("[null] is not an allowed message"). Efectos: ninguno. isLocal true; isTerminated = whenTerminated.isCompleted.

**Orden / concurrencia / ciclo de vida.** Igual que tell a cualquier actor.

**Dependencias.** AT-REF-001

**Equivalente Rust propuesto.** impl Deref<Target=ActorRef<M>> o metodo system.tell(msg). **Diferencias observables:** Ninguna observable.

**Aserciones.**

- A1: sys ! Probe("hello", replyTo) llega al guardian que responde — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.scala:74-89

- A2: sys ! null lanza InvalidMessageException — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.scala:172-178

- A3: path = RootActorPath(Address("pekko", name)) / "user"; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorSystemAdapter.scala:85-86

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): enviar al sistema y comparar path

**Decisiones.** ninguna (sin decision abierta)

## AT-SYS-003 — Arranque del guardian (GuardianStartupBehavior)

*Visibilidad:* internal-required · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Arranque del guardian (GuardianStartupBehavior): el guardian de usuario es un Behavior intermedio que espera el mensaje Start; los mensajes que lleguen antes se guardan (hasta 1000) y se entregan despues de Start dentro de un GuardianStopInterceptor; el Behavior del usuario no se arranca antes de la inicializacion del sistema clasico.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/GuardianStartupBehavior.scala:24-55; actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorSystem.scala:313-324 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Errores (texto exacto): StashOverflowException("Guardian Behavior did not receive start and buffer is full.") si hay 1000 mensajes en espera (GuardianStartupBehavior.scala:49-51). Normales: Start => withStash(1000){ stash.unstashAll(intercept(GuardianStopInterceptor)(guardian)) }. Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** Orden de llegada preservado: Start, luego los mensajes guardados en orden.

**Dependencias.** AT-STASH-001; AC-SYS

**Equivalente Rust propuesto.** En Rust el sistema puede construirse completo antes de arrancar el guardian, haciendo innecesario el buffer; mantener el contrato observable (no perder mensajes tempranos). **Diferencias observables:** En Rust el sistema puede construirse completo antes de arrancar el guardian, haciendo innecesario el buffer; mantener el contrato observable (no perder mensajes tempranos).

**Aserciones.**

- A1: Un mensaje enviado inmediatamente despues de crear el sistema llega al guardian — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/GuardianStartupSpec.scala:34-51

- A2: El Behavior del guardian solo se arranca cuando el sistema clasico ya esta inicializado (assertInitialized no falla) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/GuardianStartupSpec.scala:53-71

- A3: Con mas de 1000 mensajes antes de Start se lanza StashOverflowException con el texto exacto; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/GuardianStartupBehavior.scala:35, 49-51

**Oraculo.** no aplica: depende del orden interno de arranque del sistema clasico; se valida con los tests de ordenamiento (A1, A2).

**Decisiones.** Q-T-SYS-3

## AT-SYS-004 — Parada del guardian termina el sistema (GuardianStopInterceptor)

*Visibilidad:* internal-required · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Parada del guardian termina el sistema (GuardianStopInterceptor): cuando el Behavior del guardian deja de estar vivo (tras un mensaje o una senal), se llama ctx.system.terminate() para correr CoordinatedShutdown; el Behavior detenido se devuelve para que el adaptador entregue PostStop al Behavior anterior.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/GuardianStartupBehavior.scala:58-90; docs/src/main/paradox/typed/actor-lifecycle.md:87 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: aroundReceive y aroundSignal evaluan Behavior.isAlive(next); si no, system.terminate(). Limites: terminate inmediato antes de arrancar el guardian: el guardian no recibe PostStop (no estaba arrancado) (ISS 126-135). Errores: ninguno. Efectos: whenTerminated completa; CoordinatedShutdown.shutdownReason = ActorSystemTerminateReason.

**Orden / concurrencia / ciclo de vida.** terminate se llama antes de PostStop del guardian: el hook stopped(cb) corre igualmente (GS 73-89).

**Dependencias.** AT-CS-001

**Equivalente Rust propuesto.** Interceptor interno sobre el guardian que dispara terminate. **Diferencias observables:** Ninguna observable (interceptor interno).

**Aserciones.**

- A1: Guardian que devuelve stopped tras un mensaje: el sistema termina y la razon es ActorSystemTerminateReason — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.scala:74-89

- A2: Guardian que se detiene de inmediato (issue #24172) termina el sistema — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.scala:92-101

- A3: terminate inmediato antes de arrancar el guardian termina sin error — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.scala:126-135

- A4: Crear y parar el guardian 11 veces seguidas no deja el sistema colgado ni lanza — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpec.scala:210-235

- A5: El hook stopped(cb) del guardian corre en un cierre inmediato — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/GuardianStartupSpec.scala:73-89

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): guardian que devuelve stopped inmediatamente; observar whenTerminated y los hooks

**Decisiones.** ninguna (sin decision abierta)

## AT-SYS-005 — terminate / whenTerminated / getWhenTerminated / close

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** terminate / whenTerminated / getWhenTerminated / close: terminate corre CoordinatedShutdown con ActorSystemTerminateReason (salvo run-by-actor-system-terminate=off), para el guardian, sus hijos y por ultimo el system guardian; es asincrono; whenTerminated completa al final (despues de los terminate hooks); close() bloquea hasta terminar o lanza TimeoutException tras close-actor-system-timeout (desde 1.3.0) y hace que ActorSystem sea AutoCloseable (try-with-resources / scala.util.Using).

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorSystem.scala:108-168; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorSystemAdapter.scala:117-127 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: terminate() delega en classic terminate; whenTerminated = system.whenTerminated.map(_ => Done); close() = system.close(). Errores: close sin completar en el plazo => TimeoutException (classic, AC-SYS). Efectos: tras close() getWhenTerminated esta completo.

**Orden / concurrencia / ciclo de vida.** Hijos paran antes que el guardian completa; PostStop del guardian ocurre antes de whenTerminated.

**Dependencias.** AT-CS-001; AC-CS; AC-SYS

**Equivalente Rust propuesto.** system.terminate(); system.when_terminated() -> impl Future<Output=()>; Drop o close() bloqueante. **Diferencias observables:** Drop no puede bloquear en async; ver Q-T-SYS-4.

**Aserciones.**

- A1: terminate() hace que whenTerminated complete y CoordinatedShutdown reporta ActorSystemTerminateReason; el guardian recibe PostStop — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.scala:103-124

- A2: Using(ActorSystem(...)) termina el sistema automaticamente (whenTerminated.isCompleted) — actor-typed-tests/src/test/scala-2.13+/org/apache/pekko/actor/typed/ActorSystemScala213PlusSpec.scala:28-34

- A3: getWhenTerminated tras terminate completa en 2 segundos — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/ActorSystemTest.java:27-34

- A4: getWhenTerminated sin terminate no esta completo — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/ActorSystemTest.java:36-41

- A5: try-with-resources cierra el sistema y getWhenTerminated queda completo — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/ActorSystemTest.java:43-50

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): terminate y close con y sin CoordinatedShutdown

**Decisiones.** Q-T-SYS-4

## AT-SYS-006 — Settings tipados y restart-stash-capacity

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Settings tipados y restart-stash-capacity: Settings envuelve la config clasica; RestartStashCapacity sale de pekko.actor.typed.restart-stash-capacity (default 1000) y debe ser >= 0.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorSystem.scala:335-354; actor-typed/src/main/resources/reference.conf:22-27 (restart-stash-capacity = 1000)

**Condiciones.** Errores (texto exacto): valor negativo => IllegalArgumentException("requirement failed: restart-stash-capacity must be >= 0") por Helpers.Requiring (ActorSystem.scala:351). Normales: Settings.setup = classicSettings.setup. Efectos: toString redacta (ver AT-SYS-012).

**Orden / concurrencia / ciclo de vida.** Se lee al construir Settings (cada vez que se pide system.settings: ActorSystemAdapter crea un Settings nuevo, SA 111).

**Dependencias.** AC-CFG

**Equivalente Rust propuesto.** struct TypedSettings { restart_stash_capacity: usize, .. } desde la config. **Diferencias observables:** Ninguna observable.

**Aserciones.**

- A1: El stash de supervision usa withStashCapacity cuando se define (capacidad 2 => 2 mensajes guardados) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:888-916

- A2: Sin withStashCapacity la capacidad sale de system.settings.RestartStashCapacity; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:369-371

- A3: Valor por defecto 1000; derivada de reference.conf; no afirmada por spec — actor-typed/src/main/resources/reference.conf:27

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): leer system.settings.RestartStashCapacity con config por defecto y con -1

**Decisiones.** ninguna (sin decision abierta)

## AT-SYS-007.1 — ActorSystem.name y address

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** name devuelve el nombre del sistema; address devuelve la direccion externa del transporte por defecto: en no-cluster, Address("pekko", name) sin host ni puerto.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorSystem.scala:48-52, 209-217; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorSystemAdapter.scala:110, 139 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: delegan en classic (name; provider.getDefaultAddress). Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AC-SYS

**Equivalente Rust propuesto.** system.name() -> &str; system.address() -> Address. **Diferencias observables:** Ninguna.

**Aserciones.**

- A1: sys.name == nombre dado ("adapter-name") — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.scala:141-145

- A2: sys.address == Address("pekko", "adapter-address") — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.scala:180-184

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): system.name y system.address

**Decisiones.** ninguna (sin decision abierta)

## AT-SYS-007.2 — ActorSystem.startTime y uptime

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** startTime: instante de arranque en milisegundos desde epoch; uptime: tiempo de vida en segundos.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorSystem.scala:72-80; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorSystemAdapter.scala:113, 114 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: uptime = segundos enteros desde startTime. Limites: justo tras crear, uptime < 1.

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AC-SYS

**Equivalente Rust propuesto.** system.start_time() (ms desde epoch) y system.uptime() (segundos enteros) con reloj inyectable. **Diferencias observables:** Reloj inyectable para tests.

**Aserciones.**

- A1: uptime < 1 al crear y >= 1 tras 1 segundo — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.scala:147-153

**Oraculo.** no aplica: depende del reloj del sistema; se valida con reloj inyectable.

**Decisiones.** Q-T-TMR-4

## AT-SYS-007.3 — ActorSystem.log, logConfiguration y printTree

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** (1) `system.log: org.slf4j.Logger` es un logger SLF4J con nombre de clase `org.apache.pekko.actor.typed.ActorSystem`, sin origen de actor ni MDC de actor; es un `val` (misma instancia siempre). (2) `system.logConfiguration()` emite UNA entrada de log a nivel INFO con el texto de la configuracion del sistema, redactado (variables de entorno ocultas y `user.name` sustituido por `<username>`, ver AT-SYS-012); la entrada la escribe el `LoggingAdapter` CLASICO del sistema (fuente de log = el sistema clasico), no el `Logger` de (1). Se invoca automaticamente al arrancar el sistema si `pekko.log-config-on-start = on` (por defecto `off`). (3) `system.printTree: String` devuelve una cadena multilinea con la jerarquia de actores desde la raiz (`rootGuardian`, no desde `/user`), pensada solo para depuracion; el formato no es API estable (Scaladoc). Lo usan el testkit para diagnosticar un shutdown que excede el plazo.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorSystem.scala:58-70, 179-185; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorSystemAdapter.scala:108-109, 115; actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:544-546 (logConfiguration), 920 (`log.info(settings.toString)`), 492 (Settings.toString = `config.renderWithRedactions()`), 440 (`LogConfigOnStart`), 1074 (`if (LogConfigOnStart) logConfiguration()`), 800-806 (printTree: `private[pekko]`, "may OOM on large actor systems"), 1264-1301 (printNode); actor/src/main/scala/org/apache/pekko/util/Helpers.scala:185-194 (redaccion); actor/src/main/resources/reference.conf:48-50 | reference.conf: `pekko.log-config-on-start = off` (actor/src/main/resources/reference.conf:50)

**Precondiciones.** Sistema en marcha. `printTree` recorre el arbol sin sincronizacion: es una foto aproximada si hay actores creandose o muriendo.

**Comportamiento.** `logConfiguration()` => `log.info(settings.toString)` del sistema clasico (ActorSystem.scala:920); `Settings.toString` renderiza `config.root` con `ConfigRenderOptions.defaults().setShowEnvVariableValues(false)` y reemplaza el valor de `user.name` (AT-SYS-012). `printTree` (clasico) imprime recursivamente: por nodo con celda `ActorRefWithCell`, la linea `"-> " + nombre + " " + simpleName(ref) + " " + claseDelActor + " status=" + estadoDelMailbox + " " + resumenDeHijos` (donde el resumen es `EmptyChildrenContainer`/`TerminatedChildrenContainer`, `<n> children` o `Terminating(<motivo>) ...`), indentando a los hijos ordenados por path con `|` y `⌊->`; para nodos sin celda (refs que no son `ActorRefWithCell`) solo `nombre simpleName`. Efectos: ninguno sobre el sistema (solo lectura).

**Errores.** Ninguno contractual. `printTree` puede lanzar o ser inexacto si el arbol cambia durante el recorrido (sin garantia). `logConfiguration()` con `user.name` nulo/vacio hereda el caso de AT-SYS-012.

**Orden / concurrencia.** Ambos metodos son invocables desde cualquier hilo. `logConfiguration()` es sincrona respecto a la llamada; la escritura del log pasa por el event stream de logging (asincrona respecto al appender).

**Dependencias.** AC-SYS; AT-LOG-001; AT-SYS-012; AT-SYS-006 (Settings); Q-T-SYS-1; Q-T-SYS-2

**Equivalente Rust propuesto.** `system.log()` => `tracing` con `target = "rakka::ActorSystem"` (sin span de actor); `system.log_configuration()` emite un evento INFO con la configuracion redactada; `system.print_tree() -> String` produce un arbol textual propio. **Diferencias observables:** el formato de `printTree` no es contractual (Scaladoc: "no stable API"), por lo que cualquier formato estable en Rust es una mejora, no una desviacion; la clase del actor de la columna 3 (`getClass` de la JVM) y el `status=` del mailbox (entero interno del `Mailbox`) no tienen equivalente directo: se reemplazan por el tipo de Behavior/estado de ejecucion. Q-T-SYS-1 (propuesta (a), no aceptada) mantiene la fila en alcance.

**Aserciones.**

- A1: `log` es `LoggerFactory.getLogger(classOf[ActorSystem[_]])`; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorSystemAdapter.scala:108. Caracterizacion propuesta: `system.log.getName == "org.apache.pekko.actor.typed.ActorSystem"` y `eq` entre dos llamadas.

- A2: `logConfiguration()` delega en el sistema clasico y escribe `settings.toString` a INFO; no afirmada por spec (el unico uso en tests, `docs/src/test/scala/docs/config/ConfigDocSpec.scala:39-44`, es `compileOnly`: solo invoca el metodo y no afirma salida) — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorSystemAdapter.scala:109; actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:937. Caracterizacion propuesta: captura de logs (LogCapturing/testkit) afirma una entrada INFO que contiene `pekko {` y no contiene el valor de `user.name`.

- A3: `printTree` delega en el clasico; no afirmada por spec como contenido. Solo se usa como diagnostico: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestKitUtils.scala:93-102 (mensaje "Failed to stop [<name>] within [<timeout>]" seguido de `printTree`), actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:319-322 (se imprime antes de `fail(...)`; no se afirma el contenido) — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorSystemAdapter.scala:116. Caracterizacion propuesta: sistema con `/user/a` y `/user/a/b`: la cadena contiene los nombres `user`, `a`, `b` en ese orden de anidamiento y empieza por `-> ` seguido del nombre de la raiz.

- A4: `pekko.log-config-on-start = on` dispara `logConfiguration()` una vez tras arrancar; no afirmada por spec — actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1105. Caracterizacion propuesta: sistema creado con la clave `on` y captura de logs.

**Validacion prevista.** Diferencial con el oraculo para A1, A2 y A4 (las entradas de log son observables con un appender de captura en `jshell`); A3 solo verifica contenido estructural (nombres y orden), nunca igualdad de cadena. Para A2 revisar tambien AT-SYS-012 (redaccion).

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh) con tools/oracle/run.sh y escenario .jsh nuevo (no existe aun): crear sistema, llamar `logConfiguration()` con un appender de captura, y `printTree` con una jerarquia de dos niveles.

**Decisiones.** Q-T-SYS-1 (equivalente Rust; propuesta, no aceptada: el contrato observable de log/logConfiguration esta cerrado, solo el formato del arbol depende de la respuesta); Q-T-SYS-2 (redaccion).

## AT-SYS-007.4 — ActorSystem.threadFactory, dynamicAccess y executionContext

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** threadFactory: ThreadFactory que el transporte puede usar para crear hilos; dynamicAccess: acceso reflexivo (ClassLoader wrapper); executionContext: dispatcher por defecto del sistema (pekko.actor.default-dispatcher).

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorSystem.scala:82-111; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorSystemAdapter.scala:106-107, 113 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: delegan en el sistema clasico. Efectos: threadFactory.newThread(r).start() ejecuta r; executionContext ejecuta Futures.

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AC-SYS; AC-DSP

**Equivalente Rust propuesto.** thread_factory/dynamic_access segun Q-T-SYS-1; system.executor() -> handle del runtime. **Diferencias observables:** Rust no tiene ClassLoader; ver Q-T-SYS-1.

**Aserciones.**

- A1: threadFactory crea un hilo que ejecuta su Runnable — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.scala:155-163

- A2: executionContext ejecuta Futures — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.scala:165-170

**Oraculo.** no aplica: depende de la JVM (hilos y classloaders); equivalente a decidir en Q-T-SYS-1.

**Decisiones.** Q-T-SYS-1, Q-X-4

## AT-SYS-008 — systemActorOf

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** systemActorOf(behavior, name, props): crea actores bajo /system (paran despues de los de usuario), para librerias; el Behavior se envuelve con supervise(stop) y rethrowTypedFailure=false.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorSystem.scala:189-196; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorSystemAdapter.scala:128-136 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: system.systemActorOf(PropsAdapter(() => supervise(behavior).onFailure(stop), props, false), name). Limites: falla del Behavior => context.stop (AT-SUP-022). Efectos: path /system/<name>.

**Orden / concurrencia / ciclo de vida.** Termina despues de los actores de usuario durante terminate.

**Dependencias.** AC-SYS; AC-PATH

**Equivalente Rust propuesto.** system.system_actor_of(behavior, name, props). **Diferencias observables:** Ninguna observable.

**Aserciones.**

- A1: Un systemActorOf que lanza se detiene (default Stop) y entrega PostStop — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/coexistence/ClassicSupervisingTypedSpec.scala:143-151

- A2: El Behavior se envuelve con supervise(stop); derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorSystemAdapter.scala:130-132

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): systemActorOf y path del actor

**Decisiones.** ninguna (sin decision abierta)

## AT-SYS-009 — Scheduler tipado (scheduleOnce, scheduleWithFixedDelay, scheduleAtFixedRate)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Scheduler tipado (scheduleOnce, scheduleWithFixedDelay, scheduleAtFixedRate): fachada sobre el scheduler clasico para ejecutar Runnables (Scala y Java); excepcion en el Runnable de una tarea repetida aborta la repeticion; IllegalArgumentException si el delay excede el alcance (delay / tickNanos > Int.MaxValue).

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/Scheduler.scala:24-175; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/SchedulerAdapter.scala:29-75 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: delega 1:1 en el scheduler clasico. Errores: IllegalArgumentException por delay excesivo (Scaladoc; lo lanza clasico). Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** FixedDelay no compensa demoras; FixedRate si (AT-TMR-003/004).

**Dependencias.** AC-SCH

**Equivalente Rust propuesto.** Trait Scheduler sobre el timer del runtime. **Diferencias observables:** Trait Scheduler sobre el timer del runtime con reloj inyectable (Q-T-TMR-4).

**Aserciones.**

- A1: system.scheduler.toTyped es un SchedulerAdapter y toClassic devuelve el scheduler original — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpec.scala:237-241

- A2: scheduleWithFixedDelay/AtFixedRate/Once aceptan lambdas (solo compilacion) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SchedulerSpec.scala:25-27

- A3: La semantica de ejecucion es la del scheduler clasico; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/SchedulerAdapter.scala:46-73

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): programar tareas con delay 10 ms y contar ejecuciones

**Decisiones.** Q-T-TMR-4

## AT-SYS-010 — deadLetters

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** deadLetters: destino que acepta y descarta todo, publicando DeadLetter en el event stream.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorSystem.scala:170-174; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorSystemAdapter.scala:91 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: ActorRefAdapter(system.deadLetters). Efectos: DeadLetter(message, sender, recipient) en el event stream.

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AT-EVT-002; AC-EVT

**Equivalente Rust propuesto.** system.dead_letters::<M>(). **Diferencias observables:** Ninguna observable.

**Aserciones.**

- A1: Un mensaje a un actor ya terminado llega como DeadLetter — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/MessageAdapterSpec.scala:316-336

- A2: Ask contra deadLetters expira con TimeoutException "had already been terminated" — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorContextAskSpec.scala:128-146

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): suscribir un probe a DeadLetter y enviar a deadLetters

**Decisiones.** Q-X-6

## AT-SYS-011 — ActorSystemAdapter / AdapterExtension (sistema tipado sobre el clasico)

*Visibilidad:* internal-required · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** ActorSystemAdapter / AdapterExtension (sistema tipado sobre el clasico): el ActorSystem tipado es un adaptador sobre classic.ActorSystemImpl; hay un solo adaptador por sistema clasico (toTyped devuelve la misma instancia); LoadTypedExtensions carga extensiones tipadas en sistemas clasicos.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorSystemAdapter.scala:39-191; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/AdapterExtension.scala:17-34 (ExtensionId clasico que crea el ActorSystemAdapter); actor-typed/src/main/resources/reference.conf:35-36

**Condiciones.** Normales: AdapterExtension crea un unico ActorSystemAdapter por sistema; ActorSystemAdapter.apply(system) y toClassic. Efectos: pekko.library-extensions += LoadTypedExtensions (reference.conf:36).

**Orden / concurrencia / ciclo de vida.** El adaptador se crea durante el arranque (puede crearse antes de que el sistema clasico este inicializado).

**Dependencias.** AC-SYS; AC-EXT

**Equivalente Rust propuesto.** En Rust: decidir si existe un runtime clasico debajo (porte en capas) o un runtime tipado nativo. **Diferencias observables:** Depende de Q-X-8.

**Aserciones.**

- A1: system.toTyped dos veces devuelve la misma instancia — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpec.scala:202-208

- A2: Hay un unico ActorSystemAdapter por sistema clasico (AdapterExtension); derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorSystemAdapter.scala:144-171

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): toTyped dos veces sobre el mismo sistema

**Decisiones.** Q-X-8

## AT-SYS-012 — Settings.toString redacta valores sensibles

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Settings.toString (y logConfiguration) imprime la config con config.renderWithRedactions(): no muestra valores de variables de entorno y reemplaza el nombre del usuario del proceso por "<username>"; un ActorSystem.toString/Settings no debe filtrar user.home ni user.name.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorSystem.scala:345-348; actor/src/main/scala/org/apache/pekko/util/Helpers.scala:185-194 (renderWithRedactions, @since 1.0.2) | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: configText = config.root.render(ConfigRenderOptions.defaults().setShowEnvVariableValues(false)); luego replace(System.getProperty("user.name"), "<username>"). Errores: si user.name es null o vacio, replace con null lanza NullPointerException (derivada; no afirmada por spec). Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AC-CFG; AC-SYS

**Equivalente Rust propuesto.** impl Display para Settings que aplica una funcion redact(config). **Diferencias observables:** Definir que se redacta en Rust (usuario, env, secretos) (Q-T-SYS-2).

**Aserciones.**

- A1: Settings.toString no contiene user.name ni user.home y si contiene "<username>" — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorSystemSpec.scala:32-48

- A2: Los valores de variables de entorno no se muestran (setShowEnvVariableValues(false)); derivada del codigo; no afirmada por spec — actor/src/main/scala/org/apache/pekko/util/Helpers.scala:191-193

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): imprimir system.settings con una propiedad ${user.home} y comparar

**Decisiones.** Q-T-SYS-2, Q-X-1

## AT-SUP-001 — Supervision por defecto: Stop

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Supervision por defecto: Stop. Sin Behaviors.supervise, una excepcion detiene el actor (en clasico seria restart). Bajo un padre typed, el ActorAdapter del padre decide Stop para hijos typed, registra la causa si el padre observa al hijo (para ChildFailed) y no vuelve a loguearla cuando ya la logueo la supervision typed (TypedActorFailedException).

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:41-49, 164-172, 232-273; docs/src/main/paradox/typed/fault-tolerance.md:29-35; docs/src/main/paradox/typed/coexisting.md:143-150 (leido: con guardian typed el sistema clasico ya no permite crear actores de usuario; un hijo typed de padre clasico para por defecto y un hijo clasico de padre typed reinicia) | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: excepcion no supervisada => el actor para; se loguea ERROR una vez; PostStop se entrega al Behavior; el padre observador recibe ChildFailed. Limites: errores fatales (p.ej. StackOverflowError) no se capturan nunca (NonFatal). Errores: BehaviorTestKit propaga la excepcion tal cual. Efectos: hijos paran con el padre.

**Orden / concurrencia / ciclo de vida.** La excepcion sale del handler; la supervision la convierte en Failed (si existe) o sube al ActorCell; luego PostStop; luego Terminated/ChildFailed al padre.

**Dependencias.** AT-SIG-005; AC-SUP; AC-CELL

**Equivalente Rust propuesto.** El runtime trata el error no supervisado como fin del actor (Stop) y notifica ChildFailed al padre observador. **Diferencias observables:** El runtime trata el Err no supervisado como fin del actor (Q-X-5).

**Aserciones.**

- A1: Sin supervise, Throw detiene al actor, se loguea Exc3 como ERROR, el Behavior recibe PostStop y el actor termina — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:389-398, 492-500

- A2: En BehaviorTestKit la excepcion no supervisada se propaga y el Behavior recibe PostStop — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:131-139

- A3: El padre observador recibe ChildFailed con la causa — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/WatchSpec.scala:100-128

- A4: El ActorAdapter padre decide Stop para hijos typed y registra la causa; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:236-270

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): actor sin supervise que lanza; observar PostStop y ChildFailed

**Decisiones.** Q-X-5

## AT-SUP-002 — Behaviors.supervise(b).onFailure[Thr](strategy) y onFailure(clazz, strategy)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Behaviors.supervise(b).onFailure[Thr](strategy) y onFailure(clazz, strategy): aplica la estrategia solo a excepciones del tipo dado o subclases (default Throwable); solo NonFatal; los errores fatales no se capturan; valida el Behavior envuelto como inicial; AbstractBehavior directo se rechaza (AT-SUP-013). Cada onFailure agrega una capa; excepciones no cubiertas siguen subiendo.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Behaviors.scala:226-261; actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:80-107; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:45-67, 70-131, 136-157 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: Supervisor.apply segun la estrategia (Restart/Backoff => RestartSupervisor; Resume => ResumeSupervisor; Stop => StopSupervisor). Limites: tag Nothing se trata como Throwable (Behavior.scala:84); la clase se compara con throwableClass.isAssignableFrom(UnstashException.unwrap(t).getClass). Errores: Behavior envuelto same/unhandled => IllegalArgumentException (validateAsInitial). Efectos: ver estrategias.

**Orden / concurrencia / ciclo de vida.** Intercepta start, receive y signal del Behavior envuelto.

**Dependencias.** AT-BEH-013; AT-BEH-011

**Equivalente Rust propuesto.** behaviors::supervise(b).on_failure::<E: Error + 'static>(strategy) usando downcast_ref de un error dinamico (Box<dyn Error>/anyhow); panics: decision aparte. **Diferencias observables:** La jerarquia de excepciones JVM no existe: matcher por tipo concreto (downcast) o predicado (Q-T-SUP-1).

**Aserciones.**

- A1: Excepcion no cubierta por onFailure[Exc1] detiene al actor (Exc3 => PostStop) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:141-149

- A2: No captura errores fatales: StackOverflowError se propaga y no hay PreRestart — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:216-224

- A3: Un Behavior[Any] estrechado y supervisado maneja una excepcion de un mensaje de otro tipo — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1504-1532

- A4: Solo se capturan NonFatal; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:149, 163, 171, 297, 333

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): supervise con subclases de excepcion y Error fatal

**Decisiones.** Q-T-SUP-1, Q-X-5

## AT-SUP-003 — SupervisorStrategy.stop

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** SupervisorStrategy.stop: registra la falla (ERROR por defecto) con el texto "Supervisor StopSupervisor saw failure: <mensaje>" y detiene el actor (Failed); el padre recibe ChildFailed.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/SupervisorStrategy.scala:47-50, 132-140; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:162-171 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: StopSupervisor.handleException loguea y devuelve BehaviorImpl.failed(t). Limites: excepciones en setup inicial tambien paran (se loguea una vez). Errores: ninguno. Efectos: logging segun loggingEnabled/logLevel.

**Orden / concurrencia / ciclo de vida.** Igual que AT-SUP-001.

**Dependencias.** AT-SUP-001; AT-SUP-010

**Equivalente Rust propuesto.** SupervisorStrategy::Stop{logging}. **Diferencias observables:** Ninguna observable.

**Aserciones.**

- A1: stop explicito: se loguea Exc3, llega PostStop y el actor termina — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:399-408

- A2: stop con excepcion en setup: se loguea Exc3 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:410-420

- A3: El padre recibe ChildFailed con la causa — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/WatchSpec.scala:130-162

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): supervise(stop) con excepcion en mensaje y en setup

**Decisiones.** ninguna (sin decision abierta)

## AT-SUP-004 — SupervisorStrategy.resume

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** SupervisorStrategy.resume: mantiene el Behavior actual (Behaviors.same; o el Behavior de UnstashException) y procesa el siguiente mensaje; se loguea la falla (ERROR por defecto); si falla un Behavior diferido al arrancar, el actor se detiene (no hay estado previo).

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/SupervisorStrategy.scala:28-35, 125-130; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:173-182; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:88-92, 294-311 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: ResumeSupervisor.handleException => log + Behaviors.same. Limites: con UnstashException se devuelve e.behavior (el Behavior en curso al fallar el unstash) y el resto del stash se conserva. Errores: fallo en el setup inicial => ActorInitializationException; se loguean TestException y ActorInitializationException. Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** El mensaje que fallo se descarta; el siguiente se procesa con el mismo estado.

**Dependencias.** AT-SUP-002

**Equivalente Rust propuesto.** SupervisorStrategy::Resume. **Diferencias observables:** Ninguna observable (ver Q-T-SUP-5 para la falla de arranque).

**Aserciones.**

- A1: resume: el contador sigue en 1 despues de la excepcion — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:165-176, 834-847

- A2: resume no resetea el estado del setup (Ping devuelve 2 tras la falla) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:281-305

- A3: Fallo en la fabrica con resume: se loguean TestException y ActorInitializationException — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1113-1121

- A4: Resume durante unstash: el actor continua desde el Behavior en curso, current-2 y 5 en el stash — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashSpec.scala:565-610

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): supervise(resume) con falla en mensaje, en setup y durante unstash

**Decisiones.** Q-T-SUP-5

## AT-SUP-005 — SupervisorStrategy.restart (ilimitado)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** SupervisorStrategy.restart (ilimitado): entrega PreRestart al Behavior que fallo, recrea el Behavior inicial (setup corre de nuevo) y descarta el estado; por defecto detiene hijos y espera su terminacion; si la falla ocurre al arrancar con restarts ilimitados se relanza (el actor para) para evitar bucles.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/SupervisorStrategy.scala:37-45, 157-184; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:298-315, 313-330, 332-419 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: handleException => cancelAllTimers, signalRestart, prepareRestart; si no hay hijos a parar, restartCompleted ya reinicia en la misma llamada. Errores: fallo en el setup del primer arranque con maxRestarts=-1 => throw t (ActorInitializationException; no se loguea doble) (SUP 301-305). Efectos: timers cancelados; Behavior inicial re-arrancado (setup de nuevo).

**Orden / concurrencia / ciclo de vida.** PreRestart -> stop de hijos (si stopChildren) -> espera de Terminated -> start del Behavior inicial -> unstash de lo acumulado.

**Dependencias.** AT-SIG-002; AT-BEH-007

**Equivalente Rust propuesto.** SupervisorStrategy::Restart{limit: Option<(u32, Duration)>, stop_children, stash_capacity, logging}. **Diferencias observables:** Ninguna observable (ver Q-T-SUP-4).

**Aserciones.**

- A1: restart: PreRestart llega y el estado vuelve a 0 (stubbed) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:151-163

- A2: restart: el log es Exc2 como ERROR, PreRestart y estado 0 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:518-532

- A3: restart recrea el setup (counter reinicia en 1) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:255-279

- A4: El Behavior diferido se crea de inmediato (sin esperar el primer mensaje) bajo restart — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:274-283, 1078-1088

- A5: Fallo de la fabrica en el arranque inicial con restart ilimitado falla en vez de reiniciar (ActorInitializationException) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1096-1111

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): supervise(restart) con estado, setup y hijos

**Decisiones.** Q-T-SUP-4

## AT-SUP-006 — RestartSupervisorStrategy.withLimit(maxNrOfRetries, withinTimeRange)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** RestartSupervisorStrategy.withLimit(maxNrOfRetries, withinTimeRange): si dentro de la ventana se supera el maximo, la excepcion se relanza al padre (bajo padre typed: Stop); la ventana se inicia en la primera falla y se reinicia (contador a 1) cuando expira; maxNrOfRetries=0 detiene en la primera falla; tambien aplica a fallas al arrancar.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/SupervisorStrategy.scala:251-285; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:298-315, 332-338, 430-441; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:219-222 (deadlineHasTimeLeft) | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: updateRestartCount: timeLeft = deadline sin definir o con tiempo; nuevo deadline = ahora + withinTimeRange si no habia o expiro; restartCount = timeLeft ? count+1 : 1. Limites: limite alcanzado si maxRestarts != -1 && restartCount >= maxRestarts && deadlineHasTimeLeft => throw t (Restart). Errores: limite 0 => primera falla propaga. En start: (restartCount + 1) >= maxRestarts && deadlineHasTimeLeft => throw t. Efectos: el actor para tras PostStop.

**Orden / concurrencia / ciclo de vida.** El limite se evalua antes de PreRestart.

**Dependencias.** AT-SUP-005; AC-SUP

**Equivalente Rust propuesto.** Restart{limit: Some((n, window))} con contador y deadline por Instant. **Diferencias observables:** Ninguna observable salvo el reloj inyectable (Q-T-SUP-8).

**Aserciones.**

- A1: withLimit(2, 1.minute): dos PreRestart y a la tercera falla la excepcion sube y llega PostStop — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:226-239

- A2: La ventana expira y el contador vuelve a 0 (2 PreRestart mas despues de 2.1 s) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:241-261

- A3: withLimit(0, ...) detiene en la primera falla (PostStop) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:263-272

- A4: Con limite 2 y ventana 500 ms: tres fallas Exc2 logueadas, dos PreRestart y la tercera PostStop; el actor termina — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:534-554

- A5: Con ventana de 500 ms, esperar mas que la ventana resetea el limite y el estado vuelve a 0 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:556-579

- A6: Falla de la fabrica con withLimit(3,1s) y un fallo: reinicia y luego Started — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1152-1162

- A7: Fabrica que siempre falla con withLimit(2,1s): 3 intentos (1+2 reinicios) y luego ActorInitializationException — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1164-1180

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): withLimit(2, 1 minuto) con tres fallas y con espera de la ventana

**Decisiones.** Q-T-SUP-7, Q-T-TMR-4

## AT-SUP-007.1 — SupervisorStrategy.restartWithBackoff(min, max, randomFactor): espera exponencial

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Reinicia con espera exponencial: delay(n) = min(maxBackoff, minBackoff * 2^n) * (1 + random * randomFactor), con n = numero de reinicios previos (el primero espera minBackoff); si el calculo desborda o no es finito se usa maxBackoff; el factor aleatorio se aplica despues del tope, por lo que el delay real puede exceder maxBackoff hasta randomFactor veces. Durante la espera el supervisor acumula en su stash los mensajes y senales (AT-SUP-009).

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/SupervisorStrategy.scala:52-86, 116-120, 186-202; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:189-200, 353-390 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: prepareRestart => log con numero de error, updateRestartCount, ctx.scheduleOnce(restartDelay, self, ScheduledRestart(this)), Behaviors.empty (durante la espera el Behavior interno no recibe nada). Errores: ninguno propio. Efectos: Backoff por defecto resetBackoffAfter = (min+max)/2.

**Orden / concurrencia / ciclo de vida.** Falla -> PreRestart -> espera -> ScheduledRestart (senal propia) -> start del Behavior inicial -> unstash.

**Dependencias.** AT-TMR-001; AC-SCH; AT-SUP-009

**Equivalente Rust propuesto.** Backoff{min, max, random_factor} con jitter y reloj inyectables. **Diferencias observables:** El jitter aleatorio obliga a tests con randomFactor=0 o jitter inyectado (Q-T-SUP-8).

**Aserciones.**

- A1: Primer reinicio tras ~minBackoff (1 s) y segundo tras ~2 s, con randomFactor 0; el Behavior vuelve a estado 0 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:918-964

- A2: Fallo de la fabrica con backoff: reinicia tras el retraso y luego Started — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1128-1140

- A3: Excepcion no cubierta por el tipo del supervisor con backoff: ActorInitializationException sin reinicio — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1142-1150

- A4: Mensajes y senales llegados durante el backoff no se pierden si hay stash (Pong(1), Pong(2)) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:888-916

- A5: calculateDelay: formula y tope; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:189-200

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): restartWithBackoff(100ms,1s,0): medir los tres primeros reinicios con reloj inyectable

**Decisiones.** Q-T-SUP-2, Q-T-SUP-8

## AT-SUP-007.2 — SupervisorStrategy.withResetBackoffAfter: reinicio del contador de backoff (ResetRestartCount)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** withResetBackoffAfter: el contador de backoff se resetea si el actor no vuelve a fallar durante resetBackoffAfter tras completarse el restart (default (minBackoff + maxBackoff) / 2); se implementa con la senal ResetRestartCount(current, owner) agendada con scheduleOnce y solo resetea si el contador no cambio.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/SupervisorStrategy.scala:52-86, 310-327, 189-210; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:202-207, 241-250, 392-402 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: restartCompleted agenda ResetRestartCount(restartCount, this) a resetBackoffAfter; al llegar, si current == restartCount => restartCount = 0. Limites: si hubo otra falla en medio (restartCount cambio), no se resetea. Efectos: tambien resetea el nivel de log critico (SUP-010).

**Orden / concurrencia / ciclo de vida.** La senal se procesa en el hilo del actor; si el owner no es este supervisor, pasa al siguiente supervisor/Behavior.

**Dependencias.** AT-SUP-007.1

**Equivalente Rust propuesto.** Backoff.reset_after (default (min+max)/2) y senal interna ResetRestartCount. **Diferencias observables:** Ninguna observable.

**Aserciones.**

- A1: resetBackoffAfter por defecto es (100ms + 1s) / 2 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1123-1126

- A2: Con resetBackoffAfter 100 ms, tras esperar el reset el segundo reinicio vuelve a esperar minBackoff — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:997-1035

- A3: El nivel critico de log se reinicia tras resetBackoffAfter (2 s): vuelve a INFO — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1470-1502

- A4: ResetRestartCount sin duenio coincidente pasa al Behavior de usuario como senal opaca (ver AT-SUP-017) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1038-1082

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): backoff con resetBackoffAfter corto; dos fallas separadas por mas que el reset

**Decisiones.** Q-T-SUP-3

## AT-SUP-007.3 — BackoffSupervisorStrategy.withMaxRestarts: tope de reinicios fallidos seguidos

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** withMaxRestarts(n): maximo de reinicios fallidos seguidos con backoff; al alcanzarlo el actor falla y para (se loguea la falla con el numero de error y se devuelve Failed). El Scaladoc dice que cero o negativo desactiva el limite; el codigo solo trata -1 como ilimitado.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/SupervisorStrategy.scala:189-230, 329-333; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:148-155 (unlimitedRestarts), 332-351, 414-418 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: handleException: maxRestarts != -1 && restartCount >= maxRestarts && deadlineHasTimeLeft => Backoff: log(ctx, t, restartCount+1) y Failed(t). Limites derivados, no afirmados: withMaxRestarts(0) falla en la primera excepcion sin reiniciar; withMaxRestarts(-5) tambien (solo -1 es ilimitado) => contradice el Scaladoc 'Zero or negative disables the upper limit'. Errores: ninguno propio. Efectos: el actor termina.

**Orden / concurrencia / ciclo de vida.** Los fallos durante el start de reinicios cuentan igual (restartCompleted usa handleException).

**Dependencias.** AT-SUP-007.1

**Equivalente Rust propuesto.** Backoff.max_restarts: Option<u32>. **Diferencias observables:** Pendiente de Q-T-SUP-3 (0 y negativos distintos de -1).

**Aserciones.**

- A1: withMaxRestarts(2): la excepcion inicial y dos fallos de arranque (TestException x2) terminan el actor — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:966-995

- A2: maxRestarts 0 o negativo distinto de -1 no desactiva el limite; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:339

- A3: Scaladoc afirma que cero o negativo desactiva el limite; contradice el codigo — actor-typed/src/main/scala/org/apache/pekko/actor/typed/SupervisorStrategy.scala:329-333

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): withMaxRestarts(0), (1), (2) y (-1): contar reinicios

**Decisiones.** Q-T-SUP-3

## AT-SUP-008 — Hijos detenidos en restart (withStopChildren)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Hijos detenidos en restart (withStopChildren): por defecto los hijos se detienen antes del restart y este espera sus Terminated; withStopChildren(false) los conserva (el setup solo corre en el primer arranque); un watchWith previo del padre sobre el hijo se revoca (unwatch + watch) para poder esperar su terminacion.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/SupervisorStrategy.scala:286-288; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:224-279, 353-376, 421-428; docs/src/main/paradox/typed/fault-tolerance.md:105-131 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: prepareRestart: childrenToStop = ctx.children (si stopChildren); stopChildren(unwatch, watch, stop); el restart espera Terminated de todos (estado 'children'); si ScheduledRestart llega antes, gotScheduledRestart=true y se espera. Limites: Terminated de otros actores no esperados se acumulan en el stash y se entregan despues. Errores: ninguno propio. Efectos: los hijos reciben PostStop antes de que corra el nuevo setup.

**Orden / concurrencia / ciclo de vida.** Solo cuando los hijos paran y (en backoff) ya vencio el delay se arranca el nuevo Behavior.

**Dependencias.** AT-CTX-005; AT-CTX-006; AC-DW

**Equivalente Rust propuesto.** Opcion stop_children: bool; la espera de Terminated se modela como estado 'restarting' del supervisor. **Diferencias observables:** Ninguna observable.

**Aserciones.**

- A1: Con restart y con backoff los hijos reciben PostStop antes del nuevo arranque; GetState queda en stash y se responde con children vacio — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:581-622

- A2: El Terminated de otro actor observado llega despues, al nuevo Behavior — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:613-621

- A3: El restart revoca el watchWith sobre el hijo: no llega Ping(1) pero si Ping(0) del otro sujeto — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:624-666

- A4: withStopChildren(false): el hijo sigue vivo y no recibe senales — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:668-695

- A5: Dos fallas seguidas durante unstash: se paran los hijos en cada reinicio — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:697-740

- A6: Una excepcion en el PRIMER `setup` tras haber creado hijos: los hijos creados antes de lanzar (`child1`, `child2`) se detienen (reciben `PostStop`), el supervisor reintenta y, cuando el `setup` deja de fallar, queda solo `child1` (creado por el setup exitoso); vale con `restart.withLimit(10, 1.second)` y con `restartWithBackoff(10.millis, 10.millis, 0)`. Se loguea ERROR del `TestException` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:742-783 (cuerpo en :744-777: `child2Probe.expectMessage(ReceivedSignal(PostStop))`, luego `GetState` => `children.keySet == Set("child1")`). El segundo hijo con parada lenta (`slowStop2`) evita un bucle de reinicios que agotaria el limite antes de que `throwFromSetup` vuelva a `false` (comentario :755-756): la espera de `Terminated` de los hijos es parte de la semantica.

- A7: Una excepcion en un `setup` POSTERIOR (durante un reinicio provocado por `Exc1`): `PreRestart` al padre; el nuevo `setup` crea `child1` y `child2` y lanza; se paran ambos y se reintenta; al dejar de fallar queda `child1`. Con restart limitado y con backoff — SupervisionSpec.scala:779-832 (cuerpo :787-832: `parentProbe.expectMessage(ReceivedSignal(PreRestart))`, dos `child1Probe.expectMessage(ReceivedSignal(PostStop))` — el original y el de la encarnacion fallida —, `child2Probe.expectMessage(ReceivedSignal(PostStop))`, `children.keySet == Set("child1")`). Esta afirmacion cubre el caso de AT-SUP-005/Q-T-SUP-4 (setup que falla durante un restart): el supervisor reintenta hasta que el setup tiene exito, dentro del limite configurado.

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): padre con 2 hijos de parada lenta; restart con y sin stopChildren

**Decisiones.** ninguna (sin decision abierta)

## AT-SUP-009 — Stash durante restart/backoff (withStashCapacity) y Dropped al llenarse

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Stash durante restart/backoff (withStashCapacity) y Dropped al llenarse: mientras el restart esta en curso los mensajes y senales se guardan en un StashBuffer[Any] del supervisor (capacidad de la estrategia o pekko.actor.typed.restart-stash-capacity=1000) y se desapilan al nuevo Behavior; si el stash esta lleno se publica Dropped(msg, "Stash is full in [RestartSupervisor]", self) en el event stream.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/SupervisorStrategy.scala:290-298, 340-350; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:123-127, 220-292, 365-376; actor-typed/src/main/resources/reference.conf:22-27

**Condiciones.** Normales: stash.stash(m) si no isFull; si lleno, dropped(ctx, m). Limites: el mismo StashBuffer se conserva si el reinicio vuelve a fallar (SUP 369-373). Errores: ninguno (nunca StashOverflowException). Efectos: Dropped en eventStream clasico con reason "Stash is full in [RestartSupervisor]" (nombre simple de la clase interna).

**Orden / concurrencia / ciclo de vida.** Al terminar el restart se hace unstashAll sobre el nuevo Behavior; las senales (p.ej. Terminated) tambien.

**Dependencias.** AT-STASH-001; AT-EVT-004; AT-SYS-006

**Equivalente Rust propuesto.** Buffer acotado VecDeque en el estado del supervisor; Dropped por EventStream. **Diferencias observables:** Ninguna observable (Dropped via EventStream, Q-X-6).

**Aserciones.**

- A1: Backoff con stashCapacity 2 y 4 mensajes: Pong(1), Pong(2) tras reiniciar y Dropped(Ping(3)), Dropped(Ping(4)) con el texto exacto — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:888-916

- A2: stashCapacity 0: Ping(1) se publica como Dropped inmediatamente — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:918-942

- A3: No se pierden mensajes si fallan nuevamente durante el unstash (Pong(1), Pong(2) tras dos PreRestart) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1202-1237

- A4: Fallo en el constructor con hijo de parada lenta: no se pierden los Pong(1), Pong(2) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1239-1248

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): backoff con stashCapacity 2 y 4 mensajes; suscribir probe a Dropped

**Decisiones.** Q-T-SUP-2, Q-T-STASH-1, Q-X-6

## AT-SUP-010 — Logging de supervision (withLoggingEnabled, withLogLevel, withCriticalLogLevel)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Logging de supervision (withLoggingEnabled, withLogLevel, withCriticalLogLevel): cada falla se loguea con el nivel configurado (ERROR por defecto) y el texto "Supervisor <ClaseSupervisor> saw failure[ [n]]: <mensaje>"; backoff puede elevar a un nivel critico despues de N errores (el contador se resetea con resetBackoffAfter); loggingEnabled=false desactiva el log.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/SupervisorStrategy.scala:120-145, 356-366; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:100-121 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: logMessage = s"Supervisor $this saw failure$errorCountStr: ${unwrapped.getMessage}"; $this = nombre simple de la clase (StopSupervisor, ResumeSupervisor, RestartSupervisor); errorCountStr = " [n]" si errorCount >= 0 (solo backoff). Nivel critico: errorCount > criticalLogLevelAfter. La causa se loguea desenvuelta de UnstashException. Efectos: logger del ActorContext (MDC del actor).

**Orden / concurrencia / ciclo de vida.** Se loguea al capturar (antes del PreRestart en Restart; despues de PreRestart en Backoff, dentro de prepareRestart).

**Dependencias.** AT-LOG-001

**Equivalente Rust propuesto.** Campos logging_enabled y level en la estrategia; tracing::event! con nivel dinamico. **Diferencias observables:** Ninguna observable salvo el backend de logging (Q-X-7).

**Aserciones.**

- A1: withLogLevel(INFO) loguea "exc-1" a INFO — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1448-1457

- A2: withLogLevel(DEBUG) no loguea a INFO — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1459-1468

- A3: Backoff con withLogLevel(INFO).withCriticalLogLevel(ERROR, 3): 3 a INFO, 2 a ERROR y tras el reset vuelve a INFO — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1470-1502

- A4: El texto "Supervisor RestartSupervisor saw failure.*: unstash-fail" — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashSpec.scala:495-498

- A5: El texto exacto "Supervisor ResumeSupervisor saw failure: unstash-fail" — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashSpec.scala:578-581

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): capturar el log de cada estrategia

**Decisiones.** Q-X-7

## AT-SUP-011 — Supervision anidada y orden de manejo (distintas excepciones)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Supervision anidada y orden de manejo: supervise puede anidarse (o encadenar onFailure) para manejar distintas excepciones con distintas estrategias; el supervisor mas interno que matchea maneja; un supervisor para la misma clase de excepcion que reaparece en el Behavior devuelto se deduplica (isSame por clase de excepcion).

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:73-86 (isSame); docs/src/main/paradox/typed/fault-tolerance.md:63-72 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: isSame = AbstractSupervisor con la misma throwableClass. Limites: la pila crece una capa por clase distinta; el spec cuenta 2 InterceptorImpl.receive en la pila para 5 supervise anidados (RuntimeException deduplicada). Errores: ninguno. Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** El supervisor mas interno intercepta primero; si no matchea, sube al siguiente.

**Dependencias.** AT-BEH-015

**Equivalente Rust propuesto.** Pila de interceptores de supervision; is_same compara TypeId del error. **Diferencias observables:** Ninguna observable salvo el selector de errores (Q-T-SUP-1).

**Aserciones.**

- A1: Anidado y plano: Exc2 resume, Exc3 restart, Exc1 para (BehaviorTestKit) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:178-214

- A2: Anidado y plano: IOException restart, IllegalArgumentException stop o restart/backoff, segun la combinacion — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:450-496

- A3: Anidado en ejecucion: Exc2 resume (sin mensajes), Exc3 restart, Exc1 stop — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:855-886

- A4: Supervisiones con defers anidados y planos arrancan y llegan a Started — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1250-1275

- A5: Al devolver un Behavior que vuelve a supervisar RuntimeException, solo se conserva el mas interno (resume); 2 InterceptorImpl.receive en la pila — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1277-1314

- A6: Idem con interceptores intercalados (Supervisor.aroundReceive aparece 2 veces) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1316-1360

- A7: Un backoff supervisor creado en setup se reemplaza por el resume del Behavior nuevo — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1362-1413

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): anidamientos de supervise con distintas excepciones

**Decisiones.** ninguna (sin decision abierta)

## AT-SUP-012 — La supervision se mantiene al cambiar de Behavior (re-wrap)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** La supervision se mantiene al cambiar de Behavior (re-wrap): con estilo funcional basta supervisar el Behavior de nivel superior; cada Behavior devuelto queda envuelto automaticamente por el interceptor (y se deduplica).

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/InterceptorImpl.scala:90-123; docs/src/main/paradox/typed/fault-tolerance.md:84-103 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: receive devuelve new InterceptorImpl(interceptor, started) con el Behavior nuevo. Errores: ninguno. Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** Despues de cada interpretacion.

**Dependencias.** AT-BEH-015

**Equivalente Rust propuesto.** El interceptor de supervision vive fuera del Behavior interno y sobrevive a los cambios. **Diferencias observables:** Ninguna observable.

**Aserciones.**

- A1: Tras switch a otro Behavior la supervision (resume) sigue activa y se maneja boom como resume — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1277-1314

- A2: Estado cambiante A a B con restart vuelve a A y sigue supervisado (ActorContextSpec reset behavior) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:255-279

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): Behavior que cambia de estado y falla

**Decisiones.** ninguna (sin decision abierta)

## AT-SUP-013 — Rechazo de AbstractBehavior supervisado directamente

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Rechazo de AbstractBehavior supervisado directamente: supervise(b).onFailure(...) con una instancia AbstractBehavior (scaladsl o javadsl) lanza IllegalArgumentException porque el restart necesita una nueva instancia.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:45-54 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Errores (texto exacto): IllegalArgumentException("The supervised Behavior must not be a AbstractBehavior instance directly,because a different instance should be created when it is restarted. Wrap in Behaviors.setup.") (nota: falta un espacio tras la coma en el original). Con contexto null se lanza antes IllegalArgumentException del constructor de AbstractBehavior.

**Orden / concurrencia / ciclo de vida.** Se lanza al construir, no al ejecutar.

**Dependencias.** AT-BEH-010

**Equivalente Rust propuesto.** En Rust: supervise exige una fabrica (Fn) por tipo, haciendo imposible el caso. **Diferencias observables:** En Rust supervise exige una fabrica (Fn) por tipo, haciendo imposible el caso.

**Aserciones.**

- A1: supervise(new AbstractBehavior(wrongContext)).onFailure(restart) lanza IllegalArgumentException — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1543-1548

- A2: supervise(new AbstractBehavior(null)) lanza IllegalArgumentException (ya en el constructor) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1550-1554

- A3: El texto exacto del mensaje (incluyendo "directly,because"); derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:51-53

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): supervise de una instancia AbstractBehavior; capturar texto

**Decisiones.** Q-T-BEH-5

## AT-SUP-014 — DeathPactException supervisable

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** DeathPactException supervisable: un DeathPactException (Terminated no manejado) puede recuperarse con supervision (p.ej. restart) porque se lanza dentro de interpretSignal.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:280-287 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: onFailure[DeathPactException](restart): PreRestart implicito y el actor sigue respondiendo. Efectos: se loguea DeathPactException como ERROR.

**Orden / concurrencia / ciclo de vida.** Ver AT-SIG-006.

**Dependencias.** AT-SIG-006

**Equivalente Rust propuesto.** ActorError::DeathPact matcheable por on_failure. **Diferencias observables:** Ninguna observable.

**Aserciones.**

- A1: supervise(...).onFailure[DeathPactException](restart): tras morir el hijo observado, el actor sigue y responde "pong" — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1415-1446

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): padre que observa sin manejar Terminated con restart

**Decisiones.** ninguna (sin decision abierta)

## AT-SUP-015 — Supervision de fallas en funciones adaptadoras

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Supervision de fallas en funciones adaptadoras: la excepcion de un message adapter / pipeToSelf / ask llega como MessageAdaptionFailure y, relanzada, la toma la supervision del actor (nested y flat).

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:202-222 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: restart => PreRestart y nuevo arranque; stop mas especifico dentro de restart general => PostStop y termina. Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** Ver AT-SIG-007.

**Dependencias.** AT-SIG-007

**Equivalente Rust propuesto.** Ver AT-SIG-007. **Diferencias observables:** Ver AT-SIG-007.

**Aserciones.**

- A1: pipeToSelf cuyo mapeo lanza TestException con restart: PreRestart, Starting, y luego procesa mensajes — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1582-1616

- A2: Con supervise anidado: stop para TestException dentro de restart para Exception => se detiene (PostStop) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1618-1655

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): message adapter que lanza bajo restart y stop anidados

**Decisiones.** Q-X-5

## AT-SUP-016 — Timers cancelados en restart y descarte de TimerMsg de encarnaciones previas

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Timers cancelados en restart y descarte de TimerMsg de encarnaciones previas: en falla/restart se cancelan todos los timers; los mensajes de timer ya encolados de la encarnacion previa se descartan (por owner).

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:301-303, 332-335, 394; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:313-327; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/TimerSchedulerImpl.scala:162-189 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: cancelAllTimers en handleException, restartCompleted y preRestart/postRestart; interceptTimerMsg descarta si owner distinto. Efectos: debug log "Received timer [{}] from old restarted instance, discarding".

**Orden / concurrencia / ciclo de vida.** Antes de PreRestart.

**Dependencias.** AT-TMR-006

**Equivalente Rust propuesto.** Generacion por timer + id de encarnacion. **Diferencias observables:** Ninguna observable.

**Aserciones.**

- A1: Tras restart no llegan Tock del timer viejo; GotPreRestart(false) (timer inactivo en PreRestart) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TimerSpec.scala:174-200

- A2: Idem con estado reiniciado (Tock(1) tras Bump y falla) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TimerSpec.scala:202-230

- A3: Un TimerMsg encolado antes del restart se descarta sin ClassCastException — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TimerSpec.scala:327-370

- A4: Excepcion dentro del bloque withTimers tambien descarta el TimerMsg — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TimerSpec.scala:372-404

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): timer repetido y falla con restart

**Decisiones.** ninguna (sin decision abierta)

## AT-SUP-017 — Senales internas ScheduledRestart y ResetRestartCount

*Visibilidad:* internal-required · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Senales internas ScheduledRestart(owner) y ResetRestartCount(current, owner): el supervisor duenio las consume; si el duenio no es este supervisor (p.ej. otro supervisor anidado) pasan al siguiente nivel y finalmente al Behavior de usuario como senal opaca, que se ignora si no se maneja.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:202-207, 220-250 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: owner eq this => procesa; si no => super.aroundSignal (target). Errores: ScheduledRestart sin restart en curso => IllegalStateException("Unexpected ScheduledRestart when restart not in progress") (SUP 233-234). Efectos: el Behavior de usuario puede observar ResetRestartCount como senal; el nombre de clase termina en "ResetRestartCount".

**Orden / concurrencia / ciclo de vida.** Llegan por la cola de mensajes del actor como senales.

**Dependencias.** AT-SIG-001

**Equivalente Rust propuesto.** Variantes internas de Signal con id de duenio. **Diferencias observables:** Ninguna observable (la senal es opaca).

**Aserciones.**

- A1: En dos restartWithBackoff anidados/planos el Behavior interno recibe una senal cuyo nombre de clase termina en ResetRestartCount y sigue vivo — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1038-1082

- A2: ScheduledRestart inesperado lanza IllegalStateException con el texto exacto; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:237-238

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): dos backoff anidados y observar senales en el Behavior

**Decisiones.** Q-T-SUP-6

## AT-SUP-018 — Falla durante unstash (UnstashException) y supervision

*Visibilidad:* internal-required · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Falla durante unstash (UnstashException) y supervision: si falla un mensaje desapilado, PreRestart/PostStop van al ultimo Behavior desapilado, el resto del stash se conserva para el nuevo Behavior y resume continua desde ese Behavior; UnstashException(cause, behavior) envuelve la causa y el supervisor la desenvuelve.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/StashBufferImpl.scala:192-249, 268-287; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:317-334, 345-349, 76; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:156-166 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: interpretOne captura NonFatal => throw UnstashException(e, b2). ActorAdapter.handleUnstashException: behavior = e.behavior y relanza e.cause (tambien envuelto en TypedActorFailedException o ActorInitializationException). Errores: texto de UnstashException: "[<cause>] when unstashing in [<behavior>]". Efectos: mensajes restantes sin desapilar quedan en el buffer.

**Orden / concurrencia / ciclo de vida.** PostStop/PreRestart se entregan al Behavior vigente en el unstash (current-2).

**Dependencias.** AT-STASH-006

**Equivalente Rust propuesto.** El error de unstash lleva el Behavior en curso (ActorError::Unstash{cause, behavior}). **Diferencias observables:** Ninguna observable (error interno).

**Aserciones.**

- A1: Falla del 3er mensaje: unstashing-0, unstashing-1, stash-fail-2 y post-stop-2 (stop supervision) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashSpec.scala:460-488

- A2: restart: pre-restart-2 al Behavior vigente y luego current-00; stash size 0 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashSpec.scala:490-525

- A3: backoff: idem con 100 ms; tambien con hijo de parada lenta — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashSpec.scala:539-563

- A4: resume: continua desde el Behavior en curso; stash size 5, luego 1, luego 0 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashSpec.scala:565-610

- A5: restart.withStashCapacity(4) ante dos Throw seguidos conserva Pong(1), Pong(2) tras dos PreRestart — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1202-1237

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): stash con 4 mensajes, falla el tercero con cada estrategia

**Decisiones.** Q-T-STASH-1

## AT-SUP-019 — Supervision sobre Behavior que para (inicialmente, tras setup o tras restart)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Supervision sobre Behavior que para (inicialmente, tras setup o tras restart): un Behavior supervisado que es o se vuelve stopped deja el actor parado, con cualquier estrategia (stop, restart, resume, backoff, withLimit).

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/InterceptorImpl.scala:106-111 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: deduplicate no re-envuelve un Behavior no vivo (isAlive false). Efectos: PostStop/terminacion normal.

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AT-BEH-004

**Equivalente Rust propuesto.** Sin logica extra. **Diferencias observables:** Sin logica extra.

**Aserciones.**

- A1: supervise(stopped) con cada estrategia termina el actor — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1668-1671

- A2: supervise(setup(stopped)) con cada estrategia termina el actor — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1673-1676

- A3: Stopped tras un restart (setup devuelve stopped en la segunda ejecucion) termina el actor (excepto resume) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1678-1699

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): supervise de stopped con las 5 estrategias

**Decisiones.** ninguna (sin decision abierta)

## AT-SUP-020 — Padre clasico supervisando hijo typed: default Stop

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Un actor typed creado desde el mundo clasico (`classicSystem.spawn/spawnAnonymous`, `classicContext.spawn/spawnAnonymous`, `classicSystem.toTyped.systemActorOf`, o `PropsAdapter(behavior)` para pasarlo como `classic.Props`) se envuelve con `Behaviors.supervise(behavior).onFailure(SupervisorStrategy.stop)`: si el Behavior lanza, el actor se detiene (recibe `PostStop`, no `PreRestart`) y su padre clasico lo ve terminar; no se reinicia aunque el padre clasico tenga la estrategia clasica por defecto (que reiniciaria). El usuario puede sobrescribirlo envolviendo su propio `Behaviors.supervise(...).onFailure(restart)` (la supervision mas interna gana, AT-SUP-011). Ademas `rethrowTypedFailure = false` en estas rutas: la falla NO se relanza al padre clasico como `TypedActorFailedException`; el actor hace `context.stop(self)` (AT-SUP-022, AT-BEH-024).

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/adapter/package.scala:49-81 (ClassicActorSystemOps.spawn/spawnAnonymous, `Behaviors.supervise(..).onFailure(stop)`, `rethrowTypedFailure = false`), 103-130 (ClassicActorContextOps.spawn/spawnAnonymous), 86-98 (internalSystemActorOf); actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/adapter/PropsAdapter.scala:31-37 (PropsAdapter publico, mismo envoltorio); actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorSystemAdapter.scala:128-134 (systemActorOf); actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorRefFactoryAdapter.scala:45-70 (spawnAnonymous no valida el Behavior inicial; spawn si, vía `validateAsInitial` dentro del productor del Props); actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:168-184 (next: FailedBehavior con rethrowTypedFailure=false => context.stop(self)); actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:162-171 (StopSupervisor: log + BehaviorImpl.failed); actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/PropsAdapter.scala:32-70 (Props clasico con Deploy.local) | reference.conf: n/a (sin clave asociada a esta fila)

**Precondiciones.** Un padre clasico (actor o sistema clasico, o systemActorOf). `classicSystem.spawn/spawnAnonymous` delega en el `actorOf` del sistema: solo funciona en un sistema clasico con guardian por defecto; en un sistema creado con `typed.ActorSystem(...)` (guardian typed personalizado) lanza `UnsupportedOperationException("cannot create top-level actor [<name>] from the outside on ActorSystem with custom user guardian")` (actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:944-969; docs/src/main/paradox/typed/coexisting.md:143-150), caso no cubierto por ClassicSupervisingTypedSpec y relativo a AC-SYS/AT-SYS-011. Behavior inicial valido para `spawn` con nombre (no `same`/`unhandled`; AT-BEH-012); `spawnAnonymous` clasico NO llama a `validateAsInitial` (`ActorRefFactoryAdapter.scala:34`, contraste con :50), asimetria sin spec.

**Comportamiento.** (1) Excepcion no fatal del Behavior: StopSupervisor la loguea (ERROR por defecto, AT-SUP-010) y devuelve `FailedBehavior`; `ActorAdapter.next` con `rethrowTypedFailure=false` ejecuta `context.stop(self)`: se publica `PostStop` al Behavior y el actor termina; un watcher clasico recibe `Terminated`. La estrategia del padre clasico NO interviene (no hay excepcion que llegue a su `supervisorStrategy`). (2) Con `supervise(restart)` propio la excepcion la resuelve el RestartSupervisor interno: `PreRestart` y el actor sigue vivo (AT-SUP-005). (3) Props: siempre `Deploy.local`; remoto prohibido (`ConfigurationException("Remote deployment not allowed for typed actors")` si el clasico lo rechaza por config remota, `ActorRefFactoryAdapter.scala:27, 36-37, 53-54`).

**Errores.** Falla del Behavior: sin error al llamador; terminacion del actor (arriba). Behavior inicial invalido en `spawn`: `ActorInitializationException` al crear la instancia (AT-BEH-012). Desplegar remoto: `ConfigurationException` con el texto citado.

**Orden / concurrencia.** `PostStop` ocurre antes de `Terminated` al watcher (AT-SIG-003). Los tests usan `probe.expectNoMessage()` tras el `PostStop` para afirmar que no hay `PreRestart` posterior.

**Dependencias.** AT-ADP-002; AT-SUP-001; AT-SUP-003; AT-SUP-005; AT-SUP-022; AT-BEH-024; AT-SIG-003; AC-SUP (supervision clasica); Q-X-8

**Equivalente Rust propuesto.** Solo con capa clasica (Q-X-8 opcion a). En ese caso: `ClassicSystem::spawn(behavior, name, props)` y `ClassicContext::spawn(...)` que envuelven con `supervise(behavior).on_failure(stop)` y `rethrow_typed_failure=false`; `PropsAdapter::new(factory)` para integraciones que exigen Props clasico. Con Q-X-8 nativo (b) la fila se reduce a "el guardian y `system_actor_of` usan Stop" (ya cubierto por AT-SUP-022). **Diferencias observables:** los textos de error de despliegue remoto dependen de la decision de remoting (fuera de estos modulos); `PropsAdapter` solo existe si hay `Props` clasico.

**Aserciones.**

- A1: Spawn desde un sistema clasico: la excepcion del Behavior produce `PostStop`, ningun otro mensaje a la sonda y `Terminated` al watcher clasico — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/coexistence/ClassicSupervisingTypedSpec.scala:75-83 (`probe.expectMsg(PostStop)`, `probe.expectNoMessage()`, `expectTerminated`); `spawnAnonymous` identico: :84-92.

- A2: Override con `Behaviors.supervise(...).onFailure(SupervisorStrategy.restart)`: `PreRestart`, ninguna terminacion (`expectNoMessage()` del testkit clasico) — ClassicSupervisingTypedSpec.scala:94-103.

- A3: Mismas tres propiedades desde un contexto clasico (`context.spawn`, override, `spawnAnonymous`) — ClassicSupervisingTypedSpec.scala:105-140 (el actor `ClassicToTyped` hace `context.spawn(behav, name)` y `context.spawnAnonymous(behav)`, :49-57).

- A4: `systemActorOf` desde `classicSystem.toTyped`: default Stop — ClassicSupervisingTypedSpec.scala:142-150.

- A5: `PropsAdapter(behavior)` pasado a `classicSystem.actorOf`: default Stop (PostStop y Terminated clasico) — ClassicSupervisingTypedSpec.scala:152-160.

- A6: El padre NO recibe la excepcion ni aplica su estrategia clasica; no afirmada por spec como tal (los tests solo observan `Terminated`). Caracterizacion propuesta: padre clasico con `supervisorStrategy` que cuenta `Decider` invocaciones y afirma 0 con `rethrowTypedFailure=false` (frente a 1 con `ctx.spawn` typed, AT-SUP-001).

- A7: `spawn` con Behavior inicial invalido vs `spawnAnonymous` sin validar; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorRefFactoryAdapter.scala:51, 50.

**Validacion prevista.** A1-A5: tests portados 1:1 (requieren la capa clasica; dependientes de Q-X-8). A6-A7: caracterizacion con oraculo (`tools/oracle/run.sh`, escenario nuevo con un padre clasico que cuenta invocaciones del decider y que spawnea un Behavior `same` anonimo y con nombre). Los `expectNoMessage` son de tiempo real (50 ms configurado en el spec, :67): usar reloj inyectable (Q-T-TMR-4).

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) con tools/oracle/run.sh y escenario .jsh nuevo (no existe aun): spawn de typed desde clasico que lanza, con y sin override; conteo de invocaciones del decider del padre.

**Decisiones.** Q-X-8 (existencia de la capa clasica; sin ella la fila es no ejecutable pero su contrato esta cerrado); Q-T-SUP-7 (decider clasico, relacionada).

**Frontera Classic/Typed (a cruzar con typed-extended / ADP).** Las extensiones `spawn/spawnAnonymous/toTyped/toClassic/watch/unwatch/stop` del `package object adapter` y `PropsAdapter` publico son del grupo ADP (AT-ADP-002 y vecinos); este bloque solo fija el default Stop y `rethrowTypedFailure=false` que ellas aplican. ADP debe citar `package.scala:49-81, 103-137` y `PropsAdapter.scala:31-37`; no duplicar aqui la semantica de watch/stop. Tambien debe cruzarse: el rechazo de `actorOf`/`spawn` de nivel superior en un sistema con guardian typed (`ActorSystem.scala:927-938`, ya en contracts/classic-system.md) y coexisting.md:143-150 (supervision por defecto segun el tipo del hijo).

## AT-SUP-021 — Padre typed supervisando hijo clasico: decider clasico

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Padre typed supervisando hijo clasico: decider clasico. Los hijos clasicos de un padre typed se deciden con el defaultDecider clasico (ActorInitializationException, ActorKilled y DeathPactException paran; el resto reinicia) y se loguea en ERROR.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:55-58, 232-266 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: isTypedActor = el hijo es ActorAdapter => Stop; si no => classicSupervisorDecider (defaultDecider.applyOrElse(exc, Restart)). El log: ctx.log.error(mensaje de la causa, ex); para ActorInitializationException con causa usa el mensaje de la causa (o del InvocationTargetException interno). Efectos: recordChildFailure solo si se observa al hijo.

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AT-ADP-005; AC-SUP

**Equivalente Rust propuesto.** Solo con capa clasica. **Diferencias observables:** Solo con capa clasica.

**Aserciones.**

- A1: Hijo clasico que lanza en receive bajo padre typed se reinicia (preStart, postStop, preStart) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/coexistence/TypedSupervisingClassicSpec.scala:61-72

- A2: Hijo clasico que lanza ThrowIt3: se loguea como ERROR y se reinicia — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpec.scala:317-330

- A3: Hijo clasico que lanza en el constructor: ActorInitializationException logueada y el hijo termina — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpec.scala:332-343

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): padre typed con hijo clasico que lanza en receive y en constructor

**Decisiones.** Q-T-SUP-7

## AT-SUP-022 — rethrowTypedFailure: falla del guardian / systemActorOf vs hijo typed

*Visibilidad:* internal-required · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** El ActorAdapter recibe rethrowTypedFailure: true para hijos creados con ActorContext.spawn/spawnAnonymous (el padre es un ActorAdapter que sabe interpretar TypedActorFailedException) y false para el guardian de usuario, systemActorOf y spawns desde clasico. Con true, un Failed del Behavior se rethrow como TypedActorFailedException(cause) (el padre registra la causa y no la reloguea); con false se hace context.stop(self).

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:65, 164-172, 232-240; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorContextAdapter.scala:75-83; actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorSystem.scala:313-320; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorSystemAdapter.scala:128-134 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: con true, el padre typed ve ChildFailed con la causa; con false el actor para y el padre (guardian/clasico) solo ve Terminated. Errores: ninguno propio. Efectos: evita doble logueo.

**Orden / concurrencia / ciclo de vida.** Dentro de ActorAdapter.next al recibir un FailedBehavior.

**Dependencias.** AT-BEH-024; AT-SUP-001; AT-SYS-008

**Equivalente Rust propuesto.** Parametro interno del runtime al crear el actor (report_failure_to_parent: bool). **Diferencias observables:** Solo relevante si hay capa clasica (Q-X-8); en un runtime nativo basta una sola via de notificacion de fallos.

**Aserciones.**

- A1: Un hijo typed spawneado por un typed padre produce ChildFailed con la misma causa — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/WatchSpec.scala:100-162

- A2: Un typed spawneado desde clasico con default stop recibe PostStop y termina — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/coexistence/ClassicSupervisingTypedSpec.scala:75-83

- A3: El guardian y systemActorOf usan rethrowTypedFailure=false; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorSystem.scala:319; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorSystemAdapter.scala:128-134

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): fallo de un hijo spawneado por padre typed vs desde clasico

**Decisiones.** Q-X-5, Q-X-8

## AT-TMR-001 — TimerScheduler ligado al actor (withTimers)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** TimerScheduler ligado al actor (withTimers): timers con clave que envian mensajes a self; se cancelan todos al parar o reiniciar el actor; varios withTimers anidados comparten el mismo scheduler (ctx.timer); no es thread-safe (solo dentro del actor).

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/TimerScheduler.scala:25-34; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/TimerSchedulerImpl.scala:39-49, 93-99; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorContextImpl.scala:116-133; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:313-339 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: ctx.timer se crea lazy en el primer acceso con checkCurrentActorThread (UnsupportedOperationException fuera del actor). Limites: timers = Map[key -> Timer]; cancelAll en postStop, preRestart y postRestart. Errores: contexto no soportado => IllegalArgumentException (AT-BEH-020). Efectos: tras parar, ningun tick llega a dead letters.

**Orden / concurrencia / ciclo de vida.** cancelAll ocurre antes de interpretar PostStop, pero isTimerActive en PostStop ya es false (el estado se limpia en cancelAll; TimerSpec lo comprueba con GotPostStop(false)).

**Dependencias.** AT-BEH-020; AC-SCH

**Equivalente Rust propuesto.** struct TimerScheduler<M> con HashMap<TimerKey, Timer>; clave: Box<dyn Hash+Eq> o tipo generico K. **Diferencias observables:** Clave como K: Hash+Eq+Clone o enum de claves.

**Aserciones.**

- A1: Al parar por excepcion los timers estan cancelados (GotPostStop(false)) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TimerSpec.scala:232-243

- A2: Al parar voluntariamente los timers estan cancelados (GotPostStop(false)) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TimerSpec.scala:245-254

- A3: Dos withTimers anidados reciben mensajes de ambos timers — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TimerSpec.scala:256-281

- A4: Los timers se mantienen al cambiar de Behavior (newBehavior(n+1) con nuevo withTimers) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TimerSpec.scala:283-306

- A5: Un timer repetido cancelado al parar con stopped(cb) no filtra mensajes a dead letters — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TimerSpec.scala:308-324

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): actor con timer repetido que para voluntariamente; observar dead letters

**Decisiones.** Q-T-TMR-1

## AT-TMR-002 — startSingleTimer (con y sin clave)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** startSingleTimer(key, msg, delay) / startSingleTimer(msg, delay): envia msg una vez tras delay; sin clave usa el propio msg como clave; tras entregarse, el timer deja de estar activo.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/TimerScheduler.scala:224-247; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/TimerSchedulerImpl.scala:109-110, 133-134, 180-184 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: scheduleOnce(delay, () => self ! timerMsg) con executor parasitico. Limites: al llegar el TimerMsg valido, si no repite, se elimina de timers (interceptTimerMsg). Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** El mensaje llega por la cola; el adaptador intercepta TimerMsg antes del Behavior.

**Dependencias.** AT-TMR-001

**Equivalente Rust propuesto.** timers.start_single_timer(key, msg, delay). **Diferencias observables:** Ninguna observable (clave: Q-T-TMR-1).

**Aserciones.**

- A1: Un single timer entrega Tock(1) una vez y no mas (expectNoMessage), y GotPostStop(false) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TimerSpec.scala:98-111

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): startSingleTimer de 10 ms

**Decisiones.** Q-T-TMR-1

## AT-TMR-003 — startTimerWithFixedDelay

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** startTimerWithFixedDelay(key, msg, [initialDelay,] delay): envio repetido con retraso fijo entre envios (no compensa demoras; frecuencia algo menor a 1/delay); sin initialDelay el primer envio sale tras delay; sin clave usa msg como clave.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/TimerScheduler.scala:35-111; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/TimerSchedulerImpl.scala:103-107, 135-137 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: scheduleWithFixedDelay(initialDelay, delay). Errores: una excepcion en el Runnable aborta la repeticion (clasico). Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** Un TimerMsg por tick; los ya encolados se descartan si hay reemplazo (AT-TMR-006).

**Dependencias.** AT-TMR-001

**Equivalente Rust propuesto.** tokio::time::interval con MissedTickBehavior::Delay (o loop con sleep). **Diferencias observables:** Ninguna observable salvo el reloj inyectable (Q-T-TMR-4).

**Aserciones.**

- A1: Tres ticks Tock(1) dentro de 4 intervalos con delay 1 s — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TimerSpec.scala:113-129

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): fixed delay 10 ms durante 100 ms

**Decisiones.** Q-T-TMR-4

## AT-TMR-004 — startTimerAtFixedRate

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** startTimerAtFixedRate(key, msg, [initialDelay,] interval): envio repetido a tasa fija compensando demoras (puede enviar rafagas tras pausas largas); sin initialDelay el primero sale tras interval.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/TimerScheduler.scala:113-222; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/TimerSchedulerImpl.scala:97-101, 138-140 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: scheduleAtFixedRate(initialDelay, interval). Errores: igual que fixed delay.

**Orden / concurrencia / ciclo de vida.** Ver AT-TMR-003.

**Dependencias.** AT-TMR-001

**Equivalente Rust propuesto.** interval con MissedTickBehavior::Burst. **Diferencias observables:** Ninguna observable salvo el reloj inyectable (Q-T-TMR-4).

**Aserciones.**

- A1: startTimerAtFixedRate delega en scheduleAtFixedRate con initialDelay = interval por defecto; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/TimerSchedulerImpl.scala:97-101, 138-140

- A2: Con ManualTime, avanzar varios intervalos de golpe entrega rafagas (Scaladoc); no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/TimerScheduler.scala:113-135

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): fixed rate 10 ms con una pausa artificial; contar ticks

**Decisiones.** Q-T-TMR-4

## AT-TMR-005 — startPeriodicTimer

*Visibilidad:* deprecated · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** startPeriodicTimer: deprecado desde Akka 2.6.0; segun el Scaladoc se comporta como fixed rate. Implementaciones inconsistentes: la API Scala (TimerSchedulerImpl) usa fixed rate; la sobrecarga Java (TimerSchedulerCrossDslSupport) delega en startTimerWithFixedDelay.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/TimerScheduler.scala:222; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/TimerSchedulerImpl.scala:80, 113-114 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: Scala startPeriodicTimer(key,msg,interval) = startTimer(..., FixedRateMode(interval)); Java startPeriodicTimer(key,msg,java.time.Duration) = startTimerWithFixedDelay(key,msg,interval). Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** Ver AT-TMR-003/004.

**Dependencias.** AT-TMR-004

**Equivalente Rust propuesto.** No portar (o alias deprecado). **Diferencias observables:** Alias deprecado de fixed_rate (o fixed_delay): decidir.

**Aserciones.**

- A1: Scala: startPeriodicTimer usa FixedRateMode; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/TimerSchedulerImpl.scala:108

- A2: Java: startPeriodicTimer delega en startTimerWithFixedDelay; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/TimerSchedulerImpl.scala:80

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): comparar los tiempos de startPeriodicTimer (Scala) y la sobrecarga Java

**Decisiones.** Q-T-TMR-3

## AT-TMR-006 — Reemplazo por clave con garantia de no entrega del timer previo (generaciones)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Reemplazo por clave con garantia de no entrega del timer previo (generaciones): un nuevo timer con la misma clave cancela el anterior y se garantiza que el mensaje del anterior no se recibe aunque ya estuviera encolado en el mailbox; idem tras cancel y tras restart (owner distinto).

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/TimerScheduler.scala:45-49; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/TimerSchedulerImpl.scala:34-37, 119-145, 169-196; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:129-148 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: timerGen = Iterator.from(1); TimerMsg(key, generation, owner); interceptTimerMsg: sin timer => debug "Received timer [{}] that has been removed, discarding"; owner distinto => debug "Received timer [{}] from old restarted instance, discarding"; generacion distinta => debug "Received timer [{}] from old generation [{}], expected generation [{}], discarding"; valida => devuelve el mensaje original (y borra si no repite). Limites: NotInfluenceReceiveTimeout se propaga al TimerMsg (AT-TMR-007.2). Efectos: solo logs debug.

**Orden / concurrencia / ciclo de vida.** Se evalua antes de interpretar el mensaje en el adaptador (ActorAdapter.handleMessage).

**Dependencias.** AT-TMR-001

**Equivalente Rust propuesto.** TimerMsg interno {key, generation, owner} filtrado antes del Behavior. **Diferencias observables:** Ninguna observable.

**Aserciones.**

- A1: replace: el Tock(1) ya encolado se descarta y llega Tock(2) del nuevo timer — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TimerSpec.scala:131-149

- A2: cancel: tras recibir Cancelled no llegan mas ticks — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TimerSpec.scala:151-172

- A3: restart alt 1: Tock(1) viejo descartado; GotPreRestart(false); luego Tock(2) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TimerSpec.scala:174-200

- A4: restart alt 2: Tock(2) viejo descartado y luego Tock(1) del estado nuevo — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TimerSpec.scala:202-230

- A5: TimerMsg de encarnacion previa descartado sin ClassCastException — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TimerSpec.scala:327-370

- A6: Excepcion en el bloque withTimers descarta el TimerMsg pendiente — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TimerSpec.scala:372-404

- A7: Los tres textos de debug; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/TimerSchedulerImpl.scala:166-185

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): replace con latch, cancel y restart como en TimerSpec

**Decisiones.** ninguna (sin decision abierta)

## AT-TMR-007.1 — cancel / cancelAll / isTimerActive

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** cancel(key) por clave es idempotente (sin efecto si no existe); cancelAll cancela todos los timers; isTimerActive(key) indica si hay un timer activo con la clave (false tras entregar un single timer, tras cancel y tras parar/restart).

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/TimerScheduler.scala:248-268; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/TimerSchedulerImpl.scala:140-160 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: cancel => task.cancel() y timers -= key. Limites: un TimerMsg ya encolado de un timer cancelado se descarta (AT-TMR-006). Errores: ninguno. Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** Solo desde el hilo del actor.

**Dependencias.** AT-TMR-001

**Equivalente Rust propuesto.** timers.cancel(key), timers.cancel_all(), timers.is_timer_active(key). **Diferencias observables:** Ninguna observable.

**Aserciones.**

- A1: Tras cancel y Cancelled no llega ningun tick — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TimerSpec.scala:151-172

- A2: isTimerActive es false en PostStop (timers cancelados al parar) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TimerSpec.scala:110, 128, 171, 199, 229, 241, 253

- A3: cancelAll vacia el mapa; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/TimerSchedulerImpl.scala:155-160

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): cancel antes del disparo; isTimerActive antes y despues

**Decisiones.** ninguna (sin decision abierta)

## AT-TMR-007.2 — NotInfluenceReceiveTimeout en mensajes de timer

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Si el mensaje `msg` con el que se inicia un timer (`startSingleTimer`, `startTimerWithFixedDelay`, `startTimerAtFixedRate`, `startPeriodicTimer`) es instancia del marker clasico `org.apache.pekko.actor.NotInfluenceReceiveTimeout`, el sobre interno `TimerMsg` que se envia al buzon del actor tambien lo es; por lo tanto, cuando ese tick llega, el `ActorCell` clasico NO cancela ni reprograma el receive timeout del actor (`ctx.setReceiveTimeout`, AT-CTX-008), igual que para cualquier mensaje con ese marker. Si `msg` no es del marker, el tick se comporta como un mensaje normal y reinicia el plazo. El marker se evalua sobre el `msg` ORIGINAL del usuario en el momento de `startTimer` (no en cada entrega), y el `Behavior` recibe `msg`, no el `TimerMsg`.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/TimerSchedulerImpl.scala:112-138 (en particular 126-130); actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:129-146 (handleMessage entrega `t.msg` tras `interceptTimerMsg`); actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorContextAdapter.scala:126-137 (setReceiveTimeout/cancelReceiveTimeout); actor/src/main/scala/org/apache/pekko/actor/Actor.scala:161-164 (marker `trait NotInfluenceReceiveTimeout`) y :81 (`Identify` lo lleva); actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:568-585 (`invoke`: `cancelReceiveTimeoutIfNeeded(msg)` antes y `checkReceiveTimeoutIfNeeded(msg, timeoutBeforeReceive)` en `finally`); actor/src/main/scala/org/apache/pekko/actor/dungeon/ReceiveTimeout.scala:50-101; actor/src/main/scala/org/apache/pekko/actor/dungeon/TimerSchedulerImpl.scala:34-40, 93-99 (la version clasica de `Timers` aplica la misma regla con `NotInfluenceReceiveTimeoutTimerMsg`) | reference.conf: n/a (sin clave asociada a esta fila)

**Precondiciones.** Un timer iniciado con `msg` cuyo tipo mezcla `NotInfluenceReceiveTimeout` (en Scala/Java se declara en la clase del mensaje). Un receive timeout configurado en el actor para que el efecto sea observable.

**Comportamiento.** (1) Regla del ActorCell clasico sobre cada mensaje recibido: antes de procesar, si hay datos de timeout y el mensaje NO es `NotInfluenceReceiveTimeout`, se cancela la tarea pendiente (`cancelReceiveTimeoutIfNeeded`, `ReceiveTimeout.scala:71-76`); despues de procesar (`finally`), se reprograma el timeout salvo que el mensaje sea del marker y el handler no haya cambiado el timeout (`ReceiveTimeout.scala:40-42`: `reschedule = !marker || receiveTimeoutChanged`). (2) Consecuencia: ticks marcados no impiden que el timeout dispare aunque lleguen mas rapido que el plazo; ticks no marcados lo posponen indefinidamente. (3) Si el handler de un tick marcado llama `setReceiveTimeout`/`cancelReceiveTimeout`, el cambio SI surte efecto y fuerza reprogramacion (`receiveTimeoutChanged`). (4) La decision se toma por la identidad del `TimerMsg` (instancia anonima `new TimerMsg(...) with NotInfluenceReceiveTimeout`), que se crea una vez por `startTimer` y se reutiliza en todos los ticks de un timer periodico.

**Errores.** Ninguno. Un tick que luego se descarta (clave cancelada, generacion antigua o instancia reiniciada: AT-TMR-006) tambien pasa por la regla (1) en el `ActorCell` aunque el Behavior nunca lo vea.

**Orden / concurrencia.** La regla se aplica en el hilo del actor al procesar el sobre, antes de que `interceptTimerMsg` decida descartarlo; el reloj del receive timeout es el scheduler clasico (AC-SCH).

**Dependencias.** AT-CTX-008; AT-TMR-002; AT-TMR-003; AT-TMR-006; AC-CELL (ReceiveTimeout clasico); AC-SCH

**Equivalente Rust propuesto.** Parametro explicito al iniciar el timer (opcion `no_influence_receive_timeout: bool`) o un metodo del mensaje `fn influences_receive_timeout(&self) -> bool` (Q-T-TMR-2, propuesta (a), no aceptada); el sobre interno hereda la marca. **Diferencias observables:** el marker de clase JVM (`isInstanceOf`) se reemplaza por una marca explicita; los mensajes que Pekko marca por clase (p. ej. `Identify`) deben marcarse a mano; la regla (1) del ActorCell debe reimplementarse en el runtime para TODOS los mensajes, no solo timers.

**Aserciones.** (La semantica se afirma en specs CLASICOS, que fijan la regla del `ActorCell` que typed reutiliza; ningun spec de actor-typed-tests ejercita timers con mensajes del marker.)

- A1: Mensajes `NotInfluenceReceiveTimeout` (propios y `Identify`) enviados cada 100 ms NO impiden que el receive timeout de 1 s dispare — actor-tests/src/test/scala/org/apache/pekko/actor/ReceiveTimeoutSpec.scala:180-200 (el `timeoutLatch` se abre en `ReceiveTimeout`).

- A2: Con `Timers` clasico, ticks `TransparentTick` (marker) e `Identify` cada 100 ms no impiden el timeout y el actor recibio al menos un tick (`count.get() should be > 0`) — ReceiveTimeoutSpec.scala:191-213. Es el equivalente clasico exacto de esta fila.

- A3: Un handler de mensaje del marker que llama `setReceiveTimeout(500 ms)` activa el timeout — ReceiveTimeoutSpec.scala:215-229; que llama `setReceiveTimeout(Duration.Inf)` lo apaga — :231-247; que llama `Duration.Undefined` quita el timeout existente — :249-266; cambios multiples (`2*t`, luego `t`) dentro del mismo handler dejan el ultimo valor y el timeout dispara a los `t` desde el ultimo tick — :268-297 (`probe.expectNoMessage(initialTimeout / 2 + 50.millis)`, luego `expectMsgType[ReceiveTimeout]`).

- A4: Un actor que solo recibe ticks del marker recibe el timeout (dos veces: el handler reenvia un tick al recibir `ReceiveTimeout`) — ReceiveTimeoutSpec.scala:173-189.

- A5: En typed, `TimerMsg` hereda el marker cuando el `msg` del timer lo implementa; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/TimerSchedulerImpl.scala:119-123. Caracterizacion propuesta (oraculo): `Behaviors.withTimers` con `startTimerWithFixedDelay(key, Tick, 100.ms)` donde `Tick extends NotInfluenceReceiveTimeout` y `setReceiveTimeout(1.s, Timeout)`: `Timeout` llega a ~1 s; repetir con `Tick` sin marker: no llega mientras haya ticks.

- A6: Un tick marcado descartado por clave cancelada igual cae bajo la regla (1); no afirmada por spec — mismo codigo + actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:134-140.

**Validacion prevista.** A1-A4: tests portados contra la regla del ActorCell del runtime Rust (ReceiveTimeoutSpec es de AC-CELL; se porta alli y typed la reutiliza). A5-A6: diferencial con oraculo (escenario nuevo) y test de integracion Rust; tiempos con reloj inyectable (Q-T-TMR-4), no en tiempo real.

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) con tools/oracle/run.sh y escenario .jsh nuevo (no existe aun): receive timeout con ticks de timer marcados y no marcados.

**Decisiones.** Q-T-TMR-2 (marker de clase vs flag; propuesta (a), no aceptada: no bloquea el contrato, solo la forma de la API); Q-T-TMR-4 (reloj inyectable para validar sin tiempo real).

**Nota de correccion.** El estado anterior (`listed`) pedia leer `BehaviorTestKitSpec.scala:444-495`; esas lineas son "timer support" del testkit (`TimerScheduled`/`TimerCancelled`) y NO afirman nada sobre NotInfluenceReceiveTimeout; el testkit no modela el receive timeout (grupo TK). La evidencia relevante es `ReceiveTimeoutSpec` clasico y `ReceiveTimeout.scala`.

## AT-TMR-008 — Timers combinados con transformMessages y anidados

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Timers combinados con transformMessages y anidados: withTimers funciona dentro y fuera de transformMessages y anidado; el TimerMsg se ruta antes del interceptor.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/TimerSchedulerImpl.scala:39-49; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:129-148 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: handleMessage intercepta TimerMsg ANTES de Behavior.interpretMessage, por lo que no pasa por transformMessages. Errores: ninguno. Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** El tick llega ya desenvuelto al Behavior mas externo.

**Dependencias.** AT-BEH-016

**Equivalente Rust propuesto.** Sin logica extra si el TimerMsg se rutea antes del interceptor. **Diferencias observables:** Ninguna observable.

**Aserciones.**

- A1: withTimers dentro de transformMessages: el tick llega transformado ("A") — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TransformMessagesSpec.scala:160-180

- A2: withTimers fuera de transformMessages: idem — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TransformMessagesSpec.scala:182-202

- A3: Timers anidados disparan ambos — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TimerSpec.scala:256-281

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): transformMessages con timer interno y externo

**Decisiones.** ninguna (sin decision abierta)

## AT-STASH-001 — StashBuffer: creacion con capacidad (withStash) y StashOverflowException

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** StashBuffer: creacion con capacidad (Behaviors.withStash) y StashOverflowException: buffer acotado; stash por encima de capacity lanza StashOverflowException (javadsl.StashOverflowException, subclase de scaladsl.StashOverflowException); stash(null) lanza NullPointerException.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/StashBuffer.scala:24-87, 172; actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/StashBuffer.scala:160; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/StashBufferImpl.scala:44-91; docs/src/main/paradox/typed/stash.md:40-52 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Errores (texto exacto): StashOverflowException("Couldn't add [<clase del mensaje>] because stash with capacity [<capacity>] is full"); stash(null) => NullPointerException. Limites: isFull = size == capacity; capacity no validada. Efectos: no thread-safe.

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AT-BEH-021

**Equivalente Rust propuesto.** struct StashBuffer<M> { buf: VecDeque<M>, capacity } con stash() -> Result<(), StashOverflow>. **Diferencias observables:** El overflow se devuelve como Result (Q-X-5) con el mismo texto; no existe el caso null.

**Aserciones.**

- A1: Con capacidad 3, stash del 4to lanza StashOverflowException (y tambien la javadsl) y size sigue en 3 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashBufferSpec.scala:65-78

- A2: capacity devuelve la capacidad dada — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashBufferSpec.scala:267-272

- A3: El texto del overflow y NPE por null; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/StashBufferImpl.scala:74-78

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): stash por encima de la capacidad; capturar el texto

**Decisiones.** Q-X-5

## AT-STASH-002 — unstashAll (orden, stash durante unstash)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** unstashAll(behavior): pasa al Behavior y procesa todo lo guardado en orden de llegada; lo stasheado durante el unstash no se procesa en esa llamada (se desapilan a lo sumo los mensajes presentes al llamar); Behavior unhandled no es valido; el resultado puede ser un Behavior deferido (se arranca).

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/StashBuffer.scala:126-142; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/StashBufferImpl.scala:161-165, 162-185, 187-244 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: unstash(behavior, size, identity) y luego stashCleared. Limites: stash vacio => devuelve behavior sin arrancar. Errores: ver AT-STASH-005. Efectos: clear durante el unstash vacia el buffer y detiene el procesamiento restante.

**Orden / concurrencia / ciclo de vida.** Orden FIFO; un unstash dentro de otro mantiene el Behavior en curso (currentBehaviorWhenUnstashInProgress).

**Dependencias.** AT-STASH-001

**Equivalente Rust propuesto.** stash.unstash_all(behavior) -> Behavior<M>. **Diferencias observables:** Ninguna observable.

**Aserciones.**

- A1: El orden es m1,m2,m3 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashBufferSpec.scala:80-92

- A2: Procesa con los Behaviors devueltos (estado acumulado "m1m2m3") — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashBufferSpec.scala:106-127

- A3: Arranca (undefer) los Behaviors devueltos al desapilar — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashBufferSpec.scala:129-150

- A4: Mensajes guardados durante el unstash no se procesan en esa llamada (m1m3, quedando m2) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashBufferSpec.scala:152-184

- A5: clear durante unstash: size 0 y solo m1m2 se procesan — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashBufferSpec.scala:231-259

- A6: unstashAll con el actor: a,b,c,d,e,f en orden — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashSpec.scala:221-240

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): stash a,b,c y unstashAll; stash durante unstash

**Decisiones.** ninguna (sin decision abierta)

## AT-STASH-003 — unstash(behavior, n, wrap)

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** unstash(behavior, numberOfMessages, wrap): procesa a lo sumo n mensajes envolviendolos con wrap (p.ej. Unstashed(msg)); permite intercalar mensajes propios entre varias llamadas; nunca desapila mas que los presentes al llamar.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/StashBuffer.scala:144-165; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/StashBufferImpl.scala:167-190, 254-255 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Normales: iterador take(min(n, size)) sobre los nodos; wrap se aplica a cada mensaje antes de interpretarlo. Limites: n mayor que size => size. Efectos: dropHeadForUnstash va quitando nodos.

**Orden / concurrencia / ciclo de vida.** Los mensajes ajenos pueden llegar entre unstash y unstash (Msg("g") puede llegar en medio).

**Dependencias.** AT-STASH-002

**Equivalente Rust propuesto.** stash.unstash(behavior, n, |m| wrap(m)). **Diferencias observables:** Ninguna observable.

**Aserciones.**

- A1: unstash(.., 20, identity) procesa solo los presentes (m1m3) y los nuevos m4/get requieren otra llamada — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashBufferSpec.scala:186-219

- A2: unstash de 2 en 2: a,b,c,d,e,f,g procesados en orden, con GetProcessed tambien apilado — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashSpec.scala:242-262

- A3: unstash(..,1,..) con same y cambio de Behavior: a,b1,b2 en orden — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashSpec.scala:419-458

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): stash 5, unstash(2): quedan 3

**Decisiones.** ninguna (sin decision abierta)

## AT-STASH-004 — Inspeccion: isEmpty, nonEmpty, size, capacity, isFull, head, foreach, contains, exists, clear

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Inspeccion: isEmpty, nonEmpty, size, capacity, isFull, head, foreach, contains, exists, clear; head en buffer vacio lanza NoSuchElementException("head of empty buffer"); foreach recorre en orden; contains usa ==.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/StashBuffer.scala:45-124; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/StashBufferImpl.scala:65-159 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Errores (texto exacto): NoSuchElementException("head of empty buffer") (SBI 125, 129). Normales: clear => vacia y notifica stashCleared. Efectos: ninguno.

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AT-STASH-001

**Equivalente Rust propuesto.** Metodos sobre StashBuffer; head -> Option<&M>. **Diferencias observables:** head devuelve Option en vez de lanzar NoSuchElementException.

**Aserciones.**

- A1: isEmpty/nonEmpty tras stash — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashBufferSpec.scala:33-40

- A2: size crece con stash y decrece con unstash(1); head de vacio lanza NoSuchElementException — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashBufferSpec.scala:42-63

- A3: foreach produce m1m2m3 y luego m2m3 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashBufferSpec.scala:80-92

- A4: contains y exists correctos — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashBufferSpec.scala:94-104

- A5: clear deja size 0 y permite volver a guardar — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashBufferSpec.scala:221-229

- A6: El texto "head of empty buffer"; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/StashBufferImpl.scala:130

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): stash a: contains/exists/clear/size

**Decisiones.** ninguna (sin decision abierta)

## AT-STASH-005 — Semantica del unstash: same, unhandled, stopped, Behavior inicial

*Visibilidad:* public · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** Semantica del unstash: same mantiene el Behavior en curso del unstash (no el del actor); unhandled se trata como unhandled normal (onUnhandled => UnhandledMessage) y mantiene el Behavior; si el Behavior para durante el unstash, el resto va a dead letters; unhandled como Behavior inicial lanza IllegalArgumentException; si el inicial esta muerto, todos van a dead letters.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/StashBufferImpl.scala:192-257 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Errores (texto exacto): IllegalArgumentException("Cannot unstash with unhandled as starting behavior"). Normales: same inicial => currentBehaviorWhenUnstashInProgress o ctx.currentBehavior. Limites: stopped => unstashRestToDeadLetters: DeadLetter(wrap(msg), system.deadLetters, self) para cada restante. Efectos: UnhandledMessage por cada mensaje unhandled.

**Orden / concurrencia / ciclo de vida.** Cada mensaje se interpreta con el Behavior resultante del anterior.

**Dependencias.** AT-EVT-002; AT-EVT-003

**Equivalente Rust propuesto.** Logica en unstash_all. **Diferencias observables:** Ninguna observable.

**Aserciones.**

- A1: unstashAll(Behaviors.same) dentro de otro Behavior usa ese Behavior (inicial e intermedio, con y sin supervise) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashSpec.scala:331-417

- A2: Switch de Behavior durante el unstash con same: a,b1,b2 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashSpec.scala:419-458

- A3: unhandled se comporta como unhandled normal (UnhandledMessage) y los handled siguen contando — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashSpec.scala:636-671

- A4: unstashAll(stopped) detiene el actor — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashSpec.scala:673-686

- A5: Si el Behavior para tras el primer mensaje, "two" aparece como DeadLetter — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashSpec.scala:688-710

- A6: unstashAll(unhandled) lanza IllegalArgumentException — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashBufferSpec.scala:261-265

- A7: unstashAll(same) como Behavior inicial — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashSpec.scala:712-729

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): unstash con same, unhandled y stopped

**Decisiones.** Q-X-6

## AT-STASH-006 — UnstashException (falla durante unstash)

*Visibilidad:* internal-required · *analysis:* extracted · *impl/validation:* pending/pending

**Contrato observable.** UnstashException (falla durante unstash): la excepcion de un mensaje desapilado se envuelve en UnstashException(cause, behavior) para que la supervision y el adaptador senalen PreRestart/PostStop al Behavior en curso; ver AT-SUP-018.

**Fuente.** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/StashBufferImpl.scala:209-212, 268-287 | reference.conf: n/a (sin clave asociada a esta fila)

**Condiciones.** Errores (texto exacto): RuntimeException("[<cause>] when unstashing in [<behavior>]", cause). UnstashException.unwrap(t) devuelve la causa. Efectos: ver AT-SUP-018.

**Orden / concurrencia / ciclo de vida.** Ver AT-SUP-018.

**Dependencias.** AT-SUP-018

**Equivalente Rust propuesto.** ActorError::Unstash. **Diferencias observables:** Ninguna observable (error interno).

**Aserciones.**

- A1: Stop supervision: post-stop-2 sobre el ultimo Behavior desapilado — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashSpec.scala:460-488

- A2: Restart, backoff y resume ven el Behavior desapilado — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashSpec.scala:490-610

- A3: El texto de UnstashException; derivada del codigo; no afirmada por spec — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/StashBufferImpl.scala:294-295

**Oraculo.** Jar oficial pekko-actor-typed_2.13-1.7.1 + pekko-actor_2.13-1.7.1 (tools/oracle/fetch-jars.sh, SHA-1 en docs/upstream.md) ejecutado con tools/oracle/run.sh y un escenario .jsh nuevo (no existe aun): ver AT-SUP-018

**Decisiones.** Q-T-STASH-1


## Descomposiciones

- **AT-BEH-008** (Behaviors.receive / receiveMessage / receivePartial / receiveMessagePartial / receiveMessageWithSame) se descompone en **AT-BEH-008.1** (receive/receiveMessage con y sin contexto y senal por defecto), **AT-BEH-008.2** (variantes Partial: undefined => unhandled) y **AT-BEH-008.3** (receiveMessageWithSame, desde 1.1.0). Motivo: cada una tiene aserciones y errores distintos y la fila padre mezclaba tres contratos no verificables como una unidad. El padre desaparece. Ninguna implementacion previa (impl/validation pending).
- **AT-SUP-007** (restartWithBackoff + withResetBackoffAfter + withMaxRestarts) se descompone en **AT-SUP-007.1** (espera exponencial y jitter), **AT-SUP-007.2** (resetBackoffAfter y ResetRestartCount) y **AT-SUP-007.3** (withMaxRestarts). Motivo: tres reglas independientes, con discrepancias doc/codigo distintas (Q-T-SUP-2, Q-T-SUP-3). El padre desaparece.
- **AT-TMR-007** (cancel / cancelAll / isTimerActive y NotInfluenceReceiveTimeout) se descompone en **AT-TMR-007.1** (cancel, cancelAll, isTimerActive) y **AT-TMR-007.2** (NotInfluenceReceiveTimeout). Motivo: la segunda depende del receive timeout clasico (AT-CTX-008) y no tiene spec en este area. El padre desaparece.
- **AT-SYS-007** (accesores del sistema) se descompone en **AT-SYS-007.1** (name/address), **AT-SYS-007.2** (startTime/uptime), **AT-SYS-007.3** (log/logConfiguration/printTree) y **AT-SYS-007.4** (threadFactory/dynamicAccess/executionContext). Motivo: la fila agrupaba accesores con contratos y pruebas distintos y con una propuesta de exclusion JVM-only que no esta autorizada (Q-T-SYS-1). El padre desaparece.

## Omisiones agregadas

- **AT-BEH-024** (Behavior fallido / FailedBehavior). Evidencia: `actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/BehaviorImpl.scala:71, 94-96` y `.../Behavior.scala:303-304`: es el tipo que devuelven las supervisiones Stop y backoff agotado y que ActorAdapter.next trata distinto segun rethrowTypedFailure. Ninguna fila lo cubria.
- **AT-SUP-022** (rethrowTypedFailure: guardian/systemActorOf vs hijo typed). Evidencia: `.../internal/adapter/ActorAdapter.scala:65, 164-172`, `.../ActorSystem.scala:311-318` y `.../internal/adapter/ActorSystemAdapter.scala:127-133`. Observable: ChildFailed con causa (padre typed) frente a simple parada (guardian, clasico).
- **AT-SYS-012** (Settings.toString redacta valores sensibles). Evidencia: `.../ActorSystem.scala:343-346`, `actor/src/main/scala/org/apache/pekko/util/Helpers.scala:185-194` y el test sin cita `actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorSystemSpec.scala:29-48`. El inventario viejo citaba el spec en AT-SYS-007 pero sin fila para la redaccion.
- Candidatas revisadas y NO agregadas (cubiertas o de otro grupo): `internal/InternalMessage.scala` (AdaptMessage/AdaptWithRegisteredMessageAdapter) queda cubierta por AT-CTX-011/014; `pekko.actor.typed.default-mailbox` (reference.conf:30-32, Props.scala:213) es del grupo DSP; `SerializedActorRef`/MiscMessageSerializer son AT-SER-001/003; `pekko.use-slf4j`, `LoggerClass`, `ActorMdc` son del grupo LOG; `extensions`/`library-extensions` son del grupo EXT; `internal/ActorFlightRecorder.scala` solo instrumenta delivery (DLV).

## Contradicciones halladas y corregidas

1. AT-BEH-001: la cita `docs/src/main/paradox/typed/actors.md:186` del inventario viejo no respalda el contrato (es el texto introductorio del ejemplo chat-room). Se reemplaza por `Behavior.scala:28-47`.
2. AT-BEH-013/014: BehaviorSignalInterceptor esta en `BehaviorInterceptor.scala:157-177`, no 150-170.
3. AT-BEH-009: `Behaviors.receiveSignal` (solo senales) se define como `receive((_, _) => same).receiveSignal(handler)` (`Behaviors.scala:176-177`): los mensajes NO quedan unhandled sino que se consumen con same. El inventario viejo decia unhandled.
4. AT-BEH-012: "stopped es valido como inicial" es cierto para `ctx.spawn`/`spawn`, pero `ActorSystem.apply` lo rechaza con `require(Behavior.isAlive(...))` (`ActorSystem.scala:303-304`; confirmado por el comentario de `GuardianStartupSpec.scala:76-77`).
5. AT-BEH-012 / AT-CTX-002: el inventario afirmaba que `spawn` valida el Behavior inicial; la validacion corre dentro del creador por nombre de `Props` (`ActorRefFactoryAdapter.scala:50` con `classic Props.scala:87`), es decir al crear la instancia; falla como ActorInitializationException y no en el sitio de llamada. Marcado "verificar con oraculo".
6. AT-SIG-004: `Terminated(r) == ChildFailed(r, c)` es true pero la inversa es false (`MessageAndSignals.scala:90-93` vs `115-118`). No estaba registrado.
7. AT-SIG-005: la causa de ChildFailed solo se registra si el padre esta observando al hijo (`ActorAdapter.scala:270-272`).
8. AT-SUP-007.1/AT-SUP-009: doc dice "los mensajes se descartan" (`SupervisorStrategy.scala:64, 100`) pero el codigo y `SupervisionSpec.scala:882-910` fijan stash + Dropped (Q-T-SUP-2).
9. AT-SUP-007.2: el Scaladoc Java de `withResetBackoffAfter(java.time.Duration)` dice "same value as minBackoff" (`SupervisorStrategy.scala:324-326`); codigo y spec fijan (min+max)/2 (`SupervisionSpec.scala:1117-1120`).
10. AT-SUP-007.3: Scaladoc "Zero or negative disables the upper limit" (`SupervisorStrategy.scala:329-333`) vs codigo que solo trata -1 como ilimitado (`Supervision.scala:145-152, 335`) (Q-T-SUP-3).
11. AT-TMR-004: el inventario citaba `TimerSpec.scala:113` como prueba incidental de fixed rate, pero esa linea usa `startTimerWithFixedDelay`; no hay spec de startTimerAtFixedRate en actor-typed-tests.
12. AT-TMR-005: la API Scala usa fixed rate (`TimerSchedulerImpl.scala:113-114`) y la sobrecarga Java delega en fixed delay (`TimerSchedulerImpl.scala:81-84`); el inventario solo decia "se comporta como fixed rate".
13. AT-SYS-010: `EventStreamDocSpec.scala:96-106` no afirma nada sobre deadLetters (spawnea un listener); se agrega `MessageAdapterSpec.scala:314-334`.
14. AT-SYS-006: la cita `reference.conf:1-33` se precisa a `reference.conf:22-27`; ningun spec leido ejerce la clave.
15. AT-REF-007: el inventario citaba solo el titulo (`ActorRefResolverSpec.scala:29-30`); los cuerpos estan en `:30-58`.
16. AT-BEH-003: el inventario citaba `ActorTestKitSpec.scala:133` (grupo testkit, no leido por este grupo).
17. AT-STASH-001..006: estaban `listed`; se leyeron `StashBufferImpl` completo, `StashBufferSpec` y `StashSpec` y pasan a `extracted`.

## Filas que siguen listed

Cierre documental 2026-10-31: las cuatro filas que seguian `listed` (AT-CTX-017, AT-SUP-020, AT-SYS-007.3, AT-TMR-007.2) se leyeron contra la implementacion (`scaladsl/adapter/package.scala`, `scaladsl/adapter/PropsAdapter.scala`, `ActorSystem.scala` clasico `logConfiguration`/`printTree`, `dungeon/ReceiveTimeout.scala`, `ActorCell.invoke`) y contra el cuerpo de sus specs (`ClassicSupervisingTypedSpec.scala:74-160`, `ReceiveTimeoutSpec.scala:151-297`) y pasan a `extracted`. Ninguna fila de typed-core queda `listed`. Hallazgos:

- AT-CTX-017: el trait `TypedActorContext` no tiene logica; el puente (`toClassic`, `actorOf`) esta en `scaladsl/adapter/package.scala:142-152` y `javadsl/Adapter.scala:116-141`. Ningun spec afirma el texto de `UnsupportedOperationException`: queda como caracterizacion con oraculo.
- AT-SUP-020: se agrega `PropsAdapter` publico (spec :152-160, antes sin citar), la asimetria `spawn` (valida) vs `spawnAnonymous` (no valida) del adaptador (`ActorRefFactoryAdapter.scala:34, 50`) y la restriccion de crear actores de nivel superior en un sistema con guardian typed (`ActorSystem.scala:927-938`, `coexisting.md:143-150`).
- AT-SYS-007.3: no hay spec que afirme `log`, `logConfiguration` ni `printTree`; las aserciones son caracterizaciones propuestas para el oraculo; el unico uso en tests es de diagnostico o `compileOnly`.
- AT-TMR-007.2: la referencia `BehaviorTestKitSpec.scala:444-495` del estado anterior no trata NotInfluenceReceiveTimeout (son los tests de "timer support" del testkit); la evidencia es `ReceiveTimeoutSpec` clasico (:151-297) y la regla de `ReceiveTimeout.scala:40-82`.

Notas de profundidad (revisadas en el cierre documental 2026-10-31): los casos `SupervisionSpec.scala:736-832` de AT-SUP-008 (excepcion en el primer setup o en uno posterior) se leyeron completos y pasan a A6/A7 de AT-SUP-008. Los tests Java (`actor-typed-tests/src/test/java`, p.ej. `javadsl/AdaptationFailureSpec.scala`, `InterceptTest.java`, `WatchTest.java`) no se leyeron; se citan solo cuando el inventario viejo los citaba y se marca como no leidos.

## Cobertura

Archivos de `actor-typed/src/main` (y tests) del area typed-core y las filas que los cubren.

| Archivo (relativo a upstream/pekko/actor-typed/src/main/scala/org/apache/pekko/actor/typed/) | Filas |
|---|---|
| Behavior.scala | AT-BEH-001, 002, 003, 005, 006, 011, 012, 016, 022; AT-SUP-002 (SuperviseBehavior) |
| BehaviorInterceptor.scala | AT-BEH-013, 014, 015 |
| LogOptions.scala | AT-BEH-018 |
| MessageAndSignals.scala | AT-SIG-001..007 |
| TypedActorContext.scala | AT-CTX-017 |
| ActorRef.scala | AT-REF-001, 003, 004 (SerializedActorRef: AT-SER-003, otro grupo) |
| ActorRefResolver.scala | AT-REF-007 |
| ActorSystem.scala | AT-SYS-001, 002, 005, 006, 007.x, 008, 010, 012; AT-REF-006 |
| SupervisorStrategy.scala | AT-SUP-003..010 |
| Scheduler.scala | AT-SYS-009 |
| scaladsl/Behaviors.scala | AT-BEH-002..010, 017..021; AT-SUP-002 |
| scaladsl/AbstractBehavior.scala | AT-BEH-010 |
| scaladsl/ActorContext.scala | AT-CTX-001..018 |
| scaladsl/StashBuffer.scala | AT-STASH-001..005 |
| scaladsl/TimerScheduler.scala | AT-TMR-001..008 |
| internal/BehaviorImpl.scala | AT-BEH-001..009, 024 |
| internal/InterceptorImpl.scala | AT-BEH-013..017 |
| internal/WithMdcBehaviorInterceptor.scala | AT-BEH-019 |
| internal/Supervision.scala | AT-SUP-001..019 |
| internal/StashBufferImpl.scala | AT-STASH-001..006; AT-SUP-018 |
| internal/TimerSchedulerImpl.scala | AT-TMR-001..008; AT-SUP-016 |
| internal/ActorContextImpl.scala | AT-CTX-001..016, 018 |
| internal/ActorRefImpl.scala | AT-REF-001, 003, 005 |
| internal/InternalRecipientRef.scala | AT-REF-004 |
| internal/InternalMessage.scala (sin cita previa) | AT-CTX-011, 014 (AdaptMessage, AdaptWithRegisteredMessageAdapter) |
| internal/PoisonPill.scala | AT-SIG-008 |
| internal/adapter/ActorAdapter.scala | AT-BEH-003, 004, 007, 024; AT-SIG-*; AT-SUP-001, 015, 021, 022; AT-CTX-011, 014 |
| internal/adapter/ActorContextAdapter.scala | AT-CTX-002..009, 017, 018 |
| internal/adapter/ActorRefAdapter.scala | AT-REF-001, 002 |
| internal/adapter/ActorRefFactoryAdapter.scala | AT-CTX-002; AT-BEH-012 |
| internal/adapter/ActorSystemAdapter.scala | AT-SYS-002..011 |
| internal/adapter/AdapterExtension.scala (sin cita previa) | AT-SYS-011 |
| internal/adapter/GuardianStartupBehavior.scala | AT-SYS-003, 004 |
| internal/adapter/PropsAdapter.scala | AT-CTX-002 (aplicacion de Props); detalle en AT-DSP-* (otro grupo) |
| internal/adapter/SchedulerAdapter.scala | AT-SYS-009 |
| internal/SystemMessage.scala | Sin comportamiento observable propio: listas de mensajes de sistema (SystemMessageList, Create/Terminate/Watch/Unwatch/DeathWatchNotification/NoMessage) que solo consume `ActorRefAdapter.sendSystemMessage` (AT-REF-001, cond.); es plumbing de la arquitectura en capas (Q-X-8) |
| internal/ActorFlightRecorder.scala | Sin comportamiento en este grupo: solo trait/ExtensionId de instrumentacion JFR de entrega confiable (grupo DLV) |
| internal/ActorMdc.scala | Claves MDC pekkoSource, sourceActorSystem, pekkoAddress, pekkoTags consumidas por AT-BEH-019 (el contrato de logging es del grupo LOG: AT-LOG-002) |
| internal/LoggerClass.scala, internal/EventStreamExtension.scala, internal/ExtensionsImpl.scala, internal/PropsImpl.scala, internal/MiscMessageSerializer.scala | De otros grupos (LOG, EVT, EXT, DSP, SER); sin filas typed-core |
| scala-2.12/2.13/3 `internal/receptionist/Platform.scala`, `scala-jdk-9/.../jfr/Events.scala`, `delivery/internal/DeliverySerializable.scala`, `internal/receptionist/ReceptionistMessages.scala`, `Dispatchers.scala`, `Props.scala`, `Extensions.scala`, `SpawnProtocol.scala`, `eventstream/EventStream.scala`, `scaladsl/AskPattern.scala`, `scaladsl/Routers.scala` | Fuera de typed-core (grupos RCP, DLV, DSP, EXT, SPV, EVT, ASK, RT); sin filas aqui |
| resources/reference.conf | `pekko.actor.typed.restart-stash-capacity` -> AT-SYS-006; el resto de claves (extensions/library-extensions, default-mailbox, serializers, use-slf4j, reliable-delivery) pertenecen a EXT, DSP, SER, LOG y DLV |

Tests relevantes (relativos a `actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/` salvo indicacion):

| Archivo de test | Filas |
|---|---|
| BehaviorSpec, DeferredSpec, scaladsl/OnSignalSpec, scaladsl/ReceivePartialSpec | AT-BEH-001..012 |
| InterceptSpec, TransformMessagesSpec, MonitorSpec, LogMessagesSpec, scaladsl/ActorLoggingSpec (MDC) | AT-BEH-013..019 |
| scaladsl/StopSpec, scaladsl/GracefulStopSpec | AT-BEH-004, AT-SIG-003 |
| TerminatedSpec, WatchSpec | AT-SIG-004..006; AT-CTX-005..007 |
| scaladsl/AdaptationFailureSpec, scaladsl/MessageAdapterSpec | AT-SIG-007; AT-CTX-011 |
| ActorContextSpec, scaladsl/ActorContextAskSpec, scaladsl/ActorContextPipeToSelfSpec, scaladsl/ActorContextDelegateSpec, scaladsl/ActorThreadSpec | AT-CTX-001..016 |
| ActorRefIgnoreSpec, ActorRefResolverSpec, internal/ActorRefSerializationSpec (SER) | AT-REF-006, 007 |
| internal/ActorSystemSpec, ActorSystemSpec, scaladsl/adapter/GuardianStartupSpec, ActorSystemScala213PlusSpec (`actor-typed-tests/src/test/scala-2.13+/`), SchedulerSpec, java/ActorSystemTest.java, java/SchedulerTest.java | AT-SYS-001..012 |
| SupervisionSpec, coexistence/ClassicSupervisingTypedSpec, coexistence/TypedSupervisingClassicSpec, scaladsl/adapter/AdapterSpec | AT-SUP-001..022 |
| TimerSpec | AT-TMR-001..008; AT-SUP-016 |
| scaladsl/StashBufferSpec, scaladsl/StashSpec | AT-STASH-001..006; AT-SUP-018 |
| OrElseSpec | Sin fila: prueba una composicion por funciones escrita en el propio test (el `OrElseBehavior` de Akka se elimino; su Scaladoc lo dice en `OrElseSpec.scala:25-29`); no define API de Behaviors |
| docs/org/apache/pekko/typed/** y jdocs/** (IntroSpec, FaultToleranceDocSpec, supervision/SupervisionCompileOnly, StashDocSpec, etc.) y los tests Java de `javadsl` | Ejemplos de documentacion y fachada Java; no se leyeron salvo el encabezado de SupervisionCompileOnly (solo compilacion); las reglas que ilustran estan cubiertas por las filas anteriores; las fachadas son del grupo JAPI |

## Revisión de exhaustividad (cierre documental 2026-10-31)

Criterio: se contrastaron las APIs publicas y el comportamiento interno de `actor-typed/src/main` del area (Behavior, Behaviors, BehaviorInterceptor, MessageAndSignals, ActorContext, ActorRef, ActorSystem, SupervisorStrategy, TimerScheduler, StashBuffer, Scheduler, `internal/*` y `internal/adapter/*`) con las filas del grupo, el `reference.conf` de typed y los specs. "Pendiente de analisis" nunca se convierte en "fuera de alcance".

| Elemento revisado | Clasificacion | Fila / frontera / motivo |
|---|---|---|
| `Behavior`, `ExtensibleBehavior`, tags, `Behaviors.same/unhandled/stopped/empty/ignore/setup/receive*/receiveSignal/intercept/monitor/logMessages/withMdc/transformMessages/withTimers/withStash/supervise` | Cubierto por contratos concretos | AT-BEH-001..024, AT-SUP-002, AT-TMR-001, AT-STASH-001 |
| `Behavior.start/canonicalize/interpretMessage/interpretSignal/existsInStack/validateAsInitial/isAlive/isUnhandled/isDeferred` | Cubierto por contratos concretos | AT-BEH-011, AT-BEH-012 |
| `Behavior.narrow`, `ActorRef.narrow/unsafeUpcast` | Cubierto por contratos concretos | AT-BEH-022, AT-REF-003 (Q-T-BEH-4) |
| `FailedBehavior`, `StoppedBehavior` (con callback), `ComposedStoppingBehavior` | Cubierto por contratos concretos | AT-BEH-024, AT-BEH-004 (secuencia PostStop previo -> callback: `ActorAdapter.scala:342-361`) |
| `javadsl.Behaviors`, `BehaviorBuilder`, `ReceiveBuilder`, `AbstractBehavior` (Java), `AbstractMatchingBehavior` | Duplicado o fachada equivalente | Fachada Java del mismo comportamiento; contratos AT-JAPI-* (grupo typed-extended). En Rust no hay dos DSL |
| `Signal`: `PreRestart`, `PostStop`, `Terminated`, `ChildFailed`, `MessageAdaptionFailure`, `DeathPactException` | Cubierto por contratos concretos | AT-SIG-001..007 |
| `PoisonPill` y `PoisonPillInterceptor`, `ScheduledRestart`, `ResetRestartCount` | Detalle interno sin contrato independiente (justificado) | `PoisonPill` tiene AT-SIG-008 (solo requerido si se porta sharding-typed); las dos senales de supervision viven en AT-SUP-017 porque solo son observables dentro del supervisor |
| `ActorContext`: self, system, log, setLoggerName, children, child, spawn, spawnAnonymous, stop, watch, watchWith, unwatch, setReceiveTimeout, scheduleOnce, executionContext, messageAdapter, ask, askWithStatus, pipeToSelf, delegate | Cubierto por contratos concretos | AT-CTX-001..016; `log` y `setLoggerName` en AT-LOG-001 (grupo LOG) |
| `ActorContext` internos: spawnMessageAdapter, onUnhandled, currentBehavior, hasTimer, cancelAllTimers, clearMdc, `setCurrentActorThread/clearCurrentActorThread/checkCurrentActorThread` | Cubierto por contratos concretos | AT-CTX-018, AT-CTX-016 |
| `TypedActorContext.asScala/asJava`, `toClassic`, `actorOf` clasico desde contexto | Cubierto por contratos concretos | AT-CTX-017 (ahora `extracted`); el resto del `package object adapter` es AT-ADP-* |
| `ActorRef`, `RecipientRef`, `ignoreRef`, `ActorRefResolver`, `ActorRefImpl`, `InternalRecipientRef` | Cubierto por contratos concretos | AT-REF-001..007; `SerializedActorRef`/serializacion: AT-SER-* (otro grupo) |
| `ActorSystem` (apply/create, wrap, guardian, name, address, log, logConfiguration, printTree, startTime, uptime, threadFactory, dynamicAccess, executionContext, systemActorOf, scheduler, deadLetters, terminate, whenTerminated, close, Settings) | Cubierto por contratos concretos | AT-SYS-001..012 |
| `ActorSystem.wrap(classicSystem)` (`ActorSystem.scala:329-330`), `ActorSystemAdapter`/`AdapterExtension` | Duplicado o fachada equivalente | Frontera Classic/Typed: contrato en AT-SYS-011 (existencia del adaptador) y AT-ADP-* (grupo typed-extended, no editado aqui) |
| `GuardianStartupBehavior` y `GuardianStopInterceptor` | Cubierto por contratos concretos | AT-SYS-003, AT-SYS-004 |
| `Scheduler` tipado y `SchedulerAdapter` | Cubierto por contratos concretos | AT-SYS-009 (la semantica de ejecucion es AC-SCH) |
| `Dispatchers` tipado (lookup por selector) | Fuera de estos modulos con frontera explicita | AT-DSP-*; `ActorSystemAdapter.dispatchers` (`ActorSystemAdapter.scala:94-103`) cae en el grupo DSP |
| `SupervisorStrategy` (stop, resume, restart, withLimit, restartWithBackoff, withResetBackoffAfter, withMaxRestarts, withStopChildren, withStashCapacity, withLoggingEnabled, withLogLevel, withCriticalLogLevel) | Cubierto por contratos concretos | AT-SUP-003..010 |
| Supervision: anidada, re-wrap, DeathPact supervisable, funciones adaptadoras, timers en restart, UnstashException, setup que para | Cubierto por contratos concretos | AT-SUP-011..019 |
| Coexistencia Classic/Typed: padre clasico/hijo typed, padre typed/hijo clasico, `rethrowTypedFailure`, `TypedActorFailedException` | Cubierto por contratos concretos | AT-SUP-020, 021, 022 |
| `ActorAdapter` hooks `preStart/preRestart/postRestart/postStop` y `aroundReceive` | Duplicado o fachada equivalente | Contrato en AT-ADP-006.3 (typed-extended); aqui solo se cita desde AT-BEH-007/AT-SIG-002/003 |
| `ActorAdapter.supervisorStrategy` y `recordChildFailure` | Cubierto por contratos concretos | AT-SUP-021 y AT-SIG-005 |
| `TimerScheduler` (startSingleTimer, startTimerWithFixedDelay, startTimerAtFixedRate, startPeriodicTimer, cancel, cancelAll, isTimerActive) y generaciones de `TimerMsg` | Cubierto por contratos concretos | AT-TMR-001..008 |
| `NotInfluenceReceiveTimeout` en ticks de timer | Cubierto por contratos concretos | AT-TMR-007.2 (regla del ActorCell: AC-CELL) |
| `StashBuffer` (stash, unstash, unstashAll, clear, foreach, contains, exists, head, size, capacity, isFull, isEmpty, nonEmpty) | Cubierto por contratos concretos | AT-STASH-001..006 |
| `StashOverflowException` en stash de supervision y en arranque del guardian | Cubierto por contratos concretos | AT-SUP-009, AT-SYS-003 |
| `reference.conf`: `pekko.actor.typed.restart-stash-capacity` | Cubierto por contratos concretos | AT-SYS-006 |
| `reference.conf`: `pekko.actor.typed.default-mailbox`, `library-extensions`, `extensions`, `use-slf4j`, `reliable-delivery` | Fuera de estos modulos con frontera explicita | DSP, EXT, LOG, DLV (cada clave tiene fila en su grupo; no son de typed-core) |
| `reference.conf` clasico: `pekko.log-config-on-start` | Cubierto por contratos concretos | AT-SYS-007.3 (A4) y AC-SYS |
| `scaladsl/package.scala` (`LoggerOps`) | Fuera de estos modulos con frontera explicita | AT-LOG-004 |
| `internal/ActorMdc`, `LoggerClass`, `WithMdcBehaviorInterceptor` | Cubierto por contratos concretos | AT-BEH-019 (claves MDC) y AT-LOG-*; Q-T-BEH-6 |
| `internal/InternalMessage` (AdaptMessage, AdaptWithRegisteredMessageAdapter) | Detalle interno sin contrato independiente (justificado) | Su unico efecto observable (adaptacion en el hilo del actor y descarte de `null`) esta en AT-CTX-011 y AT-CTX-014 |
| `internal/SystemMessage`, `internal/ActorFlightRecorder`, `internal/PropsImpl` | Detalle interno sin contrato independiente (justificado) | Plumbing de la arquitectura en capas (Q-X-8) o instrumentacion JFR (DLV); sin comportamiento propio observable |
| `ActorSystem` classic rechaza `actorOf` de nivel superior con guardian typed (`ActorSystem.scala:927-938`) | Fuera de estos modulos con frontera explicita | contracts/classic-system.md (AC-SYS); AT-SUP-020 y AT-SYS-011 lo citan |
| Tests Java de `actor-typed-tests/src/test/java` (InterceptTest, WatchTest, StashBufferTest, AdapterTest...) | Duplicado o fachada equivalente | Repiten escenarios Scala por la fachada javadsl (AT-JAPI-*); no definen reglas distintas |
| Ejemplos `docs/` y `jdocs/` de `actor-typed-tests` | Detalle interno sin contrato independiente (justificado) | Documentacion ejecutable; las reglas que ilustran estan en las filas anteriores |
| `OrElseSpec` | Detalle interno sin contrato independiente (justificado) | Composicion escrita en el propio test; sin API en 1.7.1 (`OrElseSpec.scala:25-29`) |
| Capacidades de testkit para estas filas (`BehaviorTestKit`, `ManualTime`, `LoggingTestKit`, `TestProbe`, `ActorTestKit`) | Fuera de estos modulos con frontera explicita | AT-TK-* (grupo testkit); aqui solo se cita su uso como evidencia |
| Casos no afirmados por spec (textos de excepcion, igualdades, `Dropped` en capacidad 0, `toString`) | Cubierto por contratos concretos | Se marcan "no afirmada por spec" y llevan caracterizacion con oraculo en cada fila |
| Diferencias Java/Scala: `startPeriodicTimer` (Java=FixedDelay, Scala=FixedRate), `withResetBackoffAfter` (doc Java), `withMaxRestarts` | Cubierto por contratos concretos | AT-TMR-005, AT-SUP-007.2, AT-SUP-007.3 (Q-T-TMR-3, Q-T-SUP-3) |
| Filas nuevas agregadas en este cierre | Cubierto por contratos concretos | Ninguna: no se hallaron comportamientos de typed-core sin fila; las omisiones detectadas (hooks del ActorAdapter, adaptadores del package object, `actorOf` de nivel superior) pertenecen a ADP o a AC-SYS y se anotan como frontera |

**Contradicciones nuevas halladas en este cierre.**

18. AT-TMR-007.2: la nota de `listed` decia que `BehaviorTestKitSpec.scala:444-495` ayudaba a fijar NotInfluenceReceiveTimeout; esas lineas son tests de timers del testkit y no tratan el marker. Corregido arriba.
19. AT-CTX-008: la descripcion afirma que los mensajes NotInfluenceReceiveTimeout no reinician el plazo, pero ningun spec typed lo afirma; se agrego A3 que remite a `ReceiveTimeoutSpec` clasico y marca la caracterizacion con oraculo.
20. AT-SUP-008: los casos de excepcion en el setup (:736-832) estaban "leidos hasta la preparacion"; ahora se afirman en A6/A7. Son ademas la unica evidencia de spec del comportamiento de AT-SUP-005 (setup que falla durante un restart con limite) y alimentan Q-T-SUP-4.
21. AT-SUP-020: se agrego `PropsAdapter` (spec :152-160) a la fila; el inventario citaba solo hasta :150.

**Notas de frontera Classic/Typed para coordinar con typed-extended (ADP), sin editar ese archivo.** (1) `package object adapter` completo (`scaladsl/adapter/package.scala:49-195`) y `javadsl/Adapter.scala` son de ADP: typed-core solo contrata el default Stop de `spawn/spawnAnonymous/systemActorOf/PropsAdapter` (AT-SUP-020) y el puente de contexto (AT-CTX-017). (2) El hook `ActorAdapter.preRestart/postRestart` y `ComposedStoppingBehavior` estan contratados en AT-ADP-006.3 y AT-BEH-004; ADP debe citar `ActorAdapter.scala:309-323, 342-361`. (3) `ActorSystemAdapter.settings/startTime/uptime/threadFactory/dynamicAccess/printTree/logConfiguration` delegan en el sistema clasico: typed-core los contrata como accesores (AT-SYS-007.x); ADP solo debe describir la delegacion, no la semantica. (4) `AdapterExtension` (una instancia por sistema): AT-SYS-011.
