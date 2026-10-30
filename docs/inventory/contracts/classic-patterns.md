# Contratos: classic-patterns (ASK, FSM, RT, CS)

Fuente de verdad: Apache Pekko 1.7.1 (`upstream/pekko`, 9a54d129). Rutas de tests relativas a `upstream/pekko/`. Abreviaturas: `AT` = `actor-tests/src/test/scala/org/apache/pekko`, `AM` = `actor/src/main/scala/org/apache/pekko`, `RC` = `actor/src/main/resources/reference.conf`. Dentro de las secciones, siglas de archivo: `ASKS` = `AM/pattern/AskSupport.scala`, `CBS` = `AM/pattern/CircuitBreaker.scala`, `CBT` = `AT/pattern/CircuitBreakerSpec.scala`, `CBM` = `AT/pattern/CircuitBreakerMTSpec.scala`, `BOS`/`BORS` = `AM/pattern/internal/BackoffOn{Stop,Restart}Supervisor.scala`, `HB` = `AM/pattern/HandleBackoff.scala`, `BST`/`BORT` = `AT/pattern/Backoff{,OnRestart}SupervisorSpec.scala`, `F` = `AM/actor/FSM.scala`, `C` = `AM/actor/CoordinatedShutdown.scala`, `CT` = `AT/actor/CoordinatedShutdownSpec.scala`. Las citas `path:linea` verificadas contra el arbol fijado; las marcadas *no afirmada por spec* se derivan del codigo.

## AC-ASK-001.1 - ask / `?` sobre ActorRef: ref temporal PromiseActorRef como sender, Future completado con la primera respuesta

- **Contrato observable**: [contractual por spec] ask envia el mensaje con una referencia temporal como remitente; el Future completa con el primer mensaje que reciba la referencia temporal (valor tal cual). El sender implicito del ask no cambia el remitente visto por el destino: el destino ve la ref temporal.
- **Fuente**: actor/src/main/scala/org/apache/pekko/pattern/AskSupport.scala:337-390,703-745 ; docs/src/main/paradox/actors.md (seccion Ask) ; tests: actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:34-39
- **Condiciones**: Normal: respuesta antes del timeout. Limites/errores: ver AC-ASK-001.2 y .3. Efecto: al completar la promesa la ref temporal se detiene (ASKS:740-743, `result.future.onComplete { try a.stop() finally f.cancel() }`).
- **Orden / concurrencia / ciclo de vida**: La respuesta se completa desde el hilo del emisor de la respuesta (`!` de la ref temporal es sincrono: actor/src/main/scala/org/apache/pekko/pattern/AskSupport.scala:614-632). Solo la primera respuesta completa; las siguientes van a deadLetters (ver AC-ASK-002).
- **Dependencias**: AC-SYS-017 ; AC-UTIL-001 ; AC-ASK-002
- **Equivalente Rust propuesto**: `async fn ask<M, R>(target: &ActorRef, msg: M, timeout: Duration) -> Result<R, AskError>`; internamente crea una TemporaryReplyRef con tokio::sync::oneshot y la pasa como sender. **Diferencias observables**: En Rust el resultado es tipado (`R`) y el fallo de tipo es un error propio (en Scala `mapTo` falla con ClassCastException); el `sender` implicito se pasa explicito.
- **Aserciones**:
  - A1: ask a un actor eco que responde `sender() ! x` completa con "ping" - actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:34-39.
  - A2: El mensaje se entrega con la ref temporal como sender (se deduce de que `sender() ! x` responde) - actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:36-38; derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/AskSupport.scala:372-388 (`PromiseActorRef(...).ask(actorRef, message, timeout)` hace `actorRef.tell(message, this)`, actor/src/main/scala/org/apache/pekko/pattern/AskSupport.scala:687-690).
- **Oraculo**: oraculo: jshell con pekko-actor 1.7.1 permite comparar mensajes de excepcion y resultados deterministas; los tiempos solo con tolerancia.
- **Decisiones**: Q-ASK-1, Q-X-3, Q-X-4, Q-X-9
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-ASK-001.2 - ask: entradas invalidas (timeout <= 0, destinatario no soportado, destinatario ya terminado/deadLetters/EmptyLocalActorRef)

- **Contrato observable**: [contractual por spec] Sin enviar nada (salvo el caso terminado) el Future falla de inmediato: timeout <= 0 -> IllegalArgumentException; ref no InternalActorRef (p. ej. null) -> IllegalArgumentException; ref terminada (deadLetters, EmptyLocalActorRef) -> el mensaje se re-envia a esa ref (queda como dead letter) y el Future falla con AskTimeoutException.
- **Fuente**: actor/src/main/scala/org/apache/pekko/pattern/AskSupport.scala:286-332,372-388 ; tests: actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:41-92
- **Condiciones**: Mensaje de partes: `Message of type [<clase>]` y, solo si el sender no es noSender, ` was sent by [<sender>]` y punto final (ASKS:292-298). `message == null` imprime tipo "unknown".
- **Orden / concurrencia / ciclo de vida**: La validacion es sincrona en la llamada a ask (antes de crear la ref temporal); no hay scheduler involucrado.
- **Dependencias**: AC-ASK-001.1 ; AC-REF-002
- **Equivalente Rust propuesto**: enum `AskError { Timeout{..}, InvalidTimeout{..}, UnsupportedRecipient{..}, RecipientTerminated{..} }` con los mensajes exactos de Pekko en `Display`. **Diferencias observables**: El nombre de clase del mensaje se reemplaza por `std::any::type_name` del mensaje (la plataforma no tiene `getClass.getName`): el texto difiere en ese fragmento (`java.lang.String` vs `alloc::string::String`).
- **Aserciones**:
  - A1: ask a system.deadLetters completa ya (`isCompleted`) con Failure(AskTimeoutException) - actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:41-48.
  - A2: ask a EmptyLocalActorRef (`resolveActorRef("/user/unknown")`) completa ya con AskTimeoutException - actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:50-59.
  - A3: ask(null: ActorRef, 3.14) falla con IllegalArgumentException y mensaje exacto "Unsupported recipient type, question not sent to [null]. Message of type [java.lang.Double]." - actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:61-70.
  - A4: Timeout(0 s) -> IllegalArgumentException "Timeout length must be positive, question not sent to [$echo]. Message of type [java.lang.String]." - actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:72-81.
  - A5: Timeout(-1000 s) -> el mismo mensaje exacto - actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:83-92.
  - A6: Destinatario ya terminado: el mensaje se envia igualmente a la ref y falla con AskTimeoutException "Recipient [$recipient] had already been terminated. Message of type [...]" (+ " was sent by [$sender]." si hay sender) - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/AskSupport.scala:299-331,372-376; *no afirmada por spec* en el texto exacto (el spec solo comprueba el tipo).
- **Oraculo**: oraculo: jshell con pekko-actor 1.7.1 permite comparar mensajes de excepcion y resultados deterministas; los tiempos solo con tolerancia.
- **Decisiones**: Q-ASK-1
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-ASK-001.3 - ask: AskTimeoutException al vencer el timeout (target, timeout, sender, clase del mensaje)

- **Contrato observable**: [contractual por spec] Si no hay respuesta en `timeout`, el Future falla con AskTimeoutException cuyo mensaje contiene el destino, el timeout en ms, la clase del mensaje y el sender si lo hay.
- **Fuente**: actor/src/main/scala/org/apache/pekko/pattern/AskSupport.scala:703-745 (scheduleOnce con el texto), actor/src/main/scala/org/apache/pekko/pattern/AskSupport.scala:40-45 (AskTimeoutException extiende TimeoutException con NoStackTrace) ; tests: actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:94-126
- **Condiciones**: La tarea de timeout se cancela si la promesa completa antes (ASKS:740-743).
- **Orden / concurrencia / ciclo de vida**: El timeout corre en el scheduler del sistema (`provider.guardian.underlying.system.scheduler`) y completa con `tryComplete`: gana la primera de respuesta/timeout.
- **Dependencias**: AC-ASK-001.1 ; AC-SCH-001
- **Equivalente Rust propuesto**: AskError::Timeout { target: String, timeout: Duration, message_type: &str, sender: Option<String> } con `Display` igual a Pekko. **Diferencias observables**: ninguna prevista.
- **Aserciones**:
  - A1: El mensaje contiene el path del destino ("/user/silent") - actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:94-101.
  - A2: Contiene el timeout en milisegundos (`timeout.duration.toMillis.toString`, p. ej. "500") - actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:103-109.
  - A3: Contiene `sender.toString` cuando se ask con sender implicito - actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:111-118.
  - A4: Contiene "[java.lang.Integer" para un mensaje Integer - actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:120-126.
  - A5: Texto completo (codigo): "Ask timed out on [<target>] after [<ms> ms]. Message of type [<clase>]( was sent by [<sender>])?. A typical reason for `AskTimeoutException` is that the recipient actor didn't send a reply." - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/AskSupport.scala:722-737; *texto completo no afirmado por spec*.
- **Oraculo**: oraculo: jshell con pekko-actor 1.7.1 permite comparar mensajes de excepcion y resultados deterministas; los tiempos solo con tolerancia.
- **Decisiones**: Q-ASK-1, Q-X-4
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-ASK-002 - PromiseActorRef: ciclo de vida, vigilable, path /temp que codifica el nombre del target, respuestas tardias a deadLetters

- **Contrato observable**: [contractual por spec] La ref temporal es un actor ref minimo: tiene path lazy bajo /temp con prefijo = nombre del destino (o el path del selection con "/"->"_" URL-encoded); se puede vigilar (watch) y al completar se detiene y notifica Terminated; tras detenerse conserva un path valido; mensajes posteriores van a deadLetters; stop externo completa el Future con ActorKilledException("Stopped").
- **Fuente**: actor/src/main/scala/org/apache/pekko/pattern/AskSupport.scala:515-700,703-745 ; actor/src/main/java/org/apache/pekko/pattern/AbstractPromiseActorRef.java (solo offsets Unsafe) ; tests: actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:227-289
- **Condiciones**: Mensaje null -> InvalidMessageException("Message is null") (ASKS:620). Watch/Unwatch ilegales (watchee != this o watcher == this) escriben "BUG: illegal Watch(...)" por System.err (ASKS:634-646).
- **Orden / concurrencia / ciclo de vida**: Estados atomicos: null -> Registering -> path | Stopped -> StoppedWithPath. El path se materializa y registra solo al consultarlo (serializacion, toString, equals, hashCode). Watch sobre ref ya detenida: el watcher recibe DeathWatchNotification de inmediato (ASKS:634-640).
- **Dependencias**: AC-SYS-017 ; AC-DW-001 ; AC-PATH-001
- **Equivalente Rust propuesto**: TemporaryReplyRef { state: AtomicU8/enum {Fresh, Registering, Registered(path), Stopped, StoppedWithPath(path)}, watched_by: Mutex<Vec<ActorRef>>, promise: oneshot::Sender } **Diferencias observables**: Sin Unsafe: AtomicPtr/Mutex. El registro lazy del path es una optimizacion observable solo por toString/equals; se puede registrar de forma perezosa igual.
- **Aserciones**:
  - A1: Watch sobre la ref temporal + enviar "complete" -> Future completa con "complete" y se recibe Terminated en <=1 s - actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:227-246.
  - A2: El nombre de la ref temporal empieza por el nombre del destino ("myName"); para ActorSelection empieza por "_user_myName" - actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:248-266.
  - A3: Tras completar, `promiseActorRef.path.toString` contiene "pathPrefix" (path valido aun detenido) - actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:268-289.
  - A4: Mensaje posterior al stop -> `deadLetters ! DeadLetter(message, sender, this)` - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/AskSupport.scala:614-619; *no afirmada por spec* (si por AskSpec:138-151 indirectamente).
  - A5: Segunda respuesta con la promesa ya completa -> `deadLetters ! message` - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/AskSupport.scala:627-629; *no afirmada por spec*.
  - A6: stop() completa la promesa con Failure(ActorKilledException("Stopped")) y notifica DeathWatchNotification a cada watcher - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/AskSupport.scala:653-680,708; *no afirmada por spec*.
  - A7: Un prefijo con "/" lanza IllegalArgumentException "refPathPrefix must not contain slash, was: <prefijo>" - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/AskSupport.scala:719-720; *no afirmada por spec*.
- **Oraculo**: oraculo: jshell puede obtener el path de la ref temporal (nombre) y confirmar Terminated; no aplica a la maquina de estados interna.
- **Decisiones**: Q-ASK-2
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-ASK-003 - ask sobre ActorSelection y respuestas dirigidas via selection

- **Contrato observable**: [contractual por spec] ask sobre ActorSelection funciona como sobre un ref (la respuesta puede ser ActorIdentity). Una respuesta enviada via `actorSelection(sender().path)` llega; selecciones con `*`, `..` o hijos inexistentes de la ref temporal no resuelven: el Future termina con AskTimeoutException y el mensaje queda como DeadLetter.
- **Fuente**: actor/src/main/scala/org/apache/pekko/pattern/AskSupport.scala:433-471,473-512 ; tests: actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:128-225
- **Condiciones**: Timeout <= 0 -> IllegalArgumentException como en ask de ref (ASKS:458-460). Ancla no InternalActorRef -> IllegalArgumentException "Unsupported recipient type..." (ASKS:468).
- **Orden / concurrencia / ciclo de vida**: Igual que AC-ASK-002; la resolucion de selection es asincrona (mensaje ActorSelectionMessage).
- **Dependencias**: AC-REF-011 ; AC-ASK-002
- **Equivalente Rust propuesto**: ask_selection(selection, msg, timeout) reutilizando el mismo ReplyRef que ask; prefijo = `urlencode(path_string.replace("/", "_"))`. **Diferencias observables**: El mensaje muerto se observa como dead letter de ActorSelectionMessage; en Rust el tipo exacto debe conservarse si se publica en el EventStream (Q-X-6).
- **Aserciones**:
  - A1: (system.actorSelection("/user/select-echo") ? Identify(None)) -> ActorIdentity con ref == echo - actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:128-136.
  - A2: Responder con `context.actorSelection(sender().path) ! x` llega y no genera DeadLetter (200 ms sin dead letters) - actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:138-151.
  - A3: actorSelection("/temp/*") ! x -> AskTimeoutException y se recibe un DeadLetter - actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:153-167.
  - A4: `parent / ".." / "temp" / name` -> AskTimeoutException y DeadLetter - actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:169-188.
  - A5: Seleccion de hijo inexistente (`parent / "missing"`) -> AskTimeoutException y DeadLetter - actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:190-207.
  - A6: Responder a `sender().path / "missing"` -> AskTimeoutException; el DeadLetter lleva ActorSelectionMessage("hi", Vector(SelectChildName("missing")), false) (aserto en :222 dentro del intercept) - actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:209-225.
- **Oraculo**: oraculo: viable para "*" y ".." con pekko-actor oficial (observando DeadLetter).
- **Decisiones**: Q-ASK-1, Q-X-3
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-ASK-004 - Ask explicito (fabrica de mensaje que recibe replyTo)

- **Contrato observable**: [contractual por spec] Variante `ask(ref, replyTo => Msg)`: la fabrica recibe la ref temporal como destinatario de respuesta; `messageClassName` de la ref se fija con la clase del mensaje producido. Soporta ActorSelection.
- **Fuente**: actor/src/main/scala/org/apache/pekko/pattern/AskSupport.scala:391-431,473-512 ; actor/src/main/scala/org/apache/pekko/pattern/extended/package.scala ; tests: actor-tests/src/test/scala/org/apache/pekko/pattern/extended/ExplicitAskSpec.scala:33-59
- **Condiciones**: timeout <= 0: la fabrica se invoca con deadLetters solo para construir el texto del error.
- **Orden / concurrencia / ciclo de vida**: La fabrica se invoca una vez, en el hilo del llamador, antes del envio.
- **Dependencias**: AC-ASK-001.1
- **Equivalente Rust propuesto**: `ask_with(target, |reply_to: ActorRef| Msg { reply_to }, timeout)` **Diferencias observables**: ninguna prevista.
- **Aserciones**:
  - A1: target ? (respondTo => Request(respondTo)) completa con Response(target) - actor-tests/src/test/scala/org/apache/pekko/pattern/extended/ExplicitAskSpec.scala:33-44.
  - A2: Lo mismo sobre ActorSelection - actor-tests/src/test/scala/org/apache/pekko/pattern/extended/ExplicitAskSpec.scala:46-59.
  - A3: Destinatario terminado: la fabrica recibe deadLetters, el mensaje se envia y falla con AskTimeoutException - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/AskSupport.scala:403-407; *no afirmada por spec*.
  - A4: Sin InternalActorRef y sender == null -> IllegalArgumentException "No recipient for the reply was provided, question not sent to [<ref>]." - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/AskSupport.scala:418-421; *no afirmada por spec*.
- **Oraculo**: oraculo: viable (jshell) para el caso feliz.
- **Decisiones**: Q-ASK-1
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-ASK-005 - StatusReply (Success/Error/Ack) y askWithStatus

- **Contrato observable**: [contractual por spec] StatusReply<T> es Success(valor) o Error(excepcion); Ack = Success(Done); Error(String) crea ErrorMessage (RuntimeException sin stacktrace cuyo toString es el texto). askWithStatus hace ask y aplana: Success->valor, Error->fallo con la excepcion. null no es Success ni Error.
- **Fuente**: actor/src/main/scala/org/apache/pekko/pattern/StatusReply.scala:37-184 ; actor/src/main/scala/org/apache/pekko/pattern/AskSupport.scala:345-360 ; tests: actor-tests/src/test/scala/org/apache/pekko/pattern/StatusReplySpec.scala:31,39,46,53,60,67-77,82,90,98
- **Condiciones**: ErrorMessage es case class con igualdad por texto (los tests comparan con ===).
- **Orden / concurrencia / ciclo de vida**: n/a: valores inmutables.
- **Dependencias**: AC-ASK-001.1 ; AC-UTIL-001
- **Equivalente Rust propuesto**: `enum StatusReply<T> { Success(T), Error(Box<dyn Error + Send + Sync>) }` + `StatusReply::ack()` + `ask_with_status` **Diferencias observables**: Error ya no es `Throwable`: ErrorMessage debe seguir comparandose por texto.
- **Aserciones**:
  - A1: Patron Success(text) distingue tipos y ack coincide con StatusReply.Ack - actor-tests/src/test/scala/org/apache/pekko/pattern/StatusReplySpec.scala:31-45.
  - A2: null no empareja ni Success ni Error - actor-tests/src/test/scala/org/apache/pekko/pattern/StatusReplySpec.scala:46-52.
  - A3: Error(String) y Error(Throwable) empareja Error - actor-tests/src/test/scala/org/apache/pekko/pattern/StatusReplySpec.scala:53-65.
  - A4: flattenStatusFuture: Success("woho")->"woho"; Ack->Done; Error("boo")->falla con ErrorMessage("boo"); Error(TestException("boo"))->falla con esa excepcion - actor-tests/src/test/scala/org/apache/pekko/pattern/StatusReplySpec.scala:67-77.
  - A5: askWithStatus desenvuelve Success - actor-tests/src/test/scala/org/apache/pekko/pattern/StatusReplySpec.scala:82-88; Error con mensaje -> ErrorMessage("boho") - :90-96; Error con excepcion -> esa excepcion - :98-104.
  - A6: StatusReply(null) lanza InvalidMessageException("[null] is not an allowed status"); getError sobre exito lanza IllegalArgumentException("Expected reply to be a failure, but was a success"); toString "Success(x)"/"Error(msg)" - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/StatusReply.scala:37-69; *no afirmada por spec*.
  - A7: Respuesta que no es StatusReply: el `mapTo[StatusReply[Any]]` falla con ClassCastException - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/AskSupport.scala:356-358; *no afirmada por spec*.
- **Oraculo**: oraculo: viable para flattenStatusFuture via askWithStatus (jshell).
- **Decisiones**: Q-X-9 (Done), Q-ASK-1
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-ASK-006 - pipeTo / pipeToSelection (Future y CompletionStage) y Status.Failure

- **Contrato observable**: [contractual por spec] pipeTo envia el valor del Future al destino con el sender dado; si el Future falla envia Status.Failure(causa). Devuelve el mismo Future (andThen). Version CompletionStage: no envia si el valor es null.
- **Fuente**: actor/src/main/scala/org/apache/pekko/pattern/PipeToSupport.scala:28-122 ; actor/src/main/scala/org/apache/pekko/actor/Actor.scala (Status.Success/Failure) ; tests: actor-tests/src/test/scala/org/apache/pekko/pattern/PipeToSpec.scala:29,35,41,49,55,66,73,80,89,96
- **Condiciones**: El envio ocurre en el hilo que completa el Future (andThen sobre el ExecutionContext dado).
- **Orden / concurrencia / ciclo de vida**: Envio posterior a la completion; sin garantia entre dos pipeTo distintos.
- **Dependencias**: AC-REF-002 ; AC-ASK-002
- **Equivalente Rust propuesto**: `fn pipe_to<T>(fut: impl Future<Output=Result<T,E>>, target: ActorRef, sender: Option<ActorRef>)` que hace spawn en el runtime **Diferencias observables**: Un Future que falla en Rust es `Err(E)`; Status.Failure(e) debe envolver ese error (Q-X-5).
- **Aserciones**:
  - A1: Future(42).pipeTo(p.ref) -> p recibe 42 - actor-tests/src/test/scala/org/apache/pekko/pattern/PipeToSpec.scala:29-33.
  - A2: Future.failed(new Exception("failed")) -> p recibe Status.Failure con causa de mensaje "failed" - actor-tests/src/test/scala/org/apache/pekko/pattern/PipeToSpec.scala:35-39.
  - A3: Con sender implicito s, `p.lastSender == s` - actor-tests/src/test/scala/org/apache/pekko/pattern/PipeToSpec.scala:41-47.
  - A4: Idem para pipeToSelection (valor, fallo, sender) - actor-tests/src/test/scala/org/apache/pekko/pattern/PipeToSpec.scala:66-87; y formas Java `pipe(f) to ref` / `.to(ref, sender)` - :49-64,89-104.
  - A5: CompletionStage con valor null no envia nada; con excepcion envia Status.Failure(ex) - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/PipeToSupport.scala:57-62; *no afirmada por spec*.
- **Oraculo**: oraculo: viable (jshell) para valor/fallo.
- **Decisiones**: Q-X-4, Q-X-5
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-ASK-007 - gracefulStop

- **Contrato observable**: [contractual por spec] Vigila el actor, le envia stopMessage (default PoisonPill) y completa true cuando llega su Terminated; si ya estaba muerto completa; si no termina dentro del timeout el Future falla con AskTimeoutException (y se desvigila).
- **Fuente**: actor/src/main/scala/org/apache/pekko/pattern/GracefulStopSupport.scala:25-70 ; docs/src/main/paradox/actors.md (Graceful stop) ; tests: actor-tests/src/test/scala/org/apache/pekko/pattern/PatternSpec.scala:38-56
- **Condiciones**: La ref de espera es una PromiseActorRef con el timeout dado (se reusa su AskTimeoutException).
- **Orden / concurrencia / ciclo de vida**: Orden: Watch (mensaje de sistema) antes del stopMessage.
- **Dependencias**: AC-DW-001 ; AC-ASK-002
- **Equivalente Rust propuesto**: `async fn graceful_stop(target: &ActorRef, timeout: Duration, stop: impl Into<Msg>) -> Result<bool, AskError>` **Diferencias observables**: El tipo `Boolean` se mantiene; el error de timeout es AskError::Timeout.
- **Aserciones**:
  - A1: gracefulStop(target, 5 s) -> true - actor-tests/src/test/scala/org/apache/pekko/pattern/PatternSpec.scala:38-42.
  - A2: Sobre un actor ya terminado el Future completa (no cuelga) - actor-tests/src/test/scala/org/apache/pekko/pattern/PatternSpec.scala:44-48.
  - A3: Actor que no termina en 500 ms -> falla con AskTimeoutException - actor-tests/src/test/scala/org/apache/pekko/pattern/PatternSpec.scala:50-56.
  - A4: Si el resultado no es Terminated del mismo path o falla, se envia Unwatch(target, ref) antes de devolver false/propagar - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/GracefulStopSupport.scala:65-68; *no afirmada por spec*.
- **Oraculo**: oraculo: viable (jshell) para true/AskTimeoutException.
- **Decisiones**: Q-ASK-1
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-ASK-008 - after / timeout (FutureTimeoutSupport)

