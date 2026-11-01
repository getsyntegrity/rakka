# Contratos verificables: actor-typed, grupo typed-extended

Fuente de verdad: `upstream/pekko` fijado en `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (v1.7.1). Unidades: ASK, RCP, RT, ADP, EVT, LOG, PS, SPV, EXT, DSP, DLV, CS, SER, JAPI, TK. Todas las citas son `path:linea` relativas a `upstream/pekko` y se verificaron abriendo los archivos (ver script de verificacion en el cierre del informe). Decisiones transversales: Q-X-1..Q-X-9 (orquestador); especificas en `typed-extended.decisions.tsv`.

Convencion de aserciones: `A<n>` indica el comportamiento y el `path:linea` del cuerpo del test de Pekko que lo afirma; las que no tienen test se marcan "no afirmada por spec" y citan el codigo, con una caracterizacion propuesta (nunca presentada como test original de Pekko).

Cierre documental 2026-10-31: las filas `listed` de este grupo se leyeron contra implementacion y cuerpos de test; DLV (ProducerController, ConsumerController, WorkPullingProducerController, chunking) y JAPI y TK se descompusieron en hijos coherentes (ver "Descomposiciones"); las filas `extracted` con textos "derivada"/"no leido"/"no afirmada" se auditaron contra el cuerpo del spec (ver "Contradicciones halladas y corregidas").

## AT-ASK-001: AskPattern.ask / ? desde fuera de un actor (public)

- **Contrato observable**: Desde fuera de un actor, `ask` fabrica un ActorRef temporal atado a una Promise, lo inyecta en el mensaje mediante la funcion `replyTo => msg` y devuelve un Future con la primera respuesta. El operador `?` es un alias exacto.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/AskPattern.scala:84-137,145-181; actor/src/main/scala/org/apache/pekko/pattern/AskSupport.scala:722-736; docs/src/main/paradox/typed/interaction-patterns.md:203-242
- **Condiciones**: Normal: respuesta recibida antes del timeout completa el Future con ella. Limites: orden de chequeo en PromiseRef (actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/AskPattern.scala:148-162): (1) destino terminado, (2) timeout<=0, (3) caso normal; en (1) y (2) el ref entregado a la funcion es deadLetters y el Future ya viene fallido, pero la funcion se evalua y el mensaje se envia igual (actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/AskPattern.scala:176-181 llama p.ask que hace target ! message). Errores: TimeoutException (no AskTimeoutException) con los textos del contrato; IllegalArgumentException para timeout no positivo; el timeout sucede en scheduler (actor/src/main/scala/org/apache/pekko/pattern/AskSupport.scala:724-736) y el texto incluye " was sent by [<sender>]" solo si el sender no es noSender. Efectos: al expirar el ref temporal se detiene (actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/AskSpec.scala:174 expectTerminated(request.replyTo)) y una respuesta posterior se publica como DeadLetter cuyo recipient no es deadLetters ni el destino (actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/AskSpec.scala:184-188). messageClassName se fija despues de aplicar la funcion (actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/AskPattern.scala:178).
- **Orden / concurrencia / ciclo de vida**: El Future se completa desde el hilo del scheduler (timeout) o del actor que responde. El destino puede responder antes de que `ask` retorne. Un solo completado (tryComplete): gana la primera de respuesta/timeout.
- **Dependencias**: filas: AC-ASK (PromiseActorRef); AC-REF; AT-REF-004; AT-EVT-002. Externas: Scala Future/Promise y TimeoutException de la stdlib; PromiseActorRef del clasico.
- **Equivalente Rust propuesto**: async fn ask<Req,Res>(target:&dyn RecipientRef<Req>, make:impl FnOnce(ActorRef<Res>)->Req, timeout:Duration)->Result<Res,AskError>; AskError::{Timeout(String), RecipientTerminated(String), InvalidTimeout(String)}. **Diferencias observables**: Rust async: el Future es un future tokio, no un Scala Future; `?` no existe (se descarta el alias simbolico, no el comportamiento). Excepciones -> Result. El texto de timeout debe conservar el prefijo "Ask timed out on".
- **Aserciones**:
  - A1. ask a actor vivo completa con "foo" (Foo("bar", _) responde "foo") — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/AskSpec.scala:84-88
  - A2. `?` produce el mismo resultado que `ask` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/AskSpec.scala:90-94
  - A3. ask a actor ya detenido: Future fallido con TimeoutException cuyo mensaje incluye "had already been terminated." — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/AskSpec.scala:71-82
  - A4. ask a ref inexistente (/foo/bar resuelto): TimeoutException "had already been terminated" y el mensaje Foo("bar",_) aparece como DeadLetter — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/AskSpec.scala:109-130
  - A5. sin respuesta en 10ms: TimeoutException con mensaje que empieza con "Ask timed out on" y el mensaje no manejado se publica como UnhandledMessage — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/AskSpec.scala:96-107
  - A6. tras el timeout el ref temporal termina y una respuesta tardia se publica como DeadLetter con recipient != deadLetters — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/AskSpec.scala:164-189
  - A7. timeout <= 0 -> IllegalArgumentException "Timeout length must be positive, question not sent to [<destino>]" (no afirmada por spec; derivada del codigo actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/AskPattern.scala:152-157)
  - A8. texto completo del timeout "...A typical reason for `AskTimeoutException`..." (no afirmada por spec salvo el prefijo; derivada de actor/src/main/scala/org/apache/pekko/pattern/AskSupport.scala:729-735)
- **Oraculo**: Comparable contra jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh: escenarios ask vivo, ask a detenido, timeout (comparar clase y mensaje textual), respuesta tardia y DeadLetter.recipient. Procedencia: jar oficial.
- **Decisiones**: Q-T-ASK-1; Q-X-9

## AT-ASK-002: askWithStatus y StatusReply (public)

- **Contrato observable**: Contractual: `askWithStatus` aplana `StatusReply`: `Success(v)` completa con `v`; `Error(ex)` falla el `Future` con `ex` (`ErrorMessage(texto)` para `error(String)`, la excepción dada para `error(Throwable)`); `StatusReply.success/error/ack`, `getValue/getError/isError/isSuccess`; `toString` `"Success(v)"` / `"Error(<msg>)"`; un `StatusReply` con `status == null` lanza `InvalidMessageException("[null] is not an allowed status")` al construirse; `getError` sobre un éxito lanza `IllegalArgumentException("Expected reply to be a failure, but was a success")`; `getValue` sobre un error **relanza** la excepción de error (`ErrorMessage` o la original). `StatusReply.Ack` = `success(Done)`. Igualdad y `hashCode` por el `Try` interno.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/AskPattern.scala:135-137; actor/src/main/scala/org/apache/pekko/pattern/StatusReply.scala:30-116 (clase, factories, ErrorMessage, extractores), 173-183 (flattenStatusFuture)
- **Precondiciones**: ver AT-ASK-001.
- **Comportamiento**: `Success(v)` -> `Ok(v)`; `Error(ex)` -> `Err(ex)`; un éxito de otro tipo dentro del `Future` -> `IllegalArgumentException("Unexpected status reply success value: <v>")` (StatusReply.scala:179-180, inalcanzable con tipos estáticos); los fallos del propio `ask` (timeout) pasan intactos (182). `ErrorMessage` es `RuntimeException` sin stacktrace cuyo `toString` es el texto. Los extractores `Success.unapply`/`Error.unapply` son seguros ante `null`: `null` no coincide con ninguno.
- **Errores**: ver arriba; `InvalidMessageException` en la construcción con `status == null` (38-40), `IllegalArgumentException` en `getError` de un éxito.
- **Orden / concurrencia / ciclo de vida**: `flattenStatusFuture` usa el contexto parasitario: la transformación corre en el hilo que completa el `Future`.
- **Dependencias**: AC-ASK; AC-UTIL (Done); AT-ASK-001. Externas: Scala Future/Promise y TimeoutException de la stdlib; PromiseActorRef del clasico.
- **Equivalente Rust propuesto**: `enum StatusReply<T> { Success(T), Error(StatusError) }`; `StatusError::Message(String) | Custom(Box<dyn Error + Send + Sync>)`; `StatusReply::ack() -> StatusReply<Done>`; `ask_with_status()` que devuelve `Result<T, StatusError>`. **Se preserva**: aplanado, tipo del error (mensaje vs causa original), `getValue`/`getError` fallidos como `Err`/`Option`. **Diferencias/límites**: Rust no tiene excepciones; la identidad "es una instancia de ErrorMessage" se conserva con la variante `StatusError::Message`; `null` no existe (la rama `InvalidMessageException` desaparece).
- **Aserciones**:
  - A1. `Success("goodie")` entrega "goodie" — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/AskSpec.scala:267-272
  - A2. `error("boom")` falla el Future con `StatusReply.ErrorMessage` y `getMessage == "boom"` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/AskSpec.scala:273-279
  - A3. `error(TestException("boom"))` falla con esa clase de excepción y mensaje "boom" — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/AskSpec.scala:281-287
  - A4. (corrección de la auditoría 2026-10-31) `StatusReply` con `null`: el spec clásico **solo** afirma que los extractores no coinciden con `null` (`(null: StatusReply[_]) match { case Success(_) => fail() ... }`), no que la construcción lance `InvalidMessageException`; ese texto/tipo sigue siendo "no afirmada por spec" (StatusReply.scala:38-40) — actor-tests/src/test/scala/org/apache/pekko/pattern/StatusReplySpec.scala:46-52
  - A5. `toString` `Success(v)`/`Error(msg)` — no afirmada por spec (StatusReply.scala:65-68); caracterización propuesta: `StatusReply.success("x").toString == "Success(x)"` y `error("m").toString == "Error(m)"`
  - A6. extractores `Success(v)`, `Ack` y `Error(..)` coinciden (con texto y con excepción) — actor-tests/src/test/scala/org/apache/pekko/pattern/StatusReplySpec.scala:31-66
  - A7. `flattenStatusFuture`: `Success("woho")` -> "woho"; `Ack` -> `Done`; `Error("boo")` falla con `ErrorMessage("boo")`; `Error(TestException)` falla con esa excepción — actor-tests/src/test/scala/org/apache/pekko/pattern/StatusReplySpec.scala:67-77
  - A8. API Java: `isSuccess/isError/getValue` de un éxito; `getError` de un éxito lanza `IllegalArgumentException`; `getValue` de `error("boho")` lanza `StatusReply.ErrorMessage`; `getValue` de `error(TestException)` relanza `TestException`; `getError().getMessage()` conserva el texto — actor-tests/src/test/java/org/apache/pekko/pattern/StatusReplyTest.java:41-73
  - A9. `askWithStatus` clásico (misma semántica de aplanado): éxito, `ErrorMessage` para texto y excepción original — actor-tests/src/test/scala/org/apache/pekko/pattern/StatusReplySpec.scala:79-107 y actor-tests/src/test/java/org/apache/pekko/pattern/StatusReplyTest.java:75-115 (frontera: pertenece a AC-ASK; se referencia como respaldo del aplanado)
- **Validación prevista**: A1-A3 portados 1:1 (tipado); A6-A9 como pruebas del tipo `StatusReply` y de `ask_with_status` (los de `actor-tests` son de la unidad AC-ASK y se reutilizan sin duplicarlos); A4/A5 caracterización. Diferencial con el jar oficial (`StatusReply` está en pekko-actor, mismo jar de runtime): `toString`, `getError` de un éxito.
- **Decisiones**: Q-X-9 (Done); Q-T-ASK-1 (taxonomía de errores)

## AT-ASK-003: Status.Failure clasico como respuesta falla el Future (public)

- **Contrato observable**: Contractual por spec: si el destino responde al ref temporal con classic Status.Failure(ex) el Future falla con ex; Status.Success(r) completa con r (PromiseActorRef clasico).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/AskPattern.scala:160-162; actor/src/main/scala/org/apache/pekko/pattern/AskSupport.scala:615-625
- **Condiciones**: Normal: Failure(f) -> Future fallido con f identico (=== ex). Se requiere ActorRef[AnyRef] para poder enviar el mensaje clasico; en Rust solo existe si el mensaje es dinamico (Q-X-3).
- **Orden / concurrencia / ciclo de vida**: Igual que AT-ASK-001.
- **Dependencias**: filas: AC-ASK; AT-ASK-001. Externas: Scala Future/Promise y TimeoutException de la stdlib; PromiseActorRef del clasico.
- **Equivalente Rust propuesto**: Si se adopta Q-X-8(a): el ref temporal acepta el mensaje de fallo clasico; en (b) no aplica salvo capa de compatibilidad. **Diferencias observables**: Solo observable con mensajes dinamicos (Q-X-3); con mensajes estaticamente tipados la rama es inalcanzable.
- **Aserciones**:
  - A1. respuesta Status.Failure(ex) -> typedLegacy.ask(...).failed.futureValue === ex — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/AskSpec.scala:191-216
  - A2. Status.Success(r) -> Success(r) (no afirmada por spec; derivada de actor/src/main/scala/org/apache/pekko/pattern/AskSupport.scala:621)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh usando un actor clasico (jshell). Procedencia: jar oficial.
- **Decisiones**: Q-X-3; Q-X-8

## AT-ASK-004: ask solo sobre InternalRecipientRef (internal-accidental)

- **Contrato observable**: Accidental: un RecipientRef que no es InternalRecipientRef lanza IllegalStateException "Only expect references to be RecipientRef, ActorRefAdapter or ActorSystemAdapter until native system is implemented: <clase>" (al llamar, no en el Future).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/AskPattern.scala:117-127; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/InternalRecipientRef.scala
- **Condiciones**: Error sincrono (throw) con el texto indicado; no hay efecto secundario porque no se crea ref temporal.
- **Orden / concurrencia / ciclo de vida**: n/a: error sincrono previo a cualquier mensaje.
- **Dependencias**: filas: AT-REF-004. Externas: Scala Future/Promise y TimeoutException de la stdlib; PromiseActorRef del clasico.
- **Equivalente Rust propuesto**: trait RecipientRef<T> con ask() como metodo provisto: no hay restriccion; Q-T-ASK-2 decide si se conserva la IllegalStateException. **Diferencias observables**: Se propone no portar la restriccion (limitacion transitoria declarada por Pekko); decision Q-T-ASK-2.
- **Aserciones**:
  - A1. no afirmada por spec; derivada del codigo actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/AskPattern.scala:118-127
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh solo con un RecipientRef propio (jshell): difiere el mensaje con la clase. Poca utilidad; procedencia: jar oficial.
- **Decisiones**: Q-T-ASK-2

## AT-RCP-001: Receptionist (extension) y seleccion local/cluster (public)

- **Contrato observable**: Contractual: Receptionist es una Extension cargada al inicio (library-extensions en actor-typed/src/main/resources/reference.conf:21); ReceptionistImpl elige el proveedor: con cluster carga por reflexion ClusterReceptionist (RuntimeException "ClusterReceptionist could not be loaded dynamically. Make sure you have 'pekko-cluster-typed' in the classpath." si falta), si no LocalReceptionist (nombre de actor "localReceptionist", no choca con el "receptionist" del ClusterClient); corre como actor de sistema con dispatcher interno; system.receptionist responde a Find desde el arranque.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/receptionist/Receptionist.scala:28-34,106-113; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/ReceptionistImpl.scala:27-50; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionist.scala:28-47; actor-typed/src/main/resources/reference.conf:20-21; docs/src/main/paradox/typed/actor-discovery.md:29-107
- **Condiciones**: Normal: ref = internalSystemActorOf(provider.behavior, provider.name, Props.empty.withDispatcherFromConfig(InternalDispatcherId)) (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/ReceptionistImpl.scala:46-49). hasCluster viene de ProviderSelectionType (classic). Errores: ver contrato. Efecto: el actor existe antes de cualquier uso porque es library-extension (eager).
- **Orden / concurrencia / ciclo de vida**: La extension se crea al arrancar el sistema (eager). El comportamiento corre en el dispatcher interno, serializado como cualquier actor.
- **Dependencias**: filas: AT-EXT-002; AC-SYS; AC-DSP. Externas: Ninguna externa; usa DeathWatch y la cola del actor del runtime.
- **Equivalente Rust propuesto**: trait ReceptionistBehaviorProvider{name(),behavior()}; struct LocalReceptionist implementa el trait; Receptionist::get(system)->ReceptionistRef; ClusterReceptionist queda fuera (extension point). **Diferencias observables**: Sin cluster-typed el proveedor es siempre local; el hook de seleccion se conserva como trait para extension futura.
- **Aserciones**:
  - A1. system.receptionist ! Find(ServiceKeyA, probe) responde un Listing para ServiceKeyA con serviceInstances == Set() — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionistSpec.scala:194-200
  - A2. se puede crear systemActorOf(Behaviors.ignore,"receptionist") sin conflicto de nombre — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionistSpec.scala:202-204
  - A3. con cluster y sin pekko-cluster-typed el arranque falla con RuntimeException (no afirmada por spec; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/ReceptionistImpl.scala:35-41)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh (receptionist local). Procedencia: jar oficial.
- **Decisiones**: Q-T-RCP-2; Q-X-2

## AT-RCP-002: ServiceKey[T](id) (public)

- **Contrato observable**: Contractual: clave tipada por id y nombre de clase del mensaje; DefaultServiceKey(id, typeName) es case class: igualdad por (id, typeName), toString "ServiceKey[<typeName>](<id>)"; ServiceKey[T](id) usa classTag.runtimeClass.getName, create(clazz,id) usa clazz.getName. Por borrado de genericos dos claves con el mismo id y tipos que comparten clase runtime son iguales.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/receptionist/Receptionist.scala:36-95; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/ServiceKey.scala:27-43
- **Condiciones**: Igualdad estructural (id, typeName); el typeName es una String, no un tipo: ServiceKey[List[Int]]("k") == ServiceKey[List[String]]("k"). ServiceKey es @DoNotInherit.
- **Orden / concurrencia / ciclo de vida**: n/a: valor inmutable.
- **Dependencias**: filas: AT-SER-002. Externas: Ninguna externa; usa DeathWatch y la cola del actor del runtime.
- **Equivalente Rust propuesto**: struct ServiceKey<M>{id:String, type_name:&'static str, _m:PhantomData<fn(M)>}; PartialEq/Hash por (id,type_name) o (id,TypeId) segun Q-T-RCP-1. **Diferencias observables**: En Rust no existe el mismo borrado: type_name::<M>() distingue Vec<i32> de Vec<String>. Diferencia observable; decide Q-T-RCP-1.
- **Aserciones**:
  - A1. DefaultServiceKey(id,typeName) con igualdad de case class — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/ServiceKey.scala:41-43
  - A2. ServiceKey[T](id) usa el nombre de clase runtime; create(clazz,id) idem — actor-typed/src/main/scala/org/apache/pekko/actor/typed/receptionist/Receptionist.scala:41-48
  - A3. claves de distinto tipo no comparten Listing: Listing(ServiceKeyA,...) vs Listing(ServiceKeyB,...) se reciben por separado — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionistSpec.scala:172-185
  - A4. el spec de API solo comprueba compilacion — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/receptionist/ReceptionistApiSpec.scala (sin aserciones sobre igualdad)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh: igualdad/hash/toString de claves con tipos genericos (jshell). Procedencia: jar oficial.
- **Decisiones**: Q-T-RCP-1

## AT-RCP-003: Register / Registered (public)

- **Contrato observable**: Contractual: Register(key, ref[, replyTo]) registra la instancia bajo la clave; si hay replyTo recibe Registered(key, ref) ANTES de que los suscriptores reciban el Listing actualizado; el receptionist observa (watchWith) al actor solo la primera vez que aparece (si ya tiene algun servicio no se vuelve a observar); los suscriptores de la clave reciben Listing nuevo siempre (tambien si el registro es repetido).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/receptionist/Receptionist.scala:115-212; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionist.scala:157-193; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/ReceptionistMessages.scala:29-57
- **Condiciones**: Normal: ver contrato. Limites: registrar un actor ya terminado dispara watchWith inmediato -> RegisteredActorTerminated (efecto de AC-DW). Errores: ninguno. Efectos: log debug2 "Actor was registered: {} {}" (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionist.scala:186). La respuesta Registered se envia antes de updateServices (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionist.scala:189-193).
- **Orden / concurrencia / ciclo de vida**: Orden fijo dentro de un mismo mensaje: (1) watchWith si corresponde, (2) Registered al replyTo, (3) Listing a suscriptores. El estado es un actor: serializa comandos.
- **Dependencias**: filas: AT-CTX-006; AT-RCP-006; AT-RCP-005.2. Externas: Ninguna externa; usa DeathWatch y la cola del actor del runtime.
- **Equivalente Rust propuesto**: enum ReceptionistCommand<M>{Register{key,service,reply_to:Option<ActorRef<Registered>>}}; el estado interno: mapa clave->conjunto de refs y ref->conjunto de claves. **Diferencias observables**: Ninguna esperada si Q-X-4 mantiene un actor por receptionist.
- **Aserciones**:
  - A1. Register(ServiceKeyA, serviceA, probe) -> probe recibe Registered(ServiceKeyA, serviceA) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionistSpec.scala:58-75
  - A2. dos servicios bajo la misma clave -> Find devuelve Listing(Set(a1,a2)) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionistSpec.scala:248-262 (comportamiento sincrono con BehaviorTestKit)
  - A3. un actor registrado bajo A y B aparece en ambos Find — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionistSpec.scala:70-79
  - A4. registrar bajo dos claves y suscribirse a ambas entrega un Listing por clave — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionistSpec.scala:172-185
  - A5. ask funciona: receptionist.ask(Register(key, svc, _)) -> Registered — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionistSpec.scala:187-192
  - A6. re-registro tras Deregister vuelve a notificar — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionistSpec.scala:127-143
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh (receptionist local: orden Registered/Listing con probes). Procedencia: jar oficial.
- **Decisiones**: Q-T-RCP-3

## AT-RCP-004: Deregister / Deregistered (public)

- **Contrato observable**: Contractual: Deregister(key, ref[, replyTo]) quita la instancia SOLO de esa clave (no de las demas claves del actor); si el actor no tenia ningun servicio registrado es no-op sin respuesta; si queda sin claves se hace unwatch; replyTo recibe Deregistered(key, ref); los suscriptores reciben Listing actualizado; re-registro posible.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/receptionist/Receptionist.scala:214-298; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionist.scala:195-218
- **Condiciones**: Normal: ver contrato. Limites: si servicesForActor.isEmpty -> Behaviors.same sin notificar ni responder (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionist.scala:196-199); si el actor tiene otras claves pero no `key`, igual se responde Deregistered y se notifica Listing sin cambios (201-216). Efectos: unwatch cuando (servicesForActor - key).isEmpty (202-203) y otra vez dentro de updateServices (213-214).
- **Orden / concurrencia / ciclo de vida**: Deregistered al replyTo se envia antes del Listing a suscriptores.
- **Dependencias**: filas: AT-RCP-003. Externas: Ninguna externa; usa DeathWatch y la cola del actor del runtime.
- **Equivalente Rust propuesto**: ReceptionistCommand::Deregister{key,service,reply_to}; misma estructura de estado que Register. **Diferencias observables**: Ninguna si se conserva el orden; el doble unwatch es idempotente.
- **Aserciones**:
  - A1. Deregister(A, svc) notifica Listing(A, Set.empty) al suscriptor — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionistSpec.scala:91-101
  - A2. desregistrar por clave no afecta otras claves del mismo actor: Find(B) sigue devolviendo svc — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionistSpec.scala:103-125
  - A3. tras Deregister se puede volver a Register y el suscriptor recibe Listing(Set(svc)) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionistSpec.scala:127-143
  - A4. Deregister de actor sin servicios no responde ni notifica (no afirmada por spec; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionist.scala:196-199)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-T-RCP-3

## AT-RCP-005.1: Subscribe / Listing y Find: Find (public)

- **Contrato observable**: Contractual: Find(key, replyTo) responde una vez con Listing(key, instancias actuales, allServiceInstances == mismas, servicesWereAddedOrRemoved = true); sin registros el conjunto es vacio; no crea suscripcion.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/receptionist/Receptionist.scala:300-462; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionist.scala:162-236; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/ReceptionistMessages.scala:72-103
- **Condiciones**: Sin efectos secundarios: no modifica estado (Behaviors.same, actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionist.scala:220-222). replyWithListing construye Listing(key, l, l, true) (178-181).
- **Orden / concurrencia / ciclo de vida**: Respuesta inmediata, sin orden respecto de otros Find de otros actores mas alla del orden de mailbox.
- **Dependencias**: filas: AT-RCP-002; AT-RCP-003. Externas: Ninguna externa; usa DeathWatch y la cola del actor del runtime.
- **Equivalente Rust propuesto**: enum ReceptionistCommand::{Find,Subscribe}; struct Listing<M>{key,service_instances,all_service_instances,services_were_added_or_removed}. **Diferencias observables**: Ninguna esperada.
- **Aserciones**:
  - A1. Find(ServiceKeyA) tras registrar A devuelve Listing(ServiceKeyA, Set(serviceA, serviceC)) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionistSpec.scala:76-79
  - A2. Find sobre clave sin registros devuelve conjunto vacio con isForKey(key) == true — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionistSpec.scala:194-200
  - A3. Find no genera efectos en BehaviorTestKit (retrieveAllEffects == Nil) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionistSpec.scala:224-227
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-T-RCP-1

## AT-RCP-005.2: Subscribe / Listing y Find: Subscribe y Listing inicial/incremental (public)

- **Contrato observable**: Contractual: Subscribe(key, subscriber) responde de inmediato el Listing actual y despues un Listing en cada cambio del conjunto de la clave (alta, baja explicita o terminacion del servicio); el receptionist observa al suscriptor (watchWith) solo la primera vez que aparece y lo elimina al terminar sin notificar; no existe Unsubscribe en el protocolo.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/receptionist/Receptionist.scala:300-462; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionist.scala:162-236; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/ReceptionistMessages.scala:72-103
- **Condiciones**: El suscriptor es ActorRef[Listing]; una subscripcion a dos claves recibe un Listing por clave. Terminacion del suscriptor: SubscriberTerminated -> subscriberRemoved (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionist.scala:248-253). Subscribe repetido de la misma clave y actor es idempotente en el conjunto pero reenvia el Listing inicial (224-231).
- **Orden / concurrencia / ciclo de vida**: Listing inicial se envia antes de registrar la suscripcion (229 vs 231): no puede haber un Listing previo duplicado. Notificacion a suscriptores en el mismo mensaje que el cambio, tras el Registered/Deregistered.
- **Dependencias**: filas: AT-RCP-002; AT-RCP-003. Externas: Ninguna externa; usa DeathWatch y la cola del actor del runtime.
- **Equivalente Rust propuesto**: enum ReceptionistCommand::{Find,Subscribe}; struct Listing<M>{key,service_instances,all_service_instances,services_were_added_or_removed}. **Diferencias observables**: Ninguna esperada.
- **Aserciones**:
  - A1. Subscribe devuelve Listing(Set.empty) inicial y despues uno por alta — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionistSpec.scala:145-158
  - A2. al detenerse un servicio el suscriptor recibe el Listing sin el — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionistSpec.scala:166-169
  - A3. un mismo suscriptor a dos claves recibe Listing(A,{}) y Listing(B,{}) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionistSpec.scala:172-178
  - A4. un suscriptor terminado se elimina sin notificar (no afirmada por spec; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionist.scala:248-253)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-T-RCP-1

## AT-RCP-005.3: Subscribe / Listing y Find: Listing: accesores y chequeo de clave (public)

- **Contrato observable**: Contractual: Listing expone serviceInstances(key), allServiceInstances(key), servicesWereAddedOrRemoved, isForKey y getters Java; Registered/Deregistered exponen serviceInstance(key); usar una clave distinta lanza IllegalArgumentException "Wrong key [<k>] used, must use listing key [<kl>]"; en el receptionist local allServiceInstances == serviceInstances y servicesWereAddedOrRemoved == true siempre; Listing(key, set) usa los mismos valores (apply de 2 args).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/receptionist/Receptionist.scala:300-462; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionist.scala:162-236; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/ReceptionistMessages.scala:72-103
- **Condiciones**: allServiceInstances/servicesWereAddedOrRemoved solo difieren en cluster (alcanzables vs no alcanzables); el router de grupo los usa para caer a inalcanzables (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/GroupRouterImpl.scala:139-146).
- **Orden / concurrencia / ciclo de vida**: n/a: valores inmutables entregados por mensaje.
- **Dependencias**: filas: AT-RCP-002; AT-RCP-003. Externas: Ninguna externa; usa DeathWatch y la cola del actor del runtime.
- **Equivalente Rust propuesto**: enum ReceptionistCommand::{Find,Subscribe}; struct Listing<M>{key,service_instances,all_service_instances,services_were_added_or_removed}. **Diferencias observables**: Rust: Result<_,WrongKey> en vez de IllegalArgumentException; mensaje textual a conservar.
- **Aserciones**:
  - A1. Listing(key, Set(refs)) == Listing recibido (igualdad estructural con allServiceInstances = serviceInstances y added/removed = true) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionistSpec.scala:77,85
  - A2. IllegalArgumentException "Wrong key [..] used, must use listing key [..]" — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/ReceptionistMessages.scala:50-52,63-65,84-86,93-95 (no afirmada por spec)
  - A3. Listing.apply de 4 args conserva allServiceInstances y flag — actor-typed/src/main/scala/org/apache/pekko/actor/typed/receptionist/Receptionist.scala:435-444 (no afirmada por spec)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-T-RCP-1

## AT-RCP-006: Desregistro automatico al terminar (public)

- **Contrato observable**: Contractual: cuando un actor registrado termina, el receptionist lo quita de todas sus claves y notifica a los suscriptores de cada una (un Listing por clave); cuando un suscriptor termina se lo quita sin notificar; si el actor ya se habia desregistrado antes de procesar la terminacion no hay efecto.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionist.scala:238-254
- **Condiciones**: keys vacias -> Behaviors.same (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionist.scala:241-243). Si no, updateServices(keys, serviceInstanceRemoved(ref)) notifica una vez por clave (245-246).
- **Orden / concurrencia / ciclo de vida**: La notificacion llega de forma asincrona respecto del Stop del actor (el test usa eventually). Dependencia: orden de entrega de Terminated (AC-DW).
- **Dependencias**: filas: AC-DW; AT-RCP-003. Externas: Ninguna externa; usa DeathWatch y la cola del actor del runtime.
- **Equivalente Rust propuesto**: Observacion de terminacion via watch_with(RegisteredActorTerminated) sobre el estado del receptionist. **Diferencias observables**: Ninguna si el DeathWatch Rust notifica exactamente una vez.
- **Aserciones**:
  - A1. tras detener serviceC (registrado en A y B) Find(A)=={serviceA} y Find(B)=={serviceB} (eventually) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionistSpec.scala:81-88
  - A2. el suscriptor recibe Listing sin la instancia detenida y luego Listing vacio — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionistSpec.scala:166-169
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: ninguna especifica (hereda Q-X-* citadas en notes)

## AT-RCP-007: ReceptionistSetup (reemplazo para tests) (public)

- **Contrato observable**: Contractual: ReceptionistSetup(createExtension) es un ExtensionSetup[Receptionist] para ActorSystemSetup que sustituye la implementacion de la extension (ver AT-EXT-003).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/receptionist/Receptionist.scala:463-477; actor-typed/src/main/scala/org/apache/pekko/actor/typed/Extensions.scala:170-187
- **Condiciones**: ReceptionistSetup(createExtension) extiende ExtensionSetup[Receptionist](Receptionist, createExtension) (RS:476-477). La busqueda del setup ocurre en createExtensionInstance (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:97-101).
- **Orden / concurrencia / ciclo de vida**: Se resuelve al crear la extension (eager, arranque).
- **Dependencias**: filas: AT-EXT-003. Externas: Ninguna externa; usa DeathWatch y la cola del actor del runtime.
- **Equivalente Rust propuesto**: ReceptionistSetup::new(|system| Receptionist) pasado al builder del sistema. **Diferencias observables**: Ninguna.
- **Aserciones**:
  - A1. no afirmada por spec; derivada del codigo actor-typed/src/main/scala/org/apache/pekko/actor/typed/receptionist/Receptionist.scala:476-477 y actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:97-101
- **Oraculo**: No aplica como golden: es un punto de configuracion; se verificaria en jshell que el setup reemplaza la extension. Procedencia: jar oficial.
- **Decisiones**: ninguna especifica (hereda Q-X-* citadas en notes)

## AT-RCP-008: Extractores ServiceKey.Listing / ServiceKey.Registered (public)

- **Contrato observable**: Contractual: key.Listing(refs) y key.Registered(ref) son extractores (Scala) que hacen coincidir solo si el mensaje es para esa clave (isForKey) y devuelven las instancias; los usan Topic y los routers para filtrar el Listing recibido via message adapter.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/receptionist/Receptionist.scala:77-95; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/pubsub/TopicImpl.scala:84-87; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/GroupRouterImpl.scala:88,135
- **Condiciones**: Si el Listing es para otra clave, unapply devuelve None; en TopicImpl un Listing de otra clave lanza IllegalArgumentException() sin mensaje (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/pubsub/TopicImpl.scala:86).
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: filas: AT-RCP-002; AT-RCP-005.3; AT-RT-004. Externas: Ninguna externa; usa DeathWatch y la cola del actor del runtime.
- **Equivalente Rust propuesto**: ServiceKey::match_listing(&Listing)->Option<Set<ActorRef<M>>> y match_registered. **Diferencias observables**: Ninguna.
- **Aserciones**:
  - A1. unapply devuelve Some(serviceInstances(key)) solo si isForKey(key) — actor-typed/src/main/scala/org/apache/pekko/actor/typed/receptionist/Receptionist.scala:81-85
  - A2. uso: el router de grupo recibe Listing para su clave y arma GroupRouterImpl — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/RoutersSpec.scala:161-184
- **Oraculo**: No aplica: es azucar sintactico; la semantica queda cubierta por AT-RCP-005.3.
- **Decisiones**: ninguna especifica (hereda Q-X-* citadas en notes)

## AT-RT-001: Routers.pool(poolSize)(behavior) (public)

- **Contrato observable**: Contractual: crea poolSize hijos anonimos con el mismo behavior (spawnAnonymous con routeeProps) y los observa; ruteo round robin por defecto; si un hijo termina sigue con el resto (log debug "Pool child stopped [<path>]"); si todos paran, el router para (log info "Last pool child stopped, stopping pool [<path>]"); poolSize < 1 lanza IllegalArgumentException "pool size must be positive, was <n>".
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Routers.scala:24-60; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/PoolRouterImpl.scala:29-123; docs/src/main/paradox/typed/routers.md:28-75
- **Condiciones**: Normal: ver contrato. Limites: si varios hijos paran a la vez children puede estar vacio al llegar el primer Terminated (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/PoolRouterImpl.scala:108-112), asi que el log por hijo no esta garantizado. Efectos: onRouteesChanged() tras crear y tras cada baja (91,115).
- **Orden / concurrencia / ciclo de vida**: Los hijos se crean en la construccion del router (init del AbstractBehavior), antes del primer mensaje. El Terminated llega por signal; el mensaje siguiente ya usa el conjunto actualizado.
- **Dependencias**: filas: AT-CTX-002; AT-CTX-005; AT-BEH-023. Externas: Scala ThreadLocalRandom (JDK); ConsistentHash de pekko.routing (clasico).
- **Equivalente Rust propuesto**: fn pool<M>(size, behavior_factory) -> PoolRouter<M>; el Behavior de las rutas debe ser reusable (Arc<dyn Fn>) segun AT-BEH-023. **Diferencias observables**: Ninguna esperada.
- **Aserciones**:
  - A1. 4 hijos "started 0..3" y 9 mensajes reparten entre recipients {0,1,2,3} — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/RoutersSpec.scala:48-82
  - A2. tras parar 2 hijos (2 logs debug "Pool child stopped") los 5 mensajes llegan a los restantes — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/RoutersSpec.scala:84-110
  - A3. si todos paran, log info "Last pool child stopped, stopping pool" y el pool termina — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/RoutersSpec.scala:112-124
  - A4. poolSize < 1 -> IllegalArgumentException (no afirmada por spec; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/PoolRouterImpl.scala:38)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh (conteo de hijos, logs). Procedencia: jar oficial.
- **Decisiones**: Q-T-RT-3

## AT-RT-002.1: Estrategia de ruteo: Round robin (por defecto de pool) (public)

- **Contrato observable**: Contractual: orden estable por (path sin direccion, direccion) ; cursor nextIdx avanza circularmente; al cambiar el conjunto el cursor se corrige: si nextIdx > primer indice distinto se decrementa en 1 (no salta ni repite al quitar un routee); conjunto nuevo completamente distinto empieza por el primero ordenado.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/RoutingLogic.scala:52-85; actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Routers.scala:60-193; docs/src/main/paradox/typed/routers.md:99-150
- **Condiciones**: Sin routees: round robin falla por NPE/ArrayIndexOutOfBounds si nunca recibio routeesUpdated (currentRoutees=null, actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/RoutingLogic.scala:54-62) y random por IllegalArgumentException; el unico caso afirmado por spec es consistent hashing.
- **Orden / concurrencia / ciclo de vida**: Estado mutable no thread-safe: lo accede solo el actor router (un hilo a la vez).
- **Dependencias**: filas: AT-RT-001; AT-RT-004. Externas: Scala ThreadLocalRandom (JDK); ConsistentHash de pekko.routing (clasico).
- **Equivalente Rust propuesto**: trait RoutingLogic<M>{select_routee(&M)->Option<ActorRef<M>>; routees_updated(set)}; RoundRobin/Random/ConsistentHash. **Diferencias observables**: Rust: select_routee devuelve Option; la funcion de hash consistente puede no ser byte-compatible (Q-T-RT-1). La distribucion random no es reproducible.
- **Aserciones**:
  - A1. round robin A,B,C,A — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/routing/RoutingLogicSpec.scala:32-45
  - A2. al quitar B tras A,B el siguiente es C (no salta) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/routing/RoutingLogicSpec.scala:47-61
  - A3. al quitar el ultimo cae a A — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/routing/RoutingLogicSpec.scala:63-75
  - A4. varios removidos: mueve al siguiente — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/routing/RoutingLogicSpec.scala:77-93
  - A5. varios removidos con envoltura — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/routing/RoutingLogicSpec.scala:95-112
  - A6. conjunto nuevo: empieza por el primero — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/routing/RoutingLogicSpec.scala:114-133
- **Oraculo**: Round robin y errores comparables con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh; el anillo de hashing es comparable solo si ConsistentHash (classic, AC) se porta identico. Procedencia: jar oficial.
- **Decisiones**: Q-T-RT-1;Q-T-RT-2

## AT-RT-002.2: Estrategia de ruteo: Random (por defecto de group) (public)

- **Contrato observable**: Contractual: elige uniformemente (ThreadLocalRandom) entre los routees vigentes; sin routees nextInt(0) lanza IllegalArgumentException (no alcanzable: el group nunca selecciona si esta vacio).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/RoutingLogic.scala:87-99; actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Routers.scala:60-193; docs/src/main/paradox/typed/routers.md:99-150
- **Condiciones**: Sin routees: round robin falla por NPE/ArrayIndexOutOfBounds si nunca recibio routeesUpdated (currentRoutees=null, actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/RoutingLogic.scala:54-62) y random por IllegalArgumentException; el unico caso afirmado por spec es consistent hashing.
- **Orden / concurrencia / ciclo de vida**: Estado mutable no thread-safe: lo accede solo el actor router (un hilo a la vez).
- **Dependencias**: filas: AT-RT-001; AT-RT-004. Externas: Scala ThreadLocalRandom (JDK); ConsistentHash de pekko.routing (clasico).
- **Equivalente Rust propuesto**: trait RoutingLogic<M>{select_routee(&M)->Option<ActorRef<M>>; routees_updated(set)}; RoundRobin/Random/ConsistentHash. **Diferencias observables**: Rust: select_routee devuelve Option; la funcion de hash consistente puede no ser byte-compatible (Q-T-RT-1). La distribucion random no es reproducible.
- **Aserciones**:
  - A1. la seleccion siempre pertenece al conjunto — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/routing/RoutingLogicSpec.scala:136-154 (la aleatoriedad no se afirma)
- **Oraculo**: Round robin y errores comparables con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh; el anillo de hashing es comparable solo si ConsistentHash (classic, AC) se porta identico. Procedencia: jar oficial.
- **Decisiones**: Q-T-RT-1;Q-T-RT-2

## AT-RT-002.3: Estrategia de ruteo: Consistent hashing (public)

- **Contrato observable**: Contractual: anillo con virtualNodesFactor nodos virtuales por routee (require > 0: IllegalArgumentException "requirement failed: virtualNodesFactor has to be a positive integer"); mapping T=>String da la clave; sin routees selectRoutee lanza IllegalStateException "Can't get node for [<k>] from an empty node ring"; los routees se identifican por path completo (toStringWithAddress(baseAddress) si el path es local); el mismo mensaje va siempre al mismo routee mientras no cambie el conjunto; solo se mueven los routees afectados al agregar/quitar. El group con consistent hashing fuerza preferLocalRoutees = false.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/RoutingLogic.scala:101-122; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/GroupRouterImpl.scala:54-63; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/PoolRouterImpl.scala:54-60; actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Routers.scala:60-193; docs/src/main/paradox/typed/routers.md:99-150
- **Condiciones**: Sin routees: round robin falla por NPE/ArrayIndexOutOfBounds si nunca recibio routeesUpdated (currentRoutees=null, actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/RoutingLogic.scala:54-62) y random por IllegalArgumentException; el unico caso afirmado por spec es consistent hashing.
- **Orden / concurrencia / ciclo de vida**: Estado mutable no thread-safe: lo accede solo el actor router (un hilo a la vez).
- **Dependencias**: filas: AT-RT-001; AT-RT-004. Externas: Scala ThreadLocalRandom (JDK); ConsistentHash de pekko.routing (clasico).
- **Equivalente Rust propuesto**: trait RoutingLogic<M>{select_routee(&M)->Option<ActorRef<M>>; routees_updated(set)}; RoundRobin/Random/ConsistentHash. **Diferencias observables**: Rust: select_routee devuelve Option; la funcion de hash consistente puede no ser byte-compatible (Q-T-RT-1). La distribucion random no es reproducible.
- **Aserciones**:
  - A1. virtualNodesFactor < 1 -> IllegalArgumentException con mensaje exacto — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/routing/RoutingLogicSpec.scala:163-168
  - A2. sin routees -> IllegalStateException "Can't get node for [..] from an empty node ring" — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/routing/RoutingLogicSpec.scala:170-178
  - A3. hash consistente (factor 1 y 13) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/routing/RoutingLogicSpec.scala:180-187
  - A4. agregar/quitar varios mantiene consistencia — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/routing/RoutingLogicSpec.scala:188-234
- **Oraculo**: Round robin y errores comparables con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh; el anillo de hashing es comparable solo si ConsistentHash (classic, AC) se porta identico. Procedencia: jar oficial.
- **Decisiones**: Q-T-RT-1;Q-T-RT-2

## AT-RT-003: Pool: withBroadcastPredicate, withPoolSize, withRouteeProps (public)

- **Contrato observable**: Contractual: los mensajes que cumplen el predicado van a TODOS los routees (ctx.children), el resto a uno por la logica; el predicado por defecto es anyToFalse (identidad comparada por referencia para evitar evaluarlo); withRouteeProps pasa Props (dispatcher) a spawnAnonymous; withPoolSize reemplaza el tamano; los withX devuelven un builder nuevo (copy).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Routers.scala:176-193; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/PoolRouterImpl.scala:29-71,98-105; actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/routing/PoolRouterSpec.scala:41-62
- **Condiciones**: Broadcast: for each child !(msg) (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/PoolRouterImpl.scala:99-101) sin pasar por la logica de ruteo.
- **Orden / concurrencia / ciclo de vida**: El broadcast se envia en el orden de ctx.children; sin garantia de orden entre routees.
- **Dependencias**: filas: AT-DSP-001; AT-DSP-002. Externas: Scala ThreadLocalRandom (JDK); ConsistentHash de pekko.routing (clasico).
- **Equivalente Rust propuesto**: PoolRouter::with_broadcast_predicate(Fn(&M)->bool), with_pool_size, with_routee_props(Props). **Diferencias observables**: Ninguna.
- **Aserciones**:
  - A1. BCast llega a 4 routees distintos y luego ReplyWithAck a uno solo — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/RoutersSpec.scala:126-155
  - A2. routees usan el dispatcher por defecto si no se especifica — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/routing/PoolRouterSpec.scala:43-50
  - A3. withRouteeProps(DispatcherSelector.blocking()) ubica routees en pekko.actor.default-blocking-io-dispatcher — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/routing/PoolRouterSpec.scala:52-61
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-T-RT-3

## AT-RT-004: Routers.group(serviceKey) (public)

- **Contrato observable**: Contractual: al iniciar se suscribe al Receptionist para la clave; hasta el primer Listing guarda hasta 10000 mensajes (StashBuffer capacity 10000) y si se llena publica Dropped(msg, "Stash is full in group router for [<key>]") al event stream sin fallar; con el primer Listing pasa al router activo y desencola el stash; sin routees publica Dropped(msg, "No routees in group router for [<key>]"); acompana los cambios del conjunto; por defecto random.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Routers.scala:24-45; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/GroupRouterImpl.scala:32-156; docs/src/main/paradox/typed/routers.md:77-98
- **Condiciones**: Normal: ver contrato. Limites: un Listing vacio con allServiceInstances no vacio usa allServiceInstances (inalcanzables) en lugar de descartar (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/GroupRouterImpl.scala:137-144). Errores: ninguno (no falla con stash lleno). Efectos: log debug "Update from receptionist: [{}]" (136).
- **Orden / concurrencia / ciclo de vida**: Los mensajes previos al primer Listing se entregan en orden FIFO tras el cambio de comportamiento (unstashAll). El orden relativo con mensajes nuevos mantiene el del stash.
- **Dependencias**: filas: AT-RCP-005.2; AT-EVT-004; AT-BEH-023 (StashBuffer: AT-STASH). Externas: Scala ThreadLocalRandom (JDK); ConsistentHash de pekko.routing (clasico).
- **Equivalente Rust propuesto**: fn group<M>(key:ServiceKey<M>) -> GroupRouter<M>; internamente suscripcion a Listing y buffer acotado. **Diferencias observables**: Ninguna esperada; la fuente de Listing es el receptionist local.
- **Aserciones**:
  - A1. 4 routees registrados antes: 4 mensajes llegan a probe — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/RoutersSpec.scala:161-184
  - A2. sin routees se publica Dropped para cada mensaje (createDroppedMessageProbe) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/RoutersSpec.scala:186-198
  - A3. tras detener ref2 el group sigue entregando (con delays del receptionist) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/RoutersSpec.scala:200-244
  - A4. stash lleno (10000) publica Dropped "Stash is full in group router for [..]" (no afirmada por spec; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/GroupRouterImpl.scala:96-102)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh (Dropped, orden). Procedencia: jar oficial.
- **Decisiones**: Q-T-EVT-1;Q-T-RT-3

## AT-RT-005: Group: preferLocalRoutees y manejo de inalcanzables (public)

- **Contrato observable**: Contractual: con preferLocalRoutees usa solo los routees con direccion local si hay alguno, si no todos (GroupRouterHelper); si el Listing no tiene alcanzables usa allServiceInstances (inalcanzables); consistent hashing fuerza preferLocalRoutees=false.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/GroupRouterImpl.scala:44-52,111-118,134-147
- **Condiciones**: El test crea GroupRouterImpl directamente con routeesInitiallyEmpty=true y le envia un Listing a mano (actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/RoutersSpec.scala:246-278). Hay un typo de nombre duplicado "group-routing-4" en ambos tests.
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: filas: AT-RCP-005.3. Externas: Scala ThreadLocalRandom (JDK); ConsistentHash de pekko.routing (clasico).
- **Equivalente Rust propuesto**: GroupRouter::prefer_local_routees(bool). **Diferencias observables**: En local todo es alcanzable; la rama de inalcanzables solo se ejercita via Listing sintetico.
- **Aserciones**:
  - A1. con reachable={a}, unreachable={u} los mensajes "one","two" llegan solo a a — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/RoutersSpec.scala:246-259
  - A2. con reachable={} y all={u} los mensajes llegan a u — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/RoutersSpec.scala:261-278
  - A3. preferLocalRoutees filtra por address.hasLocalScope (no afirmada por spec; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/GroupRouterImpl.scala:112-117)
- **Oraculo**: No aplica sin cluster; la logica pura se verifica con Listing sinteticos igual que el spec.
- **Decisiones**: Q-T-RT-3

## AT-RT-006: Validacion e inmutabilidad de los builders de router (public)

- **Contrato observable**: Contractual: PoolRouterBuilder/GroupRouterBuilder son case classes inmutables (cada withX hace copy); PoolRouterBuilder con poolSize < 1 lanza IllegalArgumentException "pool size must be positive, was <n>" en construccion (tambien via withPoolSize); apply(ctx) instancia el router; el predicado de broadcast y la logica se resuelven al construir el behavior (logicFactory(system)).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/PoolRouterImpl.scala:29-71; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/GroupRouterImpl.scala:32-64; actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Routers.scala:24-193
- **Condiciones**: Error: IllegalArgumentException("pool size must be positive, was 0") ocurre al construir el builder, antes de spawnear. Notar que withPoolSize usa copy, que vuelve a ejecutar la validacion.
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: filas: AT-RT-001; AT-RT-003. Externas: Scala ThreadLocalRandom (JDK); ConsistentHash de pekko.routing (clasico).
- **Equivalente Rust propuesto**: builders con consumo por valor (self -> Self); validar tamano en with_pool_size. **Diferencias observables**: Rust: Result en el builder o panic documentado.
- **Aserciones**:
  - A1. poolSize < 1 -> IllegalArgumentException (no afirmada por spec; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/PoolRouterImpl.scala:38)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh (excepcion al construir). Procedencia: jar oficial.
- **Decisiones**: ninguna especifica (hereda Q-X-* citadas en notes)

## AT-ADP-001: Sistema clasico a typed: toTyped, ActorSystem.wrap; typed a clasico: toClassic (public)

- **Contrato observable**: Contractual: ClassicActorSystemOps.toTyped devuelve ActorSystem[Nothing] sobre el MISMO sistema clasico (AdapterExtension, una sola instancia por sistema); TypedActorSystemOps.toClassic devuelve sys.classicSystem; las extensiones clasicas (p.ej. SerializationExtension) se pueden usar con el sistema typed.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/adapter/package.scala:49-98; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorSystemAdapter.scala:143-176; actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorSystem.scala
- **Condiciones**: toTyped = AdapterExtension(sys).adapter (actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/adapter/package.scala:80) con ActorSystemAdapter creado una vez por ExtensionId clasico. SerializationExtension(typedSystem) no es null (actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpec.scala:243-246).
- **Orden / concurrencia / ciclo de vida**: La adaptacion es perezosa al primer toTyped pero idempotente; library-extensions tipadas se cargan en el arranque clasico via LoadTypedExtensions (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorSystemAdapter.scala:165-169).
- **Dependencias**: filas: AT-SYS-011; AC-SYS; AT-ADP-008. Externas: Runtime clasico completo (pekko-actor): ActorCell, supervision, ActorSystemImpl.
- **Equivalente Rust propuesto**: trait ClassicInterop{to_typed(&ClassicSystem)->TypedSystem; to_classic(&TypedSystem)->ClassicSystem} solo bajo Q-X-8(a). **Diferencias observables**: Con Q-X-8(b) solo existe como capa opcional de compatibilidad; con (a) es el mecanismo base.
- **Aserciones**:
  - A1. dos toTyped devuelven la misma instancia — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpec.scala:203-208
  - A2. toClassic de un Scheduler typed devuelve el scheduler clasico original (theSameInstanceAs) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpec.scala:237-241
  - A3. SerializationExtension(typedSystem) disponible — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpec.scala:243-245
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh (identidad de instancia). Procedencia: jar oficial.
- **Decisiones**: Q-X-8

## AT-ADP-002: spawn/spawnAnonymous de typed desde sistema o contexto clasico (public)

- **Contrato observable**: Contractual: ClassicActorSystemOps/ClassicActorContextOps.spawn(Anonymous) crean un actor typed bajo user (o bajo el contexto clasico) envolviendo el Behavior en supervise(...).onFailure(SupervisorStrategy.stop) y con rethrowTypedFailure=false; el estrategia por defecto es Stop (a diferencia del clasico) y se puede sobreescribir con Behaviors.supervise; un typed hijo de un padre clasico se detiene (PostStop) al lanzar y no se reinicia.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/adapter/package.scala:49-137; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorRefFactoryAdapter.scala:25-57; actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/Adapter.scala
- **Condiciones**: spawn valida Behavior.validateAsInitial(behavior) (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorRefFactoryAdapter.scala:50) y spawnAnonymous no. Un ConfigurationException de despliegue remoto se re-lanza como "Remote deployment not allowed for typed actors" (ver AT-ADP-007).
- **Orden / concurrencia / ciclo de vida**: La excepcion del Behavior llega al supervisor typed envuelto (Stop) y el actor recibe PostStop antes de Terminated para observadores.
- **Dependencias**: filas: AT-SUP-020; AC-CELL. Externas: Runtime clasico completo (pekko-actor): ActorCell, supervision, ActorSystemImpl.
- **Equivalente Rust propuesto**: ClassicCompat::spawn(behavior, props) con supervision Stop por defecto. **Diferencias observables**: Ninguna si el modelo de supervision conserva el default Stop.
- **Aserciones**:
  - A1. spawn desde sistema clasico: tras "throw" recibe PostStop, ningun mensaje mas y Terminated — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/coexistence/ClassicSupervisingTypedSpec.scala:74-82
  - A2. spawnAnonymous: idem — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/coexistence/ClassicSupervisingTypedSpec.scala:84-92
  - A3. con supervise(restart) recibe PreRestart — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/coexistence/ClassicSupervisingTypedSpec.scala:94-103
  - A4. spawn desde contexto clasico: tres variantes con el mismo default — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/coexistence/ClassicSupervisingTypedSpec.scala:105-140
  - A5. crear hijo typed desde padre clasico responde "ok" — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpec.scala:275-281
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-X-8;Q-T-ADP-3

## AT-ADP-003: PropsAdapter (typed como Props clasico) (public)

- **Contrato observable**: Contractual: PropsAdapter(behavior, props) produce un Props clasico (Props(new ActorAdapter(behavior(), rethrowTypedFailure))) con: dispatcher segun DispatcherSelector (default/fromConfig/sameAsParent; otro -> RuntimeException "Unsupported dispatcher selector: <x>"), mailbox segun MailboxSelector (default -> mailbox "pekko.actor.typed.default-mailbox"; bounded(n) -> prefijo BoundedCapacityPrefix+n; fromConfig(path)), Deploy.local siempre, y tags de ActorTags; el publico PropsAdapter(behavior, props) envuelve en supervise(stop).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/adapter/PropsAdapter.scala:22-37; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/PropsAdapter.scala:25-58
- **Condiciones**: tags vacios -> no se llama withActorTags (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/PropsAdapter.scala:53-55). Dispatcher: sameAsParent -> Deploy.DispatcherSameAsParent.
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: filas: AT-DSP-003; AC-CELL. Externas: Runtime clasico completo (pekko-actor): ActorCell, supervision, ActorSystemImpl.
- **Equivalente Rust propuesto**: fn to_classic_props(behavior, props) -> ClassicProps (solo en capa de compat). **Diferencias observables**: Idem Q-X-8.
- **Aserciones**:
  - A1. mailbox por defecto es "pekko.actor.typed.default-mailbox" — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/adpater/PropsAdapterSpec.scala:28-32
  - A2. actor creado con PropsAdapter para: PostStop al lanzar — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/coexistence/ClassicSupervisingTypedSpec.scala:152-165
  - A3. Deploy.local siempre (no afirmada por spec; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/PropsAdapter.scala:39,51)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh (props.mailbox). Procedencia: jar oficial.
- **Decisiones**: Q-X-8

## AT-ADP-004: Conversion de ActorRef, ActorContext y Scheduler (toClassic/toTyped) (public)

- **Contrato observable**: Contractual: ActorRef typed y clasico se convierten (ClassicActorRefOps.toTyped / implicit actorRefAdapter; TypedActorRefOps.toClassic / ActorRefAdapter.toClassic), ActorContext typed->clasico (ActorContextAdapter.toClassic), Scheduler typed<->clasico (SchedulerAdapter); ActorRefAdapter delega path/isLocal/provider/isTerminated al ref clasico; toClassic de un ref que no es adaptador lanza UnsupportedOperationException "Only adapted classic ActorRefs permissible (<ref> of class <clase>)"; ActorSystemAdapter se convierte en el guardian; tell(null) lanza InvalidMessageException "[null] is not an allowed message".
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/adapter/package.scala:139-196; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorRefAdapter.scala:28-82; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/SchedulerAdapter.scala:44-75
- **Condiciones**: sendSystemMessage mapea Create->IllegalStateException("WAT? No, seriously."), Terminate->classicRef.stop(), Watch/Unwatch/DeathWatchNotification a mensajes de sistema clasicos (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorRefAdapter.scala:70-81).
- **Orden / concurrencia / ciclo de vida**: Los mensajes de sistema se entregan por el canal de sistema del ref clasico.
- **Dependencias**: filas: AC-REF; AC-SCH. Externas: Runtime clasico completo (pekko-actor): ActorCell, supervision, ActorSystemImpl.
- **Equivalente Rust propuesto**: ActorRef<T>::from_classic / to_classic; Scheduler::to_classic. **Diferencias observables**: Conversion implicita no replicable; refPrefix = path.name (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorRefAdapter.scala:51).
- **Aserciones**:
  - A1. tell de typed a clasico entrega "ok" — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpec.scala:250-256
  - A2. tell(null) -> InvalidMessageException — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpec.scala:258-265
  - A3. mensaje de clasico a typed — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpec.scala:267-273
  - A4. convert Scheduler: clase SchedulerAdapter y toClassic identico — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpec.scala:237-241
  - A5. UnsupportedOperationException para refs no adaptados (no afirmada por spec; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorRefAdapter.scala:60-67)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-X-8

## AT-ADP-005: actorOf de hijos clasicos desde contexto typed; watch y stop cruzados (public)

- **Contrato observable**: Contractual: TypedActorContextOps.actorOf(props[,name]) crea hijos clasicos desde un actor typed; watch/unwatch/stop cruzados funcionan en ambos sentidos (ClassicActorContextOps.watch/unwatch/stop); un padre typed supervisa a un hijo clasico con la estrategia clasica por defecto (Restart ante Exception) y registra el fallo (error log); fallo en constructor (ActorInitializationException) lo detiene.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/adapter/package.scala:100-152; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:232-266; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorContextAdapter.scala
- **Condiciones**: Un hijo clasico que no es adaptador usa classicSupervisorDecider = defaultDecider.applyOrElse(exc, Restart) (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:57); un hijo typed (ActorAdapter) recibe Stop (258-259).
- **Orden / concurrencia / ciclo de vida**: Terminated llega al padre como signal; si el hijo fallo y era observado se convierte en ChildFailed (ver AT-ADP-006.4).
- **Dependencias**: filas: AT-SUP-021; AC-DW. Externas: Runtime clasico completo (pekko-actor): ActorCell, supervision, ActorSystemImpl.
- **Equivalente Rust propuesto**: ClassicCompat::actor_of desde typed; watch cruzado. **Diferencias observables**: Idem Q-X-8.
- **Aserciones**:
  - A1. actorOf de hijo clasico desde typed — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpec.scala:291-297
  - A2. actorOf typed via Props desde padre clasico — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpec.scala:283-289
  - A3. watch typed desde clasico recibe "terminated" — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpec.scala:299-306
  - A4. watch clasico desde typed — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpec.scala:308-315
  - A5. supervise hijo clasico desde padre typed (error log ThrowIt3) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpec.scala:317-330
  - A6. hijo clasico que lanza en constructor: ActorInitializationException y "terminated" — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpec.scala:332-343
  - A7. stop cruzado en ambos sentidos — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpec.scala:345-359
  - A8. typed supervisando clasico: postStop y luego preStart (restart) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/coexistence/TypedSupervisingClassicSpec.scala:60-75
  - A9. excepcion no manejada por supervisor typed se loguea (error "sad panda") — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpec.scala:361-369
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-X-8

## AT-ADP-006.1: ActorAdapter: Despacho de mensajes (aroundReceive) (internal-required)

- **Contrato observable**: Interno y estructural, pero con efectos observables por los tests de otros grupos: `aroundReceive` fija el hilo de actor, despacha y siempre (finally) libera el hilo y limpia el MDC. Traducción por tipo de mensaje, en este orden: (1) `classic.Terminated(ref)` -> `ChildFailed(ref, causa)` si `ref` está en `failures` (y borra la entrada) o `Terminated(ref)` si no; (2) `classic.ReceiveTimeout` -> el mensaje de receive-timeout configurado en el contexto; (3) `AdaptMessage` (ask/pipeToSelf/spawnMessageAdapter) -> se ejecuta `adapt()` de forma segura y su resultado se enruta como mensaje o como `AdaptWithRegisteredMessageAdapter`; (4) `AdaptWithRegisteredMessageAdapter(msg)` -> primer adaptador registrado cuya clase acepte `msg` (si ninguno: `unhandled`); (5) `Signal` -> `handleSignal`; (6) cualquier otro valor -> mensaje del actor. Los `TimerMsg` se filtran con `interceptTimerMsg` (los de una encarnación anterior se descartan).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:87-123 (aroundReceive); 125-144 (handleMessage y filtro de TimerMsg); 146-150 (handleSignal); 152-162 (UnstashException); 182-196 (adaptAndHandle); 198-218 (withSafelyAdapted); 220-230 (unhandled)
- **Precondiciones**: el actor está vivo y `ctx` inicializado perezosamente (ActorAdapter.scala:73-78). Solo se invoca desde el hilo del actor.
- **Comportamiento**: (a) ChildFailed solo existe para hijos directos que fallaron y que el padre observa (`isWatching`, ver AT-ADP-006.4); (b) `withSafelyAdapted`: si `adapt()` lanza una excepción NonFatal se entrega la señal `MessageAdaptionFailure(ex)` por la cadena de señales (el comportamiento puede manejarla; si no, la excepción de la señal por defecto hace fallar al actor y lo decide la supervisión); si `adapt()` devuelve `null` el valor se ignora y se registra un `warn` con el texto fijo "Adapter function returned null which is not valid as an actor message, ignoring. ..." (216); (c) `unhandled(Terminated(ref))` lanza `DeathPactException(ref)` (222-225), `unhandled(Signal)` no hace nada, el resto delega en el unhandled clásico (publica `UnhandledMessage`); (d) tras cada mensaje se limpia el MDC solo si se usó.
- **Errores**: excepciones del Behavior se propagan al supervisor clásico. `UnstashException` (y sus envoltorios `TypedActorFailedException`/`ActorInitializationException`) se desenvuelven restaurando `behavior = e.behavior` y relanzando la causa (152-162). Una excepción de `adapt()` nunca sale directamente: pasa por `MessageAdaptionFailure`.
- **Orden / concurrencia / ciclo de vida**: Todo ocurre en el hilo del actor (`setCurrentActorThread`/`clearCurrentActorThread` en finally, 88 y 120). El orden de evaluación de los casos del `match` es normativo (Terminated antes que Signal, `AdaptMessage` antes que mensaje plano).
- **Dependencias**: AC-CELL; AC-SUP; AC-DW; AC-MBX; AT-ADP-006.2; AT-ADP-006.4; AT-CTX-011; AT-CTX-014; AT-CTX-008; AT-TMR (timers, otro grupo)
- **Equivalente Rust propuesto**: bucle de actor propio con un enum de sobre interno `Envelope<M> { Msg(M), Signal(S), Adapted(Box<dyn FnOnce()->Option<M>>), ReceiveTimeout, TimerFired(clave, generación) }`; el `match` conserva el orden. Bajo runtime nativo (Q-X-8 b) las reglas sobreviven como lógica del intérprete de Behavior, no como código de adaptador clásico (Q-T-ADP-2). **Se preserva**: traducción Terminated/ChildFailed, receive-timeout como mensaje, MessageAdaptionFailure como señal supervisable, descarte de `null`, descarte de TimerMsg obsoleto, DeathPact en Terminated no manejado. **Diferencias/límites**: `null` solo existe si la función de adaptación devuelve `Option` (la rama del warn pasa a `None`); la excepción de `adapt()` pasa a ser `Result::Err`/pánico capturado (Q-X-5).
- **Aserciones**:
  - A1. un padre que observa un hijo que falló por una excepción recibe `ChildFailed` con `cause == ex` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/WatchSpec.scala:100-128 (afirma `probe.expectMessageType[ChildHasFailed].t.cause shouldEqual ex`)
  - A2. igual con un hijo supervisado con `SupervisorStrategy.stop` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/WatchSpec.scala:130-163
  - A3. `Terminated` no manejado produce `DeathPactException` (error registrado) y el actor falla — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/WatchSpec.scala:165-203 (afirma los dos `LoggingTestKit.error` y `probe.expectMessageType[Failed]`)
  - A4. `ChildFailed` es patrón-compatible con `Terminated(ref)` y con `ChildFailed(ref, e)` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TerminatedSpec.scala:24-45 (propiedad del tipo, no del adaptador)
  - A5. una excepción en la función de adaptación entrega `MessageAdaptionFailure`: sin manejarla el actor se detiene (7 comportamientos que "crashean"), manejándola (`receiveSignal` con `MessageAdaptionFailure`, `AbstractBehavior` con `onSignal`, `empty`, `ignore`) el actor sigue vivo — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/AdaptationFailureSpec.scala:75-129 (afirma `probe.expectTerminated(ref)` en el primer grupo y `probe.expectNoMessage()` en el segundo)
  - A6. función de `pipeToSelf` que devuelve `null`: se registra el warn con el prefijo "Adapter function returned null which is not valid as an actor message, ignoring" y no se entrega nada — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorContextPipeToSelfSpec.scala:47-72
  - A7. receive-timeout: `setReceiveTimeout(1.nano, msg)` entrega el mensaje configurado — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:578-595
  - A8. mensaje `Terminated` sin comportamiento `receiveSignal` que lo maneje: `unhandled` -> DeathPact (ver A3); un mensaje normal no manejado se publica como `UnhandledMessage` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:99-129 (afirma `unhandledProbe.receiveMessage()` tras `Behaviors.unhandled`)
  - A9. los `TimerMsg` de una encarnación anterior se descartan tras un restart: el mensaje ya encolado del timer viejo no llega y el comportamiento reiniciado empieza de cero (`GotPreRestart(false)`, luego `Tock(2)`/`Tock(1)`) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TimerSpec.scala:174-200 y 202-232; sin ClassCastException del TimerMsg — 327-370
  - A10. ningún adaptador registrado acepta el mensaje (rama `Nil` -> `unhandled`) — no afirmada por spec (ActorAdapter.scala:185-187); caracterización propuesta: enviar a `messageAdapter` un valor de tipo que ningún adaptador acepta publica `UnhandledMessage`
- **Validación prevista**: tests portados 1:1 de A1-A9 sobre el intérprete Rust; A10 como caracterización nueva (marcada como tal). Diferencial con el jar oficial (tools/oracle) para el orden Terminated/ChildFailed/DeathPact y MessageAdaptionFailure. Las aserciones comunes con WatchSpec/TimerSpec se ejecutan una sola vez en sus unidades dueñas (WATCH/TMR) y aquí se referencian.
- **Decisiones**: Q-X-8; Q-T-ADP-2; Q-T-ADP-3. Si Q-X-8 elige runtime nativo, esta fila queda como reglas de intérprete (no de adaptador) pero las aserciones A1-A9 no cambian.

## AT-ADP-006.2: ActorAdapter: Resultado del Behavior (next): unhandled, failed, stopped (internal-required)

- **Contrato observable**: Interno y estructural: `next(b, msg)` decide según la etiqueta del Behavior devuelto: `UnhandledBehavior` -> `unhandled(msg)` clásico (publica `UnhandledMessage` para mensajes); `FailedBehavior` -> si `rethrowTypedFailure` lanza `TypedActorFailedException(cause)` (el padre clásico decide y la excepción llega al padre sin volver a loguearse) y si no `context.stop(self)`; `StoppedBehavior` -> `behavior = ComposedStoppingBehavior(behavior, stopped)` y `context.stop(self)`: al llegar `PostStop` primero se entrega al comportamiento anterior (salvo Deferred sin iniciar) y luego se ejecuta el `onPostStop` del `stopped`, resultado `empty`; cualquier otro -> `Behavior.canonicalize(b, behavior, ctx)`.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:164-179 (next); 339-362 (ComposedStoppingBehavior)
- **Precondiciones**: `b` es el resultado de `interpretMessage`/`interpretSignal`.
- **Comportamiento**: el `PostStop` de un actor que se detiene por `Behaviors.stopped` se entrega por `ComposedStoppingBehavior` y no por `postStop()` directo; después `postStop()` ve `behavior` = `stopped`/`empty` y no vuelve a entregarlo (ver AT-ADP-006.3).
- **Errores**: `ComposedStoppingBehavior.receive` lanza `IllegalStateException("Stopping, should never receive a message")` (346-347); `receiveSignal` con una señal distinta de PostStop lanza `IllegalArgumentException("The ComposedStoppingBehavior should only ever receive a PostStop signal, but received <msg>")` (349-351). Son invariantes internas, no alcanzables por API pública.
- **Orden / concurrencia / ciclo de vida**: en el hilo del actor; PostStop al comportamiento anterior ocurre antes que el hook `onPostStop` del `stopped` (352-359).
- **Dependencias**: AC-CELL; AC-SUP; AT-BEH-003; AT-BEH-004; AT-BEH-024; AT-ADP-006.3; AT-EVT-003
- **Equivalente Rust propuesto**: `fn handle_next(Behavior) -> ActorAction { Continue, Unhandled, Fail(cause), Stop(on_post_stop) }`; las dos invariantes internas pasan a `debug_assert!`. **Se preserva**: orden PostStop(anterior) -> hook `stopped(cb)`; `Failed` con `rethrow` informa de la causa al padre. **Diferencias/límites**: `TypedActorFailedException` como tipo solo existe si el supervisor es clásico (Q-T-ADP-3).
- **Aserciones**:
  - A1. `Behaviors.unhandled` publica `UnhandledMessage` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorContextDelegateSpec.scala:82-99 (AT-EVT-003) y actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:99-129 (afirma `unhandledProbe.receiveMessage()`)
  - A2. un comportamiento que lanza en tipado spawneado desde clásico, sin `supervise`, se detiene (`PostStop`, luego `Terminated`) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/coexistence/ClassicSupervisingTypedSpec.scala:74-93 (afirma `probe.expectMsg(PostStop)` y `expectTerminated`); con `supervise(restart)` recibe `PreRestart` y no se detiene — 94-103
  - A3. el hook `stopped(cb)` del guardian se ejecuta en un cierre inmediato — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/GuardianStartupSpec.scala:73-89 (documentado en AT-SYS-004 A5)
  - A4. invariantes de ComposedStoppingBehavior (textos de IllegalStateException/IllegalArgumentException) — no afirmada por spec (ActorAdapter.scala:346-352); caracterización propuesta: test unitario con llamada directa a `receive`/`receiveSignal`
- **Validación prevista**: A1-A3 portados; A4 caracterización unitaria. Oráculo: escenarios jshell de orden de señales (PostStop antes de `stopped(cb)`).
- **Decisiones**: Q-X-8; Q-T-ADP-2; Q-T-ADP-3

## AT-ADP-006.3: ActorAdapter: Hooks de ciclo de vida: preStart, preRestart, postRestart, postStop (internal-required)

- **Contrato observable**: Interno y estructural con efectos observables: `preStart` arranca el Behavior (`start` + `validateAsInitial`) solo si sigue vivo y detiene el actor si ya no lo está; `preRestart` cancela todos los timers, entrega `PreRestart` al comportamiento actual y deja `behavior = stopped` (el estado se reconstruye en `postRestart`, no se reutiliza); `postRestart` cancela timers de nuevo y reinicia con `start` + `validateAsInitial` sobre el comportamiento inicial (detiene el actor si el reinicio produce un comportamiento muerto); `postStop` cancela timers, entrega `PostStop` salvo que el comportamiento sea `Deferred` (setup no iniciado, para no ejecutar efectos de creación al terminar) y deja `behavior = stopped`; los `aroundPreStart/PreRestart/PostRestart/PostStop` fijan y liberan el hilo de actor; cada hook limpia el MDC en finally.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:275-297 (around*); 299-306 (preStart); 309-315 (preRestart); 317-323 (postRestart); 325-335 (postStop)
- **Precondiciones**: el actor fue creado con `behavior` inicial (posiblemente `Deferred`).
- **Comportamiento**: tras un restart el comportamiento inicial vuelve a arrancarse (`setup` se re-ejecuta); los timers anteriores no sobreviven (se cancelan en `preRestart` y `postRestart`, y los mensajes de timer ya encolados se descartan, ver AT-ADP-006.1 A9); `PostStop` se entrega una única vez.
- **Errores**: una excepción en `preStart`/`postRestart` (p. ej. `validateAsInitial` rechaza `same`/`unhandled`) se propaga como `ActorInitializationException` al supervisor; el MDC se limpia igualmente.
- **Orden / concurrencia / ciclo de vida**: restart = `preRestart` (PreRestart al comportamiento viejo) -> nueva instancia -> `postRestart` (start). `postStop` tras `Behaviors.stopped` encuentra `ComposedStoppingBehavior` y entrega PostStop según AT-ADP-006.2. Todo en el hilo del actor.
- **Dependencias**: AC-CELL; AC-SUP; AT-BEH-007; AT-BEH-011; AT-BEH-012; AT-SUP-005; AT-TMR (timers)
- **Equivalente Rust propuesto**: hooks `on_start`/`on_pre_restart`/`on_post_restart`/`on_stop` del ciclo de vida del actor Rust con las mismas reglas: timers cancelados en ambos extremos del restart, `Deferred` sin iniciar no recibe PostStop, `PreRestart` entregado al comportamiento viejo. **Diferencias/límites**: si el runtime Rust reinicia reconstruyendo el actor desde `Behavior: Clone` (Q-T-BEH-1) `preRestart`/`postRestart` colapsan en una operación; el orden observable (PreRestart antes de nuevo start) se conserva.
- **Aserciones**:
  - A1. restart con `supervise(restart)`: el comportamiento recibe `PreRestart` y no `PostStop` ni `Terminated` del propio actor — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/coexistence/ClassicSupervisingTypedSpec.scala:94-103 (afirma `probe.expectMsg(PreRestart)`, `probe.expectNoMessage()`, `expectNoMessage()`); con `spawn` desde el contexto clásico — 117-128
  - A2. tras el restart los timers de la encarnación anterior no actúan: `GotPreRestart(false)` (el timer ya no está activo en PreRestart) y el nuevo `setup` vuelve a registrar el suyo — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TimerSpec.scala:174-200 y 202-232
  - A3. `PostStop` no se entrega a un `Deferred` sin iniciar: un sistema terminado inmediatamente no hace que el guardián reciba PostStop — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.scala:126-135 (el test solo afirma que `terminate` completa sin error; que no se entregue PostStop lo dice el comentario 121-124 y el código ActorAdapter.scala:328-331)
  - A4. el timer no se filtra tras PostStop con `stopped(cb)`: sin dead letters del timer — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TimerSpec.scala:308-325
  - A5. el hilo de actor se fija en los hooks around* (acceso a `context` desde el hook) — no afirmada por spec; caracterización propuesta: comprobar que `checkCurrentActorThread` no falla dentro de `setup` durante restart
- **Validación prevista**: A1, A2, A4 portados; A3 portado + caracterización de ausencia de PostStop; A5 caracterización. Oráculo: orden de señales con restart y stop (jshell).
- **Decisiones**: Q-X-8; Q-T-ADP-2; Q-T-ADP-3

## AT-ADP-006.4: ActorAdapter: Supervisión de hijos y registro de fallos (ChildFailed) (internal-required)

- **Contrato observable**: Interno y estructural: la `supervisorStrategy` del adaptador es `OneForOneStrategy(loggingEnabled = false)`. Con `TypedActorFailedException(cause)` registra el fallo (solo si el padre observa al hijo con `isWatching`) y decide `Stop` sin loguear (la supervisión tipada del hijo ya lo hizo). Con cualquier otra excepción: registra el fallo, loguea en nivel ERROR (mensaje = mensaje de la causa si es `ActorInitializationException` con causa, desenvolviendo `InvocationTargetException`; si no el mensaje de la excepción) y decide `Stop` si el hijo es un `ActorAdapter` (tipado) o el decisor clásico por defecto si es clásico (Restart salvo `ActorInitializationException` -> Stop, etc.). La entrada de `failures` se consume al llegar `Terminated` y produce `ChildFailed(ref, ex)` (ver AT-ADP-006.1).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:41-52 (TypedActorFailedException y classicSupervisorDecider); 83 (failures); 94-100; 232-273
- **Precondiciones**: el hijo falla mientras el padre es un actor tipado (ActorAdapter).
- **Comportamiento**: para hijos tipados el default es Stop (no Restart): lo confirma la ausencia de reinicios en ClassicSupervisingTypedSpec; para hijos clásicos se aplica el decisor clásico (`classicSupervisorDecider` = `defaultDecider` con Restart para lo no cubierto, 49-52).
- **Errores**: la excepción se registra con `ctx.log.error(logMessage, ex)` (257): el evento de log es parte del contrato (los tests lo capturan con LoggingTestKit).
- **Orden / concurrencia / ciclo de vida**: la decisión se toma en el hilo del padre (`setCurrentActorThread` en la estrategia, 234/264); `recordChildFailure` usa `sender()`.
- **Dependencias**: AC-CELL; AC-SUP; AC-DW; AT-ADP-006.1; AT-SUP-001; AT-CTX-005
- **Equivalente Rust propuesto**: tabla `ref_hijo -> causa` consumida al procesar la terminación del hijo; decisor Stop para hijos tipados y Restart-por-defecto para hijos clásicos solo si existe la capa de compatibilidad (Q-X-8 a). **Diferencias/límites**: con runtime tipado nativo el único decisor es el tipado (Stop por defecto, AT-SUP-001) y `TypedActorFailedException` desaparece (Q-T-ADP-3).
- **Aserciones**:
  - A1. hijo clásico (`classic1`) que lanza `ThrowIt3` bajo un padre tipado se registra como error y se reinicia por el decisor clásico por defecto: sigue vivo y responde el `ping` posterior, de modo que el probe recibe "ok" — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpec.scala:317-330 (afirma `LoggingTestKit.error[ThrowIt3.type]` y `probe.expectMsg("ok")`; el cuerpo del hijo está en 40-110 de ese archivo)
  - A2. hijo clásico que falla en su constructor: error `ActorInitializationException` registrado y el padre observa `terminated` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpec.scala:332-343
  - A3. excepción no manejada por un supervisor tipado se registra con el mensaje de la excepción ("sad panda") — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpec.scala:361-369
  - A4. tipado supervisando clásico: tras el fallo del hijo clásico se entrega `postStop` y luego `preStart` (el padre tipado lo reinicia) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/coexistence/TypedSupervisingClassicSpec.scala:61-73
  - A5. `ChildFailed` llega al padre que observa un hijo tipado que falló — ver AT-ADP-006.1 A1/A2
- **Validación prevista**: A1-A4 portados (A1-A4 solo si existe la capa Classic, Q-X-8 a); A5 vía AT-ADP-006.1. Oráculo: jar oficial con padre tipado e hijo clásico.
- **Decisiones**: Q-X-8; Q-T-ADP-2; Q-T-ADP-3

## AT-ADP-007: Despliegue remoto prohibido para actores typed (public)

- **Contrato observable**: Contractual: los Props typed fuerzan Deploy.local; si la configuracion de despliegue dispararia remoto, ConfigurationException del clasico (mensaje con prefijo ErrorMessages.RemoteDeploymentConfigErrorPrefix) se re-lanza como ConfigurationException("Remote deployment not allowed for typed actors", causa).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/PropsAdapter.scala:39,51; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorRefFactoryAdapter.scala:25-57
- **Condiciones**: Solo se convierte la ConfigurationException cuyo mensaje empieza por el prefijo; otras se propagan.
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: filas: AC-CFG. Externas: Runtime clasico completo (pekko-actor): ActorCell, supervision, ActorSystemImpl.
- **Equivalente Rust propuesto**: n/a bajo alcance sin remoting: Deploy::Local fijo. **Diferencias observables**: Sin remoting no hay despliegue remoto: la fila queda como "siempre local".
- **Aserciones**:
  - A1. no afirmada por spec; derivada de actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorRefFactoryAdapter.scala:35-38
- **Oraculo**: No aplica (requiere pekko-remote).
- **Decisiones**: ninguna especifica (hereda Q-X-* citadas en notes)

## AT-ADP-008: toTyped es idempotente: una sola ActorSystemAdapter por sistema clasico (public)

- **Contrato observable**: Contractual: ActorSystemAdapter(system) = AdapterExtension(system).adapter; la AdapterExtension clasica crea la ActorSystemAdapter una sola vez; las extensiones tipadas library-extensions se cargan al arranque del sistema clasico (pekko.library-extensions += LoadTypedExtensions) y una extension tipada no se crea dos veces al usar toTyped repetido.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorSystemAdapter.scala:143-176; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/AdapterExtension.scala:25-34; actor-typed/src/main/resources/reference.conf:35-36
- **Condiciones**: ExtensionId clasico con lookup propio (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorSystemAdapter.scala:155-161).
- **Orden / concurrencia / ciclo de vida**: Creada en arranque por la library-extension clasica.
- **Dependencias**: filas: AT-ADP-001; AT-EXT-002.3. Externas: Runtime clasico completo (pekko-actor): ActorCell, supervision, ActorSystemImpl.
- **Equivalente Rust propuesto**: Singleton por sistema. **Diferencias observables**: Solo relevante con Q-X-8(a).
- **Aserciones**:
  - A1. toTyped dos veces -> misma instancia — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpec.scala:203-208
  - A2. extension tipada creada una sola vez via ActorSystemAdapter — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:222-234
  - A3. extensiones listadas se cargan antes del primer acceso (createCount aumenta en 1 al crear el sistema) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:204-220
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-X-8

## AT-ADP-009: ActorRefAdapter: mensajes de sistema y rechazo de null (internal-required)

- **Contrato observable**: Interno: ActorRefAdapter.sendSystemMessage convierte SystemMessage typed (Create/Terminate/Watch/Unwatch/DeathWatchNotification/NoMessage) en los clasicos (Create -> IllegalStateException "WAT? No, seriously."); tell(null) lanza InvalidMessageException("[null] is not an allowed message") antes de enviar.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorRefAdapter.scala:35-38,43-45,70-81
- **Condiciones**: DeathWatchNotification -> sysmsg.DeathWatchNotification(ref, existenceConfirmed=true, addressTerminated=false) (79).
- **Orden / concurrencia / ciclo de vida**: Via canal de mensajes de sistema.
- **Dependencias**: filas: AT-ADP-004; AC-DW. Externas: Runtime clasico completo (pekko-actor): ActorCell, supervision, ActorSystemImpl.
- **Equivalente Rust propuesto**: Mapeo de mensajes de sistema de la capa de compat. **Diferencias observables**: Idem.
- **Aserciones**:
  - A1. tell(null) -> InvalidMessageException — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpec.scala:258-265
  - A2. mapeo de mensajes de sistema (no afirmada por spec; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorRefAdapter.scala:70-81)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-X-8

## AT-ADP-010: Aislamiento del guardián: user guardian detenido termina el sistema (GuardianStartupBehavior) (internal-required)

- **Contrato observable**: Fila de integración en el adaptador; el contrato canónico del arranque diferido y de la parada vive en AT-SYS-003 (GuardianStartupBehavior) y AT-SYS-004 (GuardianStopInterceptor); aquí se fija solo lo que aporta el adaptador: el user guardian se crea con `PropsAdapter(() => GuardianStartupBehavior(guardian), guardianProps, rethrowTypedFailure = false)` y el sistema le envía `GuardianStartupBehavior.Start` justo después de `system.start()`. Mientras no llega `Start` el guardián acumula hasta 1000 mensajes (Vector); el 1001º lanza `StashOverflowException("Guardian Behavior did not receive start and buffer is full.")`. Al llegar `Start` los almacenados pasan a un `withStash(1000)` y se desencolan en orden dentro de `intercept(GuardianStopInterceptor)(guardian)`; cuando el guardián deja de estar vivo se llama `system.terminate()` (para correr CoordinatedShutdown) y se devuelve el comportamiento detenido para que el adaptador entregue PostStop al anterior.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/GuardianStartupBehavior.scala:31-55 (arranque y límite); 57-90 (GuardianStopInterceptor); actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorSystem.scala:311-322 (construcción y Start)
- **Precondiciones**: sistema tipado creado con `ActorSystem(guardianBehavior, name)` (AT-SYS-001).
- **Comportamiento**: el Behavior del usuario no se arranca (no se ejecuta su `setup`) hasta `Start`; terminar el sistema antes de ese punto no entrega PostStop a un guardián no arrancado (Deferred, ver AT-ADP-006.3).
- **Errores**: `StashOverflowException` con el texto exacto si hay `StashCapacity = 1000` mensajes antes de `Start` (49-51). En la práctica `Start` se envía antes de devolver el sistema (ActorSystem.scala:321), por lo que el buffer solo se usa si otro emisor encola antes (hallazgo ya registrado en AT-SYS-003).
- **Orden / concurrencia / ciclo de vida**: `Start` primero; mensajes previos en orden de llegada; terminate se invoca antes de PostStop del guardián.
- **Dependencias**: AT-SYS-003; AT-SYS-004; AT-CS-001; AT-STASH (withStash)
- **Equivalente Rust propuesto**: en Rust el sistema puede construirse completo antes de arrancar el guardián, haciendo innecesario el buffer (Q-X-8 b); se conserva el efecto observable: ningún mensaje temprano se pierde y la parada del guardián termina el sistema ejecutando CoordinatedShutdown. **Diferencias/límites**: la excepción de 1000 mensajes no es alcanzable con construcción atómica; si se mantiene el buffer debe conservar el texto y el límite.
- **Aserciones**:
  - A1. guardián creado y detenido (`Behaviors.setup(_ => Behaviors.stopped)`) 11 veces seguidas no cuelga ni lanza — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpec.scala:210-219
  - A2. guardián que se detiene muy rápido (se envía `Done` a sí mismo y devuelve `stopped`) 11 veces — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpec.scala:221-235
  - A3. el guardián recibe un mensaje enviado inmediatamente tras crear el sistema, y su Behavior solo se arranca con el sistema clásico ya inicializado — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/GuardianStartupSpec.scala:34-71 (documentado en AT-SYS-003 A1/A2)
  - A4. guardián que termina tras un mensaje: el sistema se termina con `ActorSystemTerminateReason` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.scala:74-89; el guardián recibe PostStop al terminar — 103-124; un terminate inmediato antes de arrancar el guardián completa sin error — 126-135
  - A5. overflow del buffer previo a Start (1000 mensajes) con el texto exacto — no afirmada por spec (GuardianStartupBehavior.scala:49-51); caracterización propuesta: encolar 1001 mensajes antes de `Start` en una prueba directa del Behavior
- **Validación prevista**: A1-A4 portados (ya cubiertos también por AT-SYS-003/004: se ejecutan una sola vez, en SYS, y aquí solo se referencian); A5 caracterización unitaria si se conserva el buffer.
- **Decisiones**: Q-X-8; Q-T-SYS-3 (límite 1000 y excepción, definida en el grupo typed-core)

## AT-EVT-001: EventStream tipado: Publish / Subscribe / Unsubscribe (public)

- **Contrato observable**: Contractual: ActorSystem.eventStream es un ActorRef[EventStream.Command] (system actor "eventstream", EventStreamAdapter) con Publish(event), Subscribe[E](subscriber) (recibe E y sus subtipos, por classTag) y Unsubscribe(subscriber) (lo quita de TODOS los canales); Publish(null) lanza InvalidMessageException "[null] is not an allowed event" al construir el comando; implementado sobre el EventStream clasico (publish/subscribe(subscriber.toClassic, topic)/unsubscribe).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/eventstream/EventStream.scala:22-75; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/EventStreamAdapter.scala:27-48; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/EventStreamExtension.scala:30-37; actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorSystem.scala:201-206; docs/src/main/paradox/typed/event-stream.md:18
- **Condiciones**: Subscribe usa el runtimeClass del ClassTag como canal; Unsubscribe llama eventStream.unsubscribe(subscriber) global. Publicar es asincrono via el actor "eventstream".
- **Orden / concurrencia / ciclo de vida**: Los comandos se procesan en orden del mailbox del actor eventstream; la suscripcion no es inmediata (el test deja 100ms: "might still be a risk that the eventStream ! Subscribe hasn't arrived").
- **Dependencias**: filas: AC-EVT. Externas: EventStream y DeadLetterActorRef del clasico.
- **Equivalente Rust propuesto**: trait EventBus{publish<E:Any>(E); subscribe<E>(ActorRef<E>); unsubscribe}; Q-X-6 define subtipos. **Diferencias observables**: La suscripcion por superclase depende de la jerarquia JVM (Q-X-6). Rust: enum raiz o registro de traits.
- **Aserciones**:
  - A1. Subscribe(EventObj) + Publish(EventObj): solo el suscriptor de EventObj recibe, el de EventClass no — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/eventstream/EventStreamSpec.scala:36-54
  - A2. Unsubscribe deja de entregar — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/eventstream/EventStreamSpec.scala:56-60
  - A3. suscriptor de supertipo recibe subclases (Depth1/Depth2) y el de Level1 no recibe Depth1 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/eventstream/EventStreamSpec.scala:113-145
  - A4. Publish(null) -> InvalidMessageException (no afirmada por spec; actor-typed/src/main/scala/org/apache/pekko/actor/typed/eventstream/EventStream.scala:35-38)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-X-6;Q-T-EVT-1

## AT-EVT-002: Dead letters (DeadLetter) y suscripcion via message adapter (public)

- **Contrato observable**: Contractual: mensajes a actores terminados o adaptadores tras terminar el actor terminan en dead letters (deadLetters ! x publica DeadLetter(x, sender, deadLetters)); se puede suscribir con messageAdapter[DeadLetter] sin bucles infinitos al parar el actor suscrito; DeadLetter y Dropped extienden AllDeadLetters.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorSystem.scala:168-172; actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:572-631,733-743; docs/src/main/paradox/typed/logging.md:308-327
- **Condiciones**: DeadLetterActorRef: DeadLetter se publica salvo specialHandle; cualquier otro mensaje se envuelve en DeadLetter(msg, sender or deadLetters, this) (actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:736-743).
- **Orden / concurrencia / ciclo de vida**: El mensaje llega al suscriptor asincronamente; el test deja 100ms para la suscripcion.
- **Dependencias**: filas: AC-EVT; AC-REF. Externas: EventStream y DeadLetterActorRef del clasico.
- **Equivalente Rust propuesto**: Suscripcion de un actor a DeadLetter mediante adaptador. **Diferencias observables**: Ninguna si se conserva la regla de envoltura.
- **Aserciones**:
  - A1. deadLetters ! "msg1".."msg3" llegan adaptados al suscriptor — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/eventstream/EventStreamSpec.scala:62-85
  - A2. parar al suscriptor con mensajes pendientes termina (no bucle) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/eventstream/EventStreamSpec.scala:87-110
  - A3. ejemplos de documentacion de dead letters solo construyen el actor — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/eventstream/EventStreamDocSpec.scala:96-113
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-X-6

## AT-EVT-003: UnhandledMessage publicado al quedar un mensaje sin manejar (public)

- **Contrato observable**: Contractual: Behaviors.unhandled hace que ActorAdapter llame a Actor.unhandled clasico: Terminated -> DeathPactException(dead); otro mensaje -> eventStream.publish(UnhandledMessage(msg, sender, self)); una Signal sin manejar no publica nada (se ignora).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:220-230; actor/src/main/scala/org/apache/pekko/actor/Actor.scala:656-661
- **Condiciones**: unhandled(msg): case Terminated(ref) => throw DeathPactException; case _: Signal => nada; case other => super.unhandled(other) (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:220-229).
- **Orden / concurrencia / ciclo de vida**: Se publica en el hilo del actor al terminar el procesamiento del mensaje.
- **Dependencias**: filas: AT-BEH-003; AC-EVT. Externas: EventStream y DeadLetterActorRef del clasico.
- **Equivalente Rust propuesto**: Evento UnhandledMessage{message,sender,recipient}. **Diferencias observables**: sender se reporta como el del contexto clasico; en typed puro el sender es noSender salvo adaptadores.
- **Aserciones**:
  - A1. UnPingable no manejado produce UnhandledMessage(UnPingable,_,_) y el actor sigue (ForwardTo/ResponseFrom) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorContextDelegateSpec.scala:82-99
  - A2. createUnhandledMessageProbe recibe UnhandledMessage publicado — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitSpec.scala:133-139
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-T-EVT-1

## AT-EVT-004: Dropped (stash de supervision lleno, group router sin routees, Topic sin suscriptores) (public)

- **Contrato observable**: Contractual: ciertos descartes se publican como Dropped(msg, reason, sender): (1) restart con backoff y stash lleno -> eventStream.publish(Dropped(msg, "Stash is full in [RestartSupervisor]", self)) (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:120-124; texto afirmado por el spec); (2) group router -> EventStream.Publish(Dropped(msg, "Stash is full in group router for [<key>]" | "No routees in group router for [<key>]", self)); (3) Topic sin suscriptores -> deadLetters ! Dropped(msg, "No topic subscribers known", self), que DeadLetterActorRef envuelve como DeadLetter(Dropped(...)) (actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:740-742) en vez de publicar el Dropped (camino distinto a (1) y (2)).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:120-124; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/GroupRouterImpl.scala:95-102,148-154; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/pubsub/TopicImpl.scala:92-97; actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:622-631,736-743
- **Condiciones**: Ver contrato. Reasons textuales afirmados: solo "no reason" en testkit; los demas por codigo.
- **Orden / concurrencia / ciclo de vida**: Publicacion sincrona en el hilo del actor que descarta.
- **Dependencias**: filas: AC-EVT. Externas: EventStream y DeadLetterActorRef del clasico.
- **Equivalente Rust propuesto**: fn publish_dropped(system, msg, reason, sender) con un unico camino; la variante Topic se decide en Q-T-EVT-1. **Diferencias observables**: Rust puede unificar los caminos; si se unifica, Topic cambia su evento observable (de DeadLetter(Dropped) a Dropped).
- **Aserciones**:
  - A1. con stash=2 y backoff 1s, Ping(3) y Ping(4) se publican como Dropped(Ping(n), "Stash is full in [RestartSupervisor]", ref.toClassic) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:882-909
  - A2. group router sin routees publica Dropped por mensaje — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/RoutersSpec.scala:186-198
  - A3. createDroppedMessageProbe recibe el Dropped publicado manualmente — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitSpec.scala:146-151
  - A4. Topic sin suscriptores: deadLetters ! Dropped (no afirmada por spec; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/pubsub/TopicImpl.scala:96) -> DeadLetter(Dropped) por actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:740-742 (derivada, a validar con oraculo)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh (jshell: suscribirse a Dropped y DeadLetter, publicar en Topic sin suscriptores). Procedencia: jar oficial.
- **Decisiones**: Q-T-EVT-1

## AT-LOG-001: context.log (SLF4J Logger), nombre de logger detectado y setLoggerName (public)

- **Contrato observable**: Contractual: context.log crea perezosamente un Logger por actor; su nombre se detecta por la clase que lo uso primero (LoggerClass.detectLoggerClassFromStack sobre Behavior) y se puede cambiar con setLoggerName(String|Class) (solo dentro del hilo del actor; no thread-safe); cada acceso a `log` reinstala el MDC del actor; el logger acepta markers y causa.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/ActorContext.scala:78-106; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorContextImpl.scala:160-188; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/LoggerClass.scala:24-60; docs/src/main/paradox/typed/logging.md:31-80,448-478
- **Condiciones**: Logger perezoso (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorContextImpl.scala:160-171); tags de Props.deploy alimentan el LoggingContext. setLoggerName reemplaza el logger y marca hasCustomName (183-185).
- **Orden / concurrencia / ciclo de vida**: Se obtiene dentro del actor; checkCurrentActorThread valida el hilo (AT-CTX-016).
- **Dependencias**: filas: AC-EVT (logging clasico). Externas: SLF4J (Logger, MDC, Marker) y Logback/Slf4jLogger de pekko-slf4j.
- **Equivalente Rust propuesto**: trait ActorLog en contexto; nombre por defecto = type_name del comportamiento que loguea (#[track_caller]/module_path) o set_logger_name(&str). **Diferencias observables**: Detectar la clase del llamador por stack no es portable: Rust debe fijar el nombre explicitamente o por type_name.
- **Aserciones**:
  - A1. context.log.info("Started") y "got message Hello" se capturan con LoggingTestKit — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.scala:78-93
  - A2. setLoggerName(classOf[AnotherLoggerClass]) emite con ese nombre y solo se capturan eventos de ese logger (2 ocurrencias) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.scala:95-129
  - A3. el nombre por defecto es la clase donde se llamo primero (spec, objeto y AbstractBehavior) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.scala:131-173
  - A4. markers y causa llegan al evento (5 y 2 ocurrencias) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.scala:175-202
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh a nivel de logger name y evento capturado (LoggingTestKit). Procedencia: jar oficial.
- **Decisiones**: Q-T-LOG-1;Q-T-LOG-2;Q-X-7

## AT-LOG-002: MDC automatico: pekkoSource, pekkoAddress, sourceActorSystem, pekkoTags (public)

- **Contrato observable**: Cada evento emitido con `context.log` lleva en el MDC cuatro claves (nombres exactos): `pekkoSource` (path completo del actor, `self.path.toString`), `sourceActorSystem` (nombre del sistema), `pekkoAddress` (`provider.addressString` si el sistema es un `ActorSystemAdapter`; si no `Address("pekko", nombre).toString`) y `pekkoTags` (tags de `Props` unidos con coma; la clave **no se pone** si no hay tags). Se (re)instalan en cada acceso a `context.log`, de modo que otro código que haya tocado el MDC entre dos accesos queda sobrescrito solo en esas claves. Tras procesar cada mensaje/señal/hook el MDC se limpia **solo si se usó** (`mdcUsed`), por lo que no se filtra entre mensajes. `Behaviors.withMdc` añade valores estáticos y/o por mensaje; el más externo gana en la declaración (el interceptor reemplaza a otro `withMdc` anidado en `aroundStart`), el MDC por mensaje se aplica en `aroundReceive`, las señales reciben solo el estático, y el MDC se limpia al terminar cada invocación del interceptor.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorMdc.scala:23-45 (claves y setMdc/clearMdc); actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorContextImpl.scala:44-90 (LoggingContext), 173-178 (`log`), 192-201 (`clearMdc`); actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:119-122,306,314,322,334 (limpieza); actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/WithMdcBehaviorInterceptor.scala:44-118; actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Behaviors.scala:284-322 (withMdc)
- **Precondiciones**: el acceso a `context.log` ocurre en el hilo del actor (`checkCurrentActorThread`, AT-CTX-016).
- **Comportamiento**: `pekkoTags` proviene de `Props.deploy.tags` (AT-DSP-004). `mdcUsed` se activa en `ActorMdc.setMdc` (ActorMdc.scala:31-33) y se desactiva en `clearMdc` (ActorContextImpl.scala:196-198); el acceso al ThreadLocal del MDC se evita si nunca se usó el log (optimización observable solo por rendimiento). `withMdc`: valores estáticos primero, luego los del mensaje (los del mensaje pisan a los estáticos con la misma clave, ver Scaladoc Behaviors.scala:301-306). El `ClassTag[T]` hace que solo los mensajes de tipo T pasen por el interceptor.
- **Errores**: ninguno propio.
- **Orden / concurrencia / ciclo de vida**: el MDC es un ThreadLocal: es válido solo durante el procesamiento síncrono del mensaje en el hilo del actor; `Future`/callbacks que corren en otro hilo no lo heredan. La limpieza ocurre en `finally` aunque el comportamiento lance.
- **Dependencias**: AT-DSP-004; AT-BEH-019 (withMdc); AT-LOG-001; AT-CTX-016; AT-ADP-006.1
- **Equivalente Rust propuesto**: campos estructurados del span `tracing` abierto por mensaje: `pekko_source`, `pekko_address`, `source_actor_system`, `pekko_tags` (más los campos de `with_mdc`); el span se abre antes de invocar el comportamiento y se cierra al terminar (equivale a "limpiar al terminar"), sin ThreadLocal, por lo que sobrevive a `.await` dentro del mismo mensaje. **Se preserva**: valores, condición de ausencia de `pekkoTags`, aislamiento entre mensajes, prioridad del MDC por mensaje sobre el estático, "el más externo gana". **Diferencias/límites**: los nombres camelCase de las claves son parte de la compatibilidad de configuración de appenders JVM; en Rust se proponen mantenerlos como nombres de campo para que las pruebas diferenciales comparen 1:1 (Q-T-LOG-1 decide si se renombran a snake_case); el efecto "una tarea lanzada desde el actor no hereda el MDC" no se reproduce de forma natural (los spans se pueden heredar): diferencia aceptada y a documentar.
- **Aserciones**:
  - A1. los tags de `Props` aparecen como `pekkoTags -> "tag1,tag2"` en el log de arranque y en el de un mensaje — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.scala:270-289
  - A2. `withMdc` estático + por mensaje: el log de `setup` solo lleva el estático (`static -> 1`) y claves `pekko*`/`sourceActorSystem`/`static`; el log de un mensaje lleva `static`, `txId` y `first` según el mensaje; el MDC por mensaje no se filtra al siguiente (`!event.mdc.contains("first")`) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.scala:340-382
  - A3. con `withMdc` anidados gana el más externo: aparece `outermost` y no `innermost` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.scala:384-404
  - A4. el MDC de `withMdc` se mantiene cuando el comportamiento cambia a otro — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.scala:406-432
  - A5. si el nuevo comportamiento vuelve a envolverse en `withMdc`, el MDC se reemplaza (`mdc-version` 1 -> 2) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.scala:434-462
  - A6. código del usuario puede modificar el MDC (`MDC.put` sobre `mdc`) y verlo en el siguiente log (`inner-outer`) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.scala:464-481
  - A7. siempre están `pekkoAddress`, `pekkoSource` (path del actor) y `sourceActorSystem` en los logs de `setup` y de 10 mensajes, y el MDC se limpia entre mensajes (`MDC.get("logSource") == null` antes de acceder a `context.log`; si no, el comportamiento lanza) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.scala:483-529
  - A8. ausencia de `pekkoTags` cuando no hay tags — no afirmada por spec (ActorMdc.scala:36-37); caracterización propuesta: log de un actor sin tags no contiene la clave
  - A9. `pekkoAddress` cae a `pekko://<sistema>` cuando el sistema no es `ActorSystemAdapter` — no afirmada por spec (ActorContextImpl.scala:60-64); caracterización propuesta con un `ActorSystemStub` (AT-TK-011) cuya address sea la del stub
- **Validación prevista**: A1-A7 portados con el captor de logs de AT-TK-006 (propiedad de eventos capturados: mensaje, MDC); A8-A9 caracterización. Diferencial con el jar oficial: capturar el MDC completo de un evento (AT-TK-006) para un actor con y sin tags.
- **Decisiones**: Q-T-LOG-1 (claves y representación del MDC); Q-T-TK-2 (captura de logs sin Logback)

## AT-LOG-003: Configuracion SLF4J: pekko.use-slf4j=on y Slf4jLogger agregado automaticamente (internal-required)

- **Contrato observable**: Contractual por spec: con pekko.use-slf4j=on (default de actor-typed/src/main/resources/reference.conf:68) y Slf4jLogger en el classpath, Settings.amendSlf4jConfig agrega Slf4jLogger al frente de pekko.loggers, quita DefaultLogger y fija pekko.logging-filter=Slf4jLoggingFilter si no estaba configurado (tambien en sistemas clasicos); use-slf4j=off lo desactiva; si Slf4jLogger ya esta en loggers solo se completa el filtro.
- **Fuente**: actor-typed/src/main/resources/reference.conf:56-68; actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:335-370
- **Condiciones**: Normal: ver contrato. Requiere que la clase Slf4jLogger este en el classpath (dynamicAccess.classIsOnClasspath).
- **Orden / concurrencia / ciclo de vida**: Se evalua al construir Settings del sistema (antes de crear loggers).
- **Dependencias**: filas: AC-EVT; AC-CFG. Externas: SLF4J (Logger, MDC, Marker) y Logback/Slf4jLogger de pekko-slf4j.
- **Equivalente Rust propuesto**: Config key aceptada; el backend por defecto se decide en Q-T-LOG-1/Q-X-7. **Diferencias observables**: Rust no tiene SLF4J: el valor por defecto se traduce a "usar tracing".
- **Aserciones**:
  - A1. por defecto loggers=[Slf4jLogger] y logging-filter=Slf4jLoggingFilter (tipado y clasico) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.scala:298-320
  - A2. use-slf4j=off deja DefaultLogger y DefaultLoggingFilter — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.scala:322-336
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh (config efectiva). Procedencia: jar oficial.
- **Decisiones**: Q-T-LOG-3;Q-X-7

## AT-LOG-004: LoggerOps (trace2/traceN... para Scala 2) y Markers (public)

- **Contrato observable**: `scaladsl.LoggerOps` es una clase implícita (`AnyVal`) sobre `org.slf4j.Logger` que agrega, para cada nivel (`trace`, `debug`, `info`, `warn`, `error`) cuatro métodos: `<nivel>2(format, arg1, arg2)`, `<nivel>2(marker, format, arg1, arg2)`, `<nivel>N(format, args*)` y `<nivel>N(marker, format, args*)`. Cada uno delega **sin alterar el mensaje** en la sobrecarga SLF4J equivalente (`log.info(format, arg1, arg2)`); las `N` convierten los `Any*` a `Array[Object]` (box de primitivos con `asInstanceOf[AnyRef]` si el array no es de objetos) antes de llamar a la sobrecarga varargs. Su única razón de ser es resolver la ambigüedad de sobrecarga del compilador de Scala 2 con 2 argumentos y con varargs de primitivos; el texto emitido es el de SLF4J (`{}` sustituidos en orden). Los `Marker` se pasan al backend sin tratamiento y llegan a `LoggingEvent.marker`.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/package.scala:30-397 (20 métodos: 5 niveles x {2, 2 con marker, N, N con marker}; p. ej. `info2` en 193-210, `infoN` en 224-260); actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.scala:175-189 (markers)
- **Precondiciones**: `import org.apache.pekko.actor.typed.scaladsl.LoggerOps` (o `scaladsl._`). Solo aplica a Scala 2 (en Scala 3 las sobrecargas no son ambiguas).
- **Comportamiento**: formato SLF4J estándar: `"[template {} {}]"` con `a`, `2` produce `"[template a 2]"`; valores `case class` se renderizan con su `toString`.
- **Errores**: ninguno propio (los de SLF4J: demasiados/pocos argumentos no fallan, quedan `{}` sin sustituir o se ignoran).
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: AT-LOG-001; AT-TK-006 (captura para los tests)
- **Equivalente Rust propuesto**: **subsumido por las macros de `tracing`** (`info!("template {} {}", a, b)` o campos estructurados): no hay sobrecarga ambigua ni varargs, por lo que no se porta `LoggerOps` ni sus 20 métodos; el efecto observable que se preserva es el texto formateado y el marcador. Marker -> campo `marker` o `target` del evento (propuesta, Q-T-LOG-4). **Diferencias/límites**: se pierde la API simbólica (`info2`, `infoN`); se conserva el resultado (mensaje renderizado idéntico) que las aserciones comprueban.
- **Aserciones**:
  - A1. markers llegan al backend en `debug`/`info`/`warn`/`error` (5 ocurrencias con `event.marker.map(_.getName) == marker`) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.scala:175-189
  - A2. `info2("[template {} {}]", ...)` con combinaciones (String,String), (String,Int), (Int,Int), (Int,String), (String,Value), (Value,Value), (Value,Value distinto) produce exactamente `"[template a b]"`, `"[template a 2]"`, `"[template 1 2]"`, `"[template 1 b]"`, `"[template a Value2(2)]"`, `"[template Value1(1) Value1(1)]"`, `"[template Value1(1) Value2(2)]"` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/LoggerOpsSpec.scala:37-59 (usa `LoggingTestKit.info(msg).expect`)
  - A3. `infoN("[template {} {} {}]", ...)` con 8 combinaciones de String/Int/case class produce el mensaje con las tres sustituciones — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/LoggerOpsSpec.scala:61-86
  - A4. el test solo ejercita `info2`/`infoN`; los otros 18 métodos (trace, debug, warn, error con y sin marker) delegan por el mismo patrón — no afirmada por spec (package.scala:51-397); caracterización propuesta: no portar (subsumido); si se portara, una prueba por nivel con marker
  - A5. todas las sobrecargas de `context.log` (debug/info/warn/error con 0-3 argumentos, marker y causa) llegan al listener: 36 ocurrencias — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.scala:204-261 (cuenta de eventos, no su contenido)
- **Validación prevista**: A1, A2, A3 y A5 se portan como pruebas del **mensaje renderizado** con la macro de logging elegida (mismo texto) usando el captor de AT-TK-006; la API `LoggerOps` no tiene test Rust porque no se porta (Q-T-LOG-4).
- **Decisiones**: Q-T-LOG-4 (subsumir); Q-T-LOG-1

## AT-LOG-005: Mensajes de log del LocalActorRefProvider en logger "serialization" (internal-accidental)

- **Contrato observable**: Accidental (clasico observado desde typed): resolver un path invalido, que no corresponde a un actor activo, o foraneo con provider.resolveActorRef emite un evento debug en el logger "org.apache.pekko.actor.LocalActorRefProvider.Deserialization" con los textos "of unknown (invalid) path [dummy/path]", "Resolve (deserialization) of path [user/invalid] doesn't match an active actor." y "Resolve (deserialization) of foreign path [pekko://otherSystem/user/foo]".
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorRefResolver.scala:80-92; actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala (LocalActorRefProvider.resolveActorRef)
- **Condiciones**: Los mensajes son nivel DEBUG con loggerName especifico.
- **Orden / concurrencia / ciclo de vida**: Sincrono en la resolucion.
- **Dependencias**: filas: AC-REF; AC-SER. Externas: SLF4J (Logger, MDC, Marker) y Logback/Slf4jLogger de pekko-slf4j.
- **Equivalente Rust propuesto**: Log debug con target fijo al resolver refs por path. **Diferencias observables**: Depende de Q-T-LOG-1 (target de log).
- **Aserciones**:
  - A1. path invalido: debug "of unknown (invalid) path [dummy/path]" — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/LocalActorRefProviderLogMessagesSpec.scala:40-49
  - A2. path no activo: debug "Resolve (deserialization) of path [user/invalid] doesn't match an active actor." — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/LocalActorRefProviderLogMessagesSpec.scala:51-61
  - A3. path foraneo: debug "Resolve (deserialization) of foreign path [pekko://otherSystem/user/foo]" — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/LocalActorRefProviderLogMessagesSpec.scala:63-79
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-T-LOG-1

## AT-PS-001.1: Topic: pub-sub local (Subscribe, Unsubscribe y terminacion de suscriptores) (public)

- **Contrato observable**: Contractual: Subscribe(subscriber) agrega un suscriptor local (idempotente: un segundo Subscribe del mismo ref se ignora con log debug), lo observa (watchWith); al pasar de 0 a 1 suscriptores el Topic se registra en el Receptionist bajo ServiceKey[TopicImpl.Command[T]](topicName); Unsubscribe lo quita y hace unwatch; cuando el ultimo local se va (por Unsubscribe o terminacion) el Topic se desregistra.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/pubsub/Topic.scala:26-140; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/pubsub/TopicImpl.scala:33-169; docs/src/main/paradox/typed/distributed-pub-sub.md:19-69
- **Condiciones**: Unsubscribe de un ref que no estaba hace unwatch y, si el conjunto queda vacio, igual se desregistra (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/pubsub/TopicImpl.scala:133-143). Se registra solo cuando size == 1 tras agregar (119-124).
- **Orden / concurrencia / ciclo de vida**: Los efectos hacia el Receptionist son asincronos (mensajes); GetTopicStats permite observar que el registro se completo.
- **Dependencias**: filas: AT-RCP-005.2; AT-CTX-006; AT-EVT-004. Externas: Receptionist (AT-RCP) y DeathWatch.
- **Equivalente Rust propuesto**: Topic::<M>::new(name) -> Behavior<TopicCommand<M>>; TopicCommand::{Publish,Subscribe,Unsubscribe,GetTopicStats}. **Diferencias observables**: Ninguna con receptionist local.
- **Aserciones**:
  - A1. 3 Subscribe -> localSubscriberCount == 3 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/pubsub/LocalPubSubSpec.scala:28-46
  - A2. tras Unsubscribe el suscriptor no recibe y localSubscriberCount baja — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/pubsub/LocalPubSubSpec.scala:128-149
  - A3. suscribir a topic con otro nombre no cruza mensajes — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/pubsub/LocalPubSubSpec.scala:96-126
  - A4. la terminacion del suscriptor lo elimina y desregistra si era el ultimo (no afirmada por spec; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/pubsub/TopicImpl.scala:145-154)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: ninguna especifica (hereda Q-X-* citadas en notes)

## AT-PS-001.2: Topic: pub-sub local (Publish: entrega local y entre instancias del mismo topic) (public)

- **Contrato observable**: Contractual: Publish(msg) (null -> InvalidMessageException "[null] is not an allowed message") se envia como MessagePublished a todas las instancias de Topic conocidas via Listing (incluida si misma si tiene suscriptores) y cada una lo reenvia a sus suscriptores locales; si aun no hay Listing pero hay suscriptores locales entrega solo a los locales; sin ninguno publica Dropped(msg, "No topic subscribers known", self) a deadLetters (ver Q-T-EVT-1).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/pubsub/Topic.scala:26-140; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/pubsub/TopicImpl.scala:33-169; docs/src/main/paradox/typed/distributed-pub-sub.md:19-69
- **Condiciones**: Rama (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/pubsub/TopicImpl.scala:92-108): topicInstances vacio -> (locales vacios -> Dropped a deadLetters | locales -> entrega directa) ; topicInstances no vacio -> MessagePublished a cada instancia. En local con receptionist local la instancia propia aparece en el Listing tras registrarse.
- **Orden / concurrencia / ciclo de vida**: Entre instancias no hay garantia de entrega ni orden estricto (documentado en actor-typed/src/main/scala/org/apache/pekko/actor/typed/pubsub/Topic.scala:31-33); la lista de suscriptores es eventualmente consistente.
- **Dependencias**: filas: AT-RCP-005.2; AT-CTX-006; AT-EVT-004. Externas: Receptionist (AT-RCP) y DeathWatch.
- **Equivalente Rust propuesto**: Topic::<M>::new(name) -> Behavior<TopicCommand<M>>; TopicCommand::{Publish,Subscribe,Unsubscribe,GetTopicStats}. **Diferencias observables**: Ninguna con receptionist local.
- **Aserciones**:
  - A1. Publish("banana") llega a los 3 suscriptores locales — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/pubsub/LocalPubSubSpec.scala:28-57
  - A2. dos instancias del mismo topic: Publish en una llega a los suscriptores de la otra — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/pubsub/LocalPubSubSpec.scala:59-94
  - A3. tras Unsubscribe Publish ya no llega (expectNoMessage 200ms) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/pubsub/LocalPubSubSpec.scala:128-149
  - A4. Publish sin suscriptores -> Dropped a deadLetters (no afirmada por spec; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/pubsub/TopicImpl.scala:92-97)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-T-EVT-1

## AT-PS-001.3: Topic: pub-sub local (GetTopicStats) (public)

- **Contrato observable**: Contractual: GetTopicStats(replyTo) responde TopicStats(localSubscriberCount, topicInstanceCount) como instantanea; topicInstanceCount cuenta instancias del topic (local y cluster) con al menos un suscriptor; se usa en tests para saber que el registro se completo.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/pubsub/Topic.scala:26-140; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/pubsub/TopicImpl.scala:33-169; docs/src/main/paradox/typed/distributed-pub-sub.md:19-69
- **Condiciones**: topicInstanceCount = tamano del ultimo Listing recibido (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/pubsub/TopicImpl.scala:156-163). Un topic sin suscriptores propios no se cuenta.
- **Orden / concurrencia / ciclo de vida**: Observacion eventualmente consistente (el test usa awaitAssert).
- **Dependencias**: filas: AT-RCP-005.2; AT-CTX-006; AT-EVT-004. Externas: Receptionist (AT-RCP) y DeathWatch.
- **Equivalente Rust propuesto**: Topic::<M>::new(name) -> Behavior<TopicCommand<M>>; TopicCommand::{Publish,Subscribe,Unsubscribe,GetTopicStats}. **Diferencias observables**: Ninguna con receptionist local.
- **Aserciones**:
  - A1. localSubscriberCount == 3 y topicInstanceCount == 1 con 3 suscriptores — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/pubsub/LocalPubSubSpec.scala:28-46
  - A2. la otra instancia ve topicInstanceCount == 1 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/pubsub/LocalPubSubSpec.scala:59-94
  - A3. otro topic no ve la instancia (0) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/pubsub/LocalPubSubSpec.scala:96-126
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: ninguna especifica (hereda Q-X-* citadas en notes)

## AT-SPV-001: SpawnProtocol (Spawn -> ActorRef) con nombres unicos (public)

- **Contrato observable**: Contractual: Behavior que, ante Spawn(behavior, name, props, replyTo), crea un hijo y responde con su ActorRef; name vacio o null -> spawnAnonymous; si el nombre esta tomado agrega sufijo "-1","-2",... (el primer candidato libre; el spec fija el formato aunque el Scaladoc lo declare indefinido); usable como guardian (hijos bajo /user) y con ask; no tiene mensaje de parada: se detiene el padre o por el protocolo del hijo. `SpawnProtocol.create()` (Java) es `apply()`.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/SpawnProtocol.scala:36-89 (create 60, apply 65-87, Spawn 54-55); docs/src/main/paradox/typed/actor-lifecycle.md:125-158; ejemplo Java: actor-typed-tests/src/test/java/jdocs/org/apache/pekko/typed/SpawnProtocolDocTest.java:36-98
- **Condiciones**: spawnWithUniqueName(c): c==0 -> name; si no name-c; recursa mientras ctx.child(candidato) exista (actor-typed/src/main/scala/org/apache/pekko/actor/typed/SpawnProtocol.scala:74-82). Nombre null o "" -> anonimo (70-71). Un nombre invalido propaga la excepcion de ctx.spawn (InvalidActorNameException) al actor SpawnProtocol (fallo del guardian) — derivada del codigo, no afirmada por spec.
- **Orden / concurrencia / ciclo de vida**: Responde con replyTo ! ref inmediatamente tras crear (SpawnProtocol.scala:84); Behaviors.same (85). Un hijo anterior ya parado libera su nombre (ctx.child devuelve None), por lo que el nombre se reutiliza (derivada del codigo).
- **Dependencias**: filas: AT-CTX-002; AT-CTX-003; AT-ASK-001. Externas: Ninguna externa.
- **Equivalente Rust propuesto**: struct Spawn<M>{behavior,name,props,reply_to:ActorRef<ActorRef<M>>} y enum SpawnCommand con borrado de tipo (Box<dyn Any>) segun Q-T-SPV-1. **Diferencias observables**: El existencial Spawn[T] choca con Rust: requiere borrado de tipo en el mensaje y downcast al responder (Q-T-SPV-1). El unico fallo contractual de `Spawn` Java es el mismo.
- **Aserciones**:
  - A1. hijo "child" bajo "parent": path.name == "child", parent.name == "parent" — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SpawnProtocolSpec.scala:47-57 (el `child ! Ping(...)` de la linea 56 no verifica la respuesta)
  - A2. ask de Spawn devuelve un ActorRef que completa el Future (`parentReply.futureValue`); el test no compara el valor, solo que se obtiene y se le puede enviar un Ping (sin verificar respuesta) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SpawnProtocolSpec.scala:59-68
  - A3. como guardian los hijos estan en /user/child1 y /user/child2 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SpawnProtocolSpec.scala:70-84
  - A4. nombre tomado: "child","child-1", y tras "child-2" explicito -> "child-3" — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SpawnProtocolSpec.scala:86-107
  - A5. BehaviorTestKit: con nombre "child" el efecto es Spawned(target,"child") y la respuesta trae path.name == "child" — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SpawnProtocolSpec.scala:117-124
  - A6. BehaviorTestKit: con nombre "" el efecto es SpawnedAnonymous(target) y path.name empieza por "$" — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SpawnProtocolSpec.scala:126-133
  - A7. no afirmada por spec: nombre `null` (rama de SpawnProtocol.scala:70), nombre invalido, reutilizacion de un nombre liberado. Caracterizacion propuesta: oraculo jshell con name=null y con "a/b".
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh (nombres de hijos). Procedencia: jar oficial.
- **Decisiones**: Q-T-SPV-1

## AT-EXT-001.1: Extension / ExtensionId: identidad, una instancia por sistema y concurrencia (public)

- **Contrato observable**: Contractual: registerExtension(id) crea la extension una vez por sistema de forma concurrentemente segura (CountDownLatch + putIfAbsent; los demas hilos esperan); apply(system)/get(system) = registerExtension; la identidad de ExtensionId es por referencia (equals = eq, hashCode = identityHashCode, `final`), de modo que dos ids de la misma clase son distintos (MultiExtensionId(1) y (2)); createExtension "se invoca a lo sumo una vez por sistema" (Scaladoc).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:78-136; actor-typed/src/main/scala/org/apache/pekko/actor/typed/Extensions.scala:118-137 (equals/hashCode 130-131, createExtension 120-123, `id` Java 133-136)
- **Condiciones**: Un creador que accede a otras extensiones (p.ej. receptionist) desde el constructor funciona (AccessSystemFromConstructor, actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:70-77). createExtension devolviendo null lanza IllegalStateException "Extension instance created as 'null' for extension [<id>]" (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:103; no afirmada por spec).
- **Orden / concurrencia / ciclo de vida**: Registro: putIfAbsent(latch) -> crear -> replace(latch, instancia) -> countDown (finally) (ExtensionsImpl.scala:91-123). Si otro hilo ya esta registrando, el segundo reintenta `registerExtension` (119-121) y bloquea en el latch (findExtension, 128-136).
- **Dependencias**: filas: AC-EXT. Externas: Typesafe Config (getStringList), DynamicAccess/reflexion JVM, CountDownLatch/ConcurrentHashMap.
- **Equivalente Rust propuesto**: ExtensionId<T> como valor estatico/Arc con identidad por puntero; registry Mutex<HashMap<id, OnceCell<Result<Arc<T>,Error>>>>. **Diferencias observables**: Rust: OnceCell/Mutex en lugar de latch; null no existe (Result).
- **Aserciones**:
  - A1. registerExtension dos veces, apply y get devuelven la misma instancia — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:90-102
  - A2. 1001 registros concurrentes de SlowExtension (25 ms) devuelven la misma instancia (el propio test dice "not exactly water tight") — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:104-119
  - A3. dos ExtensionId de la misma clase -> instancias con n distinto (1 y 2) y estables — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:155-163
  - A4. crear una extension que usa log y receptionist en su constructor no falla — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:254-265
  - A5. con un sistema clasico, DummyExtension1(classicSystem.toTyped) dos veces devuelve la misma instancia (la ActorSystemAdapter es unica) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:222-234
  - A6. Java: MyExtension.get(system) dos veces -> misma instancia (assertSame) — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/ExtensionsTest.java:82-93 (variante Scala) y 60-80 (variante Java)
  - A7. no afirmada por spec: null devuelto por createExtension (IllegalStateException).
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh (identidad, una sola creacion bajo concurrencia). Procedencia: jar oficial.
- **Decisiones**: Q-T-EXT-2;Q-X-2

## AT-EXT-001.2: Extension: fallo de creacion cacheado (public)

- **Contrato observable**: Contractual: si createExtension lanza, la excepcion queda almacenada en el registro (replace(latch, throwable)) y cada acceso posterior (registerExtension, extension, hasExtension, apply) relanza la MISMA excepcion; un sistema no arranca si una extension de la configuracion falla al crearse. El cacheo esta SOLO en el codigo: el spec no lo afirma (ver A1).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:91-136
- **Condiciones**: replace(ext, latch, t) y throw t en createExtensionInstance (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:110-114); findExtension: case t: Throwable => throw t (134).
- **Orden / concurrencia / ciclo de vida**: countDown en finally garantiza que los hilos en espera continuan y ven el Throwable.
- **Dependencias**: filas: AC-EXT. Externas: Typesafe Config (getStringList), DynamicAccess/reflexion JVM, CountDownLatch/ConcurrentHashMap.
- **Equivalente Rust propuesto**: Registro con Result<Arc<T>,Arc<Error>> cacheado y clonado en cada acceso. **Diferencias observables**: Rust: error clonable (Arc<dyn Error>) para relanzar "la misma" excepcion.
- **Aserciones**:
  - A1. Crear un ActorSystem cuya extension de `pekko.actor.typed.extensions` falla en createExtension lanza RuntimeException; un SEGUNDO ActorSystem nuevo (la funcion `create()` construye otro sistema) tambien lanza. El comentario del test dice "a retry to create on each access", pero el test NO accede dos veces al mismo registro: no afirma el cacheo — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:136-153
  - A2. no afirmada por spec: la misma excepcion en cada acceso del mismo sistema (derivada de actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:134). Caracterizacion propuesta: registrar la extension que falla en un sistema vivo (via ExtensionId.apply) dos veces y comprobar `eq` entre las excepciones; oraculo jshell.
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-T-EXT-3

## AT-EXT-001.3: Extension: extension() sin registro y hasExtension (public)

- **Contrato observable**: Contractual: extension(id) sin registrar lanza IllegalArgumentException "Trying to get non-registered extension [<id>]" (el `<id>` es el toString por defecto del ExtensionId, p.ej. `...DummyExtension1$@<hash>`: no estable); hasExtension(id) es true solo si hay instancia (o si hay un Throwable almacenado lo relanza); findExtension espera si la creacion esta en curso. CONTRADICCION con la documentacion: el Scaladoc de `Extensions.extension` dice que lanza IllegalStateException (actor-typed/src/main/scala/org/apache/pekko/actor/typed/Extensions.scala:154-160) pero la implementacion lanza IllegalArgumentException (ExtensionsImpl.scala:81); manda el codigo.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:78-89,128-136; actor-typed/src/main/scala/org/apache/pekko/actor/typed/Extensions.scala:145-167
- **Condiciones**: Error: IllegalArgumentException (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:81). hasExtension = findExtension(ext) != null (78).
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: filas: AC-EXT. Externas: Typesafe Config (getStringList), DynamicAccess/reflexion JVM, CountDownLatch/ConcurrentHashMap.
- **Equivalente Rust propuesto**: Result<Arc<T>,ExtensionError::NotRegistered> y has_extension(). **Diferencias observables**: Rust: Err en vez de excepcion; texto a conservar.
- **Aserciones**:
  - A1. hasExtension(DummyExtension1) y hasExtension(SlowExtension) true tras cargarlas por config; extension(...) devuelve la instancia — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:121-134
  - A2. hasExtension true tambien con extension sustituida por ExtensionSetup — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:245-251
  - A3. Java: `system.hasExtension(MyExtension.getInstance())` true tras cargarla por config — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/ExtensionsTest.java:71
  - A4. extension() no registrada: IllegalArgumentException "Trying to get non-registered extension [..]" — no afirmada por spec (derivada de actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:80-82; ningun test del repositorio contiene ese texto). Caracterizacion propuesta: oraculo jshell.
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh (jshell: extension de id no registrado). Procedencia: jar oficial.
- **Decisiones**: Q-T-EXT-2

## AT-EXT-002.1: Carga desde configuracion: pekko.actor.typed.library-extensions (public)

- **Contrato observable**: Contractual: library-extensions (default [] + Receptionist$ en actor-typed/src/main/resources/reference.conf:18-21; se extiende con +=) se cargan al arrancar y un fallo de CARGA (clase ausente: RuntimeException "While trying to load extension [<fqcn>]" con causa; no es ExtensionId: RuntimeException "[<fqcn>] is not an 'ExtensionId'") o de creacion hace fallar el arranque del sistema; el FQCN puede terminar en `$` (objeto Scala) o la clase exponer getInstance() estatico (Java, resuelto con getDeclaredMethod("getInstance") e invoke(null), ExtensionsImpl.scala:69-76). La fila AT-EXT-002.2 cubre la lista `extensions` (que ademas se carga en el mismo mecanismo y es la que prueba ExtensionsTest.java).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:38-76; actor-typed/src/main/resources/reference.conf:5-21
- **Condiciones**: Orden: loadExtensionsFor("pekko.actor.typed.library-extensions", throwOnLoadFail=true) y luego "...extensions" (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:40-43).
- **Orden / concurrencia / ciclo de vida**: Se carga en el arranque del sistema, antes de aceptar mensajes.
- **Dependencias**: filas: AC-EXT; AC-CFG. Externas: Typesafe Config (getStringList), DynamicAccess/reflexion JVM, CountDownLatch/ConcurrentHashMap.
- **Equivalente Rust propuesto**: Lista de nombres sustituida por registro explicito (Q-X-2): SystemBuilder::library_extension(id). **Diferencias observables**: Registro explicito: no hay carga por nombre; el error de "clase ausente" se vuelve error de compilacion/registro.
- **Aserciones**:
  - A1. library-extensions con una extension cuyo createExtension falla -> RuntimeException — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:173-183
  - A2. library-extensions con una extension inexistente (clase ausente) -> RuntimeException (no se afirma el mensaje ni la causa) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:185-193
  - A3. las extensiones listadas se crean al arrancar (createCount > 0 y la lista no esta vacia) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:165-171
  - A4. CORRECCION: el test "load an extension implemented in Java" NO usa getInstance() desde la configuracion: solo llama a `ExtensionsTest.MyExtension.get(sys)` y compara instancias — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:195-202. La carga Java via getInstance() se afirma en ExtensionsTest.java (clave `extensions`, ver AT-EXT-002.2 A5).
  - A5. no afirmada por spec: el texto "[<fqcn>] is not an 'ExtensionId'" y la causa de "While trying to load extension [..]". Caracterizacion propuesta: oraculo jshell con una clase que no es ExtensionId y con una ausente, comparando mensaje y causa.
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-X-2

## AT-EXT-002.2: Carga desde configuracion: pekko.actor.typed.extensions (public)

- **Contrato observable**: Contractual: extensions se cargan despues de library-extensions; un fallo de CARGA (clase ausente o no es ExtensionId) NO falla el arranque: se loguea error ("[<fqcn>] is not an 'ExtensionId', skipping..." o "While trying to load extension <fqcn>, skipping..." sin corchetes) y sigue; un fallo de CREACION (createExtension lanza) si falla el arranque (RuntimeException, la original sin envolver), cacheado (ver EXT-001.2).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:45-66
- **Condiciones**: throwOnLoadFail=false solo cambia el tratamiento de Success(_) no-ExtensionId y Failure(problem) de la resolucion (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:56-65).
- **Orden / concurrencia / ciclo de vida**: Segunda pasada tras library-extensions.
- **Dependencias**: filas: AC-EXT; AC-CFG. Externas: Typesafe Config (getStringList), DynamicAccess/reflexion JVM, CountDownLatch/ConcurrentHashMap.
- **Equivalente Rust propuesto**: Registro explicito con politica "skip" solo para errores de resolucion de nombre. **Diferencias observables**: Sin carga por nombre, el caso "skip" desaparece salvo que el registro acepte nombres opcionales.
- **Aserciones**:
  - A1. extensions por configuracion cargan DummyExtension1$ y SlowExtension$ (hasExtension true) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:121-134
  - A2. extension de `pekko.actor.typed.extensions` que falla al CREARSE hace fallar el sistema (no se omite) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:136-153
  - A3. ExtensionSetup sustituye DummyExtension1 (listada en extensions) por DummyExtension1ViaSetup — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:236-252
  - A4. el caso "skip" por clase ausente o por no ser ExtensionId NO esta afirmado por spec (derivada de actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:56-65). Caracterizacion propuesta: oraculo jshell que arranque un sistema con `extensions = ["no.existe"]` y compruebe que arranca y que se loguea el error.
  - A5. Java: `pekko.actor.typed.extensions += "org.apache.pekko.actor.typed.ExtensionsTest$MyExtension"` (clase con getInstance() estatico, sin `$`) se carga: hasExtension(MyExtension.getInstance()) es true y get(system) devuelve la misma instancia dos veces — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/ExtensionsTest.java:60-80
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh (incluyendo el caso de clase ausente en extensions, a validar). Procedencia: jar oficial.
- **Decisiones**: Q-X-2;Q-T-EXT-1

## AT-EXT-002.3: Carga de extensiones tipadas desde el sistema clasico (LoadTypedExtensions) (public)

- **Contrato observable**: Contractual: el sistema clasico lista pekko.library-extensions += LoadTypedExtensions (reference.conf:36), que fuerza la creacion de la ActorSystemAdapter (`AdapterExtension(system).adapter.loadExtensions()`) y la carga de las extensiones tipadas, de modo que las extensiones registradas se crean eagerly aun cuando el sistema se cree como clasico y solo despues se acceda con toTyped.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorSystemAdapter.scala:143-176 (LoadTypedExtensions 158-173); actor-typed/src/main/resources/reference.conf:35-36
- **Condiciones**: before == beforeCreation + 1 y after == before (actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:213-216; el valor +1 supone que InstanceCountingExtension esta en library-extensions por la config del test, ExtensionsSpec.scala:79-85).
- **Orden / concurrencia / ciclo de vida**: Eager en arranque.
- **Dependencias**: filas: AT-ADP-008; AC-EXT. Externas: Typesafe Config (getStringList), DynamicAccess/reflexion JVM, CountDownLatch/ConcurrentHashMap.
- **Equivalente Rust propuesto**: n/a si no hay sistema clasico (Q-X-8 b). **Diferencias observables**: Solo relevante con interop clasico.
- **Aserciones**:
  - A1. la extension listada se crea durante la construccion del sistema clasico y el acceso posterior no la recrea — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:204-220
  - A2. con un sistema clasico, la ActorSystemAdapter es unica: dos `toTyped` dan la misma instancia de extension — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:222-234
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-X-8

## AT-EXT-003: ExtensionSetup / AbstractExtensionSetup (override en tests) (public)

- **Contrato observable**: Contractual: un ExtensionSetup(extId, createExtension) incluido en ActorSystemSetup reemplaza la fabrica de la extension: createExtensionInstance busca el primer setup cuyo extId == ext (igualdad de referencia) y lo usa en vez de ext.createExtension. Limite: ActorSystemSetup guarda los Setup en un Map indexado por la CLASE del setup (`setups + (t.getClass -> t)`), de modo que dos instancias de la misma clase de ExtensionSetup se reemplazan entre si (la ultima gana); cada extension necesita su propia subclase de ExtensionSetup para convivir con otras.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/Extensions.scala:169-187; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:96-101; actor/src/main/scala/org/apache/pekko/actor/setup/ActorSystemSetup.scala:64,86
- **Condiciones**: collectFirst sobre settings.setup.setups (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:97-100); el setup no se usa si la extension ya estaba registrada.
- **Orden / concurrencia / ciclo de vida**: En la creacion de la extension.
- **Dependencias**: filas: AC-EXT. Externas: Typesafe Config (getStringList), DynamicAccess/reflexion JVM, CountDownLatch/ConcurrentHashMap.
- **Equivalente Rust propuesto**: Builder del sistema acepta ExtensionOverride(id, factory). **Diferencias observables**: Ninguna (la limitacion de "una por clase de setup" no se porta si el builder admite varios overrides: documentar).
- **Aserciones**:
  - A1. extension sustituida por setup: extension(...) es DummyExtension1ViaSetup y DummyExtension1(sys) devuelve la misma instancia — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:236-252 (clases de apoyo: 37-40)
  - A2. Java: MyExtensionSetup(sys -> new MyExtImplViaSetup()) sustituye a MyExtension; get(system) devuelve MyExtImplViaSetup y es estable (assertSame) — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/ExtensionsTest.java:95-113
  - A3. no afirmada por spec: dos ExtensionSetup de la misma clase se pisan en ActorSystemSetup. Caracterizacion propuesta: prueba con dos instancias y comprobar que solo se aplica la ultima.
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: ninguna especifica (hereda Q-X-* citadas en notes)

## AT-DSP-001: Props tipado (empty, encadenado, firstOrElse, allOf) (public)

- **Contrato observable**: Contractual: Props es una lista enlazada inmutable de selectores (DispatcherSelector, MailboxSelector, ActorTags); withDispatcherDefault/FromConfig/SameAsParent/withMailboxFromConfig ANTEPONEN el selector (el ultimo agregado es el primero de la lista); firstOrElse[T](default) devuelve el primero de ese tipo (el mas reciente gana) o el default; allOf[T] devuelve los de ese tipo en orden de la lista (cada uno aislado con EmptyProps); filterNot elimina un tipo; EmptyProps.next lanza NoSuchElementException "EmptyProps has no next".
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/Props.scala:26-152; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/PropsImpl.scala:24-78
- **Condiciones**: ActorTagsImpl con tags null lanza IllegalArgumentException "Tags must not be null" (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/PropsImpl.scala:68-71). PropsAdapter usa solo firstOrElse para dispatcher y mailbox y para tags (el primer ActorTags gana; los otros se ignoran).
- **Orden / concurrencia / ciclo de vida**: n/a: valor inmutable.
- **Dependencias**: filas: AC-CFG. Externas: Dispatchers/Mailboxes clasicos y Typesafe Config.
- **Equivalente Rust propuesto**: struct Props{selectors: Vec<Selector>} con builder; first_of::<T>() y all_of::<T>(). **Diferencias observables**: Rust: Vec ordenado o lista inmutable; mantener orden "ultimo agregado primero".
- **Aserciones**:
  - A1. firstOrElse[DispatcherSelector](null) == dispatcherFirst (el mas reciente) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/PropsSpec.scala:27-29
  - A2. allOf[DispatcherSelector] == [DispatcherSelector.default(), DispatcherSelector.fromConfig("pool")] — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/PropsSpec.scala:31-34
  - A3. ActorTags(null) -> IllegalArgumentException (no afirmada por spec; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/PropsImpl.scala:68-71)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh (orden de allOf/firstOrElse; son private[pekko]: via jshell con acceso reflexivo). Procedencia: jar oficial.
- **Decisiones**: Q-T-DSP-1

## AT-DSP-002: DispatcherSelector: default, blocking, fromConfig, sameAsParent (public)

- **Contrato observable**: Contractual: selecciona el dispatcher del actor: default() -> pekko.actor.default-dispatcher; blocking() = fromConfig("pekko.actor.default-blocking-io-dispatcher"); fromConfig(path); sameAsParent() usa el del padre (y en el guardian de usuario el default); un dispatcher desconocido hace fallar el spawn (ConfigurationException de classic); el ActorTestKit registra "Spawn failed, props [<props>]" y devuelve un ref a un actor detenido.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/Props.scala:154-204; actor-typed/src/main/scala/org/apache/pekko/actor/typed/Dispatchers.scala:20-40; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/PropsAdapter.scala:33-38; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestKitUtils.scala:62-70; docs/src/main/paradox/typed/dispatchers.md:42-86
- **Condiciones**: sameAsParent en cadena (varios niveles) conserva el del ancestro con dispatcher explicito (actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/DispatcherSelectorSpec.scala:108-120). Spawn con dispatcher desconocido: error "Spawn failed" y termina el ref.
- **Orden / concurrencia / ciclo de vida**: La eleccion se fija al crear el actor (props -> classic Props.withDispatcher).
- **Dependencias**: filas: AC-DSP; AC-CFG; Q-X-4. Externas: Dispatchers/Mailboxes clasicos y Typesafe Config.
- **Equivalente Rust propuesto**: DispatcherSelector::{default,blocking,from_config(path),same_as_parent}; mapeo a pools tokio/threads segun Q-X-4. **Diferencias observables**: Rust: "nombre de hilo" no es observable igual; usar nombre de runtime/pool.
- **Aserciones**:
  - A1. hilo empieza por "DispatcherSelectorSpec-ping-pong-dispatcher" con Props.empty.withDispatcherFromConfig — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/DispatcherSelectorSpec.scala:69-76
  - A2. idem con DispatcherSelector.fromConfig — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/DispatcherSelectorSpec.scala:78-85
  - A3. dispatcher desconocido: error "Spawn failed" y el ref termina — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/DispatcherSelectorSpec.scala:87-93
  - A4. sameAsParent (un nivel y varios) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/DispatcherSelectorSpec.scala:95-120
  - A5. sameAsParent para el guardian de usuario usa el default — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/DispatcherSelectorSpec.scala:122-135
  - A6. blocking() -> "pekko.actor.default-blocking-io-dispatcher" — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/routing/PoolRouterSpec.scala:52-61
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh (nombre de hilo, error de dispatcher desconocido). Procedencia: jar oficial.
- **Decisiones**: Q-X-4;Q-T-DSP-2

## AT-DSP-003: MailboxSelector: default (SingleConsumerOnlyUnboundedMailbox), bounded(capacity), fromConfig (public)

- **Contrato observable**: Contractual: default() = fromConfig("pekko.actor.typed.default-mailbox") (single consumer, sin limite; PropsAdapter lo traduce a Props.mailbox == "pekko.actor.typed.default-mailbox"); bounded(n) usa el mailbox clasico de capacidad n (prefijo BoundedCapacityPrefix+n): lleno -> el mensaje va a dead letters; fromConfig(path) usa la config dada (p.ej. NonBlockingBoundedMailbox capacidad 4).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/Props.scala:206-236; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/PropsAdapter.scala:40-50; actor-typed/src/main/resources/reference.conf:23-32; docs/src/main/paradox/typed/mailboxes.md:32-70
- **Condiciones**: default() devuelve MailboxFromConfigSelector, por lo que el caso DefaultMailboxSelector de PropsAdapter solo se alcanza con el constructor interno (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/PropsImpl.scala:53-58).
- **Orden / concurrencia / ciclo de vida**: El actor puede estar bloqueado procesando un mensaje mientras se llena el mailbox (test de capacidad 2).
- **Dependencias**: filas: AC-MBX; AC-CFG. Externas: Dispatchers/Mailboxes clasicos y Typesafe Config.
- **Equivalente Rust propuesto**: MailboxSelector::{Default, Bounded(n), FromConfig(path)}. **Diferencias observables**: Mailbox lleno -> dead letters en Pekko; Rust debe definir el mismo destino.
- **Aserciones**:
  - A1. actor sin selector tiene mailbox UnboundedMessageQueueSemantics — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/MailboxSelectorSpec.scala:62-66
  - A2. bounded(3) -> BoundedMessageQueueSemantics — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/MailboxSelectorSpec.scala:68-73
  - A3. bounded(2): "one" bloquea, "two" cabe, "three" y "four" desbordan -> dead letters (LoggingTestKit.deadLetters) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/MailboxSelectorSpec.scala:75-95
  - A4. fromConfig("specific-mailbox") -> BoundedNodeMessageQueue con capacidad 4 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/MailboxSelectorSpec.scala:97-103
  - A5. PropsAdapter(Props.empty).mailbox == "pekko.actor.typed.default-mailbox" — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/adpater/PropsAdapterSpec.scala:28-32
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh (tipo de cola, overflow a dead letters). Procedencia: jar oficial.
- **Decisiones**: Q-T-DSP-3

## AT-DSP-004: ActorTags (public)

- **Contrato observable**: `ActorTags(tag, otros*)`, `ActorTags(Set)` y `ActorTags.create(...)` (Java) construyen un `Props` que añade tags al actor; `ActorTagsImpl` valida que el `Set` no sea `null` (`IllegalArgumentException("Tags must not be null")`). Solo el primer `ActorTags` de la cadena de Props se aplica (`firstOrElse` devuelve el primero, es decir el último añadido); los tags quedan en `Deploy.tags` del actor clásico (`withActorTags`) solo si no están vacíos; se publican en el MDC `pekkoTags` como cadena separada por comas (`tags.mkString(",")`) y la clave solo se pone si la cadena no es vacía.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/Props.scala:238-286 (ActorTags y factories); actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/PropsImpl.scala:68-76 (ActorTagsImpl); actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/PropsAdapter.scala:53-55; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorContextImpl.scala:49-56 (tagsString) y 167; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorMdc.scala:36-38
- **Precondiciones**: los tags se pasan al crear el actor (spawn con Props); no hay API para cambiarlos después.
- **Comportamiento**: `Props.firstOrElse[ActorTags]` ignora los demás `ActorTags` de la cadena (AT-DSP-001, Q-T-DSP-1). El conjunto vacío se representa como "sin tags": no toca `Deploy` ni el MDC. El orden de los tags en `pekkoTags` es el de iteración del `Set` (no determinista para más de 4 elementos en Scala; para ≤4 es de inserción).
- **Errores**: `IllegalArgumentException("Tags must not be null")` si `tags == null` (PropsImpl.scala:69-70); `ActorTags.apply(Set.empty)` no falla (construye el vacío).
- **Orden / concurrencia / ciclo de vida**: n/a (se leen una vez al crear el LoggingContext perezoso del actor, ActorContextImpl.scala:160-171).
- **Dependencias**: AT-DSP-001; AT-LOG-002; AT-CTX-002
- **Equivalente Rust propuesto**: `ActorTags(Vec<String>)` como variante de `Props` con `ActorTags::new(impl IntoIterator<Item = impl Into<String>>)`; el vector conserva el orden de inserción (más determinista que Pekko; a documentar como refinamiento, Q-T-LOG-1). **Se preserva**: tags en Deploy y MDC/campo `pekko_tags` unidos con coma; vacío = ausente; el primero gana. **Diferencias/límites**: `null` no existe (se elimina el error); el orden del `Set` de Pekko no es contractual.
- **Aserciones**:
  - A1. tags `tag1,tag2` pasados por Props aparecen en el MDC `pekkoTags` tanto en el log de arranque como en el de un mensaje — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.scala:270-289 (afirma `withMdc(Map(ActorMdc.PekkoTagsKey -> "tag1,tag2"))` en "Starting up" y "Got message"; el orden "tag1,tag2" lo cumple el `Set` de 2 elementos en Scala)
  - A2. `tags == null` -> `IllegalArgumentException("Tags must not be null")` — no afirmada por spec (PropsImpl.scala:69-70); caracterización propuesta: construir `ActorTagsImpl(null)`
  - A3. el primero de varios `ActorTags` de la cadena es el que se aplica — no afirmada por spec para ActorTags (PropsSpec cubre `firstOrElse` genérico para otros selectores, AT-DSP-001); caracterización propuesta: `Props.empty.withNext(ActorTags("a")).withNext(ActorTags("b"))` aplica "b"
  - A4. tags vacíos no ponen `pekkoTags` en el MDC ni `withActorTags` en el Deploy — no afirmada por spec (PropsAdapter.scala:54; ActorMdc.scala:36-37); caracterización propuesta: log de un actor sin tags no contiene la clave
- **Validación prevista**: A1 portado con el mecanismo de captura de logs de AT-TK-006; A2-A4 caracterización unitaria. Oráculo: capturar el MDC de un evento del jar oficial (diferencial sobre la cadena `tag1,tag2`).
- **Decisiones**: Q-T-LOG-1 (nombre de la clave `pekkoTags` y representación de MDC en Rust); Q-T-DSP-1 (el último añadido gana)

## AT-DSP-005: Dispatchers.lookup y ids (DefaultDispatcherId, InternalDispatcherId) (public)

- **Contrato observable**: `ActorSystem.dispatchers.lookup(selector)` devuelve un `ExecutionContextExecutor`: `DispatcherDefault` y `DispatcherSameAsParent` -> `system.dispatcher`; `DispatcherFromConfig(path)` -> `system.dispatchers.lookup(path)` (clásico: falla si la ruta no existe); un selector desconocido lanza `RuntimeException("Unsupported dispatcher selector: <x>")`; `shutdown()` es un no-op (en clásico no había shutdown). Constantes: `Dispatchers.DefaultDispatcherId = "pekko.actor.default-dispatcher"` (pública) y `Dispatchers.InternalDispatcherId = "pekko.actor.internal-dispatcher"` (@InternalApi, usada por el receptionist, ReceptionistImpl.scala:49). En el `ActorSystemStub` del testkit, `lookup` devuelve siempre el `ControlledExecutor`.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/Dispatchers.scala:19-40; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorSystemAdapter.scala:95-103; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/ReceptionistImpl.scala:49; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/ActorSystemStub.scala:97-100
- **Precondiciones**: sistema tipado vivo; para `fromConfig`, la ruta debe existir en la configuración del sistema.
- **Comportamiento**: `sameAsParent` en lookup directo no tiene "padre" y se resuelve como el dispatcher por defecto (a diferencia de `spawn`, donde `PropsAdapter` usa `Deploy.DispatcherSameAsParent`, PropsAdapter.scala:32).
- **Errores**: `RuntimeException("Unsupported dispatcher selector: <x>")` para selectores no previstos (ActorSystemAdapter.scala:101); ruta inexistente: excepción del `Dispatchers.lookup` clásico (`ConfigurationException`, ver AC-DSP). No se afirma por spec.
- **Orden / concurrencia / ciclo de vida**: n/a (devuelve la referencia cacheada del dispatcher).
- **Dependencias**: AC-DSP; AT-DSP-002; AT-SYS-007.4; AT-TK-011
- **Equivalente Rust propuesto**: `Dispatchers::lookup(&DispatcherSelector) -> Executor` con la semántica anterior; `Executor` = handle del runtime (Q-T-DSP-2). **Se preserva**: tabla selector -> executor, constantes de id. **Diferencias/límites**: `RuntimeException` -> `Err(UnsupportedDispatcherSelector)`; con enum cerrado de selectores la rama "desconocido" desaparece.
- **Aserciones**:
  - A1. selección por nombre vía `Props.withDispatcherFromConfig` / `DispatcherSelector.fromConfig` ejecuta en hilos del dispatcher configurado — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/DispatcherSelectorSpec.scala:67-85 (indirecto: usa spawn, no `lookup` directo; afirma `threadName should startWith("DispatcherSelectorSpec-ping-pong-dispatcher")`); `sameAsParent` hereda — 96-120 y usa `pekko.actor.default-dispatcher` sin padre configurado — 122-134
  - A2. `lookup(selector)` directo (default/sameAsParent/fromConfig) y `Unsupported dispatcher selector` — no afirmada por spec leída (ningún test llama a `system.dispatchers.lookup` en actor-typed-tests); caracterización propuesta: tres casos directos y selector inválido en una prueba unitaria
  - A3. `shutdown()` no tiene efecto — no afirmada por spec; derivada de ActorSystemAdapter.scala:102
  - A4. ids de dispatcher: `DefaultDispatcherId` se usa en documentación (`withDispatcherFromConfig(DefaultDispatcherId)`) — actor-typed-tests/src/test/scala/docs/org/apache/pekko/typed/MailboxDocSpec.scala:62 (ejemplo de docs; sin aserción propia del valor)
- **Validación prevista**: A1 portado (compartido con AT-DSP-002); A2/A3 caracterización; las constantes se verifican por igualdad de cadenas en un test unitario. Oráculo: valor de las constantes y excepción de selector (jshell).
- **Decisiones**: Q-X-4 (runtime/ejecutores); Q-T-DSP-2 (mapeo de selectores a ejecutores Rust)

## AT-DLV-001.1: ProducerController: protocolo Start/RequestNext y demanda local uno a uno (api-may-change)

- **Contrato observable**: El productor de aplicación envía `ProducerController.Start(producer)` (normalmente un adaptador de mensajes que traduce `RequestNext` a su protocolo) y recibe `RequestNext(producerId, currentSeqNr, confirmedSeqNr, sendNextTo, askNextTo)`: está autorizado a enviar **un solo** mensaje por `RequestNext`, por `sendNextTo` (tell) o por `askNextTo` (con `MessageWithConfirmation`, ver AT-DLV-001.5), y debe esperar el siguiente `RequestNext`. `currentSeqNr` es el seqNr que tendrá el próximo mensaje; `confirmedSeqNr` el mayor confirmado. El primer `RequestNext` se envía al activarse el controlador cuando no hay mensajes sin confirmar; el primer mensaje de un flujo se envía al consumidor **sin esperar** un `Request` (`requestedSeqNr` inicial = `currentSeqNr`), después de lo cual el siguiente `RequestNext` solo llega cuando el consumidor da demanda (`Request`). Un `Start` nuevo (productor reiniciado) reemplaza al productor y, si había demanda pendiente y no quedan chunks por enviar, reenvía el `RequestNext`.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/ProducerController.scala:38-123 (Scaladoc y mensajes `Start`/`RequestNext`), 255-268 (factories `apply`); actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/internal/ProducerControllerImpl.scala:131-143 (State), 145-172 (apply), 241-263 (createState), 265-318 (waitingForInitialization), 320-345 (becomeActive), 347-350 (enforceLocalProducer), 408-456 (onMsg y checkOnMsgRequestedState), 687-695 (receiveStart), 773-844 (receiveMessage)
- **Precondiciones**: el productor y el `ProducerController` son locales (comprobación en ejecución). El flujo no arranca hasta tener `Start`, un `ConsumerController` (vía `RegisterConsumer`, AT-DLV-001.4, o `ConsumerController.RegisterToProducerController`) y, si hay cola duradera, el estado cargado (AT-DLV-001.6): `waitingForInitialization` guarda lo que va llegando y pasa a activo cuando están los tres.
- **Comportamiento**: (a) `becomeActive`: sin mensajes sin confirmar -> `RequestNext(producerId, currentSeqNr, confirmedSeqNr, msgAdapter, self)` y `requested = true`; con sin confirmar (reinicio duradero) -> no se pide nada al productor y se auto-envía `ResendFirst` (336-340). (b) Tras enviar un mensaje: si `currentSeqNr == requestedSeqNr` espera demanda (`requested = false`; se mantiene `true` mientras queden chunks); si no y es el último chunk, envía `RequestNext(currentSeqNr + 1, confirmedSeqNr)` y `requested = true` (427-438). (c) `producerId` se propaga a los `SequencedMessage` y como MDC `"producerId"` (152, `Behaviors.withMdc(staticMdc = Map("producerId" -> producerId))`); el logger se llama `org.apache.pekko.actor.typed.delivery.ProducerController` (153).
- **Errores**: un mensaje del productor sin demanda (`!requested` o `currentSeqNr > requestedSeqNr`) lanza `IllegalStateException("Unexpected Msg when no demand, requested <r>, requestedSeqNr <n>, currentSeqNr <m>")` (450-456) y hace fallar al controlador (supervisión por defecto: stop). Un mensaje inesperado de otro tipo lanza `RuntimeException("Unexpected message: <msg>")` (842-843). Productor no local: `IllegalArgumentException("Consumer [<ref>] should be local.")` (348-349; el texto dice "Consumer" aunque se trate del productor: se conserva tal cual). `Request` con `confirmedSeqNr > requestUpToSeqNr` lanza `IllegalArgumentException("requirement failed: confirmedSeqNr [..] should be <= requestUpToSeqNr [..]")` al construirlo (106-109).
- **Orden / concurrencia / ciclo de vida**: un solo `RequestNext` pendiente a la vez; los `Msg` llegan por el adaptador interno del controlador (`msgAdapter`), por lo que van por el mismo buzón que `Request`/`Ack`. Estado inmutable en el actor (sin hilos compartidos).
- **Dependencias**: AT-CTX-011 (messageAdapter); AT-TMR (timers); AT-DLV-001.2; AT-DLV-001.4; AT-DLV-001.6; AT-DLV-002.3 (el otro extremo); AT-DLV-006 (Settings)
- **Equivalente Rust propuesto**: actor `ProducerController<A>` con `enum ProducerCommand<A> { Start(ActorRef<RequestNext<A>>), RegisterConsumer(ActorRef<ConsumerCommand<A>>), Msg(A), MessageWithConfirmation{..}, Request{..}, Resend{..}, Ack{..}, .. }`; `RequestNext<A> { producer_id, current_seq_nr, confirmed_seq_nr, send_next_to, ask_next_to }`. Solo variante **local**: la comprobación de localidad se reduce a una tautología (sin remoting, Q-X-10). El `ClassTag[A]` desaparece (el tipo es estático). **Se preserva**: un `RequestNext` por mensaje, primer mensaje sin demanda, reemplazo por `Start` nuevo, mensajes de error de estado. **Diferencias/límites**: `IllegalStateException` pasa a `Err`/pánico supervisado según Q-X-5; `ApiMayChange`: la API puede cambiar en Pekko, se porta la de 1.7.1.
- **Aserciones**:
  - A1. tras `Start` + `RegisterConsumer`, el productor recibe `RequestNext` y su primer mensaje llega al consumidor como `SequencedMessage(seqNr 1, first)` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ProducerControllerSpec.scala:47-61
  - A2. cada mensaje exige un nuevo `RequestNext`; los seqNr son 1,2,3,4,5 sin saltos — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ProducerControllerSpec.scala:74-109
  - A3. reinicio del productor: un `Start` nuevo recibe `RequestNext` con `currentSeqNr` coherente (tras msg-2, `currentSeqNr == 3`) y los mensajes siguientes siguen con seqNr 3 y 4 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ProducerControllerSpec.scala:229-261
  - A4. uso punto a punto de extremo a extremo (productor de prueba + consumidor de prueba, `RegisterToProducerController`; el consumidor recibe los 42 mensajes) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ReliableDeliverySpec.scala:51-71; y con productor lento/consumidor rápido y viceversa — 124-134
  - A5. sustitución del productor (nuevo `ProducerController` con el mismo `producerId`, nuevo `Start` y `RegisterConsumer` al mismo `ConsumerController`): el consumidor sigue recibiendo msg-3 y msg-4 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ReliableDeliverySpec.scala:165-205
  - A6. mensaje sin demanda -> `IllegalStateException("Unexpected Msg when no demand, ...")` — no afirmada por spec (ProducerControllerImpl.scala:450-456); caracterización propuesta: enviar dos mensajes seguidos por el mismo `sendNextTo`
  - A7. productor no local -> `IllegalArgumentException` — no afirmada por spec (ProducerControllerImpl.scala:347-350); no aplicable sin remoting
  - A8. `Request` con `confirmedSeqNr > requestUpToSeqNr` -> `IllegalArgumentException` — no afirmada por spec (ProducerControllerImpl.scala:106-109); caracterización propuesta: construir `Request(3, 2, ..)`
  - A9. (implícita) los productores de prueba lanzan `IllegalStateException("Unexpected RequestNext, already got one.")` si reciben un segundo `RequestNext` sin haber enviado: los tests de extremo a extremo pasan, luego el `ProducerController` nunca emite dos `RequestNext` seguidos — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/TestProducer.scala:66-69 (y TestProducerWithAsk.scala:86, TestProducerWorkPulling.scala:61)
- **Validación prevista**: A1-A5 portados 1:1 (los de tiempo real de A1 se ejecutan con `ManualTime` de AT-TK-007 o con tolerancias equivalentes); A6/A8 caracterización unitaria. Diferencial con el jar oficial (pares locales Producer/ConsumerController, escenario jshell) comparando la secuencia de `RequestNext(currentSeqNr, confirmedSeqNr)` y de `SequencedMessage`.
- **Decisiones**: Q-T-DLV-1 (alcance: variante local, `ApiMayChange`); Q-X-8 (actores sobre qué runtime); Q-X-5 (tratamiento de IllegalStateException)

## AT-DLV-001.2: ProducerController: secuenciación, ventana de demanda y confirmación por Request/Ack (api-may-change)

- **Contrato observable**: Cada mensaje se envuelve en `SequencedMessage(producerId, seqNr, message, first, ack)(self)` con `seqNr` monótono sin huecos desde 1 (o desde el `currentSeqNr` cargado de la cola duradera); `first = (seqNr == firstSeqNr)`. El `ConsumerController` informa con `Request(confirmedSeqNr, requestUpToSeqNr, supportResend, viaTimeout)` hasta qué seqNr puede enviarse (`requestedSeqNr`), y que todo hasta `confirmedSeqNr` fue recibido y procesado; con `Ack(confirmedSeqNr)` confirma sin ampliar la ventana. El controlador no envía por encima de `requestedSeqNr` (salvo el primer mensaje) y guarda en `unconfirmed` los mensajes enviados hasta que los confirma (solo si `supportResend`).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/internal/ProducerControllerImpl.scala:102-119 (Request/Resend/Ack), 408-448 (onMsg), 464-524 (receiveRequest), 526-534 (receiveAck), 536-561 (onAck); actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/ConsumerController.scala:158-199 (SequencedMessage)
- **Precondiciones**: el controlador está activo (AT-DLV-001.1).
- **Comportamiento**: `receiveRequest`: (1) `onAck(newConfirmedSeqNr)`; (2) recorta `unconfirmed` (o lo vacía si `!supportResend`); (3) si `newRequestedSeqNr2 > requestedSeqNr` ensancha la ventana y, si no hay `RequestNext` pendiente, no hay almacenamiento en curso ni chunks pendientes y quedan huecos (`newRequestedSeqNr2 - currentSeqNr > 0`), envía `RequestNext(currentSeqNr, newConfirmedSeqNr)`; si hay chunks pendientes se auto-envía `SendChunk`. `onAck` calcula `confirmedSeqNr = max(actual, nuevo)`, responde a los `MessageWithConfirmation` ya cubiertos (AT-DLV-001.5), cancela `ResendFirst` si `newConfirmedSeqNr == firstSeqNr` (548-549) y, con cola duradera, envía `StoreMessageConfirmed(max, NoQualifier, ahora)` solo si el máximo creció (553-558). `receiveAck` con `newConfirmedSeqNr == firstSeqNr` y mensajes sin confirmar reenvía todos (530-532).
- **Errores**: ver AT-DLV-001.1 (sin demanda, Request inválido). Una `Request`/`Ack` con `confirmedSeqNr` menor que el ya confirmado no retrocede (`max`).
- **Orden / concurrencia / ciclo de vida**: `Request` y `Ack` pueden llegar fuera de orden o duplicados y se aplican por `max`; el orden de los `SequencedMessage` enviados es el de `seqNr`.
- **Dependencias**: AT-DLV-001.1; AT-DLV-001.3; AT-DLV-001.5; AT-DLV-001.6; AT-DLV-002.3
- **Equivalente Rust propuesto**: `struct SequencedMessage<A> { producer_id, seq_nr: u64, message: MessageOrChunk<A>, first: bool, ack: bool, producer_controller: ActorRef<ProducerInternal<A>> }`; `unconfirmed: VecDeque<SequencedMessage<A>>`; mismas reglas de ventana y `max`. **Se preserva**: seqNr sin huecos, `first`, ventana por `Request`, recorte por confirmación, el campo `confirmedSeqNr` en `RequestNext`. **Diferencias/límites**: `SeqNr = Long` -> `u64` (sin desbordamiento práctico); `Long.MaxValue` como sentinela de marca de tiempo solo en tests.
- **Aserciones**:
  - A1. el primer `Request(1, 10)` ensancha la ventana y los mensajes 2,3,4 salen por demanda con seqNr consecutivos — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ProducerControllerSpec.scala:74-109
  - A2. `Request(2,10)` confirma 1 y 2 y libera el reenvío de 3 y 4 (ver AT-DLV-001.3) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ProducerControllerSpec.scala:111-147
  - A3. con cola duradera, `Request(3L, 13L)` tras cargar sin confirmar 3 y 4 envía el 4 y luego `RequestNext` permite el 5 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/DurableProducerControllerSpec.scala:54-89
  - A4. `Ack(n)` y `Request(n, ..)` confirman por igual (los `MessageWithConfirmation` 1..5 responden con cualquiera de los dos) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ProducerControllerSpec.scala:187-227
  - A5. el estado duradero refleja `highestConfirmedSeqNr` y el par `(seqNr, timestamp)` tras `Request(1,10)` y `Ack(3)` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/DurableProducerControllerSpec.scala:91-144
  - A6. `Request` fuera de orden o con `confirmedSeqNr` menor no retrocede `confirmedSeqNr` — no afirmada por spec (ProducerControllerImpl.scala:551); caracterización propuesta: `Request(3,..)` seguido de `Request(1,..)` mantiene `RequestNext.confirmedSeqNr == 3`
- **Validación prevista**: A1-A5 portados 1:1; A6 caracterización. Diferencial con el jar oficial para la serie de `SequencedMessage`/`RequestNext` ante una secuencia fija de `Request`/`Ack`.
- **Decisiones**: Q-T-DLV-1; Q-T-DLV-2 (parte duradera)

## AT-DLV-001.3: ProducerController: reenvío por Resend, Request vencida y reenvío del primero (api-may-change)

- **Contrato observable**: Cuando el consumidor detecta una pérdida envía `Resend(fromSeqNr)` y el productor reenvía **todos** los no confirmados desde ese seqNr; si el último mensaje se pierde y no llegan más, el consumidor reenvía periódicamente `Request(.., viaTimeout = true)` y el productor reenvía todos los no confirmados. El primer mensaje (seqNr == `firstSeqNr`) se reenvía cada `durable-queue.resend-first-interval` (1 s) hasta que se confirma (`ResendFirst` por timer). Si `supportResend = false` (consumidor con `only-flow-control`) el productor no guarda ni reenvía nada y solo aplica el control de flujo; la ventana `requestedSeqNr` se amplía si todo lo enviado se perdió.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/internal/ProducerControllerImpl.scala:422-423 (inicio del timer ResendFirst), 464-524 (receiveRequest: 482-485 reenvío, 487-499 expansión de ventana), 526-534 (receiveAck 530-532), 646-654 (receiveResend), 656-664 (resendUnconfirmed), 666-673 (ResendFirstUnconfirmed), 675-685 (receiveResendFirst); actor-typed/src/main/resources/reference.conf:81-93 (durable-queue; `resend-first-interval = 1s` en la línea 93)
- **Precondiciones**: `supportResend` (último valor recibido en `Request`; por defecto `true` al cargar el estado, 256).
- **Comportamiento**: (a) `Resend(from)`: reenvía `unconfirmed.dropWhile(_.seqNr < from)`; con `from == 0` y mensajes sin confirmar marca el primero como `first` (`asFirst`) y lo guarda así (646-654). (b) `Request` con `viaTimeout` o con `newConfirmedSeqNr == firstSeqNr` y `supportResend` reenvía todos los no confirmados tras recortar (482-485). (c) `ResendFirst` (timer fijo de 1 s por defecto) reenvía `unconfirmed.head.asFirst` mientras `head.seqNr == firstSeqNr`; si ya no procede y `currentSeqNr > firstSeqNr` se cancela (675-685). Se arranca al enviar el primer mensaje (422-423) y se cancela al confirmarse (548-549). (d) `ResendFirstUnconfirmed` reenvía el primer no confirmado (interno, 666-673). (e) `!supportResend`: no se reenvía; si `newRequestedSeqNr <= currentSeqNr` la ventana se expande a `currentSeqNr + (newRequestedSeqNr - newConfirmedSeqNr)` para no quedar bloqueado (487-499).
- **Errores**: ninguno propio; si el consumidor no responde el reenvío del primero continúa indefinidamente.
- **Orden / concurrencia / ciclo de vida**: los reenvíos conservan el orden de `seqNr`; el reenvío del primero y los timers del consumidor (AT-DLV-002.3/002.4) son independientes; los duplicados se deduplican en el consumidor (AT-DLV-002.6).
- **Dependencias**: AT-DLV-001.2; AT-DLV-002.3; AT-DLV-002.4; AT-DLV-002.5; AT-TMR (timers)
- **Equivalente Rust propuesto**: timer `ResendFirst` con `tokio::time::interval`/`ManualTime` en tests; `unconfirmed: VecDeque` ordenado; la rama `!supportResend` mantiene `unconfirmed` vacío. **Se preserva**: qué se reenvía (todo desde `from`), cuándo (Resend, Request vencida o primer mensaje) y el modo sin reenvío. **Diferencias/límites**: el intervalo es de reloj real: las pruebas deben usar tiempo controlado.
- **Aserciones**:
  - A1. el primer `SequencedMessage` se reenvía una vez si no hay `Request` (misma `first = true`), y tras `Request(1,10)` no se reenvía más en 1,1 s — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ProducerControllerSpec.scala:47-72
  - A2. `Resend(3)` tras enviar 1..4 reenvía 3 y 4 y el 5 sigue con normalidad — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ProducerControllerSpec.scala:74-109
  - A3. `Request(2L, 10L, true, viaTimeout = true)` reenvía 3 y 4 (los no confirmados) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ProducerControllerSpec.scala:111-147
  - A4. con `supportResend = false` una `Request(.., viaTimeout = true)` no reenvía 3 y 4 (`expectNoMessage()`) y el 5 sigue con seqNr 5 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ProducerControllerSpec.scala:310-346
  - A5. con cola duradera y mensajes sin confirmar al arrancar, el primero se envía como `first` y no se reenvía en 50 ms (el intervalo es 1 s) hasta recibir `Request(3,13)` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/DurableProducerControllerSpec.scala:54-89
  - A6. con una red que pierde mensajes (10-30 %) hacia ambos lados el consumidor termina recibiendo todos los mensajes sin duplicados (el `TestConsumer` lanza ante duplicados) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ReliableDeliveryRandomSpec.scala:155-169 (y sin reenvío, 203-216)
  - A7. expansión de `requestedSeqNr` cuando todo lo enviado se perdió con `supportResend = false` — no afirmada por spec (ProducerControllerImpl.scala:487-499); caracterización propuesta: `Request(1, 3, false, false)` con `currentSeqNr = 5` debe dar `requestedSeqNr = 5 + 2 = 7`
  - A8. `Resend(0)` marca el primer no confirmado como `first` — no afirmada por spec directa (la rama usa el test de consumidor 438-483, ver AT-DLV-002.4)
- **Validación prevista**: A1-A5 portados (con tiempo controlado); A6 test aleatorio con semilla fija (ver AT-DLV-007 para la telemetría; la semilla del original es `System.currentTimeMillis()`: en Rust se fija la semilla y se registra); A7/A8 caracterización. Diferencial con el jar oficial para la secuencia de reenvíos ante `Resend`/`Request` guionizados.
- **Decisiones**: Q-T-DLV-1; Q-T-DLV-4

## AT-DLV-001.4: ProducerController: RegisterConsumer, reemplazo del ConsumerController y envío personalizado (api-may-change)

- **Contrato observable**: `RegisterConsumer(consumerController)` vincula el `ProducerController` con un `ConsumerController` (alternativa: `ConsumerController.RegisterToProducerController` del lado consumidor). Se puede enviar antes o después de `Start`; el controlador no arranca el flujo hasta tener ambos. Un `RegisterConsumer` posterior **reemplaza** al consumidor: el siguiente `firstSeqNr` pasa a ser el primer seqNr sin confirmar (o `currentSeqNr` si no hay), el primer sin confirmar se reenvía como `first` y se repite cada `resend-first-interval` hasta que el nuevo consumidor confirma; el resto de los no confirmados se reenvía cuando llega el `Request` que confirma el primero. Existe una variante interna `apply(producerId, durable, settings, send)` con función de envío propia (para Cluster Sharding) en la que `RegisterConsumer` no se usa.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/ProducerController.scala:142-152 (RegisterConsumer), 270-289 (variante interna `send`); actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/internal/ProducerControllerImpl.scala:145-206 (apply con y sin `send`), 265-318 (waitingForInitialization), 697-713 (receiveRegisterConsumer)
- **Precondiciones**: ver AT-DLV-001.1.
- **Comportamiento**: `receiveRegisterConsumer`: `newFirstSeqNr = unconfirmed.head.seqNr` o `currentSeqNr`; si hay sin confirmar arranca `ResendFirst` con el intervalo y se auto-envía `ResendFirst`; la función `send` pasa a `consumerController.tell(_)` (697-713). El `RegisterConsumer` de la variante interna con `send` usa `deadLetters` como consumidor ficticio y no espera registro (190-196).
- **Errores**: `RegisterConsumer` que repite el mismo consumidor reinicia el reenvío del primero (no hay deduplicación de registro en el productor; la deduplicación la hace el consumidor, AT-DLV-002.6).
- **Orden / concurrencia / ciclo de vida**: el reemplazo ocurre en el hilo del actor; los mensajes en vuelo del consumidor anterior se pierden y se recuperan por reenvío.
- **Dependencias**: AT-DLV-001.1; AT-DLV-001.3; AT-DLV-002.6
- **Equivalente Rust propuesto**: `send: Box<dyn Fn(SequencedMessage<A>)>` reemplazable en el estado; la variante con `send` propio no se porta en esta fila (depende de Cluster Sharding, fuera de alcance: Q-X-10) y queda como detalle interno sin contrato independiente. **Diferencias/límites**: el `RegisterConsumer` por referencia local reemplaza al de `ActorRef` remoto: sin remoting solo hay variante local.
- **Aserciones**:
  - A1. registrar un segundo `ConsumerController` tras enviar 1..3: el nuevo recibe `SequencedMessage(2).asFirst`, no recibe nada en 100 ms, lo recibe de nuevo (reenvío del primero), tras `Request(2,10)` recibe el 3, y luego el 4 enviado después — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ProducerControllerSpec.scala:149-185
  - A2. reemplazo del destino completo (el consumidor termina con 42 mensajes, se registra otro `ConsumerController2` que recibe otros 42 desde el mismo `ProducerController`) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ReliableDeliverySpec.scala:136-163
  - A3. `RegisterConsumer` antes que `Start` y `Start` antes que `RegisterConsumer` conducen al mismo estado activo — no afirmada por spec (ProducerControllerImpl.scala:276-293); caracterización propuesta: probar ambos órdenes
  - A4. variante con `send` propio (sin `RegisterConsumer`) — no afirmada por spec en actor-typed (la usa `ShardingProducerController`, cluster-sharding-typed); no portada (Q-X-10)
- **Validación prevista**: A1-A2 portados; A3 caracterización de ambos órdenes de llegada.
- **Decisiones**: Q-T-DLV-1; Q-X-10 (Cluster Sharding fuera de alcance, frontera del `send` interno)

## AT-DLV-001.5: ProducerController: MessageWithConfirmation (respuesta de confirmación con el seqNr) (api-may-change)

- **Contrato observable**: Enviando `MessageWithConfirmation(message, replyTo: ActorRef[SeqNr])` por `askNextTo` (típicamente con `context.ask`) el productor recibe en `replyTo` el `seqNr` del mensaje: **sin cola duradera**, cuando el consumidor lo ha entregado, procesado y confirmado (llega un `Ack` o un `Request` cuyo `confirmedSeqNr` lo cubre); **con cola duradera**, cuando el mensaje se ha almacenado (el consumidor puede no haberlo procesado). El `SequencedMessage` correspondiente lleva `ack = true` (solo el último chunk si el mensaje se trocea) para que el consumidor envíe `Ack` si no manda `Request`.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/ProducerController.scala:38-95 (Scaladoc), 131-140 (MessageWithConfirmation); actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/internal/ProducerControllerImpl.scala:536-546 (onAck: responde a `replyAfterStore` con `seqNr <= confirmado`), 563-587 (reply tras almacenamiento), 735-771 (chunk: `ack && lastChunk`), 774-792 (MessageWithConfirmation)
- **Precondiciones**: mensaje enviado por `askNextTo` con demanda (AT-DLV-001.1).
- **Comportamiento**: `replyAfterStore: Map[SeqNr, ActorRef[SeqNr]]` se actualiza con la clave del **último chunk** (`chunks.last.seqNr`). `onAck` responde a todas las entradas con `seqNr <= newConfirmedSeqNr` (un `Ack(4)` responde 3 y 4 aunque se perdiera el `Ack(3)`), también cuando `supportResend = false`. El orden de las respuestas dentro de un mismo `onAck` sigue el orden de iteración del `Map` de Scala (inserción para <= 4 entradas, hash para más): **no es un contrato** de Pekko; la propuesta Rust responde en `seqNr` ascendente (refinamiento dentro del espacio no especificado).
- **Errores**: ninguno propio. El `ask` del productor puede expirar por su propio timeout si el consumidor no confirma (lo decide el productor).
- **Orden / concurrencia / ciclo de vida**: la respuesta con cola duradera sale en `StoreMessageSentCompleted` antes de enviar al consumidor (574).
- **Dependencias**: AT-DLV-001.2; AT-DLV-001.6; AT-DLV-002.7; AT-CTX-012 (ask desde el actor)
- **Equivalente Rust propuesto**: `reply_after_store: BTreeMap<u64, ActorRef<u64>>`; el productor usa `context.ask` con `RequestNext.ask_next_to`. **Se preserva**: respuesta por `seqNr`, acumulativa, con reply-after-store con cola duradera y `ack` solo en el último chunk. **Diferencias/límites**: orden `BTreeMap` (estricto) frente al `Map` de Scala (ver Comportamiento).
- **Aserciones**:
  - A1. `MessageWithConfirmation` 1..5: `ack = true`; `Request(1,..)` responde 1; `Ack(2)` responde 2; con 3 y 4 en vuelo `Ack(4)` responde 3 y 4 en ese orden; `Request(5,15,..)` responde 5 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ProducerControllerSpec.scala:187-227
  - A2. lo mismo con `supportResend = false` ("reply to MessageWithConfirmation for lost messages") — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ProducerControllerSpec.scala:348-388
  - A3. con cola duradera la respuesta (1L, 2L) llega al almacenar, antes de que el consumidor confirme — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/DurableProducerControllerSpec.scala:146-173
  - A4. extremo a extremo con ask: los `replyTo` reciben exactamente los seqNr 1..42 (sin troceo) o el mismo número de respuestas (con troceo, donde los seqNr de respuesta son los del último chunk de cada mensaje) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ReliableDeliverySpec.scala:73-122
  - A5. orden de respuestas con más de 4 pendientes — no afirmada por spec (ProducerControllerImpl.scala:537-542, `Map.partition`); el orden no es contractual
- **Validación prevista**: A1-A4 portados 1:1; A5 es una nota de contrato (no se testea orden >4).
- **Decisiones**: Q-T-DLV-1; Q-T-DLV-2

## AT-DLV-001.6: ProducerController: integración con DurableProducerQueue (carga, almacenamiento, reintentos y reanudación) (api-may-change)

- **Contrato observable**: Con `durableQueueBehavior = Some(b)` el `ProducerController` crea la cola como hijo `"durable"` (mismo dispatcher que el padre, `DispatcherSelector.sameAsParent()`), la vigila (`watchWith`) y: (1) al arrancar pide `LoadState` (timeout `durable-queue.request-timeout`, 3 s) con hasta `retry-attempts` (10) intentos; (2) reanuda con el estado cargado: `currentSeqNr`, `confirmedSeqNr = highestConfirmedSeqNr`, `firstSeqNr = highestConfirmedSeqNr + 1`, y los `unconfirmed` como `SequencedMessage` (el primero con `first = true`, con su `ack` original); si hay sin confirmar **no** pide `RequestNext` al productor hasta confirmarse el primero; (3) cada mensaje se almacena con `StoreMessageSent(MessageSent(seqNr, msg, ack, NoQualifier, ahora), replyTo)` y solo tras `StoreMessageSentAck` se envía al consumidor (y se responde el `MessageWithConfirmation`); (4) cada confirmación nueva se almacena (`StoreMessageConfirmed`, escritura sin esperar, at-least-once).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/internal/ProducerControllerImpl.scala:208-235 (askLoadState), 241-263 (createState), 294-316 (LoadState: reintento y errores), 553-558 (StoreMessageConfirmed), 563-587 (StoreMessageSentCompleted), 589-644 (StoreMessageSentFailed), 847-854 (storeMessageSent); actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/DurableProducerQueue.scala:41-246; actor-typed/src/main/resources/reference.conf:81-95
- **Precondiciones**: un `Behavior[DurableProducerQueue.Command[A]]` que implemente el protocolo (AT-DLV-004); en `pekko-persistence-typed` existe `EventSourcedProducerQueue` (fuera de estos módulos).
- **Comportamiento**: `waitingForInitialization` recibe `LoadStateReply(state)` (o `initialState = Some(State.empty)` sin cola); reintento: `LoadStateFailed(attempt)` repite con `attempt + 1`. `StoreMessageSentCompleted` solo actúa si `seqNr == storeMessageSentInProgress`; si no (reintento tardío) se ignora con debug. Un mensaje almacenado dos veces (reintento tras timeout) lo maneja la cola (responde otra vez el ack; ver `TestDurableProducerQueue` 70-76 de su archivo).
- **Errores**: `TimeoutException("LoadState failed after [n] attempts, giving up.")` (303-305) y `TimeoutException("StoreMessageSentFailed seqNr [s] failed after [n] attempts, giving up.")` (591-595) tras agotar los reintentos (el controlador falla y lo decide la supervisión); `IllegalStateException("DurableQueue was unexpectedly terminated.")` si el hijo durable muere (316, 840); `IllegalStateException("currentSeqNr [..] not matching stored seqNr [..]")` y `("seqNr [..] not matching stored seqNr [..]")` como invariantes (565-569).
- **Orden / concurrencia / ciclo de vida**: el almacenamiento es secuencial (`storeMessageSentInProgress`); mientras hay un almacenamiento en curso no se pide `RequestNext` ni se envían chunks (502-503). La confirmación duradera es asíncrona y no bloquea la entrega.
- **Dependencias**: AT-DLV-004; AT-DLV-005.3; AT-DLV-001.1; AT-DLV-001.2; AT-DLV-001.5; AT-DLV-006
- **Equivalente Rust propuesto**: `Option<Behavior<DurableCommand<A>>>` + hijo `"durable"`; implementación en memoria opcional de prueba equivalente a `TestDurableProducerQueue` (Q-T-DLV-2 decide si hay implementación durable real o solo el protocolo). **Se preserva**: orden carga -> almacenar -> enviar -> confirmar, reanudación con `firstSeqNr`, reintentos y errores por agotamiento. **Diferencias/límites**: sin `pekko-persistence-typed` (fuera de alcance, Q-X-10) la cola real no existe; los reintentos usan `ask` con timeout (AT-CTX-012).
- **Aserciones**:
  - A1. carga de estado con 3 y 4 sin confirmar (currentSeqNr 5, highestConfirmed 2): no hay `RequestNext` al productor (`expectNoMessage`), 3 se envía como `first`, 4 tras `Request(3,13)`, luego `RequestNext` y msg-5 se envía con seqNr 5 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/DurableProducerControllerSpec.scala:54-89
  - A2. almacenamiento y confirmación: tras msg-1 el estado es `State(2, 0, Map.empty, Vector(MessageSent(1,..)))`; tras `Request(1,10)`: `State(2,1,Map(""->(1,ts)),Vector.empty)`; tras 2..4 con `Ack(3)`: `State(5,3,Map(""->(3,ts)),Vector(MessageSent(4,..,ack=true)))` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/DurableProducerControllerSpec.scala:91-144
  - A3. reanudación con `unconfirmed` vacío y `highestConfirmedSeqNr = 4`: `RequestNext.currentSeqNr == 5` y el primer mensaje sale con seqNr 5 como `first` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/DurableProducerControllerSpec.scala:293-323
  - A4. con una cola durable que falla aleatoriamente (10-20 %, con reinicio con backoff) la entrega termina completa — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ReliableDeliveryRandomSpec.scala:171-201
  - A5. agotamiento de reintentos de `LoadState`/`StoreMessageSent` (`TimeoutException`) y muerte de la cola (`IllegalStateException`) — no afirmada por spec (ProducerControllerImpl.scala:301-316, 589-596, 839-840); caracterización propuesta: cola que no responde y cola detenida con `retry-attempts` bajo
  - A6. `StoreMessageSentCompleted` tardío (seqNr distinto del en curso) se ignora — no afirmada por spec (ProducerControllerImpl.scala:579-586)
- **Validación prevista**: A1-A4 portados con un `TestDurableProducerQueue` equivalente (protocolo en memoria con fallos inyectables); A5-A6 caracterización con la misma cola de prueba. Oráculo: estado de la cola tras secuencias guionizadas.
- **Decisiones**: Q-T-DLV-2 (implementación durable); Q-T-DLV-1; Q-X-10 (persistence fuera de alcance)

## AT-DLV-002.1: ConsumerController: arranque con Start, mensajes previos a Start y parada con el consumidor (api-may-change)

- **Contrato observable**: El consumidor envía `ConsumerController.Start(deliverTo)` (adaptador que traduce `Delivery` a su protocolo; si el consumidor se reinicia debe enviar `Start` de nuevo). Los `SequencedMessage` recibidos **antes** de `Start` se guardan (capacidad = `flow-control-window`; si se llena se descartan con debug) y se procesan al recibir `Start` (con `stashBuffer.unstash(.., 1, ..)`). El `ConsumerController` vigila al consumidor (`watchWith`) y se detiene cuando éste termina. `Start` repetido con el mismo `deliverTo` no hace nada; con otro `deliverTo` se des-vigila el anterior, se vigila el nuevo y, si había un `Delivery` sin confirmar, se **reenvía al nuevo consumidor**. Con `ConsumerController(serviceKey)` el controlador se registra en el Receptionist al crearse (para `WorkPullingProducerController`).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/ConsumerController.scala:38-58 (Scaladoc), 60-95 (Start, Delivery), 109-123 (Confirmed); actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/internal/ConsumerControllerImpl.scala:124-202 (apply/waitForStart), 204-209 (MDC), 211-231 (initialState, enforceLocalConsumer), 635-637, 668-681 (receiveStart), 711-714 (receiveConsumerTerminated)
- **Precondiciones**: consumidor local (`enforceLocalConsumer`).
- **Comportamiento**: `waitForStart` acepta `RegisterToProducerController` (AT-DLV-002.6), `Start`, `SequencedMessage` (stash), `DeliverThenStop` (para inmediatamente si el stash está vacío; si no marca `stopping`), `Retry` (reenvía `RegisterConsumer` si se está registrando) y `ConsumerTerminated` (parada). El MDC `producerId` se pone por mensaje (`mdcForMessage`: solo para `SequencedMessage`) y el logger es `org.apache.pekko.actor.typed.delivery.ConsumerController`.
- **Errores**: `IllegalArgumentException("Consumer [<ref>] should be local.")` si `deliverTo` tiene ámbito global (228-231).
- **Orden / concurrencia / ciclo de vida**: el consumidor terminado detiene el controlador antes de entregar lo pendiente; un reinicio del consumidor con `Start` nuevo redelivera el mensaje no confirmado (a la nueva ref).
- **Dependencias**: AT-BEH-021 (withStash); AT-CTX-006 (watchWith); AT-RCP-003 (Register); AT-DLV-002.2; AT-DLV-002.6; AT-DLV-002.7
- **Equivalente Rust propuesto**: `ConsumerController<A>` con `enum ConsumerCommand<A> { Start(ActorRef<Delivery<A>>), SequencedMessage(..), Confirmed, RegisterToProducerController(..), DeliverThenStop, .. }`; `StashBuffer` con capacidad = ventana. **Se preserva**: stash previo a Start con descarte al llenarse, redelivery tras `Start` nuevo, parada al terminar el consumidor. **Diferencias/límites**: `watchWith` -> señal de terminación del runtime (AT-CTX-006); comprobación de localidad trivial sin remoting.
- **Aserciones**:
  - A1. un `SequencedMessage` recibido antes de `Start` se entrega tras `Start`, y entonces se envía `Request(0, 20, true, false)` al `ProducerController` (reenvío inicial de Request) y tras `Confirmed` `Request(1, 20, ..)` y luego `Request(1,20,true,viaTimeout=true)` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ConsumerControllerSpec.scala:90-109
  - A2. el stash previo a `Start` (44, 41 first, 45) se descarga en orden al llegar `Start` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ConsumerControllerSpec.scala:438-483
  - A3. reinicio del consumidor (nuevo `Start` con otro `deliverTo` antes de `Confirmed(3)`): el nuevo consumidor recibe `Delivery` seqNr 3 y luego 4 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ConsumerControllerSpec.scala:310-346
  - A4. el controlador se detiene cuando el consumidor se detiene, también antes del primer mensaje — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ConsumerControllerSpec.scala:348-378
  - A5. con `serviceKey`, el controlador se registra y el `WorkPullingProducerController` lo descubre — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/WorkPullingSpec.scala:60-91 (indirecto)
  - A6. stash lleno previo a `Start` descarta con debug; `Start` desde un consumidor no local lanza `IllegalArgumentException` — no afirmada por spec (ConsumerControllerImpl.scala:158-171, 228-231); caracterización propuesta: `flow-control-window + 1` mensajes antes de `Start`
- **Validación prevista**: A1-A4 portados; A5 con AT-DLV-003.1; A6 caracterización. Diferencial con jar oficial sobre secuencia de `Request` observada en un `ProducerController` falso (probe).
- **Decisiones**: Q-T-DLV-1

## AT-DLV-002.2: ConsumerController: entrega uno a uno, Confirmed y cola mientras se espera confirmación (api-may-change)

- **Contrato observable**: Cada mensaje se entrega al consumidor como `Delivery(message, confirmTo, producerId, seqNr)`; el siguiente no se entrega hasta que el consumidor responde `Confirmed` por `confirmTo` (que es el propio controlador). Los `SequencedMessage` que llegan mientras se espera se guardan (stash de capacidad `flow-control-window`) y se entregan en orden tras cada `Confirmed`. Duplicados (seqNr menor que el esperado, mismo `ProducerController`) se descartan; si el stash está lleno se descarta (el productor puede reenviar). Un `Confirmed` sin entrega pendiente es inesperado: se registra `warn("Received unexpected Confirmed from consumer.")` y se trata como no manejado.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/ConsumerController.scala:76-123; actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/internal/ConsumerControllerImpl.scala:498-533 (deliver), 553-654 (waitingForConfirmation: 556-605 Confirmed, 607-629 mensajes durante la espera, 631-633 Retry), 716-719 (unexpected Confirmed)
- **Precondiciones**: `Start` recibido; mensaje en el orden esperado (AT-DLV-002.4 cubre huecos).
- **Comportamiento**: en `waitingForConfirmation`, un `SequencedMessage` con `seqNr < seqNr_en_espera + stash.size + 1` y el mismo `ProducerController` es duplicado (debug, descartado); si el stash está lleno se descarta (debug); si no se guarda. Al `Confirmed`: ver AT-DLV-002.3 (Request) y AT-DLV-002.7 (Ack/parada); luego `unstash(active(..), 1, ..)` entrega el siguiente. `Retry` no hace nada mientras se espera confirmación (los reintentos se hacen desde `active`).
- **Errores**: `Confirmed` inesperado: warn + `Behaviors.unhandled` (publica `UnhandledMessage`, AT-EVT-003).
- **Orden / concurrencia / ciclo de vida**: orden estricto por `seqNr`; una entrega pendiente a la vez.
- **Dependencias**: AT-DLV-002.1; AT-DLV-002.3; AT-DLV-002.7; AT-BEH-021 (stash)
- **Equivalente Rust propuesto**: `StashBuffer` acotado + estado `Waiting{seq_msg}`; `Delivery<A> { message, confirm_to: ActorRef<Confirmed>, producer_id, seq_nr }`. **Se preserva**: una entrega a la vez, orden, descarte de duplicados y por stash lleno. **Diferencias/límites**: ninguno observable.
- **Aserciones**:
  - A1. durante la espera de `Confirmed` de 2, llegan 3 y 4 y no se entregan (`consumerProbe.expectNoMessage()`); tras `Confirmed` se entregan 3 y 4 en orden; luego 5,6,7 más los reenviados 5,6,7 y 8: se entregan 5,6,7,8 sin duplicados y sin entrega extra al final — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ConsumerControllerSpec.scala:219-270
  - A2. un duplicado del primero ya confirmado (mismo `ProducerController`) no se entrega de nuevo; con otro `ProducerController` sí se acepta como nuevo flujo — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ConsumerControllerSpec.scala:380-408
  - A3. flujo completo de entrega y confirmación contra un productor real con consumidor lento y productor rápido — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ReliableDeliverySpec.scala:124-134
  - A4. `Confirmed` inesperado -> warn y `unhandled` — no afirmada por spec (ConsumerControllerImpl.scala:716-719); caracterización propuesta: enviar `Confirmed` antes de cualquier entrega y observar `UnhandledMessage` y el warn
  - A5. descarte por stash lleno mientras se espera — no afirmada por spec (ConsumerControllerImpl.scala:613-619); caracterización propuesta: `flow-control-window + 1` mensajes durante una espera
  - A6. (implícita) el consumidor de prueba lanza `RuntimeException("Received duplicate [(producerId,seqNr)]")` ante un par repetido: ningún test de extremo a extremo (incluidos los de pérdida aleatoria) entrega duplicados al consumidor — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/TestConsumer.scala:115-120
- **Validación prevista**: A1-A3 portados; A4-A5 caracterización. Diferencial con jar oficial sobre la secuencia de `Delivery`.
- **Decisiones**: Q-T-DLV-1

## AT-DLV-002.3: ConsumerController: ventana de flujo, Request y reintentos con backoff (api-may-change)

- **Contrato observable**: El consumidor envía `Request(confirmedSeqNr, requestUpToSeqNr, supportResend, viaTimeout)` al `ProducerController`: (1) al recibir el primer mensaje de un productor: `Request(0, seqNr - 1 + flowControlWindow, ..)`; (2) al confirmarse el primer mensaje: `Request(seqNr, seqNr - 1 + flowControlWindow, ..)` (cancela el reenvío del primero); (3) cada vez que se ha consumido la mitad de la ventana (`requestedSeqNr - seqNr == flowControlWindow / 2` al `Confirmed`): `Request(seqNr, requestedSeqNr + flowControlWindow / 2, ..)`; (4) por timer, si no llegan mensajes: `Request(confirmedSeqNr, requestedSeqNr, supportResend, viaTimeout = true)` (si el productor aún no se conoce, no se envía). El timer de reintento empieza en `resend-interval-min` (2 s por defecto) y se multiplica por 1,5 en cada tick hasta `resend-interval-max` (30 s); al recibir un mensaje o enviar una `Request` por demanda vuelve al mínimo. `supportResend = !only-flow-control`.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/internal/ConsumerControllerImpl.scala:233-267 (RetryTimer), 302-346 (active: 308 reset), 347-348 (Retry), 373-391 (primer mensaje), 556-592 (Confirmed -> Request), 656-666 (receiveRetry), 721-735 (retryRequest); actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/ConsumerController.scala:201-290 (Settings); actor-typed/src/main/resources/reference.conf:97-114
- **Precondiciones**: `Start` recibido y `ProducerController` conocido (primer mensaje).
- **Comportamiento**: `RetryTimer.scheduleNext`: `min(maxBackoff, interval * 1.5)` y reinicia el timer si cambió (247-260); `reset` vuelve al mínimo si había crecido (262-265); `start` reinicia el intervalo mínimo (242-245). En `retryRequest`, con `only-flow-control` el `requestedSeqNr` enviado es `receivedSeqNr + flowControlWindow / 2` (726). Los chunks tienen su propio disparo de `Request` (AT-DLV-005.2).
- **Errores**: ninguno.
- **Orden / concurrencia / ciclo de vida**: `Retry` solo actúa en `active` y `resending`; en `waitingForConfirmation` se ignora. `Request` va siempre al `ProducerController` del último mensaje recibido.
- **Dependencias**: AT-DLV-002.2; AT-DLV-001.2; AT-DLV-006 (flow-control-window, resend-interval-min/max); AT-TMR
- **Equivalente Rust propuesto**: `RetryTimer { min, max, interval }` con `schedule_next`/`reset`/`start` y temporizador del runtime (`ManualTime` en tests); `flow_control_window: usize`. **Se preserva**: las cuatro situaciones de `Request`, el factor 1,5, el tope y los reinicios. **Diferencias/límites**: el factor `1.5` aplicado a `FiniteDuration` con redondeo a nanos (`_interval * 1.5`) debe reproducirse con `Duration::mul_f64`.
- **Aserciones**:
  - A1. primer mensaje, `Confirmed` y reintento: `Request(0,20,true,false)`, luego `Request(1,20,true,false)`, luego `Request(1,20,true,true)` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ConsumerControllerSpec.scala:90-109
  - A2. `Request` a media ventana: con ventana 20, tras `Confirmed` del 10 se envía `Request(10, 30, true, false)` y no antes — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ConsumerControllerSpec.scala:111-145
  - A3. reintento por timer tras confirmar 2: `Request(2,20,true,true)`; tras 3: `Request(3,20,true,true)`; el siguiente no llega antes de 1,1 s (backoff exponencial) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ConsumerControllerSpec.scala:185-217
  - A4. ventana tras el primero de un productor nuevo: `sequencedMessage(23, producer2).asFirst` produce `Request(0, 23 + 20 - 1)`, y con otro productor y seqNr 7 produce `Request(0, 7 + 20 - 1)` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ConsumerControllerSpec.scala:410-436
  - A5. tope `resend-interval-max` y reinicio al mínimo tras recibir un mensaje — no afirmada por spec (ConsumerControllerImpl.scala:247-265); caracterización propuesta: tiempo controlado con min=1 s, max=2 s y comprobar la secuencia 1 s, 1,5 s, 2 s, 2 s
  - A6. `Request` por timer no se envía antes de conocer al productor — no afirmada por spec (ConsumerControllerImpl.scala:723-724)
- **Validación prevista**: A1-A4 portados con tiempo controlado (el original usa `expectNoMessage(1.1.second)` en tiempo real); A5-A6 caracterización con `ManualTime`.
- **Decisiones**: Q-T-DLV-1; Q-T-DLV-4

## AT-DLV-002.4: ConsumerController: detección de pérdidas, Resend y estado de reenvío (api-may-change)

- **Contrato observable**: Si llega un mensaje con `seqNr > esperado` y el reenvío está activo (`only-flow-control = false`), el consumidor envía `Resend(fromSeqNr = esperado)` al `ProducerController`, **vacía su stash**, reinicia el timer de reintento y pasa a **estado de reenvío** donde descarta todo salvo el mensaje esperado (los demás estaban en vuelo y se reenviarán). Si el `Resend` se pierde, por cada `Retry` se reenvía `Resend(receivedSeqNr + 1)`. Un mensaje con `seqNr < esperado` es duplicado y se descarta (si es `first`, se reenvía una `Request` por si se perdió la anterior). Un productor nuevo cuyo primer mensaje visto no es `first` (típico de Sharding) provoca `Resend(0)` y estado de reenvío.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/internal/ConsumerControllerImpl.scala:302-346 (active: 323-337 hueco, 338-345 duplicado), 373-413 (receiveChangedProducer: 392-402 Resend(0), 403-411 productor inesperado), 431-496 (resending), 465-473 (reintento de Resend)
- **Precondiciones**: `Start` recibido.
- **Comportamiento**: en `resending`, el mensaje esperado se entrega (debug "Received missing SequencedMessage seqNr"), un `first` descartado dispara `retryRequest`; el estado se abandona al entregarse el esperado (vuelve a `active` vía `deliver`). Un mensaje de un `ProducerController` distinto cuando ya se tiene uno y no es `first` se descarta con `warn` ("discarding message because it was from unexpected producer").
- **Errores**: `IllegalStateException("StashBuffer should be cleared before resending.")` como invariante (436).
- **Orden / concurrencia / ciclo de vida**: tras `Resend` todo mensaje distinto del esperado se ignora hasta recibirlo; los mensajes ya entregados no se vuelven a entregar.
- **Dependencias**: AT-DLV-001.3; AT-DLV-002.2; AT-DLV-002.6; AT-DLV-002.5
- **Equivalente Rust propuesto**: estados `Active`/`Resending`/`Waiting` como enum de estado interno del actor; `received_seq_nr`, `confirmed_seq_nr`. **Se preserva**: el hueco dispara `Resend(esperado)` una vez y el descarte del resto; deduplicación por seqNr. **Diferencias/límites**: ninguno observable.
- **Aserciones**:
  - A1. entregados 1 y 2, llega 5: se envía `Resend(3)`; llegan 3,4,5 y se entregan en ese orden — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ConsumerControllerSpec.scala:147-183
  - A2. un productor nuevo cuyo primer mensaje (44) no es `first`: `Resend(0)`, el stash se vacía; al llegar 41 `first` se hace `Request(0, 60)` y se entregan 41..45 y 46 en orden — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ConsumerControllerSpec.scala:438-483
  - A3. reintento de `Resend(receivedSeqNr + 1)` por timer cuando se perdió el `Resend` — no afirmada por spec (ConsumerControllerImpl.scala:465-473); caracterización propuesta: no responder al `Resend` y observar la repetición tras el intervalo
  - A4. descarte con `warn` de un mensaje no `first` de un productor inesperado — no afirmada por spec (ConsumerControllerImpl.scala:403-410)
  - A5. con red que pierde mensajes en ambos sentidos el flujo converge sin duplicados ni pérdidas — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ReliableDeliveryRandomSpec.scala:155-169, 186-201
- **Validación prevista**: A1-A2 portados; A3-A4 caracterización; A5 aleatorio con semilla fija. Diferencial con jar oficial: secuencia de `Resend`/`Request` ante huecos guionizados.
- **Decisiones**: Q-T-DLV-1; Q-T-DLV-4

## AT-DLV-002.5: ConsumerController: modo only-flow-control (sin reenvío de perdidos) (api-may-change)

- **Contrato observable**: Con `pekko.reliable-delivery.consumer-controller.only-flow-control = true` (o `Settings.withOnlyFlowControl(true)`) el consumidor mantiene el control de flujo pero **no pide reenvío**: sus `Request` llevan `supportResend = false`; un hueco en los seqNr se acepta y se entrega el mensaje recibido (sin `Resend`, `receivedSeqNr = seqNr`); un primer mensaje de productor nuevo se acepta aunque no sea `first`; el `requestedSeqNr` de las `Request` por timer es `receivedSeqNr + flowControlWindow / 2`. El `ProducerController` en ese modo no guarda sin confirmar (AT-DLV-001.3).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/internal/ConsumerControllerImpl.scala:295 (`resendLost`), 323-337 (hueco sin reenvío), 376 (`seqMsg.first || !resendLost`), 726 (retryRequest); actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/ConsumerController.scala:276-277 (withOnlyFlowControl); actor-typed/src/main/resources/reference.conf:108-113
- **Precondiciones**: configuración `only-flow-control`.
- **Comportamiento**: ver contrato; pensado para productor y consumidor en el mismo sistema local (más eficiente: no se retiene memoria en el productor) con la desventaja de que los mensajes perdidos no se entregan.
- **Errores**: ninguno.
- **Orden / concurrencia / ciclo de vida**: igual que AT-DLV-002.2.
- **Dependencias**: AT-DLV-002.3; AT-DLV-001.3; AT-DLV-006
- **Equivalente Rust propuesto**: `ConsumerSettings { only_flow_control: bool }` y la rama `resend_lost` en el estado. **Se preserva**: `supportResend=false` en `Request`, aceptación de huecos. **Diferencias/límites**: en un runtime local sin pérdidas el modo equivale a no perder nunca: se mantiene por paridad de configuración.
- **Aserciones**:
  - A1. con `onlyFlowControl`: `Request(0,20,supportResend=false,..)` y `Request(1,20,supportResend=false,..)`; saltar el 2 entrega el 3 y el 4; saltar hasta 35 entrega el 35 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ConsumerControllerSpec.scala:671-703
  - A2. el productor no reenvía con `supportResend=false` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ProducerControllerSpec.scala:310-346
  - A3. con red que pierde mensajes sin reenvío, el consumidor sigue recibiendo mensajes (sin garantía de totalidad) hasta el fin de prueba — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ReliableDeliveryRandomSpec.scala:203-216
- **Validación prevista**: A1-A2 portados; A3 aleatorio con semilla fija.
- **Decisiones**: Q-T-DLV-1; Q-T-DLV-4

## AT-DLV-002.6: ConsumerController: registro y cambio de ProducerController, deduplicación del primer mensaje (api-may-change)

- **Contrato observable**: `RegisterToProducerController(pc)` hace que el consumidor envíe `ProducerController.RegisterConsumer(self)` a `pc` y **reintente cada `Retry`** (resend-interval-min) hasta recibir un mensaje de ese `ProducerController`; mientras `registering` esté definido, los mensajes de otros productores se descartan (debug "discarding message because registering to new ProducerController"). Si `pc` es el actual no hace nada. Un mensaje cuyo `ProducerController` difiere del actual (o `receivedSeqNr == 0`) se trata como productor nuevo: si es `first` (o en modo only-flow-control) se acepta con `Request(0, seqNr - 1 + window)`; el primer mensaje duplicado del mismo productor se deduplica (no se entrega de nuevo) pero provoca una `Request` de reintento.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/ConsumerController.scala:125-136 (RegisterToProducerController); actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/internal/ConsumerControllerImpl.scala:94-122 (State: isProducerChanged, updatedRegistering), 139-145, 180-185 (Retry en waitForStart), 310-322 (active), 373-428 (receiveChangedProducer, logChangedProducer), 656-666 (receiveRetry), 683-698 (receiveRegisterToProducerController)
- **Precondiciones**: ninguna (también válido antes de `Start`).
- **Comportamiento**: `isProducerChanged = seqMsg.producerController != producerController || receivedSeqNr == 0`; `updatedRegistering` borra `registering` cuando llega un mensaje del `ProducerController` registrado; el cambio de productor se registra con debug (y `consumerChangedProducer` en el flight recorder cuando ya había uno).
- **Errores**: ninguno.
- **Orden / concurrencia / ciclo de vida**: el registro se reintenta con el timer común (`Retry`) hasta el primer mensaje del nuevo productor.
- **Dependencias**: AT-DLV-001.4; AT-DLV-002.3; AT-DLV-002.4; AT-DLV-002.1
- **Equivalente Rust propuesto**: campo `registering: Option<ActorRef<ProducerCommand<A>>>` en el estado. **Se preserva**: reintento de registro, descarte de otros productores durante el registro, identificación del productor por su ref, deduplicación del primero. **Diferencias/límites**: la identidad del `ProducerController` es la de `ActorRef`; igualdad de refs (AT-REF-005).
- **Aserciones**:
  - A1. `RegisterToProducerController` envía `RegisterConsumer` y lo reenvía tras el intervalo (1 s en el spec) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ConsumerControllerSpec.scala:53-65
  - A2. al cambiar a un segundo `ProducerController`, el segundo recibe `RegisterConsumer` y su reintento — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ConsumerControllerSpec.scala:67-88
  - A3. el mismo primer mensaje repetido por el mismo `ProducerController` no se entrega dos veces; con un `ProducerController` distinto sí — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ConsumerControllerSpec.scala:380-408
  - A4. punto a punto con `RegisterToProducerController` enviado por el consumidor (flujo completo) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ReliableDeliverySpec.scala:51-71
  - A5. mensajes de un tercer productor durante el registro se descartan — no afirmada por spec (ConsumerControllerImpl.scala:314-318, 446-450); caracterización propuesta: registrar a pc2 y enviar desde pc3
- **Validación prevista**: A1-A4 portados con tiempo controlado; A5 caracterización.
- **Decisiones**: Q-T-DLV-1; Q-T-DLV-4

## AT-DLV-002.7: ConsumerController: Ack opcional, Ack al detenerse y DeliverThenStop (api-may-change)

- **Contrato observable**: Si el `SequencedMessage` lleva `ack = true` y al `Confirmed` no se envía una `Request`, el consumidor envía `Ack(confirmedSeqNr)` (una `Request` también confirma). Al detenerse el controlador envía un `Ack(confirmedSeqNr)` de mejor esfuerzo en `PostStop`. `DeliverThenStop()` pide entregar lo pendiente y parar: si no hay stash y `receivedSeqNr == confirmedSeqNr` para inmediatamente; si no, marca `stopping`, sigue entregando los mensajes ya guardados (y los que lleguen mientras tanto se entregan en orden) y al confirmarse el último con el stash vacío se detiene con un callback que envía `Ack(seqNr)`.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/ConsumerController.scala:136-137 (DeliverThenStop); actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/internal/ConsumerControllerImpl.scala:173-178 (antes de Start), 362-363, 585-592 (Ack opcional), 594-599 (parada tras la última confirmación), 645-646, 700-709 (receiveDeliverThenStop), 737-741 (postStop)
- **Precondiciones**: `ack = true` lo decide el productor (AT-DLV-001.5).
- **Comportamiento**: ver contrato. El `Ack` por `postStop` se envía desde `active`, `resending` y `waitingForConfirmation`.
- **Errores**: ninguno.
- **Orden / concurrencia / ciclo de vida**: tras `DeliverThenStop` los mensajes nuevos que llegan se siguen aceptando hasta vaciar; el `Ack` final de la parada se envía **después** del `PostStop` (ver A3).
- **Dependencias**: AT-DLV-001.5; AT-DLV-002.2; AT-DLV-002.1
- **Equivalente Rust propuesto**: `DeliverThenStop` como comando; `on_stop` envía `Ack`. **Se preserva**: condición de parada inmediata vs diferida, `Ack` de mejor esfuerzo. **Diferencias/límites**: ninguno observable.
- **Aserciones**:
  - A1. `Ack` solo para `ack = true`: `Ack(2)` y `Ack(3)`; el mensaje 4 (`ack = false`) no genera; `Ack(5)` tras confirmar 5 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ConsumerControllerSpec.scala:272-308
  - A2. al detener el controlador con 3 pendiente se envía `Ack(2L)` (último confirmado) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ConsumerControllerSpec.scala:485-513
  - A3. parada ordenada: `DeliverThenStop` con 2 en espera de confirmación y 3,4 guardados; llegado el 5, se entregan 3,4,5 y el controlador termina; llegan `Ack(4L)` (por `postStop`) y luego `Ack(5L)` (callback de `stopped`) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ConsumerControllerSpec.scala:515-558
  - A4. `DeliverThenStop` antes de `Start` con stash vacío para inmediatamente — no afirmada por spec (ConsumerControllerImpl.scala:173-178); caracterización propuesta
- **Validación prevista**: A1-A3 portados; A4 caracterización.
- **Decisiones**: Q-T-DLV-1

## AT-DLV-003.1: WorkPullingProducerController: workers registrados vía Receptionist y GetWorkerStats (api-may-change)

- **Contrato observable**: El `WorkPullingProducerController` se suscribe al Receptionist con la `ServiceKey[ConsumerController.Command[A]]` de los workers (cada worker registra su `ConsumerController` con `ConsumerController(serviceKey)`). Por cada worker nuevo del `Listing` crea un hijo `ProducerController` (nombre = UUID, `producerId` = `"<producerId>-<uuid>"`, sin cola duradera, mismos `producerControllerSettings`, mismo dispatcher del padre), le envía `Start(adaptador de RequestNext)` y `RegisterConsumer(worker)`; por cada worker que desaparece del `Listing` detiene su hijo `ProducerController`, borra su estado y reasigna sus no confirmados (AT-DLV-003.4). `GetWorkerStats(replyTo)` responde `WorkerStats(numberOfWorkers)` con el número de workers del último `Listing` (no el de los que tienen demanda).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/WorkPullingProducerController.scala:36-108 (Scaladoc), 110-150 (Start, RequestNext, GetWorkerStats, WorkerStats); actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/internal/WorkPullingProducerControllerImpl.scala:80-87 (OutState), 125-153 (apply: suscripción 135-137), 299-300 (adaptador de RequestNext), 568-614 (receiveCurrentWorkers), 655-657 (GetWorkerStats)
- **Precondiciones**: Receptionist local disponible (AT-RCP-001); workers locales o cluster (cluster fuera de alcance).
- **Comportamiento**: `addedWorkers = nuevo.diff(actual)` y `removedWorkers = actual.diff(nuevo)`; la desregistración de un worker desconocido solo se loguea (debug). Los workers inalcanzables no se evitan (TODO #28722 en el código). `OutState.confirmationQualifier = producerController.path.name` (UUID).
- **Errores**: ninguno propio.
- **Orden / concurrencia / ciclo de vida**: las altas y bajas se procesan en el orden del `Listing`; `RegisterConsumerDone` es un mensaje residual que se ignora (659-660).
- **Dependencias**: AT-RCP-003; AT-RCP-005.2; AT-DLV-001.4; AT-DLV-002.1; AT-CTX-002
- **Equivalente Rust propuesto**: `Receptionist::subscribe(worker_key)` + mapa `out_key -> OutState`; hijos `ProducerController`. **Se preserva**: un hijo por worker, claves `"<id>-<uuid>"`, `WorkerStats` por `Listing`. **Diferencias/límites**: `UUID.randomUUID()` -> `uuid::Uuid::new_v4()` (formato textual no contractual).
- **Aserciones**:
  - A1. `GetWorkerStats` con 0, 1, 2, 1, 0 workers durante el alta/baja de probes registrados en el Receptionist — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/WorkPullingSpec.scala:46-54 (helper `awaitWorkersRegistered`) y 93-145
  - A2. dos workers reales con `ConsumerController(workerServiceKey)` reciben trabajo del mismo productor y ambos completan sus 42 mensajes; al detener los controllers `WorkerStats == 0` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/WorkPullingSpec.scala:60-91
  - A3. nombre de los hijos y `producerId` por worker (`"<producerId>-<uuid>"`) — no afirmada por spec (WorkPullingProducerControllerImpl.scala:573-584); los tests de durable ignoran el calificador (`assertState` borra `confirmationQualifier`, DurableWorkPullingSpec.scala:61-73)
  - A4. baja de un worker desconocido solo se loguea — no afirmada por spec (WorkPullingProducerControllerImpl.scala:607-609)
- **Validación prevista**: A1-A2 portados (dependen de AT-RCP-005.2 y del Receptionist local); A3-A4 caracterización. Diferencial con jar oficial: `GetWorkerStats` ante altas/bajas guionizadas.
- **Decisiones**: Q-T-DLV-1; Q-T-RCP-1 (igualdad de ServiceKey)

## AT-DLV-003.2: WorkPullingProducerController: demanda, selección aleatoria de worker y buffer (api-may-change)

- **Contrato observable**: El productor recibe `RequestNext(sendNextTo, askNextTo)` (sin seqNr, a diferencia del punto a punto) cuando **algún** worker tiene demanda. Cada mensaje se asigna a un worker con demanda elegido **al azar** (`ThreadLocalRandom`) y se envía a su `ProducerController` como `MessageWithConfirmation` (siempre `ack = true`, para conocer la confirmación del worker); el worker queda sin demanda hasta su siguiente `RequestNext`. Si cuando llega el mensaje ya no hay worker con demanda (se dieron de baja tras el `RequestNext`), el mensaje se guarda en un stash (capacidad `buffer-size`, 1000) y se entrega al llegar nueva demanda o un worker nuevo. El orden de los mensajes **no** es contractual (distintos workers los procesan por separado).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/WorkPullingProducerController.scala:35-108,126-132; actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/internal/WorkPullingProducerControllerImpl.scala:95-107 (State), 236-239 (checkStashFull), 304-396 (onMessage y tabla de decisión 360-389), 398-412 (selectWorker), 529-566 (receiveWorkerRequestNext)
- **Precondiciones**: `Start` recibido (AT-DLV-003.5) y al menos un worker.
- **Comportamiento**: tabla de decisión para el siguiente `RequestNext` según (`requested`, `wasStashed`, `hasMoreDemand` = >= 2 workers con demanda, `stash.isEmpty`) (362-389): con más demanda y el mensaje pedido -> `RequestNext` inmediato; con un solo worker con demanda -> espera a que el worker pida; combinaciones inválidas -> `IllegalStateException`. `receiveWorkerRequestNext`: actualiza `OutState(seqNr, unconfirmed, askNextTo)`, procesa la confirmación incluida (`confirmedSeqNr`), y si el stash no está vacío desencola todo (`unstashAll`), si no y no hay demanda pendiente envía `RequestNext` al productor.
- **Errores**: `IllegalStateException("Unexpected message [<msg>], wasn't requested nor unstashed.")` (313-314) si llega un mensaje sin demanda ni stash (sin cola duradera); `IllegalArgumentException("Buffer is full, size [<n>].")` (237-238) al llenarse el stash (si el productor respeta `RequestNext` solo puede ocurrir tras bajas de workers); `IllegalStateException("Invalid combination of hasRequested [..], wasStashed [..], hasMoreDemand [..], stashBuffer.isEmpty [..]")` (387-388).
- **Orden / concurrencia / ciclo de vida**: sin garantía de orden entre workers; dentro de un mismo worker se conserva el orden de su `ProducerController`.
- **Dependencias**: AT-DLV-003.1; AT-DLV-003.3; AT-DLV-001.1; AT-BEH-021 (stash)
- **Equivalente Rust propuesto**: `rand::thread_rng()` para la selección; `StashBuffer` acotado a `buffer_size`; los errores internos como `debug_assert!`/pánico supervisado. **Se preserva**: un `RequestNext` por mensaje, selección uniforme entre workers con demanda, stash con límite `buffer-size`, `RequestNext` solo con demanda. **Diferencias/límites**: la aleatoriedad impide asserts de reparto; los tests fijan la semilla o comprueban solo totales.
- **Aserciones**:
  - A1. dos workers reciben trabajo y terminan; el productor es el de prueba que envía cuando recibe `RequestNext` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/WorkPullingSpec.scala:60-91
  - A2. cuando un worker tiene demanda de 1 y se envían mensajes, el siguiente `RequestNext` solo llega con nueva demanda: tras 5 mensajes enviados sin `Ack` no hay más demanda (`producerProbe.expectNoMessage()`) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/DurableWorkPullingSpec.scala:205-212
  - A3. con un segundo worker registrado, el productor recibe `RequestNext` y el mensaje va al segundo (seqNr 1 para su par `ProducerController`/`ConsumerController`) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/DurableWorkPullingSpec.scala:227-250
  - A4. mensaje sin demanda, stash lleno, combinación inválida — no afirmada por spec (WorkPullingProducerControllerImpl.scala:313-314, 237-238, 387-388); caracterización propuesta: enviar sin `RequestNext` previo
  - A5. reparto aleatorio uniforme — no afirmada por spec (WorkPullingProducerControllerImpl.scala:409); caracterización propuesta: N mensajes entre 2 workers con demanda ilimitada y verificar que ambos reciben alguno con semilla fija
- **Validación prevista**: A1-A3 portados; A4-A5 caracterización. Diferencial con jar oficial: totales entregados por worker (no el reparto).
- **Decisiones**: Q-T-DLV-1; Q-T-DLV-4

## AT-DLV-003.3: WorkPullingProducerController: MessageWithConfirmation con Done, Ack del worker y AskTimeout (api-may-change)

- **Contrato observable**: `WorkPullingProducerController.MessageWithConfirmation(message, replyTo: ActorRef[Done])` (nótese: `Done`, no `SeqNr` como en el punto a punto) responde `Done` cuando el worker ha procesado y confirmado el mensaje; con cola duradera responde cuando está almacenado (AT-DLV-003.6). Internamente el controlador envía cada mensaje al `ProducerController` del worker con `ask` (`internal-ask-timeout`, 60 s) y recibe `Ack(outKey, seqNr)`; si el `ask` expira solo se registra debug ("Message seqNr [..] sent to worker [..] timed out. It will be be redelivered."): el mensaje sigue sin confirmar y se reasigna si el worker desaparece. Las confirmaciones llegan también dentro de los `RequestNext` del worker (`confirmedSeqNr`).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/WorkPullingProducerController.scala:134-142 (MessageWithConfirmation); actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/internal/WorkPullingProducerControllerImpl.scala:344-352 (ask al worker), 492-527 (receiveAck, onAck), 625-635 (Msg y MessageWithConfirmation), 665-670 (AskTimeout)
- **Precondiciones**: mensaje enviado por `askNextTo` con demanda.
- **Comportamiento**: `onAck` responde `Done` a cada `Unconfirmed` con `replyTo` y `outSeqNr <= confirmedSeqNr`; con cola duradera además `StoreMessageConfirmed(confirmed.last.totalSeqNr, qualifier, ahora)`. Un `Ack` de un worker ya dado de baja se ignora (`unhandled`). El `ack` se activa siempre en el `ProducerController` del worker (todos los mensajes van por `askNextTo` del worker).
- **Errores**: `AskTimeout` no es error del productor; sin respuesta indefinida si el worker nunca confirma y no se da de baja.
- **Orden / concurrencia / ciclo de vida**: las respuestas `Done` de un worker salen en orden de seqNr del worker.
- **Dependencias**: AT-DLV-001.5; AT-DLV-003.2; AT-DLV-003.4; AT-CTX-012 (ask)
- **Equivalente Rust propuesto**: `ask` con timeout `internal_ask_timeout`; `Unconfirmed { total_seq_nr, out_seq_nr, msg, reply_to: Option<ActorRef<Done>> }`. **Se preserva**: `Done` tras confirmación, sin respuesta al expirar el ask. **Diferencias/límites**: `Done` es el tipo unitario `Done` (Q-X-9: dominio de `Done`).
- **Aserciones**:
  - A1. `MessageWithConfirmation`: el `SequencedMessage` lleva `ack = true`; `replyProbe` recibe `Done` tras `Request(1,10)`; para msg-2 **no** hay respuesta hasta `Ack(2)` (`replyProbe.expectNoMessage()`); con msg-3 y msg-4 `Ack(4)` responde dos `Done` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/WorkPullingSpec.scala:147-187
  - A2. expiración del ask del worker (`AskTimeout`) sin efecto salvo debug — no afirmada por spec (WorkPullingProducerControllerImpl.scala:665-670); caracterización propuesta: `internal-ask-timeout` corto y worker que no responde
  - A3. `Ack` de worker ya dado de baja se ignora — no afirmada por spec (WorkPullingProducerControllerImpl.scala:497-500)
- **Validación prevista**: A1 portado; A2-A3 caracterización.
- **Decisiones**: Q-T-DLV-1; Q-X-9 (Done)

## AT-DLV-003.4: WorkPullingProducerController: reasignación de no confirmados cuando un worker desaparece (api-may-change)

- **Contrato observable**: Si un worker se detiene o se da de baja, los mensajes que le fueron enviados y no confirmó se **reenvían a otro worker** (puede procesarse alguno más de una vez): se re-encolan al propio controlador como `Msg(msg, wasStashed = true, replyTo)` (sin cola duradera) o `ResendDurableMsg(msg, oldQualifier, oldSeqNr)` (con cola duradera), por lo que el orden entre ellos no importa. Los `MessageWithConfirmation` pendientes **no** se responden al morir el worker; se responden cuando otro worker confirma el reenvío.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/internal/WorkPullingProducerControllerImpl.scala:586-611 (removedWorkers), 598-604 (reenvío a sí mismo)
- **Precondiciones**: otro worker registrado o la llegada posterior de uno (el mensaje queda en stash si no hay).
- **Comportamiento**: el nuevo worker recibe el mensaje con seqNr propio (empezando en 1 para su par `ProducerController`/`ConsumerController`), no el seqNr del anterior; el `context.stop(outState.producerController)` ocurre antes del reenvío.
- **Errores**: ninguno.
- **Orden / concurrencia / ciclo de vida**: sin garantía de orden de entrega entre los reasignados.
- **Dependencias**: AT-DLV-003.1; AT-DLV-003.2; AT-DLV-003.3; AT-DLV-003.6
- **Equivalente Rust propuesto**: reenviar `Msg` al propio buzón con `was_stashed = true`. **Se preserva**: reasignación de lo no confirmado y respuesta tardía de `Done`. **Diferencias/límites**: puede haber entrega duplicada a distintos workers (at-least-once); documentado.
- **Aserciones**:
  - A1. msg-2 y msg-3 (no confirmados) enviados a worker1, se reenvían a worker2 (seqNr 1 para worker2) y luego msg-4 va a worker2 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/WorkPullingSpec.scala:93-145
  - A2. con `MessageWithConfirmation`, al morir worker1 no se responde (`replyProbe.expectNoMessage()`), msg-3 y msg-4 se reenvían a worker2 y se responden tras su confirmación (`Request(1,10)` y `Ack(2)`) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/WorkPullingSpec.scala:189-242
  - A3. con cola duradera, el traspaso guarda los reasignados con seqNr nuevo y confirma los antiguos — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/DurableWorkPullingSpec.scala:284-403 (ver AT-DLV-003.6)
- **Validación prevista**: A1-A3 portados. Diferencial con jar oficial: conjunto de mensajes finalmente entregados.
- **Decisiones**: Q-T-DLV-1

## AT-DLV-003.5: WorkPullingProducerController: Start, reinicio del productor y estado inicial (api-may-change)

- **Contrato observable**: Hasta recibir `Start(producer)` el controlador guarda en un stash (`buffer-size`) cualquier otro mensaje y lo reprocesa al activarse (`unstashAll`). `Start` exige productor local. Con cola duradera espera además `LoadStateReply`. Al activarse reenvía a sí mismo los `unconfirmed` cargados (`ResendDurableMsg`). Un `Start` posterior (productor reiniciado) reemplaza al productor y, si hay `RequestNext` pendiente, se lo reenvía.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/internal/WorkPullingProducerControllerImpl.scala:159-234 (waitingForStart), 168-182 (becomeActive), 616-622 (receiveStart)
- **Precondiciones**: ninguna.
- **Comportamiento**: `becomeActive`: `Msg` adaptador con `wasStashed = false`, `RequestNext(msgAdapter, self)`, `active(createInitialState(currentSeqNr, producer))` y `stashBuffer.unstashAll(...)`.
- **Errores**: `IllegalArgumentException("Consumer [<ref>] should be local.")` (productor no local); el stash previo a `Start` lanza `IllegalArgumentException("Buffer is full, size [n].")` si se llena (236-239).
- **Orden / concurrencia / ciclo de vida**: el estado previo a `Start` solo acumula; no hay `RequestNext` antes de tener `Start` y algún worker con demanda.
- **Dependencias**: AT-DLV-003.2; AT-DLV-003.6; AT-DLV-001.1
- **Equivalente Rust propuesto**: estado `WaitingForStart` + `StashBuffer`. **Se preserva**: guardar todo hasta `Start`, reemplazo del productor. **Diferencias/límites**: ninguno observable.
- **Aserciones**:
  - A1. reinicio del productor: tras msg-1 y msg-2 un `Start(producerProbe2)` recibe `RequestNext`; msg-3 y msg-4 llegan al worker — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/WorkPullingSpec.scala:244-283
  - A2. con cola duradera, el estado cargado con 3 y 4 sin confirmar no pide `RequestNext` hasta que el worker da demanda — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/DurableWorkPullingSpec.scala:77-116
  - A3. mensajes previos a `Start` en stash y límite del buffer — no afirmada por spec (WorkPullingProducerControllerImpl.scala:229-239); caracterización propuesta: enviar antes de `Start`
- **Validación prevista**: A1-A2 portados; A3 caracterización.
- **Decisiones**: Q-T-DLV-1

## AT-DLV-003.6: WorkPullingProducerController: cola duradera, preselección de worker y traspaso (api-may-change)

- **Contrato observable**: Con `durableQueueBehavior = Some(..)` (hijo `"durable"`, `LoadState` con reintentos como en el punto a punto): (1) el worker se **preselecciona antes** de almacenar para conocer el `confirmationQualifier` (nombre del `ProducerController` del worker) que se guarda en `MessageSent`; (2) la respuesta de `MessageWithConfirmation` (`Done`) se envía **tras almacenar**, sin esperar al worker; (3) las confirmaciones del worker se guardan con `StoreMessageConfirmed(totalSeqNr, qualifier, ahora)`; (4) cuando un worker desaparece, sus no confirmados se **traspasan**: se almacenan con nuevo seqNr y calificador del nuevo worker y se confirma el antiguo (`StoreMessageConfirmed(oldSeqNr, oldQualifier, ..)`) al completarse el almacenamiento; (5) al arrancar, los `unconfirmed` cargados se reenvían a sí mismo como `ResendDurableMsg`.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/internal/WorkPullingProducerControllerImpl.scala:99-111 (preselected, handOver), 313-340, 414-468 (onMessageBeforeDurableQueue, onResendDurableMsg), 470-490 (receiveStoreMessageSentCompleted), 503-527 (onAck con StoreMessageConfirmed), 680-702 (reintentos y storeMessageSent)
- **Precondiciones**: AT-DLV-004 (protocolo de la cola).
- **Comportamiento**: sin worker con demanda antes de almacenar se guarda el mensaje en el stash (`wasStashed = false`: aún no almacenado) o `ResendDurableMsg`; `StoreMessageSentFailed` reintenta con `attempt + 1` hasta `retry-attempts` (`TimeoutException("StoreMessageSentFailed seqNr [s] failed after [n] attempts, giving up.")`); `LoadState` agotado: `TimeoutException("LoadState failed after [n] attempts, giving up.")`; cola muerta: `IllegalStateException("DurableQueue was unexpectedly terminated.")`. Si el worker preseleccionado desapareció cuando se completa el almacenamiento, el mensaje se re-encola como `ResendDurableMsg` (323-325).
- **Errores**: ver arriba; `IllegalStateException("Expected preselected worker for seqNr [n].")` como invariante (328).
- **Orden / concurrencia / ciclo de vida**: almacenamiento por mensaje en el orden de recepción; los traspasos no preservan orden (work-pulling).
- **Dependencias**: AT-DLV-004; AT-DLV-001.6; AT-DLV-003.3; AT-DLV-003.4
- **Equivalente Rust propuesto**: igual que AT-DLV-001.6 con `preselected_workers: HashMap<u64, Preselected>` y `hand_over`. **Se preserva**: preselección antes de almacenar, `Done` tras almacenamiento, traspaso con nuevo seqNr. **Diferencias/límites**: Q-T-DLV-2 (cola real fuera de alcance).
- **Aserciones**:
  - A1. carga de estado (3 y 4 sin confirmar, currentSeqNr 5) y reenvío a un worker: seqMsg3 con `message == msg-3` y tras `Request(1,10)` llega msg-4; luego msg-5 desde el productor — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/DurableWorkPullingSpec.scala:77-116
  - A2. la respuesta `Done` se envía al almacenar, antes de que el worker confirme (`ack = true`) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/DurableWorkPullingSpec.scala:118-159
  - A3. estado almacenado tras msg-1 `State(2,0,..)`, tras `Request` `State(2,1,..)`, tras msg-2..5 `State(6,1,..)` con `ack` correcto por mensaje (true para MessageWithConfirmation), tras un segundo worker y msg-6 `State(7,1,..)`/`State(7,6,..)`, y tras `Ack(3)` quedan 4 y 5 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/DurableWorkPullingSpec.scala:161-282 (`assertState` ignora `confirmedSeqNr` y calificadores)
  - A4. traspaso: al detener worker1 los msg-2..5 se reenvían a worker2 (msg-2 y msg-3 con la ventana de 3, 4 y 5 tras `Request(3,10)`), y tras `Ack(seqMsg9.seqNr)` el estado queda `State(11, 9, .., Vector(MessageSent(10, msg-5,..)))` (nuevo seqNr) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/DurableWorkPullingSpec.scala:284-403
  - A5. errores de reintento (`LoadState`/`StoreMessageSent`) y cola muerta — no afirmada por spec (WorkPullingProducerControllerImpl.scala:211-227, 680-692)
- **Validación prevista**: A1-A4 portados con la cola de prueba en memoria; A5 caracterización.
- **Decisiones**: Q-T-DLV-2; Q-T-DLV-1

## AT-DLV-004: DurableProducerQueue (protocolo) (api-may-change)

- **Contrato observable**: Protocolo `LoadState(replyTo: ActorRef[State])`, `StoreMessageSent(sent, replyTo: ActorRef[StoreMessageSentAck])` -> `StoreMessageSentAck(storedSeqNr)` y `StoreMessageConfirmed(seqNr, confirmationQualifier, timestampMillis)` (sin respuesta), con `NoQualifier = ""`. `State(currentSeqNr, highestConfirmedSeqNr, confirmedSeqNr: Map[Qualifier, (SeqNr, Timestamp)], unconfirmed)`; `State.empty = State(1, 0, Map.empty, Vector.empty)`. `addMessageSent(sent)`: `currentSeqNr = sent.seqNr + 1` y añade a `unconfirmed`. `confirmed(seqNr, qualifier, ts)`: quita de `unconfirmed` los de **ese** qualifier con `seqNr <=` el dado, `highestConfirmedSeqNr = max`, registra `(seqNr, ts)` para el qualifier. `cleanup(qualifiers)` quita entradas de `confirmedSeqNr`. `cleanupPartialChunkedMessages()` descarta trozos incompletos y recalcula `currentSeqNr`. `MessageSent` compara por todos sus campos (igualdad) y `hashCode` solo por `seqNr`. Eventos de persistencia internos: `MessageSent`, `Confirmed`, `Cleanup` (`DeliverySerializable`); la implementación durable (`EventSourcedProducerQueue`) vive en `pekko-persistence-typed` (fuera de estos módulos).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/DurableProducerQueue.scala:41-246
- **Precondiciones**: ninguna (las operaciones de `State` son puras).
- **Comportamiento**: `confirmed` filtra solo por (qualifier, `seqNr <=`): un mensaje con otro qualifier permanece. `cleanupPartialChunkedMessages`: sin sin-confirmar o todos de un trozo -> el mismo estado; si no recorre `unconfirmed`: mensaje de un trozo se acepta y fija `newCurrentSeqNr = seqNr + 1`; primer trozo no final inicia acumulación; un primer trozo nuevo descarta la acumulación previa; trozo final acepta lo acumulado; sin trozo final la secuencia se descarta y `currentSeqNr = último completo + 1` (o `highestConfirmedSeqNr + 1` si no hay completos).
- **Errores**: ninguno propio.
- **Orden / concurrencia / ciclo de vida**: las operaciones de `State` son puras; los comandos se envían al actor durable y se responden de forma asíncrona.
- **Dependencias**: AT-DLV-001.6; AT-DLV-005.3; AT-DLV-003.6
- **Equivalente Rust propuesto**: `trait`/protocolo `DurableCommand<A>`; `struct DurableState<A> { current_seq_nr, highest_confirmed_seq_nr, confirmed_seq_nr: HashMap<String,(u64,i64)>, unconfirmed: Vec<MessageSent<A>> }` con los mismos métodos puros; `MessageSent<A>` con `Either<A, ChunkedMessage>` (el original usa `Any`). Implementación en memoria opcional (Q-T-DLV-2). **Diferencias/límites**: serialización de eventos (`DeliverySerializable`) solo relevante con persistencia.
- **Aserciones**:
  - A1. `State.empty.addMessageSent(MessageSent(1,"a",..))`: `unconfirmed.head.message == "a"` y `currentSeqNr == 2`; segundo: `size == 2` y `currentSeqNr == 3` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/DurableProducerQueueSpec.scala:29-39
  - A2. `confirmed(1,"",0)`: queda "b" y `currentSeqNr == 3` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/DurableProducerQueueSpec.scala:41-49
  - A3. `cleanupPartialChunkedMessages` descarta trozos sin último: queda "a" y `currentSeqNr == 2`; con trozo final quedan a, d, e y `currentSeqNr == 4`; con reinicio de seqNr 4 ("h") quedan a, d, e, h y `currentSeqNr == 5` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/DurableProducerQueueSpec.scala:51-99
  - A4. la cola de prueba responde `LoadState` con el estado limpio, `StoreMessageSentAck` al almacenar el `seqNr == currentSeqNr`, re-ack ante duplicado (`seqNr == currentSeqNr - 1`) y no responde (timeout) ante un seqNr inesperado — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/TestDurableProducerQueue.scala:66-100 (comportamiento exigido por el `ProducerController`, no un spec de `State`)
  - A5. efecto de `confirmed` sobre `highestConfirmedSeqNr` y `confirmedSeqNr` — observado vía `DurableProducerControllerSpec.scala:112-141` (`State(2,1,Map(""->(1,ts)),..)`); no hay test unitario directo de `State.confirmed` sobre esos campos: no afirmada por spec en `DurableProducerQueueSpec`
  - A6. `cleanup(qualifiers)`, `MessageSent.equals/hashCode` y `withConfirmationQualifier`/`withTimestampMillis` — no afirmada por spec (DurableProducerQueue.scala:100-102,168-183); caracterización propuesta: pruebas unitarias directas
- **Validación prevista**: A1-A3 portados como pruebas puras del `State` (diferencial con jar oficial: `State` es una clase pura invocable por jshell); A4 como contrato de la cola de prueba; A5/A6 caracterización.
- **Decisiones**: Q-T-DLV-2

## AT-DLV-005.1: Chunking: partición de mensajes grandes en el ProducerController (api-may-change)

- **Contrato observable**: Con `pekko.reliable-delivery.producer-controller.chunk-large-messages` > 0 (bytes; `off` = 0 desactiva) el `ProducerController` serializa el mensaje con la serialización del sistema (`findSerializerFor`, `serializerId`, `manifest`) y lo parte en trozos de `chunkSize` bytes como `ChunkedMessage(serialized, firstChunk, lastChunk, serializerId, manifest)`; si cabe en un trozo se envía un único `ChunkedMessage` con `firstChunk = lastChunk = true`. **Cada trozo es un `SequencedMessage` con su propio seqNr** consecutivo (el seqNr del mensaje es el del último trozo, que es el único con `ack = true` si se pidió confirmación); cada trozo consume demanda de la ventana y se envía con `SendChunk` interno; el `RequestNext` solo se emite tras el último trozo. Con `chunk-large-messages = off` el mensaje viaja sin serializar. No soportado en work-pulling (AT-DLV-006).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/internal/ChunkedMessage.scala:20-31; actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/internal/ProducerControllerImpl.scala:352-375 (createChunks), 413-415 (invariante), 425-438 (envío de chunks y RequestNext), 715-733 (receiveSendChunk), 735-771 (chunk); actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/ProducerController.scala:167-178 (Settings.apply)
- **Precondiciones**: serialización del sistema configurada para el tipo del mensaje (excepciones de serialización se propagan y hacen fallar al controlador); `chunk-large-messages` <= `Int.MaxValue` bytes.
- **Comportamiento**: `createChunks(m, chunkSize, serialization)`: `bytes.length <= chunkSize` -> 1 trozo; si no `ByteString.grouped(chunkSize)` con `first` para el primero y `last` para el último. `SequencedMessage.fromChunked(producerId, seqNr, chunk, first = seqNr == firstSeqNr, ack = ack && lastChunk, self)`. El control de flujo cuenta trozos: con ventana agotada los trozos restantes esperan a una `Request` (`remainingChunks` + `SendChunk`).
- **Errores**: `IllegalStateException("seqMsg [n] was lastChunk but remaining [k] chunks.")` (413-415, invariante); excepciones de serialización (`NotSerializableException`, etc.) lanzadas tal cual (355); `IllegalArgumentException("Too large chunk-large-messages.")` al construir `Settings` con tamaño > `Int.MaxValue` (168-172); `IllegalStateException("Received unexpected message before sending remaining [k] chunks.")` si llega otro mensaje del productor con trozos pendientes (458-462).
- **Orden / concurrencia / ciclo de vida**: los trozos de un mensaje se envían en orden y contiguos en seqNr; ningún otro mensaje se intercala.
- **Dependencias**: AC-SER (serialización); AT-DLV-001.2; AT-DLV-005.2; AT-DLV-005.3; AT-DLV-006
- **Equivalente Rust propuesto**: requiere un mecanismo de serialización de mensajes (Q-T-SER-1): `trait MessageSerializer<A> { fn serialize(&A) -> (Bytes, SerializerId, Manifest); fn deserialize(..) -> A }` registrado por tipo; `Bytes::chunks(chunk_size)`. **Se preserva**: troceo en `ceil(len / chunkSize)` partes, flags, seqNr por trozo, `ack` solo en el último, un `RequestNext` por mensaje. **Diferencias/límites**: en un runtime local sin serialización el troceo no aporta rendimiento; se mantiene por paridad de comportamiento solo si Q-T-SER-1 lo exige (la fila queda `partial` en implementación hasta decidirlo).
- **Aserciones**:
  - A1. con `chunkLargeMessagesBytes = 1`, `Job("abc")` produce 3 trozos con seqNr 1 (first, no last), 2 (ni first ni last) y 3 (last); los trozos 2 y 3 salen tras `Request(0,10)`; el siguiente mensaje `Job("d")` es un único trozo (first y last) con seqNr 4 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ProducerControllerSpec.scala:263-305
  - A2. toda la batería punto a punto repetida con `chunk-large-messages = 1b` entrega los mismos 42 mensajes (también con ask: tantas respuestas como mensajes, sin comprobar los valores de seqNr) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ReliableDeliverySpec.scala:204-209 (clase `ReliableDeliveryChunkedSpec`) y 51-134
  - A3. con red que pierde mensajes y troceo, la entrega es completa — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ReliableDeliveryRandomSpec.scala:224-228 (clase `ReliableDeliveryRandomChunkedSpec`)
  - A4. `createChunks` con mensaje que cabe en un trozo (un único `ChunkedMessage`) — derivada del test A1 (`Job("d")`, tamaño 1 < len) y de ProducerControllerImpl.scala:359-362; sin aserción directa de la rama `<=`
  - A5. valor `chunk-large-messages` mayor que `Int.MaxValue` — no afirmada por spec (ProducerController.scala:168-172); caracterización propuesta: `Settings(config)` con `"3g"`
- **Validación prevista**: A1-A3 portados (requieren un serializador de prueba equivalente a `TestSerializer`, TestConsumer.scala del spec); A4/A5 caracterización unitaria. Diferencial con jar oficial: número y flags de trozos para una serialización fija.
- **Decisiones**: Q-T-SER-1 (serialización de mensajes); Q-T-DLV-1

## AT-DLV-005.2: Chunking: recolección y reensamblado en el ConsumerController (api-may-change)

- **Contrato observable**: El `ConsumerController` acumula los trozos (`collectedChunks`) y **no entrega nada** hasta recibir el último; entonces concatena los `serialized` en orden y deserializa con `serializerId`/`manifest` del último trozo; entrega un único `Delivery(message, confirmTo, producerId, seqNr)` con el **seqNr del último trozo**. El primer trozo reinicia la recolección. Mientras recolecta sigue aplicando el control de flujo: cuando `requestedSeqNr - seqNr == flowControlWindow / 2` envía `Request(confirmedSeqNr, requestedSeqNr + flowControlWindow / 2, ..)` aunque aún no haya entregado el mensaje. El `first` del mensaje reensamblado es el del primer trozo, de modo que la confirmación del primero ampliando la ventana (AT-DLV-002.3) ocurre al confirmar el mensaje completo.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/internal/ConsumerControllerImpl.scala:101 (collectedChunks), 118-121 (clearCollectedChunks), 498-549 (deliver y assembleChunks); actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/ConsumerController.scala:138-199 (SequencedMessage, isFirstChunk/isLastChunk)
- **Precondiciones**: los trozos llegan en orden de seqNr (AT-DLV-002.4 gestiona huecos); mismo serializador disponible en el consumidor.
- **Comportamiento**: `deliver` con trozo no final: `unstash(active(s.copy(collectedChunks = seqMsg :: previous, requestedSeqNr = nuevo)), 1, ..)`. `assembleChunks`: `ByteString.createBuilder` + `serialization.deserialize(bytes, serializerId, manifest).get`.
- **Errores**: las excepciones de deserialización se lanzan (el controlador falla): "it will anyway be stuck with same error if retried and we can't just ignore the message" (544-546).
- **Orden / concurrencia / ciclo de vida**: entrega única por mensaje; `Confirmed` se espera una vez por mensaje (no por trozo).
- **Dependencias**: AC-SER; AT-DLV-005.1; AT-DLV-002.2; AT-DLV-002.3
- **Equivalente Rust propuesto**: `collected_chunks: Vec<SequencedMessage>` y `MessageSerializer::deserialize`. **Se preserva**: no entregar hasta el último trozo, seqNr del último, `Request` a mitad de ventana durante la recolección, flags `first`. **Diferencias/límites**: Q-T-SER-1 (misma serialización en ambos extremos).
- **Aserciones**:
  - A1. "123" en 3 trozos: ninguna entrega tras el primero; `Request(0,20)` por el primer trozo; tras los 3, `Delivery.payload == "123"`; tras `Confirmed`, `Request(3, 22)` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ConsumerControllerSpec.scala:563-593
  - A2. segundo mensaje "45" (2 trozos, `ack = true`, `first = false`): entrega "45" y tras `Confirmed` se envía `Ack(5)` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ConsumerControllerSpec.scala:595-611
  - A3. 25 trozos con ventana 20: `Request(0,20)` con el primero; sin más hasta el trozo 10 (`Request(0,30)`) y el 20 (`Request(0,40)`); por timer `Request(0,40,true,true)`; con el 25º se entrega el payload completo — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ConsumerControllerSpec.scala:616-667
  - A4. fallo de deserialización — no afirmada por spec (ConsumerControllerImpl.scala:544-546); caracterización propuesta: serializador que falla
- **Validación prevista**: A1-A3 portados con un serializador de prueba determinista (`TestSerializer`: texto UTF-8 con id 787878 y manifest vacío, TestConsumer.scala del spec); A4 caracterización.
- **Decisiones**: Q-T-SER-1; Q-T-DLV-1

## AT-DLV-005.3: Chunking con DurableProducerQueue: almacenamiento por trozo, reintento completo y limpieza al cargar (api-may-change)

- **Contrato observable**: Con cola duradera y troceo, **cada trozo se almacena por separado** (`StoreMessageSent` con `MessageSent.fromMessageOrChunked`, el trozo viaja en `MessageSent.message` como `ChunkedMessage`) y solo tras su ack se envía al consumidor. Si falla el almacenamiento de un trozo que no es el primero, se **almacenan todos los trozos otra vez desde el primero** (los trozos parciales los descarta la cola al reiniciarse). Al cargar, una cola correcta devuelve el estado tras `State.cleanupPartialChunkedMessages()`: los mensajes con trozos incompletos se descartan como si nunca se hubieran almacenado y `currentSeqNr` se recalcula (AT-DLV-004). El `ProducerController` no llama a esa función: es responsabilidad de la implementación de la cola.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/internal/ProducerControllerImpl.scala:589-644 (reintento de trozos), 715-733 (receiveSendChunk con cola duradera); actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/DurableProducerQueue.scala:107-145 (State.cleanupPartialChunkedMessages), 219-226 (MessageSent.fromMessageOrChunked); actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/TestDurableProducerQueue.scala:41 (la cola de prueba limpia al cargar con `cleanupPartialChunkedMessages`)
- **Precondiciones**: AT-DLV-001.6 y AT-DLV-005.1.
- **Comportamiento**: tras el primer trozo almacenado la cola tiene `currentSeqNr = 2` y 1 `unconfirmed` de tipo `ChunkedMessage`; tras todos los trozos `currentSeqNr = 4` y 3 `unconfirmed`. En el reintento del trozo `k > 1` se busca el primer trozo en `unconfirmed` (si no se encuentra: `IllegalStateException("First chunk not found in unconfirmed: ..")`), se reconstruye `remainingChunks = primer :: siguientes ++ pendientes` y se vuelve a almacenar desde el primero con `attempt + 1`; se comprueba que empieza por `isFirstChunk` y acaba en `isLastChunk` (`IllegalStateException("Wrong remainingChunks: ..")`).
- **Errores**: las dos `IllegalStateException` anteriores; agotamiento de reintentos: `TimeoutException` (AT-DLV-001.6).
- **Orden / concurrencia / ciclo de vida**: nunca se envía un trozo al consumidor antes de su almacenamiento; no se pide `RequestNext` mientras haya trozos pendientes.
- **Dependencias**: AT-DLV-001.6; AT-DLV-004; AT-DLV-005.1
- **Equivalente Rust propuesto**: igual que AT-DLV-001.6 más el algoritmo de re-almacenamiento completo. **Se preserva**: almacenamiento por trozo, reintento desde el primer trozo, descarte de parciales al cargar. **Diferencias/límites**: Q-T-DLV-2 y Q-T-SER-1.
- **Aserciones**:
  - A1. `Job("abc")` con troceo 1 y cola duradera: tras el primer trozo `currentSeqNr == 2` y `unconfirmed.size == 1` con `ChunkedMessage`; tras `Request(0,10)` y los demás, `currentSeqNr == 4`, `unconfirmed.size == 3`, último trozo `isLastChunk` seqNr 3 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/DurableProducerControllerSpec.scala:175-226
  - A2. estado cargado con trozos 4 y 5 sin trozo final: el mensaje 3 (un trozo) se envía, 4 y 5 se descartan y el siguiente mensaje "g" sale con seqNr 4 como único trozo — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/DurableProducerControllerSpec.scala:228-291
  - A3. reintento del almacenamiento de trozos con fallo — no afirmada por spec (ProducerControllerImpl.scala:603-639); caracterización propuesta: cola que falla en el segundo trozo y verificar que se vuelve a almacenar desde el primero
  - A4. con cola durable y red que fallan y troceo (variante `Chunked` de las pruebas aleatorias) la entrega termina completa — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ReliableDeliveryRandomSpec.scala:224-228
- **Validación prevista**: A1-A2 y A4 portados con la cola en memoria; A3 caracterización.
- **Decisiones**: Q-T-DLV-2; Q-T-SER-1

## AT-DLV-006: Configuración pekko.reliable-delivery.* y Settings (api-may-change)

- **Contrato observable**: Claves y defaults de `reference.conf`: `producer-controller.chunk-large-messages = off`; `producer-controller.durable-queue.request-timeout = 3s`, `retry-attempts = 10`, `resend-first-interval = 1s`; `consumer-controller.flow-control-window = 50`, `resend-interval-min = 2s`, `resend-interval-max = 30s`, `only-flow-control = false`; `work-pulling.producer-controller` hereda por sustitución `${pekko.reliable-delivery.producer-controller}` y fija `buffer-size = 1000`, `internal-ask-timeout = 60s`, `chunk-large-messages = off`. `Settings(system)`/`Settings(config)` leen esas rutas; los `withX` devuelven copias (inmutables). `ProducerController.Settings`: `chunk-large-messages` se lee con `toRootLowerCase`: `"off"` -> 0; otro -> `getBytes` con `requiring(<= Int.MaxValue, "Too large chunk-large-messages.")` (`IllegalArgumentException`). `WorkPullingProducerController.Settings` lanza `IllegalArgumentException("Chunked messages not implemented for work-pulling yet.")` en su constructor si `producerControllerSettings.chunkLargeMessagesBytes > 0`. `toString`: `ProducerController.Settings(<requestTimeout>, <retryAttempts>, <resendFirstInterval>, <chunkLargeMessagesBytes>)`, `ConsumerController.Settings(<flowControlWindow>, <resendIntervalMin>, <onlyFlowControl>)` (no incluye `resendIntervalMax`), `WorkPullingProducerController.Settings(<bufferSize>,<internalAskTimeout>,<producerControllerSettings>)`.
- **Fuente**: actor-typed/src/main/resources/reference.conf:70-131; actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/ProducerController.scala:154-253 (Settings; toString 251-252); actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/ConsumerController.scala:201-291 (Settings; toString 289-290); actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/WorkPullingProducerController.scala:151-217 (Settings; validación 191-192; toString 215-216)
- **Precondiciones**: las rutas existen en la configuración (si faltan, `ConfigException.Missing` de la librería de configuración).
- **Comportamiento**: `getDuration` admite cualquier formato de duración HOCON; `getBytes` admite sufijos de tamaño (`1b`, `1MiB`...). `work-pulling.producer-controller` es un bloque que se sobreescribe sobre la copia del `producer-controller`, por lo que `Settings(config)` de work-pulling lee también `durable-queue.*` y `chunk-large-messages` de ese bloque.
- **Errores**: `IllegalArgumentException("Too large chunk-large-messages.")`; `IllegalArgumentException("Chunked messages not implemented for work-pulling yet.")`.
- **Orden / concurrencia / ciclo de vida**: n/a (valores inmutables).
- **Dependencias**: AC-CFG; AT-DLV-001.6; AT-DLV-002.3; AT-DLV-003.2; AT-DLV-005.1
- **Equivalente Rust propuesto**: structs `ProducerSettings`/`ConsumerSettings`/`WorkPullingSettings` con `Default` (valores anteriores), `from_config(&Config)` y métodos `with_*` que consumen/devuelven copia. **Se preserva**: claves, defaults, validaciones y mensajes de error, el `toString` (que omite `resendIntervalMax`: se conserva el recorte). **Diferencias/límites**: depende del parser HOCON propio (Q-X-1) con soporte de substitución `${..}` y bloque que sobrescribe (como `work-pulling`) y de `getBytes`.
- **Aserciones**:
  - A1. los defaults de ventana y reintento se pueden sobrescribir y se usan: `flow-control-window = 20` hace que las `Request` sean `Request(0, 20, ..)`, y `resend-interval-min = 1s` fija el intervalo del reintento de `RegisterConsumer` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ConsumerControllerSpec.scala:30-33,47-48,90-109 y 53-65
  - A2. `ProducerController.Settings(system).withChunkLargeMessagesBytes(1)` produce troceo — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ProducerControllerSpec.scala:263-305; y la configuración `chunk-large-messages = 1b` se lee y activa el troceo — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ReliableDeliverySpec.scala:41,204-209
  - A3. `ConsumerController.Settings(system).withOnlyFlowControl(true)` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ConsumerControllerSpec.scala:671-703
  - A4. valores por defecto de `reference.conf` (50, 2 s, 30 s, false, 3 s, 10, 1 s, off, 1000, 60 s) — no afirmada por spec (reference.conf:70-131); caracterización propuesta: leer `Settings(system)` con la configuración por defecto
  - A5. `Settings.toString` de los tres tipos y omisión de `resendIntervalMax` — no afirmada por spec (ProducerController.scala:251; ConsumerController.scala:289-290; WorkPullingProducerController.scala:215-216)
  - A6. `IllegalArgumentException` por `chunk-large-messages` > `Int.MaxValue` y por troceo en work-pulling — no afirmada por spec (ProducerController.scala:168-172; WorkPullingProducerController.scala:191-192); caracterización propuesta: `Settings(config)` con `"3g"` y `WorkPullingProducerController.Settings(config)` con `chunk-large-messages = 1b`
- **Validación prevista**: A1-A3 portados; A4-A6 caracterización unitaria (valores efectivos). Diferencial con jar oficial: volcado de los valores efectivos de `Settings`.
- **Decisiones**: Q-X-1 (HOCON); Q-X-9 (duraciones, Q-UTIL-1)

## AT-DLV-007: Flight recorder (JFR) de actor y delivery (internal-accidental)

- **Contrato observable**: Interno y específico de la JVM. `ActorFlightRecorder` es una extensión tipada cuyo `delivery: DeliveryFlightRecorder` emite 21 eventos de telemetría desde los controladores: 11 del productor (`producerCreated`, `producerStarted`, `producerRequestNext`, `producerSent`, `producerWaitingForRequest`, `producerResentUnconfirmed`, `producerResentFirst`, `producerResentFirstUnconfirmed`, `producerReceived`, `producerReceivedRequest`, `producerReceivedResend`) y 10 del consumidor (`consumerCreated`, `consumerStarted`, `consumerReceived`, `consumerReceivedPreviousInProgress`, `consumerDuplicate`, `consumerMissing`, `consumerReceivedResend`, `consumerSentRequest`, `consumerChangedProducer`, `consumerStashFull`). Con JDK >= 11 y `pekko.java-flight-recorder.enabled = true` (default) se carga `JFRActorFlightRecorder` (cada llamada hace `commit()` de un evento JFR con categoría `Pekko / Delivery / ProducerController|ConsumerController`); si no está disponible o falla su carga (warning "Failed to load JFR flight recorder, falling back to noop. Exception: ..."), se usa `NoOpActorFlightRecorder`. No tiene efecto funcional.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorFlightRecorder.scala:21-111; actor-typed/src/main/scala-jdk-9/org/apache/pekko/actor/typed/internal/jfr/Events.scala:31-181 (21 clases de evento con `@Category`/`@Label`); actor-typed/src/main/scala-jdk-9/org/apache/pekko/actor/typed/internal/jfr/JFRActorFlightRecorder.scala:23-79; actor/src/main/scala/org/apache/pekko/util/FlightRecorderLoader.scala:25-42; actor/src/main/resources/reference.conf:106-110 (`java-flight-recorder.enabled`); puntos de emisión (entre otros): actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/internal/ProducerControllerImpl.scala:151,185,328,333,425,429,432,469,508,647,660,668,677,776,796 y ConsumerControllerImpl.scala:131,150,160,307,324,339,422,452,511,527,568,577,608,611,616,727
- **Precondiciones**: JDK >= 11 para la implementación JFR.
- **Comportamiento**: el recorder se obtiene con `ActorFlightRecorder(system)` al crear cada controlador; las llamadas son inofensivas (`NoOp` por defecto fuera de JFR).
- **Errores**: fallo de carga -> warning y `NoOp`.
- **Orden / concurrencia / ciclo de vida**: las llamadas ocurren en el hilo del actor, en los puntos indicados.
- **Dependencias**: AT-EXT-001.1 (ExtensionId); AT-DLV-001.*; AT-DLV-002.*
- **Equivalente Rust propuesto**: eventos `tracing` (nivel `trace`/`debug`, `target = "pekko::delivery"`) con los mismos 21 nombres de evento y campos (`producer_id`, `seq_nr`, ...) detrás de una feature opcional; **no** se porta JFR (Q-T-DLV-3). **Se preserva**: la existencia y el nombre de los puntos de telemetría (observabilidad), no el formato binario JFR. **Diferencias/límites**: sin JFR no hay herramienta equivalente; `pekko.java-flight-recorder.enabled` se acepta y se ignora o se mapea a la activación de la feature.
- **Aserciones**:
  - A1. emisión de los 21 eventos desde los puntos descritos y la carga con JDK >= 11/`NoOp` — no afirmada por spec (ningún test en actor-typed-tests referencia `FlightRecorder`); caracterización propuesta: con un recorder de prueba inyectado, verificar que una conversación guionizada emite `producerCreated`, `producerStarted`, `producerRequestNext`, `producerSent` y `consumerReceived` en ese orden relativo
  - A2. `FlightRecorderLoader` con JFR deshabilitado devuelve `NoOp` — no afirmada por spec en estos módulos (actor/src/main/scala/org/apache/pekko/util/FlightRecorderLoader.scala:25-42); caracterización propuesta
- **Validación prevista**: caracterización unitaria con un recorder inyectado; sin oráculo (JFR es específico de la JVM).
- **Decisiones**: Q-T-DLV-3 (equivalente del flight recorder)

## AT-CS-001: CoordinatedShutdown en el ActorSystem tipado (reutiliza el clasico) (public)

- **Contrato observable**: Contractual: el ActorSystem tipado no define CoordinatedShutdown propio: terminate() ejecuta CoordinatedShutdown clasico con ActorSystemTerminateReason (si pekko.coordinated-shutdown.run-by-actor-system-terminate=off no lo corre pero igual termina el sistema); detener el guardian de usuario termina el sistema via GuardianStopInterceptor -> system.terminate() (asi corre CoordinatedShutdown); getWhenTerminated/whenTerminated se completan al terminar; terminate inmediato antes de arrancar el guardian es valido (el guardian no recibe PostStop). Tambien `close()` (desde 1.3.0) hace terminate y BLOQUEA hasta `whenTerminated` o hasta pekko.coordinated-shutdown.close-actor-system-timeout (TimeoutException).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorSystem.scala:111-166 (terminate 111-126, whenTerminated 128-146, close 148-166); actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/GuardianStartupBehavior.scala:32-90 (arranque diferido con stash 40-54, GuardianStopInterceptor 57-90); actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorSystemAdapter.scala:117-125 (terminate 119, whenTerminated 120-123, close 125)
- **Condiciones**: close(): bloquea (Await) y lanza TimeoutException si pasa close-actor-system-timeout (Scaladoc actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorSystem.scala:148-166); sin test en este modulo que lo afirme. El guardian solo queda envuelto por GuardianStopInterceptor despues de recibir Start (GuardianStartupBehavior.scala:42-47); antes de Start los mensajes se acumulan (capacidad 1000) y el siguiente lanza StashOverflowException (49-51; AT-SYS-003).
- **Orden / concurrencia / ciclo de vida**: terminate() es asincrono; whenTerminated se completa al final de la secuencia.
- **Dependencias**: filas: AT-SYS-*; AC-CS. Externas: CoordinatedShutdown clasico (AC-CS).
- **Equivalente Rust propuesto**: Delega en el CoordinatedShutdown portado (AC-CS). **Diferencias observables**: Ninguna si AC-CS se porta.
- **Aserciones**:
  - A1. guardian que devuelve Behaviors.stopped termina el sistema y CoordinatedShutdown(sys).shutdownReason() == Some(ActorSystemTerminateReason) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.scala:74-89
  - A2. guardian que para al primer mensaje (issue #24172): whenTerminated se completa — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.scala:91-101
  - A3. terminate(): guardian recibe PostStop ("done") y shutdownReason es ActorSystemTerminateReason — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.scala:103-124
  - A4. terminate antes de arrancar: no hay PostStop y es valido — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.scala:126-135
  - A5. Java: getWhenTerminated, getWhenTerminated sin terminar y try-with-resources (close) — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/ActorSystemTest.java:31-56 (los cuerpos pertenecen a AT-SYS-005; no se releyeron en esta fila)
  - A6. no afirmada por spec: run-by-actor-system-terminate=off (termina sin correr CoordinatedShutdown) y el TimeoutException de close(). Caracterizacion propuesta: oraculo jshell con la clave en off y comprobar `shutdownReason() == None`.
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-X-4

## AT-SER-001: MiscMessageSerializer para ActorRef tipado (id 24) (internal-required)

- **Contrato observable**: Contractual por config: ActorRef tipado (e instancias ActorRefAdapter) se serializan con MiscMessageSerializer (id 24, binding en reference.conf): manifest "a", bytes = ActorRefResolver.toSerializationFormat(ref) en UTF-8; fromBinary resuelve con ActorRefResolver.resolveActorRef; manifest desconocido -> NotSerializableException "Unimplemented deserialization of message with manifest [<m>] in [<clase>]"; objeto no ActorRef -> IllegalArgumentException "Can't serialize object of type <clase> in [<serializer>]" / "Cannot serialize object of type [<clase>]".
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/MiscMessageSerializer.scala:28-55; actor-typed/src/main/resources/reference.conf:38-53
- **Condiciones**: El resolver es lazy val (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/MiscMessageSerializer.scala:33).
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: filas: AT-REF-005; AC-SER. Externas: Serialization extension del clasico (serializers/bindings), UTF-8.
- **Equivalente Rust propuesto**: trait MessageSerializer con ActorRefCodec; solo si se implementa serializacion (Q-T-SER-1). **Diferencias observables**: Q-T-SER-1.
- **Aserciones**:
  - A1. findSerializerFor(ref) es MiscMessageSerializer y round trip da el mismo ref — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorRefSerializationSpec.scala:48-58
  - A2. errores de manifest/objeto (no afirmadas por spec; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/MiscMessageSerializer.scala:36-52)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-T-SER-1

## AT-SER-002: ServiceKeySerializer (id 26) (internal-required)

- **Contrato observable**: Contractual por config: DefaultServiceKey se serializa con ServiceKeySerializer (id 26): manifest = key.typeName, bytes = id en UTF-8; fromBinary reconstruye DefaultServiceKey(id, manifest); otro objeto -> IllegalArgumentException "Can't serialize object of type <clase> in [<serializer>]" o "Cannot serialize object of type [<clase>]".
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/ServiceKeySerializer.scala:26-44; actor-typed/src/main/resources/reference.conf:38-53
- **Condiciones**: El manifest es un nombre de clase JVM: no portable.
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: filas: AT-RCP-002; AC-SER. Externas: Serialization extension del clasico (serializers/bindings), UTF-8.
- **Equivalente Rust propuesto**: Serializer de ServiceKey con (type_name, id). **Diferencias observables**: Q-T-SER-2.
- **Aserciones**:
  - A1. round trip de ServiceKey[Int]("testKey") igual al original — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/receptionist/ServiceKeySerializationSpec.scala:33-43
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh (bytes y manifest). Procedencia: jar oficial.
- **Decisiones**: Q-T-SER-1;Q-T-SER-2

## AT-SER-003: Serializacion Java de ActorRef (SerializedActorRef + currentSystem) (internal-accidental)

- **Contrato observable**: Accidental/JVM: ActorRef (y ActorRefAdapter) es java.io.Serializable via writeReplace -> SerializedActorRef(address) y readResolve usando JavaSerializer.currentSystem; sin sistema en scope lanza IllegalStateException "Trying to deserialize a serialized typed ActorRef without an ActorSystem in scope." (texto completo mas largo, sin leer completo).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorRef.scala:60-112; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorRefAdapter.scala:53-54
- **Condiciones**: Requiere allow-java-serialization=on en el spec.
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: filas: AC-SER. Externas: Serialization extension del clasico (serializers/bindings), UTF-8.
- **Equivalente Rust propuesto**: Sin equivalente (serializacion Java); Q-T-SER-3 propone no portar o equivalente serde. **Diferencias observables**: Q-T-SER-3.
- **Aserciones**:
  - A1. MessageWrappingActorRef("some message", ref) hace round trip con JavaSerializer — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorRefSerializationSpec.scala:60-72
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-T-SER-3

## AT-JAPI-001.1: javadsl.Behaviors: fabricas delegadas sin logica propia (public)

- **Contrato observable**: Contractual (fachada): javadsl.Behaviors reexpone, con tipos Java (japi.function, java.util.function, Class[T] en lugar de ClassTag, Supplier, java.util.Map), las fabricas de scaladsl.Behaviors. Cada metodo de esta fila solo convierte tipos y delega en la misma implementacion interna que la API Scala, de modo que su semantica es la de las filas Scala indicadas. Metodos: setup (AT-BEH-007), withStash (AT-BEH-021), same (AT-BEH-002), unhandled (AT-BEH-003), stopped y stopped(Effect) (AT-BEH-004), empty (AT-BEH-005), ignore (AT-BEH-006), receive(onMessage) y receiveMessage (AT-BEH-008.1), receiveMessageWithSame (AT-BEH-008.3), intercept (AT-BEH-013/015), monitor (AT-BEH-017), logMessages (AT-BEH-018), supervise/onFailure (AT-SUP-002), transformMessages (AT-BEH-016), withTimers (AT-BEH-020), receive(Class[T]) (devuelve BehaviorBuilder: AT-JAPI-003). Diferencia de superficie: javadsl NO tiene receivePartial / receiveMessagePartial (el equivalente Java es receive(Class) + BehaviorBuilder).
- **Fuente**: EVIDENCIA DE DELEGACION (fachada equivalente), javadsl -> implementacion compartida: setup `BehaviorImpl.DeferredBehavior(ctx => factory.apply(ctx.asJava))` actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/Behaviors.scala:51-52 (Scala: actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Behaviors.scala:39-40); withStash `setup(ctx => factory(StashBufferImpl[T](ctx.asScala, capacity)))` actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/Behaviors.scala:57-60; same/unhandled/stopped/stopped(Effect)/empty/ignore -> BehaviorImpl actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/Behaviors.scala:68,76,87,101,106,111; receive(onMessage) y receiveMessage -> `new BehaviorImpl.ReceiveBehavior(...)` actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/Behaviors.scala:125-126,145-146; receiveMessageWithSame actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/Behaviors.scala:166-170; receive(Class) -> BehaviorBuilder.create actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/Behaviors.scala:205; intercept `BehaviorImpl.intercept(() => supplier.get())(behavior)` actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/Behaviors.scala:225-226 (Scala: actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Behaviors.scala:190-191); monitor `scaladsl.Behaviors.monitor(monitor, behavior)(ClassTag(interceptMessageClass))` actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/Behaviors.scala:240-241; logMessages actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/Behaviors.scala:248-257; supervise -> `new SuperviseBehavior[T](wrapped).onFailure(...)` actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/Behaviors.scala:284-304 (impl compartida actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:77-99); transformMessages `BehaviorImpl.transformMessages(behavior, selector.apply(new PFBuilder).build())(ClassTag(interceptMessageClass))` actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/Behaviors.scala:335-339; withTimers `TimerSchedulerImpl.withTimers(timers => factory.apply(timers))` actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/Behaviors.scala:348-349; tests: actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/InterceptTest.java:33-111; actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/WatchTest.java:86-130; actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ActorCompile.java:47-235; actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ActorLoggingTest.java:115-131
- **Condiciones**: (1) Comportamiento propio de la fachada, solo ergonomia: `Class[T]` reemplaza a `ClassTag[T]` en monitor y transformMessages (`ClassTag(clazz)` en las citas de arriba), y por tanto el filtro por clase del interceptor usa `Class.isAssignableFrom` sobre la clase en tiempo de ejecucion. (2) Las lambdas japi.function (setup, receive, withTimers, ActorContext.ask...) permiten excepciones comprobadas (se tratan como fallo del actor; ver ActorCompile.java:197-206,208-215,227-231). (3) transformMessages: el `selector` (PFBuilder) se evalua UNA vez al construir el Behavior (actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/Behaviors.scala:339), los mensajes que el selector no cubre se descartan (ejemplo del Scaladoc actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/Behaviors.scala:312-323).
- **Comportamiento**: Idem al de las filas Scala enlazadas; no hay logica Java adicional salvo la conversion de tipos descrita.
- **Errores**: Los de las filas Scala. La fachada no agrega excepciones propias.
- **Orden / concurrencia / ciclo de vida**: Identico al de las filas Scala (setup diferido hasta el arranque del actor; supervise re-envuelve; timers cancelados en restart).
- **Dependencias**: filas: AT-BEH-002; AT-BEH-003; AT-BEH-004; AT-BEH-005; AT-BEH-006; AT-BEH-007; AT-BEH-008.1; AT-BEH-008.3; AT-BEH-013; AT-BEH-015; AT-BEH-016; AT-BEH-017; AT-BEH-018; AT-BEH-020; AT-BEH-021; AT-SUP-002; AT-JAPI-003. Externas: JVM: japi.function, java.util.function.
- **Equivalente Rust propuesto**: Una sola API Rust (Q-T-JAPI-1): estas fabricas son las de las filas AT-BEH-* sin fachada adicional. SE PRESERVA: toda la semantica (es la de las filas Scala). SOLO ERGONOMIA JAVA (no se porta): `Class[T]` vs `ClassTag`, tipos japi.function, excepciones comprobadas, Supplier de interceptores. En Rust el filtro por clase pasa a ser un tipo/`TypeId` o un enum de protocolo (decision de las filas AT-BEH-013/AT-BEH-016, no de esta).
- **Aserciones**:
  - A1. intercept(() -> interceptor, receiveMessage(...)): el interceptor ve "Hello" y luego el comportamiento interno lo recibe — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/InterceptTest.java:33-66
  - A2. interceptor con `Message.class` intercepta instancias de subclases (A y B) y las pasa en orden al interior — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/InterceptTest.java:74-111
  - A3. receive(onMessage, onSignal) + context.watch(hijo): al parar el hijo la funcion de senales recibe Terminated y responde Done (ask completa en 3 s) — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/WatchTest.java:86-106
  - A4. context.watchWith(hijo, CustomTerminationMessage): el mensaje personalizado llega como mensaje normal — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/WatchTest.java:108-130
  - A5. Behaviors.logMessages(Behaviors.empty()) registra un evento con mensaje "received message Hello" — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ActorLoggingTest.java:115-131 (ATENCION: no afirmada en la practica; el test crea un CustomEventFilter pero no lo instala (no usa intercept ni Mute) y descarta el Boolean de awaitDone (actor-testkit en testkit/src/main/scala/org/apache/pekko/testkit/TestEventListener.scala:106-109, assertDone 115-119); solo falla si spawn/tell lanzan. La afirmacion real vive en AT-BEH-018 (Scala))
  - A6. SOLO COMPILA (sin ejecucion): receive(2 funciones), receive(1), empty, ignore, intercept, narrow, setup+monitor, transformMessages (dos formas), stopped(Effect), AskPattern.ask, messageAdapter, withTimers (con y sin clave), supervise+onFailure, withStash con forEach que lanza excepcion comprobada — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ActorCompile.java:47-235
  - A7. no afirmada por spec (en Java): ejecucion de setup, withStash, same, unhandled, stopped(Effect), receiveMessageWithSame, monitor (incluido el filtro por clase), supervise y transformMessages desde la fachada Java. Caracterizacion propuesta: no se escribe una prueba Java; la semantica ya la afirman las filas AT-BEH-* / AT-SUP-002 (Scala), y la delegacion se valida leyendo las lineas citadas.
- **Validacion prevista / Oraculo**: No requiere golden propio. Validacion: (a) las aserciones de las filas AT-BEH-* / AT-SUP-002 portadas a Rust; (b) A1-A4 son escenarios de interceptor/watch ya cubiertos por AT-BEH-013 y AT-CTX-005/006, se reutilizan como pruebas de integracion. Opcionalmente un escenario jshell que invoque javadsl.Behaviors.monitor con una clase no ajustada para fijar el filtro por clase (no afirmado en Java).
- **Decisiones**: Q-T-JAPI-1 (no portar fachada Java; las aserciones Scala cubren esta fila). Q-T-BEH-* de las filas enlazadas para el filtro por clase.

## AT-JAPI-001.2: javadsl.Behaviors.receive(onMessage, onSignal) y receiveSignal: funciones de senal totales (public)

- **Contrato observable**: Contractual (comportamiento propio de la fachada Java): `Behaviors.receive(onMessage, onSignal)` envuelve la funcion de senales Java como una funcion TOTAL (`{ case (ctx, sig) => onSignal.apply(ctx.asJava, sig) }`), a diferencia de scaladsl donde la `PartialFunction` no definida cae al manejador por defecto. Consecuencia: con la fabrica de dos funciones nunca se alcanza el valor por defecto de BehaviorImpl.unhandledSignal (relanzar MessageAdaptionFailure; resto -> unhandled): si la funcion Java devuelve `unhandled` ante una MessageAdaptionFailure el actor NO falla (el adaptador ignora la senal no manejada), mientras que `Behaviors.receive(onMessage)` de una funcion y `receiveMessage` SI relanzan la excepcion del adaptador (AT-SIG-007). `Behaviors.receiveSignal(handler)` = `receive(two2same, handler)`: los MENSAJES se ignoran devolviendo `same` (no `unhandled`, no se publica UnhandledMessage); es el mismo efecto que scaladsl `receive((_, _) => same).receiveSignal(handler)`.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/Behaviors.scala:35-38,184-191,210-212 (two2same y receive de dos funciones); actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/Behaviors.scala:125-126,145-146 (receive/receiveMessage de una funcion: sin onSignal explicito); actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/BehaviorImpl.scala:73-76 (unhandledSignal) y 135-150 (ReceiveBehavior: onSignal por defecto = unhandledSignal; applyOrElse); actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Behaviors.scala:176-177 (receiveSignal Scala); actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:270-277 (Terminated no manejado -> DeathPactException); actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:220-229 (senal unhandled: "that's ok"); tests: actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/WatchTest.java:56-84
- **Condiciones**: La funcion de senales Java debe devolver explicitamente `unhandled()` (o `same()`) para cada senal que no maneje; una Terminated devuelta como unhandled sigue produciendo DeathPactException en Behavior.interpretSignal (actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:274), porque ese chequeo vive en el interprete y no en el manejador por defecto.
- **Comportamiento**: (1) receive(m, s): mensajes -> m(ctx, msg); senales -> s(ctx, sig) para toda senal (PreRestart, PostStop, Terminated, ChildFailed, MessageAdaptionFailure...). (2) receive(m) / receiveMessage(m): senales -> unhandledSignal (MessageAdaptionFailure se relanza; el resto unhandled). (3) receiveSignal(s): mensajes -> same.
- **Errores**: Una excepcion (incluso comprobada) lanzada por m o s es fallo del actor (supervision). MessageAdaptionFailure: ver Comportamiento (2) frente a (1).
- **Orden / concurrencia / ciclo de vida**: Todo en el hilo del actor; las funciones se invocan con `ctx.asJava` (el mismo objeto ActorContext).
- **Dependencias**: filas: AT-BEH-008.1; AT-BEH-009; AT-SIG-006; AT-SIG-007; AT-JAPI-001.1. Externas: JVM: japi.function.Function2.
- **Equivalente Rust propuesto**: SE PRESERVA el efecto observable de que el manejador de senales por defecto solo se aplica cuando el usuario no suministra uno: un manejador `Fn(&mut Ctx, Signal) -> Behavior` total no hereda el relanzamiento de MessageAdaptionFailure; si Rust ofrece un manejador parcial (`Option<Behavior>`) el equivalente de (1) es el caso en que el usuario devuelve `Some(unhandled)`. SOLO ERGONOMIA JAVA: que Java obligue a funciones totales (no tiene PartialFunction); en Rust basta la forma parcial de AT-BEH-009 mas la forma total. El requisito es documentar la diferencia y probarla.
- **Aserciones**:
  - A1. receive((ctx,msg)->unhandled(), (ctx,sig)-> si Terminated avisa y devuelve same()) recibe la senal Terminated del hijo vigilado — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/WatchTest.java:63-72 usado por 86-106
  - A2. receive(m) con una sola funcion y receive(m, s) compilan; stopped(() -> {}) con receive de dos funciones compila — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ActorCompile.java:47-49,98-99 (solo compila)
  - A3. no afirmada por spec: con receive(m, s) cuya s devuelve unhandled() ante MessageAdaptionFailure el actor sigue vivo, mientras que receive(m) de una funcion muere (contraste con AT-SIG-007 A1). Caracterizacion propuesta: escenario nuevo con un messageAdapter que lanza en cada una de las dos fabricas y observar Terminated del actor.
  - A4. no afirmada por spec: Behaviors.receiveSignal(handler) ignora mensajes devolviendo same (sin UnhandledMessage). Caracterizacion propuesta: suscribirse a UnhandledMessage, enviar un mensaje al actor de receiveSignal y comprobar que NO se publica (en contraste con Behaviors.receive(m) que devuelve unhandled).
- **Validacion prevista / Oraculo**: Oraculo con jshell + jar oficial pekko-actor-typed_2.13 1.7.1 para A3 y A4 (escenario nuevo, aun no existe). A1 como prueba portada de integracion (WatchTest). Procedencia: jar oficial.
- **Decisiones**: Q-T-JAPI-1 (la fila sigue en alcance mientras el Owner no autorice la exclusion, porque A3/A4 son semantica propia de la fachada que la API Rust unica tiene que decidir explicitamente).

## AT-JAPI-001.3: javadsl.Behaviors.withMdc: normalizacion de mapas (public)

- **Contrato observable**: Contractual (comportamiento propio de la fachada Java): tres sobrecargas withMdc(Class[T], ...) que convierten java.util.Map a Map Scala y delegan en WithMdcBehaviorInterceptor: (a) solo MDC por mensaje (`Function[T, Map]`), (b) solo MDC estatico, (c) combinado. Reglas propias de la fachada: un mapa estatico `null` o vacio equivale a `Map.empty`; una funcion `mdcForMessage` `null` equivale a "siempre vacio"; un mapa devuelto por mdcForMessage que sea `null` o vacio equivale a `Map.empty` (la conversion `asScalaMap` tolera null). Si una clave esta en ambos, la del mensaje sobrescribe a la estatica (Scaladoc). Solo se interceptan mensajes de la clase indicada o subclases (`ClassTag(interceptMessageClass)`).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/Behaviors.scala:362-420 (sobrecargas 362-366, 378-382, 402-420; asScalaMap 408-411; funcion nula 413-417; delegacion a `WithMdcBehaviorInterceptor` 419); Scala: actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Behaviors.scala:284,298,320; test: actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ActorLoggingTest.java:83-113
- **Condiciones**: El MDC se limpia tras procesar cada mensaje (Scaladoc actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/Behaviors.scala:357-358); se refleja en `context.getLog` (AT-LOG-002).
- **Comportamiento**: Ver Contrato observable. La normalizacion de null evita NullPointerException en las rutas Java -> Scala.
- **Errores**: Si mdcForMessage lanza, es fallo del actor (se ejecuta antes de entregar el mensaje al interior; semantica de AT-BEH-019).
- **Orden / concurrencia / ciclo de vida**: mdcForMessage se invoca en el hilo del actor antes de cada mensaje de la clase.
- **Dependencias**: filas: AT-BEH-019; AT-LOG-002; AT-JAPI-001.1. Externas: JVM: java.util.Map; SLF4J MDC.
- **Equivalente Rust propuesto**: SE PRESERVA el comportamiento de AT-BEH-019 (estatico + por mensaje, el del mensaje gana, limpieza tras cada mensaje). SOLO ERGONOMIA JAVA: la tolerancia a `null` (en Rust `Option<HashMap>` o mapa vacio); no se porta `Class[T]`.
- **Aserciones**:
  - A1. withMdc(Protocol.class, fn msg->{txId}, receive(...)) : se espera un evento de log con MDC que contiene "txId" — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ActorLoggingTest.java:83-113 (ATENCION: no afirmada en la practica: el test NO envia ningun mensaje al actor y su CustomEventFilter no se instala ni se afirma con assertDone; ver nota en AT-JAPI-001.1 A5)
  - A2. no afirmada por spec: mapa estatico null/vacio, mdcForMessage null, mapa devuelto null, precedencia de la clave del mensaje, y filtro por clase. Caracterizacion propuesta: oraculo jshell con un LogEvent capturado (LoggingTestKit) para cada combinacion; las partes no especificas de Java las afirma AT-BEH-019 para Scala.
- **Validacion prevista / Oraculo**: Diferencial con el jar oficial (jshell) para null/vacio; el resto se hereda de AT-BEH-019. Procedencia: jar oficial.
- **Decisiones**: Q-T-JAPI-1; Q-T-BEH-6 (MDC) de AT-BEH-019.

## AT-JAPI-002.1: javadsl.AbstractBehavior (estilo orientado a objetos Java) (public)

- **Contrato observable**: Contractual (comportamiento propio de la fachada Java, sin test Java que lo afirme): `AbstractBehavior<T>(context)` es un ExtensibleBehavior cuyo `receive`/`receiveSignal` son `final` y delegan en el `Receive<T>` producido por `createReceive()`, creado PEREZOSAMENTE en el primer mensaje o senal y memorizado (`createReceive` se invoca como maximo una vez por instancia). Reglas: (1) constructor con `context == null` lanza IllegalArgumentException "context must not be null. Wrap in Behaviors.setup and pass the context to the constructor of AbstractBehavior."; (2) en cada mensaje o senal se comprueba que el ActorContext recibido sea el mismo objeto con que se construyo; si no, IllegalStateException "Actor [<self>] of AbstractBehavior class [<clase>] was created with wrong ActorContext [<self del contexto>]. Wrap in Behaviors.setup and pass the context to the constructor of AbstractBehavior." (se detecta en el primer mensaje, no al hacer spawn); (3) `getContext()` devuelve el contexto del constructor; (4) `newReceiveBuilder()` = ReceiveBuilder.create.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/AbstractBehavior.scala:45-94 (null 47-50; cache de Receive 52-59; getContext 61; checkRightContext 63-69; receive/receiveSignal finales 71-81; createReceive 87; newReceiveBuilder 93); Receive: actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/Receive.scala:27-68; equivalente Scala (para contraste): actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/AbstractBehavior.scala:78-103; tests: actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ReceiveBuilderTest.java:37-65; actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/javadsl/AdaptationFailureSpec.scala:30-107; actor-typed-tests/src/test/java/jdocs/org/apache/pekko/typed/OnMessageIntroTest.java:253-288
- **Condiciones**: La instancia debe crearse dentro de Behaviors.setup (una instancia nueva por reinicio con supervision de restart); reutilizar el mismo contexto de otro actor (p.ej. el del padre) falla en el primer mensaje. Cambiar a otro AbstractBehavior puede reutilizar el contexto original.
- **Comportamiento**: receive(ctx,msg) = checkRightContext + receive.receive(ctx,msg); idem senales. El Receive memorizado se comparte entre mensajes; el estado mutable vive en campos de la clase.
- **Errores**: IllegalArgumentException (constructor, antes de cualquier mensaje), IllegalStateException (contexto equivocado; dos apariciones en el log: una al mensaje y otra por la senal PostStop, como muestran los tests Scala). Tipo y texto contractuales; el hash/identidad del self varia.
- **Orden / concurrencia / ciclo de vida**: createReceive se llama en el hilo del actor al primer mensaje/senal (no en el constructor); la caché `_receive` es una variable no sincronizada, valida porque el Behavior solo se ejecuta en el hilo del actor.
- **Dependencias**: filas: AT-BEH-010; AT-BEH-007; AT-SUP-013; AT-CTX-016; AT-SIG-007; AT-JAPI-002.2. Externas: JVM: reflexion `getClass.getName` para el mensaje.
- **Equivalente Rust propuesto**: Una API Rust unica (Q-T-JAPI-1). SE PRESERVA: (a) el comportamiento "construye con el contexto del actor y falla pronto si es de otro" (en Rust normalmente imposible por tipos/lifetimes; si se permite pasar un contexto por valor hay que conservar el IllegalState equivalente), (b) la construccion diferida del receptor (createReceive una vez) solo si Rust expone un Behavior orientado a objetos (AT-BEH-010). SOLO ERGONOMIA JAVA: herencia de clase abstracta, metodos protegidos, caché por `OptionVal`.
- **Aserciones**:
  - A1. Con AbstractBehavior vacio (newReceiveBuilder.build()) y con uno que maneja solo PreRestart, un adaptador de mensajes que lanza hace TERMINAR al actor (probe.expectTerminated) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/javadsl/AdaptationFailureSpec.scala:54-77
  - A2. Con un AbstractBehavior que maneja MessageAdaptionFailure el actor sigue vivo (no se observa Terminated en el observador, probe.expectNoMessage()) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/javadsl/AdaptationFailureSpec.scala:79-106 (la Promise `threw` no se afirma)
  - A3. El contador mutable con newReceiveBuilder().onMessage(Increase.class, ...).onMessage(Get.class, ...) COMPILA y construye el Behavior, pero NUNCA se hace spawn ni se envia un mensaje: el test no afirma nada — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ReceiveBuilderTest.java:37-65
  - A4. no afirmada por spec (javadsl): IllegalArgumentException con context nulo; IllegalStateException por contexto equivocado (mensaje y senal); createReceive invocado una sola vez. NOTA: los tests con esos textos (SupervisionSpec.scala:1550-1570, ActorThreadSpec.scala:188-215,255-275) prueban scaladsl.AbstractBehavior, no el javadsl. Caracterizacion propuesta: oraculo jshell con una subclase Java anonima; el contador de invocaciones de createReceive se mide con una subclase que cuenta.
- **Validacion prevista / Oraculo**: A1/A2 se portan como pruebas (reutilizan AT-SIG-007). A4 como escenario jshell nuevo (jar oficial 1.7.1) con clase Java compilada en el propio jshell. Procedencia: jar oficial.
- **Decisiones**: Q-T-JAPI-1; Q-T-BEH-* de AT-BEH-010 para el chequeo de contexto.

## AT-JAPI-002.2: javadsl.Receive / ReceiveBuilder (builder mutable de casos) (public)

- **Contrato observable**: Contractual (comportamiento propio de la fachada Java): `ReceiveBuilder<T>` es un builder MUTABLE (cada onXxx modifica y devuelve `this`) cuyos casos se evaluan EN EL ORDEN EN QUE SE AGREGARON; gana el primer caso cuyo tipo (`Class.isAssignableFrom(msg.getClass)`) y predicado opcional coinciden; sin coincidencia (o sin casos) el resultado es `Behaviors.unhandled`. Operaciones: onMessage(Class, handler), onMessage(Class, predicado, handler), onMessageUnchecked (sin chequeo de tipo generico), onMessageEquals(msg, Creator) (coincide con mensajes asignables a `msg.getClass` Y `==`/equals a msg; el handler no recibe el mensaje), onAnyMessage (todo lo posterior queda inalcanzable), onSignal(Class,handler), onSignal(Class,predicado,handler), onSignalEquals(signal, Creator). `build()` devuelve `Receive<T>` (ExtensibleBehavior sellado `@DoNotInherit`) con arreglos inmutables copiados en ese momento. Regla de senales por defecto: si el usuario NO registro ningun manejador de senal, el Receive usa solo el manejador por defecto que RELANZA la excepcion de MessageAdaptionFailure; si registro alguno, el manejador por defecto se agrega AL FINAL de la lista (los del usuario, incluidos los de MessageAdaptionFailure, tienen prioridad). Toda otra senal no manejada -> unhandled.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/ReceiveBuilder.scala:32-44 (build; signalHandlers vacio vs adapterExceptionSignalHandler al final tras reverse), 54-67 (onMessage), 79-80 (unchecked), 89-97 (onMessageEquals: `msg.getClass` + predicado `param == msg`), 106-107 (onAnyMessage), 117-149 (senales), 151-165 (withMessage/withSignal: prepend a var mutable), 171 (create), 182-198 (defaultSignalHandlers); actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/ReceiveBuilder.scala:207-232 (BuiltReceive: handlers vacios -> unhandled; primer coincidente; ultimo sin coincidencia -> unhandled); Receive: actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/Receive.scala:27-68; tests: actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ReceiveBuilderTest.java:67-166; actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/javadsl/AdaptationFailureSpec.scala:30-107
- **Condiciones**: Los handlers son japi.function (pueden lanzar excepciones comprobadas). El tipo coincide por clase en tiempo de ejecucion (los genericos se borran; para List<String> usar onMessageUnchecked). Reutilizacion del builder: como los onXxx mutan el mismo objeto, un segundo `build()` tras agregar casos produce un Receive distinto que incluye los nuevos casos; el Receive ya construido no cambia (los arreglos se copian en build). El builder no es seguro para uso concurrente.
- **Comportamiento**: Ver Contrato observable. Ejemplo: dos onMessage(String.class) -> siempre el primero; onMessage(String.class, predicado) cuyo predicado falla cae al siguiente caso.
- **Errores**: Excepciones de handlers y predicados se propagan como fallo del actor. Un predicado se evalua solo si el tipo coincide (cortocircuito `&&`, ReceiveBuilder.scala:224).
- **Orden / concurrencia / ciclo de vida**: Orden de insercion estricto (la lista interna se guarda invertida y se invierte en build()). El Receive se invoca en el hilo del actor.
- **Dependencias**: filas: AT-BEH-008.1; AT-BEH-009; AT-SIG-007; AT-JAPI-002.1; AT-JAPI-003. Externas: JVM: Class.isAssignableFrom, japi.function.Function/Predicate/Creator.
- **Equivalente Rust propuesto**: SE PRESERVA: seleccion del primer caso coincidente en orden de registro; `unhandled` por defecto cuando nada coincide; onAnyMessage como comodin que sombrea lo posterior; manejador de senal por defecto que relanza MessageAdaptionFailure solo cuando el usuario no lo maneja (prioridad del usuario); onMessageEquals como (tipo + igualdad). En Rust: `match` sobre un enum de protocolo o un builder con `TypeId`/downcast; la mutabilidad del builder se vuelve `self` por valor (el efecto "un segundo build incluye casos nuevos" desaparece). SOLO ERGONOMIA JAVA: `Class<M>` vs generico, onMessageUnchecked (no aplica), lambdas con excepcion comprobada.
- **Aserciones**:
  - A1. dos onMessage(String.class) -> "message" lo atiende el primer handler ("handler 1: message") — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ReceiveBuilderTest.java:67-88
  - A2. onMessage(String.class, "other"::equals-like predicado, h1) seguido de onMessage(String.class, h2): "message" no cumple el predicado y lo atiende h2 — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ReceiveBuilderTest.java:90-112
  - A3. onAnyMessage atrapa cualquier mensaje ("message" llega al sondeo) — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ReceiveBuilderTest.java:114-128
  - A4. onMessageEquals / onMessage / onMessage con predicado / onSignalEquals / onSignal / onSignal con predicado aceptan handlers que lanzan Exception comprobada (solo compila; el metodo no tiene @Test) — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ReceiveBuilderTest.java:130-166
  - A5. Receive sin manejador de MessageAdaptionFailure (vacio, o solo con PreRestart): un adaptador que lanza termina el actor — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/javadsl/AdaptationFailureSpec.scala:54-77
  - A6. Receive con onSignal(MessageAdaptionFailure.class, ... same): el actor sobrevive — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/javadsl/AdaptationFailureSpec.scala:79-106 (el manejador del usuario precede al por defecto)
  - A7. no afirmada por spec: onMessageEquals (coincidencia por clase + igualdad, handler sin mensaje), onMessageUnchecked, onSignalEquals, sin coincidencia -> unhandled, orden de senales, reutilizacion del builder tras build(). Caracterizacion propuesta: pruebas Rust con esos escenarios y oraculo jshell para el comportamiento de "build() dos veces con casos intermedios".
- **Validacion prevista / Oraculo**: A1-A3 y A5-A6 portados 1:1 como pruebas (con TestProbe/Terminated). A7 con escenario jshell nuevo contra el jar oficial 1.7.1. Procedencia: jar oficial.
- **Decisiones**: Q-T-JAPI-1 (las propiedades de orden/unhandled/senal por defecto son de la fachada Java y deben quedar explicitas en la API Rust unica).

## AT-JAPI-003: javadsl.BehaviorBuilder (builder inmutable de mensajes y senales) (public)

- **Contrato observable**: Contractual (comportamiento propio de la fachada Java): `BehaviorBuilder<T>` es INMUTABLE: cada onXxx devuelve un builder NUEVO (`create` devuelve una instancia vacia compartida), de modo que se puede ramificar a partir de un builder intermedio sin efectos cruzados (contraste con ReceiveBuilder, AT-JAPI-002.2). Casos en orden de insercion, primer caso coincidente (clase por `isAssignableFrom` y predicado opcional), sin coincidencia -> `Behaviors.unhandled`. Operaciones: onMessage, onMessage con predicado, onMessageUnchecked, onMessageEquals (clase de `msg` + igualdad; handler sin mensaje), onAnyMessage, onSignal, onSignal con predicado, onSignalEquals. `build()` devuelve un ExtensibleBehavior (BuiltBehavior) que NO es un `Receive` ni un AbstractBehavior. A diferencia de ReceiveBuilder NO agrega manejador por defecto para MessageAdaptionFailure: sin manejador de senal explicito la senal cae en `unhandled`, que el adaptador ignora, de modo que un adaptador que lanza NO termina el actor (distinto de Behaviors.receive(m) de una funcion y de AbstractBehavior; ver AT-JAPI-001.2).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/BehaviorBuilder.scala:42-48 (build), 59-72 (onMessage), 84-85 (unchecked), 94-98 (onMessageEquals), 107-108 (onAnyMessage), 118-147 (senales), 149-164 (withMessage/withSignal: `new BehaviorBuilder` con el caso antepuesto), 167-185 (companion, `_empty`), 192-214 (BuiltBehavior: sin handlers o sin coincidencia -> Behaviors.unhandled); tests: actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/BehaviorBuilderTest.java:51-201; evidencia de la senal no manejada: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:220-229 y actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:270-277
- **Condiciones**: Los handlers son japi.function. En onMessage con predicado el predicado recibe el mensaje ya tipado (cast sin chequeo, BehaviorBuilder.scala:72). onSignal con `Class[M]` convierte el handler a `JFunction[Signal, ...]` (cast, BehaviorBuilder.scala:119). No es thread-safe en el sentido de que la unica garantia es la inmutabilidad de los builders.
- **Comportamiento**: Ver Contrato observable. Un Terminated no manejado por el BehaviorBuilder devuelve unhandled y por tanto Behavior.interpretSignal lanza DeathPactException (Behavior.scala:274).
- **Errores**: Excepciones de handlers/predicados -> fallo del actor. No hay errores de construccion propios.
- **Orden / concurrencia / ciclo de vida**: Orden de insercion; el comportamiento resultante es inmutable y se ejecuta en el hilo del actor.
- **Dependencias**: filas: AT-BEH-008.1; AT-BEH-009; AT-SIG-006; AT-SIG-007; AT-JAPI-001.1; AT-JAPI-002.2. Externas: JVM: Class.isAssignableFrom, japi.function.
- **Equivalente Rust propuesto**: Una API unica (Q-T-JAPI-1). SE PRESERVA: primer-caso-coincidente en orden, `unhandled` sin coincidencia, onAnyMessage sombrea lo posterior, inmutabilidad (builder consumido por valor), y la diferencia de senales por defecto (sin relanzar MessageAdaptionFailure) SOLO si la API Rust ofrece un builder con esa semantica; si se unifica con ReceiveBuilder el Owner debe decidir cual de los dos defaults se adopta (hoy divergen en Pekko). SOLO ERGONOMIA JAVA: `Class<M>`, onMessageUnchecked, Creator.
- **Aserciones**:
  - A1. dos onMessage(String.class) -> atiende el primero ("handler 1: message") — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/BehaviorBuilderTest.java:76-97
  - A2. onMessageEquals("message", ...) atiende el mensaje igual ("got it") — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/BehaviorBuilderTest.java:99-114
  - A3. onMessage con predicado "other"::equals: "message" cae al segundo handler — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/BehaviorBuilderTest.java:116-138
  - A4. onAnyMessage atrapa "message" — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/BehaviorBuilderTest.java:140-154
  - A5. Contador inmutable con Behaviors.receive(CounterMessage.class).onMessage(Increase...).onMessage(Get...): Get -> 0, Increase, Get -> 1 — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/BehaviorBuilderTest.java:176-201
  - A6. onMessageUnchecked con `MyList.class` generico y onSignal(Terminated.class, ...) -> stopped(): COMPILA (el metodo shouldCompile no tiene @Test) — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/BehaviorBuilderTest.java:51-74
  - A7. no afirmada por spec: inmutabilidad (ramificar un builder), onSignalEquals, onSignal con predicado, sin coincidencia -> unhandled, Terminated no manejado -> DeathPactException, MessageAdaptionFailure con BehaviorBuilder vacio NO termina el actor. Caracterizacion propuesta: pruebas Rust + escenario jshell nuevo (jar oficial 1.7.1) que contraste BehaviorBuilder vacio con ReceiveBuilder vacio ante un adaptador que lanza.
- **Validacion prevista / Oraculo**: A1-A5 portados 1:1; A7 oraculo jshell (escenario nuevo). Procedencia: jar oficial.
- **Decisiones**: Q-T-JAPI-1.

## AT-JAPI-004: javadsl.AbstractMatchingBehavior (public)

- **Contrato observable**: Contractual (comportamiento propio de la fachada Java, desde 1.3.0): `AbstractMatchingBehavior<T>(context)` es un ExtensibleBehavior de estilo OO SIN builder: el usuario implementa `onMessage(T)` (y opcionalmente `onSignal(Signal)`); `receive`/`receiveSignal` son `final` y solo hacen checkRightContext y llamar a esos metodos (sin cache ni Receive). Reglas: (1) `context == null` -> IllegalArgumentException "context must not be null.  Wrap in Behaviors.setup and pass the context to the constructor of AbstractMatchingBehavior." (dos espacios tras el punto); (2) contexto distinto del de construccion -> IllegalStateException "Actor [<self>] of AbstractMatchingBehavior class [<clase>] was created with the wrong ActorContext [<self del contexto>]. Wrap in Behaviors.setup and pass the context to the constructor of AbstractMatchingBehavior." (nota: dice "the wrong", distinto de "wrong" de AbstractBehavior); (3) `onSignal` por defecto: MessageAdaptionFailure -> `Behaviors.unhandled` (NO relanza, a diferencia de AbstractBehavior y ReceiveBuilder), cualquier otra senal -> `this`; (4) devolver `this` o `Behaviors.same()` significa "sin cambio".
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/AbstractMatchingBehavior.scala:45-90 (null 46-49; onMessage 56; onSignal por defecto 62-68; getContext 70; receive/receiveSignal 72-82; checkRightContext 84-89); ejemplo: actor-typed-tests/src/test/java/jdocs/org/apache/pekko/typed/OnMessageIntroTest.java:120-188,190-246; docs: docs/src/main/paradox/typed/actors.md:415-430; notas de version: docs/src/main/paradox/release-notes/releases-1.3.md:30; contraste: actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/AbstractBehavior.scala:47-81,87-93 y actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/AbstractBehavior.scala:78-103
- **Condiciones**: Misma disciplina de construccion que AbstractBehavior (Behaviors.setup, contexto propio). No hay API de builder: el despacho por tipo lo hace el usuario (instanceof / pattern matching de Java 16+/21).
- **Comportamiento**: Ver Contrato observable. Se evita el costo de construir y recorrer un Receive.
- **Errores**: IllegalArgumentException / IllegalStateException como arriba; las excepciones de onMessage/onSignal son fallo del actor. Ninguno afirmado por un test.
- **Orden / concurrencia / ciclo de vida**: Todo en el hilo del actor. onMessage se invoca en cada mensaje; no hay cache que inicializar.
- **Dependencias**: filas: AT-BEH-010; AT-SIG-007; AT-JAPI-002.1; AT-SUP-013. Externas: JVM: pattern matching de Java (solo en el ejemplo).
- **Equivalente Rust propuesto**: Una API Rust unica (Q-T-JAPI-1). Probablemente el `match` sobre el enum de protocolo de un Behavior orientado a trait. SE PRESERVA solo si la API Rust expone un Behavior OO: la senal por defecto que NO relanza MessageAdaptionFailure (divergencia observable respecto de AbstractBehavior) debe documentarse y decidirse (cual de los dos defaults adopta el Behavior OO de Rust: ver AT-BEH-010). SOLO ERGONOMIA JAVA: herencia, `getContext`, el texto "the wrong".
- **Aserciones**:
  - A1. Ejemplo ChatRoom con dos subclases de AbstractMatchingBehavior (ChatRoomBehavior y SessionBehavior): solo se compila y se ejecuta en `main` (`ActorSystem.create(Main.create(), "ChatRoomDemo")`); no hay @Test ni asercion — actor-typed-tests/src/test/java/jdocs/org/apache/pekko/typed/OnMessageIntroTest.java:120-188,293-311
  - A2. no afirmada por spec: IllegalArgumentException por contexto nulo; IllegalStateException con texto "the wrong ActorContext"; onSignal por defecto (MessageAdaptionFailure -> unhandled, otras -> this); `this`/`same` equivalentes; Terminated no manejado -> DeathPactException (por devolver `this`, NO se produce: `this` mantiene el comportamiento y el pacto de muerte solo salta con unhandled). Caracterizacion propuesta: escenario jshell nuevo con una subclase Java (jar oficial 1.7.1) que mida: actor sobrevive a adaptador que lanza, texto del IllegalStateException, y que un Terminated no manejado NO mata el actor porque onSignal devuelve `this`.
- **Validacion prevista / Oraculo**: Oraculo diferencial (jshell) unico modo de fijar el contrato porque Pekko no tiene prueba que lo afirme; procedencia: jar oficial 1.7.1. Hasta contar con ese escenario la validacion es solo caracterizacion propuesta.
- **Decisiones**: Q-T-JAPI-1.

## AT-JAPI-005.1: javadsl.ActorContext: espejo directo del contexto (public)

- **Contrato observable**: Contractual (fachada): `javadsl.ActorContext<T>` es la misma instancia que scaladsl.ActorContext (`asScala` y `asJava` devuelven `this`) con nombres y tipos Java. Metodos y su fila Scala: getSelf (AT-CTX-001), getSystem (ActorSystem<Void>, AT-CTX-001), getLog / setLoggerName(String|Class) (AT-LOG-001), getChildren / getChild (AT-CTX-003), spawn / spawnAnonymous con y sin Props (AT-CTX-002), stop (AT-CTX-004), watch / watchWith / unwatch (AT-CTX-005..007), setReceiveTimeout(Duration, msg) / cancelReceiveTimeout (AT-CTX-008), scheduleOnce(Duration, target, msg) (AT-CTX-009), getExecutionContext (AT-CTX-010), delegate (AT-CTX-015), asScala (AT-CTX-017). Diferencias de representacion: `Optional<ActorRef<Void>>` por `Option` (vacio = Optional.empty), `java.util.List` por `Iterable` con COPIA (ArrayList nueva por llamada, sin vista viva), `java.time.Duration` por `FiniteDuration`, hijos como `ActorRef<Void>` (unsafeUpcast), `ActorSystem<Void>` en lugar de `ActorSystem[Nothing]`.
- **Fuente**: EVIDENCIA DE DELEGACION: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorContextImpl.scala:133-158 (asJava/asScala `this`, getChild -> `child(name)` convertido a Optional 137-141, getChildren copia a ArrayList 143-149, getExecutionContext 151-152, getSelf 154-155, getSystem 157-158), 180-188 (getLog = log, setLoggerName(Class) = setLoggerName(clazz.getName)), 203-213 (Duration -> `duration.asScala`, spawn sin Props -> Props.empty), 215-226 (delegate); interfaz: actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/ActorContext.scala:49-345; Duration->FiniteDuration: actor/src/main/scala-2/org/apache/pekko/util/JavaDurationConverters.scala:29 (`Duration.fromNanos(self.toNanos)`); chequeo de hilo: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorContextImpl.scala:173-188,362-377 y actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorContextAdapter.scala:59-131; tests: actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ActorLoggingTest.java:61-81; actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/WatchTest.java:86-130; actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/AdapterTest.java:376-384
- **Condiciones**: Seguros desde otros hilos (documentado): asScala, getSelf, getSystem, scheduleOnce, getExecutionContext, pipeToSelf, ask. El resto (incluido getLog, setLoggerName, getChild(ren), spawn, stop, watch, setReceiveTimeout, messageAdapter) comprueba el hilo del actor y lanza UnsupportedOperationException ("Unsupported access to ActorContext operation from the outside of <self>. Current message is processed by <t>, but ActorContext was called from <callerThread>." o "...No message is currently processed by the actor, but ActorContext was called from <callerThread>."). Esa regla es AT-CTX-016; la fachada no la altera.
- **Comportamiento**: Idem a las filas Scala. getChildren devuelve una lista nueva (no refleja hijos posteriores). getChild devuelve vacio si el hijo no existe o ya paro.
- **Errores**: `stop(child)` con una ref que no es hijo directo lanza IllegalArgumentException (Scaladoc actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/ActorContext.scala:176); `context.stop(context.getSelf())` falla con IllegalArgumentException. watch tras watchWith con otro mensaje: IllegalStateException (AT-CTX-005/006).
- **Orden / concurrencia / ciclo de vida**: Como las filas Scala.
- **Dependencias**: filas: AT-CTX-001; AT-CTX-002; AT-CTX-003; AT-CTX-004; AT-CTX-005; AT-CTX-006; AT-CTX-007; AT-CTX-008; AT-CTX-009; AT-CTX-010; AT-CTX-015; AT-CTX-016; AT-CTX-017; AT-LOG-001. Externas: JVM: java.util.Optional, java.time.Duration, SLF4J Logger.
- **Equivalente Rust propuesto**: Una API Rust unica (Q-T-JAPI-1): `Option<ActorRef<..>>`, `Vec<ActorRef<..>>` (copia/snapshot) y `std::time::Duration`. SE PRESERVA: la semantica de las filas AT-CTX-*, incluida la regla de "snapshot" de la lista de hijos y la prohibicion de uso fuera del hilo del actor (en Rust la da `&mut Context`/`!Send`; no se necesita chequeo en runtime, hay que registrar el cambio en AT-CTX-016). SOLO ERGONOMIA JAVA: Optional, java.util.List, ActorRef<Void>, ActorSystem<Void>, sobrecargas de nombres `getXxx`.
- **Aserciones**:
  - A1. context.getLog().info(...) dentro de setup registra un evento cuyo logClass es la clase del test (clase detectada de la pila) — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ActorLoggingTest.java:61-81 (ATENCION: no afirmada en la practica: el CustomEventFilter no se instala ni se afirma con assertDone; ver AT-JAPI-001.1 A5. La afirmacion real: AT-LOG-001)
  - A2. context.spawn(exitingActor, "exitingActor"), context.watch(watched) y watchWith(watched, msg) funcionan desde el contexto Java — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/WatchTest.java:86-130
  - A3. context.stop(context.getSelf()) lanza IllegalArgumentException (se captura y se reenvia al sondeo) — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/AdapterTest.java:100-106,376-384
  - A4. no afirmada por spec (javadsl): getChild -> Optional vacio/presente, getChildren como copia, setReceiveTimeout(Duration), scheduleOnce(Duration), conversion Duration (overflow de toNanos -> ArithmeticException), delegate. Caracterizacion propuesta: se heredan las aserciones de AT-CTX-003/008/009/015; para la conversion de Duration, una prueba Rust de duraciones limite y escenario jshell de `Duration.ofSeconds(Long.MAX_VALUE)`.
- **Validacion prevista / Oraculo**: Las aserciones de AT-CTX-*; A4 de Duration por jshell opcional. Procedencia: jar oficial.
- **Decisiones**: Q-T-JAPI-1.

## AT-JAPI-005.2: javadsl.ActorContext.ask / askWithStatus (convencion (respuesta, excepcion)) (public)

- **Contrato observable**: Contractual (comportamiento propio de la fachada Java): `context.ask(resClass, target, timeout, createRequest, applyToResponse)` envia una peticion y entrega al propio actor, como mensaje, el resultado de `applyToResponse(res, failure)` con la convencion Java de DOS argumentos de los cuales exactamente uno es no nulo: (res, null) si hubo respuesta, (null, throwable) si hubo error o timeout (java.util.concurrent.TimeoutException tras `responseTimeout`, scaladsl/AskPattern.scala:141; un Status.Failure clasico llega como su excepcion, AT-ASK-003). `resClass` es un parametro "de adorno" (no se usa para filtrar). `askWithStatus` espera respuestas `StatusReply<Res>`: Success(v) -> applyToResponse(v, null); Error(why) -> applyToResponse(null, why) (why = ErrorMessage o la excepcion personalizada); fallo/timeout del ask -> applyToResponse(null, failure). Si applyToResponse lanza, es como si la recepcion normal lanzara (fallo del actor / supervision).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorContextImpl.scala:243-276 (ask Java usa `javadsl.AskPattern.ask(target, ref => createRequest(ref), responseTimeout, system.scheduler)` y `pipeToSelf`; askWithStatus Java reusa ask con `classOf[StatusReply]` y mapea Success/Error/null 269-275); javadsl.AskPattern: actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/AskPattern.scala:47-65; interfaz: actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/ActorContext.scala:288-332; Scala (contraste): actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorContextImpl.scala:227-241; tests: actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ActorContextAskTest.java:48-171
- **Condiciones**: El timeout lo cuenta el ask interno (dentro de AskPattern, no de un reloj del contexto). La respuesta llega como mensaje interno AdaptMessage procesado en el hilo del actor (AT-CTX-014). Un valor de respuesta nulo en Success daria (null, null): caso limite no cubierto.
- **Comportamiento**: Ver Contrato observable. Es la unica via segura de pedir cosas a otros actores desde un actor Java.
- **Errores**: TimeoutException en el segundo argumento si no hay respuesta en `responseTimeout`; excepciones de StatusReply.error; un fallo en applyToResponse -> MessageAdaptionFailure (AT-SIG-007).
- **Orden / concurrencia / ciclo de vida**: Resultado procesado en el hilo del actor, ordenado con los demas mensajes; no se cancela al parar el actor (respuestas posteriores a PostStop se descartan).
- **Dependencias**: filas: AT-CTX-012; AT-CTX-013; AT-CTX-014; AT-ASK-001; AT-ASK-002; AT-SIG-007; AT-JAPI-006.4. Externas: JVM: japi.function.Function / Function2.
- **Equivalente Rust propuesto**: Una API Rust unica (Q-T-JAPI-1): `ctx.ask(target, |reply_to| req, timeout, |result: Result<Res, AskError>| -> T)`. SE PRESERVA: entrega del resultado como mensaje propio en el hilo del actor, timeout, mapeo de StatusReply::Success/Error, tratamiento de fallos del adaptador como MessageAdaptionFailure. SOLO ERGONOMIA JAVA: la convencion "exactamente uno nulo" (en Rust `Result`), `resClass` de adorno, excepciones comprobadas.
- **Aserciones**:
  - A1. ask a un actor Ping/Pong: applyToResponse(pong, ex) devuelve el pong; el actor recibe y reenvia un `Pong` al sondeo — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ActorContextAskTest.java:48-83
  - A2. askWithStatus: tras `StatusReply.success(new Pong())` applyToResponse recibe (pong, null) y el actor procesa el Pong ("got pong") — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ActorContextAskTest.java:93-123
  - A3. askWithStatus con `StatusReply.error("boho")`: applyToResponse recibe (null, ErrorMessage) y el actor ve "got error: org.apache.pekko.pattern.StatusReply$ErrorMessage, boho" — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ActorContextAskTest.java:125-160
  - A4. askWithStatus con `StatusReply.error(new TestException("boho"))`: la excepcion personalizada llega tal cual ("got error: org.apache.pekko.actor.testkit.typed.TestException, boho") — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ActorContextAskTest.java:162-171
  - A5. no afirmada por spec (javadsl): timeout (null, TimeoutException), Status.Failure clasico, Success con valor nulo, y excepcion en applyToResponse. Caracterizacion propuesta: se heredan AT-CTX-012/013 (Scala) y AT-ASK-003; un escenario jshell para el timeout con la convencion (null, ex).
- **Validacion prevista / Oraculo**: A1-A4 portados como pruebas de integracion; A5 por oraculo (jshell, jar oficial 1.7.1) para el timeout. Procedencia: jar oficial.
- **Decisiones**: Q-T-JAPI-1.

## AT-JAPI-005.3: javadsl.ActorContext.pipeToSelf(CompletionStage) (public)

- **Contrato observable**: Contractual (comportamiento propio de la fachada Java): `context.pipeToSelf(future, applyToResult)` registra un `future.handle` que, al completar, envia al propio actor un AdaptMessage con el resultado de `applyToResult(value, ex)` (convencion (valor, null) o (null, excepcion)); la funcion se ejecuta DENTRO del actor (hilo del despachador del actor), no en el hilo que completo el CompletionStage. Si el stage completa con excepcion, se pasa esa excepcion tal como la entrega CompletableFuture.handle (sin envoltura para un stage completado directamente; puede venir envuelta en CompletionException si deriva de otro stage: no afirmado). Si applyToResult devuelve `null` el mensaje se IGNORA con un warning "Adapter function returned null which is not valid as an actor message, ignoring..." y la funcion SI se ejecuta; si lanza, es MessageAdaptionFailure.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorContextImpl.scala:283-292 (Java) y 278-281 (Scala); manejo de nulos: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:198-218; interfaz: actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/ActorContext.scala:334-343; tests: actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ActorContextPipeToSelfTest.java:59-157
- **Condiciones**: Seguro desde otros hilos (no comprueba el hilo). `future.handle[Unit]` registra el callback inmediatamente; el envio a self puede ocurrir tras parar el actor (se pierde en dead letters).
- **Comportamiento**: Ver Contrato observable.
- **Errores**: Warning para resultado nulo; MessageAdaptionFailure si la funcion lanza.
- **Orden / concurrencia / ciclo de vida**: El mensaje se encola en el buzon como cualquier otro; el orden relativo respecto de otros mensajes depende del momento de completar el stage.
- **Dependencias**: filas: AT-CTX-014; AT-SIG-007; AT-JAPI-005.2. Externas: JVM: CompletionStage/CompletableFuture.
- **Equivalente Rust propuesto**: Una API Rust unica (Q-T-JAPI-1): `ctx.pipe_to_self(future, |result| -> Option<T>)`. SE PRESERVA: ejecucion de la funcion en el actor, descarte con warning del resultado nulo (en Rust: `None`), fallos del adaptador como MessageAdaptionFailure. SOLO ERGONOMIA JAVA: CompletionStage, la convencion de dos argumentos, CompletionException.
- **Aserciones**:
  - A1. future completado con "hi": applyToResult(string, null) -> "ok: hi" — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ActorContextPipeToSelfTest.java:59-62,122-157
  - A2. future fallido con RuntimeException("boom"): applyToResult(null, ex) -> "ko: boom" (ex es la misma excepcion, getMessage() == "boom") — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ActorContextPipeToSelfTest.java:64-67
  - A3. la funcion se ejecuta en el actor: selfName == nombre del actor y el hilo comienza con "ActorContextPipeToSelfTest-pipe-to-self-spec-dispatcher" (despachador PinnedDispatcher del actor) — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ActorContextPipeToSelfTest.java:146-155
  - A4. future completado con null y funcion que devuelve null: la funcion se invoca ("adapting"), el mensaje NO llega al actor (expectNoMessage 200 ms) y se registra el warning "Adapter function returned null which is not valid as an actor message, ignoring" — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ActorContextPipeToSelfTest.java:69-114
  - A5. no afirmada por spec: excepcion envuelta en CompletionException para stages derivados; funcion que lanza (MessageAdaptionFailure). Caracterizacion propuesta: escenario jshell con thenApply sobre un stage fallido.
- **Validacion prevista / Oraculo**: A1-A4 portados 1:1 (con despachador fijo para A3); A5 oraculo jshell. Procedencia: jar oficial.
- **Decisiones**: Q-T-JAPI-1.

## AT-JAPI-005.4: javadsl.ActorContext.messageAdapter(Class, Function) (public)

- **Contrato observable**: Contractual (comportamiento propio de la fachada Java): `messageAdapter(messageClass, f)` registra un adaptador por CLASE de mensaje (en Scala por `ClassTag`) y devuelve siempre el MISMO `ActorRef<U>` (un FunctionRef por actor, nombre "adapter"). La clase se "boxea" (`Integer.class` y `int.class` son la misma entrada). Solo un adaptador por clase: registrar otro para la misma clase lo reemplaza. Se prueban los adaptadores en orden inverso de registro (el ultimo registrado primero) y se usa el primero cuya clase sea asignable al mensaje. La funcion `f` es `japi.function.Function` (puede lanzar excepciones comprobadas) y se ejecuta en el actor.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorContextImpl.scala:306-336 (Scala 306-309, Java 311-312, `internalMessageAdapter` 314-331 con `BoxedType` y reemplazo de la misma clase); seleccion: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:181-194; interfaz: actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/ActorContext.scala:260-286; tests: actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ActorCompile.java:119-134,162-169,208-215
- **Condiciones**: Comprueba el hilo del actor (ActorContextImpl.scala:315). Recomendado registrar en setup / constructor.
- **Comportamiento**: Ver Contrato observable. Un mensaje cuya clase no tiene adaptador -> unhandled (AT-CTX-011).
- **Errores**: Si f lanza (NonFatal) -> MessageAdaptionFailure (AT-SIG-007).
- **Orden / concurrencia / ciclo de vida**: Mismo ciclo de vida que el actor; no hace falta pararlo.
- **Dependencias**: filas: AT-CTX-011; AT-SIG-007; AT-JAPI-005.1. Externas: JVM: Class, BoxedType.
- **Equivalente Rust propuesto**: Una API Rust unica (Q-T-JAPI-1): adaptadores por `TypeId` del tipo de mensaje. SE PRESERVA: uno por tipo, reemplazo, mismo ref devuelto, orden inverso de prueba por asignabilidad (en Rust solo igualdad de tipo, salvo trait objects: registrar la divergencia en AT-CTX-011). SOLO ERGONOMIA JAVA: boxing de primitivos, `Class<U>`.
- **Aserciones**:
  - A1. messageAdapter(String.class, s -> new MyMsgB(...)) y messageAdapter(Integer.class, n -> ...) con handler que lanza Exception comprobada COMPILAN y devuelven ActorRef — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ActorCompile.java:119-134,162-169,208-215 (solo compila)
  - A2. no afirmada por spec (javadsl): boxing Integer/int, reemplazo, mismo ref. Caracterizacion propuesta: se heredan las aserciones de AT-CTX-011 (Scala, MessageAdapterSpec); un caso extra con `Integer.class` en jshell.
- **Validacion prevista / Oraculo**: Se hereda AT-CTX-011; boxing por jshell. Procedencia: jar oficial.
- **Decisiones**: Q-T-JAPI-1.

## AT-JAPI-006.1: javadsl.TimerScheduler: Duration Java y startPeriodicTimer (public)

- **Contrato observable**: Contractual (fachada con una diferencia propia): las sobrecargas con `java.time.Duration` convierten con `duration.asScala` (`Duration.fromNanos(toNanos)`) y llaman a la API Scala (AT-TMR-002..005). Diferencia propia: `startPeriodicTimer(key, msg, interval: Duration)` (DEPRECATED desde Akka 2.6.0) se implementa como `startTimerWithFixedDelay`, mientras que la API Scala `startPeriodicTimer(FiniteDuration)` se implementa como FIXED RATE (AT-TMR-005). Sobrecargas sin clave (`startTimerWithFixedDelay(msg, delay)` etc.): la clave es el propio mensaje (default en el trait javadsl, igual que en scaladsl). `isTimerActive`, `cancel`, `cancelAll` y la garantia de no entrega del timer reemplazado/cancelado son los de AT-TMR-006/007.1.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/TimerSchedulerImpl.scala:64-88 (TimerSchedulerCrossDslSupport: conversion Duration y startPeriodicTimer Java -> WithFixedDelay, 81-84) y 101-117 (Scala: startPeriodicTimer -> FixedRateMode 113-114); interfaz: actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/TimerScheduler.scala:32-275 (overloads con clave por defecto 87-88,107-108,190-191,219-220,252-253); Duration: actor/src/main/scala-2/org/apache/pekko/util/JavaDurationConverters.scala:29; tests: actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ActorCompile.java:136-152
- **Condiciones**: No thread-safe: solo desde el actor propietario (AT-TMR-001). Duraciones fuera del rango de `toNanos` (~292 anos) -> ArithmeticException de Duration.toNanos al convertir.
- **Comportamiento**: Ver Contrato observable.
- **Errores**: ArithmeticException por overflow de Duration; resto como AT-TMR-*.
- **Orden / concurrencia / ciclo de vida**: Como AT-TMR-001/006.
- **Dependencias**: filas: AT-TMR-001; AT-TMR-002; AT-TMR-003; AT-TMR-004; AT-TMR-005; AT-TMR-006; AT-TMR-007.1. Externas: JVM: java.time.Duration.
- **Equivalente Rust propuesto**: Una API Rust unica (Q-T-JAPI-1) con `std::time::Duration` (sin overflow a 292 anos). SE PRESERVA: la semantica de AT-TMR-002..007.1; la divergencia Java=FixedDelay / Scala=FixedRate de `startPeriodicTimer` NO se porta si el metodo deprecado se excluye (decision de AT-TMR-005, Q-T-TMR-3); solo si se porta el deprecado hay que elegir un comportamiento y registrarlo. SOLO ERGONOMIA JAVA: `Duration`, overloads por clave.
- **Aserciones**:
  - A1. withTimers(timers -> timers.startTimerWithFixedDelay("key", msg, Duration.ofSeconds(1)) y la variante sin clave COMPILAN — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ActorCompile.java:136-152 (solo compila)
  - A2. no afirmada por spec (javadsl): conversion de Duration, startPeriodicTimer Java = fixed delay, overflow. Caracterizacion propuesta: oraculo jshell que mida los instantes de entrega de startPeriodicTimer Java con un mensaje que tarda en procesarse (para distinguir delay de rate); las demas aserciones de timers se heredan de AT-TMR-* (ManualTime).
- **Validacion prevista / Oraculo**: Escenario jshell para A2 (jar oficial 1.7.1); el resto heredado. Procedencia: jar oficial.
- **Decisiones**: Q-T-JAPI-1; Q-T-TMR-3 (startPeriodicTimer).

## AT-JAPI-006.2: javadsl.StashBuffer (fachada sobre StashBufferImpl) (public)

- **Contrato observable**: Contractual (fachada): `javadsl.StashBuffer<T>` y `scaladsl.StashBuffer[T]` son DOS vistas del MISMO objeto `StashBufferImpl` (implementa ambos traits), con tipos Java (Procedure para forEach, Predicate para anyMatch, `java.util.function.Function` para wrap, StashOverflowException Java que extiende la Scala). Semantica = AT-STASH-001..006: capacidad fija, stash lanza StashOverflowException si esta lleno, head lanza NoSuchElementException si esta vacio, forEach en orden de llegada, contains por equals, anyMatch con predicado, unstashAll/unstash(n, wrap) con las reglas de same/unhandled/stopped.
- **Fuente**: EVIDENCIA DE DELEGACION: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/StashBufferImpl.scala:54-55 (implementa ambos), 71-76 (StashOverflowException Java y texto), 127-129 (head), 139 (`forEach(f: Procedure[T]) = foreach(f.apply)`), 141-142 (contains = exists(_ == message)), 154 (anyMatch = exists(predicate.test)), 156-160 (unstashAll), 254-255 (`unstash(behavior, n, wrap: JFunction)` = Scala con `x => wrap.apply(x)`); interfaz: actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/StashBuffer.scala:31-160; fabrica: actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/Behaviors.scala:57-60; tests: actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/StashBufferTest.java:39-68
- **Condiciones**: Misma instancia no thread-safe (AT-STASH-001). El test construye el buffer con `StashBufferImpl.apply(context, 10)` (API interna) sobre un StubbedActorContext.
- **Comportamiento**: Idem AT-STASH-*. Un `message == null` en stash lanza NullPointerException (StashBufferImpl.scala:72).
- **Errores**: StashOverflowException (javadsl, texto "Couldn't add [<clase>] because stash with capacity [N] is full"), NoSuchElementException ("head of empty buffer"), IllegalArgumentException "Cannot unstash with unhandled as starting behavior", NullPointerException.
- **Orden / concurrencia / ciclo de vida**: Como AT-STASH-002/003/006.
- **Dependencias**: filas: AT-STASH-001; AT-STASH-002; AT-STASH-003; AT-STASH-004; AT-STASH-005; AT-STASH-006. Externas: JVM: Procedure, Predicate, Function.
- **Equivalente Rust propuesto**: Una API Rust unica (Q-T-JAPI-1): un solo `StashBuffer<T>`. SE PRESERVA toda la semantica de AT-STASH-*. SOLO ERGONOMIA JAVA: dos traits, Procedure/Predicate/Function, la excepcion Java que hereda de la Scala (en Rust un unico tipo de error).
- **Aserciones**:
  - A1. stash("m1","m2","m3"): forEach recorre en orden ("m1m2m3"); unstash(Behaviors.ignore(), 1, identity) deja "m2m3" — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/StashBufferTest.java:39-55
  - A2. anyMatch(m -> m.startsWith("m")) y anyMatch(m -> m.endsWith("2")) true; contains("m1"), contains("m2") true — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/StashBufferTest.java:57-68
  - A3. withStash(5, stash -> { stash.forEach(msg -> { throw new Exception("checked"); }) ... }) COMPILA (excepcion comprobada permitida en Procedure) — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ActorCompile.java:221-235 (solo compila)
  - A4. no afirmada por spec (javadsl): desbordamiento, head vacio, unstashAll, wrap. Caracterizacion propuesta: heredar las de AT-STASH-001..006 (StashBufferSpec/ScalaDSL).
- **Validacion prevista / Oraculo**: A1/A2 portados; el resto heredado de AT-STASH-*. Procedencia: jar oficial.
- **Decisiones**: Q-T-JAPI-1.

## AT-JAPI-006.3: javadsl.Routers (fachada sobre los builders de router) (public)

- **Contrato observable**: Contractual (fachada): `Routers.group(key)` y `Routers.pool(poolSize, behavior)` (en Scala `pool(poolSize)(behavior)`, curried; desde Java son dos argumentos) devuelven `GroupRouter<T>` / `PoolRouter<T>`, que son los mismos `GroupRouterBuilder` / `PoolRouterBuilder` que implementan tambien el DSL Scala. Los metodos Java son sobrecargas con `java.util.function.Function` / `Predicate` y sin `ClassTag`: withRandomRouting(), withRandomRouting(preferLocalRoutees), withRoundRobinRouting(...), withConsistentHashingRouting(virtualNodesFactor, mapping), withPoolSize, withRouteeProps, withBroadcastPredicate(Predicate). Semantica = AT-RT-001..006.
- **Fuente**: EVIDENCIA DE DELEGACION: actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/Routers.scala:39-52 (group/pool crean los builders), 62-133 (GroupRouter abstracta), 141-203 (PoolRouter abstracta); implementacion compartida: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/PoolRouterImpl.scala:36 (extends javadsl.PoolRouter), 54-70 (consistent hashing y broadcast: Predicate -> `predicate.test`), actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/GroupRouterImpl.scala:37,44-56; tests: actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/RoutersTest.java:40-76
- **Condiciones**: Las de AT-RT-006 (validacion e inmutabilidad de los builders; cada with* devuelve un builder nuevo).
- **Comportamiento**: Idem AT-RT-001..006.
- **Errores**: Los de AT-RT-006.
- **Orden / concurrencia / ciclo de vida**: Como AT-RT-*.
- **Dependencias**: filas: AT-RT-001; AT-RT-002.1; AT-RT-002.2; AT-RT-002.3; AT-RT-003; AT-RT-004; AT-RT-005; AT-RT-006. Externas: JVM: java.util.function.
- **Equivalente Rust propuesto**: Una API Rust unica (Q-T-JAPI-1). SE PRESERVA AT-RT-*. SOLO ERGONOMIA JAVA: sobrecargas, Predicate/Function, curry de pool.
- **Aserciones**:
  - A1. Routers.pool(4, behavior).withBroadcastPredicate(str -> str.startsWith("bc-")): un mensaje "not-bc-message" llega una vez y "bc-message" llega 4 veces (receiveSeveralMessages(5), conteo por igualdad de referencia `==`) — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/RoutersTest.java:49-76
  - A2. Routers.group(key).withRandomRouting().withRoundRobinRouting() y Routers.pool(5, empty).withRandomRouting().withRoundRobinRouting() compilan (metodo sin @Test) — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/RoutersTest.java:40-47
  - A3. no afirmada por spec (javadsl): el resto de AT-RT-*. Caracterizacion propuesta: heredar las aserciones Scala de AT-RT-*.
- **Validacion prevista / Oraculo**: A1 portado; resto heredado. Procedencia: jar oficial.
- **Decisiones**: Q-T-JAPI-1.

## AT-JAPI-006.4: javadsl.AskPattern (ask/askWithStatus fuera de un actor) (public)

- **Contrato observable**: Contractual (fachada): `AskPattern.ask(actor, messageFactory, timeout: Duration, scheduler)` y `askWithStatus(...)` convierten el timeout (`timeout.asScala`), llaman a `ask` / `askWithStatus` de scaladsl.AskPattern y convierten el Future a `CompletionStage` (`.asJava`). Semantica = AT-ASK-001..003.
- **Fuente**: EVIDENCIA DE DELEGACION: actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/AskPattern.scala:47-52 (ask), 60-65 (askWithStatus); Scala: actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/AskPattern.scala:78-79,130-137; tests: actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/WatchTest.java:86-130; actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ActorCompile.java:101-116
- **Condiciones**: Las de AT-ASK-001/004 (solo sobre InternalRecipientRef).
- **Comportamiento**: Idem AT-ASK-*; el CompletionStage se completa con la respuesta o con TimeoutException (AskTimeoutException).
- **Errores**: Los de AT-ASK-*.
- **Orden / concurrencia / ciclo de vida**: Como AT-ASK-*.
- **Dependencias**: filas: AT-ASK-001; AT-ASK-002; AT-ASK-003; AT-ASK-004. Externas: JVM: CompletionStage, java.time.Duration.
- **Equivalente Rust propuesto**: Una API Rust unica (Q-T-JAPI-1): `ask(...) -> impl Future`. SE PRESERVA AT-ASK-*. SOLO ERGONOMIA JAVA: CompletionStage y Duration.
- **Aserciones**:
  - A1. AskPattern.ask(exitingRef, RunTest::new, timeout, testKit.scheduler()) devuelve un CompletionStage que completa cuando el actor responde Done (get(3 s)) — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/WatchTest.java:103-105,127-129
  - A2. AskPattern.ask(...).thenApply(rsp -> rsp.toUpperCase()) COMPILA con y sin tipo explicito de replyTo — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ActorCompile.java:101-116 (solo compila)
  - A3. no afirmada por spec (javadsl): askWithStatus Java, timeout, fallo por Status.Failure. Caracterizacion propuesta: heredar AT-ASK-002/003.
- **Validacion prevista / Oraculo**: Heredada de AT-ASK-*. Procedencia: jar oficial.
- **Decisiones**: Q-T-JAPI-1.

## AT-JAPI-006.5: javadsl.Adapter (equivalente a scaladsl.adapter para Java) (public)

- **Contrato observable**: Contractual (fachada): `javadsl.Adapter` expone como metodos estaticos las extensiones de `scaladsl.adapter` (importa `scaladsl.adapter._` y delega): spawnAnonymous/spawn desde ActorSystem clasico y desde ActorContext clasico (con y sin Props), toTyped/toClassic de sistema, contexto, ActorRef y Scheduler, watch/unwatch/stop entre contexto clasico y ref typed y viceversa, actorOf desde contexto typed, y `props(Creator<Behavior<T>>[, Props])` (PropsAdapter con supervision Stop y `rethrowTypedFailure = false`). Semantica = AT-ADP-001..005.
- **Fuente**: EVIDENCIA DE DELEGACION: actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/Adapter.scala:27,51-60,67-76,83-92,99-108 (spawn* -> `sys.spawn`/`ctx.spawn` de scaladsl.adapter), 110-117 (toTyped/toClassic de sistema y contexto), 119-135 (watch/unwatch/stop), 137-141 (actorOf), 143-147 (toClassic/toTyped de ref), 158-176 (props -> internal PropsAdapter con `Behaviors.supervise(behavior.create()).onFailure(SupervisorStrategy.stop)`), 178-182 (Scheduler); Scala: actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/adapter/package.scala:49-98,103-137,142-152,157-195 y actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/adapter/PropsAdapter.scala:31-36; tests: actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/AdapterTest.java:265-392
- **Condiciones**: Las de AT-ADP-*. `toTyped(sys)` devuelve `ActorSystem<Void>` (cast). `toTyped(ref)` usa la conversion implicita ActorRefAdapter.
- **Comportamiento**: Idem AT-ADP-*.
- **Errores**: Los de AT-ADP-* (p.ej. IllegalArgumentException al hacer stop de un actor que no es hijo).
- **Orden / concurrencia / ciclo de vida**: Como AT-ADP-*.
- **Dependencias**: filas: AT-ADP-001; AT-ADP-002; AT-ADP-003; AT-ADP-004; AT-ADP-005; AT-SUP-020; AT-SUP-021. Externas: JVM: japi.Creator.
- **Equivalente Rust propuesto**: Una API Rust unica (Q-T-JAPI-1) sujeta a Q-X-8 (interop clasico). SE PRESERVA AT-ADP-* si hay interop clasica; si Q-X-8 elige "sin sistema clasico" esta fila no tiene equivalente. SOLO ERGONOMIA JAVA: metodos estaticos, Creator.
- **Aserciones**:
  - A1. typed->clasico: Adapter.spawnAnonymous(system, Typed1.create(...)); "send" hace que el clasico responda "pong" y el typed notifique "ok" — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/AdapterTest.java:265-273
  - A2. clasico->typed: Adapter.toTyped(getSelf()) como replyTo — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/AdapterTest.java:275-282
  - A3. spawn de hijo typed desde un padre clasico (Adapter.spawnAnonymous(getContext(), typed2())) y via Props (Adapter.props, getContext().actorOf) — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/AdapterTest.java:284-300
  - A4. actorOf de hijo clasico desde contexto typed (Adapter.actorOf(context, classic1())) — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/AdapterTest.java:302-310
  - A5. watch cruzado: typed vigila clasico (PoisonPill -> "terminated") y clasico vigila typed (StopIt -> "terminated") — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/AdapterTest.java:312-335
  - A6. padre typed supervisa hijo clasico con Restart por defecto: tras ThrowIt3 el hijo sigue respondiendo ("ok") y no se observa "terminated" (expectNoMessage 100 ms) — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/AdapterTest.java:337-355
  - A7. stop de hijos cruzados (Adapter.stop) y context.stop(self) -> IllegalArgumentException — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/AdapterTest.java:357-384
  - A8. Adapter.toTyped(system.scheduler()) devuelve un SchedulerAdapter y Adapter.toClassic(typedScheduler) devuelve el mismo scheduler clasico (assertSame) — actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/AdapterTest.java:386-392
  - A9. no afirmada por spec (javadsl): las ramas supervise-stop y supervise-resume de Classic2 (testSupervice con ThrowIt1/ThrowIt2, 185-186,198-203) estan definidas en la clase pero ningun @Test las ejecuta; Adapter.props con Creator. Caracterizacion propuesta: se heredan AT-SUP-020/021 y AT-ADP-003.
- **Validacion prevista / Oraculo**: A1-A8 coinciden con AdapterSpec (Scala) citadas en AT-ADP-*; se portan una sola vez. Procedencia: jar oficial.
- **Decisiones**: Q-T-JAPI-1; Q-X-8.

## AT-TK-001.1: ActorTestKit: construccion, nombre del sistema y configuracion (public)

- **Contrato observable**: Contractual: `ActorTestKit()` crea un ActorSystem de test con guardian propio (`ActorTestKitGuardian.testKitGuardian`); el nombre sale de la clase de test (`testNameFromCallStack`) o del nombre dado normalizado (`scrubActorSystemName`: caracteres invalidos -> `_`, nombre de paquete eliminado: `scala.collection.immutable.Vector` -> `Vector`); por defecto carga `application-test.conf` (`ConfigFactory.load("application-test")`) y NO `application.conf`; con `Config` explicita NO se carga `application-test`; `ActorTestKit(system)` agrega un guardian de testkit `test`, `test-2`, ... bajo `systemActorOf` (contador global); `settings` opcional (si no, `TestKitSettings(system)`).
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKit.scala:48-180,194-218; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestKitUtils.scala:80-91; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestKitUtils.scala:31-71 (guardian)
- **Precondiciones**: caracteres validos de nombre: letras, digitos, `-`, `_` (el test conserva todos). Con `Config` explicita: `hasPath("test.from-application-test") == false`.
- **Comportamiento**: (1) `apply()` = `ActorSystem(guardian, testNameFromCallStack(classOf[ActorTestKit]), ApplicationTestConfig)`; (2) `apply(name)` usa `scrubActorSystemName(name)` + `ApplicationTestConfig`; (3) `apply(Config)` / `apply(name, Config[, settings])` usan la config tal cual (sin `application-test`); (4) `apply(system)` no crea sistema: lo envuelve. `name = internalSystem.name`, `config = internalSystem.settings.config`; el constructor toca SLF4J con `debug("Starting ActorTestKit")` solo para evitar ruido (#28673). `testKitSettings` implicito (lazy por llamada) y `timeout = testKitSettings.DefaultTimeout`.
- **Errores**: nombre de sistema invalido tras normalizar -> el error es el del ActorSystem (fuera de este contrato, AT-SYS-001). Nota: la regex `TestKitRegex` de `TestKitUtils` usa el paquete `org.apache.pekko.testkit.typed...` (no `...actor.testkit.typed...`); el filtrado de marcos de pila depende ademas de `classToStartFrom`, por eso el nombre sale correcto (actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestKitUtils.scala:80).
- **Orden / concurrencia / ciclo de vida**: n/a; el contador del guardian es `AtomicInteger` global del proceso (los nombres `test`, `test-2` no se reinician entre sistemas).
- **Dependencias**: AT-SYS-001; AT-SYS-008 (`systemActorOf`); AC-CFG-001; AT-TK-010. Externas: `pekko.testkit.TestKitUtils` clasico (`testNameFromCallStack`, `scrubActorSystemName`, fuera de estos modulos: AC-TK).
- **Equivalente Rust propuesto**: `ActorTestKit::new()` / `with_name(&str)` / `with_config(Config)`; nombre por defecto desde `module_path!()`/`type_name` del test o nombre explicito (no hay pila de llamadas fiable). Se preserva: normalizacion de nombre, no cargar `application.conf` en el modo por defecto, carga de `application-test.conf` como fuente por defecto, y los nombres `test`, `test-N`. Diferencia: nombre por stack (JVM) sustituido (Q-T-TK-3).
- **Aserciones**:
  - A1. ScalaTestWithActorTestKit usa el nombre de la clase de test (`system.name === "ActorTestKitSpec"`) — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitSpec.scala:36-38
  - A2. `ActorTestKit()` usa el nombre de la clase de test — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitSpec.scala:50-55
  - A3. `ActorTestKit(classOf[Vector[_]].getName)` -> `"Vector"` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitSpec.scala:57-63
  - A4. nombre con caracteres invalidos -> `"actor__sys-tem_Name_1__with_invalid_ch_racter___"` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitSpec.scala:65-71
  - A5. nombre valido se conserva — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitSpec.scala:73-80
  - A6. `ActorTestKit(system)` expone el mismo sistema (`internalSystem` y `system`) — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitSpec.scala:40-48
  - A7. `application-test.conf` se carga por defecto (`test.from-application-test = "yes"`, sin `test.from-application`) — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitSpec.scala:109-112
  - A8. con Config explicita no se carga `application-test.conf` (tambien via `ScalaTestWithActorTestKit("...")`) — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitSpec.scala:114-124
  - A9. los nombres de probes tipadas no chocan con los de `pekko.testkit.TestProbe` clasico (no lanza) — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitSpec.scala:126-131
  - A10. clases derivadas y composicion conservan el nombre de la clase concreta (`MyConcreteDerivateSpec`, `CompositionSpec`) — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitSpec.scala:161-170,189-191
  - A11. espejo Java: nombre por recurso JUnit, `ActorTestKit.create()` y `create(HashMap.class.getName())` -> `"HashMap"` — actor-testkit-typed/src/test/java/org/apache/pekko/actor/testkit/typed/javadsl/ActorTestKitTest.java:36-60
  - A12. nombres `test`/`test-N` del guardian de `ActorTestKit(system)`: *no afirmada por spec*. Caracterizacion propuesta: dos `ActorTestKit(system)` sobre el mismo sistema crean guardianes `test` y `test-2` (leyendo actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKit.scala:77-85).
- **Oraculo / Validacion prevista**: test portado 1:1 de A1-A11 (sin JVM). Diferencial con jar oficial `pekko-actor-testkit-typed_2.13` 1.7.1 (a agregar a tools/oracle/fetch-jars.sh con su SHA-1; hoy ausente y el entorno tiene Java 11, el oraculo exige Java 21) solo para `scrubActorSystemName` y A12.
- **Decisiones**: Q-T-TK-3 (nombre por stack; integraciones de framework).

## AT-TK-001.2: ActorTestKit: spawn, stop y falla de spawn (public)

- **Contrato observable**: Contractual: `spawn(behavior[, name][, props])` pide al guardian del testkit (ask con `timeout = DefaultTimeout` dilatado, `Await.result` bloqueante) crear el hijo y devuelve su ref; `stop(ref, max = timeout)` hace `watchWith(ref, ActorStopped(reply))` + `context.stop(ref)` en el guardian y espera `Ack`; el nombre queda libre al retornar. Si el spawn falla (p.ej. dispatcher desconocido) el guardian captura `NonFatal`, loguea ERROR `Spawn failed, props [<props>]` con la excepcion y responde con un ref a un actor anonimo ya detenido (`Behaviors.stopped`).
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKit.scala:231-272; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestKitUtils.scala:31-71
- **Precondiciones**: `stop` solo detiene actores creados por ese testkit (hijos del guardian); otros actores no se detienen (comentario de contrato, actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKit.scala:259-263).
- **Comportamiento**: `SpawnActor(name, behavior, replyTo, props)` / `SpawnActorAnonymous`: `reply ! context.spawn(...)`/`spawnAnonymous(...)`, `Behaviors.same`. `StopActor(ref, reply)`: `context.watchWith(ref, ActorStopped(reply)); context.stop(ref)`; `ActorStopped(reply)` -> `reply ! Ack`. Nombre duplicado en `spawn(name)` -> el spawn del contexto lanza (InvalidActorNameException) y se maneja como "Spawn failed" (ref detenido), no como excepcion en el llamador.
- **Errores**: (1) `stop` agota `max` -> `AssertionError` con mensaje `assertion failed: timeout (<max>) during stop() waiting for actor [<path>] to stop` (se construye con `Predef.assert(false, ...)`, que antepone `assertion failed: `); (2) `spawn` que excede `timeout` -> `AskTimeoutException` (TimeoutException) propagada por `Await.result`; (3) fallo de spawn -> sin excepcion en el llamador, solo log ERROR + ref detenido.
- **Orden / concurrencia / ciclo de vida**: `spawn` y `stop` bloquean al llamador hasta la respuesta del guardian; el guardian procesa un comando a la vez.
- **Dependencias**: AT-CTX-002; AT-CTX-006 (`watchWith`); AT-ASK-001; AT-TK-010 (timeouts). Externas: ninguna.
- **Equivalente Rust propuesto**: `spawn`/`stop` sobre un guardian de testkit; async o bloqueante (Q-T-TK-1). Se preserva: el spawn pasa por un guardian (los hijos cuelgan de el), `stop` espera la terminacion real (el nombre se puede reutilizar inmediatamente), el fallo de spawn devuelve un ref ya terminado y loguea ERROR `Spawn failed, props [..]`, y el texto de timeout de `stop`. Decision a fijar: conservar o no el prefijo `assertion failed: ` en los mensajes que Pekko produce con `Predef.assert` (ver Q-T-TK-1).
- **Aserciones**:
  - A1. `spawn` de comportamiento anonimo ejecuta el `setup` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitSpec.scala:82-90
  - A2. `spawn` con nombre -> `context.self.path.name == "name"` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitSpec.scala:92-100
  - A3. dispatcher desconocido -> se registra un ERROR con "Spawn failed" y el ref queda terminado (`probe.expectTerminated(ref)`) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/DispatcherSelectorSpec.scala:87-93
  - A4. `stop` espera la terminacion: se puede re-crear un actor con el mismo nombre inmediatamente (`stop(pinger1)` con timeout por defecto y luego `spawn(Echo(), "pinger")`; `stop(pinger2, 10.seconds)`) — actor-testkit-typed/src/test/scala/docs/org/apache/pekko/actor/testkit/typed/scaladsl/AsyncTestingExampleSpec.scala:103-118
  - A5. espejo Java de A4 y spawn con/sin nombre — actor-testkit-typed/src/test/java/jdocs/org/apache/pekko/actor/testkit/typed/javadsl/AsyncTestingExampleTest.java:136-172
  - A6. timeout de `stop` -> `AssertionError`: *no afirmada por spec*. Caracterizacion propuesta: `stop(ref_que_ignora_Stop, 50ms)` falla con mensaje que contiene `timeout (50 milliseconds) during stop() waiting for actor [<path>] to stop` (leer actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKit.scala:264-272).
  - A7. nombre duplicado en `spawn(name)`: *no afirmada por spec*. Caracterizacion propuesta: segundo spawn con el mismo nombre devuelve ref terminado y loguea "Spawn failed" (comportamiento derivado de actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestKitUtils.scala:49-57).
- **Oraculo / Validacion prevista**: test portado (A1-A5) + caracterizacion A6/A7 contrastada con el jar oficial (jar testkit ausente en fetch-jars.sh).
- **Decisiones**: Q-T-TK-1 (API bloqueante vs async; prefijo `assertion failed:`).

## AT-TK-001.3: ActorTestKit: shutdown (public)

- **Contrato observable**: Contractual: `shutdownTestKit()` = `ActorTestKit.shutdown(system, DefaultActorSystemShutdownTimeout, ThrowOnShutdownTimeout)`; `shutdown(system)` usa los `TestKitSettings` del sistema; `shutdown(system, timeout, throwIfShutdownFails = false)`. Llama `system.terminate()` y espera `whenTerminated` (`Await.ready`) hasta el timeout; al vencer arma `"Failed to stop [<nombre>] within [<timeout>] \n<printTree>"` y lanza `RuntimeException` si `throw-on-shutdown-timeout = true`; si no lo imprime por stdout y retorna.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKit.scala:160-172,220-225; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestKitUtils.scala:93-102
- **Precondiciones**: ninguna; idempotente respecto a `terminate()`.
- **Comportamiento**: tras retornar sin error, `system.whenTerminated` esta completado (`Done`).
- **Errores**: `RuntimeException` (timeout con throw=true); con throw=false solo stdout.
- **Orden / concurrencia / ciclo de vida**: bloqueante hasta terminar o vencer; el mensaje incluye el arbol de actores (`system.printTree`).
- **Dependencias**: AT-SYS-005; AT-CS-001; AT-TK-010.
- **Equivalente Rust propuesto**: `shutdown(timeout, throw_on_timeout)` sobre el sistema; Drop no sustituye al `shutdown` explicito salvo decision (Q-T-SYS-4). Se preserva el texto del mensaje y la distincion throw/print.
- **Aserciones**:
  - A1. tras `shutdownTestKit()` el sistema esta terminado (`whenTerminated.futureValue shouldBe a[Done]`) — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitSpec.scala:102-107
  - A2. timeout de shutdown (throw=true lanza RuntimeException con el texto indicado; throw=false imprime): *no afirmada por spec*. Caracterizacion propuesta: sistema con un actor que bloquea `PostStop`, `shutdown(system, 50ms, true)` -> excepcion que empieza con `Failed to stop [<name>] within [50 milliseconds]` (fuente actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestKitUtils.scala:93-101).
- **Oraculo / Validacion prevista**: test portado A1 + caracterizacion A2 (diferencial contra jar oficial cuando se agregue).
- **Decisiones**: Q-T-TK-1.

## AT-TK-002.1: TestProbe: expectMessage, expectMessageType, expectNoMessage, receiveMessage(s) (public)

- **Contrato observable**: Contractual: la probe es un actor de sistema (`systemActorOf`, nombre `<name>-<n>` con el contador global del TestKit clasico) con cola `LinkedBlockingDeque`; `expectMessage(obj)` espera un mensaje (timeout `remainingOrDefault` = `single-expect-default` dilatado, o el resto del bloque `within`) y compara igualdad; `expectMessage(max, [hint,] obj)` dilata `max`; `expectMessageType[T]`/`expectMessageClass` comparan con el tipo boxed; `receiveMessage([max])`; `receiveMessages(n[, max])`; `expectNoMessage([max])` (default `expect-no-message-default`, no dilatado el `max` explicito, ver AT-TK-002.6); `max = 0` hace `poll` sin bloquear. Los mensajes `Terminated` NO van a la cola de mensajes sino a la de terminaciones (AT-TK-002.3). `TestProbe()` usa nombre `testProbe`.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestProbeImpl.scala:55-101,160-276; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProbe.scala:54-61
- **Precondiciones**: esperar una `Signal` no esta permitido (ver Errores).
- **Comportamiento**: `receiveOne` hace `pollFirst` (o con timeout) y siempre pone `lastWasNoMessage = false`; `expectNoMessage` lo pone a `true` cuando no llega nada (lo usa `within`). `receiveMessages(n, max)`: calcula `stop = max + now` y recibe `n` veces con el tiempo restante (`stop - now`) cada vez, devuelve en orden. Los mensajes se encolan en orden de llegada y los `expect*` consumen la cabeza.
- **Errores** (todos `java.lang.AssertionError` lanzados directamente por `assertFail`, SIN prefijo `assertion failed:`): `expectMessage` distinto -> `expected <obj>, found <m>[: <hint>]`; timeout -> `timeout (<max>) during expectMessage while waiting for <obj>[: <hint>]`; `IllegalArgumentException("<clase> is a signal, expecting signals with a TestProbe is not possible")` si `obj` (o la clase en `expectMessageType`) es `Signal`; `expectMessageType`: `Expected <c>, found <clase> (<m>)` / `Timeout (<max>) during expectMessageClass waiting for <c>`; `receiveMessage`: `Timeout (<max>) during receiveMessage while waiting for message.`; `receiveMessages`: `timeout (<max>) while expecting <n> messages (got <k>)`; `expectNoMessage`: `Received unexpected message <m>`.
- **Orden / concurrencia / ciclo de vida**: la cola es concurrente (productores = actores, consumidor = hilo de test); la probe vive hasta `stop()` o fin del sistema.
- **Dependencias**: AT-SYS-008; AT-TK-001.1; AT-TK-010; AT-TK-002.6 (dilatacion); AT-TK-002.5 (`within`).
- **Equivalente Rust propuesto**: `TestProbe<M>` con `expect_message`, `expect_message_type`, `expect_no_message`, `receive_message(s)` (async o bloqueante, Q-T-TK-1). Diferencia: sin jerarquia de tipos, `expect_message_type::<T>()` exige que `M` sea un enum/trait object con downcast (comparar por variante o `TypeId`); el rechazo de `Signal` solo aplica si el modelo tiene el tipo `Signal`. Se preservan textos de error.
- **Aserciones**:
  - A1. `receiveMessages(3)` devuelve `List("one","two","three")` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProbeSpec.scala:135-145
  - A2. `receiveMessages(3, 50.millis)` con un solo mensaje -> `AssertionError` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProbeSpec.scala:147-155
  - A3. `receiveMessage()` devuelve cada `EventT` en orden y luego `expectNoMessage()` no falla — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProbeSpec.scala:157-165
  - A4. `receiveMessage(100.millis)` sin mensaje -> `AssertionError` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProbeSpec.scala:167-169
  - A5. con `default-timeout = 100ms` (y `pekko.test.default-timeout`), `receiveMessage()` sin mensaje falla (`AssertionError`) — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProbeSpec.scala:201-207
  - A6. espejo Java: `receiveMessage()` / `receiveMessage(Duration)` devuelven lo enviado y `expectNoMessage()` no falla — actor-testkit-typed/src/test/java/org/apache/pekko/actor/testkit/typed/javadsl/TestProbeTest.java:35-63
  - A7. espejo Java: `receiveMessage(100ms)` sin mensaje lanza `AssertionError` — actor-testkit-typed/src/test/java/org/apache/pekko/actor/testkit/typed/javadsl/TestProbeTest.java:65-69
  - A8. espejo Java: `expectMessage("message")` y `expectMessageClass(String.class)` devuelven el mensaje, `expectNoMessage()` sin mensajes pendientes — actor-testkit-typed/src/test/java/org/apache/pekko/actor/testkit/typed/javadsl/TestProbeTest.java:105-113
  - A9. textos de error de `expectMessage`/`expectMessageType`/`receiveMessage(s)`/`expectNoMessage`, rechazo de `Signal` y `max = 0` (poll): *no afirmada por spec*. Caracterizacion propuesta (derivada de actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestProbeImpl.scala:173-251): `expectMessage("a")` tras enviar `"b"` -> mensaje `expected a, found b`; `expectMessage(hint)` agrega `: <hint>`; `expectNoMessage(50ms)` tras enviar `"x"` -> `Received unexpected message x`; `expectMessage(Terminated(..))` -> IllegalArgumentException.
- **Oraculo / Validacion prevista**: test portado A1-A8; caracterizacion A9 diferencial contra jar oficial (textos de AssertionError).
- **Decisiones**: Q-T-TK-1.

## AT-TK-002.2: TestProbe: fishForMessage y FishingOutcome (public)

- **Contrato observable**: Contractual: `fishForMessage(max[, hint])(fisher)` recibe mensajes en orden y aplica `fisher`: `Continue` (conserva el mensaje y sigue), `ContinueAndIgnore` (lo descarta y sigue), `Complete` (devuelve los acumulados incluido el actual) o `Fail(error)` (`AssertionError("<error>, hint: <hint>")`). `fishForMessagePF` delega en `fishForMessage` con la funcion parcial; `max` se dilata; cada iteracion descuenta el tiempo ya gastado (`timeout - (nanoTime - start)`). `FishingOutcomes.{complete, continue, continueAndIgnore, fail(msg)}` son los valores publicos (Java: metodos).
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestProbeImpl.scala:278-332; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/FishingOutcome.scala:25-34; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProbe.scala:31-52
- **Precondiciones**: `hint` por defecto `""`.
- **Comportamiento**: bucle `loop(timeout, seen)`: `receiveOne_internal(timeout)`; si hay mensaje aplica `fisher`; un `MatchError` del fisher (solo `PF`) se convierte en `AssertionError("Unexpected message <m> while fishing for messages, seen messages <lista>, hint: <hint>")` con el `MatchError` como causa. Sin mensaje antes de `timeout` -> `AssertionError("timeout (<max>) during fishForMessage, seen messages <lista>, hint: <hint>")` (la lista son los `Continue` vistos, en orden).
- **Errores**: los tres `AssertionError` indicados; ninguno lleva prefijo `assertion failed:` (`assertFail` / `throw new AssertionError`).
- **Orden / concurrencia / ciclo de vida**: consume la cola en orden; los mensajes consumidos no se devuelven aunque la pesca falle.
- **Dependencias**: AT-TK-002.1; AT-TK-002.6.
- **Equivalente Rust propuesto**: `fish_for_message(max, hint, |m| FishingOutcome)`; closure total (sin `MatchError`): el caso "mensaje desconocido" pasa a ser que el closure devuelva `Fail(..)`/panic con el mismo texto; `fish_for_message_pf` se omite o se implementa con `Option<FishingOutcome>` donde `None` produce el texto `Unexpected message ...`. Se preservan los textos de timeout y de fail.
- **Aserciones**:
  - A1. `continue` luego `complete` devuelve `List("one","two")` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProbeSpec.scala:69-82
  - A2. `FishingOutcomes.fail(..)` -> `AssertionError` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProbeSpec.scala:84-97
  - A3. CORREGIDA (antes decia "fisher lento"): la probe no recibe ningun mensaje, asi que `fishForMessage(100.millis)` agota el tiempo en `receiveOne` y lanza `AssertionError`; el `Thread.sleep(150)` del fisher nunca se ejecuta — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProbeSpec.scala:99-108
  - A4. mensaje fuera del dominio de la funcion parcial (`"two"` sin caso) en `fishForMessagePF` -> `AssertionError` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProbeSpec.scala:110-121
  - A5. tras consumir `"one"` no hay mas mensajes -> timeout `AssertionError` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProbeSpec.scala:123-133
  - A6. Java: `continueAndIgnore` descarta `"one","one"` y `complete` en `"two"` -> `List("two")` — actor-testkit-typed/src/test/java/org/apache/pekko/actor/testkit/typed/javadsl/TestProbeTest.java:115-131
  - A7. textos exactos de `Fail`, `Unexpected message` y `timeout ... seen messages`, y el descuento de tiempo por iteracion: *no afirmada por spec*. Caracterizacion propuesta (de actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestProbeImpl.scala:300-331): enviar `"a"`,`"b"`; fisher `Continue` para `"a"` y `Fail("boom")` para `"b"` -> mensaje `boom, hint: <hint>`; con solo `"a"` y max 100ms -> `timeout (100 milliseconds) during fishForMessage, seen messages List(a), hint: `.
- **Oraculo / Validacion prevista**: test portado A1-A6 (A3 con la causa correcta) + caracterizacion A7 diferencial.
- **Decisiones**: Q-T-TK-1.

## AT-TK-002.3: TestProbe: expectTerminated y stop (public)

- **Contrato observable**: Contractual: `expectTerminated(ref[, max])` pide a la probe observar (`context.watch`) el actor (mensaje interno `WatchActor`) y espera su `Terminated` en la cola `terminations`; `max` finito se dilata (`Duration.Zero` -> `pollFirst`, infinito -> `takeFirst`); sin `max` usa `remainingOrDefault`. Sirve aunque el actor ya haya terminado (watch de actor muerto entrega `Terminated`). `probe.stop()` envia `Stop` al actor de la probe, que hace `Behaviors.stopped` (observable desde otra probe con `expectTerminated`).
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestProbeImpl.scala:51-74,334-355,412-414
- **Precondiciones**: ninguna.
- **Comportamiento**: la probe usa `receiveSignal { case (_, t: Terminated) => terminations.offerLast(t) }`; `expectTerminated` compara `message.ref == actorRef`. Los `Terminated` de otros actores observados antes quedan en la cola y pueden hacer fallar la comparacion de la llamada siguiente.
- **Errores**: `AssertionError("assertion failed: timeout (<max>) during expectTerminated waiting for actor [<path>] to stop")` y `AssertionError("assertion failed: expected [<path>] to stop, but saw [<path2>] stop")` (ambos con `Predef.assert`, por tanto con prefijo `assertion failed: `).
- **Orden / concurrencia / ciclo de vida**: `Terminated` llega por senal al actor de la probe; la espera es del hilo del test.
- **Dependencias**: AC-DW-001; AC-DW-002; AT-CTX-005; AT-TK-002.1.
- **Equivalente Rust propuesto**: `expect_terminated(&ActorRef, max)` y `probe.stop()`; requiere DeathWatch correcto. Sin diferencias observables esperadas.
- **Aserciones**:
  - A1. `expectTerminated` sobre actor ya detenido (`spawn(Behaviors.stopped)`) — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProbeSpec.scala:49-53
  - A2. `expectTerminated(ref, 500.millis)` cuando el actor se detiene ~300ms despues (el propio test dice "race") — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProbeSpec.scala:55-67
  - A3. `probe1.stop()` y `probe2.expectTerminated(probe1.ref, probe2.remainingOrDefault)` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProbeSpec.scala:171-176
  - A4. mensajes de timeout y de "otro actor se detuvo", `max = 0` y `max` infinito: *no afirmada por spec*. Caracterizacion propuesta (de actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestProbeImpl.scala:343-355): `expectTerminated(vivo, 50ms)` -> falla con `timeout (50 milliseconds) during expectTerminated waiting for actor [<path>] to stop`.
- **Oraculo / Validacion prevista**: test portado A1-A3 + caracterizacion A4.
- **Decisiones**: Q-T-TK-1.

## AT-TK-002.4: TestProbe: awaitAssert (public)

- **Contrato observable**: Contractual: `awaitAssert(a[, max[, interval]])` reintenta `a` cada `interval` (100 ms por defecto; en Java `awaitAssert(max, interval, creator)`) hasta que `a` no lance (excepcion `NonFatal`, incluye `AssertionError`) o venza `max` (se relanza la ultima excepcion); devuelve el resultado de `a`. Sin `max` usa `remainingOrDefault`; con `max` explicito lo dilata.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestProbeImpl.scala:357-403; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProbe.scala
- **Precondiciones**: `a` debe ser re-ejecutable (efectos idempotentes).
- **Comportamiento**: `stop = now + max`; `poll(t = min(max, interval))`: ejecuta `a`; si lanza `NonFatal e`: si `now + t >= stop` relanza `e`, si no duerme `t.toMillis` (`Thread.sleep`, truncando a milisegundos) y reintenta con `t = min(stop - now, interval)`. Una excepcion no `NonFatal` (p.ej. `InterruptedException`, `VirtualMachineError`) escapa de inmediato.
- **Errores**: la ultima excepcion de `a` tal cual (sin envolver); en Java una excepcion comprobada del `Creator` tambien se propaga.
- **Orden / concurrencia / ciclo de vida**: bloquea el hilo llamante; sin spawn de tareas.
- **Dependencias**: AT-TK-010; AT-TK-002.5 (`remainingOrDefault`).
- **Equivalente Rust propuesto**: `await_assert(f: FnMut() -> Result<T, E>, max, interval)` (o panics capturados con `catch_unwind`); sleep async con reloj inyectable. Se preserva: intervalo 100 ms, reintento hasta el limite y relanzado de la ultima falla. Diferencia: "excepcion `NonFatal`" se traduce a "panic/Err recuperable".
- **Aserciones**:
  - A1. Java: sin fallo, devuelve el valor del creator (`"some result"`) con las tres sobrecargas — actor-testkit-typed/src/test/java/org/apache/pekko/actor/testkit/typed/javadsl/TestProbeTest.java:71-94
  - A2. Java: una excepcion comprobada del creator se propaga al agotar el tiempo (`expected = Exception.class`) — actor-testkit-typed/src/test/java/org/apache/pekko/actor/testkit/typed/javadsl/TestProbeTest.java:96-103
  - A3. uso con reintentos reales: `statsProbe.awaitAssert { ... localSubscriberCount should ===(3) }` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/pubsub/LocalPubSubSpec.scala:44-52
  - A4. el propio testkit lo usa para confirmar suscripciones (`subscribeEventBusAndVerifySubscribed`) — actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKit.scala:310-322 (cubierto por AT-TK-008 A1-A3)
  - A5. intervalo de 100 ms, relanzado de la ultima excepcion, y cantidad de intentos: *no afirmada por spec*. Caracterizacion propuesta: `a` que falla 3 veces y luego tiene exito con `interval = 20ms` -> devuelve el valor y se invoco 4 veces; `a` que siempre falla con `max = 100ms` -> se propaga la ultima excepcion y hubo entre 2 y 6 intentos.
- **Oraculo / Validacion prevista**: test portado A1-A2 + caracterizacion A5 (tolerante a tiempo).
- **Decisiones**: Q-T-TK-1.

## AT-TK-002.5: TestProbe: within, remaining, remainingOr, remainingOrDefault (public)

- **Contrato observable**: Contractual: `within([min,] max)(f)` ejecuta `f` y verifica la duracion del bloque; `end` (fin del bloque) condiciona `remaining*` y los plazos por defecto de los `expect*` dentro del bloque. `remaining` fuera de `within` falla; `remainingOr(d)` devuelve `d` si no hay bloque; `remainingOrDefault = remainingOr(single-expect-default)`. En `within(min, max)` solo `max` se dilata (`min` no); el bloque anidado se acota por el `end` del bloque externo.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestProbeImpl.scala:103-158
- **Precondiciones**: `min <= tiempo restante` del bloque externo.
- **Comportamiento**: `start = now`; `rem = end indefinido ? Inf : end - start`; `assert(rem >= min)`; `lastWasNoMessage = false`; `maxDiff = max min rem`; `end = start + maxDiff` (se restaura el `end` previo en `finally`); tras `f`: `diff = now - start`; `assert(min <= diff)`; si la ultima operacion NO fue un `expectNoMessage` que paso: `assert(diff <= maxDiff)`.
- **Errores**: `AssertionError` con prefijo `assertion failed: ` y textos `required min time <min> not possible, only <resto> left`, `block took <d>, should at least have been <min>`, `block took <d>, exceeding <maxDiff>` (duraciones con `PrettyDuration.pretty`); `remaining` fuera de bloque -> `AssertionError("`remaining` may not be called outside of `within`")` (sin prefijo, `assertFail`); `remainingOr` con `end` infinito -> `IllegalArgumentException("`end` cannot be infinite")`.
- **Orden / concurrencia / ciclo de vida**: estado `end`/`lastWasNoMessage` es por probe, no thread-safe (uso desde un unico hilo de test).
- **Dependencias**: AT-TK-002.1; AT-TK-002.6; AT-TK-010.
- **Equivalente Rust propuesto**: `within(min, max, f)` y `remaining()/remaining_or()/remaining_or_default()` con reloj monotono inyectable (permite ManualTime). Se preservan los tres textos de error y la regla de `lastWasNoMessage`.
- **Aserciones**:
  - A1. Java: `within(3s, supplier)` devuelve el valor del bloque (`"result"`) — actor-testkit-typed/src/test/java/org/apache/pekko/actor/testkit/typed/javadsl/TestProbeTest.java:133-144
  - A2. compila `within(shortDuration){...}` y `remainingOrDefault` (solo API, sin aserciones de comportamiento) — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProbeSpec.scala:29-45,175
  - A3. los tres textos de `within`, `remaining` fuera de bloque, `end` infinito, anidamiento y `lastWasNoMessage`: *no afirmada por spec*. Caracterizacion propuesta (de actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestProbeImpl.scala:103-158): `within(100ms){ sleep(150ms) }` -> `block took 150 milliseconds..., exceeding 100 milliseconds` (aprox.); `within(200ms, 300ms){ }` -> `block took ..., should at least have been 200 milliseconds`; `within(50ms){ expectNoMessage(100ms) }` no falla por exceso; `remaining` fuera de `within` -> falla.
- **Oraculo / Validacion prevista**: test portado A1 + caracterizacion A3 diferencial (jar oficial) con tolerancias de tiempo.
- **Decisiones**: Q-T-TK-1.

## AT-TK-002.6: TestProbe y TestKitSettings: tabla de dilatacion de plazos (timefactor) (public)

- **Contrato observable**: Contractual: `dilated(d) = Duration.fromNanos((d.toNanos * timefactor + 0.5).toLong)` (`TestKitSettings.dilated`, implicito `TestDuration(d).dilated` en el paquete scaladsl). Se dilatan: `max` explicito de `expectMessage`, `expectMessageType`, `receiveMessage`, `receiveMessages`, `fishForMessage`, `expectTerminated`, `awaitAssert`, y el `max` (no el `min`) de `within`; los valores por defecto de `TestKitSettings` ya vienen dilatados (`single-expect-default`, `expect-no-message-default`, `default-timeout`, `system-shutdown-default`, `filter-leeway`). NO se dilatan: el `max` explicito de `expectNoMessage(max)` (se pasa directo a `expectNoMessage_internal`), el `min` de `within`, el `interval` de `awaitAssert`, ni `ManualTime.timePasses` (documentado). Los plazos `Duration.Zero`/infinitos conservan su significado.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestProbeImpl.scala:124-134,162-171,190,211-218,232-238,257-264,279-298,335-341,357-373; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/package.scala:34-36; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/TestKitSettings.scala:90-100
- **Precondiciones**: `timefactor` finito y > 0 (AT-TK-010).
- **Comportamiento**: ver tabla arriba; la correccion respecto de la version anterior del inventario ("todos los plazos con .dilated") es que `expectNoMessage(max)` NO se dilata (los `expectNoMessage()` sin argumento si, por usar el default de settings).
- **Errores**: ninguno propio.
- **Orden / concurrencia / ciclo de vida**: n/a (funcion pura sobre settings).
- **Dependencias**: AT-TK-010; AT-TK-002.1; AT-TK-002.4; AT-TK-002.5.
- **Equivalente Rust propuesto**: `TestKitSettings::dilated(Duration)` y un metodo explicito por operacion; la tabla de "dilata / no dilata" debe copiarse literalmente. Con la decision de duraciones (Q-UTIL-1 rechazada: se preserva el dominio de Pekko), `dilated` debe aceptar el dominio de `FiniteDuration`.
- **Aserciones**:
  - A1. el default dilatado se usa: con `default-timeout = 100ms`, `receiveMessage()` falla a los 100ms (no verifica el factor, solo que el default de settings gobierna) — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProbeSpec.scala:201-207
  - A2. `dilated` con `timefactor` distinto de 1, y la tabla de operaciones no dilatadas: *no afirmada por spec*. Caracterizacion propuesta: con `timefactor = 2`, `expectMessage(100ms, obj)` espera 200 ms y `expectNoMessage(100ms)` espera 100 ms (no 200); `dilated(3s) = 6s`.
- **Oraculo / Validacion prevista**: caracterizacion A2 diferencial con jar oficial (valores efectivos) y test portado A1.
- **Decisiones**: Q-X-1 (lectura de config), Q-T-TK-1.

## AT-TK-003.1: BehaviorTestKit: construccion, ruta, sistema y configuracion (public)

- **Contrato observable**: Contractual: `BehaviorTestKit(initialBehavior[, name[, config]])` (Java: `BehaviorTestKit.create(...)`) construye un `ActorSystemStub("StubbedActorContext", config)` y un `BehaviorTestKitImpl` cuyo actor tiene ruta `pekko://StubbedActorContext/user/<name>#<uid aleatorio>`; `name` por defecto `"testkit"`; `config` por defecto `ConfigFactory.defaultReference` (solo `reference.conf`, NO `application.conf` ni `application-test.conf`); `BehaviorTestKit.ApplicationTestConfig` (= `ActorTestKit.ApplicationTestConfig`) expone la config de `application-test.conf` para usarla de forma explicita. El comportamiento inicial se arranca en el constructor (`Behavior.start` + `validateAsInitial`), asi que `setup` y sus efectos ocurren ya en la construccion.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKit.scala:30-46; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/javadsl/BehaviorTestKit.scala:31-64; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/BehaviorTestKitImpl.scala:41-65,163-168; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/ActorSystemStub.scala:46-65,139-150
- **Precondiciones**: el `Behavior` inicial no puede ser `same` ni `unhandled`.
- **Comportamiento**: tras `Behavior.start`, `runAllTasks()` ejecuta las tareas pendientes del `ControlledExecutor` (AT-TK-012). El hilo "actor" se fija solo durante `start` y `run`/`signal`; fuera de ellos cualquier operacion de contexto lanza (AT-TK-003.4). `system.address` = `Address("pekko","StubbedActorContext")`.
- **Errores**: `IllegalArgumentException("cannot use <behavior> as initial behavior")` si el inicial es `same`/`unhandled` (actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:240-243); cualquier excepcion del `setup` se propaga al constructor.
- **Orden / concurrencia / ciclo de vida**: sincronico, hilo del llamante; no hay actor ni mailbox reales.
- **Dependencias**: AT-TK-011 (sistema stub); AT-TK-012 (executor); AT-TK-010 (ApplicationTestConfig); AT-BEH-*; AT-CTX-016.
- **Equivalente Rust propuesto**: `BehaviorTestKit::new(behavior)`, `with_name`, `with_config`; requiere interprete de `Behavior` sin runtime (Q-T-TK-4). Se preservan: ruta/direccion `pekko://StubbedActorContext/user/<name>`, config por defecto sin `application.conf`, arranque del `setup` en el constructor y el error de comportamiento inicial invalido.
- **Aserciones**:
  - A1. direccion por defecto `Address("pekko","StubbedActorContext")` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:277-280
  - A2. config explicita (`ApplicationTestConfig.withFallback(...)`) visible via `ctx.system.settings.config` en el actor y en un hijo — actor-testkit-typed/src/test/scala/docs/org/apache/pekko/actor/testkit/typed/scaladsl/SyncTestingExampleSpec.scala:226-245
  - A3. Java: `BehaviorTestKit.create(Hello.create(), "hello", BehaviorTestKit.applicationTestConfig())` compila y construye — actor-testkit-typed/src/test/java/jdocs/org/apache/pekko/actor/testkit/typed/javadsl/SyncTestingExampleTest.java:338-342
  - A4. nombre por defecto `"testkit"`, ruta con `#uid`, config por defecto sin `application.conf`, `validateAsInitial`: *no afirmada por spec*. Caracterizacion propuesta: `BehaviorTestKit(b).ref.path.name == "testkit"`; `BehaviorTestKit(Behaviors.same)` -> IllegalArgumentException (de actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKit.scala:35-44).
- **Oraculo / Validacion prevista**: test portado A1-A3 + caracterizacion A4 (jar testkit-typed a agregar a fetch-jars.sh).
- **Decisiones**: Q-T-TK-4, Q-X-8.

## AT-TK-003.2: BehaviorTestKit: run, runOne, signal y estado del comportamiento (public)

- **Contrato observable**: Contractual: `run(message)` entrega el mensaje al comportamiento actual en el hilo llamante, canonicaliza el resultado (`same`/`unhandled` -> el actual; `Deferred` se arranca) y despues ejecuta las tareas pendientes del `ControlledExecutor`; `runOne()` = `run(selfInbox.receiveMessage())`; `signal(sig)` entrega la senal y canonicaliza (sin `runAllTasks`); `currentBehavior` es el comportamiento canonico, `returnedBehavior` es el devuelto por el ultimo mensaje/senal sin canonicalizar (puede ser `same`), `isAlive` = el actual no es `stopped`/`failed`. Los `AdaptMessage` y `AdaptWithRegisteredMessageAdapter` (ask y message adapters) se resuelven con un interceptor antes del comportamiento.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/BehaviorTestKitImpl.scala:150-198,211-246; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKit.scala:141-171; actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:184-192,248-249
- **Precondiciones**: no reentrante: llamar `run` desde dentro de un `run` del mismo testkit lanza `IllegalStateException("Invalid access by thread from the outside of <self>...")` (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorContextImpl.scala:341-349).
- **Comportamiento**: `run`: `setCurrentActorThread`; `intercepted = Interceptor.inteceptBehaviour(current, context)` (envuelve el comportamiento actual en `Behaviors.intercept` cada vez); `currentUncanonical = interpretMessage(intercepted, context, message)`; `current = canonicalize(currentUncanonical, current, context)`; `clearCurrentActorThread`; `runAllTasks`. Se pasa `current` (no `intercepted`) a `canonicalize` para que `same` se resuelva al comportamiento sin interceptar. `signal`: `interpretSignal(current, ...)` + `canonicalize`, y no ejecuta `runAllTasks`.
- **Errores**: si el comportamiento lanza (`NonFatal e`): se intenta entregar `PostStop` al comportamiento actual (el resultado se descarta y sus propios `NonFatal` se ignoran) y se relanza `e` tal cual; `current` no cambia (`isAlive` sigue true). `runOne` con `selfInbox` vacio -> `NoSuchElementException("polling on an empty inbox: <path>")`.
- **Orden / concurrencia / ciclo de vida**: estrictamente sincronico; los mensajes que el comportamiento envia a `self` quedan en `selfInbox` hasta que el test llama `runOne`/`run` (no se auto-procesan).
- **Dependencias**: AT-TK-003.1; AT-TK-003.4 (`selfInbox`); AT-TK-004.4 (interceptor); AT-BEH-*; AT-CTX-016.
- **Equivalente Rust propuesto**: `run(msg)`, `run_one()`, `signal(sig)`, `current_behavior()`, `returned_behavior()`, `is_alive()` sobre un interprete puro (Q-T-TK-4). Se preservan: no auto-procesar `selfInbox`, `signal` sin `runAllTasks`, `PostStop` antes de propagar la falla, y la distincion `returnedBehavior`/`currentBehavior`.
- **Aserciones**:
  - A1. `run` sin canonicalizar: tras `run(SpawnAdapterWithName)`, `currentBehavior` no es `same` y `returnedBehavior` es `Behaviors.same` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:357-363
  - A2. `signal(Terminated(..))` sobre un comportamiento que loguea al recibir senal no lanza errores de validacion de hilo — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:366-374
  - A3. `runOne()` procesa el primer mensaje de `selfInbox` (mensaje adaptado) — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:339-354
  - A4. `isAlive` y la propagacion de excepciones con `PostStop`: *no afirmada por spec*. Caracterizacion propuesta: comportamiento que lanza en `run` -> el llamador recibe la misma excepcion y su handler de `PostStop` se ejecuto exactamente una vez (de actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/BehaviorTestKitImpl.scala:154-161); `Behaviors.stopped` -> `isAlive == false`.
- **Oraculo / Validacion prevista**: test portado A1-A3 + caracterizacion A4 con el jar oficial.
- **Decisiones**: Q-T-TK-4, Q-X-8.

## AT-TK-003.3: BehaviorTestKit: cola de efectos (retrieveEffect, expectEffect*, hasEffects) (public)

- **Contrato observable**: Contractual: cada operacion de contexto registra un `Effect` en una cola FIFO (`ConcurrentLinkedQueue`); `retrieveEffect()` consume el mas antiguo o devuelve `NoEffects` si no hay; `retrieveAllEffects()` los consume todos en orden; `hasEffects()` = cola no vacia; `expectEffect(e)` consume y compara igualdad (un `NoEffects` esperado coincide con cola vacia); `expectEffectType[E]`/`expectEffectClass(c)` consumen y devuelven el efecto concreto (cola vacia coincide si `E` es compatible con `NoEffects`); `expectEffectPF(pf)` consume y aplica la funcion parcial (cola vacia se evalua como `NoEffects`). Java: `getEffect()`, `getAllEffects()`, `expectEffectClass`.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/BehaviorTestKitImpl.scala:88-91,106-148,200; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKit.scala:80-139; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/EffectfulActorContext.scala:44
- **Precondiciones**: ninguna.
- **Comportamiento**: todas las variantes `expect*` CONSUMEN el efecto aunque la comprobacion falle.
- **Errores** (`AssertionError`): `expectEffect`: `assertion failed: expected: <e> but no effects were recorded` / `assertion failed: expected: <e> but found <eff>` (por `Predef.assert`); `expectEffectClass`: `expected: effect type <clase> but no effects were recorded` / `expected: effect class <clase> but found <otro>` (sin prefijo); `expectEffectPF`: `expected matching effect but got: <otro>` (sin prefijo; con cola vacia y PF no definida en `NoEffects`, `<otro>` es `null`).
- **Orden / concurrencia / ciclo de vida**: orden de las llamadas al contexto; cola segura entre hilos.
- **Dependencias**: AT-TK-003.2; AT-TK-004.1; AT-TK-004.2; AT-TK-004.3; AT-TK-004.4; AT-TK-004.5; AT-TK-004.6.
- **Equivalente Rust propuesto**: `retrieve_effect()`, `retrieve_all_effects()`, `has_effects()`, `expect_effect(&Effect)`, `expect_effect_type::<E>()`, `expect_effect_pf(|e| ..)`; `Effect` como enum sellado con `NoEffects`. Se preservan consumo en fallo y textos de error (decision sobre el prefijo `assertion failed:` en Q-T-TK-1).
- **Aserciones**:
  - A1. `expectEffectType[SpawnedAnonymous[_]]` devuelve el efecto concreto (`props === Props.empty`) — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:209-214
  - A2. `expectEffectType[NoEffects]` y `expectEffect(NoEffects)` con cola vacia — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:216-224
  - A3. `hasEffects()` false antes y true despues de `run(SpawnAnonymous(1))` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:226-231
  - A4. `expectEffectPF` sin coincidencia -> `AssertionError` cuyo mensaje empieza con `expected matching effect but got: ` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:233-242
  - A5. `expectEffectPF` con coincidencia devuelve el valor (`"child0"`) y tambien sobre `NoEffects` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:244-259
  - A6. Java: `expectEffectClass(Effect.Spawned.class)`, `expectEffect(Effects.noEffects())`, `expectEffectClass(NoEffects.class)`, `hasEffects`, `getAllEffects` — actor-testkit-typed/src/test/java/org/apache/pekko/actor/testkit/typed/javadsl/BehaviorTestKitTest.java:278-296,318-340
  - A7. consumo del efecto en fallo y textos de `expectEffect`/`expectEffectClass`: *no afirmada por spec*. Caracterizacion propuesta: tras `expectEffect(X)` que falla, `hasEffects()` es false; mensajes segun Errores (de actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/BehaviorTestKitImpl.scala:119-134).
- **Oraculo / Validacion prevista**: test portado A1-A6 + caracterizacion A7.
- **Decisiones**: Q-T-TK-4, Q-T-TK-1 (prefijo).

## AT-TK-003.4: BehaviorTestKit: hijos, selfInbox y childInbox/childTestKit (public)

- **Contrato observable**: Contractual: en `BehaviorTestKit` el contexto es un `StubbedActorContext`: cada `spawn`/`spawnAnonymous` crea un `BehaviorTestKitImpl` hijo (con su propio contexto, `selfInbox` y cola de efectos) registrado en un `TreeMap` por nombre; el nombre anonimo es `Helpers.base64(contador)` (`$a`, `$b`, ...); `selfInbox` recibe lo que el comportamiento envia a `context.self`; `ref = selfInbox().ref`; `childInbox(name | ref)` devuelve el inbox del hijo; `childTestKit(ref)` devuelve su `BehaviorTestKit`. Un hijo no procesa mensajes solo: el test llama `childTestKit(ref).run/runOne`. `context.stop(child)` solo lo quita del registro (no entrega `PostStop` ni `Terminated`).
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/StubbedActorContext.scala:71-143,188-217; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/BehaviorTestKitImpl.scala:93-104; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKit.scala:87-111
- **Precondiciones**: las operaciones del contexto exigen estar dentro de `run`/`signal`/arranque (`checkCurrentActorThread`); fuera lanzan `UnsupportedOperationException("Unsupported access to ActorContext from the outside of <self>. No message is currently processed by the actor, but ActorContext was called from <hilo>.")`.
- **Comportamiento**: `spawn(behavior, name)` falla si el nombre existe; el hijo se arranca en el momento del spawn (su `setup` corre ya). `stop(child)` solo para hijos directos. `children`/`child(name)` leen el registro. `classicActorContext` no existe. `removeChildInbox` (interno) quita un hijo.
- **Errores**: spawn con nombre repetido -> `InvalidActorNameException("actor name <name> is already taken")`; `stop` de un no-hijo -> `IllegalArgumentException("Only direct children of an actor can be stopped through the actor context, but [<child>] is not a child of [<self>]. Stopping other actors has to be expressed as an explicit stop message that the actor accepts.")` (el efecto `Stopped` ya se registro antes de lanzar, ver AT-TK-004.2); `childInbox(name)` inexistente -> `AssertionError("assertion failed: Child not created: <name>. Children created: [<n1>,<n2>]")`; `childTestKit(ref)` inexistente -> `NoSuchElementException("key not found: <name>")`; ref con mismo nombre pero distinto -> `IllegalArgumentException("<ref> is not a child of Inbox(<self>)")`; `tell(null)` a un inbox -> `InvalidMessageException("[null] is not an allowed message")`; `classicActorContext` -> `UnsupportedOperationException("No classic ActorContext available with the stubbed actor context, ...")`.
- **Orden / concurrencia / ciclo de vida**: el orden de `children` es el del `TreeMap` (alfabetico por nombre), no el de creacion.
- **Dependencias**: AT-TK-003.2; AT-TK-005.1; AT-CTX-002; AT-CTX-003; AT-CTX-004; AC-UTIL-002.1 (`base64`).
- **Equivalente Rust propuesto**: arbol de `BehaviorTestKit` hijos en un `BTreeMap<String, ..>`; `child_inbox`, `child_test_kit`, `self_inbox`, `ref`. Se preservan nombres anonimos `$a`, `$b`, el orden alfabetico de `children` y que un hijo no se auto-ejecuta.
- **Aserciones**:
  - A1. `childInbox("hello").ref` es el ref del hijo creado por `spawn` (igual al de `Watched`/`Unwatched`) — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:376-393
  - A2. `childTestKit(sessionRef).run("hello")` y `selfInbox`/handler: el hijo anonimo reenvia al `h`; `KillSession` -> `Stopped` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:396-414
  - A3. parar un hijo con nombre y volver a crearlo con el mismo nombre funciona — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:416-427
  - A4. mensaje enviado a un hijo con nombre queda en `childInbox("child")` — actor-testkit-typed/src/test/scala/docs/org/apache/pekko/actor/testkit/typed/scaladsl/SyncTestingExampleSpec.scala:150-157
  - A5. mensaje a un hijo anonimo se recupera con `childInbox(effect.ref)` — actor-testkit-typed/src/test/scala/docs/org/apache/pekko/actor/testkit/typed/scaladsl/SyncTestingExampleSpec.scala:159-168
  - A6. el primer hijo anonimo se llama `"$a"` (Java) — actor-testkit-typed/src/test/java/jdocs/org/apache/pekko/actor/testkit/typed/javadsl/SyncTestingExampleTest.java:264-272
  - A7. `selfInbox().hasMessages` tras enviar al message adapter; `selfInbox().hasMessages == false` tras un timer `Single` ya disparado — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:339-354,472-493
  - A8. Java: `childTestKit`, `childInbox`, `getRef` y `Stopped` — actor-testkit-typed/src/test/java/org/apache/pekko/actor/testkit/typed/javadsl/BehaviorTestKitTest.java:380-424
  - A9. errores (`already taken`, `Only direct children`, `Child not created`, acceso fuera de hilo) y orden de `children`: *no afirmada por spec*. Caracterizacion propuesta (de actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/StubbedActorContext.scala:118-143 y actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/BehaviorTestKitImpl.scala:93-97): segundo `spawn("child")` -> InvalidActorNameException; `childInbox("x")` inexistente -> mensaje `Child not created: x. Children created: [..]`.
- **Oraculo / Validacion prevista**: test portado A1-A8 + caracterizacion A9.
- **Decisiones**: Q-T-TK-4.

## AT-TK-003.5: BehaviorTestKit: logEntries y clearLog (CapturedLogEvent) (public)

- **Contrato observable**: Contractual: `context.log` es un `Logger` SLF4J de sustitucion (`SubstituteLoggerFactory`, nombre "StubbedLogger") que graba cada evento en una cola; `logEntries()` los devuelve en orden como `CapturedLogEvent(level, message, cause, marker)` con el mensaje YA formateado (`MessageFormatter.arrayFormat`: `{}` sustituidos), `cause` = throwable opcional y `marker` = primer marker opcional; `clearLog()` vacia la cola. No se capturan MDC ni nombre de logger (`setLoggerName` es no-op). Todos los niveles se graban (incluido DEBUG).
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/StubbedActorContext.scala:93-95,221-263; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/CapturedLogEvent.scala:31-98; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/BehaviorTestKitImpl.scala:202-206
- **Precondiciones**: acceso desde el hilo "actor" (`log` hace `checkCurrentActorThread`).
- **Comportamiento**: cada `BehaviorTestKitImpl` (incluido cada hijo) tiene su propio log; los logs de un hijo no aparecen en los del padre. Igualdad de `CapturedLogEvent` = case class `(level, message, cause, marker)`.
- **Errores**: `log` fuera de `run` -> `UnsupportedOperationException` (ver AT-TK-003.4).
- **Orden / concurrencia / ciclo de vida**: orden de emision.
- **Dependencias**: AT-LOG-001; AT-TK-003.2.
- **Equivalente Rust propuesto**: un `Vec<CapturedLogEvent>` por testkit alimentado por la fachada de log del contexto (Q-T-LOG-1, Q-X-7); `CapturedLogEvent{level, message, cause, marker}` con mensaje formateado. Se preserva: formateo previo, orden, ausencia de MDC/logger, log por testkit. Diferencia: `cause` es un `Box<dyn Error>`/String en Rust.
- **Aserciones**:
  - A1. `run(Log("Hello!"))` -> `logEntries() === Seq(CapturedLogEvent(Level.INFO, "Hello!"))` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:261-266
  - A2. `clearLog()` deja `logEntries()` vacio — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:268-275
  - A3. mensaje con `{}` formateado: `"Saying hello to Inboxer"` — actor-testkit-typed/src/test/scala/docs/org/apache/pekko/actor/testkit/typed/scaladsl/SyncTestingExampleSpec.scala:170-177
  - A4. Java: `getAllLogEntries()` y `clearLog()`; el evento Java con constructor de 5 argumentos es igual a `CapturedLogEvent(INFO, msg)` — actor-testkit-typed/src/test/java/org/apache/pekko/actor/testkit/typed/javadsl/BehaviorTestKitTest.java:298-316; actor-testkit-typed/src/test/java/jdocs/org/apache/pekko/actor/testkit/typed/javadsl/SyncTestingExampleTest.java:274-293
  - A5. `cause`, `marker` y la captura de DEBUG: *no afirmada por spec*. Caracterizacion propuesta: `log.error("x", ex)` -> `cause == Some(ex)`; `log.debug("Terminated")` en una senal aparece en `logEntries`.
- **Oraculo / Validacion prevista**: test portado A1-A4 + caracterizacion A5.
- **Decisiones**: Q-T-LOG-1, Q-X-7.

## AT-TK-003.6: BehaviorTestKit: receptionistInbox (public)

- **Contrato observable**: Contractual: `context.system.receptionist` dentro del `BehaviorTestKit` es `receptionistInbox.ref` (un `TestInbox` en `/receptionist`); los `Receptionist.Register/Subscribe/Find` enviados por el comportamiento (y por sus hijos, que comparten el `ActorSystemStub`) se acumulan en `testkit.receptionistInbox()` y no se procesan.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/BehaviorTestKitImpl.scala:208; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/ActorSystemStub.scala:91-93; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKit.scala:183-186
- **Precondiciones**: ninguna.
- **Comportamiento**: sin Listing automatico: el test debe simular la respuesta del receptionist enviando el mensaje al comportamiento.
- **Errores**: ninguno propio.
- **Orden / concurrencia / ciclo de vida**: orden de llegada.
- **Dependencias**: AT-TK-005.1; AT-TK-011; AT-RCP-*.
- **Equivalente Rust propuesto**: `receptionist_inbox()` como `TestInbox<ReceptionistCommand>` compartido por el arbol de testkits.
- **Aserciones**:
  - A1. `run(RegisterWithReceptionist(..))` no falla — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:430-433
  - A2. el `Register(ServiceKey("aladin"), testkit.ref)` queda en `receptionistInbox()` (hasMessages false -> true -> expectMessage -> false) — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:434-441
  - A3. el receptionist compartido con hijos: *no afirmada por spec*. Caracterizacion propuesta: un `Register` enviado desde un hijo aparece en `receptionistInbox()` del padre.
- **Oraculo / Validacion prevista**: test portado A1-A2 + caracterizacion A3.
- **Decisiones**: Q-T-TK-4.

## AT-TK-003.7: BehaviorTestKit: runAsk y runAskWithStatus (public)

- **Contrato observable**: Contractual (desde 1.3.0): `runAsk(f)` crea un `TestInbox("replyTo")` (ruta `pekko.actor.typed.inbox://anonymous/replyTo`, sin uid), ejecuta `run(f(replyToInbox.ref))` y devuelve un `ReplyInbox` sobre ese inbox; `runAskWithStatus(f)` lo mismo con `StatusReply[Res]` y devuelve un `StatusReplyInbox`. Java ademas tiene sobrecargas con `Class[Res]` de inferencia que se ignoran.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/BehaviorTestKitImpl.scala:67-86; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKit.scala:60-75; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/javadsl/BehaviorTestKit.scala:76-126
- **Precondiciones**: el comportamiento debe responder al `replyTo` de forma sincronica dentro del `run` para que haya respuesta disponible.
- **Comportamiento**: la respuesta se lee con `ReplyInbox`/`StatusReplyInbox` (AT-TK-005.2/.3).
- **Errores**: los de `run` (AT-TK-003.2).
- **Orden / concurrencia / ciclo de vida**: sincronico.
- **Dependencias**: AT-TK-003.2; AT-TK-005.2; AT-TK-005.3.
- **Equivalente Rust propuesto**: `run_ask(|reply_to| msg) -> ReplyInbox<Res>`; `run_ask_with_status`. Sin diferencias observables.
- **Aserciones**:
  - A1. `runAsk[ActorRef[String]](SpawnSession(_, h.ref)).receiveReply()` devuelve el ref del hijo — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:400-405
  - A2. `runAsk(KillSession(sessionRef, _)).expectReply(Done)` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:411
  - A3. `runAsk(IsTimerActive("abc", _)).expectReply(false/true)` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:448,461,469
  - A4. Java `runAsk(replyTo -> ...)` con `ReplyInbox` — actor-testkit-typed/src/test/java/org/apache/pekko/actor/testkit/typed/javadsl/BehaviorTestKitTest.java:403-424
  - A5. `runAskWithStatus` + `expectDone()` / `receiveStatusReply().isError` (uso fuera del modulo) — cluster-sharding-typed/src/test/scala/docs/org/apache/pekko/cluster/sharding/typed/AccountExamplePersistenceProbeDocSpec.scala:37,52,56,66
- **Oraculo / Validacion prevista**: test portado A1-A4; A5 queda como uso externo (frontera: cluster-sharding-typed).
- **Decisiones**: Q-T-TK-4.

## AT-TK-004.1: Effects: AskInitiated (ask reificado) (public)

- **Contrato observable**: Contractual: `context.ask`/`askWithStatus` en BehaviorTestKit no envia un ask real: registra `Effect.AskInitiated(target, responseTimeout, responseClass)(askMessage, forwardResponse, mapResponse)`, envia `askMessage` al target (con `replyTo = system.ignoreRef`, AT-TK-012) y permite completar con `respondWith(resp)` o `timeout()` (corre `mapResponse` en el Behavior via `AdaptMessage` sincronicamente); solo se completa una vez. `adaptResponse(resp)`/`adaptTimeout[(msg)]` devuelven el mensaje adaptado sin correr el behavior (repetibles). Igualdad del efecto: solo los argumentos de la primera lista `(target, responseTimeout, responseClass)` (por eso `Effects.askInitiated(..)` con `askMessage` nulo es igual al real).
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/Effect.scala:55-92; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/EffectfulActorContext.scala:46-101; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/Effects.scala:35-39
- **Precondiciones**: el timeout debe ser positivo.
- **Comportamiento**: el mensaje de timeout es `Ask timed out on [<target>] after [<ms> ms]. Message of type [<askMessage.getClass.getName>]. A typical reason for `AskTimeoutException` is that the recipient actor didn't send a reply.` envuelto en `TimeoutException` dentro de un `Failure`. El efecto no se vuelve a registrar al completar. En Java `applyToResponse(null, ex)` en fallo.
- **Errores**: segunda llamada (`respondWith` o `timeout`) -> `IllegalStateException("Can only complete the ask once")` (se marca `sentResponse` antes de reenviar); efecto creado con `Effects.askInitiated` (sin `forwardResponse`) -> `IllegalStateException("Can only complete and ask from a BehaviorTestKit-emitted effect")`; timeout <= 0 -> `IllegalArgumentException("requirement failed: Timeout length must be positive, question not sent to [<target>]")` (`require` de Scala; Java: `getSeconds > 0 || getNano > 0`).
- **Orden / concurrencia / ciclo de vida**: `sendResponse` es `synchronized`; completar corre el Behavior en el hilo llamante.
- **Dependencias**: AT-CTX-012; AT-CTX-013; AT-TK-003.2; AT-TK-003.3; AT-TK-012.
- **Equivalente Rust propuesto**: `Effect::AskInitiated<Req,Res,M>` con `respond_with`, `timeout`, `adapt_response`, `adapt_timeout`. Sin diferencias observables (el texto del timeout usa el nombre de clase del mensaje: en Rust `type_name`, diferencia aceptable solo con decision del Owner).
- **Aserciones**:
  - A1. `expectEffectType[AskInitiated]` es igual a `Effects.askInitiated(cdInbox.ref, 10.seconds, classOf[CookiesForYou])`; `cdInbox.receiveMessage()` es `effect.askMessage`; `adaptResponse` produce `Log("Got 10 cookies...")`; `adaptTimeout` produce un `Log` que empieza con `Failed to get cookies: Ask timed out on [` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:532-554
  - A2. `respondWith` completa: `selfInbox` sin mensajes y un unico log `Got <n> cookies from distributor` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:556-582
  - A3. `timeout()` deja un unico log que empieza con `Failed to get cookies: Ask timed out on [` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:584-609
  - A4. completar dos veces (`respondWith` o `timeout`) -> `IllegalStateException`; solo el primer completado deja log; `hasEffects == false` y `selfInbox` vacio — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:611-642
  - A5. tras `timeout()`, completar o repetir `timeout()` -> `IllegalStateException`, sin logs ni efectos nuevos — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:644-676
  - A6. ejemplo documental: `respondWith` procesado sincronicamente (1 log), `adaptResponse`/`adaptTimeout`/`responseTimeout` repetibles — actor-testkit-typed/src/test/scala/docs/org/apache/pekko/actor/testkit/typed/scaladsl/SyncTestingExampleSpec.scala:179-224; espejo Java (`responseTimeout().toSeconds() == 10`, `effect.askMessage() == question`) — actor-testkit-typed/src/test/java/jdocs/org/apache/pekko/actor/testkit/typed/javadsl/SyncTestingExampleTest.java:296-335
  - A7. Java: `Effects.askInitiated(ref, Duration.ofSeconds(10), CookiesForYou.class, Command.class)` es igual al efecto real — actor-testkit-typed/src/test/java/org/apache/pekko/actor/testkit/typed/javadsl/BehaviorTestKitTest.java:440-455
  - A8. texto exacto "Can only complete the ask once" / "Can only complete and ask from a BehaviorTestKit-emitted effect" / timeout <= 0: *no afirmada por spec* (los tests solo comprueban el tipo `IllegalStateException`). Caracterizacion propuesta: `Effects.askInitiated(..).respondWith(x)` -> ISE con el segundo texto; ask con `Timeout(0)` -> IllegalArgumentException con `requirement failed: Timeout length must be positive...`.
- **Oraculo / Validacion prevista**: test portado A1-A7 + caracterizacion A8 (jar testkit-typed a agregar a fetch-jars.sh).
- **Decisiones**: Q-T-TK-4.

## AT-TK-004.2: Effects: Spawned, SpawnedAnonymous y Stopped (public)

- **Contrato observable**: Contractual: `spawn(behavior, name, props)` -> `Spawned(behavior, childName, props)(ref)`; `spawnAnonymous(behavior, props)` -> `SpawnedAnonymous(behavior, props)(ref)`; `stop(child)` -> `Stopped(child.path.name)`. Igualdad de `Spawned` por `(behavior, childName, props)` (el `ref` no cuenta; la fabrica `apply` pone `ref = null`); `SpawnedAnonymous` por `(behavior, props)`; `Stopped(childName)` es case class. `Spawned` y `SpawnedAnonymous` exponen `ref` (para obtener el hijo sin respuesta explicita).
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/Effect.scala:94-150,228-231; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/EffectfulActorContext.scala:103-107,128-136; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/Effects.scala:41-90
- **Precondiciones**: ninguna.
- **Comportamiento**: `spawn`/`spawnAnonymous` registran el efecto DESPUES de crear el hijo (un spawn fallido por nombre repetido no registra efecto); `stop` registra `Stopped` ANTES de validar el padre (un `stop` de un no-hijo registra el efecto y luego lanza). `Props` se compara por igualdad (incluye dispatcher `cat`).
- **Errores**: los del contexto (AT-TK-003.4).
- **Orden / concurrencia / ciclo de vida**: orden de las llamadas.
- **Dependencias**: AT-TK-003.3; AT-TK-003.4; AT-CTX-002; AT-CTX-004.
- **Equivalente Rust propuesto**: variantes `Effect::Spawned{behavior, name, props, ref}` / `SpawnedAnonymous` / `Stopped`; `PartialEq` manual que ignora `ref`; igualdad de `Behavior` en Rust requiere identidad (p.ej. `Rc::ptr_eq`/id) porque los closures no son comparables (decision a documentar en Q-T-TK-4).
- **Aserciones**:
  - A1. `spawn` sin props -> `Spawned(Child.initial,"child0")` y `Spawned(Child.initial,"child1", Props.empty)` (`contain.only`, sin orden) — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:284-289
  - A2. `spawn` con props (`Props.empty.withDispatcherFromConfig("cat")`) registra esas props — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:291-296
  - A3. `spawnAnonymous` con y sin props -> `Seq(SpawnedAnonymous(..), SpawnedAnonymous(..))` en orden — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:300-313
  - A4. `stop` registra `Stopped(child.childName)` y el nombre se puede reutilizar — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:416-427
  - A5. el efecto de `spawnAnonymous` expone `ref` igual al ref respondido — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:396-405
  - A6. Java: `Effects.spawned(childInitial,"child0")`, `spawnedAnonymous`, `props()` y `Stopped` — actor-testkit-typed/src/test/java/org/apache/pekko/actor/testkit/typed/javadsl/BehaviorTestKitTest.java:330-366,423
  - A7. ejemplo documental `expectEffect(Spawned(childActor,"child"))` y `expectEffect(SpawnedAnonymous(childActor))` — actor-testkit-typed/src/test/scala/docs/org/apache/pekko/actor/testkit/typed/scaladsl/SyncTestingExampleSpec.scala:125-139
  - A8. orden del efecto en `stop` de no-hijo y en spawn duplicado: *no afirmada por spec*. Caracterizacion propuesta: `stop(refAjeno)` -> la cola contiene `Stopped(nombre)` y el contexto lanzo IllegalArgumentException; spawn duplicado -> cola sin `Spawned` nuevo.
- **Oraculo / Validacion prevista**: test portado A1-A7 + caracterizacion A8.
- **Decisiones**: Q-T-TK-4.

## AT-TK-004.3: Effects: Watched, WatchedWith y Unwatched (public)

- **Contrato observable**: Contractual: `watch(other)` -> `Watched(other)`, `watchWith(other, message)` -> `WatchedWith(other, message)`, `unwatch(other)` -> `Unwatched(other)` (case classes; igualdad por `other` y `message`). El testkit NO observa realmente: no se entrega `Terminated` ni el mensaje de `watchWith` automaticamente; el test los simula con `signal(Terminated(ref))` o `run(message)`.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/Effect.scala:233-246; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/EffectfulActorContext.scala:137-148; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/StubbedActorContext.scala:144-152
- **Precondiciones**: dentro de `run`.
- **Comportamiento**: el efecto se registra antes de la llamada base (no-op salvo chequeo de hilo).
- **Errores**: acceso fuera de hilo (AT-TK-003.4).
- **Orden / concurrencia / ciclo de vida**: orden de llamadas.
- **Dependencias**: AT-TK-003.3; AT-CTX-005; AT-CTX-006; AT-CTX-007.
- **Equivalente Rust propuesto**: variantes `Watched`, `WatchedWith`, `Unwatched`; sin entrega automatica de senales.
- **Aserciones**:
  - A1. `spawn + watch + unwatch` produce `Seq(spawned, watched(child), unwatched(child))` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:377-383
  - A2. `watchWith` produce `Seq(spawned, watchedWith(child, msg))` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:385-392
  - A3. Java: `Effect.Watched.other()`, `Unwatched.other()`, `WatchedWith.message()` — actor-testkit-typed/src/test/java/org/apache/pekko/actor/testkit/typed/javadsl/BehaviorTestKitTest.java:380-401
  - A4. ausencia de entrega automatica de `Terminated`: *no afirmada por spec*. Caracterizacion propuesta: tras `watch(child)` y `stop(child)` no aparece ninguna senal; `testkit.signal(Terminated(child))` si la entrega.
- **Oraculo / Validacion prevista**: test portado A1-A3 + caracterizacion A4.
- **Decisiones**: Q-T-TK-4.

## AT-TK-004.4: Effects: MessageAdapter, SpawnedAdapter y SpawnedAnonymousAdapter (public)

- **Contrato observable**: Contractual: `messageAdapter[U](f)` / `messageAdapter(clase, f)` -> `MessageAdapter(messageClass, adapt)` (case class; igualdad incluye la funcion, por referencia); internamente se registra una unica referencia adaptadora (hijo `$n-adapter`) y una lista `(clase boxed, f)` sin duplicar clases (la mas reciente reemplaza). Un mensaje enviado al ref adaptador llega a `selfInbox` como `AdaptWithRegisteredMessageAdapter(msg)` (y a la bandeja del hijo adaptador como el mensaje original); al hacer `runOne()`, el interceptor busca el primer adaptador cuya clase `isInstance` y aplica `f` antes de entregar. `spawnMessageAdapter(f[, name])` (API interna) -> `SpawnedAdapter(name)` / `SpawnedAnonymousAdapter` (igualdad solo por nombre / siempre iguales); el ref es un `FunctionRef` que adapta y reenvia a `selfInbox`.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/Effect.scala:152-226; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/EffectfulActorContext.scala:108-127; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/StubbedActorContext.scala:172-186; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/BehaviorTestKitImpl.scala:211-238; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorContextImpl.scala:294-336
- **Precondiciones**: dentro de `run`.
- **Comportamiento**: `messageAdapter` NO registra `SpawnedAdapter` (llama `internalSpawnMessageAdapter` directamente); si `f` devuelve `null` el mensaje no se reenvia. El ref adaptador no se puede usar como destino de `ask` (`provider` lanza `UnsupportedOperationException("ActorRefs created for synchronous testing cannot be used as targets for asking...")`).
- **Errores**: sin adaptador para el mensaje -> `RuntimeException("can't find a message adaptor for <msg>")` (`sys.error`) al hacer `runOne`.
- **Orden / concurrencia / ciclo de vida**: el efecto `MessageAdapter` se registra cada vez que se llama a `messageAdapter` aunque reemplace uno existente.
- **Dependencias**: AT-CTX-011; AT-CTX-018; AT-TK-003.2; AT-TK-003.4; AT-TK-005.1.
- **Equivalente Rust propuesto**: `Effect::MessageAdapter{type_id, adapt}` donde la igualdad compara `type_id` y la identidad de la funcion; el adaptador por tipo exacto (sin subclases; Q-T-CTX-1).
- **Aserciones**:
  - A1. `spawnMessageAdapter` sin nombre -> `Seq(SpawnedAnonymousAdapter())`; con nombre -> `Seq(SpawnedAdapter("adapter"))` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:316-330
  - A2. `messageAdapter` registra un efecto `MessageAdapter` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:333-337
  - A3. un mensaje enviado al ref adaptador llega a `selfInbox` y `runOne()` lo adapta (`2` -> `SpawnChildren(2)` -> `Spawned child0`, `child1`) — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:339-354
  - A4. Java: `MessageAdapter.messageClass()` y `adaptFunction().apply("anything")` — actor-testkit-typed/src/test/java/org/apache/pekko/actor/testkit/typed/javadsl/BehaviorTestKitTest.java:368-378
  - A5. reemplazo por clase, `null` no reenviado, y `can't find a message adaptor`: *no afirmada por spec*. Caracterizacion propuesta (de actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorContextImpl.scala:314-331 y actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/BehaviorTestKitImpl.scala:220-228): dos `messageAdapter` de la misma clase -> solo el segundo aplica.
- **Oraculo / Validacion prevista**: test portado A1-A4 + caracterizacion A5.
- **Decisiones**: Q-T-TK-4, Q-T-CTX-1.

## AT-TK-004.5: Effects: ReceiveTimeoutSet, ReceiveTimeoutCancelled, Scheduled y NoEffects (public)

- **Contrato observable**: Contractual: `setReceiveTimeout(d, msg)` -> `ReceiveTimeoutSet(d, msg)` (Java `duration()`); `cancelReceiveTimeout()` -> `ReceiveTimeoutCancelled` (case object); `scheduleOnce(delay, target, msg)` -> `Scheduled(delay, target, msg)` (Java `duration()`) y devuelve un `Cancellable` inerte (`cancel() = false`, `isCancelled = true`); `NoEffects` representa cola vacia. En el testkit el receive timeout nunca dispara y los `scheduleOnce` nunca se entregan: el test debe enviar el mensaje manualmente.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/Effect.scala:248-269,304-312; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/EffectfulActorContext.scala:149-160; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/StubbedActorContext.scala:153-164
- **Precondiciones**: dentro de `run` para receive timeout (`checkCurrentActorThread`); `scheduleOnce` no valida hilo.
- **Comportamiento**: ver contrato; `FIXME` del codigo: efectos del scheduler tipado (`system.scheduler`) no se reflejan (`scheduler` del stub lanza).
- **Errores**: `system.scheduler` -> `UnsupportedOperationException("no scheduler")` (AT-TK-011).
- **Orden / concurrencia / ciclo de vida**: orden de llamadas.
- **Dependencias**: AT-TK-003.3; AT-CTX-008; AT-CTX-009; AT-TK-011.
- **Equivalente Rust propuesto**: variantes `ReceiveTimeoutSet`, `ReceiveTimeoutCancelled`, `Scheduled`, `NoEffects`.
- **Aserciones**:
  - A1. `NoEffects` por tipo y por valor con cola vacia — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:216-224
  - A2. `Effects.noEffects()` (Java) — actor-testkit-typed/src/test/java/org/apache/pekko/actor/testkit/typed/javadsl/BehaviorTestKitTest.java:286-296
  - A3. `ReceiveTimeoutSet`, `ReceiveTimeoutCancelled` y `Scheduled`: *no afirmada por spec* (ningun test del repositorio los ejercita; solo existen las fabricas en `Effects`). Caracterizacion propuesta: un comportamiento que llama `setReceiveTimeout(1s, M)`, `cancelReceiveTimeout()` y `scheduleOnce(2s, ref, X)` produce, en ese orden, `ReceiveTimeoutSet(1s, M)`, `ReceiveTimeoutCancelled`, `Scheduled(2s, ref, X)`, y el `Cancellable` devuelto tiene `cancel() == false`.
- **Oraculo / Validacion prevista**: A1-A2 test portado; A3 caracterizacion diferencial con el jar testkit-typed.
- **Decisiones**: Q-T-TK-4.

## AT-TK-004.6: Effects: TimerScheduled, TimerCancelled y simulacion de timers (public)

- **Contrato observable**: Contractual: el `TimerScheduler` del contexto de pruebas no programa tiempo real: cada `startSingleTimer`/`startTimerWithFixedDelay`/`startTimerAtFixedRate`/`startPeriodicTimer` registra `TimerScheduled(key, msg, delay, mode, overriding)(send)` y lo guarda como activo; `cancel(key)` registra `TimerCancelled(key)` solo si el timer estaba activo; `isTimerActive(key)` consulta el mapa. `mode`: `SingleMode`; `FixedDelayMode` / `FixedDelayModeWithInitialDelay(initial)` (con `delay` = el retraso) y `FixedRateMode` / `FixedRateModeWithInitialDelay(initial)`; `startPeriodicTimer` = `FixedRateMode`; las sobrecargas SIN `initialDelay` de fixed delay/rate usan el modo sin initial, las que lo reciben usan `...WithInitialDelay`. `overriding` es `true` si la clave ya estaba activa. El test dispara el timer llamando `effect.send()`: busca el efecto ACTUAL de la clave (si fue reemplazado o cancelado, `send` usa el nuevo/ no hace nada), si es `SingleMode` lo quita del activo, y envia `msg` a `self` (queda en `selfInbox`).
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/Effect.scala:271-302; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/EffectfulActorContext.scala:162-211; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorContextImpl.scala:114-132
- **Precondiciones**: timers por contexto (`Behaviors.withTimers`), creados al primer uso (con chequeo de hilo).
- **Comportamiento**: `cancelAll()` esta implementado como `activeTimers.foreach(cancel)`, que pasa la TUPLA `(clave, efecto)` a `cancel(key: Any)`: `activeTimers.keySet(tupla)` es falso, por lo que `cancelAll()` NO cancela nada ni registra `TimerCancelled` (inferido de la semantica de Scala; ningun test lo ejercita; el contrato propuesto es paridad con este no-op, ver Q-T-TK-5). `timers` de Single se retiran al `send`; los repetitivos permanecen. No hay avance de tiempo: ManualTime es un mecanismo distinto (AT-TK-007).
- **Errores**: acceso fuera de hilo.
- **Orden / concurrencia / ciclo de vida**: los efectos se encolan en orden; `activeTimers` es un `var` sin sincronizacion (uso mono-hilo).
- **Dependencias**: AT-TMR-001..AT-TMR-007.1; AT-TK-003.3; AT-TK-003.2.
- **Equivalente Rust propuesto**: `Effect::TimerScheduled{key, msg, delay, mode, overriding, send}` con `send` como closure/handle; `TimerMode` enum de 5 variantes. Claves: `Any` -> tipo de clave de Q-T-TMR-1.
- **Aserciones**:
  - A1. `IsTimerActive` false -> `ScheduleCommand("abc", 42s, SingleMode, SpawnChild)` produce `TimerScheduled("abc", SpawnChild, 42s, SingleMode, false)`; activo true; `CancelScheduleCommand` produce `TimerCancelled("abc")`; activo false — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:445-470
  - A2. `effect.send()` + `runOne()` entrega `SpawnChild` (se ve `Spawned(_,"child",_)`); un segundo `send()` no entrega nada (el timer `Single` fue retirado, `selfInbox` vacio) — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:472-493
  - A3. `startTimerAtFixedRate(key, cmd, delay, delay)` produce `FixedRateModeWithInitialDelay(delay)`; el timer repetitivo sigue activo tras disparar y tras parar el hijo; `CancelScheduleCommand` -> `TimerCancelled("abc")` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:495-528
  - A4. Java: `Effects.timerScheduled(key, msg, Duration.ofSeconds(42), Effect.timerScheduled().fixedDelayMode(), false, () -> {})` compila — actor-testkit-typed/src/test/java/org/apache/pekko/actor/testkit/typed/javadsl/BehaviorTestKitTest.java:426-438
  - A5. `overriding = true` al reiniciar una clave, `startTimerWithFixedDelay` sin initial -> `FixedDelayMode`, `startPeriodicTimer` -> `FixedRateMode`, `cancel` de clave inactiva sin efecto, y `cancelAll` como no-op: *no afirmada por spec*. Caracterizacion propuesta (de actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/EffectfulActorContext.scala:162-211): arrancar dos veces la misma clave -> segundo `TimerScheduled` con `overriding = true`; `cancel("zz")` inactivo -> `hasEffects == false`; `cancelAll()` con un timer activo -> `isTimerActive == true` y sin `TimerCancelled`.
- **Oraculo / Validacion prevista**: test portado A1-A4; caracterizacion A5 diferencial con el jar testkit-typed (confirma `cancelAll`).
- **Decisiones**: Q-T-TK-4, Q-T-TK-5 (cancelAll), Q-T-TMR-1.

## AT-TK-004.7: Effects: fabricas Effects (scaladsl/javadsl) y convenciones de Effect (public)

- **Contrato observable**: Contractual: `Effects.{askInitiated, spawned(..4 sobrecargas), spawnedAnonymous(..4), stopped, watched, watchedWith, unwatched, receiveTimeoutSet, scheduled, noEffects}` (y su espejo Java `javadsl.Effects`, mas `timerScheduled`/`messageAdapter`/`receiveTimeoutCancelled` segun el `object Effect`) construyen efectos para comparar con `expectEffect`; `Effect` es abstracto y sellado para extension (`@DoNotInherit`, constructor `private[pekko]`). Las fabricas de `spawned*` con `ref` permiten comparar tambien por referencia.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/Effects.scala:25-123; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/javadsl/Effects.scala:14-134; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/Effect.scala:37-40,299-300
- **Precondiciones**: ninguna.
- **Comportamiento**: fabricas puras; `askInitiated` crea el efecto con `askMessage`, `forwardResponse` y `mapResponse` nulos (no completable, AT-TK-004.1).
- **Errores**: ninguno.
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: AT-TK-004.1; AT-TK-004.2; AT-TK-004.3; AT-TK-004.5; AT-TK-004.6.
- **Equivalente Rust propuesto**: constructores asociados de `Effect` (`Effect::spawned(..)`, ...); una sola API Rust cubre scaladsl y javadsl (fachada duplicada, Q-T-JAPI-1).
- **Aserciones**:
  - A1. `Effects.spawned/watched/unwatched/watchedWith` comparados con `retrieveAllEffects()` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:377-392
  - A2. `Effects.askInitiated` igual al efecto real — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:544
  - A3. Java: `Effects.noEffects()`, `Effects.spawned(..)`, `Effects.spawnedAnonymous(..)`, `Effects.askInitiated(..)` — actor-testkit-typed/src/test/java/org/apache/pekko/actor/testkit/typed/javadsl/BehaviorTestKitTest.java:286-290,331-340,350-358,440-455
  - A4. `receiveTimeoutSet`, `scheduled`, `stopped` por fabrica: *no afirmada por spec*. Caracterizacion propuesta: `Effects.stopped("a") == Stopped("a")`, `Effects.scheduled(1s, ref, m) == Scheduled(1s, ref, m)`.
- **Oraculo / Validacion prevista**: test portado A1-A3 + caracterizacion A4.
- **Decisiones**: Q-T-JAPI-1, Q-T-TK-4.

## AT-TK-005.1: TestInbox (public)

- **Contrato observable**: Contractual: `TestInbox[T](name = "inbox")` (Java `TestInbox.create([name])`) es un buzon sincronico sin actor: `ref` es un `FunctionRef` cuyo `tell` encola en una `ConcurrentLinkedQueue`; `receiveMessage()` consume el mas antiguo; `expectMessage(m)` consume y compara (devuelve `this`); `receiveAll()` (Java `getAllReceived()`) drena en orden; `hasMessages`. Ruta del inbox: `pekko.actor.typed.inbox://anonymous/<name>#<uid>` (el uid es aleatorio en `TestInbox(...)`; los inboxes internos de `TestInboxImpl(name)` no llevan uid).
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestInbox.scala:28-78; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestInboxImpl.scala:32-79; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/StubbedActorContext.scala:39-62
- **Precondiciones**: ninguna.
- **Comportamiento**: el `FunctionRef` no soporta watch (`sendSystem` es no-op), `isTerminated = false`, `isLocal = true`.
- **Errores**: `receiveMessage` vacio -> `NoSuchElementException("polling on an empty inbox: <path>")`; `expectMessage` sin mensajes -> `AssertionError("assertion failed: expected message: <m> but no messages were received")`; distinto -> `AssertionError("assertion failed: expected: <m> but received <otro>")` (ambos via `Predef.assert`); `ref ! null` -> `InvalidMessageException("[null] is not an allowed message")`; usar el ref como destino de `ask` -> `UnsupportedOperationException("ActorRefs created for synchronous testing cannot be used as targets for asking. Use asynchronous testing instead. See https://pekko.apache.org/docs/pekko/current/typed/testing.html#asynchronous-testing")`.
- **Orden / concurrencia / ciclo de vida**: FIFO; segura entre hilos.
- **Dependencias**: AT-REF-001; AT-TK-003.4.
- **Equivalente Rust propuesto**: `TestInbox<M>` con `VecDeque` protegido (o canal mpsc no bloqueante); `r#ref()` devuelve un `ActorRef<M>` local. Se preservan: FIFO, ruta `pekko.actor.typed.inbox://anonymous/<name>`, errores y rechazo de `ask`.
- **Aserciones**:
  - A1. `receiveMessage()` devuelve el mensaje enviado al ref adaptador — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:339-345
  - A2. `receiveAll()` devuelve `Seq("hello")` en orden — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:407-409
  - A3. `expectMessage` correcto, encadenado (`inbox.expectMessage("hello")`, `childInbox.expectMessage(..)`) — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:439-440; actor-testkit-typed/src/test/scala/docs/org/apache/pekko/actor/testkit/typed/scaladsl/SyncTestingExampleSpec.scala:141-157
  - A4. `hasMessages` false/true — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:436-440,565
  - A5. Java: `TestInbox.create()`, `getRef()`, `getAllReceived()` — actor-testkit-typed/src/test/java/org/apache/pekko/actor/testkit/typed/javadsl/BehaviorTestKitTest.java:403-424
  - A6. errores (`polling on an empty inbox`, `expectMessage` sin/ con mensaje distinto, null, ask): *no afirmada por spec*. Caracterizacion propuesta (de actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestInboxImpl.scala:41-52 y actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/StubbedActorContext.scala:44-58): inbox vacio -> `receiveMessage` falla con `polling on an empty inbox: pekko.actor.typed.inbox://anonymous/inbox#<uid>`.
- **Oraculo / Validacion prevista**: test portado A1-A5 + caracterizacion A6.
- **Decisiones**: Q-T-TK-4.

## AT-TK-005.2: ReplyInbox (public)

- **Contrato observable**: Contractual (desde 1.3.0): buzon de una unica respuesta devuelto por `runAsk`: `receiveReply()` consume la respuesta (y marca el inbox como consumido); `expectReply(r)` consume y compara; `expectNoReply()` comprueba que NUNCA llego respuesta (y no consume) devolviendo `this`; `hasReply`.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestInboxImpl.scala:85-122; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestInbox.scala:80-110
- **Precondiciones**: ninguna.
- **Comportamiento**: el `underlying` se pone a `None` ANTES de leer el mensaje: tras `receiveReply`/`expectReply` el `ReplyInbox` queda consumido aunque no hubiese respuesta.
- **Errores**: segunda lectura -> `AssertionError("Reply was already received")`; `receiveReply` sin respuesta -> `NoSuchElementException("polling on an empty inbox: pekko.actor.typed.inbox://anonymous/replyTo")` (y queda consumido); `expectReply` distinto -> `AssertionError("Expected <esperado> but received <recibido>")`; `expectNoReply` con respuesta -> `AssertionError("Expected no reply, but <r> was received")`; `expectNoReply` tras consumir -> `AssertionError("Improper expectation of no reply: reply was already received")`. Estos son `throw new AssertionError` directos (sin prefijo `assertion failed:`).
- **Orden / concurrencia / ciclo de vida**: `underlying` es `var` sin sincronizar (mono-hilo).
- **Dependencias**: AT-TK-003.7; AT-TK-005.1.
- **Equivalente Rust propuesto**: `ReplyInbox<R>` con `Option<TestInbox<R>>`; `receive_reply(&mut self)`. Se preservan textos y la regla de consumo previo a la lectura.
- **Aserciones**:
  - A1. `receiveReply()` devuelve la respuesta — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:400-405
  - A2. `expectReply(Done)`, `expectReply(false)`, `expectReply(true)` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:411,448,461,469
  - A3. Java `ReplyInbox<Done>.expectReply(Done.getInstance())` — actor-testkit-typed/src/test/java/org/apache/pekko/actor/testkit/typed/javadsl/BehaviorTestKitTest.java:420-421
  - A4. `expectNoReply`, `hasReply`, consumo previo y todos los textos de error: *no afirmada por spec*. Caracterizacion propuesta (de actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestInboxImpl.scala:89-121): `runAsk` de un comportamiento que no responde -> `expectNoReply()` devuelve el inbox y `hasReply == false`; luego `receiveReply()` -> NoSuchElementException y despues `receiveReply()` -> `Reply was already received`.
- **Oraculo / Validacion prevista**: test portado A1-A3 + caracterizacion A4.
- **Decisiones**: Q-T-TK-4.

## AT-TK-005.3: StatusReplyInbox (public)

- **Contrato observable**: Contractual (desde 1.3.0): como `ReplyInbox` pero para `StatusReply`: `receiveStatusReply()`, `receiveValue()` (exige `Success`), `receiveError()` (exige `Error`), `expectValue(v)`, `expectErrorMessage(msg)` (compara `getMessage`), `expectDone()` (Scala: solo si `T =:= Done`), `expectNoReply()`, `hasReply`.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestInboxImpl.scala:128-184; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestInbox.scala:112-170
- **Precondiciones**: ninguna.
- **Comportamiento**: igual regla de consumo que `ReplyInbox`.
- **Errores** (`AssertionError` directos): `Reply was already received`; `Expected a successful reply but received <r>`; `Expected an error reply but received <r>`; `Expected <esperado> but received <recibido>`; `Expected a throwable with message <m>, but got <m2>`; `Expected no reply, but <r> was received`; `Improper expectation of no reply: reply was already received`.
- **Orden / concurrencia / ciclo de vida**: mono-hilo.
- **Dependencias**: AT-TK-003.7; AT-TK-005.1; AT-CTX-013 (`StatusReply`).
- **Equivalente Rust propuesto**: `StatusReplyInbox<R>` sobre `Result<R, StatusError>`; `expect_done()` disponible solo para `R = Done` (trait bound o metodo especializado).
- **Aserciones**:
  - A1. `runAskWithStatus[Done](..).expectDone()` y `receiveStatusReply().isError shouldBe true` (uso fuera del modulo, frontera cluster-sharding-typed) — cluster-sharding-typed/src/test/scala/docs/org/apache/pekko/cluster/sharding/typed/AccountExamplePersistenceProbeDocSpec.scala:37,52,56,66
  - A2. todos los metodos de `StatusReplyInbox` y sus mensajes de error: *no afirmada por spec* en `actor-testkit-typed` (ningun test del modulo la usa). Caracterizacion propuesta (de actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestInboxImpl.scala:132-177): respuesta `StatusReply.success(5)` -> `receiveValue() == 5`; `StatusReply.error("x")` -> `expectErrorMessage("x")` ok y `receiveValue()` falla con `Expected a successful reply but received Error(x)`.
- **Oraculo / Validacion prevista**: caracterizacion A2 diferencial con el jar testkit-typed.
- **Decisiones**: Q-T-TK-4.

## AT-TK-006.1: LoggingTestKit: condiciones de coincidencia (matches) y LoggingEvent (public)

- **Contrato observable**: Contractual: `LoggingTestKit` es un filtro inmutable (case class con `occurrences = 1`, `checkExcess = true`, resto vacio) construido con `withX`; `matches(event)` es verdadero si TODAS las condiciones presentes coinciden: `logLevel` igual; `source` igual al MDC `pekkoSource` (cadena vacia si falta); `messageContains` (subcadena del mensaje; `null` se trata como ""); `messageRegex` (`findFirstIn` definido, es decir busqueda, no coincidencia total); `cause` (hay `throwable` y es instancia de la clase, subclases incluidas); `mdc` (el MDC del evento CONTIENE todas las entradas pedidas; puede tener mas); `custom` (funcion sobre el evento). `loggerName` NO se evalua en `matches` (se aplica al instalar el filtro, AT-TK-006.4). `LoggingEvent` = `(level, loggerName, threadName, message, timeStamp, marker, throwable, mdc)`.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/LoggingTestKitImpl.scala:36-96,136-175; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/LoggingTestKit.scala:35-123; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/LoggingEvent.scala:25-88
- **Precondiciones**: ninguna.
- **Comportamiento**: `withX` devuelven una copia; `withMdc` REEMPLAZA el mapa; `withCause[A]` guarda la clase; `withOccurrences`, `withCheckExcess` ver AT-TK-006.3. Java: `withMdc(java.util.Map)`, `withCause(Class)`, `LoggingEvent.create(..)`.
- **Errores**: ninguno (la custom puede lanzar; en el appender eso cuenta como "no coincide", AT-TK-006.4).
- **Orden / concurrencia / ciclo de vida**: las condiciones se evaluan en cortocircuito: nivel, source, contains, regex, cause, mdc, custom.
- **Dependencias**: AT-LOG-002 (clave `pekkoSource`); AT-LOG-001.
- **Equivalente Rust propuesto**: `LoggingTestKit` builder inmutable sobre un `LogEvent{level, target, thread, message, timestamp, marker, error, fields}`; `matches(&LogEvent)`. Diferencias (limites de `tracing` frente a SLF4J): `cause` por clase se resuelve con el tipo concreto del error registrado (`record_error` + `downcast_ref`) y no hay herencia, de modo que "instancia de subclase" se reduce a tipo exacto o a un predicado dado por el usuario; `mdc` se corresponde con campos del evento mas campos de los spans activos (incluida la clave `pekkoSource` si Q-T-LOG-1 la conserva); `marker` como campo opcional; el regex es el de `regex` crate (sintaxis distinta de `java.util.regex`/`scala.util.matching.Regex`).
- **Aserciones**:
  - A1. nivel ERROR coincide con y sin causa — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/LoggingEventFilterSpec.scala:70-78
  - A2. mensaje por subcadena: `error("an error")` coincide con "this is an error" y `error("another error")` no — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/LoggingEventFilterSpec.scala:80-84
  - A3. MDC: el evento puede tener mas entradas que las pedidas; faltar alguna o MDC vacio no coincide — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/LoggingEventFilterSpec.scala:86-98
  - A4. causa: sin causa no coincide, causa no relacionada no coincide, causa del tipo coincide, causa + mensaje — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/LoggingEventFilterSpec.scala:102-120
  - A5. WARN con/sin causa y por mensaje — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/LoggingEventFilterSpec.scala:124-135
  - A6. `withSource`: coincide con el MDC `pekkoSource` igual y no con otro — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/LoggingEventFilterSpec.scala:136-144
  - A7. Java: mensaje, causa (`error(TestException.class)`, `withCause(Class)`) y `custom(event -> ...)` — actor-testkit-typed/src/test/java/org/apache/pekko/actor/testkit/typed/javadsl/LoggingTestKitTest.java:60-90
  - A8. `messageRegex` (busqueda, no coincidencia total), `message == null`, subclases en `cause`, y combinacion de condiciones: *no afirmada por spec*. Caracterizacion propuesta (de actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/LoggingTestKitImpl.scala:61-71): `withMessageRegex("warn.*dead")` coincide con "a warn about dead letters"; `withCause[RuntimeException]` coincide con `IllegalStateException`.
- **Oraculo / Validacion prevista**: test portado A1-A7 (predicado puro, sin logging) + caracterizacion A8.
- **Decisiones**: Q-T-TK-2, Q-T-LOG-1.

## AT-TK-006.2: LoggingTestKit: fabricas (error, warn, info, debug, trace, messageContains, custom, deadLetters, empty) (public)

- **Contrato observable**: Contractual: `empty` (sin condiciones: coincide con todo evento, `occurrences = 1`); `messageContains(s)`; `trace/debug/info/warn/error(s)` = `messageContains(s).withLogLevel(L)`; `warn[A]`/`error[A]` = `empty.withLogLevel(L).withCause[A]`; `custom(f)` = `empty.withCustom(f)`; `deadLetters()` = `empty.withLogLevel(INFO).withMessageRegex(".*was not delivered.*dead letters encountered.*")`. Java: `error(Class)`, `warn(Class)`. La documentacion de `empty` dice "no coincide con ningun evento" pero la implementacion coincide con todos.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/LoggingTestKit.scala:130-225; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/javadsl/LoggingTestKit.scala
- **Precondiciones**: ninguna.
- **Comportamiento**: `deadLetters()` solo coincide con eventos de un mensaje multilinea si `.` no cruza saltos de linea (regex Scala sin `DOTALL`).
- **Errores**: ninguno.
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: AT-TK-006.1; AT-EVT-002 (formato del log de dead letters).
- **Equivalente Rust propuesto**: funciones asociadas `LoggingTestKit::{empty, message_contains, trace, debug, info, warn, error, warn_of::<E>, error_of::<E>, custom, dead_letters}`.
- **Aserciones**:
  - A1. `error[AnError]`, `error("...").withCause[AnError]`, `warn("this is a warning")` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/LoggingEventFilterSpec.scala:102-135
  - A2. uso real de `info/error(msg)` con `withOccurrences` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1473-1485
  - A3. `deadLetters()` envuelve un bloque con mailbox lleno — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/MailboxSelectorSpec.scala:90-93
  - A4. `empty`, `trace`, `debug`, `custom` por fabrica, y el patron exacto de `deadLetters`: *no afirmada por spec* (solo `custom` en Java). Caracterizacion propuesta: `deadLetters().matches(info("Message [X] to ... was not delivered. [1] dead letters encountered"))` es verdadero; `empty.matches(cualquier_evento)` es verdadero.
- **Oraculo / Validacion prevista**: test portado A1; caracterizacion A4.
- **Decisiones**: Q-T-TK-2.

## AT-TK-006.3: LoggingTestKit: expect (ocurrencias, exceso, plazos y ausencia) (public)

- **Contrato observable**: Contractual: `expect(code)(implicit system)` (Java `expect(system, Supplier)`; `intercept` deprecado, alias) instala el filtro para el logger `loggerName` (por defecto raiz), ejecuta `code`, espera hasta `filter-leeway` (dilatado, 3 s) a que se alcance `occurrences` coincidencias y devuelve el resultado de `code`; retira SIEMPRE el filtro y reinicia el contador (`finally`). `withOccurrences(0)` comprueba ausencia; `occurrences = Int.MaxValue` acepta cualquier numero. Con `occurrences > 0` solo cuenta lo sincrono + lo que llegue durante la espera hasta completar; con `occurrences = 0` ademas espera `expect-no-message-default` por excesos asincronos. `withCheckExcess(false)` desactiva el error por exceso.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/LoggingTestKitImpl.scala:58-59,79-134; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/LoggingTestKit.scala:37-47,89-92,104-121; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/TestKitSettings.scala:88
- **Precondiciones**: backend de logging con soporte (Logback en Pekko, `IllegalStateException("LoggingEventFilter requires logback-classic dependency in classpath.")` si no esta).
- **Comportamiento**: cada evento que coincide decrementa `todo` (salvo `todo == Int.MaxValue`); `awaitDone(leeway)` (solo espera si `todo > 0`; `TestKit.awaitCond(todo <= 0, max, noThrow = true)`) devuelve `todo == MaxValue || todo == 0`; si `occurrences == 0` primero `awaitNoExcess(expect-no-message-default)` (si `todo == 0` espera hasta que `todo < 0`; resultado ignorado). Si `awaitDone` es falso: `todo > 0` -> falta de eventos; si no y `checkExcess` -> exceso; si no y no `checkExcess` -> sin error.
- **Errores**: `AssertionError("Timeout (<leeway>) waiting for <todo> messages on <filtro>.")` (el filtro se imprime con el `toString` de la case class `LoggingTestKitImpl(...)`); `AssertionError("Received <n> excess messages on <filtro>.")`; si `code` lanza, la excepcion se propaga sin evaluar conteos (el filtro se retira igualmente).
- **Orden / concurrencia / ciclo de vida**: los eventos llegan desde cualquier hilo (el appender los entrega sincronicamente en el hilo que loguea); `todo` es `@volatile`; el exceso asincrono solo se detecta con `occurrences = 0`.
- **Dependencias**: AT-TK-006.1; AT-TK-006.4; AT-TK-010 (`filter-leeway`, `expect-no-message-default`); AC-TK-001.5 (`awaitCond`).
- **Equivalente Rust propuesto**: `LoggingTestKit::expect(system, || ..) -> T` (o `expect_async(fut)`), apoyado en una capa de captura global (AT-TK-006.4). Se preservan: ocurrencias exactas, exceso, plazo `filter-leeway`, ausencia con espera adicional y retiro en `finally` (RAII/Drop). Limite: la "excepcion de `code`" en Rust es un panic; usar un guard que retire el filtro.
- **Aserciones**:
  - A1. 2 errores esperados `withOccurrences(2)` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestAppenderSpec.scala:39-44
  - A2. causa + mensaje dentro de `expect` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestAppenderSpec.scala:46-50
  - A3. solo cuentan las coincidencias (`warn` entre `error`) — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestAppenderSpec.scala:52-59
  - A4. exceso sincrono: 3 warn con `withOccurrences(2)` -> `AssertionError` que incluye `Received 1 excess messages` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestAppenderSpec.scala:61-74
  - A5. `withOccurrences(0)`: sin coincidencia no falla; una coincidencia -> `Received 1 excess messages`; dos -> `Received 2 excess messages` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestAppenderSpec.scala:94-115
  - A6. exceso asincrono con `withOccurrences(0)` (log tras 20 ms, `expect-no-message-default = 1000 ms`) -> `AssertionError` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestAppenderSpec.scala:117-128
  - A7. uso real: `withOccurrences(3)` y `(2)` sobre supervision, `withOccurrences(0)` con `withSource` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1452-1485
  - A8. el mensaje `Timeout (...) waiting for <n> messages on ...`, `withCheckExcess(false)`, `Int.MaxValue`, retiro del filtro cuando `code` lanza, y el error de dependencia ausente: *no afirmada por spec*. Caracterizacion propuesta (de actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/LoggingTestKitImpl.scala:98-121): `error("x").expect { }` sin logs -> `AssertionError` que empieza con `Timeout (3 seconds) waiting for 1 messages on LoggingTestKitImpl(`; `withCheckExcess(false)` con 3 warn y `withOccurrences(2)` no falla.
- **Oraculo / Validacion prevista**: test portado A1-A7 (necesita la capa de captura) + caracterizacion A8; los plazos (`3 s` por defecto) se acortan con `filter-leeway` en la config del test.
- **Decisiones**: Q-T-TK-2, Q-T-LOG-1, Q-X-7.

## AT-TK-006.4: LoggingTestKit: captura de eventos (TestAppender) y logger name (public)

- **Contrato observable**: Contractual: la captura es un appender por logger (`PekkoTestAppender`), instalado en forma perezosa e idempotente sobre el logger `loggerName` (raiz si vacio) al llamar `expect`; cada evento que llega a ese logger (incluidos los de loggers hijos por la jerarquia de nombres y aditividad, y sujeto al nivel configurado del logger) se convierte en `LoggingEvent(level, message formateado, loggerName, threadName, timeStamp, marker = primer marker, throwable, mdc)` y se ofrece a los filtros registrados: `filters.exists(f.apply(event))`, por lo que cada evento lo cuenta COMO MAXIMO el filtro mas recientemente registrado que coincida; una excepcion lanzada por un filtro (p.ej. la funcion `custom`) cuenta como "no coincide". El MDC es el del evento o, si esta vacio, el `MDC` del hilo que loguea. Niveles soportados: TRACE, DEBUG, INFO, WARN, ERROR (otro -> `IllegalArgumentException("Level <str>, <int> is unknown.")`).
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestAppender.scala:35-137; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/LoggingTestKitImpl.scala:98-121; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/LogbackUtil.scala:25-59
- **Precondiciones**: Logback como implementacion de SLF4J.
- **Comportamiento**: `setupTestAppender(name)`: si ya hay un `TestAppender` lo reutiliza; si hay otro appender con el mismo nombre -> error. `addFilter` antepone (`filters ::= filter`); `removeFilter` quita la primera aparicion (por igualdad) y si no existe no hace nada. `append` esta serializado por `doAppend`.
- **Errores**: `IllegalStateException("Unexpected PekkoTestAppender already added: <otro>")`; `IllegalStateException("No PekkoTestAppender was setup for logger [<name>]")` / `Unexpected PekkoTestAppender already added for logger [<name>]: <otro>` en `getTestAppender`.
- **Orden / concurrencia / ciclo de vida**: el hilo que loguea ejecuta el filtro; el appender nunca se desinstala (solo los filtros).
- **Dependencias**: AT-TK-006.1; AT-TK-006.3; AT-LOG-001; AT-LOG-002; AT-TK-013 (`LogbackUtil`).
- **Equivalente Rust propuesto**: efecto observable a preservar: captura por sistema de eventos de log con nivel, mensaje ya formateado, nombre de logger (`target`), MDC (campos), causa, marcador, hilo y marca de tiempo; filtros dinamicos por logger; conteo de ocurrencias; plazo y verificacion de exceso. Representacion propuesta: una `tracing_subscriber::Layer` de captura instalada UNA vez por proceso (`set_global_default` perezoso con `Once`), con un registro de filtros activos (`Mutex<Vec<Arc<Filter>>>`, el mas reciente primero) y `on_event` que arma el `LogEvent` (nivel de `Metadata::level`, `target` como logger, mensaje del campo `message`, campos y campos de spans como MDC, `record_error` para la causa, hilo `std::thread::current().name()`), aplicando `exists` y cortocircuito de la misma forma. Limites: (1) los nombres jerarquicos de SLF4J (`a.b.C`) corresponden a `target` con `::` o `.`: hay que normalizar y comparar por prefijo de segmento; (2) un subscriber por hilo (`with_default`) no ve eventos de tareas en otros hilos: se necesita el global; (3) el filtro de nivel global debe permitir DEBUG (equivalente al `root level="DEBUG"` de logback-test.xml) o los eventos no llegan al layer; (4) sin throwable ni jerarquia de excepciones (ver AT-TK-006.1); (5) sin appender que cuelgue de un logger concreto: la seleccion por logger se emula en el layer.
- **Aserciones**:
  - A1. solo se filtran eventos del logger dado: `custom` cuenta 2 llamadas y el evento de otro logger (`log.info("Hello wrong logger")`) no llega al filtro — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestAppenderSpec.scala:76-92
  - A2. nombre de logger por jerarquia en codigo de produccion: `withLoggerName("org.apache.pekko.actor.LocalActorRefProvider.Deserialization")` — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/LocalActorRefProviderLogMessagesSpec.scala:40-49
  - A3. MDC decorado (`withMdc`) visible a traves del appender — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/LogMessagesSpec.scala:94-108
  - A4. `exists` (solo el filtro mas reciente cuenta), excepcion de un filtro = no coincide, `marker`, `threadName` y `timeStamp`: *no afirmada por spec*. Caracterizacion propuesta (de actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestAppender.scala:84-120): dos `expect` anidados con filtros que coinciden con el mismo evento -> solo el interno cuenta el evento; `custom` que lanza -> el evento no cuenta.
- **Oraculo / Validacion prevista**: test portado A1-A3 (captura real) + caracterizacion A4. No hay oraculo directo: Logback no existe en Rust; se valida por el efecto observable.
- **Decisiones**: Q-T-TK-2, Q-T-LOG-1, Q-X-7.

## AT-TK-006.5: LogCapturing (ScalaTest, JUnit4) y CapturingAppender (public)

- **Contrato observable**: Contractual: `LogCapturing` (trait ScalaTest `BeforeAndAfterAll` con `withFixture`; regla JUnit4 `TestRule` en javadsl) silencia el log de un test y SOLO lo vuelca si el test falla: el `CapturingAppender` de la raiz acumula eventos en un buffer; antes de cada test registra `Logging started for test [<clase>: <test>]` y despues `Logging finished ... that [<resultado>]`; si el resultado no es exito ni pendiente imprime `--> [<test>] Start of log messages of test that [<res>]`, vuelca el buffer a los appenders del logger `org.apache.pekko.actor.testkit.typed.internal.CapturingAppenderDelegate` (p.ej. STDOUT) y `<-- ... End of log messages ...`; `clearLogsAfterEachTest` (false por defecto en Scala; la regla JUnit limpia siempre) y `clearCapturedLogs()`; `afterAll` limpia, y si `super.afterAll()` lanza loguea ERROR y vuelca. Requiere `logback-test.xml` con el `CapturingAppender` en la raiz.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/LogCapturing.scala:42-96; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/javadsl/LogCapturing.scala:47-77; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/CapturingAppender.scala:25-100; actor-testkit-typed/src/test/resources/logback-test.xml:1-34
- **Precondiciones**: appender `CapturingAppender` configurado (fail-fast: `CapturingAppender.get("")` en la inicializacion).
- **Comportamiento**: `append` llama `prepareForDeferredProcessing` y acumula; `flush` reenvia cada evento a los appenders del delegado (excluyendo a si mismo) y limpia; `clear` descarta.
- **Errores**: `IllegalStateException("CapturingAppender not defined for [<logger>] in logback-test.xml")` si falta; `IllegalStateException("Unexpected <CapturingAppender>: <otro>")` si el nombre esta ocupado por otro appender.
- **Orden / concurrencia / ciclo de vida**: `flush`/`clear` sincronizados; el buffer es global por proceso (no por test) y se vacia por test en la regla JUnit.
- **Dependencias**: AT-TK-013 (`LogbackUtil`); AT-TK-006.4.
- **Equivalente Rust propuesto**: la salida de `cargo test` ya oculta stdout/stderr de los tests que pasan, de modo que el efecto observable (log visible solo si falla) se obtiene con `tracing_subscriber::fmt().with_test_writer()`; para el caso de hilos que escapan de la captura de libtest, una capa que bufferiza en memoria y un guard RAII cuyo `Drop` vuelca el buffer cuando `std::thread::panicking()`. Se pierde: el reporte con los bloques `-->`/`<--` y el volcado por resultado `Failed/Pending` (libtest no expone el resultado al guard); el texto de los mensajes de inicio/fin se puede conservar si se desea.
- **Aserciones**:
  - A1. los tests de todo el modulo se ejecutan con `LogCapturing` mezclado sin romper (uso, no comportamiento de fallo) — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestAppenderSpec.scala:25-32; actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitSpec.scala:32
  - A2. ejemplos documentales ScalaTest y JUnit4 — actor-testkit-typed/src/test/scala/docs/org/apache/pekko/actor/testkit/typed/scaladsl/ScalaTestIntegrationExampleSpec.scala:48-60; actor-testkit-typed/src/test/java/jdocs/org/apache/pekko/actor/testkit/typed/javadsl/LogCapturingExampleTest.java:28-41
  - A3. el volcado solo en fallo, `clear` por test y los mensajes `Logging started/finished`: *no afirmada por spec* (ningun test del repositorio falla a proposito). Caracterizacion propuesta: test que loguea y falla -> el buffer se vuelca una vez en el delegado y queda vacio; test que pasa -> nada se vuelca y el buffer queda vacio (de actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/LogCapturing.scala:77-95).
- **Oraculo / Validacion prevista**: A1-A2 solo como smoke; A3 por test propio de Rust con un buffer inyectable (no hay oraculo).
- **Decisiones**: Q-T-TK-2, Q-X-7.

## AT-TK-007: ManualTime (scheduler disparado explicitamente) (public)

- **Contrato observable**: Contractual: `ManualTime.config` fija `pekko.scheduler.implementation = "org.apache.pekko.testkit.ExplicitlyTriggeredScheduler"`; `ManualTime()` (Java `ManualTime.get(system)`) exige que el scheduler del sistema sea el adaptador clasico con `ExplicitlyTriggeredScheduler`; `timePasses(amount)` avanza el reloj logico y ejecuta en el hilo llamante todas las tareas vencidas, en orden de tiempo (semantica de AC-TK-008) SIN dilatar `amount`; `expectNoMessageFor(duration, probes*)` avanza el tiempo y exige `expectNoMessage(Duration.Zero)` en cada probe.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ManualTime.scala:31-81; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/javadsl/ManualTime.scala; testkit/src/main/scala/org/apache/pekko/testkit/ExplicitlyTriggeredScheduler.scala (frontera: AC-TK-008)
- **Precondiciones**: crear el sistema con `ManualTime.config` (p.ej. `ScalaTestWithActorTestKit(ManualTime.config)`).
- **Comportamiento**: `expectNoMessageFor` primero avanza y luego comprueba, de modo que un mensaje entregado durante el avance se detecta como `Received unexpected message` (AT-TK-002.1).
- **Errores**: `IllegalArgumentException("ActorSystem not configured with explicitly triggered scheduler, make sure to include org.apache.pekko.actor.testkit.typed.scaladsl.ManualTime.config() when setting up the test")` si el scheduler clasico no es el explicito; `IllegalArgumentException("ActorSystem.scheduler is not a classic SchedulerAdapter but a <clase>, this is not supported")` si el scheduler tipado no es `SchedulerAdapter`.
- **Orden / concurrencia / ciclo de vida**: `timePasses` ejecuta las tareas en el hilo llamante; los timers de actores enviaran a mailboxes reales, luego la entrega es asincrona (por eso el patron `expectNoMessageFor`/`expectMessage`).
- **Dependencias**: AC-TK-008 (ExplicitlyTriggeredScheduler); AC-SCH-010; AT-TMR-002; AT-TMR-003; AT-TK-002.1; AT-SYS-009.
- **Equivalente Rust propuesto**: reloj inyectable (`trait Clock`) en el scheduler y `ManualTime::time_passes(Duration)` / `expect_no_message_for(duration, probes)`; `ManualTime::config()` como opcion del builder del sistema (Q-T-TMR-4, Q-X-4). Se preservan: avance explicito, ejecucion en el hilo llamante, orden por tiempo y los dos errores.
- **Aserciones**:
  - A1. temporizador unico de 10 ms: `expectNoMessageFor(9ms)` sin mensaje, `timePasses(2ms)` entrega `Tock`, `expectNoMessageFor(10s)` ya sin mensajes — actor-testkit-typed/src/test/scala/docs/org/apache/pekko/actor/testkit/typed/scaladsl/ManualTimerExampleSpec.scala:34-55
  - A2. `startTimerWithFixedDelay(10ms)` repetido 5 veces: cada ciclo `expectNoMessageFor(9ms)` y `timePasses(1ms)` -> `Tock` — actor-testkit-typed/src/test/scala/docs/org/apache/pekko/actor/testkit/typed/scaladsl/ManualTimerExampleSpec.scala:58-79
  - A3. reemplazo de timer por clave: el `Tick(1)` ya encolado se descarta y el nuevo timer dispara `Tock(2)` pasados 9+2 ms — actor-testkit-typed/src/test/scala/docs/org/apache/pekko/actor/testkit/typed/scaladsl/ManualTimerExampleSpec.scala:81-124
  - A4. Java: `ManualTime.get(testKit.system())`, `expectNoMessageFor(Duration, probe)`, `timePasses` — actor-testkit-typed/src/test/java/jdocs/org/apache/pekko/actor/testkit/typed/javadsl/ManualTimerExampleTest.java:49-72
  - A5. los dos mensajes de `IllegalArgumentException` y la no dilatacion de `timePasses`: *no afirmada por spec*. Caracterizacion propuesta: `ManualTime()` sobre un sistema sin `ManualTime.config` -> IAE con el primer texto; `timePasses(1s)` con `timefactor = 2` avanza 1 s.
- **Oraculo / Validacion prevista**: test portado A1-A4 (la logica de planificacion corresponde a AC-TK-008); caracterizacion A5 diferencial (jshell con el scheduler explicito).
- **Decisiones**: Q-X-4, Q-T-TMR-4.

## AT-TK-008: Probes de eventos del sistema: createUnhandledMessageProbe, createDeadLetterProbe, createDroppedMessageProbe (public)

- **Contrato observable**: Contractual: crean una TestProbe suscripta al event stream tipado (EventStream.Subscribe) para UnhandledMessage, DeadLetter y Dropped; el probe verifica la suscripcion publicando un evento de prueba (dummyMessage DeadLetterSuppression) con awaitAssert + fishForMessage hasta ver ese mismo evento (eq), de modo que al retornar la suscripcion esta activa.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKit.scala:286-322
- **Condiciones**: Dropped de prueba: Dropped(dummyMessage, "no reason", deadLetters, deadLetters).
- **Orden / concurrencia / ciclo de vida**: El constructor bloquea hasta confirmar la suscripcion.
- **Dependencias**: filas: AT-EVT-001; AT-EVT-002; AT-EVT-003; AT-EVT-004. Externas: Runtime de actores, scheduler clasico (ExplicitlyTriggeredScheduler), Logback (LoggingTestKit), ScalaTest/JUnit.
- **Equivalente Rust propuesto**: ActorTestKit::create_unhandled_probe/dead_letter_probe/dropped_probe. **Diferencias observables**: Ninguna.
- **Aserciones**:
  - A1. createUnhandledMessageProbe recibe UnhandledMessage("message", ...) — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitSpec.scala:133-139
  - A2. createDeadLetterProbe recibe el mensaje enviado a system.deadLetters — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitSpec.scala:140-145
  - A3. createDroppedMessageProbe recibe Dropped("message","it had gone bad", noSender) — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitSpec.scala:146-151
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-T-EVT-1

## AT-TK-009.1: ScalaTestWithActorTestKit, ScalaTestWithActorTestKitBase y ActorTestKitBase (public)

- **Contrato observable**: Contractual: `ScalaTestWithActorTestKit` (clase abstracta) crea UN `ActorTestKit` por suite, con nombre tomado de la clase de test (`ActorTestKitBase.testNameFromCallStack()`), lo expone como `testKit` y lo apaga en `afterAll`. Constructores: `()` (nombre + `application-test.conf`), `(system)`, `(config: String)`, `(config: Config)`, `(config, settings)` (config explicita => sin `application-test.conf`). `ScalaTestWithActorTestKitBase` (trait) agrega `Matchers`, `BeforeAndAfterAll`, `ScalaFutures`, `Eventually` y un `PatienceConfig(timeout = DefaultTimeout, interval = 100 ms)` implicito; `ActorTestKitBase` delega `system`, `testKitSettings`, `timeout` (implicitos), `spawn` (4), `createTestProbe` (2), las tres probes de eventos y `serializationTestKit`.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ScalaTestWithActorTestKit.scala:41-105; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitBase.scala:31-33,44-138
- **Precondiciones**: ninguna.
- **Comportamiento**: `afterAll`: `super.afterAll()` y luego `testKit.shutdownTestKit()`.
- **Errores**: los de `shutdownTestKit` (AT-TK-001.3).
- **Orden / concurrencia / ciclo de vida**: el sistema se crea al construir la suite y se apaga una vez; los tests comparten estado.
- **Dependencias**: AT-TK-001.1; AT-TK-001.2; AT-TK-001.3; AT-TK-008; AT-TK-009.4.
- **Equivalente Rust propuesto**: fixture/RAII `TestKitFixture` (o macro `#[actor_test]` sobre `#[tokio::test]`) que crea el `ActorTestKit` y lo apaga en `Drop`; la paciencia de `Eventually` se sustituye por `await_assert`. No se porta ScalaTest ni `Matchers`; se conserva el comportamiento "un testkit por suite, apagado al final, nombre desde el test, config explicita sustituye a `application-test`". Detalle en Q-T-TK-3.
- **Aserciones**:
  - A1. el nombre del sistema es el de la clase de test, tambien en clases derivadas — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitSpec.scala:36-38,161-170
  - A2. `new ScalaTestWithActorTestKit("test.specific-config = yes") {}` no carga `application-test.conf` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitSpec.scala:114-124
  - A3. `spawn` delegado y `futureValue` (usa la paciencia) — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitSpec.scala:82-100
  - A4. ejemplos documentales: suite basica, con `LogCapturing`, y `ScalaTestWithActorTestKitBase` extendido varias veces (`TestWithOneImplementation`/`TestWithAnotherImplementation`) — actor-testkit-typed/src/test/scala/docs/org/apache/pekko/actor/testkit/typed/scaladsl/ScalaTestIntegrationExampleSpec.scala:34-78
  - A5. `PatienceConfig(DefaultTimeout, 100ms)` y apagado en `afterAll`: *no afirmada por spec*. Caracterizacion propuesta: con `default-timeout = 100ms`, `futureValue` de un future que tarda 300 ms falla; el sistema esta terminado tras `afterAll`.
- **Oraculo / Validacion prevista**: A1-A4 como tests de fixture en Rust; no hay oraculo (integracion de framework JVM).
- **Decisiones**: Q-T-TK-3.

## AT-TK-009.2: TestKitJunitResource (JUnit 4) (public)

- **Contrato observable**: Contractual: `TestKitJunitResource` (`ExternalResource`) crea un `ActorTestKit` (nombre desde la pila de llamadas, o `(system)`, `(String config)`, `(Config)`, `(Config, settings)`), lo expone en el campo publico `testKit` anotado `@Rule` y lo apaga en `after()`; usable como `@ClassRule` estatico. Delega `system`, `testKitSettings`, `timeout`, `scheduler`, `spawn` (4), `createTestProbe` (4 con `Class`/nombre), `stop` (con/sin `Duration`), las probes de eventos y `serializationTestKit`.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/javadsl/TestKitJunitResource.scala:62-195
- **Precondiciones**: JUnit 4.
- **Comportamiento**: `after()` -> `testKit.shutdownTestKit()`.
- **Errores**: los de `shutdownTestKit`.
- **Orden / concurrencia / ciclo de vida**: como `@ClassRule` un sistema para toda la clase; como `@Rule` uno por test.
- **Dependencias**: AT-TK-001.1; AT-TK-001.3.
- **Equivalente Rust propuesto**: mismo fixture que AT-TK-009.1 (no hay JUnit); se documenta como fachada Java equivalente.
- **Aserciones**:
  - A1. nombre del sistema = nombre de la clase de test via recurso JUnit — actor-testkit-typed/src/test/java/org/apache/pekko/actor/testkit/typed/javadsl/ActorTestKitTest.java:32-39
  - A2. `testKit.spawn(Behaviors.setup(..))` ejecuta el setup — actor-testkit-typed/src/test/java/org/apache/pekko/actor/testkit/typed/javadsl/ActorTestKitTest.java:62-72
  - A3. uso como `@ClassRule` con `LogCapturing` y probes — actor-testkit-typed/src/test/java/org/apache/pekko/actor/testkit/typed/javadsl/TestProbeTest.java:29-48; actor-testkit-typed/src/test/java/jdocs/org/apache/pekko/actor/testkit/typed/javadsl/JunitIntegrationExampleTest.java:28-45
  - A4. apagado en `after()`: *no afirmada por spec*. Caracterizacion propuesta: tras la clase de test, `testKit.system().whenTerminated` esta completado.
- **Oraculo / Validacion prevista**: no aplica (integracion JVM); A1-A3 quedan cubiertos por el fixture de AT-TK-009.1.
- **Decisiones**: Q-T-TK-3.

## AT-TK-009.3: TestKitJUnit5Extension, @JUnit5TestKit y JUnit5TestKitBuilder (JUnit 5) (public)

- **Contrato observable**: Contractual: `JUnit5TestKitBuilder` construye un `ActorTestKit`: `withName(n)` (por defecto el de la pila de llamadas al construir el builder), `withCustomConfig(c)` (por defecto `ApplicationTestConfig`; sin `withFallback`), `withSystem(s)` (usa `ActorTestKit.create(system)` e ignora nombre y config); `build()`. `TestKitJUnit5Extension` (`BeforeTestExecutionCallback` + `AfterAllCallback`) busca en la instancia de test el primer campo anotado `@JUnit5TestKit` (anotacion en campo, retencion runtime) y lo guarda; en `afterAll` apaga el testkit guardado.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/javadsl/JUnit5TestKitBuilder.scala:26-56; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/javadsl/TestKitJUnit5Extension.scala:24-49; actor-testkit-typed/src/main/java/org/apache/pekko/actor/testkit/typed/annotations/JUnit5TestKit.java:14-25
- **Precondiciones**: JUnit 5; campo anotado inicializado.
- **Comportamiento**: la extension sobrescribe su `testKit` en cada `beforeTestExecution`; con ciclo de vida por metodo (instancia nueva por test) cada instancia crea su propio sistema pero `afterAll` solo apaga el ultimo registrado (los anteriores no se apagan por la extension; derivado del codigo).
- **Errores**: sin campo anotado -> `IllegalArgumentException("Could not find field annotated with @JUnit5TestKit")`; `afterAll` sin testkit registrado -> `NoSuchElementException` (`Option.get`).
- **Orden / concurrencia / ciclo de vida**: callbacks de JUnit 5 en orden por test.
- **Dependencias**: AT-TK-001.1; AT-TK-001.3.
- **Equivalente Rust propuesto**: builder `ActorTestKitBuilder::new().name(..).config(..).system(..)` con la misma precedencia (`system` anula el resto); extension JUnit sustituida por el fixture de AT-TK-009.1. Se preserva la precedencia del builder y el nombre por defecto.
- **Aserciones**:
  - A1. `withName("hello")` -> nombre `hello` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/JUnit5TestKitBuilderSpec.scala:28-34
  - A2. sin nombre, el de la clase de test (`JUnit5TestKitBuilderSpec`) — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/JUnit5TestKitBuilderSpec.scala:36-43
  - A3. `withCustomConfig(application.conf)`: `test.value == "someValue"` y nombre por defecto — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/JUnit5TestKitBuilderSpec.scala:45-56
  - A4. config y nombre juntos — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/JUnit5TestKitBuilderSpec.scala:58-70
  - A5. `withSystem(system)`: el nombre es el del sistema dado (`AkkaQuickStart`) — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/JUnit5TestKitBuilderSpec.scala:72-82
  - A6. ejemplos JUnit 5: `@ExtendWith(TestKitJUnit5Extension.class)` + `@JUnit5TestKit` + builder; nombre del sistema `ActorTestKitJUnit5Test` — actor-testkit-typed/src/test/java/jdocs/org/apache/pekko/actor/testkit/typed/javadsl/JUnit5IntegrationExampleTest.java:29-60; actor-testkit-typed/src/test/java/org/apache/pekko/actor/testkit/typed/javadsl/ActorTestKitJUnit5Test.java:29-73
  - A7. error por falta de campo anotado, `afterAll` y fuga de sistemas por instancia: *no afirmada por spec*. Caracterizacion propuesta: extension sobre una clase sin campo `@JUnit5TestKit` -> IAE con el texto indicado.
- **Oraculo / Validacion prevista**: A1-A5 como tests del builder de Rust; A6-A7 no portables (JUnit).
- **Decisiones**: Q-T-TK-3.

## AT-TK-009.4: SerializationTestKit (verifySerialization) (public)

- **Contrato observable**: Contractual: `new SerializationTestKit(system)` (tambien `ActorTestKit.serializationTestKit`) ofrece `verifySerialization(obj)` = `verifySerialization(obj, assertEquality = true)`; hace el ciclo completo con la `Serialization` clasica del sistema: `serialize(obj)`, `findSerializerFor(obj)`, `Serializers.manifestFor`, `deserialize(bytes, serializer.identifier, manifest)` y devuelve el objeto deserializado; si `assertEquality` y el resultado no es igual (`!=`) lanza `AssertionError("Serialization verification expected <obj>, but was <resultado>")`; los fallos del serializador se propagan tal cual.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/SerializationTestKit.scala:25-70; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKit.scala:324-327; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/javadsl/SerializationTestKit.scala
- **Precondiciones**: serializer configurado para la clase (AC-SER-001.1).
- **Comportamiento**: sin transporte de red; solo serializacion local.
- **Errores**: `AssertionError` de igualdad; excepciones del serializador (`Try.get`).
- **Orden / concurrencia / ciclo de vida**: sincronico.
- **Dependencias**: AC-SER-001.1; AC-SER-001.3; AC-SER-002; AT-SER-001.
- **Equivalente Rust propuesto**: `SerializationTestKit::verify_serialization(&obj) -> obj` y `..._with(obj, assert_equality)` sobre el registro de serializadores de Rust (Q-T-SER-1); requiere `PartialEq + Debug`. Se preservan la ruta completa (serializer id + manifest) y el texto del error.
- **Aserciones**:
  - A1. uso en pruebas de Jackson: `verifySerialization(obj)` devuelve un objeto igual (uso externo, frontera persistence-typed-tests) — persistence-typed-tests/src/test/scala/org/apache/pekko/persistence/typed/jackson/ReplicatedEventSourcingJacksonSpec.scala:47-55
  - A2. uso por `EventSourcedBehaviorTestKit` para verificar comandos/eventos/estado (frontera persistence-testkit) — persistence-testkit/src/main/scala/org/apache/pekko/persistence/testkit/internal/EventSourcedBehaviorTestKitImpl.scala:134,205-225
  - A3. texto de `AssertionError`, `assertEquality = false`, propagacion de errores del serializador: *no afirmada por spec* en `actor-testkit-typed`. Caracterizacion propuesta: objeto con `equals` roto -> `Serialization verification expected X, but was Y`; con `assertEquality = false` no falla.
- **Oraculo / Validacion prevista**: caracterizacion A3 diferencial con el jar testkit-typed + serializador de pruebas.
- **Decisiones**: Q-T-SER-1, Q-T-TK-3.

## AT-TK-010: TestKitSettings y pekko.actor.testkit.typed.* (timefactor, timeouts) (public)

- **Contrato observable**: Contractual: `TestKitSettings(system)` (extension privada `Ext`, una por sistema) o `TestKitSettings(config)` leen la seccion `pekko.actor.testkit.typed`: `timefactor = 1.0` (finito y > 0), `single-expect-default = 3s`, `expect-no-message-default = 100ms`, `default-timeout = 5s` (`Timeout` implicito), `system-shutdown-default = 10s`, `throw-on-shutdown-timeout = true`, `filter-leeway = 3s`; todos los plazos (menos el booleano) se dilatan: `dilated(d) = Duration.fromNanos((d.toNanos * timefactor + 0.5).toLong)`. Los plazos se leen con `getMillisDuration` (milisegundos enteros, truncados).
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/TestKitSettings.scala:27-101; actor-testkit-typed/src/main/resources/reference.conf:9-39
- **Precondiciones**: `timefactor` > 0 y finito.
- **Comportamiento**: `timefactor` se lee con `getDouble`; el redondeo es +0.5 y truncado (round-half-up para positivos). Ver la tabla de operaciones que dilatan en AT-TK-002.6.
- **Errores**: `timefactor` invalido -> `IllegalArgumentException("requirement failed: timefactor must be positive finite double")` (`Helpers.Requiring`, AC-UTIL-002.3); clave ausente o con tipo incorrecto -> `ConfigException` (AC-CFG-017).
- **Orden / concurrencia / ciclo de vida**: inmutable tras construirse.
- **Dependencias**: AC-CFG-001; AC-UTIL-002.3; AC-UTIL-002.4 (`getMillisDuration`); AC-UTIL-001 (`Timeout`).
- **Equivalente Rust propuesto**: `struct TestKitSettings{test_time_factor, single_expect_default, expect_no_message_default, default_timeout, system_shutdown_default, throw_on_shutdown_timeout, filter_leeway}` con `dilated()`; parser HOCON propio (Q-X-1); duraciones con el dominio de Pekko (Q-UTIL-1 rechazada: se preserva el dominio de `FiniteDuration`).
- **Aserciones**:
  - A1. `default-timeout = 100ms` (y `pekko.test.default-timeout = 100ms`) hace fallar `receiveMessage()` sin mensaje — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProbeSpec.scala:182-184,194-207
  - A2. valores por defecto de `reference.conf` y su dilatacion: *no afirmada por spec*. Caracterizacion propuesta: `TestKitSettings(ConfigFactory.defaultReference.getConfig("pekko.actor.testkit.typed"))` -> `SingleExpectDefaultTimeout == 3.seconds`, `ExpectNoMessageDefaultTimeout == 100.millis`, `DefaultTimeout.duration == 5.seconds`, `DefaultActorSystemShutdownTimeout == 10.seconds`, `ThrowOnShutdownTimeout == true`, `FilterLeeway == 3.seconds` (de actor-testkit-typed/src/main/resources/reference.conf:9-39).
  - A3. `dilated(d)` y el error de `timefactor` invalido: *no afirmada por spec*. Caracterizacion propuesta: `timefactor = 2` -> `dilated(3.seconds) == 6.seconds`; `timefactor = 0` o `Infinity` -> `IllegalArgumentException` con el texto indicado (de actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/TestKitSettings.scala:69-71,93-95).
- **Oraculo / Validacion prevista**: test portado A1 + caracterizacion A2/A3 diferencial (valores efectivos con el jar testkit-typed).
- **Decisiones**: Q-X-1, Q-T-TK-1.

## AT-TK-011: ActorSystemStub: ActorSystem minimo sin runtime real que usa BehaviorTestKit (internal-required)

- **Contrato observable**: [contractual interno] `ActorSystemStub(name, config)` es un `ActorSystem[Nothing]` (que tambien es el `ActorRef` raiz) sobre el que corre `BehaviorTestKit`: `path = pekko://<name>/user`; `address = pekko://<name>`; `settings` construidos desde `config` (con `amendSlf4jConfig`); `deadLetters` y `ignoreRef` devuelven el mismo `DebugRef` (`/deadLetters`); `receptionist` es un `TestInbox` (`/receptionist`); `executionContext` y `dispatchers.lookup(_)` devuelven el `ControlledExecutor`; `terminate()` completa `whenTerminated` con `Done` (y `close()` espera); `startTime`/`uptime` con `currentTimeMillis`; `printTree = "no tree for ActorSystemStub"`; `logConfiguration` loguea `settings.toString`; `log` = `LoggerFactory.getLogger(ActorSystemStub)`; `threadFactory` crea `Thread` simples. Las operaciones no soportadas lanzan `UnsupportedOperationException`.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/ActorSystemStub.scala:46-150
- **Precondiciones**: ninguna (se crea por `BehaviorTestKit.apply`; por defecto `config = ConfigFactory.defaultReference(classloader)`).
- **Comportamiento**: ver contrato; el stub se comparte entre el testkit raiz y los hijos (mismo receptionist, mismo executor, mismo dead letters).
- **Errores** (`UnsupportedOperationException`): `tell` -> `"must not send message to ActorSystemStub"`; `sendSystem` -> `"must not send SYSTEM message to ActorSystemStub"`; `provider` -> `"no provider"`; `classicSystem` -> `"no classic actor system available"`; `scheduler` -> `"no scheduler"`; `systemActorOf` -> `"ActorSystemStub cannot create system actors"`; `registerExtension`/`extension`/`hasExtension` -> `"ActorSystemStub cannot register extensions"`.
- **Orden / concurrencia / ciclo de vida**: `terminate` idempotente (`trySuccess`); `close` bloquea hasta terminado (inmediato).
- **Dependencias**: AT-TK-003.1; AT-TK-012; AT-SYS-001; AT-SYS-005; AC-CFG-002; Q-X-7 (`amendSlf4jConfig`).
- **Equivalente Rust propuesto**: `ActorSystemStub` que implementa el trait de sistema tipado con las operaciones no soportadas devolviendo un error/`panic!` con el MISMO texto; DebugRef y executor controlado como piezas compartidas (AT-TK-012). Se preservan el conjunto exacto de operaciones soportadas/no soportadas y los textos.
- **Aserciones**:
  - A1. `system.address == Address("pekko","StubbedActorContext")` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:277-280
  - A2. `receptionist` es un inbox que recibe el `Register` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:434-441
  - A3. `settings.config` refleja la config dada — actor-testkit-typed/src/test/scala/docs/org/apache/pekko/actor/testkit/typed/scaladsl/SyncTestingExampleSpec.scala:226-245
  - A4. el `askMessage` del ask reificado se envia con `replyTo = ignoreRef` (indirecto: el inbox recibe el mensaje y el efecto lo expone) — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:539-545
  - A5. los siete `UnsupportedOperationException`, `terminate`/`whenTerminated`/`close`, `deadLetters`/`ignoreRef`, `dispatchers.lookup`, `printTree`, `uptime`: *no afirmada por spec*. Caracterizacion propuesta (de actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/ActorSystemStub.scala:67-135): cada operacion no soportada lanza con el texto indicado; `terminate(); whenTerminated.isCompleted`; `deadLetters eq ignoreRef`.
- **Oraculo / Validacion prevista**: A1-A4 test portado; A5 caracterizacion diferencial con el jar testkit-typed (jshell sobre `ActorSystemStub`, clase `private[pekko]`: usar reflexion o `BehaviorTestKit`).
- **Decisiones**: Q-T-TK-4, Q-X-7, Q-X-8.

## AT-TK-012: ControlledExecutor y DebugRef (internal-required)

- **Contrato observable**: [contractual interno] `ControlledExecutor` (`ExecutionContextExecutor`) acumula `Runnable` en una `LinkedList` FIFO y NO los ejecuta: `queueSize`, `runOne()` (`pop().run()`) y `runAll()` (`while (!isEmpty) runOne()`, incluyendo tareas encoladas durante la ejecucion); `reportFailure` imprime la traza. `DebugRef[T](path, isLocal)` es un `ActorRef` que guarda en una `ConcurrentLinkedQueue[Either[SystemMessage, T]]` todo lo que recibe: `tell` -> `Right(msg)`, `sendSystem` -> `Left(signal)`; `hasMessage`/`hasSignal`/`hasSomething` miran la cabeza; `receiveMessage`/`receiveSignal`/`receiveAll`; `isTerminated = false`.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/ControlledExecutor.scala:26-42; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/DebugRef.scala:31-80; uso: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/ActorSystemStub.scala:86-100; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/BehaviorTestKitImpl.scala:163-168
- **Precondiciones**: `runOne` requiere cola no vacia.
- **Comportamiento**: `BehaviorTestKitImpl` llama `runAllTasks()` tras el arranque y tras cada `run` (no tras `signal`), de modo que las tareas encoladas por `executionContext.execute` o `dispatchers.lookup` corren al final del `run`. `deadLettersInbox` es un `DebugRef` en `/deadLetters` y es tambien el `ignoreRef` (todo lo enviado a el, incluidos los `replyTo` de los ask reificados, queda acumulado y nunca se lee por la API publica).
- **Errores**: `runOne` con cola vacia -> `NoSuchElementException` (`LinkedList.pop`); `DebugRef.receiveMessage/receiveSignal` vacio -> `NoSuchElementException("empty DebugRef")`; tipo equivocado -> `IllegalStateException("expected message but found signal <s>")` / `IllegalStateException("expected signal but found message <m>")`; `provider` -> `UnsupportedOperationException("no provider")`.
- **Orden / concurrencia / ciclo de vida**: `ControlledExecutor` NO es thread-safe (uso desde el hilo del test); `DebugRef` si.
- **Dependencias**: AT-TK-011; AT-TK-003.2.
- **Equivalente Rust propuesto**: `ControlledExecutor` como `VecDeque<Box<dyn FnOnce()>>` con `run_one`/`run_all`/`queue_size`; `DebugRef<M>` como cola de `enum Item{Message(M), Signal(SystemMessage)}`. Se preservan FIFO, `run_all` incluye tareas nuevas, y los cuatro textos de error.
- **Aserciones**:
  - A1. `runAllTasks` en el constructor y tras `run` es lo que permite que el `setup` y las tareas diferidas corran: *no afirmada por spec* directamente (ningun test encola tareas en el executor). Caracterizacion propuesta: comportamiento cuyo `setup` hace `context.executionContext.execute(task)` -> `task` ya corrio al terminar `BehaviorTestKit(..)`; tarea encolada durante `signal` NO corre hasta el siguiente `run`.
  - A2. FIFO, `queueSize`, `runAll` con tareas anidadas y `reportFailure`: *no afirmada por spec*. Caracterizacion propuesta: encolar A, B (B encola C) -> `runAll` ejecuta A, B, C en ese orden y `queueSize == 0`.
  - A3. `DebugRef`: `hasMessage/hasSignal`, `receiveMessage/Signal/All` y sus errores: *no afirmada por spec*. Caracterizacion propuesta: `tell("x")`, `sendSystem(sig)` -> `hasMessage`, `receiveMessage == "x"`, `hasSignal`, `receiveSignal == sig`; `receiveMessage` vacio -> `empty DebugRef`.
  - A4. el uso como `ignoreRef`/`deadLetters` del stub es indirecto en el ask reificado — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:539-554
- **Oraculo / Validacion prevista**: A1-A3 caracterizacion portada a Rust (clases internas: sin oraculo directo; se pueden comparar con jshell via reflexion).
- **Decisiones**: Q-T-TK-4, Q-X-8.

## AT-TK-013: LogbackUtil y LogCapturingExtension (JUnit 5): utilidades de captura de logs (internal-required)

- **Contrato observable**: [contractual JVM] `LogbackUtil`: `loggerNameOrRoot("")` = `ROOT`; `getLogbackLogger(name)` devuelve el `Logger` de Logback esperando hasta 50 intentos x 50 ms (2,5 s) a que SLF4J salga del estado `SubstituteLogger`; `convertLevel` mapea TRACE/DEBUG/INFO/WARN/ERROR de Logback a `Level` de SLF4J. `LogCapturingExtension` (JUnit 5 `InvocationInterceptor.interceptTestMethod`) hace con el `CapturingAppender` de la raiz lo mismo que `LogCapturing`: registra `Logging started/finished for test [<clase simple>: <metodo>]`, y si el test falla imprime `--> [..] Start of log messages of test that failed with <mensaje>`, vuelca el buffer, imprime `<-- [..] End of log messages ...`, relanza; en todo caso `clear()`.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/LogbackUtil.scala:25-59; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/javadsl/LogCapturingExtension.scala:20-53
- **Precondiciones**: Logback con `CapturingAppender` configurado (`CapturingAppender.get("")` en el constructor, fail-fast).
- **Comportamiento**: ver contrato.
- **Errores**: `IllegalArgumentException("Couldn't find logger for [<name>].")`; `IllegalArgumentException("Requires Logback logger for [<name>], it was a [<clase>]")`; `IllegalArgumentException("Level <str>, <int> is unknown.")` (p.ej. OFF/ALL); errores de `CapturingAppender.get` (AT-TK-006.5).
- **Orden / concurrencia / ciclo de vida**: la espera de inicializacion usa `Thread.sleep(50)` recursivo (tail-rec).
- **Dependencias**: AT-TK-006.5; AT-TK-006.4; AT-TK-009.3; Q-X-7.
- **Equivalente Rust propuesto**: `LogbackUtil` no tiene equivalente (no hay SLF4J/Logback): el nombre de logger (`target`), la conversion de niveles (los 5 niveles de `tracing` coinciden con SLF4J) y la espera de inicializacion desaparecen; `LogCapturingExtension` = el mismo guard de AT-TK-006.5. Diferencia a documentar: sin errores de "logger no es Logback".
- **Aserciones**:
  - A1. la extension coexiste con tests que pasan (`testSomething`, `testSomething2`) — actor-testkit-typed/src/test/java/jdocs/org/apache/pekko/actor/testkit/typed/javadsl/LogCapturingExtensionExampleTest.java:32-54
  - A2. igual en `ActorTestKitJUnit5Test` (`@ExtendWith(LogCapturingExtension.class)`) — actor-testkit-typed/src/test/java/org/apache/pekko/actor/testkit/typed/javadsl/ActorTestKitJUnit5Test.java:29-73
  - A3. volcado en fallo, mensajes `Logging started/finished`, y todos los errores/`convertLevel` de `LogbackUtil`: *no afirmada por spec*. Caracterizacion propuesta: `convertLevel(Level.OFF)` -> IllegalArgumentException; test JUnit 5 que falla -> el buffer se volco una vez antes de relanzar.
- **Oraculo / Validacion prevista**: no portable (JUnit 5/Logback); la parte de comportamiento (captura por test) se valida en AT-TK-006.5.
- **Decisiones**: Q-X-7, Q-T-LOG-1, Q-T-TK-2.

## AT-TK-014: TestException (excepcion de prueba sin traza) (public)

- **Contrato observable**: Contractual: `final case class TestException(message: String) extends RuntimeException(message) with NoStackTrace`: excepcion predefinida para tests, con mensaje y sin traza de pila; igualdad de case class.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/TestException.scala:14-21
- **Precondiciones**: ninguna.
- **Comportamiento**: `getMessage == message`; `getStackTrace` vacio.
- **Errores**: n/a.
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: ninguna.
- **Equivalente Rust propuesto**: `struct TestException{message: String}` que implementa `std::error::Error` y `Display` (sin backtrace capturado).
- **Aserciones**:
  - A1. se usa como causa en el filtro de logs: `LoggingTestKit.error("err").withCause[TestException]` captura `log.error("err", TestException("an error"))` — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestAppenderSpec.scala:46-50
  - A2. Java: `error(TestException.class)` coincide con `TestException("exc")` y no con `RuntimeException` — actor-testkit-typed/src/test/java/org/apache/pekko/actor/testkit/typed/javadsl/LoggingTestKitTest.java:67-83
  - A3. sin traza y mensaje: *no afirmada por spec*. Caracterizacion propuesta: `TestException("m").getStackTrace.length == 0` y `getMessage == "m"`.
- **Oraculo / Validacion prevista**: test portado a Rust (trivial); A3 sin oraculo (clase JVM).
- **Decisiones**: ninguna.

## Descomposiciones

Los padres desaparecen del TSV; los hijos heredan `impl`/`validation` = pending (ninguna fila de este grupo estaba implementada).

- AT-RCP-005 -> AT-RCP-005.1 (Find), .2 (Subscribe y Listing inicial/incremental), .3 (accesores y chequeo de clave del Listing).
- AT-RT-002 -> AT-RT-002.1 (round robin), .2 (random), .3 (consistent hashing): tres logicas con contratos y errores distintos.
- AT-ADP-006 -> AT-ADP-006.1 (despacho de mensajes), .2 (resultado del Behavior), .3 (hooks de ciclo de vida), .4 (supervision de hijos y ChildFailed).
- AT-PS-001 -> AT-PS-001.1 (Subscribe/Unsubscribe/terminacion), .2 (Publish), .3 (GetTopicStats).
- AT-EXT-001 -> AT-EXT-001.1 (identidad y concurrencia), .2 (fallo cacheado), .3 (extension() sin registro / hasExtension).
- AT-EXT-002 -> AT-EXT-002.1 (library-extensions), .2 (extensions), .3 (LoadTypedExtensions desde clasico).
- AT-TK-001 -> AT-TK-001.1 (construccion, nombre, config), .2 (spawn/stop), .3 (shutdown).
- AT-DLV-001 -> AT-DLV-001.1 (protocolo Start/RequestNext y demanda), .2 (secuenciación, ventana y confirmación Request/Ack), .3 (reenvío: Resend, Request vencida, ResendFirst, sin reenvío), .4 (RegisterConsumer y reemplazo del ConsumerController), .5 (MessageWithConfirmation), .6 (integración con DurableProducerQueue).
- AT-DLV-002 -> AT-DLV-002.1 (Start, stash previo y parada con el consumidor), .2 (entrega uno a uno y Confirmed), .3 (ventana de flujo, Request y backoff), .4 (pérdidas y Resend), .5 (only-flow-control), .6 (registro y cambio de ProducerController, deduplicación), .7 (Ack opcional, Ack al parar y DeliverThenStop).
- AT-DLV-003 -> AT-DLV-003.1 (workers vía Receptionist y GetWorkerStats), .2 (demanda, selección aleatoria y buffer), .3 (MessageWithConfirmation con Done, Ack del worker, AskTimeout), .4 (reasignación al desaparecer un worker), .5 (Start/reinicio del productor), .6 (cola duradera: preselección y traspaso).
- AT-DLV-005 -> AT-DLV-005.1 (partición en el productor), .2 (reensamblado en el consumidor), .3 (chunking con DurableProducerQueue).
- AT-JAPI-001 -> AT-JAPI-001.1 (fachadas delegadas), .2 (receive/receiveSignal con función de señal total), .3 (withMdc). AT-JAPI-002 -> .1 (AbstractBehavior), .2 (Receive/ReceiveBuilder). AT-JAPI-005 -> .1 (espejo), .2 (ask/askWithStatus), .3 (pipeToSelf), .4 (messageAdapter(Class)). AT-JAPI-006 -> .1 (TimerScheduler), .2 (StashBuffer), .3 (Routers), .4 (AskPattern), .5 (Adapter). AT-JAPI-003 y AT-JAPI-004 se mantienen sin hijos.
- Sin descomponer tras la lectura (un solo comportamiento verificable): AT-DLV-004 (State puro y protocolo), AT-DLV-006 (Settings y claves), AT-DLV-007 (flight recorder), AT-ADP-006.1..4 ya eran hijos, AT-ADP-010, AT-LOG-002, AT-LOG-004, AT-DSP-004, AT-DSP-005.
- AT-TK-002 -> AT-TK-002.1 (expect/receive), .2 (fishForMessage), .3 (expectTerminated/stop), .4 (awaitAssert), .5 (within/remaining), .6 (tabla de dilatación de plazos con timefactor; el esquema de IDs no admite `002.4.n`, por eso .5 y .6 son hermanos).
- AT-TK-003 -> AT-TK-003.1 (construcción, ruta, sistema y configuración), .2 (run/runOne/signal y estado), .3 (cola de efectos), .4 (hijos, selfInbox, childInbox/childTestKit), .5 (logEntries/clearLog), .6 (receptionistInbox), .7 (runAsk/runAskWithStatus).
- AT-TK-004 -> AT-TK-004.1 (AskInitiated), .2 (Spawned, SpawnedAnonymous, Stopped), .3 (Watched, WatchedWith, Unwatched), .4 (MessageAdapter, SpawnedAdapter, SpawnedAnonymousAdapter), .5 (ReceiveTimeoutSet/Cancelled, Scheduled, NoEffects), .6 (TimerScheduled/TimerCancelled y simulación de timers), .7 (fábricas Effects y convenciones de Effect).
- AT-TK-005 -> AT-TK-005.1 (TestInbox), .2 (ReplyInbox), .3 (StatusReplyInbox).
- AT-TK-006 -> AT-TK-006.1 (condiciones de coincidencia y LoggingEvent), .2 (fábricas), .3 (expect: ocurrencias, exceso, plazos y ausencia), .4 (TestAppender y logger name), .5 (LogCapturing y CapturingAppender).
- AT-TK-009 -> AT-TK-009.1 (ScalaTest), .2 (JUnit 4), .3 (JUnit 5 y builder), .4 (SerializationTestKit).


## Omisiones agregadas

- AT-RCP-008: extractores `ServiceKey.Listing` / `ServiceKey.Registered` (`actor-typed/src/main/scala/org/apache/pekko/actor/typed/receptionist/Receptionist.scala:77-95`), usados por Topic y routers.
- AT-RT-006: validacion `pool size must be positive, was <n>` y construccion inmutable de los builders (`actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/PoolRouterImpl.scala:38`).
- AT-ADP-008: singleton `AdapterExtension`/`ActorSystemAdapter` y carga de extensiones tipadas desde el clasico (`internal/adapter/AdapterExtension.scala`, `actor-typed/src/main/resources/reference.conf:35-36`; archivo sin cita en el inventario).
- AT-ADP-009: mapeo de mensajes de sistema y rechazo de `null` en `ActorRefAdapter` (`actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorRefAdapter.scala:35-38,70-81`).
- AT-ADP-010: `GuardianStartupBehavior` y `GuardianStopInterceptor` (`actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/GuardianStartupBehavior.scala:31-90`), que alimentan AT-CS-001.
- AT-TK-010: `TestKitSettings` y claves `pekko.actor.testkit.typed.*` (`actor-testkit-typed/src/main/resources/reference.conf:9-39`, `actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/TestKitSettings.scala:27-101`).
- AT-TK-014: `TestException` (`actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/TestException.scala`), excepción de prueba sin traza que usan decenas de tests; no tenía fila (omisión de exhaustividad detectada en el cierre 2026-10-31).
- Sin fila nueva por no definir comportamiento observable: `Platform.scala` (alias de tipos por versión de Scala) y `DeliverySerializable.scala` (trait marcador): ver la Revisión de exhaustividad.
- Cierre 2026-10-31: sin filas nuevas en ASK, RCP, RT, ADP, EVT, LOG, PS, SPV, EXT, DSP, DLV, CS, SER ni JAPI tras contrastar el árbol (ver tabla); las omisiones reales eran descomposición (DLV, JAPI, TK) y `TestException`.

## Contradicciones halladas y corregidas

- AT-EXT-002 (inventario viejo): "extensions loguea y sigue". Codigo (`actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:56-65`) y test (`actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:136-153`): solo los fallos de CARGA (clase ausente, no es ExtensionId) se omiten con log en `extensions`; un fallo de CREACION hace fallar el arranque (RuntimeException, repetible). Corregido en AT-EXT-002.2 y AT-EXT-001.2.
- AT-ASK-001 (inventario viejo): "timeout <= 0 es IllegalArgumentException". El codigo (`actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/AskPattern.scala:152-157`) devuelve `Future.failed(IllegalArgumentException)`, no la lanza, y ademas igual se envia el mensaje a dead letters (`actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/AskPattern.scala:170-181`). Sin test que lo afirme. Corregido en AT-ASK-001.
- AT-DSP-003 (inventario viejo): default = SingleConsumerOnlyUnboundedMailbox. PropsAdapter fija `mailbox == "pekko.actor.typed.default-mailbox"` (`actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/adpater/PropsAdapterSpec.scala:28-32`) y `MailboxSelector.default()` devuelve `fromConfig` (`actor-typed/src/main/scala/org/apache/pekko/actor/typed/Props.scala:213`); el tipo de cola sale de `actor-typed/src/main/resources/reference.conf:29-32`. Corregido en AT-DSP-003.
- AT-SPV-001: el Scaladoc dice que el formato del sufijo es "undefined" (`actor-typed/src/main/scala/org/apache/pekko/actor/typed/SpawnProtocol.scala:49-52`) pero el spec lo fija en "-1","-2",... (`actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SpawnProtocolSpec.scala:89-111`). Se sigue el codigo y el spec; ver contratos.
- AT-RT-002 (inventario viejo): "sin routees lanza". Solo ConsistentHashingLogic lo afirma (IllegalStateException con texto fijo, `actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/routing/RoutingLogicSpec.scala:170-178`); round robin y random fallan de otra forma y los routers nunca llaman sin routees. Corregido en AT-RT-002.1-3 (Q-T-RT-2).
- AT-EVT-004 (inventario viejo): "ciertos descartes se publican como Dropped" sin distinguir camino. Supervision usa el eventStream clasico, el router de grupo `EventStream.Publish` tipado, y Topic envia `Dropped` a `deadLetters`, que `DeadLetterActorRef` envuelve en `DeadLetter` (`actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:736-743`): efecto resuelto por lectura en el cierre 2026-10-31: los suscriptores de `Dropped` no ven el descarte de Topic y los de `DeadLetter` ven `DeadLetter(Dropped(..))`; queda por decidir si se unifica (Q-T-EVT-1).
- AT-RCP-005 (inventario viejo) omite que no existe `Unsubscribe` en el protocolo y que Deregister/Register repetidos notifican a suscriptores aunque el estado no cambie (`actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionist.scala:195-218`). Registrado en AT-RCP-004/005.2 y Q-T-RCP-3.
- AT-ADP-006.1 (antes `listed`): las aserciones "no afirmada por spec" A1-A3 sí tienen spec de otros grupos: ChildFailed/DeathPact en `WatchSpec.scala:100-203`, MessageAdaptionFailure en `AdaptationFailureSpec.scala:75-129`, adaptación nula en `ActorContextPipeToSelfSpec.scala:47-72`, receive-timeout en `ActorContextSpec.scala:578-595`, TimerMsg obsoleto en `TimerSpec.scala:174-232`. Solo "ningún adaptador acepta el mensaje" sigue sin afirmar (A10).
- AT-ADP-006.3: el test "be able to terminate immediately" (`ActorSystemSpec.scala:126-135`) solo afirma que `terminate` completa; la ausencia de PostStop en un `Deferred` sin iniciar la fija el código (`ActorAdapter.scala:328-331`) y el comentario del test, no una aserción.
- AT-ADP-006.4: el antiguo A1 decía "restart por defecto" sin mostrar la prueba; `AdapterSpec.scala:317-330` afirma que el hijo clásico que lanza `ThrowIt3` se registra como error y sigue vivo (el probe recibe "ok" por ping/pong).
- AT-ADP-010 duplica AT-SYS-003/004 (typed-core): se mantiene como fila de integración y la fuente canónica es SYS; los antiguos "GuardianStartupSpec no leído" quedan resueltos (`GuardianStartupSpec.scala:34-89`).
- AT-LOG-002 / AT-DSP-004: `ActorLoggingSpec` "pass tags from props to MDC" está en 270-289 (no 270-290); además el spec cubre `withMdc` estático/por mensaje/anidado/reemplazo y las tres claves automáticas (338-529), que el bloque no citaba.
- AT-LOG-004: `LoggerOpsSpec` solo ejercita `info2`/`infoN` (renderizado del mensaje); los otros 18 métodos son no afirmados por spec; `ActorLoggingSpec.scala:204-261` cuenta 36 eventos sin comprobar su contenido.
- AT-DSP-005: ningún test llama a `system.dispatchers.lookup` directamente; `DispatcherSelectorSpec` lo ejercita por `spawn`.
- AT-ASK-002 (A4): `StatusReplySpec.scala:46-52` solo afirma que los extractores no coinciden con `null`; el `InvalidMessageException("[null] is not an allowed status")` sigue sin afirmación. Se agregan las aserciones reales del spec clásico y de `StatusReplyTest.java`.
- AT-DLV-001 (antiguo): `RequestNext` no es `(sendNextTo, askNextTo, currentSeqNr, confirmedSeqNr)`: el orden real es `(producerId, currentSeqNr, confirmedSeqNr, sendNextTo, askNextTo)` (`ProducerController.scala:118-123`). Además `enforceLocalProducer` lanza `IllegalArgumentException("Consumer [..] should be local.")` aun siendo el productor.
- AT-DLV-003 (antiguo): `MessageWithConfirmation` de work-pulling responde `Done` (no `SeqNr`); `GetWorkerStats` cuenta los workers del último `Listing`, no los que tienen demanda.
- AT-DLV-004: confirmado contra `DurableProducerQueueSpec.scala:29-99`; el efecto de `confirmed` sobre `highestConfirmedSeqNr` y el mapa de confirmados no está afirmado por ese spec sino por `DurableProducerControllerSpec.scala:112-141`.
- AT-DLV-005 (antiguo): no decía que cada trozo es un `SequencedMessage` con su propio seqNr consecutivo (el seqNr del mensaje es el del último trozo, el único con `ack = true`), ni que el troceo consume demanda de la ventana (`ProducerControllerImpl.scala:735-771`, `ProducerControllerSpec.scala:263-305`). La exclusión de work-pulling se confirma: `WorkPullingProducerController.Settings` lo rechaza (`WorkPullingProducerController.scala:191-192`).
- AT-DLV-006: `ConsumerController.Settings.toString` está en `ConsumerController.scala:289-290` (antes "aprox 288") y omite `resendIntervalMax`; `WorkPullingProducerController.Settings` valida el troceo en su constructor.
- AT-DLV (varios): el original usa tiempo real (1 s, 1,1 s) y semillas `System.currentTimeMillis()`; no es una discrepancia con Pekko sino una limitación para portar los tests (Q-T-DLV-4).
- Q-T-EVT-1: el efecto del descarte de `Topic` (`deadLetters ! Dropped(..)` publicado como `DeadLetter(Dropped(..))`) queda resuelto por lectura de `ActorRef.scala:736-743`; ya no es "probable diferencia pendiente de oráculo".
- Q-T-EXT-1: la política de `library-extensions` es lanzar `RuntimeException` ante un nombre no resoluble (no solo en `extensions`); solo `extensions` hace log y omite (`ExtensionsImpl.scala:39-66`).

Hallazgos del cierre en JAPI y auditoría de SPV/EXT/CS:

1. **Tests Java que no afirman nada**: `ReceiveBuilderTest.testMutableCounter` (37-65, no hace spawn); `ActorLoggingTest` (los tres @Test): el `CustomEventFilter` nunca se instala (sin Mute/intercept) y `awaitDone` devuelve un booleano que se descarta (`TestEventListener.scala:106-109`); `loggingProvidesMDC` ni siquiera envia un mensaje. `shouldCompile`, `compileOnlyHandlerAllowedToThrowCheckedException`, `compileOnlyApiTest` y las ramas supervise-stop/resume de `AdapterTest.Classic2` no son `@Test`. `OnMessageIntroTest` (AbstractMatchingBehavior) solo tiene `main`. Las aserciones reales de logging/MDC viven en AT-LOG-001/AT-BEH-018/019 (Scala).
2. **Tests de contexto equivocado** citados para AbstractBehavior (`SupervisionSpec:1550-1570`, `ActorThreadSpec:188-215,255-275`) ejercitan `scaladsl.AbstractBehavior`; el javadsl no tiene prueba de esos chequeos. Los textos difieren: javadsl `AbstractMatchingBehavior` dice "the wrong ActorContext".
3. **Defaults divergentes de MessageAdaptionFailure** entre ReceiveBuilder/AbstractBehavior (relanza), BehaviorBuilder y AbstractMatchingBehavior (unhandled: el actor sobrevive) y `receive(m,s)` Java (depende de la funcion). Derivado del codigo (`ActorAdapter.scala:220-229` ignora senales unhandled); no afirmado por spec -> requiere decision de la API Rust unica (Q-T-JAPI-1).
4. **Javadoc de startPeriodicTimer**: dice "same semantics as startTimerAtFixedRate" pero la sobrecarga Java usa `startTimerWithFixedDelay` (`TimerSchedulerImpl.scala:81-84`); ya registrado en typed-core (AT-TMR-005).
5. **Scaladoc de Extensions.extension** dice IllegalStateException; el codigo lanza IllegalArgumentException (`ExtensionsImpl.scala:81`). Ningun test del repo contiene "non-registered".

- AT-SPV-001: A4 86-113 -> 86-107 (el rango incluia otra clase); A2 solo comprueba que el ask completa; A5 dividida en A5/A6 (efecto Spawned vs SpawnedAnonymous con nombre `$`); null/nombre invalido/reuso de nombre liberado -> no afirmados; se agrega ejemplo Java `SpawnProtocolDocTest` y `AT-CTX-003`.
- AT-EXT-001.2: **A1 no afirma el cacheo**: el test crea dos ActorSystem distintos; el comentario "retry on each access" es enganoso. Cacheo = solo codigo (A2 no afirmada).
- AT-EXT-002.1: **A4 incorrecta**: `ExtensionsSpec:195-202` no carga por `getInstance()` desde config; esa carga la afirma `ExtensionsTest.java:60-80` (clave `extensions`, movida a AT-EXT-002.2 A5).
- AT-EXT-001.3: contradiccion Scaladoc/codigo; ningun test del mensaje; se agregan A2/A3 (setup, Java).
- AT-EXT-001.1 / 002.3: se agregan aserciones de adaptador unico (`ExtensionsSpec:222-234`) y Java; AT-EXT-002.3 aclara que el "+1" depende de la config del test.
- AT-EXT-003: limite no documentado: `ActorSystemSetup` indexa setups por clase (`ActorSystemSetup.scala:86`), dos ExtensionSetup de la misma clase se pisan; se agrega variante Java (`ExtensionsTest:95-113`).
- AT-CS-001: se agrega `close()` (desde 1.3.0, `ActorSystem.scala:148-166`), la condicion de arranque del guardian (stash 1000 / StashOverflowException) y el test Java `ActorSystemTest`; `run-by-actor-system-terminate=off` y el `TimeoutException` de `close()` quedan no afirmados.

Hallazgos del cierre en TK (testkit tipado):

- **AT-TK-002.2 (error de contenido)**: `TestProbeSpec.scala:99-108` NO prueba un fisher lento; la probe esta vacia y la espera expira (el `Thread.sleep(150)` jamas corre). Corregidos `scenarios`, bloque y notes.
- **Prefijo `assertion failed: `** (hallazgo transversal): los mensajes construidos con `Predef.assert` llevan ese prefijo (`stop` de ActorTestKit, `expectTerminated`, `within`, `TestInbox.expectMessage`, `expectEffect`, `childInbox`), mientras que `assertFail`/`throw new AssertionError` no (expectMessage, receiveMessage, fish, ReplyInbox, expectEffectClass/PF). Corregidos AT-TK-001.2, 002.3, y reflejado en 002.5, 003.3, 005.1. Los inventarios previos citaban los textos sin prefijo.
- **Dilatacion (AT-TK-002.4 antiguo / 010)**: "todos los plazos con .dilated" es falso: `expectNoMessage(max)` no se dilata, tampoco el `min` de `within`, el intervalo de `awaitAssert` ni `ManualTime.timePasses`. Ahora en AT-TK-002.6.
- **AT-TK-004.2 antiguo**: afirmaba "cancelAll cancela activos"; `EffectfulActorContext.scala:191` hace `activeTimers.foreach(cancel)` con la tupla `(clave, efecto)`, por lo que `cancelAll` no cancela nada (inferido por semantica de Scala; ver Q-T-TK-5).
- **AT-TK-010**: el error de `timefactor` es `IllegalArgumentException("requirement failed: timefactor must be positive finite double")` (via `Helpers.Requiring`); plazos leidos en ms truncados (`getMillisDuration`).
- AT-TK-001.1/001.2/001.3: se anaden aserciones de clases derivadas/composicion (`ActorTestKitSpec.scala:161-170,189-191`), espejos Java y `AsyncTestingExampleSpec.scala:103-118` (stop libera el nombre, antes sin citar); notas sobre `TestKitRegex` (usa `org.apache.pekko.testkit.typed...`).
- AT-TK-002.1: espejos Java y `Terminated` fuera de la cola de mensajes. AT-TK-002.3: texto con prefijo.
- AT-TK-004.1: las citas A1-A5 se verificaron correctas; se anade que la igualdad del efecto solo usa la primera lista de parametros, ejemplo documental y Java, y que los textos exactos de las `IllegalStateException`/`require` NO estan afirmados (los tests solo comprueban el tipo).
- AT-TK-008: verificada contra cuerpos (`ActorTestKitSpec.scala:133-150`), sin cambios de contenido (solo nota).

Hallazgos y contradicciones del testkit (detalle):
1. `Predef.assert` vs `assertFail` (prefijo), arriba.
2. `cancelAll` del timer del stub es no-op (inferido; sin test; confirmar con jar oficial).
3. `LoggingTestKit.empty`: el Scaladoc dice "no coincide con ningun evento", la implementacion coincide con todos (`LoggingTestKitImpl.scala:61-71`).
4. `TestAppender.filter` usa `filters.exists`, asi que con filtros solapados solo el mas reciente que coincide cuenta el evento (no documentado).
5. `TestKitJUnit5Extension`: guarda un unico `testKit` (el ultimo `beforeTestExecution`); con instancia por test, `afterAll` solo apaga el ultimo (fuga de sistemas; derivado, sin spec).
6. `ReplyInbox.receiveReply` marca el inbox como consumido ANTES de leer: sin respuesta lanza `NoSuchElementException` y deja el inbox consumido.
7. `ControlledExecutor`/`DebugRef`/`ActorSystemStub` (011-012): ningun test del repositorio los ejercita directamente; todo el contrato sale de lectura y caracterizacion propuesta.
8. `ReceiveTimeoutSet`, `ReceiveTimeoutCancelled`, `Scheduled`, `StatusReplyInbox` y `SerializationTestKit` no tienen ningun test dentro de `actor-testkit-typed` (solo uso externo en cluster-sharding-typed / persistence-*): caracterizacion `no afirmada por spec`.
9. Entorno: `java -version` = 11 (el oraculo exige 21) y `pekko-actor-testkit-typed_2.13` no esta en `tools/oracle/fetch-jars.sh`; la columna "Oraculo" de los bloques indica que el jar hay que agregarlo.

## Filas que siguen listed

Ninguna. Tras el cierre documental 2026-10-31 las 95 filas de este grupo que estaban `listed` (ADP-006.1/.3/-010, LOG-002/-004, DSP-004/-005, DLV-001/-002/-003/-005/-007, JAPI-001..006, TK-002.4/-003/-004.2/-005/-006/-007/-009/-011/-012/-013) pasan a `extracted` con sus hijos. Esto **no** significa paridad: varias aserciones son "no afirmada por spec" con caracterización propuesta (se indica en cada bloque y en las `notes` de la fila).

Pendientes de análisis (no se convierten en "fuera de alcance"): tests Java de DSP/EVT/RCP (`DispatcherSelectorTest`, `MailboxSelectorTest`, `eventstream/*`, `receptionist/ReceptionistApiTest`), los `jdocs` con `@Test` (`AggregatorTest`, `BubblingSampleTest`, `InteractionPatterns*Test`, `MailboxDocTest`, `RouterTest`, `StashDocTest`, `coexistence/*`), `OrElseSpec` y las variantes `scala-2.13+`/`scala-3` de `actor-typed-tests`; además, la ejecución del oráculo del testkit (el jar `pekko-actor-testkit-typed_2.13` no está en `tools/oracle/fetch-jars.sh` y el entorno tiene Java 11 en lugar de 21).

## Cobertura

Archivos de Pekko del area y fila(s) que los cubren (`actor-typed/src/main/scala/org/apache/pekko/actor/typed/` salvo indicacion):

- `scaladsl/AskPattern.scala; `actor/src/main/scala/org/apache/pekko/pattern/StatusReply.scala`; internal/InternalRecipientRef.scala` -> AT-ASK-001..004.
- `receptionist/Receptionist.scala; internal/receptionist/{LocalReceptionist,ReceptionistImpl,ReceptionistMessages,ServiceKey}.scala` -> AT-RCP-001..008.
- `internal/receptionist/Platform.scala (scala-2.12, scala-2.13, scala-3)` -> sin comportamiento observable: alias de tipos (Service/Subscriber) distintos por version de Scala; no genera fila.
- `scaladsl/Routers.scala; internal/routing/{PoolRouterImpl,GroupRouterImpl,RoutingLogic}.scala` -> AT-RT-001..006.
- `scaladsl/adapter/{package,PropsAdapter}.scala; internal/adapter/{ActorAdapter,ActorRefAdapter,ActorRefFactoryAdapter,PropsAdapter,AdapterExtension,GuardianStartupBehavior,SchedulerAdapter}.scala; internal/InternalMessage.scala` -> AT-ADP-001..010 (ActorContextAdapter y ActorSystemAdapter se cubren en CTX y SYS, otros grupos, salvo lo citado).
- `eventstream/EventStream.scala; internal/adapter/EventStreamAdapter.scala; internal/EventStreamExtension.scala` -> AT-EVT-001; AT-EVT-002..004 (DeadLetter/Unhandled/Dropped).
- `internal/{ActorMdc,LoggerClass}.scala; ActorContextImpl.scala (parte de logging)` -> AT-LOG-001..005.
- `pubsub/Topic.scala; internal/pubsub/TopicImpl.scala` -> AT-PS-001.1..3.
- `SpawnProtocol.scala` -> AT-SPV-001.
- `Extensions.scala; internal/ExtensionsImpl.scala` -> AT-EXT-001.1..3, AT-EXT-002.1..3, AT-EXT-003.
- `Props.scala; internal/PropsImpl.scala; Dispatchers.scala` -> AT-DSP-001..005.
- `delivery/{ProducerController,ConsumerController,WorkPullingProducerController,DurableProducerQueue}.scala` -> AT-DLV-001..006.
- `delivery/internal/{ProducerControllerImpl,ConsumerControllerImpl,WorkPullingProducerControllerImpl,ChunkedMessage}.scala` -> AT-DLV-001..003, 005 (leídos en el cierre 2026-10-31: AT-DLV-001..003, 005).
- `delivery/internal/DeliverySerializable.scala` -> sin comportamiento observable: trait marcador para los serializadores de persistencia/cluster (OUT).
- `internal/ActorFlightRecorder.scala; scala-jdk-9/internal/jfr/{Events,JFRActorFlightRecorder}.scala` -> AT-DLV-007 (JVM-only, extraído).
- `internal/MiscMessageSerializer.scala; internal/receptionist/ServiceKeySerializer.scala; ActorRef.scala (SerializedActorRef)` -> AT-SER-001..003.
- `resources/reference.conf (typed)` -> extensions/library-extensions: AT-EXT-002; default-mailbox: AT-DSP-003; restart-stash-capacity: unidad SUP (otro grupo); serializers: AT-SER-001/002; use-slf4j: AT-LOG-003; reliable-delivery: AT-DLV-006.
- `javadsl/{Behaviors,AbstractBehavior,AbstractMatchingBehavior,ActorContext,Adapter,AskPattern,BehaviorBuilder,Receive,ReceiveBuilder,Routers,StashBuffer,TimerScheduler}.scala` -> AT-JAPI-001..006 (extraídas con hijos).
- actor-testkit-typed/src/main: scaladsl/ActorTestKit.scala, internal/TestKitUtils.scala -> AT-TK-001.1..3.
- actor-testkit-typed/src/main: internal/TestProbeImpl.scala, scaladsl/TestProbe.scala, javadsl/TestProbe.scala, FishingOutcome.scala -> AT-TK-002.1..4.
- actor-testkit-typed/src/main: scaladsl/BehaviorTestKit.scala, internal/{BehaviorTestKitImpl,StubbedActorContext,EffectfulActorContext,ActorSystemStub,ControlledExecutor,DebugRef}.scala, Effect.scala, scaladsl/Effects.scala -> AT-TK-003, AT-TK-004.1..2.
- actor-testkit-typed/src/main: scaladsl/TestInbox.scala, internal/TestInboxImpl.scala -> AT-TK-005.
- actor-testkit-typed/src/main: scaladsl/LoggingTestKit.scala, scaladsl/LogCapturing.scala, internal/{LoggingTestKitImpl,TestAppender,CapturingAppender,LogbackUtil}.scala, LoggingEvent.scala, CapturedLogEvent.scala -> AT-TK-006.
- actor-testkit-typed/src/main: scaladsl/ManualTime.scala -> AT-TK-007.
- actor-testkit-typed/src/main: TestKitSettings.scala, resources/reference.conf, scaladsl/package.scala -> AT-TK-010.
- actor-testkit-typed/src/main: scaladsl/{ScalaTestWithActorTestKit,ActorTestKitBase,SerializationTestKit}.scala, javadsl/{TestKitJunitResource,TestKitJUnit5Extension,JUnit5TestKitBuilder,LogCapturingExtension}.scala, java/annotations/JUnit5TestKit.java -> AT-TK-009 (listed).
- actor-testkit-typed/src/main: TestException.scala -> sin comportamiento observable: clase de excepcion para tests.
- `actor-typed-tests (Scala): AskSpec, SpawnProtocolSpec, ExtensionsSpec, PropsSpec, MailboxSelectorSpec, LocalActorRefProviderLogMessagesSpec, eventstream/*, internal/{ActorRefSerializationSpec,ActorSystemSpec}, internal/receptionist/*, internal/routing/*, internal/adpater/PropsAdapterSpec, pubsub/LocalPubSubSpec, coexistence/*, scaladsl/{ActorLoggingSpec,RoutersSpec,DispatcherSelectorSpec,adapter/AdapterSpec,ActorContextDelegateSpec}` -> leidos y citados en las filas correspondientes.
- `actor-typed-tests: delivery/*Spec.scala y Test*.scala` -> leídos en el cierre 2026-10-31 (AT-DLV-001..007); `receptionist/ReceptionistApiSpec.scala`, `scaladsl/adapter/GuardianStartupSpec` (AT-ADP-010) y `LoggerOpsSpec` (AT-LOG-004) leídos; `OrElseSpec` sigue pendiente de análisis (composición escrita en el propio test, ver actor-typed.md).
- `actor-typed-tests: docs/** y jdocs/** (ejemplos de documentacion, ~40 archivos Scala/Java, incl. RouterSpec, SpawnProtocolDocSpec, MailboxDocSpec, DispatchersDocSpec, ExtensionDocSpec, PubSubExample, LoggingDocExamples, coexistence)` -> ejemplos que ilustran el uso de las APIs de las filas; no leidos: no definen comportamiento adicional salvo compilacion (a confirmar al implementar cada unidad).
- `actor-typed-tests: src/test/java/**` -> los 13 tests Java de `javadsl` y `ExtensionsTest`, `ActorSystemTest`, `SchedulerTest` leídos (AT-JAPI-*, AT-EXT-*, AT-CS-001); el resto sigue **pendiente de análisis** (ver "Filas que siguen listed"); `resources/mailbox-config-sample.conf`, `scala-2.13+/ActorSystemScala213PlusSpec.scala` y `scala-3/**` sin leer (pendiente).
- actor-testkit-typed/src/test (Scala y Java), resources/application*.conf -> los 28 archivos clasificados en la Revisión de exhaustividad (AT-TK-*).

## Revisión de exhaustividad (cierre documental 2026-10-31)

Se contrastó el árbol de fuentes del grupo con las filas: `actor-typed/src/main/scala/org/apache/pekko/actor/typed/**` (92 archivos del módulo; los de este grupo y los que son de otro grupo del mismo módulo), `actor-typed/src/main/resources/reference.conf`, `actor-typed/src/main/scala-jdk-9/**`, `actor-typed/src/main/scala-2.12|2.13|3/**`, `actor-testkit-typed/src/main/**` y `src/test/**` (ver tabla TK), y los specs de `actor-typed-tests` que usa el grupo. Clasificación: **C** = cubierto por contratos concretos, **D** = duplicado o fachada equivalente (se explica), **I** = detalle interno sin contrato independiente (se justifica), **F** = fuera de estos módulos con frontera explícita, **P** = pendiente de análisis (nunca se convierte en "fuera de alcance"), **O** = cubierto por un contrato de otro grupo del mismo módulo (archivo indicado).

| Elemento revisado | Clase | Fila(s) o justificación |
|---|---|---|
| `scaladsl/AskPattern.scala`, `internal/InternalRecipientRef.scala` | C | AT-ASK-001..004 |
| `actor/.../pattern/StatusReply.scala` (usado por `askWithStatus`) | C / F | AT-ASK-002; los specs de `StatusReply` (`actor-tests`) pertenecen a AC-ASK (frontera: se citan como respaldo, no se duplican) |
| `receptionist/Receptionist.scala`, `internal/receptionist/{LocalReceptionist,ReceptionistImpl,ReceptionistMessages,ServiceKey}.scala` | C | AT-RCP-001..008 |
| `internal/receptionist/ServiceKeySerializer.scala`, `internal/MiscMessageSerializer.scala`, `ActorRef.scala` (SerializedActorRef) | C | AT-SER-001..003 |
| `internal/receptionist/Platform.scala` (scala-2.12/2.13/3) | I | alias de tipos distintos por versión de Scala, sin comportamiento |
| `scaladsl/Routers.scala`, `internal/routing/{PoolRouterImpl,GroupRouterImpl,RoutingLogic}.scala` | C | AT-RT-001..006 |
| `internal/adapter/{ActorAdapter,ActorRefAdapter,ActorRefFactoryAdapter,AdapterExtension,GuardianStartupBehavior,PropsAdapter,SchedulerAdapter}.scala`, `scaladsl/adapter/{package,PropsAdapter}.scala` | C | AT-ADP-001..010 (010 es de integración; el contrato canónico del guardián está en AT-SYS-003/004) |
| `internal/adapter/{ActorContextAdapter,ActorSystemAdapter}.scala`, `internal/InternalMessage.scala` | O | grupo typed-core (AT-CTX-*, AT-SYS-011, AT-SYS-010); aquí solo se cita `ActorSystemAdapter.dispatchers` (AT-DSP-005) y el singleton (AT-ADP-008) |
| `eventstream/EventStream.scala`, `internal/EventStreamExtension.scala`, `internal/adapter/EventStreamAdapter.scala` | C | AT-EVT-001..004 |
| `internal/ActorMdc.scala`, `internal/LoggerClass.scala`, parte de logging de `internal/ActorContextImpl.scala`, `internal/WithMdcBehaviorInterceptor.scala` | C | AT-LOG-001, AT-LOG-002 (y AT-BEH-019 para `withMdc`, otro grupo) |
| `scaladsl/package.scala` (`LoggerOps`) | C | AT-LOG-004 (subsumido por macros de tracing: Q-T-LOG-4) |
| `pekko.use-slf4j` + `Settings.amendSlf4jConfig` (`actor/.../ActorSystem.scala:335-370`) | C / F | AT-LOG-003; la función es del módulo `actor` (AC-CFG-018): frontera, depende de Q-X-7 |
| Mensajes de log del `LocalActorRefProvider` (`ActorRefResolver`) | C | AT-LOG-005 |
| `pubsub/Topic.scala`, `internal/pubsub/TopicImpl.scala` | C | AT-PS-001.1..3 |
| `SpawnProtocol.scala` | C | AT-SPV-001 |
| `Extensions.scala`, `internal/ExtensionsImpl.scala` | C | AT-EXT-001.1..003 |
| `Props.scala`, `internal/PropsImpl.scala`, `Dispatchers.scala` | C | AT-DSP-001..005 (ActorTags: AT-DSP-004) |
| `delivery/ProducerController.scala` + `internal/ProducerControllerImpl.scala` | C | AT-DLV-001.1..6 y AT-DLV-005.1/005.3 |
| `delivery/ConsumerController.scala` + `internal/ConsumerControllerImpl.scala` | C | AT-DLV-002.1..7 y AT-DLV-005.2 |
| `delivery/WorkPullingProducerController.scala` + `internal/WorkPullingProducerControllerImpl.scala` | C | AT-DLV-003.1..6 (Settings: AT-DLV-006) |
| `delivery/DurableProducerQueue.scala` | C | AT-DLV-004 |
| `delivery/internal/ChunkedMessage.scala` | C | AT-DLV-005.1/005.2 |
| `delivery/internal/DeliverySerializable.scala` | I | trait marcador para serializadores de persistencia/cluster, sin comportamiento |
| Helpers Java de delivery (`requestNextClass`, `deliveryClass`, `serviceKeyClass`, `confirmed()`, `create(...)`) | D | fachada Java de la API Scala (AT-JAPI-006.* / Q-T-JAPI-1); la semántica es la de las filas DLV |
| `internal/ActorFlightRecorder.scala`, `scala-jdk-9/internal/jfr/{Events,JFRActorFlightRecorder}.scala`, `actor/.../util/FlightRecorderLoader.scala` | C | AT-DLV-007 (JVM-only; Q-T-DLV-3) |
| `javadsl/*.scala` (12 archivos) | C / D | AT-JAPI-001..006: ver tabla JAPI; `actor-typed/src/main/java` no existe en 1.7.1 |
| `reference.conf` (typed): `extensions`, `library-extensions` | C | AT-EXT-002.1/002.2 |
| `reference.conf`: `restart-stash-capacity` | O | AT-SYS-006 / AT-SUP-009 (typed-core) |
| `reference.conf`: `default-mailbox` | C | AT-DSP-003 |
| `reference.conf`: `serializers`, `serialization-identifiers`, `serialization-bindings` | C | AT-SER-001/002 |
| `reference.conf`: `pekko.library-extensions += LoadTypedExtensions` | C | AT-EXT-002.3 / AT-ADP-008 |
| `reference.conf`: `pekko.use-slf4j` | C | AT-LOG-003 |
| `reference.conf`: `reliable-delivery.*` (14 claves) | C | AT-DLV-006 (efecto: AT-DLV-001.3, 002.3, 002.5, 003.2, 005.1) |
| `Behavior.scala`, `BehaviorInterceptor.scala`, `LogOptions.scala`, `MessageAndSignals.scala`, `Scheduler.scala`, `SupervisorStrategy.scala`, `TypedActorContext.scala`, `ActorSystem.scala`, `scaladsl/{AbstractBehavior,ActorContext,Behaviors,StashBuffer,TimerScheduler}.scala`, `internal/{ActorRefImpl,BehaviorImpl,InterceptorImpl,PoisonPill,StashBufferImpl,Supervision,SystemMessage,TimerSchedulerImpl}.scala` | O | grupo typed-core (AT-BEH-*, AT-CTX-*, AT-REF-*, AT-SYS-*, AT-SUP-*, AT-SIG-*, AT-TMR-*, AT-STASH-*) |
| `ActorRefResolver.scala` | O / C | AT-REF-007 (typed-core); los mensajes de log de resolución están en AT-LOG-005 |
| `actor-typed-tests` (Scala) usado por este grupo: AskSpec, SpawnProtocolSpec, ExtensionsSpec, PropsSpec, MailboxSelectorSpec, LocalActorRefProviderLogMessagesSpec, eventstream/*, internal/{ActorRefSerializationSpec,ActorSystemSpec}, internal/receptionist/*, internal/routing/*, internal/adpater/PropsAdapterSpec, pubsub/LocalPubSubSpec, coexistence/*, scaladsl/{ActorLoggingSpec,LoggerOpsSpec,RoutersSpec,DispatcherSelectorSpec,AdaptationFailureSpec,ActorContextPipeToSelfSpec,adapter/AdapterSpec,adapter/GuardianStartupSpec,ActorContextDelegateSpec}, WatchSpec, TerminatedSpec, TimerSpec (partes), ActorContextSpec (partes) | C | leídos y citados en las filas |
| `actor-typed-tests` delivery/{ProducerControllerSpec,ConsumerControllerSpec,DurableProducerControllerSpec,DurableProducerQueueSpec,WorkPullingSpec,DurableWorkPullingSpec,ReliableDeliverySpec,ReliableDeliveryRandomSpec} | C | AT-DLV-001..006 (aserciones A# con líneas) |
| `actor-typed-tests` delivery/{TestConsumer,TestProducer,TestProducerWithAsk,TestProducerWorkPulling,TestDurableProducerQueue}.scala | C / I | helpers de test; sus invariantes implícitas (duplicados, `RequestNext` doble, comportamiento de la cola) se citan como aserciones en AT-DLV-001.1, 001.3, 002.2, 004 |
| `actor-typed-tests` `docs/**` y `jdocs/**` (ejemplos de documentación) | I | ilustran usos de APIs ya contratadas; los que contienen `@Test`/aserciones propias figuran como P en la tabla JAPI hasta contrastarlos con su fila |
| `actor-typed-tests/src/test/java/**` | P / C | ver tabla JAPI: `DispatcherSelectorTest`, `MailboxSelectorTest`, `eventstream/*`, `receptionist/ReceptionistApiTest` y los `jdocs` con `@Test` siguen **pendientes de análisis** (propiedad de DSP/EVT/RCP/ASK/STASH/RT/ADP) |
| `OrElseSpec`, `scala-2.13+/ActorSystemScala213PlusSpec.scala`, `scala-3/**` | P | no leídos en este cierre; variantes de versión de Scala y composición en el propio test (ver typed-core) |
| Capacidad no contratada: `ProducerController` y `ConsumerController` con función `send` propia (Cluster Sharding) | F | frontera: `cluster-sharding-typed` está fuera de los módulos (Q-X-10); documentada en AT-DLV-001.4 |
| Capacidad no contratada: `EventSourcedProducerQueue` (cola durable real) | F | `pekko-persistence-typed` (Q-X-10); el protocolo está en AT-DLV-004 y la cola de referencia en memoria en Q-T-DLV-2 |

### Tabla JAPI (fachadas Java)

Árbol: `actor-typed/src/main/scala/.../typed/javadsl` (12 archivos), `actor-typed/src/main/java` (no existe en 1.7.1), `scaladsl/adapter`, `actor-typed-tests/src/test/java` (47 archivos) y `.../test/scala/.../javadsl`.

| Archivo / area | Clasificacion | Fila(s) / justificacion |
|---|---|---|
| javadsl/Behaviors.scala | Duplicado o fachada equivalente (+ comportamiento propio) | 001.1 (delegacion), 001.2, 001.3 |
| javadsl/AbstractBehavior.scala, Receive.scala | Cubierto por contratos concretos | 002.1, 002.2 |
| javadsl/ReceiveBuilder.scala | Cubierto por contratos concretos | 002.2 |
| javadsl/BehaviorBuilder.scala | Cubierto por contratos concretos | 003 |
| javadsl/AbstractMatchingBehavior.scala | Cubierto por contratos concretos | 004 (todo no afirmado por spec) |
| javadsl/ActorContext.scala | Duplicado o fachada equivalente (+ propio) | 005.1 (espejo), 005.2, 005.3, 005.4 |
| javadsl/TimerScheduler.scala | Duplicado o fachada equivalente (+ startPeriodicTimer propio) | 006.1 |
| javadsl/StashBuffer.scala (+ StashOverflowException) | Duplicado o fachada equivalente | 006.2 |
| javadsl/Routers.scala | Duplicado o fachada equivalente | 006.3 |
| javadsl/AskPattern.scala | Duplicado o fachada equivalente | 006.4 |
| javadsl/Adapter.scala | Duplicado o fachada equivalente | 006.5 (sujeto a Q-X-8) |
| scaladsl/adapter/package.scala, PropsAdapter.scala | Cubierto por contratos concretos | AT-ADP-001..005 (fuera de este grupo; 006.5 los cita) |
| actor-typed/src/main/java | Fuera de estos modulos con frontera explicita | No existe en 1.7.1 |
| test/java javadsl/{ReceiveBuilderTest, BehaviorBuilderTest, InterceptTest, WatchTest, ActorCompile, ActorContextAskTest, ActorContextPipeToSelfTest, ActorLoggingTest, StashBufferTest, RoutersTest, AdapterTest}.java | Cubierto por contratos concretos | citados en 001..006 con su alcance real |
| test/scala javadsl/AdaptationFailureSpec.scala | Cubierto por contratos concretos | 002.1/002.2 (ojo: tambien deberia citarse en AT-SIG-007; ahi se cita la version scaladsl) |
| test/java ExtensionsTest, ActorSystemTest, SchedulerTest | Cubierto por contratos concretos | AT-EXT-*, AT-CS-001/AT-SYS-005, AT-SYS-009 |
| test/java DispatcherSelectorTest, MailboxSelectorTest | Pendiente de analisis | Solo comprueban que las fabricas se ven desde Java (compilacion); revisar contra AT-DSP-002/003 (grupo DSP/ADP-LOG-DSP) |
| test/java eventstream/{EventStreamTest, EventStreamDocTest, EventStreamSuperClassDocTest}, receptionist/ReceptionistApiTest | Pendiente de analisis | Pertenecen a AT-EVT-* y AT-RCP-*; no leidos aqui |
| test/java jdocs/** (Aggregator, AggregatorTest, BubblingSample(+Test), DispatchersDocTest, FSMDocTest, GracefulStopDocTest, InteractionPatterns*Test, IntroTest, LoggingDocExamples, MailboxDocTest, OOIntroTest, OnMessageIntroTest, RouterTest, SpawnProtocolDocTest, StashDoc*, StyleGuideDocExamples, TailChopping, coexistence/*, extensions/ExtensionDocTest, fromclassic/*, pubsub/PubSubExample, supervision/SupervisionCompileOnlyTest) | Detalle interno sin contrato independiente (ejemplos de documentacion), excepto los citados | Ejemplos que repiten reglas ya contratadas (typed-core cierre); `OnMessageIntroTest` y `SpawnProtocolDocTest` se citan como evidencia en AT-JAPI-004 y AT-SPV-001. Los jdocs con @Test (AggregatorTest, BubblingSampleTest, InteractionPatterns*Test, MailboxDocTest, RouterTest, StashDocTest, coexistence/*) no se leyeron: **Pendiente de analisis** hasta contrastarlos con AT-ASK/AT-STASH/AT-RT/AT-ADP; no se declaran fuera de alcance |

### Tabla TK (`actor-testkit-typed/src/main`, 47 archivos incl. `.excludes`)

| Archivo | Clasificacion | Fila / justificacion |
|---|---|---|
| annotations/JUnit5TestKit.java | Cubierto | AT-TK-009.3 |
| mima-filters/1.3.x.backwards.excludes/add-ask-behavior-methods.excludes | Detalle interno sin contrato independiente | filtro de compatibilidad binaria de MiMa; sin comportamiento de ejecucion |
| resources/reference.conf | Cubierto | AT-TK-010 |
| CapturedLogEvent.scala | Cubierto | AT-TK-003.5 |
| Effect.scala | Cubierto | AT-TK-004.1..004.6 |
| FishingOutcome.scala | Cubierto | AT-TK-002.2 |
| LoggingEvent.scala | Cubierto | AT-TK-006.1 |
| TestException.scala | Cubierto | AT-TK-014 (fila nueva) |
| TestKitSettings.scala | Cubierto | AT-TK-010, AT-TK-002.6 |
| internal/ActorSystemStub.scala | Cubierto | AT-TK-011 |
| internal/BehaviorTestKitImpl.scala | Cubierto | AT-TK-003.1..003.4, 003.7 |
| internal/CapturingAppender.scala | Cubierto | AT-TK-006.5 |
| internal/ControlledExecutor.scala | Cubierto | AT-TK-012 |
| internal/DebugRef.scala | Cubierto | AT-TK-012 |
| internal/EffectfulActorContext.scala | Cubierto | AT-TK-004.1..004.6 |
| internal/LogbackUtil.scala | Cubierto | AT-TK-013 |
| internal/LoggingTestKitImpl.scala | Cubierto | AT-TK-006.1, 006.3 |
| internal/StubbedActorContext.scala | Cubierto | AT-TK-003.4, 003.5, 004.x, 005.1 (`FunctionRef`) |
| internal/TestAppender.scala | Cubierto | AT-TK-006.4 |
| internal/TestInboxImpl.scala | Cubierto | AT-TK-005.1..005.3 |
| internal/TestKitUtils.scala | Cubierto | AT-TK-001.1..001.3 (`ActorTestKitGuardian` en 001.2) |
| internal/TestProbeImpl.scala | Cubierto | AT-TK-002.1..002.6 |
| javadsl/ActorTestKit.scala | Duplicado o fachada equivalente | fachada de scaladsl/ActorTestKit (001.x, 008) con `create(..)`, `createTestProbe(Class)`, `stop(Duration)`; `ActorTestKitApiTest.java:26-46` compile-only |
| javadsl/BehaviorTestKit.scala | Duplicado o fachada equivalente | AT-TK-003.x (`create`, `getEffect`, `getAllEffects`, `expectEffectClass`, `getAllLogEntries`, `runAsk(Class, f)`) |
| javadsl/Effects.scala | Duplicado o fachada equivalente | AT-TK-004.7 |
| javadsl/JUnit5TestKitBuilder.scala | Cubierto | AT-TK-009.3 (es la implementacion real, no fachada) |
| javadsl/LogCapturing.scala | Cubierto | AT-TK-006.5 |
| javadsl/LogCapturingExtension.scala | Cubierto | AT-TK-013 |
| javadsl/LoggingTestKit.scala | Duplicado o fachada equivalente | AT-TK-006.1/006.2 (`withMdc(java.util.Map)`, `withCause(Class)`, `expect(system, Supplier)`) |
| javadsl/ManualTime.scala | Duplicado o fachada equivalente | AT-TK-007 (`ManualTime.get`) |
| javadsl/SerializationTestKit.scala | Duplicado o fachada equivalente | AT-TK-009.4 |
| javadsl/TestInbox.scala | Duplicado o fachada equivalente | AT-TK-005.1 (`getRef`, `getAllReceived`, `create`) |
| javadsl/TestKitJUnit5Extension.scala | Cubierto | AT-TK-009.3 |
| javadsl/TestKitJunitResource.scala | Cubierto | AT-TK-009.2 |
| javadsl/TestProbe.scala | Duplicado o fachada equivalente | AT-TK-002.x (`receiveSeveralMessages`, `getRemaining*`, `awaitAssert(Creator)`) |
| scaladsl/ActorTestKit.scala | Cubierto | AT-TK-001.1..001.3, 008, 009.4 |
| scaladsl/ActorTestKitBase.scala | Cubierto | AT-TK-009.1 |
| scaladsl/BehaviorTestKit.scala | Cubierto | AT-TK-003.x |
| scaladsl/Effects.scala | Cubierto | AT-TK-004.7 |
| scaladsl/LogCapturing.scala | Cubierto | AT-TK-006.5 |
| scaladsl/LoggingTestKit.scala | Cubierto | AT-TK-006.1..006.3 |
| scaladsl/ManualTime.scala | Cubierto | AT-TK-007 |
| scaladsl/ScalaTestWithActorTestKit.scala | Cubierto | AT-TK-009.1 |
| scaladsl/SerializationTestKit.scala | Cubierto | AT-TK-009.4 |
| scaladsl/TestInbox.scala | Cubierto | AT-TK-005.1..005.3 |
| scaladsl/TestProbe.scala | Cubierto | AT-TK-002.x (`FishingOutcomes`, `TestProbe.apply` nombre `testProbe`) |
| scaladsl/package.scala | Cubierto | AT-TK-002.6 (`TestDuration.dilated`) |

Detalle interno sin contrato independiente (dentro de archivos cubiertos): `StubbedActorContext.unhandledMessages/clearUnhandled/removeChildInbox` (hooks internos; no estan en la API de `BehaviorTestKit`, solo `onUnhandled` lo alimenta; ver AT-CTX-018) y `BehaviorTestKitImpl.as[U]` (cast de tipos).

### Tabla TK (`actor-testkit-typed/src/test`, 28 archivos)

| Archivo | Clasificacion | Fila / justificacion |
|---|---|---|
| java/jdocs/.../AsyncTestingExampleTest.java | Duplicado o fachada equivalente | espejo Java de AsyncTestingExampleSpec; leido: AT-TK-001.2 A5, 002.1 |
| java/jdocs/.../JUnit5IntegrationExampleTest.java | Cubierto | AT-TK-009.3 A6 |
| java/jdocs/.../JunitIntegrationExampleTest.java | Cubierto | AT-TK-009.2 A3 |
| java/jdocs/.../LogCapturingExampleTest.java | Cubierto | AT-TK-006.5 A2 |
| java/jdocs/.../LogCapturingExtensionExampleTest.java | Cubierto | AT-TK-013 A1 |
| java/jdocs/.../ManualTimerExampleTest.java | Cubierto | AT-TK-007 A4 |
| java/jdocs/.../SyncTestingExampleTest.java | Duplicado o fachada equivalente | espejo Java de SyncTestingExampleSpec; usado en AT-TK-003.1/.4/.5, 004.1 (`"$a"` solo aqui) |
| java/jdocs/.../TestConfigExample.java | Detalle interno sin contrato independiente | snippet de documentacion de `ConfigFactory` (sin aserciones); la config del testkit se cubre en AT-TK-001.1/010 |
| java/.../javadsl/ActorTestKitApiTest.java | Duplicado o fachada equivalente | compile-only de la superficie Java (AT-TK-001/002 fachadas) |
| java/.../javadsl/ActorTestKitJUnit5Test.java | Cubierto | AT-TK-009.3, 001.1 A11, 013 |
| java/.../javadsl/ActorTestKitTest.java | Cubierto | AT-TK-001.1 A11, 009.2 |
| java/.../javadsl/BehaviorTestKitTest.java | Cubierto | AT-TK-003.x, 004.x |
| java/.../javadsl/LoggingTestKitTest.java | Cubierto | AT-TK-006.1 A7, AT-TK-014 |
| java/.../javadsl/TestProbeTest.java | Cubierto | AT-TK-002.1/.2/.4/.5 |
| resources/application-test.conf | Cubierto | AT-TK-001.1 A7 |
| resources/application.conf | Cubierto | AT-TK-001.1 A7 (ausencia de `test.from-application`), 009.3 A3 (`test.value`) |
| resources/logback-test.xml | Cubierto | AT-TK-006.5 (config requerida por LogCapturing) |
| scala/docs/.../AsyncTestingExampleSpec.scala | Cubierto | AT-TK-001.2 A4 |
| scala/docs/.../ManualTimerExampleSpec.scala | Cubierto | AT-TK-007 A1-A3 |
| scala/docs/.../ScalaTestIntegrationExampleSpec.scala | Cubierto | AT-TK-009.1 A4, 006.5 A2 |
| scala/docs/.../SyncTestingExampleSpec.scala | Cubierto | AT-TK-003.1/.4/.5, 004.1, 004.2 |
| scala/docs/.../TestConfigExample.scala | Detalle interno sin contrato independiente | snippet de documentacion de `ConfigFactory` |
| scala/.../ActorTestKitSpec.scala | Cubierto | AT-TK-001.x, 008, 009.1 |
| scala/.../BehaviorTestKitSpec.scala | Cubierto | AT-TK-003.x, 004.x, 005.x, 011 |
| scala/.../GreeterMain.scala | Detalle interno sin contrato independiente | fixture de JUnit5TestKitBuilderSpec (`withSystem`) |
| scala/.../JUnit5TestKitBuilderSpec.scala | Cubierto | AT-TK-009.3 |
| scala/.../LoggingEventFilterSpec.scala | Cubierto | AT-TK-006.1 (clase real `LoggingTestKitSpec`) |
| scala/.../TestAppenderSpec.scala | Cubierto | AT-TK-006.3, 006.4 |
| scala/.../TestProbeSpec.scala | Cubierto | AT-TK-002.x, 010 |

Sin "Pendiente de analisis". Nota: `actor-typed-tests/src/test` (142 archivos, 63 sin cita) no es de este grupo; se usaron `DispatcherSelectorSpec`, `LocalPubSubSpec`, `SupervisionSpec`, `LogMessagesSpec`, `LocalActorRefProviderLogMessagesSpec` y `MailboxSelectorSpec` como evidencia de uso de testkit (frontera: su contrato pertenece a otras filas).

### Clasificación de decisiones (resumen)

| Decisión | Clasificación | Filas afectadas (expandidas) | Resuelta leyendo/midiendo Pekko | Requiere al Owner |
|---|---|---|---|---|
| Q-T-ASK-1 | Representacion que conserva el contrato | 2 | No | No (representación) |
| Q-T-ASK-2 | Divergencia funcional minima, resuelta por lectura: la restriccion es accidental | 1 | Sí (restricción accidental) | Divergencia mínima: sí, confirmar |
| Q-T-RCP-1 | Divergencia funcional controlada | 3 | No | Sí (divergencia) |
| Q-T-RCP-2 | Arquitectura | 1 | No | Sí (arquitectura) |
| Q-T-RCP-3 | Incertidumbre resuelta leyendo la fuente: el comportamiento es el descrito | 3 | Sí (LocalReceptionist.scala:195-218) | No, salvo que quiera divergir |
| Q-T-RT-1 | Divergencia funcional controlada | 1 | No | Sí (divergencia) |
| Q-T-RT-2 | Incertidumbre resuelta leyendo la fuente + representacion | 3 | Sí (RoutingLogic.scala:50-122) | No |
| Q-T-RT-3 | Arquitectura; depende de AT-BEH-023 | 4 | No | Con AT-BEH-023 |
| Q-T-ADP-2 | Arquitectura; depende de Q-X-8 | 5 | No | Con Q-X-8 |
| Q-T-ADP-3 | Divergencia funcional evitada: se conserva | 3 | No | No (se conserva) |
| Q-T-EVT-1 | Incertidumbre resuelta leyendo la fuente; queda una divergencia a decidir | 3 | Sí, la parte factual (ActorRef.scala:736-743); queda decidir si se unifica | Sí |
| Q-T-LOG-1 | Representacion que conserva el contrato | 9 | No | Sí (representación) |
| Q-T-LOG-2 | Representacion | 1 | No | Sí (representación) |
| Q-T-LOG-3 | Exclusion de alcance parcial / representacion | 1 | No | Con Q-X-7 |
| Q-T-LOG-4 | Exclusion de alcance con subsuncion | 1 | No | Sí (exclusión) |
| Q-T-SPV-1 | Arquitectura / representacion | 1 | No | Sí (API) |
| Q-T-EXT-1 | Incertidumbre resuelta leyendo la fuente + representacion | 1 | Sí (ExtensionsImpl.scala:39-66) | No |
| Q-T-EXT-2 | Representacion | 2 | No | No |
| Q-T-EXT-3 | Incertidumbre resuelta leyendo la fuente | 1 | Sí (ExtensionsImpl.scala:110-114,134) | No |
| Q-T-DSP-1 | Representacion que conserva el contrato | 2 | No | No |
| Q-T-DSP-2 | Arquitectura; depende de Q-X-4 | 1 | No | Con Q-X-4 |
| Q-T-DSP-3 | Divergencia funcional evitada: paridad | 1 | No | Con Q-X-4 |
| Q-T-DLV-1 | Exclusion de alcance acotada + arquitectura | 22 | Sí (alcance y descomposición) | Sí (alcance acotado) |
| Q-T-DLV-2 | Arquitectura / exclusion acotada | 4 | No | Sí |
| Q-T-DLV-3 | Representacion / exclusion de alcance si (b) | 1 | No | Sí |
| Q-T-SER-1 | Arquitectura / divergencia funcional | 9 | No | Sí |
| Q-T-SER-2 | Divergencia funcional controlada | 1 | No | Sí |
| Q-T-SER-3 | Exclusion de alcance o divergencia; requiere Owner si (a) | 1 | No | Sí (exclusión) |
| Q-T-JAPI-1 | Exclusion de alcance parcial + divergencia funcional a decidir | 16 | No | Sí (exclusión) |
| Q-T-TK-1 | Arquitectura; depende de Q-X-4 + representacion de mensajes | 10 | No | Con Q-X-4 |
| Q-T-TK-2 | Representacion que conserva el contrato, con limites | 6 | No | Con Q-X-7 |
| Q-T-TK-3 | Exclusion de alcance con sustituto + incertidumbre resuelta | 6 | Parcial (nombre del sistema por pila) | Sí (exclusión de JUnit) |
| Q-T-TK-4 | Arquitectura; depende de Q-X-8 | 19 | No | Con Q-X-8 |
| Q-T-TK-5 | Divergencia funcional si (b) | 1 | Sí por lectura (EffectfulActorContext.scala:185-191); falta oráculo | Solo si se corrige |
| Q-T-JAPI-2 | Divergencia funcional | 5 | No | Sí |
| Q-T-DLV-4 | Arquitectura de validacion | 6 | No | No (metodología de validación) |

No se marcó ninguna decisión como aceptada. Las que de verdad requieren al Owner, por número de filas que bloquean (ver columna 3): Q-T-LOG-1 (representación del logging, también condiciona TK-006), Q-T-SER-1 (serialización: chunking de delivery, SerializationTestKit), Q-T-JAPI-1/Q-T-JAPI-2 (fachada Java y defaults de señales: 18 filas JAPI), Q-T-DLV-2 (cola durable), Q-T-EVT-1 (Dropped), Q-T-TK-3 (exclusión de JUnit), Q-T-RCP-1/Q-T-RT-1/Q-T-SER-2 (divergencias), Q-T-LOG-4/Q-T-SER-3 (exclusiones). Las dependientes de decisiones transversales (Q-X-4, Q-X-7, Q-X-8) no se repiten aquí.