- **Contrato observable**: [contractual por spec] after(d)(f): evalua f tras d con el scheduler (si d < 1 se evalua de inmediato; excepcion al evaluar -> Future fallido). timeout(d)(f): evalua f ya; si no completo al vencer d falla con TimeoutException("Timeout of <d> expired") y cancela un CompletableFuture subyacente.
- **Fuente**: actor/src/main/scala/org/apache/pekko/pattern/FutureTimeoutSupport.scala:27-156 ; tests: actor-tests/src/test/scala/org/apache/pekko/pattern/PatternSpec.scala:59-93
- **Condiciones**: after con duracion no finita se programa igual (scheduler); timeout no vuelve a evaluar f.
- **Orden / concurrencia / ciclo de vida**: El callback de timeout corre en el scheduler; el cancel del timer ocurre en el hilo que completa f (parasitic).
- **Dependencias**: AC-SCH-001
- **Equivalente Rust propuesto**: `async fn after<T>(d, fut)`/`timeout(d, fut)` sobre tokio::time **Diferencias observables**: Las variantes CompletionStage (afterCompletionStage/timeoutCompletionStage) son Java API (ver AC-ASK-017).
- **Aserciones**:
  - A1: after(1 s)(Future.successful(5)) completa con 5 - actor-tests/src/test/scala/org/apache/pekko/pattern/PatternSpec.scala:59-66.
  - A2: after(1 s)(Future.failed(IllegalStateException("Mexico"))) falla con ese mensaje - actor-tests/src/test/scala/org/apache/pekko/pattern/PatternSpec.scala:68-75.
  - A3: timeout(100 ms)(Future.successful(5)) -> 5; failed("ABC") -> falla con "ABC" - actor-tests/src/test/scala/org/apache/pekko/pattern/PatternSpec.scala:78-88.
  - A4: timeout(100 ms)(Future.never) -> TimeoutException - actor-tests/src/test/scala/org/apache/pekko/pattern/PatternSpec.scala:90-93.
  - A5: Texto exacto "Timeout of <duration> expired" y rama d < 1 evaluada inline - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/FutureTimeoutSupport.scala:52-54,109; *no afirmada por spec*.
- **Oraculo**: oraculo: jshell con pekko-actor 1.7.1 permite comparar mensajes de excepcion y resultados deterministas; los tiempos solo con tolerancia.
- **Decisiones**: Q-ASK-1
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-ASK-009.1 - retry: conteo de intentos, excepciones lanzadas como fallos, agotamiento

- **Contrato observable**: [contractual por spec] retry(attempt, attempts) invoca attempt hasta 1 + attempts veces (la ultima ejecucion se devuelve tal cual); exito corta; una excepcion lanzada por attempt cuenta como Future fallido; con attempts = 0 se hace un solo intento.
- **Fuente**: actor/src/main/scala/org/apache/pekko/pattern/RetrySupport.scala:30-50,342-397 ; tests: actor-tests/src/test/scala/org/apache/pekko/pattern/RetrySpec.scala:30-95,142-187
- **Condiciones**: `attempt` devuelve null (Future null) -> se devuelve null tal cual (RS:384-385).
- **Orden / concurrencia / ciclo de vida**: Los intentos son secuenciales: el siguiente solo tras completar (y esperar el delay) el anterior.
- **Dependencias**: AC-SCH-001
- **Equivalente Rust propuesto**: `async fn retry<T,E>(attempt: impl FnMut() -> Fut, attempts: u32, policy) -> Result<T,E>`; excepciones lanzadas = panic capturado (Q-X-5) **Diferencias observables**: Future null no existe en Rust; las validaciones de require pasan a tipos (usize no negativo) y mantienen mensajes solo si se exponen via config.
- **Aserciones**:
  - A1: Exito inmediato: devuelve 5 sin demora - actor-tests/src/test/scala/org/apache/pekko/pattern/RetrySpec.scala:30-36; un exito se ejecuta una sola vez - :38-52.
  - A2: Fallos permanentes con attempts=5 -> error "Mexico" - actor-tests/src/test/scala/org/apache/pekko/pattern/RetrySpec.scala:54-60.
  - A3: Exito tras 5 fallos con attempts=10 -> 5 - actor-tests/src/test/scala/org/apache/pekko/pattern/RetrySpec.scala:62-77.
  - A4: Con attempts=5 y exito en el 11o: falla con mensaje "6" y failCount == 6 (1 + 5 intentos) - actor-tests/src/test/scala/org/apache/pekko/pattern/RetrySpec.scala:79-95.
  - A5: attempts=0: un solo intento, en exito devuelve 1 y contador 1; en fallo devuelve "1" con contador 1 - actor-tests/src/test/scala/org/apache/pekko/pattern/RetrySpec.scala:142-170.
  - A6: Excepcion lanzada por attempt tratada como fallo y reintentada - actor-tests/src/test/scala/org/apache/pekko/pattern/RetrySpec.scala:172-187.
  - A7: Validaciones: require "Parameter attempt should not be null.", "Parameter maxAttempts must >= 0.", "Parameter delayFunction should not be null.", "Parameter attempted must >= 0." (IllegalArgumentException) - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/RetrySupport.scala:377-380; *no afirmada por spec*.
- **Oraculo**: oraculo: jshell con pekko-actor 1.7.1 permite comparar mensajes de excepcion y resultados deterministas; los tiempos solo con tolerancia.
- **Decisiones**: Q-X-5
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-ASK-009.2 - retry: demora fija, funcion de demora y backoff exponencial (calculateDelay)

- **Contrato observable**: [contractual por spec] Variantes: delay fijo, delayFunction(attempted)->Option[Duration] (None o duracion < 1 = sin espera), y backoff minBackoff/maxBackoff/randomFactor con delay = min(maxBackoff, minBackoff*2^n) * (1 + rand*randomFactor).
- **Fuente**: actor/src/main/scala/org/apache/pekko/pattern/RetrySupport.scala:115-215,287-292,342-415 ; tests: actor-tests/src/test/scala/org/apache/pekko/pattern/RetrySpec.scala:97-140 ; actor-tests/src/test/scala/org/apache/pekko/pattern/BackoffSupervisorSpec.scala:272-290
- **Condiciones**: calculateDelay: overflow o no finito -> maxBackoff (RS:410-414). random = ThreadLocalRandom.
- **Orden / concurrencia / ciclo de vida**: delay(n) se pide con n = intentos ya hechos + 1 (RS:363-370).
- **Dependencias**: AC-SCH-001
- **Equivalente Rust propuesto**: enum `RetryDelay { None, Fixed(Duration), Func(Box<dyn Fn(u32)->Option<Duration>>), Backoff{min,max,random_factor} }` **Diferencias observables**: Con randomFactor > 0 el resultado es aleatorio: comparar con tolerancia/semilla; la formula debe coincidir exactamente con randomFactor = 0.
- **Aserciones**:
  - A1: Con delayFunction `attempted => Some(100ms*attempted)`, attempts=5: el ultimo `attempted` observado es 5, error "6" - actor-tests/src/test/scala/org/apache/pekko/pattern/RetrySpec.scala:97-118.
  - A2: 1000 reintentos sin delay completan en < 100 ms (rama sin scheduler) - actor-tests/src/test/scala/org/apache/pekko/pattern/RetrySpec.scala:120-140.
  - A3: calculateDelay: (0,0m,0m,0)->0m; (0,5m,7m,0)->5m; (2,5s,7s,0)->7s; (2,5s,7d,0)->20s; (29,5m,10m,0)->10m; (29,10000d,10000d,0)->10000d; (Int.MaxValue,10000d,10000d,0)->10000d - actor-tests/src/test/scala/org/apache/pekko/pattern/BackoffSupervisorSpec.scala:272-290.
  - A4: Validaciones de backoff: minBackoff no null, "Parameter minBackoff must be > 0", "Parameter maxBackoff must be >= minBackoff", "randomFactor must be between 0.0 and 1.0" - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/RetrySupport.scala:179-183; *no afirmada por spec*.
  - A5: delayFunction que devuelve null -> Future fallido con IllegalArgumentException "The delayFunction of retry should not return null." - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/RetrySupport.scala:373-374; *no afirmada por spec*.
- **Oraculo**: oraculo: `RetrySupport.calculateDelay` es `public` en `object RetrySupport` (RS:404): comparable via jshell con randomFactor = 0.
- **Decisiones**: Q-X-4
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-ASK-009.3 - retry: predicado shouldRetry (por valor y por excepcion)

- **Contrato observable**: [contractual por spec] retry(attempt, shouldRetry(value, exception), ...) reintenta ademas cuando el valor exitoso cumple el predicado; por defecto solo reintenta si hay excepcion. Si shouldRetry devuelve false ante una excepcion, se devuelve ese fallo sin reintentar. Solo excepciones NonFatal se reintentan.
- **Fuente**: actor/src/main/scala/org/apache/pekko/pattern/RetrySupport.scala:80-84,172-189,252-259,334-339,343,387-391 ; tests: actor-tests/src/test/scala/org/apache/pekko/pattern/RetrySpec.scala:189-218
- **Condiciones**: retryOnException = (_, e) => e != null.
- **Orden / concurrencia / ciclo de vida**: Predicado evaluado en el hilo del ExecutionContext que completa el attempt.
- **Dependencias**: AC-ASK-009.1
- **Equivalente Rust propuesto**: parametro `should_retry: Fn(&Result<T,E>) -> bool` con defecto `result.is_err()` **Diferencias observables**: ninguna prevista.
- **Aserciones**:
  - A1: retry(.., (t,_) => t < 5, 10, 100 ms) con contador creciente devuelve 5 - actor-tests/src/test/scala/org/apache/pekko/pattern/RetrySpec.scala:189-200.
  - A2: Predicado `!e.isInstanceOf[IllegalArgumentException]` ante IllegalArgumentException: falla y el contador queda en 1 - actor-tests/src/test/scala/org/apache/pekko/pattern/RetrySpec.scala:202-218.
  - A3: Con intentos agotados y valor aun "reintentable" se devuelve el resultado de la ultima ejecucion (rama `else tryAttempt()`) - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/RetrySupport.scala:394-396; *no afirmada por spec*.
- **Oraculo**: oraculo: jshell con pekko-actor 1.7.1 permite comparar mensajes de excepcion y resultados deterministas; los tiempos solo con tolerancia.
- **Decisiones**: Q-X-5
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-ASK-010 - PromiseRef / FutureRef

- **Contrato observable**: [contractual por spec] PromiseRef(timeout) crea una ref cuya primera mensaje completa una promesa (sobre una PromiseActorRef con timeout); mensajes posteriores van a deadLetters; FutureRef es la vista de solo lectura; timeout no positivo -> IllegalArgumentException.
- **Fuente**: actor/src/main/scala/org/apache/pekko/pattern/PromiseRef.scala:26-174 ; tests: actor-tests/src/test/scala/org/apache/pekko/pattern/PromiseRefSpec.scala:37-82
- **Condiciones**: AskPromiseRef usa target "unknown", prefijo "deadLetters" y sender deadLetters (PromiseRef.scala:167-168).
- **Orden / concurrencia / ciclo de vida**: n/a
- **Dependencias**: AC-ASK-002
- **Equivalente Rust propuesto**: `PromiseRef<T> { reference: ActorRef, future: impl Future }` y `FutureRef<T>` **Diferencias observables**: ninguna prevista.
- **Aserciones**:
  - A1: Un destino responde `replyTo ! Response` y promiseRef.future completa con Response - actor-tests/src/test/scala/org/apache/pekko/pattern/PromiseRefSpec.scala:37-46.
  - A2: PromiseRef(-5.seconds) lanza IllegalArgumentException - actor-tests/src/test/scala/org/apache/pekko/pattern/PromiseRefSpec.scala:48-52.
  - A3: Solo recibe un mensaje: el segundo (SecondMessage) llega como DeadLetter - actor-tests/src/test/scala/org/apache/pekko/pattern/PromiseRefSpec.scala:54-65.
  - A4: PromiseRef.wrap(alice, promise) (private[pekko]) permite completar una promesa existente - actor-tests/src/test/scala/org/apache/pekko/pattern/PromiseRefSpec.scala:67-82.
  - A5: Mensaje del error: "Timeout length must not be negative, was: <timeout>" tambien para timeout = 0 (condicion `length > 0`) - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/PromiseRef.scala:166-172; *no afirmada por spec* (discrepancia texto/condicion: Q-ASK-2).
- **Oraculo**: oraculo: jshell con pekko-actor 1.7.1 permite comparar mensajes de excepcion y resultados deterministas; los tiempos solo con tolerancia.
- **Decisiones**: Q-ASK-2
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-ASK-011.1 - CircuitBreaker: maquina de estados Closed/Open/HalfOpen, conteo de fallos, CircuitBreakerOpenException

- **Contrato observable**: [contractual por spec] Closed cuenta fallos consecutivos (un exito reinicia a 0) y pasa a Open al llegar a maxFailures; Open falla rapido con CircuitBreakerOpenException(remainingDuration) hasta resetTimeout; tras resetTimeout pasa a HalfOpen, que deja pasar UNA llamada: exito -> Closed, fallo -> Open; el resto de llamadas en HalfOpen falla con CircuitBreakerOpenException(0).
- **Fuente**: actor/src/main/scala/org/apache/pekko/pattern/CircuitBreaker.scala:700-716,778-780,940-1141 ; actor/src/main/scala/org/apache/pekko/pattern/package.scala ; tests: actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerSpec.scala:136-193,196-320,322-462,464-528,530-645 ; actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerMTSpec.scala:65-117
- **Condiciones**: Transiciones solo si el CAS de estado tiene exito (CBS:700-704); Open->HalfOpen programada con scheduler al entrar en Open.
- **Orden / concurrencia / ciclo de vida**: Cada transicion notifica sus listeners en el ExecutionContext del breaker (executor.execute), no en linea; el orden relativo entre listeners de distinto evento no esta garantizado.
- **Dependencias**: AC-SCH-001 ; ext: tokio::time / scheduler (AC-SCH-001)
- **Equivalente Rust propuesto**: `struct CircuitBreaker` con `enum State { Closed(AtomicU32), HalfOpen(AtomicBool), Open(AtomicI64 instant) }` y transiciones por CAS **Diferencias observables**: currentFailureCount, isOpen, isHalfOpen, isClosed son "power API" (CBS:512-530) y se exponen igual.
- **Aserciones**:
  - A1: Sync/open: llamada antes del resetTimeout lanza CircuitBreakerOpenException con 0 < remainingDuration <= resetTimeout - actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerSpec.scala:136-146.
  - A2: Open pasa a HalfOpen al vencer resetTimeout (onHalfOpen) - actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerSpec.scala:148-152,170-174.
  - A3: succeed()/fail() en Open no cambian el estado (sigue Open) - actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerSpec.scala:154-168.
  - A4: HalfOpen: una llamada exitosa cierra (onClose) - actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerSpec.scala:196-202; una llamada que lanza reabre (onOpen) - :217-224; fail() reabre - :237-244; succeed() cierra - :246-252.
  - A5: HalfOpen: mientras la unica llamada esta en curso, otra llamada falla con CircuitBreakerOpenException y dispara onCallBreakerOpen - actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerSpec.scala:299-309.
  - A6: Closed: llamada exitosa devuelve el valor - actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerSpec.scala:323-326; fallo incrementa `currentFailureCount` y abre con maxFailures=1 - :328-334; exito reinicia el conteo - :356-366; fail() incrementa - :348-354; succeed() reinicia - :392-402.
  - A7: Concurrencia: 100 llamadas en Closed sin errores -> todas "succeed" - actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerMTSpec.scala:65-70; Open -> todas CBO - :72-79; HalfOpen -> solo algunas pasan ("succeed" y "CBO") - :81-95; tras cerrar todas "succeed" - :97-117.
  - A8: Mensaje por defecto de la excepcion: "Circuit Breaker is open; calls are failing fast"; sin stacktrace (NoStackTrace) - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/CircuitBreaker.scala:1135-1141; *no afirmada por spec*.
  - A9: remainingDuration en Open = max(0, currentResetTimeout - (nanoTime - openedAt)) - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/CircuitBreaker.scala:1078-1085; *no afirmada por spec* en el valor, solo en > 0 y <= reset.
- **Oraculo**: oraculo: jshell con pekko-actor oficial permite secuencias deterministas (maxFailures=1, resets cortos); comparar estados y tipos de excepcion, no tiempos.
- **Decisiones**: Q-ASK-3
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-ASK-011.2 - CircuitBreaker: callTimeout (TimeoutException y fallo contado)

- **Contrato observable**: [contractual por spec] Si la llamada no completa en callTimeout el Future falla con TimeoutException("Circuit Breaker Timed out.") (sin stacktrace), se cuenta como fallo y se dispara onCallTimeout; un callTimeout de Duration.Zero desactiva el timeout.
- **Fuente**: actor/src/main/scala/org/apache/pekko/pattern/CircuitBreaker.scala:782,857-908 ; tests: actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerSpec.scala:285-297,404-424,436-444,611-623,706-722,733-741
- **Condiciones**: La excepcion de timeout es un unico `val timeoutEx` compartido (CBS:782).
- **Orden / concurrencia / ciclo de vida**: El timer se cancela cuando la llamada completa primero; gana la primera de {resultado, timeout} con tryComplete/tryFailure.
- **Dependencias**: AC-ASK-011.1
- **Equivalente Rust propuesto**: tokio::time::timeout alrededor del futuro de la llamada; error `CircuitBreakerError::CallTimeout` **Diferencias observables**: ninguna prevista.
- **Aserciones**:
  - A1: Sync closed: llamada de 200 ms con callTimeout 50 ms lanza TimeoutException y currentFailureCount == 1 - actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerSpec.scala:404-412.
  - A2: El conteo de fallo se incrementa en el momento del timeout, antes de que termine la llamada - actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerSpec.scala:414-424.
  - A3: HalfOpen + timeout dispara onCallTimeout con 0 < tiempo < 2*callTimeout - actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerSpec.scala:285-297.
  - A4: Async closed: incrementa failure count en callTimeout (=1) - actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerSpec.scala:706-722; onCallTimeout - :733-741.
  - A5: Con callTimeout == Duration.Zero se usa otra rama que IGNORA defineFailureFn (cualquier Failure cuenta) - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/CircuitBreaker.scala:863-875; *no afirmada por spec* (Q-ASK-4).
- **Oraculo**: oraculo: jshell con pekko-actor 1.7.1 permite comparar mensajes de excepcion y resultados deterministas; los tiempos solo con tolerancia.
- **Decisiones**: Q-ASK-4
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-ASK-011.3 - CircuitBreaker: constructores, validaciones, backoff exponencial de resetTimeout, maxResetTimeout, randomFactor

- **Contrato observable**: [contractual por spec] Cada vez que reabre, resetTimeout se multiplica por exponentialBackoffFactor (y por 1 + rand*randomFactor) hasta maxResetTimeout; al cerrar vuelve al resetTimeout inicial. Defaults de los constructores cortos: maxResetTimeout = 36500 dias, factor 1.0, randomFactor 0.0.
- **Fuente**: actor/src/main/scala/org/apache/pekko/pattern/CircuitBreaker.scala:145-290,1060-1127 ; actor/src/main/resources/reference.conf:1402-1443 ; tests: actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerSpec.scala:80-82,480-500
- **Condiciones**: No hay validacion de maxFailures/callTimeout/resetTimeout en el constructor.
- **Orden / concurrencia / ciclo de vida**: n/a
- **Dependencias**: AC-ASK-011.1
- **Equivalente Rust propuesto**: `CircuitBreakerConfig { max_failures, call_timeout, reset_timeout, max_reset_timeout, exponential_backoff, random_factor }` **Diferencias observables**: La comparacion `nextResetTimeout < maxResetTimeout` conserva el reset actual cuando el siguiente alcanzaria o superaria el maximo (no lo satura al maximo).
- **Aserciones**:
  - A1: Con factor 5 (nonOneFactorCb: maxFailures=1, call 2000 ms, reset 1000 ms, max 1 dia, factor 5, random 0): la segunda apertura informa remainingDuration mayor que la primera - actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerSpec.scala:480-500.
  - A2: require "exponentialBackoffFactor must be >= 1.0" y "randomFactor must be between 0.0 and 1.0" (IllegalArgumentException) - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/CircuitBreaker.scala:163-164; *no afirmada por spec*.
  - A3: nextResetTimeout = currentResetTimeout * factor * (1 + rand*randomFactor); solo se actualiza si < maxResetTimeout - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/CircuitBreaker.scala:1105-1118; *no afirmada por spec*.
  - A4: Closed._enter restablece resetTimeout inicial y el conteo a 0 - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/CircuitBreaker.scala:964-990; *no afirmada por spec*.
  - A5: withExponentialBackoff(max) y withRandomFactor(f) devuelven un nuevo breaker con esos parametros - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/CircuitBreaker.scala:252-290; *no afirmada por spec*.
  - A6: Defaults reference.conf: max-failures=10, call-timeout=10s, reset-timeout=15s, max-reset-timeout=36500d, exponential-backoff=1.0, random-factor=0.0, exception-allowlist=[] - actor/src/main/resources/reference.conf:1412-1441.
- **Oraculo**: oraculo: jshell con pekko-actor 1.7.1 permite comparar mensajes de excepcion y resultados deterministas; los tiempos solo con tolerancia.
- **Decisiones**: Q-ASK-3
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-ASK-011.4 - CircuitBreaker: callbacks (onOpen/onClose/onHalfOpen/onCallSuccess/onCallFailure/onCallTimeout/onCallBreakerOpen), succeed()/fail(), API sync

- **Contrato observable**: [contractual por spec] Listeners por evento (varios, en CopyOnWriteArrayList); onCall* reciben la latencia en nanosegundos; onCallBreakerOpen se invoca en cada llamada rechazada; succeed()/fail() marcan manualmente exito/fallo; withSyncCircuitBreaker bloquea (Await, callTimeout) y relanza la excepcion del cuerpo.
- **Fuente**: actor/src/main/scala/org/apache/pekko/pattern/CircuitBreaker.scala:358-530,539-700,723-800 ; tests: actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerSpec.scala:37-75,170-193,254-283,426-462,502-528,580-645,723-759
- **Condiciones**: Telemetria (si no es Noop) se registra como listeners adicionales en el constructor (CBS:792-800).
- **Orden / concurrencia / ciclo de vida**: Listeners ejecutados via executor, no sincronicamente: los specs usan latches. Orden de listeners del mismo evento = orden de registro (CopyOnWriteArrayList).
- **Dependencias**: AC-ASK-011.1
- **Equivalente Rust propuesto**: trait/closures `on_open(Fn())`... y `with_circuit_breaker(fut)`; la variante sync = `block_on` documentado como bloqueante o se omite (Q-ASK-5) **Diferencias observables**: ninguna prevista.
- **Aserciones**:
  - A1: onCallBreakerOpen se invoca en llamada rechazada - actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerSpec.scala:176-182.
  - A2: onCallFailure ante excepcion con 0 < t < resetTimeout - actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerSpec.scala:184-192.
  - A3: onCallSuccess en HalfOpen con 0 < t < shortResetTimeout - actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerSpec.scala:254-266; onCallFailure en HalfOpen - :268-283.
  - A4: onCallSuccess/onCallFailure/onCallTimeout/onOpen en estado Closed - actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerSpec.scala:426-462 ; async - :723-759.
  - A5: Un resultado exitoso que defineFailureFn marca fallo dispara solo callFails (no onCallFailure ni onCallSuccess) - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/CircuitBreaker.scala:881-888; *no afirmada por spec*.
  - A6: Con excepcion permitida (allowlist) no se dispara onCallFailure - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/CircuitBreaker.scala:901-903; *no afirmada por spec*.
  - A7: withSyncCircuitBreaker: `Await.result(withCircuitBreaker(Try Future.successful(body)), callTimeout)` - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/CircuitBreaker.scala:452-458; *no afirmada por spec*.
- **Oraculo**: oraculo: jshell con pekko-actor 1.7.1 permite comparar mensajes de excepcion y resultados deterministas; los tiempos solo con tolerancia.
- **Decisiones**: Q-ASK-5, Q-ASK-3
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-ASK-012 - CircuitBreaker: defineFailureFn y exception-allowlist

- **Contrato observable**: [contractual por spec] defineFailureFn(Try[T]) decide si un resultado cuenta como fallo (por defecto: Failure no permitida). exception-allowlist (FQCN, tambien via causa de CompletionException) hace que esas excepciones no cuenten como fallo.
- **Fuente**: actor/src/main/scala/org/apache/pekko/pattern/CircuitBreaker.scala:358-395,802-812,857-908 ; actor/src/main/scala/org/apache/pekko/pattern/CircuitBreakersRegistry.scala:82 ; actor/src/main/resources/reference.conf:1436-1441 ; tests: actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerSpec.scala:83-91,205-214,226-235,336-390,539-568,659-704,761-801
- **Condiciones**: allowlist vacia: isIgnoredException siempre false.
- **Orden / concurrencia / ciclo de vida**: n/a
- **Dependencias**: AC-ASK-011.1 ; AC-ASK-013
- **Equivalente Rust propuesto**: closure `is_failure: Fn(&Result<T,E>) -> bool` + `allowlist: HashSet<&str>` por tipo/nombre de error **Diferencias observables**: El FQCN de excepcion de Java no existe: la allowlist en Rust compara por nombre de tipo registrado (Q-ASK-4).
- **Aserciones**:
  - A1: HalfOpen: excepcion definida como exito cierra el breaker - actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerSpec.scala:204-214.
  - A2: HalfOpen: numero par definido como fallo reabre - actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerSpec.scala:226-235.
  - A3: Closed: numero par cuenta como fallo y el valor se devuelve igual (result == 2) - actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerSpec.scala:336-345.
  - A4: Closed: excepcion definida como exito reinicia el conteo a 0 - actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerSpec.scala:368-390.
  - A5: Identificado: AllowException (allowlist) -> tras Future(throwAllowException) se dispara onCallSuccess (no abre) - actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerSpec.scala:772-777; con defineFailureFn anyExceptionIsFailure la misma excepcion SI abre - :793-800.
  - A6: CompletionException con causa en allowlist se trata como permitida - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/CircuitBreaker.scala:802-806; *no afirmada por spec*.
- **Oraculo**: oraculo: jshell con pekko-actor 1.7.1 permite comparar mensajes de excepcion y resultados deterministas; los tiempos solo con tolerancia.
- **Decisiones**: Q-ASK-4, Q-X-5
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-ASK-013 - CircuitBreakersRegistry (breakers por id desde config)

- **Contrato observable**: [contractual por spec] CircuitBreaker(id)(system) obtiene o crea (computeIfAbsent) un breaker por id usando la seccion `pekko.circuit-breaker.<id>` con fallback a `default`; la telemetria se inicia por id.
- **Fuente**: actor/src/main/scala/org/apache/pekko/pattern/CircuitBreakersRegistry.scala:33-99 ; actor/src/main/scala/org/apache/pekko/pattern/CircuitBreaker.scala:61-68 ; actor/src/main/resources/reference.conf:1402-1443 ; tests: actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerSpec.scala:94-105,761-801
- **Condiciones**: Config ausente de `pekko.circuit-breaker.default` -> ConfigException (no manejado).
- **Orden / concurrencia / ciclo de vida**: computeIfAbsent atomico por clave.
- **Dependencias**: AC-EXT-001 ; AC-CFG-001
- **Equivalente Rust propuesto**: `CircuitBreakerRegistry` como extension: `HashMap<String, Arc<CircuitBreaker>>` bajo Mutex **Diferencias observables**: `get(id)` es private[pekko]; la entrada publica es `CircuitBreaker.apply(id)`.
- **Aserciones**:
  - A1: CircuitBreaker("identified") usa max-failures=1, call-timeout=100 ms, reset-timeout=200 ms y exception-allowlist de la config del test - actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerSpec.scala:94-105,761-801.
  - A2: Si `config.hasPath(id)` -> `config.getConfig(id).withFallback(default)`, si no default - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/CircuitBreakersRegistry.scala:70-73; *no afirmada por spec*.
  - A3: Dos llamadas con el mismo id devuelven la misma instancia - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/CircuitBreakersRegistry.scala:97-98; *no afirmada por spec*.
- **Oraculo**: oraculo: viable (jshell) leyendo isOpen tras fallos con config pasada.
- **Decisiones**: Q-ASK-3, Q-X-1
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-ASK-014 - CircuitBreakerTelemetry (SPI)

- **Contrato observable**: [contractual por spec] Si `pekko.circuit-breaker.telemetry.implementations` tiene una clase se instancia por reflexion (ctor (String breakerId, ExtendedActorSystem)); con varias se usa un Ensemble que reenvia a todas; vacio/ausente -> Noop. Se enlaza a los listeners del breaker.
- **Fuente**: actor/src/main/scala/org/apache/pekko/pattern/internal/CircuitBreakerTelemetry.scala:34-167 ; actor/src/main/scala/org/apache/pekko/pattern/CircuitBreaker.scala:792-800 ; tests: - (sin specs upstream)
- **Condiciones**: Fallo al instanciar -> excepcion (`.get`). `stopped()` esta definido (CircuitBreakerTelemetry.scala:84,137,166) pero ningun archivo de pattern/ lo invoca (grep): hallazgo, nadie lo llama en 1.7.1.
- **Orden / concurrencia / ciclo de vida**: n/a
- **Dependencias**: AC-ASK-011.4 ; AC-ASK-013 ; AC-EXT-001
- **Equivalente Rust propuesto**: trait `CircuitBreakerTelemetry` + registro de fabricas por nombre (Q-X-2) **Diferencias observables**: ninguna prevista.
- **Aserciones**:
  - A1: Ausente la clave -> CircuitBreakerNoopTelemetry; 0 -> Noop; 1 -> instancia; >1 -> Ensemble - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/internal/CircuitBreakerTelemetry.scala (CircuitBreakerTelemetryProvider.start); *no afirmada por spec*.
  - A2: Metodos: onOpen, onClose, onHalfOpen, onCallSuccess(elapsedNanos), onCallFailure(elapsedNanos), onCallTimeoutFailure(elapsedNanos), onCallBreakerOpenFailure, stopped - derivada del codigo (idem); *no afirmada por spec*.
  - A3: La clave de config no aparece en reference.conf (solo se consulta con hasPath) - actor/src/main/resources/reference.conf (grep sin resultados): *hallazgo*; ver Q-ASK-6.
- **Oraculo**: no aplica: no hay specs ni salida observable sin una implementacion de telemetria.
- **Decisiones**: Q-ASK-6, Q-X-2
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-ASK-015.1 - BackoffSupervisor.props / BackoffOpts: parametros, validaciones y mensajes (GetCurrentChild, Reset, GetRestartCount, CurrentChild, RestartCount)

- **Contrato observable**: [contractual] BackoffOpts.onStop/onFailure(childProps, childName, minBackoff, maxBackoff, randomFactor) con with* (AutoReset, ManualReset, SupervisorStrategy, MaxNrOfRetries, ReplyWhileStopped, HandlerWhileStopped, DefaultStoppingStrategy, FinalStopMessage). Mensajes publicos: GetCurrentChild -> CurrentChild(Option[ActorRef]); GetRestartCount -> RestartCount(n); Reset (solo ManualReset).
- **Fuente**: actor/src/main/scala/org/apache/pekko/pattern/BackoffOptions.scala:75-459 ; actor/src/main/scala/org/apache/pekko/pattern/BackoffSupervisor.scala:140-330 ; actor/src/main/scala/org/apache/pekko/pattern/HandleBackoff.scala:25-79 ; tests: actor-tests/src/test/scala/org/apache/pekko/pattern/BackoffSupervisorSpec.scala:68-91 (no leidas) ; actor-tests/src/test/scala/org/apache/pekko/pattern/BackoffSupervisorSpec.scala:141-186
- **Condiciones**: Maximo n reintentos -1 = ilimitado (por defecto en OneForOneStrategy -1).
- **Orden / concurrencia / ciclo de vida**: El hijo se crea en preStart (startChild), observado por watch.
- **Dependencias**: AC-ASK-015.2 ; AC-ASK-015.3
- **Equivalente Rust propuesto**: `BackoffSupervisorConfig` builder + `BackoffSupervisor::props(cfg)`; mensajes como enum `BackoffCommand` **Diferencias observables**: n/a hasta leer los specs.
- **Aserciones**:
  - A1: require "minBackoff must be > 0", "maxBackoff must be >= minBackoff", "randomFactor must be between 0.0 and 1.0"; con AutoReset exige `minBackoff <= resetBackoff <= maxBackoff` (sin mensaje) - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/BackoffOptions.scala:381-386,428-433, actor/src/main/scala/org/apache/pekko/pattern/BackoffSupervisor.scala:183-185; *no afirmada por spec*.
  - A2: Reset en AutoReset se trata como unhandled (`unhandled(msg)`) - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/HandleBackoff.scala:55-59; *no afirmada por spec*.
  - A3: GetRestartCount/GetCurrentChild responden al sender (CurrentChild(child), RestartCount(restartCount)) - actor-tests/src/test/scala/org/apache/pekko/pattern/BackoffSupervisorSpec.scala:122-136 (uso).
- **Oraculo**: oraculo: viable con pekko-actor oficial para respuestas de GetRestartCount/GetCurrentChild.
- **Decisiones**: Q-ASK-7, Q-ASK-3
- **Estado**: analysis=listed, impl=pending, validation=pending

## AC-ASK-015.2 - BackoffOpts.onStop: reinicio del hijo tras su detencion con backoff exponencial

- **Contrato observable**: [contractual por spec] El supervisor crea el hijo; cuando el hijo se detiene (Terminated) programa StartChild tras calculateDelay(restartCount, min, max, randomFactor) e incrementa restartCount; reenvia (forward) los mensajes al hijo. La estrategia por defecto (OneForOne) se compone con la custom: decisiones no definidas caen al decider por defecto (Stop por defecto en onStop salvo la estrategia dada).
- **Fuente**: actor/src/main/scala/org/apache/pekko/pattern/internal/BackoffOnStopSupervisor.scala:40-119 ; actor/src/main/scala/org/apache/pekko/pattern/HandleBackoff.scala:25-79 ; tests: actor-tests/src/test/scala/org/apache/pekko/pattern/BackoffSupervisorSpec.scala:92-139,292-339
- **Condiciones**: Si strategy no es OneForOneStrategy maxNrOfRetries = -1 (BOS:78-81). Cuando se excede: log.debug("Terminating on restart #{} which exceeds max allowed restarts ({})") y context.stop(self) (BOS:89-93).
- **Orden / concurrencia / ciclo de vida**: Terminated -> restartCount+1 -> scheduleOnce(StartChild) -> startChild solo si child.isEmpty.
- **Dependencias**: AC-ASK-015.1 ; AC-ASK-009.2 ; AC-SUP-001
- **Equivalente Rust propuesto**: actor `BackoffOnStopSupervisor` con timer interno y `child: Option<ActorRef>` **Diferencias observables**: ninguna prevista.
- **Aserciones**:
  - A1: Tras detener c1 aparece otro hijo c2 != c1 (awaitAssert GetCurrentChild) - actor-tests/src/test/scala/org/apache/pekko/pattern/BackoffSupervisorSpec.scala:97-103 (estrategia custom) y :122-134 (default).
  - A2: withDefaultStoppingStrategy: tras `c1 ! "boom"` el hijo termina y se reinicia; RestartCount pasa 0 -> 1 - actor-tests/src/test/scala/org/apache/pekko/pattern/BackoffSupervisorSpec.scala:119-138.
  - A3: maxNrOfRetries=2: tras 2 reinicios y una tercera parada el supervisor termina (Terminated) - actor-tests/src/test/scala/org/apache/pekko/pattern/BackoffSupervisorSpec.scala:292-339.
  - A4: RestartCount aumenta 1 por cada Terminated del hijo - actor-tests/src/test/scala/org/apache/pekko/pattern/BackoffSupervisorSpec.scala:318-332.
  - A5: Si finalStopMessageReceived el supervisor se detiene al terminar el hijo (sin reinicio) - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/internal/BackoffOnStopSupervisor.scala:75-76; ver AC-ASK-015.5.
- **Oraculo**: oraculo: jshell con pekko-actor 1.7.1 permite comparar mensajes de excepcion y resultados deterministas; los tiempos solo con tolerancia.
- **Decisiones**: Q-ASK-7, Q-X-5
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-ASK-015.3 - BackoffOpts.onFailure: reinicio del hijo tras fallo con backoff; traduccion de Restart a Stop+StartChild

- **Contrato observable**: [contractual por spec] El supervisor traduce cualquier directiva Restart del decider en Stop del hijo, espera su Terminated y reinicia tras el backoff; maxNrOfRetries/withinTimeRange de la OneForOneStrategy se respetan (reset del contador por ventana); si el hijo termina por si solo el supervisor termina.
- **Fuente**: actor/src/main/scala/org/apache/pekko/pattern/internal/BackoffOnRestartSupervisor.scala:40-124 ; actor/src/main/scala/org/apache/pekko/pattern/HandleBackoff.scala:25-79 ; tests: actor-tests/src/test/scala/org/apache/pekko/pattern/BackoffOnRestartSupervisorSpec.scala:94-100,102-133,135-143,145-150,164-203,205-220,222-254,256-290 ; actor-tests/src/test/scala/org/apache/pekko/pattern/BackoffSupervisorSpec.scala:341-392
- **Condiciones**: Excedido: log.debug igual que onStop y `become(receive); stop(self)` (BORS:77-86).
- **Orden / concurrencia / ciclo de vida**: waitChildTerminatedBeforeBackoff ignora StartChild hasta el Terminated del hijo (BORS:94-102).
- **Dependencias**: AC-ASK-015.1 ; AC-SUP-001
- **Equivalente Rust propuesto**: actor `BackoffOnFailureSupervisor` con supervisor_strategy propia **Diferencias observables**: ninguna prevista.
- **Aserciones**:
  - A1: El hijo terminando por si mismo termina al supervisor - actor-tests/src/test/scala/org/apache/pekko/pattern/BackoffOnRestartSupervisorSpec.scala:94-100.
  - A2: Backoff exponencial: ~200 ms, 400, 800, 1600 entre arranques (STARTED) - actor-tests/src/test/scala/org/apache/pekko/pattern/BackoffOnRestartSupervisorSpec.scala:102-133.
  - A3: Excepciones que la estrategia manda Stop detienen al supervisor - actor-tests/src/test/scala/org/apache/pekko/pattern/BackoffOnRestartSupervisorSpec.scala:135-143.
  - A4: Mensajes del hijo se reenvian al padre del supervisor - actor-tests/src/test/scala/org/apache/pekko/pattern/BackoffOnRestartSupervisorSpec.scala:145-150.
  - A5: Acepta comandos (GetCurrentChild/GetRestartCount) mientras el hijo se esta deteniendo (CurrentChild sigue siendo el hijo hasta Terminated) - actor-tests/src/test/scala/org/apache/pekko/pattern/BackoffOnRestartSupervisorSpec.scala:164-203.
  - A6: Respeta maxNrOfRetries y withinTimeRange; reinicia fuera de la ventana - actor-tests/src/test/scala/org/apache/pekko/pattern/BackoffOnRestartSupervisorSpec.scala:205-290.
  - A7: maxNrOfRetries=2 con onFailure: tras el tercer fallo el supervisor termina - actor-tests/src/test/scala/org/apache/pekko/pattern/BackoffSupervisorSpec.scala:341-392.
- **Oraculo**: oraculo: jshell con pekko-actor 1.7.1 permite comparar mensajes de excepcion y resultados deterministas; los tiempos solo con tolerancia.
- **Decisiones**: Q-ASK-7, Q-X-5
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-ASK-015.4 - BackoffSupervisor: manejo de mensajes mientras no hay hijo (replyWhileStopped, handlerWhileStopped, deadLetters) y reset automatico/manual

- **Contrato observable**: [contractual por spec] Sin hijo: por defecto reenvia a deadLetters; withReplyWhileStopped(x) responde x al sender; withHandlerWhileStopped(ref) reenvia a ref. El contador de reinicios se resetea automaticamente (AutoReset: si tras resetBackoff no hubo mas reinicios) o con el mensaje Reset (ManualReset).
- **Fuente**: actor/src/main/scala/org/apache/pekko/pattern/HandleBackoff.scala:45-78 ; actor/src/main/scala/org/apache/pekko/pattern/internal/BackoffOnStopSupervisor.scala:101-118 ; actor/src/main/scala/org/apache/pekko/pattern/internal/BackoffOnRestartSupervisor.scala:111-124 ; actor/src/main/scala/org/apache/pekko/pattern/BackoffOptions.scala:361-378,452-453 ; tests: actor-tests/src/test/scala/org/apache/pekko/pattern/BackoffSupervisorSpec.scala:141-186,188-270
- **Condiciones**: Mensajes que vienen del hijo (sender == hijo) se reenvian al padre del supervisor (HB:72-74).
- **Orden / concurrencia / ciclo de vida**: ResetRestartCount programado con el scheduler del sistema.
- **Dependencias**: AC-ASK-015.2 ; AC-ASK-015.3
- **Equivalente Rust propuesto**: enum `WhileStopped { DeadLetters, ReplyWith(Msg), ForwardTo(ActorRef) }`; `enum ResetPolicy { Auto(Duration), Manual }` **Diferencias observables**: ninguna prevista.
- **Aserciones**:
  - A1: ManualReset: tras reiniciar RestartCount == 1; enviar `hello` y recibir eco; sin Reset sigue 1 y tras `Reset` vuelve a 0 - actor-tests/src/test/scala/org/apache/pekko/pattern/BackoffSupervisorSpec.scala:141-186.
  - A2: replyWhileStopped: mensaje "boom" con hijo detenido -> el sender recibe "child was stopped" - actor-tests/src/test/scala/org/apache/pekko/pattern/BackoffSupervisorSpec.scala:188-212.
  - A3: handlerWhileStopped(handler): "still there?" -> "not here!" del handler - actor-tests/src/test/scala/org/apache/pekko/pattern/BackoffSupervisorSpec.scala:214-244.
  - A4: Sin replyWhileStopped: el mensaje va a deadLetters (warning ".*boom.*" 1 vez) y el sender no recibe nada - actor-tests/src/test/scala/org/apache/pekko/pattern/BackoffSupervisorSpec.scala:246-270.
  - A5: AutoReset: ResetRestartCount(n) solo resetea si n == restartCount y > 0, programado resetBackoff tras StartChild - derivada del codigo actor/src/main/scala/org/apache/pekko/pattern/HandleBackoff.scala:46-64; *no afirmada por spec* (el spec no usa AutoReset explicito).
- **Oraculo**: oraculo: jshell con pekko-actor 1.7.1 permite comparar mensajes de excepcion y resultados deterministas; los tiempos solo con tolerancia.
- **Decisiones**: Q-ASK-7
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-ASK-015.5 - BackoffOpts.onStop: finalStopMessage

- **Contrato observable**: [contractual por spec] Con withFinalStopMessage(pred): el mensaje que cumple pred se reenvia al hijo y el supervisor se detiene cuando el hijo termina (sin reinicio); si no hay hijo y el mensaje cumple pred, el supervisor se detiene de inmediato. Otros mensajes no detienen el supervisor.
- **Fuente**: actor/src/main/scala/org/apache/pekko/pattern/internal/BackoffOnStopSupervisor.scala:72-77,101-118 ; actor/src/main/scala/org/apache/pekko/pattern/BackoffOptions.scala:343-346,378 ; tests: actor-tests/src/test/scala/org/apache/pekko/pattern/BackoffSupervisorSpec.scala:394-431
- **Condiciones**: finalStopMessage solo existe en onStop (no en onFailure).
- **Orden / concurrencia / ciclo de vida**: finalStopMessageReceived se evalua tras el forward (BOS:104-107).
- **Dependencias**: AC-ASK-015.2
- **Equivalente Rust propuesto**: campo `final_stop_message: Option<Box<dyn Fn(&Msg)->bool>>` **Diferencias observables**: ninguna prevista.
- **Aserciones**:
  - A1: Con finalStopMessage(_ == "stop"): el supervisor reenvia "stop" al hijo (eco) y al morir el hijo (PoisonPill) el supervisor termina - actor-tests/src/test/scala/org/apache/pekko/pattern/BackoffSupervisorSpec.scala:394-408.
  - A2: Sin recibir el final stop message el supervisor no termina aunque muera el hijo; "ping" va a deadLetters (warning) y recien al enviar "stop" el supervisor termina - actor-tests/src/test/scala/org/apache/pekko/pattern/BackoffSupervisorSpec.scala:410-430.
- **Oraculo**: oraculo: jshell con pekko-actor 1.7.1 permite comparar mensajes de excepcion y resultados deterministas; los tiempos solo con tolerancia.
- **Decisiones**: Q-ASK-7
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-ASK-016 - Backoff (API vieja) / BackoffOptions deprecada

- **Contrato observable**: [contractual] API deprecada (desde Akka 2.5.22) reemplazada por BackoffOpts; props(BackoffOptions) delega a options.props.
- **Fuente**: actor/src/main/scala/org/apache/pekko/pattern/Backoff.scala:27-643 ; actor/src/main/scala/org/apache/pekko/pattern/BackoffSupervisor.scala:175-237 ; tests: actor-tests/src/test/scala/org/apache/pekko/pattern/BackoffSupervisorSpec.scala:69 (inventario previo, sin leer)
- **Condiciones**: n/a: no extraido.
- **Orden / concurrencia / ciclo de vida**: n/a: sin garantias de orden propias
- **Dependencias**: AC-ASK-015.1
- **Equivalente Rust propuesto**: Q-ASK-8 propone no portar la API deprecada (equivalente: BackoffOpts) **Diferencias observables**: n/a
- **Aserciones**:
  - A1: Sin aserciones extraidas: no se leyo el cuerpo (listed).
- **Oraculo**: no aplica: API deprecada.
- **Decisiones**: Q-ASK-8
- **Estado**: analysis=listed, impl=pending, validation=pending

## AC-ASK-017 - Patterns / PatternsCS (fachadas Java)

- **Contrato observable**: [contractual] Versiones Java de ask, pipe, gracefulStop, after, retry (delegan a la implementacion Scala).
- **Fuente**: actor/src/main/scala/org/apache/pekko/pattern/Patterns.scala:31-1333 ; tests: actor-tests/src/test/java/org/apache/pekko/pattern/PatternsTest.java (no leido)
- **Condiciones**: n/a: no extraido.
- **Orden / concurrencia / ciclo de vida**: n/a: sin garantias de orden propias
- **Dependencias**: AC-ASK-001.1 ; AC-ASK-006 ; AC-ASK-007 ; AC-ASK-008 ; AC-ASK-009.1
- **Equivalente Rust propuesto**: Q-ASK-9 propone no portar las fachadas Java (el API Rust unico las cubre) **Diferencias observables**: n/a
- **Aserciones**:
  - A1: Sin aserciones extraidas: no se leyo el cuerpo (listed).
- **Oraculo**: no aplica: fachada Java.
- **Decisiones**: Q-ASK-9
- **Estado**: analysis=listed, impl=pending, validation=pending

## AC-FSM-001.1 - DSL de estado: State (goto/stay/stop/using/replying/forMax), Event(event, stateData), StopEvent, Reason, NullFunction

- **Contrato observable**: [contractual por spec] State es inmutable: goto(s) conserva el stateData actual; stay() = goto(estado actual) en modo silencioso (no dispara onTransition); using(d) cambia los datos; replying(x) acumula respuestas (orden de llamada); forMax(d) fija el timeout del estado (Duration.Inf deshabilita, no finito/otro = usa el del estado); stop(reason, data) = stay().using(data) con razon. FSM.NullFunction no esta definida en ningun valor.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/FSM.scala:30-100,172-350,432-535 ; tests: actor-tests/src/test/scala/org/apache/pekko/actor/FSMActorSpec.scala:208-221,353-385 ; actor-tests/src/test/scala/org/apache/pekko/actor/FSMTimingSpec.scala:196-288
- **Condiciones**: Reason = Normal | Shutdown | Failure(cause: Any). SilentState sobrescribe copy para conservar el modo silencioso.
- **Orden / concurrencia / ciclo de vida**: n/a: valores inmutables; el efecto ocurre al devolverse el State al motor (ver .2).
- **Dependencias**: AC-CELL-001 ; AC-FSM-002
- **Equivalente Rust propuesto**: trait `Fsm<S, D>`; `struct FsmState<S, D> { name, data, timeout: Option<Duration>, stop_reason: Option<StopReason>, replies: Vec<Msg>, notifies: bool }` con builder (`goto`, `stay`, `using`, `replying`, `for_max`) **Diferencias observables**: En Rust el stateData generico D y el estado S requieren `Eq + Hash`; los mensajes de respuesta son `Box<dyn Any + Send>` (Q-X-3).
- **Aserciones**:
  - A1: stop(FSM.Normal, "pigdog") entrega a onTermination StopEvent(Normal, 1, "pigdog") (datos actualizados) - actor-tests/src/test/scala/org/apache/pekko/actor/FSMActorSpec.scala:208-221.
  - A2: forMax(Duration.Inf) cancela el state timeout: tras StateTimeout inicial a los 300 ms y un OverrideTimeoutToInf no hay mas StateTimeout durante 1 s - actor-tests/src/test/scala/org/apache/pekko/actor/FSMActorSpec.scala:353-385.
  - A3: goto(x) sin using conserva stateData (`FSM.State(nextStateName, currentState.stateData)`) - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/FSM.scala:500; *no afirmada por spec*.
  - A4: stay() es goto(estadoActual).withNotification(false) (SilentState) - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/FSM.scala:511-513; *no afirmada por spec* (si por FSMTransitionSpec:80-87).
  - A5: forMax: FiniteDuration -> Some(d); Duration.Inf -> marcador Long.MaxValue ns (deshabilita); cualquier otro (p. ej. Undefined/MinusInf) -> None - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/FSM.scala:282-286; *no afirmada por spec*.
  - A6: replying(x) antepone a la lista; al aplicar se envian en orden de llamada (reverse) - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/FSM.scala:306-308,873-875,885-887; *no afirmada por spec*.
  - A7: Estados `State` son iguales por los 5 campos (stateName, stateData, timeout, stopReason, replies) - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/FSM.scala:237-246; *no afirmada por spec*.
- **Oraculo**: oraculo: pekko-actor 1.7.1 oficial via jshell permite ejecutar una FSM Scala/Java (AbstractFSM) con mensajes y comparar secuencias de Transition/respuestas; tiempos solo con tolerancia.
- **Decisiones**: Q-FSM-1
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-FSM-001.2 - Registro de when(estado, stateTimeout), initialize/startWith, despacho de eventos y composicion de funciones de estado

- **Contrato observable**: [contractual por spec] when(estado)(fn) registra una funcion parcial; registrar el mismo estado otra vez compone con orElse (la primera tiene prioridad) y conserva el stateTimeout previo si el nuevo es nulo. Un Event se despacha a la funcion del estado actual; si no esta definida se usa whenUnhandled (por defecto warning + stay). initialize() ejecuta makeTransition(estadoInicial) (dispara onTransition).
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/FSM.scala:475-488,774-792,813-866,719-721 ; tests: actor-tests/src/test/scala/org/apache/pekko/actor/FSMActorSpec.scala:28-107,119-168 ; actor-tests/src/test/scala/org/apache/pekko/actor/FSMTimingSpec.scala:196-288 ; actor-tests/src/test/scala/org/apache/pekko/actor/FSMTransitionSpec.scala:27-36
- **Condiciones**: Estado sin funcion registrada como destino de goto -> Failure("Next state <s> does not exist"), ver AC-FSM-006.1.
- **Orden / concurrencia / ciclo de vida**: Un solo hilo por mensaje (actor): una transicion = (1) enviar replies, (2) onTransition handlers en orden de registro, (3) gossip(Transition) a listeners, (4) currentState = nextState, (5) reprogramar state timeout (F:881-907).
- **Dependencias**: AC-FSM-001.1 ; AC-FSM-005
- **Equivalente Rust propuesto**: registro `HashMap<S, Vec<StateFn>>` y `fn process_event(&mut self, ev: Event<D>) -> FsmState<S,D>`; las "funciones parciales" son closures que devuelven `Option<FsmState>` **Diferencias observables**: Las funciones parciales de Scala (isDefinedAt) se modelan con Option: el orElse es el orden de registro.
- **Aserciones**:
  - A1: Lock FSM: secuencia de teclas correcta transita Locked -> Open y dispara onTransition, listeners y latch de bloqueo tras 1 s - actor-tests/src/test/scala/org/apache/pekko/actor/FSMActorSpec.scala:119-168 (definicion :28-107).
  - A2: FSM.initialize() dispara el onTransition inicial 0 -> 0 ("caused by initialize(), OK") - actor-tests/src/test/scala/org/apache/pekko/actor/FSMTransitionSpec.scala:27-36,80-82.
  - A3: Registrar el mismo estado dos veces: stateFunctions(name) = old.orElse(new), stateTimeouts(name) = new.orElse(old) - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/FSM.scala:774-782; *no afirmada por spec*.
  - A4: processEvent: `stateFunc.isDefinedAt(event)` ? apply : handleEvent(event) - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/FSM.scala:858-866; *no afirmada por spec*.
  - A5: receive procesa antes que el usuario: TimeoutMarker, Timer, SubscribeTransitionCallBack, Listen, UnsubscribeTransitionCallBack, Deafen; cualquier otro valor es un Event del usuario - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/FSM.scala:813-851; *no afirmada por spec*.
- **Oraculo**: oraculo: pekko-actor 1.7.1 oficial via jshell permite ejecutar una FSM Scala/Java (AbstractFSM) con mensajes y comparar secuencias de Transition/respuestas; tiempos solo con tolerancia.
- **Decisiones**: Q-FSM-1
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-FSM-002 - Transiciones: onTransition (goto incluido al mismo estado, no en stay), nextStateData, SubscribeTransitionCallBack/Listen/Deafen

- **Contrato observable**: [contractual por spec] onTransition handlers (PartialFunction[(S,S), Unit]) corren antes del cambio de estado, con stateData = datos previos y nextStateData = datos nuevos; se disparan en goto (incluso al mismo estado) y no en stay(). SubscribeTransitionCallBack(ref)/Listen(ref) registran y responden de inmediato CurrentState(fsmRef, estado); en cada cambio los listeners reciben Transition(fsmRef, from, to). nextStateData fuera de onTransition lanza IllegalStateException.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/FSM.scala:50-70,682-692,741-744,803-806,830-843,881-894 ; actor/src/main/scala/org/apache/pekko/routing/Listeners.scala:20-59 ; tests: actor-tests/src/test/scala/org/apache/pekko/actor/FSMTransitionSpec.scala:80-179 ; actor-tests/src/test/scala/org/apache/pekko/actor/FSMActorSpec.scala:334-351,119-168
- **Condiciones**: Listener muerto: el envio va a dead letters (no se desuscribe).
- **Orden / concurrencia / ciclo de vida**: onTransition handlers se ejecutan en el orden de registro antes de gossip; ambos antes de actualizar currentState (F:889-894).
- **Dependencias**: AC-FSM-001.2 ; AC-RT-015
- **Equivalente Rust propuesto**: `on_transition(Fn(&S,&S,&TransitionCtx))` + `subscribe(ref)`; listeners en `BTreeSet<ActorRef>` ordenado por ActorRef **Diferencias observables**: ninguna prevista.
- **Aserciones**:
  - A1: stay() no dispara onTransition; goto(estado actual) si (0 -> 0) - actor-tests/src/test/scala/org/apache/pekko/actor/FSMTransitionSpec.scala:80-87.
  - A2: SubscribeTransitionCallBack: CurrentState(fsm,0) y luego Transition(fsm,0,1), Transition(fsm,1,0) - actor-tests/src/test/scala/org/apache/pekko/actor/FSMTransitionSpec.scala:89-101.
  - A3: Un listener que desaparece no hace fallar a la FSM (sin mensajes extra) - actor-tests/src/test/scala/org/apache/pekko/actor/FSMTransitionSpec.scala:103-114.
  - A4: En onTransition `stateData`/`nextStateData` son (0,1) para goto(1).using(1) - actor-tests/src/test/scala/org/apache/pekko/actor/FSMTransitionSpec.scala:119-125.
  - A5: goto(1) desde 1 dispara (1,1) y Transition(fsm,1,1); en 0->1 primero el onTransition del usuario y despues el Transition al listener - actor-tests/src/test/scala/org/apache/pekko/actor/FSMTransitionSpec.scala:127-142.
  - A6: stay() no genera Transition a listeners - actor-tests/src/test/scala/org/apache/pekko/actor/FSMTransitionSpec.scala:144-154.
  - A7: Fuera de onTransition `nextStateData` lanza IllegalStateException (no retiene referencias) - actor-tests/src/test/scala/org/apache/pekko/actor/FSMTransitionSpec.scala:156-179.
  - A8: transform(...).using: el listener recibe CurrentState(0) y Transition(0,1) - actor-tests/src/test/scala/org/apache/pekko/actor/FSMActorSpec.scala:334-351.
  - A9: Listen/Deafen/UnsubscribeTransitionCallBack son mensajes FSM (no llegan a las funciones de estado); Listen tambien responde CurrentState - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/FSM.scala:830-843; *no afirmada por spec*.
  - A10: Listeners (trait Listeners) usa `TreeSet[ActorRef]`: suscribir dos veces es idempotente; gossip envia a todos en orden del TreeSet - derivada del codigo actor/src/main/scala/org/apache/pekko/routing/Listeners.scala:37-58; *no afirmada por spec*; ListenerSpec (actor-tests/src/test/scala/org/apache/pekko/actor/routing/ListenerSpec.scala:29-70) cubre Listen/Deafen/WithListeners (ver AC-RT-015).
- **Oraculo**: oraculo: pekko-actor 1.7.1 oficial via jshell permite ejecutar una FSM Scala/Java (AbstractFSM) con mensajes y comparar secuencias de Transition/respuestas; tiempos solo con tolerancia.
- **Decisiones**: Q-FSM-1
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-FSM-003 - State timeout (StateTimeout; cancelado por cualquier mensaje; forMax; Duration.Inf cancela; setStateTimeout)

- **Contrato observable**: [contractual por spec] when(estado, stateTimeout = d) arma un timer tras cada mensaje procesado; si no llega ningun otro mensaje en d se entrega StateTimeout como evento; cualquier mensaje (o timer con nombre) cancela el timer anterior e incrementa la generacion (un TimeoutMarker de generacion vieja se descarta). forMax(d) lo sustituye para el siguiente estado; forMax(Duration.Inf) lo deshabilita.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/FSM.scala:97-102,282-286,650,655,814-817,844-850,896-907 ; tests: actor-tests/src/test/scala/org/apache/pekko/actor/FSMTimingSpec.scala:33-73,113-127 ; actor-tests/src/test/scala/org/apache/pekko/actor/FSMActorSpec.scala:353-385
- **Condiciones**: forMax con duracion finita negativa no se usa: cae al timeout del estado (guard `d.length >= 0`).
- **Orden / concurrencia / ciclo de vida**: La generacion se incrementa en cada mensaje de usuario y en cada Timer valido; TimeoutMarker(gen) solo actua si gen == generation.
- **Dependencias**: AC-SCH-001 ; AC-FSM-001.1
- **Equivalente Rust propuesto**: `state_timeout` en `FsmState` + generacion `u64`; el timer usa el scheduler del sistema **Diferencias observables**: ninguna prevista.
- **Aserciones**:
  - A1: StateTimeout llega tras ~800 ms y transita a Initial (entre 500 ms y 2 s), sin mas mensajes despues - actor-tests/src/test/scala/org/apache/pekko/actor/FSMTimingSpec.scala:33-41.
  - A2: Un mensaje (Cancel) cancela el timeout: no llega StateTimeout posterior - actor-tests/src/test/scala/org/apache/pekko/actor/FSMTimingSpec.scala:43-51.
  - A3: El timeout se cancela al detenerse el actor: no hay dead letter por StateTimeout - actor-tests/src/test/scala/org/apache/pekko/actor/FSMTimingSpec.scala:53-60.
  - A4: goto(TestStateTimeout).forMax(Duration.Inf) deshabilita el timeout del estado (400 ms sin mensaje; solo Cancel provoca la transicion) - actor-tests/src/test/scala/org/apache/pekko/actor/FSMTimingSpec.scala:62-73.
  - A5: No se confunde un StateTimeout ya encolado con un timer con nombre posterior (la generacion lo descarta): Cancel recibido y transicion a Initial - actor-tests/src/test/scala/org/apache/pekko/actor/FSMTimingSpec.scala:113-127.
  - A6: forMax(Inf) en una FSM con stateTimeout 300 ms: no mas StateTimeout - actor-tests/src/test/scala/org/apache/pekko/actor/FSMActorSpec.scala:353-385.
  - A7: Tras cada makeTransition el timeout se re-arma (tambien con stay()): Some(d>=0) -> d; marcador Max -> ninguno; otro -> stateTimeouts(estado) - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/FSM.scala:896-907; *no afirmada por spec*.
- **Oraculo**: oraculo: pekko-actor 1.7.1 oficial via jshell permite ejecutar una FSM Scala/Java (AbstractFSM) con mensajes y comparar secuencias de Transition/respuestas; tiempos solo con tolerancia.
- **Decisiones**: Q-FSM-2, Q-X-4
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-FSM-004 - Timers con nombre (startSingleTimer, startTimerWithFixedDelay/AtFixedRate, setTimer deprecado, cancelTimer, isTimerActive)

- **Contrato observable**: [contractual por spec] Cada timer tiene nombre; re-crear uno con el mismo nombre cancela el anterior; los repetitivos siguen hasta cancelTimer; un timer cancelado cuyo mensaje ya estaba en el mailbox se descarta (comparacion de generacion). Un timer valido cancela el state timeout y se entrega como Event(msg, stateData). setTimer (deprecado) = startSingleTimer o AtFixedRate segun repeat.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/FSM.scala:104-160,553-645 ; tests: actor-tests/src/test/scala/org/apache/pekko/actor/FSMTimingSpec.scala:75-138 ; actor-tests/src/test/scala/org/apache/pekko/actor/FSMActorSpec.scala:223-260
- **Condiciones**: Debug de timers: "setting [repeating ]timer '<name>'/<d>: <msg>" y "canceling timer '<name>'" si debugEvent (F:616-617,631-632).
- **Orden / concurrencia / ciclo de vida**: La cancelacion y la generacion garantizan "at most once tras cancelar"; los timers son de la FSM (owner).
- **Dependencias**: AC-SCH-011 ; AC-FSM-003
- **Equivalente Rust propuesto**: `start_single_timer(name, msg, delay)` etc. sobre scheduler; `HashMap<String, TimerHandle>` con generacion **Diferencias observables**: ninguna prevista.
- **Aserciones**:
  - A1: Timer unico: llega Tick a ~500 ms y luego la FSM transita - actor-tests/src/test/scala/org/apache/pekko/actor/FSMTimingSpec.scala:75-84.
  - A2: Re-crear "blah" desde un handler: primer Tick luego Tock - actor-tests/src/test/scala/org/apache/pekko/actor/FSMTimingSpec.scala:86-98.
  - A3: cancelTimer("hallo") con Tock ya en el mailbox: nunca se entrega (se recibe Tick y luego un solo Tock del timer re-creado) - actor-tests/src/test/scala/org/apache/pekko/actor/FSMTimingSpec.scala:100-111.
  - A4: Timer repetitivo cada 100 ms con 5 Tick y cancelTimer - actor-tests/src/test/scala/org/apache/pekko/actor/FSMTimingSpec.scala:129-138.
  - A5: Al terminar la FSM se cancelan todos los timers y el state timer (isTimerActive false, isStateTimerActive false dentro de onTermination) - actor-tests/src/test/scala/org/apache/pekko/actor/FSMActorSpec.scala:223-260.
  - A6: Mensaje de timer que es AutoReceivedMessage se envia sin envolver; los demas como Timer(name,msg,mode,gen,owner) - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/FSM.scala:139-148; *no afirmada por spec*.
  - A7: Cuando un Timer expira solo se procesa si `owner eq this && timers contains name && generation coincide`; los de modo unico se eliminan del mapa - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/FSM.scala:818-829; *no afirmada por spec*.
- **Oraculo**: oraculo: pekko-actor 1.7.1 oficial via jshell permite ejecutar una FSM Scala/Java (AbstractFSM) con mensajes y comparar secuencias de Transition/respuestas; tiempos solo con tolerancia.
- **Decisiones**: Q-FSM-2, Q-X-4
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-FSM-005 - whenUnhandled y manejo por defecto de eventos no manejados (warning + stay)

- **Contrato observable**: [contractual por spec] Si la funcion del estado no define el evento se aplica whenUnhandled; el valor por defecto loguea warning "unhandled event <valor> in state <estado>" y hace stay(). whenUnhandled(f) reemplaza el manejador (con orElse sobre el defecto); whenUnhandled(NullFunction) lo restablece al defecto.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/FSM.scala:706-709,787-792 ; tests: actor-tests/src/test/scala/org/apache/pekko/actor/FSMTimingSpec.scala:140-158 ; actor-tests/src/test/scala/org/apache/pekko/actor/FSMActorSpec.scala:76-82,150-153
- **Condiciones**: El texto usa toString de valor y estado.
- **Orden / concurrencia / ciclo de vida**: n/a
- **Dependencias**: AC-FSM-001.2
- **Equivalente Rust propuesto**: `when_unhandled(fn)`; defecto `log::warn!("unhandled event {:?} in state {:?}")` **Diferencias observables**: El texto del warning usa `{:?}` (Debug) en lugar de toString de Scala.
- **Aserciones**:
  - A1: Sin handler: warnings "unhandled event Tick in state TestUnhandled" y "unhandled event Unhandled(test) in state TestUnhandled" (source = path de la FSM) - actor-tests/src/test/scala/org/apache/pekko/actor/FSMTimingSpec.scala:140-158.
  - A2: Tras SetHandler el Tick lo maneja el handler propio (Unhandled(Tick)); tras whenUnhandled(NullFunction) vuelve el defecto - actor-tests/src/test/scala/org/apache/pekko/actor/FSMTimingSpec.scala:276-287,140-158.
  - A3: Handler del Lock: `log.warning("unhandled event "+msg+" in state "+stateName+" with data "+stateData)` produce exactamente 1 warning "unhandled event..." - actor-tests/src/test/scala/org/apache/pekko/actor/FSMActorSpec.scala:76-82,150-153.
- **Oraculo**: oraculo: pekko-actor 1.7.1 oficial via jshell permite ejecutar una FSM Scala/Java (AbstractFSM) con mensajes y comparar secuencias de Transition/respuestas; tiempos solo con tolerancia.
- **Decisiones**: Q-FSM-3
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-FSM-006.1 - Terminacion: stop(), stop(reason), onTermination(StopEvent) y estado inexistente

- **Contrato observable**: [contractual por spec] stop(reason[, data]) envia las respuestas pendientes, ejecuta onTermination una vez (StopEvent(reason, estado, datos)) y detiene el actor (context.stop(self)); goto a un estado sin funcion registrada termina con Failure("Next state <s> does not exist") logueado como error.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/FSM.scala:518-528,869-884,928-941,947-951 ; tests: actor-tests/src/test/scala/org/apache/pekko/actor/FSMActorSpec.scala:170-221
- **Condiciones**: Failure(ex: Throwable) -> log.error(ex, "terminating due to Failure"); Failure(msg: AnyRef) -> log.error(msg.toString); Normal/Shutdown no loguean (F:947-951).
- **Orden / concurrencia / ciclo de vida**: Primero replies, luego terminate (cancelar timers, onTermination), luego context.stop(self) (F:873-877).
- **Dependencias**: AC-CELL-008 ; AC-FSM-001.2
- **Equivalente Rust propuesto**: `stop(reason)`, `on_termination(Fn(StopEvent))`; razon `StopReason { Normal, Shutdown, Failure(Msg) }` **Diferencias observables**: ninguna prevista.
- **Aserciones**:
  - A1: goto(2) sin when(2): error logueado "Next state 2 does not exist" con Logging.Error(_, path, _, mensaje) - actor-tests/src/test/scala/org/apache/pekko/actor/FSMActorSpec.scala:170-186.
  - A2: onTermination recibe StopEvent(Normal, 1, "pigdog") tras stop(Normal, "pigdog") - actor-tests/src/test/scala/org/apache/pekko/actor/FSMActorSpec.scala:208-221.
  - A3: terminate() corre solo si `currentState.stopReason.isEmpty` (una sola vez) - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/FSM.scala:928-941; *no afirmada por spec*.
  - A4: En makeTransition con estado inexistente se llama terminate(...) pero NO context.stop(self): el actor sigue vivo (la parada solo ocurre via applyState) - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/FSM.scala:882-884 vs 869-878; *no afirmada por spec* (Q-FSM-4).
- **Oraculo**: oraculo: pekko-actor 1.7.1 oficial via jshell permite ejecutar una FSM Scala/Java (AbstractFSM) con mensajes y comparar secuencias de Transition/respuestas; tiempos solo con tolerancia.
- **Decisiones**: Q-FSM-4, Q-X-5
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-FSM-006.2 - Terminacion externa: stop del actor -> StopEvent(Shutdown), timers cancelados en postStop

- **Contrato observable**: [contractual por spec] Al detenerse el actor desde fuera (system.stop) postStop ejecuta terminate con razon Shutdown: onTermination recibe StopEvent(Shutdown, estado, datos) y se cancelan todos los timers y el state timer.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/FSM.scala:74-95,917-926,928-941 ; tests: actor-tests/src/test/scala/org/apache/pekko/actor/FSMActorSpec.scala:188-206,223-260,95-99
- **Condiciones**: postStop -> terminate(stay().withStopReason(Shutdown)) y super.postStop().
- **Orden / concurrencia / ciclo de vida**: Corre en el hilo del actor al procesar Terminate.
- **Dependencias**: AC-CELL-008 ; AC-FSM-004
- **Equivalente Rust propuesto**: implementacion en el hook post_stop del trait Fsm **Diferencias observables**: ninguna prevista.
- **Aserciones**:
  - A1: system.stop(ref) entrega StopEvent(FSM.Shutdown, 1, null) a onTermination - actor-tests/src/test/scala/org/apache/pekko/actor/FSMActorSpec.scala:188-206.
  - A2: Dentro de onTermination ya no hay timers activos - actor-tests/src/test/scala/org/apache/pekko/actor/FSMActorSpec.scala:223-260.
  - A3: `bye` -> stop(FSM.Shutdown): onTermination (Shutdown, Locked, _) abre el latch - actor-tests/src/test/scala/org/apache/pekko/actor/FSMActorSpec.scala:65-66,95-99,119-168.
- **Oraculo**: oraculo: pekko-actor 1.7.1 oficial via jshell permite ejecutar una FSM Scala/Java (AbstractFSM) con mensajes y comparar secuencias de Transition/respuestas; tiempos solo con tolerancia.
- **Decisiones**: Q-FSM-4
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-FSM-007 - LoggingFSM: log rotativo (logDepth), getLog; debug.fsm

- **Contrato observable**: [contractual por spec] Con pekko.actor.debug.fsm = on loguea (debug) cada evento procesado y cada transicion; con logDepth > 0 guarda un buffer circular de entradas LogEntry(estado, datos, evento) accesible con getLog (mas antiguas primero).
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/FSM.scala:958-1021 ; actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:458 ; actor/src/main/resources/reference.conf:749 ; tests: actor-tests/src/test/scala/org/apache/pekko/actor/FSMActorSpec.scala:262-332
- **Condiciones**: logDepth = 0 por defecto: no hay buffer (arrays de tamano 0).
- **Orden / concurrencia / ciclo de vida**: advance() circular; `full` marca vuelta completa; getLog reordena (drop(pos) ++ take(pos)).
- **Dependencias**: AC-CFG-011 ; AC-FSM-004
- **Equivalente Rust propuesto**: `LoggingFsm` como extension del trait con `ring: Vec<Option<LogEntry>>` **Diferencias observables**: ninguna prevista.
- **Aserciones**:
  - A1: Con debug.fsm: se emiten exactamente 5 mensajes debug; "processing Event(go,null) from Actor[..." ; "setting timer 't'/1500 milliseconds: Shutdown" ; "transition 1 -> 2" ; "processing Event(stop,null) from Actor[..." ; "canceling timer 't'" - actor-tests/src/test/scala/org/apache/pekko/actor/FSMActorSpec.scala:262-312.
  - A2: logDepth = 3: getLog devuelve IndexedSeq(LogEntry(1,0,"log")); luego con count/log: (LogEntry(1,0,"log"), LogEntry(1,0,"count"), LogEntry(1,1,"log")); y rotando: (LogEntry(1,1,"log"), LogEntry(1,1,"count"), LogEntry(1,2,"log")) - actor-tests/src/test/scala/org/apache/pekko/actor/FSMActorSpec.scala:314-332.
  - A3: El formato del debug: "processing {} from {} in state {}" con {} de origen = "timer <name>" | ActorRef.toString | String | "unknown" - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/FSM.scala:981-990; *no afirmada por spec* (el spec solo verifica prefijo).
  - A4: getLog es protected; el buffer almacena el `stateName` previo al evento y el stateData del Event (anterior a la respuesta) - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/FSM.scala:992-996,1011-1019; *no afirmada por spec*.
- **Oraculo**: oraculo: pekko-actor 1.7.1 oficial via jshell permite ejecutar una FSM Scala/Java (AbstractFSM) con mensajes y comparar secuencias de Transition/respuestas; tiempos solo con tolerancia.
- **Decisiones**: Q-FSM-3
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-FSM-008 - transform(...) using

- **Contrato observable**: [contractual por spec] transform(f).using(g) devuelve una funcion de estado que aplica f y luego g al State resultante (g definida por parcial; el resto pasa igual).
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/FSM.scala:530-535 ; tests: actor-tests/src/test/scala/org/apache/pekko/actor/FSMActorSpec.scala:334-351
- **Condiciones**: n/a
- **Orden / concurrencia / ciclo de vida**: n/a
- **Dependencias**: AC-FSM-001.2
- **Equivalente Rust propuesto**: combinador `transform(f).using(g)` sobre `StateFn` **Diferencias observables**: ninguna prevista.
- **Aserciones**:
  - A1: when(0)(transform { case Event("go",_) => stay() }.using { case _ => goto(1) }) hace que "go" produzca Transition(0,1) (CurrentState(0) previo) - actor-tests/src/test/scala/org/apache/pekko/actor/FSMActorSpec.scala:334-351.
  - A2: func.andThen(andThen.orElse { case x => x }) (identidad si no definida) - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/FSM.scala:531-533; *no afirmada por spec*.
- **Oraculo**: oraculo: pekko-actor 1.7.1 oficial via jshell permite ejecutar una FSM Scala/Java (AbstractFSM) con mensajes y comparar secuencias de Transition/respuestas; tiempos solo con tolerancia.
- **Decisiones**: Q-FSM-1
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-FSM-009 - Acceso a stateName/stateData/nextStateData e initialize antes de startWith (IllegalStateException)

- **Contrato observable**: [contractual por spec] stateName/stateData lanzan IllegalStateException("You must call `startWith` before using `stateName`"/"`stateData`") si no se llamo startWith; initialize() sin startWith lanza IllegalStateException("You must call `startWith` before calling `initialize`"); nextStateData fuera de onTransition lanza IllegalStateException("nextStateData is only available during onTransition").
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/FSM.scala:719-744 ; tests: actor-tests/src/test/scala/org/apache/pekko/actor/FSMTransitionSpec.scala:156-179
- **Condiciones**: n/a
- **Orden / concurrencia / ciclo de vida**: n/a
- **Dependencias**: AC-FSM-002
- **Equivalente Rust propuesto**: metodos del trait Fsm que devuelven Result / panic documentado segun Q-X-5 **Diferencias observables**: ninguna prevista.
- **Aserciones**:
  - A1: nextStateData fuera de onTransition -> IllegalStateException (el test lo captura en "failed:" vs "ok") - actor-tests/src/test/scala/org/apache/pekko/actor/FSMTransitionSpec.scala:156-179.
  - A2: Mensajes exactos de las tres excepciones - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/FSM.scala:719-744; *no afirmada por spec* (el spec solo el tipo).
- **Oraculo**: oraculo: pekko-actor 1.7.1 oficial via jshell permite ejecutar una FSM Scala/Java (AbstractFSM) con mensajes y comparar secuencias de Transition/respuestas; tiempos solo con tolerancia.
- **Decisiones**: Q-FSM-1, Q-X-5
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-RT-001 - Router (valor inmutable), RoutingLogic, Routee (ActorRefRoutee, ActorSelectionRoutee, NoRoutee, SeveralRoutees), Broadcast y RouterEnvelope

- **Contrato observable**: [contractual por spec] Router(logic, routees).route(msg, sender): Broadcast(m) se envia a todos los routees (SeveralRoutees) con m desenvuelto; otro mensaje elige un routee con logic.select; NoRoutee (sin routees) envia el mensaje desenvuelto a deadLetters (si el sender es InternalActorRef); cualquier RouterEnvelope se desenvuelve antes de entregar. addRoutee/removeRoutee/withRoutees devuelven un Router nuevo. Broadcast(null) lanza InvalidMessageException("[null] is not an allowed message").
- **Fuente**: actor/src/main/scala/org/apache/pekko/routing/Router.scala:33-204 ; tests: actor-tests/src/test/scala/org/apache/pekko/routing/RoundRobinSpec.scala:163-210 ; actor-tests/src/test/scala/org/apache/pekko/routing/ScatterGatherFirstCompletedSpec.scala:62-90 ; actor-tests/src/test/scala/org/apache/pekko/routing/BroadcastSpec.scala:33-95
- **Condiciones**: NoRoutee.send es un no-op (Router.scala:76-78). ActorSelectionRoutee.send = selection.tell.
- **Orden / concurrencia / ciclo de vida**: Todo es sincrono en el hilo del llamador; el Router es inmutable (copy-on-write); la logica puede tener estado interno (AtomicLong, etc.).
- **Dependencias**: AC-REF-002 ; AC-ASK-001.1
- **Equivalente Rust propuesto**: `struct Router { logic: Arc<dyn RoutingLogic>, routees: Vec<Routee> }`, `enum Routee { Ref(ActorRef), Selection(ActorSelection), None, Several(Vec<Routee>) }`, `struct Broadcast<M>(M)` y trait `RouterEnvelope` **Diferencias observables**: Mensajes como `Box<dyn Any + Send>` (Q-X-3); RouterEnvelope debe poder ser extendido por el usuario (ConsistentHashableEnvelope).
- **Aserciones**:
  - A1: Router(RoundRobinRoutingLogic()) usado dentro de un actor reparte 10 iteraciones x 10 hijos uniformemente (cada hijo 10 hits) y `addRoutee`/`removeRoutee` producen routers nuevos - actor-tests/src/test/scala/org/apache/pekko/routing/RoundRobinSpec.scala:163-210.
  - A2: Broadcast("end") enviado al router llega a todos los routees (los 10 hijos paran y el actor termina al quedar sin routees) - actor-tests/src/test/scala/org/apache/pekko/routing/RoundRobinSpec.scala:204-208.
  - A3: Sin routees, `route` envia el mensaje (desenvuelto) a deadLetters: `NoRoutee` + sender InternalActorRef - derivada del codigo actor/src/main/scala/org/apache/pekko/routing/Router.scala:134-139; *no afirmada por spec*.
  - A4: Broadcast se desenvuelve: SeveralRoutees(routees).send(msg, sender) con msg interno - derivada del codigo actor/src/main/scala/org/apache/pekko/routing/Router.scala:128-131; afirmada indirectamente por actor-tests/src/test/scala/org/apache/pekko/routing/BroadcastSpec.scala:35-63.
  - A5: Broadcast(null) lanza InvalidMessageException "[null] is not an allowed message" - derivada del codigo actor/src/main/scala/org/apache/pekko/routing/Router.scala:193-196; *no afirmada por spec*.
  - A6: removeRoutee(routee) usa filterNot(_ == routee): igualdad por valor (ActorRefRoutee(ref)) - derivada del codigo actor/src/main/scala/org/apache/pekko/routing/Router.scala:171-183; *no afirmada por spec*.
- **Oraculo**: oraculo: pekko-actor 1.7.1 oficial via jshell permite crear routers y comparar los destinatarios elegidos de forma determinista (round robin, broadcast, consistent hashing con claves String/bytes); lo aleatorio/temporal solo estadisticamente.
- **Decisiones**: Q-RT-1, Q-X-3
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-RT-002 - RouterConfig / Pool / Group / CustomRouterConfig; estrategia por defecto del Pool (escala siempre), routerDispatcher, withFallback

- **Contrato observable**: [contractual por spec] RouterConfig define createRouter, routerDispatcher, mensajes de gestion y verifyConfig. Pool crea nrOfInstances hijos (usePoolDispatcher opcional con pool-dispatcher) y su supervisorStrategy por defecto es OneForOne(Escalate) para todo; Group enruta a paths via ActorSelection. withFallback (overrideUnsetConfig) hereda supervisorStrategy y resizer del otro si el propio es el valor por defecto/vacio.
- **Fuente**: actor/src/main/scala/org/apache/pekko/routing/RouterConfig.scala:54-280 ; tests: actor-tests/src/test/scala/org/apache/pekko/routing/RoutingSpec.scala:147-204 ; actor-tests/src/test/scala/org/apache/pekko/routing/ConfiguredLocalRoutingSpec.scala:139-175
- **Condiciones**: isManagementMessage: AutoReceivedMessage | Terminated | RouterManagementMesssage (RouterConfig.scala:82-85).
- **Orden / concurrencia / ciclo de vida**: n/a
- **Dependencias**: AC-RT-001 ; AC-SUP-001
- **Equivalente Rust propuesto**: trait `RouterConfig`, `trait Pool: RouterConfig`, `trait Group: RouterConfig`, `fn with_fallback(self, other) -> Self` **Diferencias observables**: ninguna prevista.
- **Aserciones**:
  - A1: RoundRobinPool(1, supervisorStrategy = escalator): al matar al routee el supervisor ve ActorKilledException - actor-tests/src/test/scala/org/apache/pekko/routing/RoutingSpec.scala:147-161; igual con .withSupervisorStrategy - :163-170.
  - A2: FromConfig.withSupervisorStrategy(escalator) tambien aplica - actor-tests/src/test/scala/org/apache/pekko/routing/RoutingSpec.scala:172-183.
  - A3: Estrategia por defecto: ante excepcion del routee se escala al padre, que reinicia: los 3 routees reciben postRestart ("restarted" x3) - actor-tests/src/test/scala/org/apache/pekko/routing/RoutingSpec.scala:185-204.
  - A4: RouterConfig se recoge de Props (RoundRobinPool(12)) o config (RandomPool(4, usePoolDispatcher = true)) o routees.paths (RandomGroup) - actor-tests/src/test/scala/org/apache/pekko/routing/ConfiguredLocalRoutingSpec.scala:141-157.
  - A5: overrideUnsetConfig: si otra es NoRouter se queda con la propia; si es Pool y la propia usa la estrategia por defecto toma la del otro; igual con resizer vacio - derivada del codigo actor/src/main/scala/org/apache/pekko/routing/RouterConfig.scala:116-141; *no afirmada por spec*.
  - A6: Pool.stopRouterWhenAllRouteesRemoved = resizer.isEmpty; routingLogicController por defecto None - derivada del codigo actor/src/main/scala/org/apache/pekko/routing/RouterConfig.scala:76,91,251; *no afirmada por spec*.
  - A7: Pool.newRoutee crea el hijo con context.actorOf y, con usePoolDispatcher, usa el dispatcher "pekko.actor.deployment.<path>.pool-dispatcher" - derivada del codigo actor/src/main/scala/org/apache/pekko/routing/RouterConfig.scala:213-225; *no afirmada por spec*.
- **Oraculo**: oraculo: pekko-actor 1.7.1 oficial via jshell permite crear routers y comparar los destinatarios elegidos de forma determinista (round robin, broadcast, consistent hashing con claves String/bytes); lo aleatorio/temporal solo estadisticamente.
- **Decisiones**: Q-RT-1, Q-X-5
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-RT-003.1 - Actor router (RoutedActorCell/RoutedActorRef): ruteo en el hilo del emisor, creacion de routees en start y mensajes de gestion

- **Contrato observable**: [contractual por spec] Un router pool/group es un actor cuyo mailbox solo recibe mensajes de gestion; el resto se rutea sincronamente en el hilo del emisor (RoutedActorCell.sendMessage -> router.route). Los routees de un pool se crean en start() antes del primer mensaje y los routees pueden enviar mensajes al router padre durante su construccion.
- **Fuente**: actor/src/main/scala/org/apache/pekko/routing/RoutedActorCell.scala:50-154 ; actor/src/main/scala/org/apache/pekko/routing/RoutedActorRef.scala:38-65 ; tests: actor-tests/src/test/scala/org/apache/pekko/routing/RouteeCreationSpec.scala:29-59 ; actor-tests/src/test/scala/org/apache/pekko/routing/RoutingSpec.scala:206-216
- **Condiciones**: RouterActor lanza ActorInitializationException("Router actor can only be used in RoutedActorRef, not in <clase>") si el contexto no es RoutedActorCell (RoutedActorCell.scala:160-164).
- **Orden / concurrencia / ciclo de vida**: El estado del router (_router) es @volatile y solo se actualiza desde el actor del router (add/remove) pero se lee desde cualquier hilo emisor.
- **Dependencias**: AC-CELL-001 ; AC-RT-002
- **Equivalente Rust propuesto**: `RoutedActorCell`: `ArcSwap<Router>` + enrutamiento en `tell` sin pasar por mailbox **Diferencias observables**: El ruteo en el hilo del emisor requiere un puntero atomico al Router (copia inmutable) y una ruta de envio que se salte el mailbox del router.
- **Aserciones**:
  - A1: 100 routees creados por un RoundRobinPool son visibles via actorSelection(self.path) durante su constructor - actor-tests/src/test/scala/org/apache/pekko/routing/RouteeCreationSpec.scala:29-42.
  - A2: Un routee puede enviar a context.parent durante su construccion y el router rutea ese mensaje a un routee: N "two" - actor-tests/src/test/scala/org/apache/pekko/routing/RouteeCreationSpec.scala:44-59.
  - A3: context.actorOf(RoundRobinPool(2).props(..)) dentro de un actor arranca en linea y responde - actor-tests/src/test/scala/org/apache/pekko/routing/RoutingSpec.scala:206-216.
  - A4: sendMessage: si es mensaje de gestion va al mailbox del router; si no router.route(envelope.message, envelope.sender) - derivada del codigo actor/src/main/scala/org/apache/pekko/routing/RoutedActorCell.scala:147-152; *no afirmada por spec*.
  - A5: start(): _router = createRouter; para Pool nrOfInstances > 0 -> addRoutees(newRoutee x n); para Group paths no vacios -> routeeFor(path); luego preSuperStart y super.start - derivada del codigo actor/src/main/scala/org/apache/pekko/routing/RoutedActorCell.scala:116-132; *no afirmada por spec*.
- **Oraculo**: no aplica: comportamiento de concurrencia/ciclo de vida; se valida con specs portados.
- **Decisiones**: Q-RT-2, Q-X-4
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-RT-003.2 - Mensajes de gestion del router: GetRoutees, AddRoutee, RemoveRoutee, AdjustPoolSize (+ Routees)

- **Contrato observable**: [contractual por spec] GetRoutees responde Routees(routees); AddRoutee agrega (y vigila) un routee; RemoveRoutee lo quita y detiene al hijo; AdjustPoolSize(+n) crea n routees, (-n) elimina los ultimos n y detiene a esos hijos (solo pools).
- **Fuente**: actor/src/main/scala/org/apache/pekko/routing/RoutedActorCell.scala:65-114,159-218 ; actor/src/main/scala/org/apache/pekko/routing/RouterConfig.scala:391-455 ; tests: actor-tests/src/test/scala/org/apache/pekko/routing/RoundRobinSpec.scala:111-127 ; actor-tests/src/test/scala/org/apache/pekko/routing/RoutingSpec.scala:112-126
- **Condiciones**: AddRoutee/RemoveRoutee con ActorRefRoutee hacen watch/unwatch; otros Routee no.
- **Orden / concurrencia / ciclo de vida**: Se procesan en el mailbox del router (son mensajes de gestion).
- **Dependencias**: AC-RT-003.1
- **Equivalente Rust propuesto**: enum `RouterManagement { GetRoutees, AddRoutee(Routee), RemoveRoutee(Routee), AdjustPoolSize(i32) }` **Diferencias observables**: ninguna prevista.
- **Aserciones**:
  - A1: RoundRobinPool(3): GetRoutees=3; AdjustPoolSize(+4) -> 7; AdjustPoolSize(-2) -> 5 - actor-tests/src/test/scala/org/apache/pekko/routing/RoundRobinSpec.scala:111-120.
  - A2: AddRoutee(ActorSelectionRoutee(...)) -> 6; RemoveRoutee(other) -> 5 - actor-tests/src/test/scala/org/apache/pekko/routing/RoundRobinSpec.scala:122-126.
  - A3: FromConfig con nr-of-instances=3 -> GetRoutees.size == 3; con RoundRobinPool(2) y deployment 3 -> tambien 3 (la config manda) - actor-tests/src/test/scala/org/apache/pekko/routing/RoutingSpec.scala:112-126.
  - A4: removeRoutees(..., stopChild = true) detiene al hijo con PoisonPill diferido 100 ms - derivada del codigo actor/src/main/scala/org/apache/pekko/routing/RoutedActorCell.scala:103-114; *no afirmada por spec*.
  - A5: AdjustPoolSize negativo elimina `currentRoutees.drop(len + change)` (los ultimos) - derivada del codigo actor/src/main/scala/org/apache/pekko/routing/RoutedActorCell.scala:211-215; *no afirmada por spec*.
- **Oraculo**: oraculo: viable (jshell): GetRoutees/AdjustPoolSize observables en tamano.
- **Decisiones**: Q-RT-2
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-RT-003.3 - Routees terminados: evictarlos y detener el router al quedar vacio (salvo con resizer)

- **Contrato observable**: [contractual por spec] Un routee terminado se elimina del router (Terminated); un pool sin resizer se detiene cuando no le quedan routees; con resizer no se detiene. PoisonPill al router (mensaje auto-recibido, gestion) lo detiene y detiene a sus hijos.
- **Fuente**: actor/src/main/scala/org/apache/pekko/routing/RoutedActorCell.scala:170-192 ; actor/src/main/scala/org/apache/pekko/routing/RouterConfig.scala:251 ; tests: actor-tests/src/test/scala/org/apache/pekko/routing/RoutingSpec.scala:66-110
- **Condiciones**: Terminated(child) -> removeRoutee(ActorRefRoutee(child), stopChild = false).
- **Orden / concurrencia / ciclo de vida**: Evicion asincrona: el siguiente mensaje tras el Terminated ya no llega al muerto (awaitCond en el test).
- **Dependencias**: AC-RT-003.1 ; AC-DW-001
- **Equivalente Rust propuesto**: RouterActor recibe Terminated(ref) -> remove_routee y `stop_if_all_removed` **Diferencias observables**: ninguna prevista.
- **Aserciones**:
  - A1: Tras detener c2 el router sigue repartiendo solo a c1; al detener c1 el router termina (existenceConfirmed) - actor-tests/src/test/scala/org/apache/pekko/routing/RoutingSpec.scala:68-88.
  - A2: Con resizer (RoundRobinPool(0, resizer = Some(..))) el router NO termina aunque los routees mueran (2 s sin Terminated) - actor-tests/src/test/scala/org/apache/pekko/routing/RoutingSpec.scala:90-110.
  - A3: stopIfAllRouteesRemoved: `routees.isEmpty && routerConfig.stopRouterWhenAllRouteesRemoved` -> context.stop(self) - derivada del codigo actor/src/main/scala/org/apache/pekko/routing/RoutedActorCell.scala:185-187; *no afirmada por spec*.
- **Oraculo**: no aplica: ciclo de vida/temporal.
- **Decisiones**: Q-RT-2
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-RT-004 - FromConfig / NoRouter y precedencia deployment vs codigo

- **Contrato observable**: [contractual por spec] FromConfig exige una entrada de deployment (si no ConfigurationException "Configuration missing for router [<path>] in 'pekko.actor.deployment' section."); la config manda sobre Props (nr-of-instances, router); NoRouter es el defecto neutro (rutea a un unico actor). Deploy.withFallback: el deployment de config pisa al de codigo.
- **Fuente**: actor/src/main/scala/org/apache/pekko/routing/RouterConfig.scala:283-384 ; actor/src/main/scala/org/apache/pekko/actor/Deployer.scala:233-290 ; tests: actor-tests/src/test/scala/org/apache/pekko/routing/ConfiguredLocalRoutingSpec.scala:141-211 ; actor-tests/src/test/scala/org/apache/pekko/routing/RoutingSpec.scala:112-126,222-257
- **Condiciones**: FromConfig.verifyConfig lanza ConfigurationException; nrOfInstances de FromConfig = 0.
- **Orden / concurrencia / ciclo de vida**: n/a
- **Dependencias**: AC-CFG-012 ; AC-CFG-014 ; AC-RT-016
- **Equivalente Rust propuesto**: `FromConfig`, `NoRouter` como RouterConfig; `Deployer::lookup(path)` + `router.type-mapping` **Diferencias observables**: ninguna prevista.
- **Aserciones**:
  - A1: FromConfig.props(...) sin entrada ("routerNotDefined") lanza ConfigurationException cuyo mensaje incluye el nombre - actor-tests/src/test/scala/org/apache/pekko/routing/RoutingSpec.scala:239-244.
  - A2: FromConfig() sin nombre configurado lanza ConfigurationException - actor-tests/src/test/scala/org/apache/pekko/routing/ConfiguredLocalRoutingSpec.scala:175-179.
  - A3: Configuracion externa `router = round-robin-pool` en deployment funciona con FromConfig - actor-tests/src/test/scala/org/apache/pekko/routing/RoutingSpec.scala:246-257.
  - A4: Config pisa codigo: RoundRobinPool(12) en Props con deployment random-pool(4) -> RandomPool(4, usePoolDispatcher = true) - actor-tests/src/test/scala/org/apache/pekko/routing/ConfiguredLocalRoutingSpec.scala:147-151,167-173.
  - A5: Props con RoundRobinPool(12) sin deployment queda RoundRobinPool(12) - actor-tests/src/test/scala/org/apache/pekko/routing/ConfiguredLocalRoutingSpec.scala:141-145,159-165.
  - A6: El patron "/weird/*" (nr-of-instances=2) no se aplica a los routees de /weird (nr 3): se crean exactamente 3 routees con paths /user/weird/$a, $b, $c y no llegan mas refs en 1 s - actor-tests/src/test/scala/org/apache/pekko/routing/ConfiguredLocalRoutingSpec.scala:181-188.
  - A7: NoRouter.props(routeeProps) entrega mensajes al unico routee en orden ("hello","end") - actor-tests/src/test/scala/org/apache/pekko/routing/RoutingSpec.scala:222-235.
  - A8: FromConfig.createRouter/createRouterActor lanzan UnsupportedOperationException("FromConfig must not create Router"/"...RouterActor"); NoRouter igual ("NoRouter has no Router", "NoRouter must not create RouterActor", "NoRouter has no dispatcher") - derivada del codigo actor/src/main/scala/org/apache/pekko/routing/RouterConfig.scala:315-325,366-375; *no afirmada por spec*.
- **Oraculo**: oraculo: pekko-actor 1.7.1 oficial via jshell permite crear routers y comparar los destinatarios elegidos de forma determinista (round robin, broadcast, consistent hashing con claves String/bytes); lo aleatorio/temporal solo estadisticamente.
- **Decisiones**: Q-RT-3, Q-X-1
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-RT-005 - RoundRobin (logic, pool, group)

- **Contrato observable**: [contractual por spec] Reparte cada mensaje al siguiente routee en orden ciclico con un contador atomico compartido por el router (indice = contador % tamano, sin routees -> NoRoutee). Pool y Group (por paths) con la misma logica.
- **Fuente**: actor/src/main/scala/org/apache/pekko/routing/RoundRobin.scala:29-173 ; tests: actor-tests/src/test/scala/org/apache/pekko/routing/RoundRobinSpec.scala:34-210
- **Condiciones**: n/a
- **Orden / concurrencia / ciclo de vida**: Un solo AtomicLong por logica (compartido entre hilos emisores): la distribucion es estricta en secuencia de llamadas.
- **Dependencias**: AC-RT-003.1
- **Equivalente Rust propuesto**: `RoundRobinRoutingLogic { next: AtomicU64 }` + `RoundRobinPool`/`RoundRobinGroup` **Diferencias observables**: Con contador u64 el desbordamiento no ocurre en la practica; el comportamiento negativo de Scala (Long) no es observable.
- **Aserciones**:
  - A1: Pool: un pool de 5 se detiene (stopLatch) con system.stop - actor-tests/src/test/scala/org/apache/pekko/routing/RoundRobinSpec.scala:36-59.
  - A2: Pool: 10 routees, 100 mensajes (iterationCount x connectionCount) -> cada routee recibe exactamente 10 y se crean 10 instancias - actor-tests/src/test/scala/org/apache/pekko/routing/RoundRobinSpec.scala:61-88.
  - A3: Broadcast("hello") llega a los 5 routees - actor-tests/src/test/scala/org/apache/pekko/routing/RoundRobinSpec.scala:90-109.
  - A4: Group: 10 destinos por path, 100 mensajes -> 10 c/u - actor-tests/src/test/scala/org/apache/pekko/routing/RoundRobinSpec.scala:130-160.
  - A5: select: `routees(if (index < 0) size + index else index)` con index = next.getAndIncrement % size; sin routees -> NoRoutee - derivada del codigo actor/src/main/scala/org/apache/pekko/routing/RoundRobin.scala:42-47; *no afirmada por spec* (negativo por desbordamiento).
- **Oraculo**: oraculo: pekko-actor 1.7.1 oficial via jshell permite crear routers y comparar los destinatarios elegidos de forma determinista (round robin, broadcast, consistent hashing con claves String/bytes); lo aleatorio/temporal solo estadisticamente.
- **Decisiones**: Q-RT-1
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-RT-006 - Random (logic, pool, group)

- **Contrato observable**: [contractual por spec] Elige un routee al azar con ThreadLocalRandom (nextInt(size)); sin routees -> NoRoutee.
- **Fuente**: actor/src/main/scala/org/apache/pekko/routing/Random.scala:29-164 ; tests: actor-tests/src/test/scala/org/apache/pekko/routing/RandomSpec.scala:28-112
- **Condiciones**: n/a
- **Orden / concurrencia / ciclo de vida**: Aleatorio, sin garantias.
- **Dependencias**: AC-RT-003.1
- **Equivalente Rust propuesto**: `RandomRoutingLogic` con `rand::thread_rng()` (rng inyectable para tests) **Diferencias observables**: La secuencia aleatoria no es reproducible con Pekko; la prueba es estadistica.
- **Aserciones**:
  - A1: Pool de 7: 5 mensajes "hello" responden "world" y system.stop detiene a los 7 - actor-tests/src/test/scala/org/apache/pekko/routing/RandomSpec.scala:30-55.
  - A2: Pool de 10, 1000 mensajes: cada routee recibe > 0 y la suma es 1000; se crean 10 instancias - actor-tests/src/test/scala/org/apache/pekko/routing/RandomSpec.scala:57-90.
  - A3: Broadcast("hello") llega a los 6 routees - actor-tests/src/test/scala/org/apache/pekko/routing/RandomSpec.scala:92-111.
- **Oraculo**: no aplica: aleatorio, no determinista (comparable solo estadisticamente).
- **Decisiones**: Q-RT-1
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-RT-007 - Broadcast (pool, group)

- **Contrato observable**: [contractual por spec] BroadcastRoutingLogic.select devuelve SeveralRoutees(todos los routees): cada mensaje se envia a todos, con ask devolviendo respuestas a la ref temporal (varias respuestas, solo la primera completa el ask).
- **Fuente**: actor/src/main/scala/org/apache/pekko/routing/Broadcast.scala:27-162 ; tests: actor-tests/src/test/scala/org/apache/pekko/routing/BroadcastSpec.scala:33-95
- **Condiciones**: n/a
- **Orden / concurrencia / ciclo de vida**: Entrega a todos en el orden de routees.
- **Dependencias**: AC-RT-001
- **Equivalente Rust propuesto**: `BroadcastRoutingLogic`; pool y group **Diferencias observables**: ninguna prevista.
- **Aserciones**:
  - A1: BroadcastGroup(paths): `routedActor ! 1` suma 1 en ambos destinos - actor-tests/src/test/scala/org/apache/pekko/routing/BroadcastSpec.scala:35-63.
  - A2: Con `?` los dos reciben el mensaje (solo uno responde "ack") - actor-tests/src/test/scala/org/apache/pekko/routing/BroadcastSpec.scala:65-95.
- **Oraculo**: oraculo: pekko-actor 1.7.1 oficial via jshell permite crear routers y comparar los destinatarios elegidos de forma determinista (round robin, broadcast, consistent hashing con claves String/bytes); lo aleatorio/temporal solo estadisticamente.
- **Decisiones**: Q-RT-1
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-RT-008 - SmallestMailbox (pool)

- **Contrato observable**: [contractual por spec] Elige el routee no suspendido con menos mensajes: score 0 (ocioso y sin mensajes) gana al instante; luego vacios, luego menor mailbox (segunda pasada profunda); desconocidos/suspendidos/no locales tienen prioridad menor; si el elegido esta terminado elige al azar. Requiere introspeccion del mailbox.
- **Fuente**: actor/src/main/scala/org/apache/pekko/routing/SmallestMailbox.scala:31-244 ; tests: actor-tests/src/test/scala/org/apache/pekko/routing/SmallestMailboxSpec.scala:26-75
- **Condiciones**: Sin routees -> NoRoutee.
- **Orden / concurrencia / ciclo de vida**: Lecturas de estado del mailbox son carreras inevitables (comentario del codigo "Race between hasMessages and numberOfMessages").
- **Dependencias**: AC-MBX-019 ; AC-RT-003.1
- **Equivalente Rust propuesto**: `SmallestMailboxRoutingLogic` que usa `ActorCellView { is_processing, has_messages, number_of_messages, is_suspended }` **Diferencias observables**: ninguna prevista.
- **Aserciones**:
  - A1: Con un routee ocupado (esperando un latch) los siguientes tres mensajes van a routees distintos al ocupado - actor-tests/src/test/scala/org/apache/pekko/routing/SmallestMailboxSpec.scala:28-74.
  - A2: selectNext: score = (procesando ? 1 : 0) + (sin mensajes ? 0 : (profundo ? n : MaxValue-3)); suspendido = MaxValue-1; gana el menor; si se elige uno terminado -> routee al azar - derivada del codigo actor/src/main/scala/org/apache/pekko/routing/SmallestMailbox.scala:64-93; *no afirmada por spec*.
  - A3: Routees no locales/ActorSelection: no se consideran en mailbox (score por defecto) - derivada del codigo actor/src/main/scala/org/apache/pekko/routing/SmallestMailbox.scala:108-153; *no afirmada por spec*.
- **Oraculo**: no aplica: depende de timing/estado del mailbox.
- **Decisiones**: Q-RT-4
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-RT-009.1 - ConsistentHashing: seleccion por clave (hashMapping, ConsistentHashable, ConsistentHashableEnvelope), virtual-nodes-factor

- **Contrato observable**: [contractual por spec] La clave se obtiene de hashMapping (funcion parcial del pool) o de ConsistentHashable.consistentHashKey (ConsistentHashableEnvelope lo implementa); la misma clave va al mismo routee mientras no cambien los routees; sin clave se loguea warning y NoRoutee; la clave String/Array[Byte] se hashea directo, otras se serializan. virtual-nodes-factor viene de config (default.virtual-nodes-factor = 10) si el valor es 0.
- **Fuente**: actor/src/main/scala/org/apache/pekko/routing/ConsistentHashing.scala:36-260,443-457 ; actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:476-486 (DefaultVirtualNodesFactor) ; actor/src/main/resources/reference.conf:218-221 ; tests: actor-tests/src/test/scala/org/apache/pekko/routing/ConsistentHashingRouterSpec.scala:76-121
- **Condiciones**: Si virtualNodesFactor == 0 usa system.settings.DefaultVirtualNodesFactor. Nodo hasheado = ConsistentRoutee.toString (path con direccion completa; Selection: anchorPath + pathString).
- **Orden / concurrencia / ciclo de vida**: Lectura sin locks con AtomicReference.
- **Dependencias**: AC-RT-003.1 ; AC-SER-001
- **Equivalente Rust propuesto**: `ConsistentHashingRoutingLogic` con clave `HashKey { Bytes(Vec<u8>), Str(String), Other(...)}` y trait `ConsistentHashable` **Diferencias observables**: Para claves que no son String/bytes, Pekko usa SerializationExtension (Java serialization en el test): las posiciones de anillo difieren si no se replica; ver Q-RT-5.
- **Aserciones**:
  - A1: ConsistentHashingPool desde config con virtual-nodes-factor = 17: 3 routees - actor-tests/src/test/scala/org/apache/pekko/routing/ConsistentHashingRouterSpec.scala:76-79.
  - A2: Msg("a") y ConsistentHashableEnvelope("AA", hashKey "a") van al mismo destino; idem con claves 17 (Int) y MsgKey("c") (case class serializada) - actor-tests/src/test/scala/org/apache/pekko/routing/ConsistentHashingRouterSpec.scala:81-96.
  - A3: Con hashMapping propio ({case Msg2(key,_) => key}) los Msg2 y los envelopes con la misma clave van al mismo destino - actor-tests/src/test/scala/org/apache/pekko/routing/ConsistentHashingRouterSpec.scala:98-121.
  - A4: Mensaje sin clave: warning "Message [<clase>] must be handled by hashMapping, or implement [..ConsistentHashable] or be wrapped in [..ConsistentHashableEnvelope]" y NoRoutee - derivada del codigo actor/src/main/scala/org/apache/pekko/routing/ConsistentHashing.scala (select, bloque `case _ =>`); *no afirmada por spec*.
  - A5: Cualquier fallo al calcular (serializacion) -> warning "Couldn't route message with consistent hash key [{}] due to [{}]" y NoRoutee - derivada del codigo (idem); *no afirmada por spec*.
  - A6: El ConsistentHash se recalcula solo si cambia la identidad/contenido de routees (consistentHashRef CAS) - derivada del codigo (idem); *no afirmada por spec*.
- **Oraculo**: oraculo: viable con claves String/bytes: comparar el routee elegido y las posiciones del anillo contra pekko-actor oficial.
- **Decisiones**: Q-RT-5, Q-X-3
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-RT-009.2 - ConsistentHash (anillo con nodos virtuales)

- **Contrato observable**: [contractual por spec] ConsistentHash[T](nodes, virtualNodesFactor>=1): para cada nodo, hash(nodo.toString) y virtualNodesFactor posiciones = finalizeHash(extendHash(startHash(nodeHash), vnode, magicA, magicB)); nodeFor(key) busca por busqueda binaria la primera posicion >= hash(key) con vuelta al principio; :+ y :- agregan/quitan.
- **Fuente**: actor/src/main/scala/org/apache/pekko/routing/ConsistentHash.scala:30-152 ; tests: - (sin spec dedicado; cubierto indirectamente por actor-tests/src/test/scala/org/apache/pekko/routing/ConsistentHashingRouterSpec.scala:81-121)
- **Condiciones**: Orden de enteros con signo (Int de 32 bits) en el anillo.
- **Orden / concurrencia / ciclo de vida**: Estructura inmutable.
- **Dependencias**: AC-RT-009.3
- **Equivalente Rust propuesto**: `ConsistentHash<T>` con `BTreeMap<i32, T>` y `nodeFor(&[u8]|&str)` **Diferencias observables**: Debe usar enteros i32 con complemento a dos para que el orden del anillo coincida.
- **Aserciones**:
  - A1: virtualNodesFactor < 1 lanza IllegalArgumentException("virtualNodesFactor must be >= 1") - derivada del codigo actor/src/main/scala/org/apache/pekko/routing/ConsistentHash.scala:34; *no afirmada por spec*.
  - A2: nodeFor en anillo vacio lanza IllegalStateException("Can't get node for [%s] from an empty node ring") - derivada del codigo actor/src/main/scala/org/apache/pekko/routing/ConsistentHash.scala:98-113; *no afirmada por spec*.
  - A3: idx(i): exacto -> i; si no j = abs(i+1), si j >= largo -> 0 - derivada del codigo actor/src/main/scala/org/apache/pekko/routing/ConsistentHash.scala:84-91; *no afirmada por spec*.
  - A4: Las posiciones colisionadas se resuelven por SortedMap (ultimo insertado gana) - derivada del codigo actor/src/main/scala/org/apache/pekko/routing/ConsistentHash.scala:123-132; *no afirmada por spec*.
- **Oraculo**: oraculo: viable (jshell): `ConsistentHash.create(nodes, vnodes).nodeFor(key)` es API publica comparable con pekko-actor oficial.
- **Decisiones**: Q-RT-5
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-RT-009.3 - MurmurHash (variante Pekko): startHash/extendHash/finalizeHash, arrayHash, stringHash, symmetricHash

- **Contrato observable**: [contractual por spec] Funciones de hash MurmurHash3 especificas de Pekko con constantes fijas (visibleMagic 0x971E137B, hiddenMagicA 0x95543787, hiddenMagicB 0x2AD7EB25, mixers, seeds) usadas por ConsistentHash; stringHash toma pares de chars en 32 bits; arrayHash usa .## de cada elemento.
- **Fuente**: actor/src/main/scala/org/apache/pekko/routing/MurmurHash.scala:50-165 ; tests: - (sin spec dedicado en actor-tests)
- **Condiciones**: Aritmetica de 32 bits con signo y desbordamiento.
- **Orden / concurrencia / ciclo de vida**: n/a
- **Dependencias**: -
- **Equivalente Rust propuesto**: modulo `murmur_hash` con las mismas constantes y aritmetica `wrapping` i32 **Diferencias observables**: stringHash de Rust debe trabajar sobre `encode_utf16`; los hashes son exactos o la distribucion difiere de Pekko.
- **Aserciones**:
  - A1: startHash(seed) = seed ^ visibleMagic; extendHash(h, v, a, b) = (h ^ rotl(v*a, 11) * b) * 3 + visibleMixer; finalizeHash aplica la mezcla fmix32 con 0x85EBCA6B/0xC2B2AE35 - derivada del codigo actor/src/main/scala/org/apache/pekko/routing/MurmurHash.scala:75-109; *no afirmada por spec*.
  - A2: stringHash(s): h0 = startHash(len * 0xF7CA7FD2); pares (c0<<16)+c1 con magics rotativos (nextMagic = magic*5 + mixer); caracter impar final con extendHash - derivada del codigo actor/src/main/scala/org/apache/pekko/routing/MurmurHash.scala:127-141; *no afirmada por spec*.
  - A3: arrayHash(bytes): h0 = startHash(len * 0x3C074A61), elemento = byte con signo (`.##` de Byte), magics rotativos - derivada del codigo actor/src/main/scala/org/apache/pekko/routing/MurmurHash.scala:112-124; *no afirmada por spec*.
  - A4: chars se interpretan como UTF-16 code units (Java String): hay que iterar unidades UTF-16, no bytes UTF-8 - derivada del codigo actor/src/main/scala/org/apache/pekko/routing/MurmurHash.scala:133-139; *no afirmada por spec*.
- **Oraculo**: oraculo: viable (jshell): `MurmurHash.stringHash/arrayHash` son `public`; comparar valores exactos con pekko-actor oficial.
- **Decisiones**: Q-RT-5
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-RT-010 - ScatterGatherFirstCompleted (pool, group, within)

- **Contrato observable**: [contractual por spec] Envia el mensaje a todos los routees con ask(timeout = within) y responde al sender con la primera respuesta (ok o fallo, via pipeTo -> Status.Failure en fallo); sin routees responde de inmediato con Status.Failure(TimeoutException("Timeout due to no routees")).
- **Fuente**: actor/src/main/scala/org/apache/pekko/routing/ScatterGatherFirstCompleted.scala:45-233 ; tests: actor-tests/src/test/scala/org/apache/pekko/routing/ScatterGatherFirstCompletedSpec.scala:60-116
- **Condiciones**: Routees que no son ActorRefRoutee/ActorSelectionRoutee se ignoran.
- **Orden / concurrencia / ciclo de vida**: Los asks se envian en secuencia a todos; gana la primera completion de la promesa.
- **Dependencias**: AC-ASK-001.1 ; AC-ASK-006
- **Equivalente Rust propuesto**: `ScatterGatherFirstCompletedRoutingLogic { within }`; pool/group **Diferencias observables**: El tipo de fallo al vencer `within` es el AskTimeoutException del primer ask (message de ask), no uno propio.
- **Aserciones**:
  - A1: Group con Broadcast(1) y Broadcast("end") suma 1 en ambos destinos - actor-tests/src/test/scala/org/apache/pekko/routing/ScatterGatherFirstCompletedSpec.scala:62-90.
  - A2: Si un actor se detuvo, la respuesta del otro llega (14) - actor-tests/src/test/scala/org/apache/pekko/routing/ScatterGatherFirstCompletedSpec.scala:92-101.
  - A3: Pool con nrOfInstances = 0: el sender recibe Status.Failure con causa de clase TimeoutException en <2 s - actor-tests/src/test/scala/org/apache/pekko/routing/ScatterGatherFirstCompletedSpec.scala:108-114.
  - A4: Texto de la excepcion para el caso sin routees: "Timeout due to no routees" - derivada del codigo actor/src/main/scala/org/apache/pekko/routing/ScatterGatherFirstCompleted.scala:62; *no afirmada por spec* (Scaladoc dice AskTimeoutException: Q-RT-6).
- **Oraculo**: oraculo: pekko-actor 1.7.1 oficial via jshell permite crear routers y comparar los destinatarios elegidos de forma determinista (round robin, broadcast, consistent hashing con claves String/bytes); lo aleatorio/temporal solo estadisticamente.
- **Decisiones**: Q-RT-6, Q-ASK-1
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-RT-011 - TailChopping (pool, group, interval, within)

- **Contrato observable**: [contractual por spec] Baraja los routees; envia el mensaje al primero (ask) y cada `interval` al siguiente hasta recibir una respuesta, agotar routees o vencer `within` (AskTimeoutException); responde al sender con la primera respuesta.
- **Fuente**: actor/src/main/scala/org/apache/pekko/routing/TailChopping.scala:60-308 ; tests: actor-tests/src/test/scala/org/apache/pekko/routing/TailChoppingSpec.scala:58-135
- **Condiciones**: Sin routees -> NoRoutee.
- **Orden / concurrencia / ciclo de vida**: scheduleWithFixedDelay(0, interval); idx por AtomicInteger; cancela timers al completar.
- **Dependencias**: AC-ASK-001.1 ; AC-SCH-001
- **Equivalente Rust propuesto**: `TailChoppingRoutingLogic { within, interval }` con scheduler **Diferencias observables**: El texto con defecto de formato se replica solo si el Owner lo decide (Q-RT-6).
- **Aserciones**:
  - A1: Group con Broadcast(1)/Broadcast("end") suma 1 en ambos - actor-tests/src/test/scala/org/apache/pekko/routing/TailChoppingSpec.scala:60-88.
  - A2: Con interval=50 ms y routee 1 lento (1 ms de sleep): se obtiene "ack" y entre ambos routees exactamente uno recibe 1 vez y el otro (1,1) - actor-tests/src/test/scala/org/apache/pekko/routing/TailChoppingSpec.scala:90-103.
  - A3: Si nadie responde en `within` (300 ms vs 500 ms de cada routee): Failure(AskTimeoutException) y ambos recibieron 1 mensaje - actor-tests/src/test/scala/org/apache/pekko/routing/TailChoppingSpec.scala:105-121.
  - A4: Responde lo antes posible: routee 1 (1 s) responde "ack" antes que el 2 (4 s) con interval 100 ms - actor-tests/src/test/scala/org/apache/pekko/routing/TailChoppingSpec.scala:123-135.
  - A5: Mensaje del AskTimeoutException del propio router: "Ask timed out on [<sender>] after [<within>.toMillis} ms]" (literal con la llave sobrante; el codigo interpola `$within.toMillis}`) - derivada del codigo actor/src/main/scala/org/apache/pekko/routing/TailChopping.scala:104-105; *no afirmada por spec* (Q-RT-6).
- **Oraculo**: no aplica: depende de timing; el texto del error es comparable.
- **Decisiones**: Q-RT-6
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-RT-012 - BalancingPool

- **Contrato observable**: [contractual por spec] Todos los routees comparten un mailbox (BalancingDispatcher registrado por pool con id "BalancingPool-<path normalizado>"); el reparto lo hace el dispatcher, el RoutingLogic siempre devuelve routees.head; funciona con nombres anonimos ($a) y codificados (%). No admite resizer.
- **Fuente**: actor/src/main/scala/org/apache/pekko/routing/Balancing.scala:43-178 ; tests: actor-tests/src/test/scala/org/apache/pekko/routing/BalancingSpec.scala:133-178 (primeros tests en `pending`) ; actor-tests/src/test/scala/org/apache/pekko/actor/dispatch/DispatchersSpec.scala:286
- **Condiciones**: withFallback hereda supervisorStrategy como un Pool normal.
- **Orden / concurrencia / ciclo de vida**: Dependiente del dispatcher; el spec upstream desactivado indica inestabilidad de timing.
- **Dependencias**: AC-DSP-012 ; AC-RT-003.1
- **Equivalente Rust propuesto**: `BalancingPool` + dispatcher de cola compartida (work-stealing/mailbox comun) **Diferencias observables**: Sin dispatcher dedicado en tokio, la cola compartida es un canal multiconsumidor propio.
- **Aserciones**:
  - A1: Los 3 tests de reparto (programatico, config, override) estan en `pending` en upstream (FIXME flaky akka/akka#30860): NO hay aserciones vigentes - actor-tests/src/test/scala/org/apache/pekko/routing/BalancingSpec.scala:133-165.
  - A2: Funciona con nombre anonimo: `invalidConfigKeyChars` ($ @ :) reemplazados por "_" en el dispatcher-id - actor-tests/src/test/scala/org/apache/pekko/routing/BalancingSpec.scala:167-171 ; codigo actor/src/main/scala/org/apache/pekko/routing/Balancing.scala:116-123,176-178.
  - A3: Con nombre codificado ("abc%C3%A5...") funciona (% es valido en claves de config) - actor-tests/src/test/scala/org/apache/pekko/routing/BalancingSpec.scala:173-178.
  - A4: Forma del reparto (derivada del test pending): 100 mensajes, un routee desbloqueado atiende todos salvo poolSize-1; los demas esperan - derivada de actor-tests/src/test/scala/org/apache/pekko/routing/BalancingSpec.scala:99-131; *no afirmada por spec vigente (pending)*.
  - A5: newRoutee registra BalancingDispatcherConfigurator si no existe, con config "pekko.actor.deployment.<path>.pool-dispatcher" - derivada del codigo actor/src/main/scala/org/apache/pekko/routing/Balancing.scala:116-143; *no afirmada por spec*.
- **Oraculo**: no aplica: comportamiento de concurrencia no determinista (spec upstream pending).
- **Decisiones**: Q-RT-7
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-RT-013.1 - DefaultResizer: capacidad (rampup/backoff), cotas y validaciones

- **Contrato observable**: [contractual por spec] capacity = delta ajustada a [lowerBound, upperBound]; delta = rampup + backoff: rampup = 0 si pressure < capacity, si no ceil(rampupRate * capacity); backoff = floor(-backoffRate * capacity) si backoffThreshold > 0 && backoffRate > 0 && capacity > 0 && pressure/capacity < backoffThreshold, si no 0. isTimeForResize(counter) = counter % messagesPerResize == 0.
- **Fuente**: actor/src/main/scala/org/apache/pekko/routing/Resizer.scala:139-259 ; actor/src/main/resources/reference.conf:246-290 ; tests: actor-tests/src/test/scala/org/apache/pekko/routing/ResizerSpec.scala:104-135
- **Condiciones**: Defaults: lowerBound=1, upperBound=10, pressureThreshold=1, rampupRate=0.2, backoffThreshold=0.3, backoffRate=0.1, messagesPerResize=10 (Resizer.scala:139-146).
- **Orden / concurrencia / ciclo de vida**: n/a: funciones puras.
- **Dependencias**: AC-RT-003.1
- **Equivalente Rust propuesto**: `DefaultResizer { lower_bound, upper_bound, pressure_threshold, rampup_rate, backoff_threshold, backoff_rate, messages_per_resize }` **Diferencias observables**: ninguna prevista.
- **Aserciones**:
  - A1: capacity(Vector.empty) con lowerBound=2 -> 2; con 4 routees ocupados a upperBound -> 0 - actor-tests/src/test/scala/org/apache/pekko/routing/ResizerSpec.scala:106-116.
  - A2: rampup(pressure=9, capacity=10) = 0; (5,5) = 1; (6,6) = 2 con rampupRate 0.2 - actor-tests/src/test/scala/org/apache/pekko/routing/ResizerSpec.scala:118-124.
  - A3: backoff con threshold 0.3 y rate 0.1: (10,10)=0, (4,10)=0, (3,10)=0, (2,10)=-1, (0,10)=-1, (1,9)=-1, (0,9)=-1 - actor-tests/src/test/scala/org/apache/pekko/routing/ResizerSpec.scala:126-135.
  - A4: Validaciones (IllegalArgumentException): "lowerBound must be >= 0, was: [%s]", "upperBound must be >= 0, was: [%s]", "upperBound must be >= lowerBound, was: [%s] < [%s]", "rampupRate must be >= 0.0, was [%s]", "backoffThreshold must be <= 1.0, was [%s]", "backoffRate must be >= 0.0, was [%s]", "messagesPerResize must be > 0, was [%s]" - derivada del codigo actor/src/main/scala/org/apache/pekko/routing/Resizer.scala:154-164; *no afirmada por spec*.
- **Oraculo**: oraculo: viable (jshell): DefaultResizer.rampup/backoff/capacity son publicas.
- **Decisiones**: Q-RT-4
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-RT-013.2 - DefaultResizer: pressure (routees ocupados) y mecanica de resize del pool (ResizablePoolCell)

- **Contrato observable**: [contractual por spec] pressure cuenta routees ocupados segun pressureThreshold: 1 -> mailbox planificado y con mensajes; <1 -> planificado y procesando mensaje; >1 -> numberOfMessages >= umbral. ResizablePoolCell evalua isTimeForResize en cada mensaje (contador atomico) y, si toca y no hay un resize en curso, encola un mensaje Resize de gestion; ademas hace un resize inicial en start.
- **Fuente**: actor/src/main/scala/org/apache/pekko/routing/Resizer.scala:206-226,264-350 ; tests: actor-tests/src/test/scala/org/apache/pekko/routing/ResizerSpec.scala:138-254
- **Condiciones**: Para routees no ActorRefWithCell la presion es false; para celdas no ActorCell: threshold<1 -> true (no iniciadas ocupadas).
- **Orden / concurrencia / ciclo de vida**: resize se ejecuta como mensaje de gestion en el actor del router (secuencial con Add/Remove).
- **Dependencias**: AC-RT-013.1 ; AC-RT-003.3 ; AC-MBX-019
- **Equivalente Rust propuesto**: `ResizablePoolCell` con `AtomicU64` contador y `AtomicBool` resize_in_progress **Diferencias observables**: ninguna prevista.
- **Aserciones**:
  - A1: Programatico: `RoundRobinPool(nrOfInstances = 0, resizer)` tras iniciar tiene 2 routees - actor-tests/src/test/scala/org/apache/pekko/routing/ResizerSpec.scala:138-153.
  - A2: En configuracion (resizer.enabled) tambien - actor-tests/src/test/scala/org/apache/pekko/routing/ResizerSpec.scala:154-166.
  - A3: Crece bajo presion: el primer mensaje crea lowerBound; 2 mensajes mas no disparan; muchos mas llegan a upperBound (rampupRate 0.1, backoffRate 0.0, pressureThreshold 1) - actor-tests/src/test/scala/org/apache/pekko/routing/ResizerSpec.scala:168-213.
  - A4: Backoff: con rampupRate 1.0, backoffRate 1.0, backoffThreshold 0.40 el pool decrece al quitar presion (z > 2) - actor-tests/src/test/scala/org/apache/pekko/routing/ResizerSpec.scala:215-253.
  - A5: Resize: requestedCapacity > 0 -> addRoutees; < 0 -> removeRoutees(ultimos, stopChild = true); `resizeInProgress` en finally - derivada del codigo actor/src/main/scala/org/apache/pekko/routing/Resizer.scala:295-310; *no afirmada por spec*.
  - A6: Un Resizer personalizado con isTimeForResize(counter == 0) y resize=2/3 usa solo el primer mensaje/inicio - actor-tests/src/test/scala/org/apache/pekko/routing/RoutingSpec.scala:90-110,128-145.
- **Oraculo**: no aplica: dependiente del estado del mailbox y timing.
- **Decisiones**: Q-RT-4
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-RT-013.3 - Resizer.fromConfig: seleccion DefaultResizer vs OptimalSizeExploringResizer, exclusion mutua

- **Contrato observable**: [contractual por spec] Resizer.fromConfig(parent) lee las secciones `resizer` y `optimal-size-exploring-resizer`: solo la primera habilitada -> DefaultResizer; solo la segunda -> OptimalSizeExploringResizer; ninguna -> None; ambas -> ResizerInitializationException("cannot enable both resizer and optimal-size-exploring-resizer").
- **Fuente**: actor/src/main/scala/org/apache/pekko/routing/Resizer.scala:69-105 ; actor/src/main/resources/reference.conf:246-346 ; tests: actor-tests/src/test/scala/org/apache/pekko/routing/ResizerSpec.scala:61-102
- **Condiciones**: Claves: enabled, lower-bound, upper-bound, pressure-threshold, rampup-rate, backoff-threshold, backoff-rate, messages-per-resize (defaults reference.conf).
- **Orden / concurrencia / ciclo de vida**: n/a
- **Dependencias**: AC-RT-004 ; AC-RT-014
- **Equivalente Rust propuesto**: `Resizer::from_config(&Config) -> Result<Option<Resizer>, ResizerInitError>` **Diferencias observables**: ninguna prevista.
- **Aserciones**:
  - A1: resizer.enabled=on -> DefaultResizer - actor-tests/src/test/scala/org/apache/pekko/routing/ResizerSpec.scala:67-75.
  - A2: optimal-size-exploring-resizer.enabled=on -> DefaultOptimalSizeExploringResizer - actor-tests/src/test/scala/org/apache/pekko/routing/ResizerSpec.scala:76-84.
  - A3: Ambas habilitadas -> ResizerInitializationException - actor-tests/src/test/scala/org/apache/pekko/routing/ResizerSpec.scala:85-98.
  - A4: Ninguna (defecto) -> None - actor-tests/src/test/scala/org/apache/pekko/routing/ResizerSpec.scala:99-102.
  - A5: DefaultResizer.fromConfig(cfg) lee `resizer.enabled` y `cfg.getConfig("resizer")` - derivada del codigo actor/src/main/scala/org/apache/pekko/routing/Resizer.scala:100-104; *no afirmada por spec*.
- **Oraculo**: oraculo: viable (jshell) con Config de strings.
- **Decisiones**: Q-RT-3, Q-X-1
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-RT-014 - OptimalSizeExploringResizer

- **Contrato observable**: [contractual] Resizer que explora/optimiza el tamano segun throughput medido (ver reference.conf: action-interval, chance-of-exploration, explore-step-size, downsize-after-underutilized-for, downsize-ratio, optimization-range, weight-of-latest-metric, chance-of-ramping-down-when-full).
- **Fuente**: actor/src/main/scala/org/apache/pekko/routing/OptimalSizeExploringResizer.scala:30-316 ; actor/src/main/resources/reference.conf:291-346 ; tests: actor-tests/src/test/scala/org/apache/pekko/routing/MetricsBasedResizerSpec.scala:91-372 (no leidos)
- **Condiciones**: n/a: no extraido.
- **Orden / concurrencia / ciclo de vida**: n/a: sin garantias de orden propias
- **Dependencias**: AC-RT-013.2
- **Equivalente Rust propuesto**: `OptimalSizeExploringResizer` con metricas por tamano **Diferencias observables**: n/a
- **Aserciones**:
  - A1: Sin aserciones extraidas: no se leyeron cuerpos (listed).
- **Oraculo**: no aplica: aleatorio y dependiente de timing.
- **Decisiones**: Q-RT-4
- **Estado**: analysis=listed, impl=pending, validation=pending

## AC-RT-015 - Listeners / Listen / Deafen / WithListeners

- **Contrato observable**: [contractual por spec] Trait mixin: listeners en TreeSet[ActorRef] (ordenado por ActorRef), Listen(l) agrega, Deafen(l) quita, WithListeners(f) aplica f a cada listener, gossip(msg) envia a todos. listenerManagement es el Receive a componer.
- **Fuente**: actor/src/main/scala/org/apache/pekko/routing/Listeners.scala:20-59 ; tests: actor-tests/src/test/scala/org/apache/pekko/actor/routing/ListenerSpec.scala:29-70
- **Condiciones**: El TreeSet exige que ActorRef sea Comparable (orden por path/uid).
- **Orden / concurrencia / ciclo de vida**: gossip itera en el orden del TreeSet; sender implicito de gossip = noSender salvo que se pase.
- **Dependencias**: -
- **Equivalente Rust propuesto**: trait `Listeners { fn listeners(&mut self) -> &mut BTreeSet<ActorRef>; fn gossip(&self, msg) }` **Diferencias observables**: Requiere Ord en ActorRef; el orden debe ser estable (path + uid).
- **Aserciones**:
  - A1: Con 3 listeners, Deafen(a3), WithListeners(_ ! "foo") y gossip("bar"): barCount == 2 y fooLatch(2) se abre - actor-tests/src/test/scala/org/apache/pekko/actor/routing/ListenerSpec.scala:29-70.
  - A2: Listen duplicado es idempotente (TreeSet) - derivada del codigo actor/src/main/scala/org/apache/pekko/routing/Listeners.scala:40-47; *no afirmada por spec*.
  - A3: WithListeners(f) y Listen/Deafen son sealed (ListenerMessage) - derivada del codigo actor/src/main/scala/org/apache/pekko/routing/Listeners.scala:24-30; *no afirmada por spec*.
- **Oraculo**: oraculo: pekko-actor 1.7.1 oficial via jshell permite crear routers y comparar los destinatarios elegidos de forma determinista (round robin, broadcast, consistent hashing con claves String/bytes); lo aleatorio/temporal solo estadisticamente.
- **Decisiones**: Q-RT-8
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-RT-016 - Configuracion de deployment de routers (router.type-mapping, nr-of-instances, within, virtual-nodes-factor, tail-chopping-router.interval, routees.paths, pool-dispatcher, resizer)

- **Contrato observable**: [contractual por spec] Deployer lee `pekko.actor.deployment.<path>` con fallback a `default`; `router` puede ser un nombre del mapping `pekko.actor.router.type-mapping` (13 pares nombre -> clase) o un FQCN de un CustomRouterConfig con constructor (Config[, DynamicAccess]); cada Pool/Group se construye con `fromConfig(config)` leyendo nr-of-instances, resizer, within, virtual-nodes-factor, interval, routees.paths.
- **Fuente**: actor/src/main/resources/reference.conf:154-170,176-240,246-346 ; actor/src/main/scala/org/apache/pekko/actor/Deployer.scala:233-290 ; actor/src/main/scala/org/apache/pekko/routing/RoundRobin.scala:92-100 ; actor/src/main/scala/org/apache/pekko/routing/ScatterGatherFirstCompleted.scala (within) ; actor/src/main/scala/org/apache/pekko/routing/TailChopping.scala (interval) ; tests: actor-tests/src/test/scala/org/apache/pekko/routing/ConfiguredLocalRoutingSpec.scala:139-211
- **Condiciones**: La config manda sobre el codigo para el mismo nombre (ver AC-RT-004).
- **Orden / concurrencia / ciclo de vida**: La resolucion del deployment ocurre al crear el actor (actorOf).
- **Dependencias**: AC-CFG-012 ; AC-CFG-014 ; AC-RT-013.3
- **Equivalente Rust propuesto**: registro de fabricas por nombre (`router.type-mapping`) y `fn from_config(&Config)` por router **Diferencias observables**: ninguna prevista.
- **Aserciones**:
  - A1: type-mapping contiene: from-code, round-robin-pool/group, random-pool/group, balancing-pool, smallest-mailbox-pool, broadcast-pool/group, scatter-gather-pool/group, tail-chopping-pool/group, consistent-hashing-pool/group - actor/src/main/resources/reference.conf:155-171.
  - A2: Defaults: router="from-code", nr-of-instances=1, within=5 seconds, virtual-nodes-factor=10, tail-chopping-router.interval=10 milliseconds, routees.paths=[], resizer.enabled=off - actor/src/main/resources/reference.conf:211-227,246-258.
  - A3: Custom router por FQCN con config `foo = bar` responde "bar" - actor-tests/src/test/scala/org/apache/pekko/routing/ConfiguredLocalRoutingSpec.scala:190-194.
  - A4: Router configurado para un hijo de un actor de sistema (`/sys-parent/round` con 6 routees) - actor-tests/src/test/scala/org/apache/pekko/routing/ConfiguredLocalRoutingSpec.scala:196-205.
  - A5: Deploy IO-DNS: "/IO-DNS/inet-address" usa consistent-hashing-pool con 4 instancias y mailbox unbounded - actor/src/main/resources/reference.conf:350-355 (fuera de alcance: IO).
- **Oraculo**: oraculo: viable (jshell) para el mapeo de tipos y valores por defecto.
- **Decisiones**: Q-RT-3, Q-X-2, Q-X-1
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-CS-001.1 - Orden topologico de fases (DAG) y deteccion de ciclos

- **Contrato observable**: [contractual por spec] Las fases se ordenan con DFS: toda fase aparece despues de las que dependen-de; fases referenciadas en depends-on pero no definidas tambien cuentan; un ciclo (incluida autodependencia) lanza IllegalArgumentException "Cycle detected in graph of phases. It must be a DAG. phase [<u>] depends transitively on itself. All dependencies: <phases>".
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/CoordinatedShutdown.scala:334-364 ; actor/src/main/resources/reference.conf:1294-1399 ; docs/src/main/paradox/coordinated-shutdown.md (sin comparar) ; tests: actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:37-65,71-121
- **Condiciones**: unmarked = claves + todas las dependencias; el orden entre fases independientes es arbitrario (depende del iterador del Set).
- **Orden / concurrencia / ciclo de vida**: n/a
- **Dependencias**: AC-CFG-001
- **Equivalente Rust propuesto**: `fn topological_sort(phases: &BTreeMap<String, Phase>) -> Result<Vec<String>, CycleError>` **Diferencias observables**: Orden entre ramas independientes no esta garantizado: los tests comparan solo restricciones; usar BTreeSet para determinismo.
- **Aserciones**:
  - A1: Map vacio -> Nil; {a} -> [a]; {b dep a} -> [a,b]; {c dep a, b dep a} -> a primero, luego b/c en cualquier orden - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:72-81.
  - A2: {b dep a, c dep b} -> [a,b,c]; {c dep (a,b), b dep a} -> [a,b,c]; {c dep (a,b)} -> c ultimo - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:83-90.
  - A3: {b->a, c->b, d->(b,c), e->d} -> [a,b,c,d,e]; dos cadenas independientes preservan su orden interno - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:92-100.
  - A4: Ciclos: a dep a; a<->b; c->(b), b->c; d->..->d -> IllegalArgumentException - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:103-119.
  - A5: Mensaje exacto del ciclo - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/CoordinatedShutdown.scala:347-350; *no afirmada por spec* (solo el tipo).
- **Oraculo**: oraculo: pekko-actor 1.7.1 oficial via jshell permite ejecutar CoordinatedShutdown.run con tareas de prueba y comparar orden de fases/resultado; `topologicalSort`/`phasesFromConfig` son private[pekko] (no comparables directamente, solo via orderedPhases/run).
- **Decisiones**: Q-CS-1
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-CS-001.2 - phasesFromConfig: lectura de fases (depends-on, timeout, recover, enabled, default-phase-timeout)

- **Contrato observable**: [contractual por spec] Cada entrada de `phases` debe ser un objeto; defaults por fase: timeout = default-phase-timeout, recover = true, enabled = true, depends-on = []; un valor que no es objeto lanza IllegalArgumentException "Expected object value for [<k>], got [<v>]".
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/CoordinatedShutdown.scala:299-332 ; actor/src/main/resources/reference.conf:1245-1400 ; tests: actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:501-520
- **Condiciones**: Las claves de fase son nombres; depends-on es lista de strings.
- **Orden / concurrencia / ciclo de vida**: n/a
- **Dependencias**: AC-CFG-001 ; AC-CS-001.1
- **Equivalente Rust propuesto**: `fn phases_from_config(&Config) -> Result<BTreeMap<String, Phase>>` **Diferencias observables**: Depende del parser HOCON (Q-X-1): duraciones y booleanos "on/off".
- **Aserciones**:
  - A1: Config con default-phase-timeout=10s, a={}, b{depends-on=[a],timeout=15s}, c{depends-on=[a,b],recover=off} produce Phase(Set.empty,10s,true,true), Phase({a},15s,true,true), Phase({a,b},10s,false,true) - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:501-520.
  - A2: Valor que no es objeto -> IllegalArgumentException("Expected object value for [k], got [v]") - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/CoordinatedShutdown.scala:329-330; *no afirmada por spec*.
- **Oraculo**: no aplica: funcion interna; observable via orderedPhases.
- **Decisiones**: Q-CS-1, Q-X-1
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-CS-002 - Fases predefinidas (incluye las de cluster, vacias en local) y constantes publicas

- **Contrato observable**: [contractual por spec] reference.conf define 12 fases encadenadas: before-service-unbind, service-unbind, service-requests-done, service-stop, before-cluster-shutdown, cluster-sharding-shutdown-region (timeout 10s), cluster-leave, cluster-exiting (10s), cluster-exiting-done, cluster-shutdown, before-actor-system-terminate, actor-system-terminate (10s); constantes Phase* publicas.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/CoordinatedShutdown.scala:44-109 ; actor/src/main/resources/reference.conf:1294-1399 ; tests: actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:123-139
- **Condiciones**: Las fases de cluster existen aunque no haya cluster (vacias).
- **Orden / concurrencia / ciclo de vida**: n/a
- **Dependencias**: AC-CS-001.1 ; AC-CS-001.2
- **Equivalente Rust propuesto**: constantes `PHASE_*: &str` y config por defecto embebida **Diferencias observables**: ninguna prevista.
- **Aserciones**:
  - A1: orderedPhases por defecto == [before-service-unbind, service-unbind, service-requests-done, service-stop, before-cluster-shutdown, cluster-sharding-shutdown-region, cluster-leave, cluster-exiting, cluster-exiting-done, cluster-shutdown, before-actor-system-terminate, actor-system-terminate] - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:123-139.
  - A2: default-phase-timeout = 5 s; timeouts propios: cluster-sharding-shutdown-region 10 s, cluster-exiting 10 s, actor-system-terminate 10 s - actor/src/main/resources/reference.conf:1248,1353,1368,1395; *no afirmados por spec*.
- **Oraculo**: oraculo: pekko-actor 1.7.1 oficial via jshell permite ejecutar CoordinatedShutdown.run con tareas de prueba y comparar orden de fases/resultado; `topologicalSort`/`phasesFromConfig` son private[pekko] (no comparables directamente, solo via orderedPhases/run).
- **Decisiones**: Q-CS-3
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-CS-003.1 - addTask / addCancellableTask: validaciones y semantica de fase

- **Contrato observable**: [contractual por spec] addTask(phase, name)(task) y addCancellableTask exigen fase conocida (definida o referenciada) y nombre no vacio (IllegalArgumentException por require); anadir no reemplaza tareas previas; las tareas de una fase corren en paralelo sin orden; la siguiente fase no empieza hasta que todas terminan.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/CoordinatedShutdown.scala:551-631 ; tests: actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:141-168
- **Condiciones**: knownPhases = phases.keySet ++ dependsOn.
- **Orden / concurrencia / ciclo de vida**: Paralelas dentro de la fase (Future.sequence); entre fases en serie.
- **Dependencias**: AC-CS-001.1
- **Equivalente Rust propuesto**: `add_task(phase: &str, name: &str, task: impl FnOnce() -> BoxFuture<Done>) -> Result<Cancellable, CsError>` **Diferencias observables**: Los requires pasan a Result::Err con los mismos mensajes.
- **Aserciones**:
  - A1: Fases a, b(dep a), c(dep b,a): orden A, B, B, C; la tarea b2 (100 ms) termina antes que c - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:141-168.
  - A2: Fase desconocida: require "Unknown phase [<phase>], known phases [<knownPhases>]. All phases (along with their optional dependencies) must be defined in configuration" (addCancellableTask) / mismo texto con espacio (addTask) - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/CoordinatedShutdown.scala:566-568,607-610; *no afirmada por spec*.
  - A3: Nombre vacio: require "Set a task name when adding tasks to the Coordinated Shutdown. Try to use unique, self-explanatory names." - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/CoordinatedShutdown.scala:569-572,611-614; *no afirmada por spec*.
- **Oraculo**: oraculo: pekko-actor 1.7.1 oficial via jshell permite ejecutar CoordinatedShutdown.run con tareas de prueba y comparar orden de fases/resultado; `topologicalSort`/`phasesFromConfig` son private[pekko] (no comparables directamente, solo via orderedPhases/run).
- **Decisiones**: Q-CS-1
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-CS-003.2 - Cancelacion (Cancellable), registro duplicado, tareas anadidas durante el run y concurrencia

- **Contrato observable**: [contractual por spec] addCancellableTask devuelve un Cancellable (cancel(): true si estaba pendiente; isCancelled). La misma tarea registrada N veces corre N veces menos las canceladas (identidad por objeto, no por nombre/igualdad). Durante el run se pueden anadir tareas a fases posteriores; cancelar una tarea en curso no la interrumpe. Cancelacion concurrente segura.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/CoordinatedShutdown.scala:405-533,565-574 ; tests: actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:170-363,485-499
- **Condiciones**: cancel() devuelve false si ya estaba en ejecucion o cancelada.
- **Orden / concurrencia / ciclo de vida**: Estado por tarea con AtomicReference; registro por fase en ConcurrentHashMap con merge (descarta canceladas al fusionar).
- **Dependencias**: AC-CS-003.1
- **Equivalente Rust propuesto**: registro `Vec<Arc<TaskDef>>` por fase con estado atomico {Pending, Cancelled, Running(promise)} **Diferencias observables**: Identidad por tarea = Arc distinto; no usar igualdad estructural (ver comentario C:411-416).
- **Aserciones**:
  - A1: Tres tareas copy1..3: cancelar la 2 deja 2 mensajes distintos (copy1, copy3); isCancelled refleja el estado - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:170-203.
  - A2: La misma tarea "labor" registrada 3 veces: cancelar la 1 deja 2 ejecuciones del mismo mensaje - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:205-236.
  - A3: taskA cancela taskB y A agrega B: mensajes {"A adds B", "A cancels B"}; B no ejecuta su cancelacion - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:238-306.
  - A4: 20 registros de concurrentTaskA, 10 cancelados desde hilos distintos (cancel() == true cada uno): corren 20 mensajes (messageA y BMessage) - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:308-363.
  - A5: Una tarea de la fase a puede anadir una tarea a la fase b y se ejecuta (A luego B) - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:485-499.
  - A6: Tarea invocada dos veces simultaneas: warning "Task [{}] in phase [{}] was invoked multiple times and deduplicated." - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/CoordinatedShutdown.scala:455-457; *no afirmada por spec*.
  - A7: Registro tardio: tareas anadidas despues de iniciar el run pueden no ejecutarse (doc) - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/CoordinatedShutdown.scala:556-563; *no afirmada por spec*.
- **Oraculo**: oraculo: pekko-actor 1.7.1 oficial via jshell permite ejecutar CoordinatedShutdown.run con tareas de prueba y comparar orden de fases/resultado; `topologicalSort`/`phasesFromConfig` son private[pekko] (no comparables directamente, solo via orderedPhases/run).
- **Decisiones**: Q-CS-2
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-CS-004.1 - run(reason, fromPhase): una sola vez, fases en orden, futuro compartido, shutdownReason

- **Contrato observable**: [contractual por spec] run(reason) ejecuta las fases en orden topologico solo la primera vez (CAS sobre Option[Reason]); llamadas posteriores devuelven el mismo Future sin re-ejecutar; run(reason, Some(fase)) empieza en esa fase; shutdownReason() es None antes y Some(razon de la primera llamada) despues. Loguea info "Running CoordinatedShutdown with reason [<reason>]".
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/CoordinatedShutdown.scala:683-785 ; tests: actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:141-168,365-400
- **Condiciones**: fromPhase desconocida: orderedPhases.dropWhile(_ != p) deja vacio (no hay error) - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/CoordinatedShutdown.scala:777-780.
- **Orden / concurrencia / ciclo de vida**: Ejecuta en system.dispatchers.internalDispatcher; cada fase es flatMap de la anterior.
- **Dependencias**: AC-CS-003.1
- **Equivalente Rust propuesto**: `fn run(&self, reason: Reason, from_phase: Option<&str>) -> SharedFuture<Done>` **Diferencias observables**: ninguna prevista.
- **Aserciones**:
  - A1: run desde "b": se ejecutan B y C (no A); shutdownReason == Some(CustomReason) - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:365-383.
  - A2: Solo corre una vez: antes de run shutdownReason == None; tras run(CustomReason) Some(CustomReason); una segunda run(UnknownReason) no re-ejecuta y la razon sigue siendo CustomReason - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:385-400.
  - A3: Fase sin tareas: debug "Performing phase [..] with [0] tasks" y continua - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/CoordinatedShutdown.scala:736-739; *no afirmada por spec*.
- **Oraculo**: oraculo: pekko-actor 1.7.1 oficial via jshell permite ejecutar CoordinatedShutdown.run con tareas de prueba y comparar orden de fases/resultado; `topologicalSort`/`phasesFromConfig` son private[pekko] (no comparables directamente, solo via orderedPhases/run).
- **Decisiones**: Q-CS-2, Q-X-4
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-CS-004.2 - Timeout por fase y recover (continuar o abortar con TimeoutException)

- **Contrato observable**: [contractual por spec] Cada fase tiene un timeout; si vence con recover=on se loguea warning "Coordinated shutdown phase [<f>] timed out after <t>" y se continua; con recover=off el Future falla con TimeoutException("Coordinated shutdown phase [<f>] timed out after <t>") y las fases siguientes no corren. Una tarea fallida con recover=on loguea warning "Task [<n>] failed in phase [<f>]: <msg>" y cuenta como Done.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/CoordinatedShutdown.scala:423-464,722-785 ; tests: actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:402-462
- **Condiciones**: after() lanza IllegalStateException cuando el scheduler esta cerrado: se devuelve result.
- **Orden / concurrencia / ciclo de vida**: firstCompletedOf(result, timeoutFut).
- **Dependencias**: AC-CS-004.1 ; AC-SCH-001
- **Equivalente Rust propuesto**: tokio::time::timeout por fase + politica `recover` **Diferencias observables**: La duracion se muestra como "100 milliseconds" (toString de Duration de Scala): el formato debe replicarse en el Display.
- **Aserciones**:
  - A1: recover=on: a1 falla ("boom") -> warning "Task [a1] failed in phase [a]: boom"; la fase a espera a a2 (100 ms); b (100 ms, nunca completa) -> warning "Coordinated shutdown phase [b] timed out after 100 milliseconds"; C se ejecuta - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:402-440.
  - A2: recover=off y timeout: Future falla con TimeoutException y C no se ejecuta - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:442-462.
  - A3: Una tarea que lanza al construir el Future: con recover warning "Task [..] in phase [..] threw an exception before its future could be constructed: <msg>" ; sin recover Future fallido - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/CoordinatedShutdown.scala:443-453; *no afirmada por spec*.
  - A4: actor-system-terminate: si el timeout dispara demasiado pronto (por terminacion del sistema) se devuelve `result` sin fallar - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/CoordinatedShutdown.scala:751-767; *no afirmada por spec*.
- **Oraculo**: oraculo: pekko-actor 1.7.1 oficial via jshell permite ejecutar CoordinatedShutdown.run con tareas de prueba y comparar orden de fases/resultado; `topologicalSort`/`phasesFromConfig` son private[pekko] (no comparables directamente, solo via orderedPhases/run).
- **Decisiones**: Q-CS-2, Q-X-5
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-CS-004.3 - Fases deshabilitadas (enabled = off)

- **Contrato observable**: [contractual por spec] Una fase con enabled=off salta todas sus tareas y loguea info "Phase [<f>] disabled through configuration, skipping [<n>] tasks." (solo si tiene tareas registradas); el run continua.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/CoordinatedShutdown.scala:730-734 ; tests: actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:464-483
- **Condiciones**: n/a
- **Orden / concurrencia / ciclo de vida**: n/a
- **Dependencias**: AC-CS-004.1
- **Equivalente Rust propuesto**: campo `enabled` en Phase **Diferencias observables**: ninguna prevista.
- **Aserciones**:
  - A1: fase b deshabilitada: B no se ejecuta, C si, el resultado es Done y el log incluye "Phase [b] disabled through configuration" - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:464-483.
- **Oraculo**: oraculo: pekko-actor 1.7.1 oficial via jshell permite ejecutar CoordinatedShutdown.run con tareas de prueba y comparar orden de fases/resultado; `topologicalSort`/`phasesFromConfig` son private[pekko] (no comparables directamente, solo via orderedPhases/run).
- **Decisiones**: Q-CS-2
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-CS-005 - Reasons y reason-overrides (exit-code, terminate-actor-system, exit-jvm)

- **Contrato observable**: [contractual por spec] Reason es un trait abierto con 7 razones predefinidas: UnknownReason, ActorSystemTerminateReason, JvmExitReason, ClusterDowningReason, ClusterJoinUnsuccessfulReason, IncompatibleConfigurationDetectedReason, ClusterLeavingReason. `reason-overrides."<getClass.getName>"` sobreescribe exit-code, exit-jvm y terminate-actor-system; por defecto ClusterDowning y ClusterJoinUnsuccessful usan exit-code = -1.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/CoordinatedShutdown.scala:111-187,205-213 ; actor/src/main/resources/reference.conf:1278-1292 ; tests: actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:522-533
- **Condiciones**: Solo exit-jvm, exit-code y terminate-actor-system pueden sobreescribirse (doc reference.conf:1278-1281).
- **Orden / concurrencia / ciclo de vida**: n/a
- **Dependencias**: AC-CS-004.1 ; AC-CFG-001
- **Equivalente Rust propuesto**: trait `Reason` con `fn config_key(&self) -> &str` (nombre estable como el de la clase JVM) **Diferencias observables**: El nombre de clase JVM no existe en Rust: se necesita un identificador estable por razon que coincida con la clave de config.
- **Aserciones**:
  - A1: exit-code por defecto == 0 - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:522-526.
  - A2: confWithOverrides(conf, Some(ClusterDowningReason)) -> exit-code == -1 - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:528-533.
  - A3: La clave es `reason-overrides."<reason.getClass.getName>"` (p. ej. "org.apache.pekko.actor.CoordinatedShutdown$ClusterDowningReason$") con fallback a conf - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/CoordinatedShutdown.scala:206-213 y actor/src/main/resources/reference.conf:1286-1291; *no afirmada por spec* en la clave literal.
- **Oraculo**: oraculo: pekko-actor 1.7.1 oficial via jshell permite ejecutar CoordinatedShutdown.run con tareas de prueba y comparar orden de fases/resultado; `topologicalSort`/`phasesFromConfig` son private[pekko] (no comparables directamente, solo via orderedPhases/run).
- **Decisiones**: Q-CS-4
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-CS-006.1 - Fase actor-system-terminate: terminate-actor-system, exit-jvm, exit-code

- **Contrato observable**: [contractual por spec] Al ejecutar la fase actor-system-terminate (tarea interna "terminate-system"): con terminate-actor-system=on llama a finalTerminate y completa al terminar el ActorSystem (y System.exit(exitCode) si exit-jvm=on y no es salida por hook); con exit-jvm sin terminate sale inmediatamente. Si exit-jvm y terminate-actor-system, un hilo "CoordinatedShutdown-exit" fuerza System.exit si el sistema no termina en el timeout de la fase.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/CoordinatedShutdown.scala:239-277 ; tests: actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:535-544
- **Condiciones**: terminate-actor-system=off + exit-jvm=on: System.exit inmediato sin terminar el sistema.
- **Orden / concurrencia / ciclo de vida**: Tarea en la fase final; no anadir fases dependientes de ella (dispatcher y scheduler ya cerrados).
- **Dependencias**: AC-CS-004.2 ; AC-SYS-004
- **Equivalente Rust propuesto**: tarea interna registrada en `CoordinatedShutdown::init`; `std::process::exit` configurable **Diferencias observables**: "exit-jvm" saldra el proceso con `std::process::exit` (no hay JVM); mantener la clave de config.
- **Aserciones**:
  - A1: CoordinatedShutdown(sys).run(CustomReason) termina el ActorSystem (whenTerminated completo) y la razon queda en CustomReason - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:535-544.
  - A2: Con exit-jvm=on: System.exit(exit-code) tras el termino del sistema salvo hook JVM (runningJvmHook) - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/CoordinatedShutdown.scala:265-270; *no afirmada por spec* (no se prueba la salida del proceso).
  - A3: Hilo de respaldo "CoordinatedShutdown-exit": si Await.ready(whenTerminated, timeout) falla y no es hook -> System.exit(exitCode) - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/CoordinatedShutdown.scala:249-263; *no afirmada por spec*.
- **Oraculo**: no aplica: termina el proceso/sistema; solo la parte de razon/futuro es comparable.
- **Decisiones**: Q-CS-5
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-CS-006.2 - Integracion con ActorSystem.terminate/close y combinacion de config invalida

- **Contrato observable**: [contractual por spec] ActorSystem.terminate() ejecuta CoordinatedShutdown(run con ActorSystemTerminateReason) si run-by-actor-system-terminate=on (y no esta abortando), si no finalTerminate. close() = terminate + Await(whenTerminated, close-actor-system-timeout). run-by-actor-system-terminate=on con terminate-actor-system=off lanza ConfigurationException "pekko.coordinated-shutdown.run-by-actor-system-terminate=on and pekko.coordinated-shutdown.terminate-actor-system=off is not a supported configuration combination." al crear el sistema.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:474-482,1086-1112 ; actor/src/main/resources/reference.conf:1250-1257,1273-1276 ; tests: actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:546-582
- **Condiciones**: close() bloquea hasta el timeout configurado.
- **Orden / concurrencia / ciclo de vida**: terminate() devuelve whenTerminated inmediatamente tras iniciar CS.
- **Dependencias**: AC-SYS-004 ; AC-CS-006.1
- **Equivalente Rust propuesto**: Settings::validate + `ActorSystem::terminate`/`close` delegando en CoordinatedShutdown **Diferencias observables**: ninguna prevista.
- **Aserciones**:
  - A1: sys.terminate() completa y la razon es ActorSystemTerminateReason - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:546-555.
  - A2: Con run-by-actor-system-terminate=off, terminate() no ejecuta CS (shutdownReason == None) - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:557-570.
  - A3: terminate-actor-system=off (con run-by... por defecto on) lanza ConfigurationException al crear el sistema - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:572-582.
  - A4: Texto exacto de la excepcion y close-actor-system-timeout = 60 s por defecto - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:478-481,1108-1109 y actor/src/main/resources/reference.conf:1257; *no afirmada por spec*.
- **Oraculo**: oraculo: viable (jshell): comprobar shutdownReason y la excepcion de configuracion.
- **Decisiones**: Q-CS-5
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-CS-007.1 - addJvmShutdownHook / addCancellableJvmShutdownHook (API de hooks de apagado del proceso)

- **Contrato observable**: [contractual por spec] addCancellableJvmShutdownHook registra un hilo en el runtime (si no se inicio el run); devuelve un Cancellable que lo quita; contador (latch) de hooks; si ya hay run o el runtime esta apagando devuelve un Cancellable ya cancelado y loguea warning "Could not addJvmShutdownHook, due to: ..." en el segundo caso.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/CoordinatedShutdown.scala:368-384,537-549,830-913 ; tests: actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:584-659,757-803
- **Condiciones**: Nombre del hilo: "<system>-shutdown-hook-<n>" - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/CoordinatedShutdown.scala:855; *no afirmada por spec*.
- **Orden / concurrencia / ciclo de vida**: Los hooks corren concurrentemente en cualquier orden antes de los hooks internos de Pekko (Artery).
- **Dependencias**: AC-CS-007.2
- **Equivalente Rust propuesto**: trait `ProcessShutdownHooks { add_hook, remove_hook }` + `add_cancellable_process_shutdown_hook` **Diferencias observables**: En Rust: `std::panic`/`ctrlc`/`tokio::signal` o atexit; no hay `Runtime.addShutdownHook` equivalente directo (Q-CS-6).
- **Aserciones**:
  - A1: run-by-jvm-shutdown-hook=off, terminate-actor-system=off: tras agregar un hook de usuario hay 1 hook (sin hook del sistema); tras cancelar y apagar el sistema quedan 0 - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:584-598,757-803.
  - A2: run-by-jvm-shutdown-hook=on, terminate-actor-system=off: 2 hooks (usuario + sistema) - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:600-615.
  - A3: on/on: 2 hooks (usuario + actor system) - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:617-630.
  - A4: pekko.jvm-shutdown-hooks=off: solo el hook de usuario (1) - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:632-645.
  - A5: CoordinatedShutdown(system) accesible tras terminar el sistema (no lanza) - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:647-659.
  - A6: Validaciones del runtime mock (JDK): hook ya vivo -> IllegalArgumentException("Hook already running"); ya registrado -> "Hook previously registered"; apagando -> IllegalStateException("Shutdown in progress") - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:766-791 (mock del test; comportamiento del JDK, no de Pekko).
- **Oraculo**: no aplica: depende del runtime del proceso (JVM).
- **Decisiones**: Q-CS-6
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-CS-007.2 - Ejecucion de CoordinatedShutdown desde el hook de proceso (run-by-jvm-shutdown-hook)

- **Contrato observable**: [contractual por spec] Con pekko.jvm-shutdown-hooks y run-by-jvm-shutdown-hook en on, init registra un hook que marca runningJvmHook, y si el sistema no termino ejecuta run(JvmExitReason) esperando max(totalTimeout, 3 s); errores -> warning "CoordinatedShutdown from JVM shutdown failed: <msg>"; el hook se cancela al terminar el sistema para no filtrar referencias.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/CoordinatedShutdown.scala:215-237,279-297 ; tests: actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:600-659
- **Condiciones**: El hook no corre si whenTerminated ya esta completo.
- **Orden / concurrencia / ciclo de vida**: runningJvmHook evita System.exit desde la fase actor-system-terminate.
- **Dependencias**: AC-CS-007.1 ; AC-CS-005
- **Equivalente Rust propuesto**: registro del hook de senal (SIGTERM/SIGINT) opcional **Diferencias observables**: SIGINT no funciona con hooks JVM (doc reference.conf:1268-1269); en Rust se puede soportar SIGINT: diferencia a decidir.
- **Aserciones**:
  - A1: Con run-by-jvm-shutdown-hook=on existe 1 hook del sistema - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:600-615 (cuenta).
  - A2: Al terminar el sistema, el hook del actor system se cancela (shutdownHooks quedan 0) - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:799-800.
  - A3: run(JvmExitReason) esperado con Await.ready(.., max(totalTimeout, 3 s)) - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/CoordinatedShutdown.scala:287-289; *no afirmada por spec*.
- **Oraculo**: no aplica: depende de senales del proceso.
- **Decisiones**: Q-CS-6
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-CS-008 - addActorTerminationTask (mensaje de stop opcional, espera Terminated) y CoordinatedShutdownTerminationWatcher

- **Contrato observable**: [contractual por spec] La tarea envia stopMsg (si hay) al actor, lo vigila a traves de un actor del sistema (coordinatedShutdownTerminationWatcher) y completa Done al llegar su Terminated; actor ya terminado o ya visto -> completa; vigilar el mismo actor dos veces (misma o distinta fase) funciona; si no termina antes del deadline de la fase la promesa falla con TimeoutException "Actor [<a>] termination timed out after [<t>] during coordinated shutdown".
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/CoordinatedShutdown.scala:649-658,917-973 ; tests: actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:692-753
- **Condiciones**: No se puede usar ask (no se pueden crear actores durante el shutdown): se usa un actor de sistema con timers.
- **Orden / concurrencia / ciclo de vida**: Watch del actor solo la primera vez; el timer por promesa se cancela al completar.
- **Dependencias**: AC-CS-003.1 ; AC-DW-001
- **Equivalente Rust propuesto**: `add_actor_termination_task(phase, name, actor, stop: Option<Msg>)` + watcher interno **Diferencias observables**: ninguna prevista.
- **Aserciones**:
  - A1: Con stop "stop": el actor recibe "stop"; el resultado no esta listo en 100 ms; tras PoisonPill completa Done - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:692-702.
  - A2: Sin mensaje de stop: no se envia nada; completa tras PoisonPill - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:704-714.
  - A3: Actor ya terminado antes: completa Done - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:716-726.
  - A4: Mismo actor en la misma fase con stop1/stop2: recibe ambos y completa tras PoisonPill - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:728-738.
  - A5: Mismo actor en dos fases: recibe solo el stop de la primera (stopa) y completa; la segunda ve alreadySeenTerminated - actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:740-753 ; codigo actor/src/main/scala/org/apache/pekko/actor/CoordinatedShutdown.scala:938,965.
  - A6: Texto de timeout del watcher - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/CoordinatedShutdown.scala:967-970; *no afirmada por spec*.
- **Oraculo**: oraculo: pekko-actor 1.7.1 oficial via jshell permite ejecutar CoordinatedShutdown.run con tareas de prueba y comparar orden de fases/resultado; `topologicalSort`/`phasesFromConfig` son private[pekko] (no comparables directamente, solo via orderedPhases/run).
- **Decisiones**: Q-CS-2
- **Estado**: analysis=extracted, impl=pending, validation=pending

## AC-CS-009 - timeout(phase), totalTimeout, default-phase-timeout, close-actor-system-timeout

- **Contrato observable**: [contractual por spec] timeout(phase) devuelve el timeout configurado o lanza IllegalArgumentException "Unknown phase [<phase>]. All phases must be defined in configuration"; totalTimeout() es la suma de timeouts de las fases que tienen alguna tarea registrada; default-phase-timeout = 5 s.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/CoordinatedShutdown.scala:805-822 ; actor/src/main/resources/reference.conf:1246-1257 ; tests: - (sin spec dedicado; usado en CoordinatedShutdownSpec:287-289 indirectamente)
- **Condiciones**: n/a
- **Orden / concurrencia / ciclo de vida**: n/a
- **Dependencias**: AC-CS-001.2
- **Equivalente Rust propuesto**: `fn timeout(&self, phase) -> Result<Duration>` y `fn total_timeout(&self) -> Duration` **Diferencias observables**: ninguna prevista.
- **Aserciones**:
  - A1: timeout(phase desconocida) lanza IllegalArgumentException con el texto indicado - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/CoordinatedShutdown.scala:810-815; *no afirmada por spec*.
  - A2: totalTimeout() == suma de timeout(fase) para fases con tareas; sin tareas es 0 (el hook JVM usa max(.., 3 s)) - derivada del codigo actor/src/main/scala/org/apache/pekko/actor/CoordinatedShutdown.scala:515-521,820-822,287-288; *no afirmada por spec*.
  - A3: Defaults: default-phase-timeout = 5 s, close-actor-system-timeout = 60 s - actor/src/main/resources/reference.conf:1248,1257; *no afirmados por spec*.
- **Oraculo**: oraculo: viable (jshell): `CoordinatedShutdown.timeout(phase)` y `totalTimeout()` son publicas.
- **Decisiones**: Q-CS-2
- **Estado**: analysis=extracted, impl=pending, validation=pending

## Descomposiciones

- AC-ASK-001 -> AC-ASK-001.1 (ask feliz), .2 (entradas invalidas/terminados), .3 (AskTimeoutException).
- AC-ASK-009 -> AC-ASK-009.1 (intentos), .2 (demoras y backoff), .3 (predicado).
- AC-ASK-011 -> AC-ASK-011.1 (maquina de estados), .2 (callTimeout), .3 (backoff exponencial/constructor), .4 (callbacks/succeed/fail/sync).
- AC-ASK-015 -> AC-ASK-015.1 (props/validaciones/mensajes), .2 (onStop), .3 (onFailure), .4 (mientras detenido + reset), .5 (finalStopMessage).
- AC-FSM-001 -> AC-FSM-001.1 (DSL de State), .2 (when/initialize/despacho).
- AC-FSM-006 -> AC-FSM-006.1 (stop/onTermination/estado inexistente), .2 (terminacion externa).
- AC-RT-003 -> AC-RT-003.1 (ruteo en hilo del emisor/start), .2 (mensajes de gestion), .3 (evicion/parada).
- AC-RT-009 -> AC-RT-009.1 (seleccion por clave), .2 (ConsistentHash), .3 (MurmurHash).
- AC-RT-013 -> AC-RT-013.1 (capacidad), .2 (pressure/resize), .3 (fromConfig/exclusion mutua).
- AC-CS-001 -> AC-CS-001.1 (toposort), .2 (phasesFromConfig).
- AC-CS-003 -> AC-CS-003.1 (addTask), .2 (cancelacion/duplicados/concurrencia).
- AC-CS-004 -> AC-CS-004.1 (una sola vez/orden), .2 (timeout/recover), .3 (fases deshabilitadas).
- AC-CS-006 -> AC-CS-006.1 (fase final), .2 (integracion ActorSystem/config invalida).
- AC-CS-007 -> AC-CS-007.1 (API de hooks), .2 (run desde el hook).

## Omisiones agregadas

- AC-FSM-009: accesores stateName/stateData/nextStateData e initialize con IllegalStateException (FSM.scala:719-744; FSMTransitionSpec.scala:156-179).
- AC-RT-016: mapeo de configuracion de deployment de routers (reference.conf:154-170, 176-346; Deployer.scala:233-290; ConfiguredLocalRoutingSpec.scala:139-211).
- AC-RT-009.2 / AC-RT-009.3 (ConsistentHash y MurmurHash como filas propias; antes agrupadas en AC-RT-009 y sin aserciones).
- Archivos medidos sin cita por el orquestador en mi area: `actor-tests/.../pattern/CircuitBreakerStressSpec.scala` y `actor-tests/src/test/java/.../pattern/CircuitBreakerTest.java` (NO leidos: ver listed); `CircuitBreakerMTSpec.scala` si se leyo y se incorporo a AC-ASK-011.1.

## Contradicciones halladas y corregidas (inventario viejo vs. codigo/tests)

- AC-ASK-001 decia "ref no soportado -> Future fallido"; el codigo/test fija IllegalArgumentException con mensaje exacto (AskSpec:61-70) y para ref terminada AskTimeoutException tras re-enviar el mensaje (AskSupport:372-376).
- AC-ASK-010 decia "timeout negativo -> IllegalArgumentException"; el codigo exige length > 0 (tambien 0 falla) con texto "must not be negative" (PromiseRef.scala:166-172): Q-ASK-2.
- AC-ASK-011 decia "HalfOpen deja pasar una llamada que cierra o reabre"; el spec ademas fija CircuitBreakerOpenException para las demas llamadas y onCallBreakerOpen (CircuitBreakerSpec:299-309). Falta en el inventario: callTimeout=Zero ignora defineFailureFn (CircuitBreaker.scala:863-875): Q-ASK-4.
- AC-FSM-003 decia "cancelado por cualquier mensaje"; el codigo re-arma el timeout en cada makeTransition (incluido stay) con el valor del estado (FSM.scala:896-907).
- AC-FSM-006 decia "Failure se loguea en error" sin precisar: Throwable -> error con causa; otro -> error(msg.toString) (FSM.scala:947-951); y "Next state X does not exist" termina sin detener el actor (FSM.scala:882-884): Q-FSM-4.
- AC-FSM-007 decia "getLog devuelve los ultimos logDepth"; el codigo lo declara `protected` y guarda datos previos al evento (FSM.scala:1011-1019).
- AC-RT-001 decia "sin routees -> deadLetters": el codigo lo hace solo si el sender es InternalActorRef (Router.scala:135).
- AC-RT-003 decia "PoisonPill al router lo detiene (y a sus routees)": PoisonPill es AutoReceivedMessage (mensaje de gestion), no hay test que lo afirme en RoutingSpec; la parada del router al quedar sin routees si (RoutingSpec:68-88).
- AC-RT-010 decia "sin routees responde de inmediato con fallo": el tipo es TimeoutException("Timeout due to no routees") frente al Scaladoc que menciona AskTimeoutException: Q-RT-6.
- AC-RT-012: los tests de reparto de BalancingSpec estan `pending` upstream (BalancingSpec.scala:133-165); el inventario no lo indicaba.
- AC-CS-006 decia "terminate-actor-system=off con run-by-actor-system-terminate=on es invalido": el tipo exacto es ConfigurationException con texto fijo (ActorSystem.scala:478-481).

## Filas que siguen listed

- AC-ASK-015.1: no se leyeron BackoffSupervisorSpec:68-91 ni tests de validacion de parametros.
- AC-ASK-016 (Backoff.scala, 643 lineas) y AC-ASK-017 (Patterns.scala, 1333 lineas + PatternsTest.java): fachadas/API deprecada, no leidas.
- AC-RT-014 (OptimalSizeExploringResizer.scala 316 lineas + MetricsBasedResizerSpec 386 lineas): no leidos.

## No revisado

- Documentacion publica `docs/src/main/paradox/{fsm,routing,coordinated-shutdown,actors}.md` no se comparo linea a linea con codigo/tests (solo se cita su existencia).
- CircuitBreakerStressSpec, CircuitBreakerTest.java, StashJavaAPITestActors.java (Stash no es de este grupo), AbstractFSM.scala (cubierto por AC-JAPI-003), PatternsTest.java.
- `pattern/package.scala` (56 lineas) y `AbstractCircuitBreaker.java`/`AbstractPromiseActorRef.java` (solo offsets Unsafe): sin comportamiento observable propio.

## Cobertura

| Archivo de Pekko | Filas |
|---|---|
| actor/src/main/scala/org/apache/pekko/pattern/AskSupport.scala | AC-ASK-001.1-.3, AC-ASK-002, AC-ASK-003, AC-ASK-004 |
| .../pattern/extended/package.scala | AC-ASK-004 |
| .../pattern/StatusReply.scala | AC-ASK-005 |
| .../pattern/PipeToSupport.scala | AC-ASK-006 |
| .../pattern/GracefulStopSupport.scala | AC-ASK-007 |
| .../pattern/FutureTimeoutSupport.scala | AC-ASK-008 |
| .../pattern/RetrySupport.scala | AC-ASK-009.1-.3 |
| .../pattern/PromiseRef.scala | AC-ASK-010 |
| .../pattern/CircuitBreaker.scala | AC-ASK-011.1-.4, AC-ASK-012 |
| .../pattern/CircuitBreakersRegistry.scala | AC-ASK-013 |
| .../pattern/internal/CircuitBreakerTelemetry.scala | AC-ASK-014 |
| .../pattern/BackoffSupervisor.scala, BackoffOptions.scala, HandleBackoff.scala | AC-ASK-015.1-.5 |
| .../pattern/internal/BackoffOnStopSupervisor.scala | AC-ASK-015.2, .4, .5 |
| .../pattern/internal/BackoffOnRestartSupervisor.scala | AC-ASK-015.3, .4 |
| .../pattern/Backoff.scala | AC-ASK-016 (listed) |
| .../pattern/Patterns.scala | AC-ASK-017 (listed) |
| .../pattern/package.scala | sin comportamiento propio (solo agrupa traits ask/pipe/... y alias de tipos) |
| actor/src/main/java/.../pattern/AbstractCircuitBreaker.java, AbstractPromiseActorRef.java | sin comportamiento observable (offsets Unsafe de CAS); cubierto por AC-ASK-011/002 |
| .../actor/FSM.scala | AC-FSM-001.1-.2, 002, 003, 004, 005, 006.1-.2, 007, 008, 009 |
| .../routing/Router.scala | AC-RT-001 |
| .../routing/RouterConfig.scala | AC-RT-002, AC-RT-004, AC-RT-003.2 |
| .../routing/RoutedActorCell.scala, RoutedActorRef.scala | AC-RT-003.1-.3 |
| .../routing/RoundRobin.scala / Random.scala / Broadcast.scala | AC-RT-005 / 006 / 007 |
| .../routing/SmallestMailbox.scala | AC-RT-008 |
| .../routing/ConsistentHashing.scala, ConsistentHash.scala, MurmurHash.scala | AC-RT-009.1 / .2 / .3 |
| .../routing/ScatterGatherFirstCompleted.scala / TailChopping.scala / Balancing.scala | AC-RT-010 / 011 / 012 |
| .../routing/Resizer.scala | AC-RT-013.1-.3 |
| .../routing/OptimalSizeExploringResizer.scala | AC-RT-014 (listed) |
| .../routing/Listeners.scala | AC-RT-015 |
| .../actor/CoordinatedShutdown.scala | AC-CS-001.1 a AC-CS-009 |
| actor/src/main/resources/reference.conf (router/deployment, coordinated-shutdown, circuit-breaker, debug.fsm) | AC-RT-016, AC-RT-013.3, AC-CS-*, AC-ASK-011.3/013, AC-FSM-007 |

