# Contratos verificables: actor-typed, grupo typed-extended

Fuente de verdad: `upstream/pekko` fijado en `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (v1.7.1). Unidades: ASK, RCP, RT, ADP, EVT, LOG, PS, SPV, EXT, DSP, DLV, CS, SER, JAPI, TK. Todas las citas son `path:linea` relativas a `upstream/pekko` y se verificaron con `sed -n` (ver script de verificacion en el cierre del informe). Decisiones transversales: Q-X-1..Q-X-9 (orquestador); especificas en `typed-extended.decisions.tsv`.

Convencion de aserciones: `A<n>` indica el comportamiento y el `path:linea` del cuerpo del test de Pekko que lo afirma; las que no tienen test se marcan "no afirmada por spec" y citan el codigo.

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

- **Contrato observable**: Contractual: askWithStatus aplana StatusReply: Success(v) completa con v; Error(ex) falla el Future con ex (ErrorMessage(texto) para error(String), la excepcion dada para error(Throwable)); StatusReply.success/error/ack, getValue/getError/isError/isSuccess; toString "Success(v)" / "Error(<msg>)"; StatusReply con status null lanza InvalidMessageException "[null] is not an allowed status"; getError sobre un exito lanza IllegalArgumentException "Expected reply to be a failure, but was a success".
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/AskPattern.scala:135-137; actor/src/main/scala/org/apache/pekko/pattern/StatusReply.scala:37-116,173-183
- **Condiciones**: Normal: Success(v) -> Ok(v). Errores: Error(ex) -> Err(ex); un exito de otro tipo dentro del Future -> IllegalArgumentException "Unexpected status reply success value: <v>" (actor/src/main/scala/org/apache/pekko/pattern/StatusReply.scala:179-180, inalcanzable con tipos); fallos del ask (timeout) pasan intactos (182). ErrorMessage es RuntimeException sin stacktrace cuyo toString es el texto (112-116). Ack = success(Done) (77).
- **Orden / concurrencia / ciclo de vida**: flattenStatusFuture usa un contexto parasitico: la transformacion corre en el hilo que completa el Future.
- **Dependencias**: filas: AC-ASK; AC-UTIL (Done); AT-ASK-001. Externas: Scala Future/Promise y TimeoutException de la stdlib; PromiseActorRef del clasico.
- **Equivalente Rust propuesto**: enum StatusReply<T>{Success(T),Error(StatusError)}; StatusError::Message(String)|Custom(Box<dyn Error+Send+Sync>); StatusReply::ack()->StatusReply<Done>; ask_with_status(). **Diferencias observables**: Rust no tiene excepciones: el error personalizado se expresa como Box<dyn Error>; la identidad "es una instancia de ErrorMessage" se pierde salvo que StatusError::Message sea un tipo propio (se propone variante).
- **Aserciones**:
  - A1. Success("goodie") entrega "goodie" — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/AskSpec.scala:267-272
  - A2. error("boom") falla el Future con StatusReply.ErrorMessage y getMessage == "boom" — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/AskSpec.scala:273-279
  - A3. error(TestException("boom")) falla con esa clase de excepcion y mensaje "boom" — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/AskSpec.scala:281-287
  - A4. StatusReply null -> InvalidMessageException "[null] is not an allowed status" (no afirmada por spec; derivada de actor/src/main/scala/org/apache/pekko/pattern/StatusReply.scala:38-40)
  - A5. toString Success(v)/Error(msg) (no afirmada por spec; actor/src/main/scala/org/apache/pekko/pattern/StatusReply.scala:65-68)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh (StatusReply es clase de pekko-actor, mismo jar de runtime). Procedencia: jar oficial.
- **Decisiones**: Q-X-9

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

- **Contrato observable**: Interno y estructural: aroundReceive (con thread de actor fijado y MDC limpiado al terminar) traduce classic.Terminated en Terminated o ChildFailed (si el ref esta en failures), classic.ReceiveTimeout en el mensaje de receive-timeout del contexto, AdaptMessage (ask/pipeToSelf/spawnMessageAdapter) aplicando la funcion de adaptacion y despues routeando, AdaptWithRegisteredMessageAdapter al primer adaptador cuya clase acepte el mensaje (si ninguno: unhandled), Signal a handleSignal y el resto como mensaje; los TimerMsg se filtran con interceptTimerMsg (descartando los obsoletos).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:87-123,125-150,182-196; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:39-362
- **Condiciones**: Ver contrato. Errores: excepciones del Behavior se propagan al supervisor; UnstashException se desenvuelve restaurando el behavior (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:152-162).
- **Orden / concurrencia / ciclo de vida**: Todo ocurre en el hilo del actor (setCurrentActorThread/clear en finally, MDC limpiado al terminar cada mensaje).
- **Dependencias**: filas: AC-CELL; AC-SUP; AC-DW; AC-MBX. Externas: Runtime clasico completo (pekko-actor): ActorCell, supervision, ActorSystemImpl.
- **Equivalente Rust propuesto**: trait/enum de envelope interno: Msg(T) | Signal(S) | Adapted(Box<dyn Any>) | ReceiveTimeout | TimerFired(key,gen). **Diferencias observables**: En Rust el interprete no necesita ser un actor clasico; las reglas (orden de hooks, PostStop sobre behavior previo, ChildFailed) deben conservarse.
- **Aserciones**:
  - A1. Terminated clasico -> Terminated, o ChildFailed si el hijo fallo y era observado (no afirmada por spec leido; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:94-101)
  - A2. mensaje cuyo adaptador no existe cae a unhandled (no afirmada por spec; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:185-187)
  - A3. adaptacion que devuelve null se ignora con warn (no afirmada por spec; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:212-217); una excepcion NonFatal en la adaptacion entrega MessageAdaptionFailure (derivada; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:203-210)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh mediante escenarios de orden de senales/hooks. Procedencia: jar oficial.
- **Decisiones**: Q-X-8;Q-T-ADP-2;Q-T-ADP-3

## AT-ADP-006.2: ActorAdapter: Resultado del Behavior (next): unhandled, failed, stopped (internal-required)

- **Contrato observable**: Interno y estructural: next(b,msg) segun la etiqueta del Behavior: UnhandledBehavior -> unhandled(msg) (clasico); FailedBehavior -> si rethrowTypedFailure lanza TypedActorFailedException(cause) (el padre decide) sino context.stop(self); StoppedBehavior -> behavior = ComposedStoppingBehavior(behavior, stopped) y context.stop(self) (PostStop se entrega a lastBehavior y luego se corre onPostStop del stopped; resultado empty); cualquier otro: canonicalize.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:164-179,340-362; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:39-362
- **Condiciones**: Ver contrato. Errores: excepciones del Behavior se propagan al supervisor; UnstashException se desenvuelve restaurando el behavior (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:152-162).
- **Orden / concurrencia / ciclo de vida**: Todo ocurre en el hilo del actor (setCurrentActorThread/clear en finally, MDC limpiado al terminar cada mensaje).
- **Dependencias**: filas: AC-CELL; AC-SUP; AC-DW; AC-MBX. Externas: Runtime clasico completo (pekko-actor): ActorCell, supervision, ActorSystemImpl.
- **Equivalente Rust propuesto**: fn handle_next(Behavior)->ActorAction{Continue,Unhandled,Fail(cause),Stop(onPostStop)}. **Diferencias observables**: En Rust el interprete no necesita ser un actor clasico; las reglas (orden de hooks, PostStop sobre behavior previo, ChildFailed) deben conservarse.
- **Aserciones**:
  - A1. unhandled publica UnhandledMessage — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorContextDelegateSpec.scala:82-99 (AT-EVT-003)
  - A2. ComposedStoppingBehavior solo acepta PostStop (IllegalArgumentException "The ComposedStoppingBehavior should only ever receive a PostStop signal, but received <msg>") — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:348-352 (derivada)
  - A3. receive sobre ComposedStoppingBehavior lanza IllegalStateException "Stopping, should never receive a message" — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:346-347 (derivada)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh mediante escenarios de orden de senales/hooks. Procedencia: jar oficial.
- **Decisiones**: Q-X-8;Q-T-ADP-2;Q-T-ADP-3

## AT-ADP-006.3: ActorAdapter: Hooks de ciclo de vida: preStart, preRestart, postRestart, postStop (internal-required)

- **Contrato observable**: Interno y estructural: preStart arranca el Behavior (start + validateAsInitial) y detiene el actor si ya no esta vivo; preRestart cancela timers, entrega PreRestart y deja behavior = stopped (el estado se reconstruye en postRestart); postRestart cancela timers y reinicia con start+validateAsInitial; postStop cancela timers, entrega PostStop salvo que el behavior sea Deferred (setup no iniciado) y deja stopped; aroundPreStart/Restart/PostStop fijan el hilo de actor.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:275-335; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:39-362
- **Condiciones**: Ver contrato. Errores: excepciones del Behavior se propagan al supervisor; UnstashException se desenvuelve restaurando el behavior (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:152-162).
- **Orden / concurrencia / ciclo de vida**: Todo ocurre en el hilo del actor (setCurrentActorThread/clear en finally, MDC limpiado al terminar cada mensaje).
- **Dependencias**: filas: AC-CELL; AC-SUP; AC-DW; AC-MBX. Externas: Runtime clasico completo (pekko-actor): ActorCell, supervision, ActorSystemImpl.
- **Equivalente Rust propuesto**: Mapear a hooks del ciclo de vida del actor Rust: on_start/on_pre_restart/on_post_restart/on_stop. **Diferencias observables**: En Rust el interprete no necesita ser un actor clasico; las reglas (orden de hooks, PostStop sobre behavior previo, ChildFailed) deben conservarse.
- **Aserciones**:
  - A1. preRestart entrega PreRestart antes del reinicio (spawn con supervise(restart) -> PreRestart) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/coexistence/ClassicSupervisingTypedSpec.scala:94-103
  - A2. postStop no entrega PostStop a un Deferred sin iniciar — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:328-331 (derivada); caso "terminate immediately" — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.scala:126-135
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh mediante escenarios de orden de senales/hooks. Procedencia: jar oficial.
- **Decisiones**: Q-X-8;Q-T-ADP-2;Q-T-ADP-3

## AT-ADP-006.4: ActorAdapter: Supervision de hijos y registro de fallos (ChildFailed) (internal-required)

- **Contrato observable**: Interno y estructural: supervisorStrategy OneForOne(loggingEnabled=false): TypedActorFailedException(cause) -> registra el fallo (si el padre observa al hijo) y Stop; otra excepcion -> registra fallo, loguea error (mensaje de la causa si es ActorInitializationException con causa) y Stop si el hijo es un ActorAdapter (typed) o el default clasico si no; la entrada en failures se consume al llegar el Terminated y produce ChildFailed(ref, ex).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:83,94-100,232-273; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:39-362
- **Condiciones**: Ver contrato. Errores: excepciones del Behavior se propagan al supervisor; UnstashException se desenvuelve restaurando el behavior (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:152-162).
- **Orden / concurrencia / ciclo de vida**: Todo ocurre en el hilo del actor (setCurrentActorThread/clear en finally, MDC limpiado al terminar cada mensaje).
- **Dependencias**: filas: AC-CELL; AC-SUP; AC-DW; AC-MBX. Externas: Runtime clasico completo (pekko-actor): ActorCell, supervision, ActorSystemImpl.
- **Equivalente Rust propuesto**: Tabla ref_hijo->causa, consumida en Terminated. **Diferencias observables**: En Rust el interprete no necesita ser un actor clasico; las reglas (orden de hooks, PostStop sobre behavior previo, ChildFailed) deben conservarse.
- **Aserciones**:
  - A1. hijo clasico supervisado por padre typed (Restart por defecto) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpec.scala:317-330
  - A2. excepcion no manejada se loguea con el mensaje de la excepcion — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpec.scala:361-369
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh mediante escenarios de orden de senales/hooks. Procedencia: jar oficial.
- **Decisiones**: Q-X-8;Q-T-ADP-2;Q-T-ADP-3

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

## AT-ADP-010: Aislamiento del guardian: user guardian detenido termina el sistema (GuardianStartupBehavior) (internal-required)

- **Contrato observable**: Interno: el user guardian se envuelve en GuardianStartupBehavior: espera Start (acumulando hasta 1000 mensajes en un Vector; al llegar a 1000 lanza StashOverflowException "Guardian Behavior did not receive start and buffer is full."), luego desencola con withStash(1000) y corre el guardian bajo GuardianStopInterceptor que llama system.terminate() cuando el guardian deja de estar vivo (para ejecutar CoordinatedShutdown).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/GuardianStartupBehavior.scala:31-90
- **Condiciones**: Ver contrato.
- **Orden / concurrencia / ciclo de vida**: Start se envia tras crear el sistema; mensajes anteriores se guardan en orden.
- **Dependencias**: filas: AT-SYS-*; AT-CS-001; AT-STASH-*. Externas: Runtime clasico completo (pekko-actor): ActorCell, supervision, ActorSystemImpl.
- **Equivalente Rust propuesto**: Arranque del sistema: buffer previo a Start y terminate al detenerse el guardian. **Diferencias observables**: Con runtime nativo (Q-X-8 b) el guardian no necesita Start diferido.
- **Aserciones**:
  - A1. guardian detenido muy rapido no cuelga — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpec.scala:210-235
  - A2. Start diferido y overflow del buffer (no afirmada por spec leida; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/GuardianStartupBehavior.scala:40-53)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-X-8

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

- **Contrato observable**: Contractual (doc + codigo): cada log del actor lleva MDC con pekkoSource (path del actor), pekkoAddress (addressString del provider, o pekko://<sistema> si no es adapter), sourceActorSystem (nombre del sistema) y pekkoTags (tags de Props separados por coma, solo si hay); MDC se limpia al terminar cada mensaje si se uso; withMdc agrega valores estaticos o por mensaje.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorMdc.scala:23-45; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorContextImpl.scala:44-90,173-178; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/WithMdcBehaviorInterceptor.scala; docs/src/main/paradox/typed/logging.md:130-163,224-251
- **Condiciones**: Claves exactas: "pekkoSource","sourceActorSystem","pekkoAddress","pekkoTags" (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorMdc.scala:24-27). pekkoTags solo si tagsString no vacio (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorMdc.scala:36-37).
- **Orden / concurrencia / ciclo de vida**: setMdc en cada acceso a context.log; clearMdc en finally de aroundReceive (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:119-122).
- **Dependencias**: filas: AT-DSP-004; AT-BEH (withMdc). Externas: SLF4J (Logger, MDC, Marker) y Logback/Slf4jLogger de pekko-slf4j.
- **Equivalente Rust propuesto**: Campos estructurados del span tracing por mensaje (pekko_source, pekko_address, source_actor_system, pekko_tags). **Diferencias observables**: Un thread-local MDC no sirve con async: span tracing; claves camelCase a mantener o no (Q-T-LOG-1).
- **Aserciones**:
  - A1. pass tags from props to MDC — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.scala:270-290 (cuerpo no leido a fondo)
  - A2. claves del MDC — actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorMdc.scala:24-27
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh (capturar MDC de un evento). Procedencia: jar oficial.
- **Decisiones**: Q-T-LOG-1

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

- **Contrato observable**: Contractual: helpers de Scala 2 sobre org.slf4j.Logger (trace2/traceN, debug2... ) que evitan la ambiguedad de varargs de SLF4J; los markers se pasan al backend.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/package.scala:30-120; docs/src/main/paradox/typed/logging.md
- **Condiciones**: Sin leer.
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: filas: AT-LOG-001. Externas: SLF4J (Logger, MDC, Marker) y Logback/Slf4jLogger de pekko-slf4j.
- **Equivalente Rust propuesto**: Subsumido por macros de tracing (sin varargs ambiguos). **Diferencias observables**: Q-T-LOG-4.
- **Aserciones**:
  - A1. markers llegan al backend — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.scala:175-189
  - A2. LoggerOps: no leido (actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/LoggerOpsSpec.scala:35-88)
- **Oraculo**: No aplica (idioma Scala sobre SLF4J).
- **Decisiones**: Q-T-LOG-4

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

- **Contrato observable**: Contractual: Behavior que, ante Spawn(behavior, name, props, replyTo), crea un hijo y responde con su ActorRef; name vacio o null -> spawnAnonymous; si el nombre esta tomado agrega sufijo "-1","-2",... (el primer candidato libre; el spec fija el formato aunque el Scaladoc lo declare indefinido); usable como guardian (hijos bajo /user) y con ask; no tiene mensaje de parada: se detiene el padre o por el protocolo del hijo.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/SpawnProtocol.scala:23-89; docs/src/main/paradox/typed/actor-lifecycle.md:125-158
- **Condiciones**: spawnWithUniqueName(c): c==0 -> name; si no name-c; recursa mientras ctx.child(candidato) exista (actor-typed/src/main/scala/org/apache/pekko/actor/typed/SpawnProtocol.scala:74-82). Nombre null o "" -> anonimo (70-71). Un nombre invalido propaga la excepcion de ctx.spawn (InvalidActorNameException) al actor SpawnProtocol (fallo del guardian).
- **Orden / concurrencia / ciclo de vida**: Responde con replyTo ! ref inmediatamente tras crear; Behaviors.same.
- **Dependencias**: filas: AT-CTX-002; AT-ASK-001. Externas: Ninguna externa.
- **Equivalente Rust propuesto**: struct Spawn<M>{behavior,name,props,reply_to:ActorRef<ActorRef<M>>} y enum SpawnCommand con borrado de tipo (Box<dyn Any>) segun Q-T-SPV-1. **Diferencias observables**: El existencial Spawn[T] choca con Rust: requiere borrado de tipo en el mensaje y downcast al responder (Q-T-SPV-1).
- **Aserciones**:
  - A1. hijo "child" bajo "parent": path.name == "child", parent.name == "parent" — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SpawnProtocolSpec.scala:47-57
  - A2. ask de Spawn devuelve el ref — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SpawnProtocolSpec.scala:59-68
  - A3. como guardian los hijos estan en /user/child1 y /user/child2 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SpawnProtocolSpec.scala:70-84
  - A4. nombre tomado: "child","child-1", y tras "child-2" explicito -> "child-3" — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SpawnProtocolSpec.scala:86-113
  - A5. BehaviorTestKit: efecto Spawned(target,"child") y anonimo cuando el nombre no esta definido — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SpawnProtocolSpec.scala:115-134
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh (nombres de hijos). Procedencia: jar oficial.
- **Decisiones**: Q-T-SPV-1

## AT-EXT-001.1: Extension / ExtensionId: identidad, una instancia por sistema y concurrencia (public)

- **Contrato observable**: Contractual: registerExtension(id) crea la extension una vez por sistema de forma concurrentemente segura (CountDownLatch + putIfAbsent; los demas hilos esperan); apply(system)/get(system) = registerExtension; la identidad de ExtensionId es por referencia (equals = eq, hashCode = identityHashCode), de modo que dos ids de la misma clase son distintos (MultiExtensionId(1) y (2)); createExtension "se invoca a lo sumo una vez por sistema".
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:78-136; actor-typed/src/main/scala/org/apache/pekko/actor/typed/Extensions.scala:118-137
- **Condiciones**: Un creador que accede a otras extensiones (p.ej. receptionist) desde el constructor funciona (AccessSystemFromConstructor, actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:254-265). createExtension devolviendo null lanza IllegalStateException "Extension instance created as 'null' for extension [<id>]" (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:103).
- **Orden / concurrencia / ciclo de vida**: Registro: putIfAbsent(latch) -> crear -> replace(latch, instancia) -> countDown (finally). Los lectores concurrentes bloquean en el latch (findExtension, 128-134).
- **Dependencias**: filas: AC-EXT. Externas: Typesafe Config (getStringList), DynamicAccess/reflexion JVM, CountDownLatch/ConcurrentHashMap.
- **Equivalente Rust propuesto**: ExtensionId<T> como valor estatico/Arc con identidad por puntero; registry Mutex<HashMap<id, OnceCell<Result<Arc<T>,Error>>>>. **Diferencias observables**: Rust: OnceCell/Mutex en lugar de latch; null no existe (Result).
- **Aserciones**:
  - A1. registerExtension dos veces, apply y get devuelven la misma instancia — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:90-102
  - A2. 1001 registros concurrentes de SlowExtension (25ms) devuelven la misma instancia — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:104-119
  - A3. dos ExtensionId de la misma clase -> instancias con n distinto (1 y 2) y estables — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:155-163
  - A4. crear una extension que usa log y receptionist en su constructor no falla — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:254-265
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh (identidad, una sola creacion bajo concurrencia). Procedencia: jar oficial.
- **Decisiones**: Q-T-EXT-2;Q-X-2

## AT-EXT-001.2: Extension: fallo de creacion cacheado (public)

- **Contrato observable**: Contractual: si createExtension lanza, la excepcion queda almacenada en el registro (replace(latch, throwable)) y cada acceso posterior (registerExtension, extension, hasExtension, apply) relanza la MISMA excepcion; un sistema no arranca si una extension de la configuracion falla al crearse.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:91-136
- **Condiciones**: replace(ext, latch, t) y throw t en createExtensionInstance (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:110-114); findExtension: case t: Throwable => throw t (134).
- **Orden / concurrencia / ciclo de vida**: countDown en finally garantiza que los hilos en espera continuan y ven el Throwable.
- **Dependencias**: filas: AC-EXT. Externas: Typesafe Config (getStringList), DynamicAccess/reflexion JVM, CountDownLatch/ConcurrentHashMap.
- **Equivalente Rust propuesto**: Registro con Result<Arc<T>,Arc<Error>> cacheado y clonado en cada acceso. **Diferencias observables**: Rust: error clonable (Arc<dyn Error>) para relanzar "la misma" excepcion.
- **Aserciones**:
  - A1. ActorSystem con extension que falla al crearse: RuntimeException, y vuelve a pasar en un segundo intento — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:136-153
  - A2. la misma excepcion en cada acceso (no afirmada por spec; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:134)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-T-EXT-3

## AT-EXT-001.3: Extension: extension() sin registro y hasExtension (public)

- **Contrato observable**: Contractual: extension(id) sin registrar lanza IllegalArgumentException "Trying to get non-registered extension [<id>]"; hasExtension(id) es true solo si hay instancia (o si hay un Throwable almacenado lo relanza); findExtension espera si la creacion esta en curso.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:78-89,128-136
- **Condiciones**: Error: IllegalArgumentException (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:81). hasExtension = findExtension(ext) != null (78).
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: filas: AC-EXT. Externas: Typesafe Config (getStringList), DynamicAccess/reflexion JVM, CountDownLatch/ConcurrentHashMap.
- **Equivalente Rust propuesto**: Result<Arc<T>,ExtensionError::NotRegistered> y has_extension(). **Diferencias observables**: Rust: Err en vez de excepcion; texto a conservar.
- **Aserciones**:
  - A1. hasExtension(DummyExtension1) true tras cargar por config — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:121-134
  - A2. extension() no registrada: IllegalArgumentException "Trying to get non-registered extension [..]" (no afirmada por spec; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:80-82)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh (jshell: extension de id no registrado). Procedencia: jar oficial.
- **Decisiones**: Q-T-EXT-2

## AT-EXT-002.1: Carga desde configuracion: pekko.actor.typed.library-extensions (public)

- **Contrato observable**: Contractual: library-extensions (default [] + Receptionist$ en actor-typed/src/main/resources/reference.conf:18-21; se extiende con +=) se cargan al arrancar y un fallo de CARGA (clase ausente: RuntimeException "While trying to load extension [<fqcn>]" con causa; no es ExtensionId: RuntimeException "[<fqcn>] is not an 'ExtensionId'") o de creacion hace fallar el arranque del sistema; el FQCN puede terminar en `$` (objeto Scala) o la clase exponer getInstance() estatico (Java).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:38-76; actor-typed/src/main/resources/reference.conf:5-21
- **Condiciones**: Orden: loadExtensionsFor("pekko.actor.typed.library-extensions", throwOnLoadFail=true) y luego "...extensions" (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:40-43).
- **Orden / concurrencia / ciclo de vida**: Se carga en el arranque del sistema, antes de aceptar mensajes.
- **Dependencias**: filas: AC-EXT; AC-CFG. Externas: Typesafe Config (getStringList), DynamicAccess/reflexion JVM, CountDownLatch/ConcurrentHashMap.
- **Equivalente Rust propuesto**: Lista de nombres sustituida por registro explicito (Q-X-2): SystemBuilder::library_extension(id). **Diferencias observables**: Registro explicito: no hay carga por nombre; el error de "clase ausente" se vuelve error de compilacion/registro.
- **Aserciones**:
  - A1. library-extensions con extension que falla -> RuntimeException — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:173-183
  - A2. library-extensions con extension inexistente -> RuntimeException — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:185-193
  - A3. las extensiones listadas se crean al arrancar (createCount > 0) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:165-171
  - A4. extension Java via getInstance() — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:195-202
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-X-2

## AT-EXT-002.2: Carga desde configuracion: pekko.actor.typed.extensions (public)

- **Contrato observable**: Contractual: extensions se cargan despues de library-extensions; un fallo de CARGA (clase ausente o no es ExtensionId) NO falla el arranque: se loguea error ("[<fqcn>] is not an 'ExtensionId', skipping..." o "While trying to load extension <fqcn>, skipping...") y sigue; un fallo de CREACION (createExtension lanza) si falla el arranque (RuntimeException), cacheado (ver EXT-001.2).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:45-66
- **Condiciones**: throwOnLoadFail=false solo cambia el tratamiento de Success(_) no-ExtensionId y Failure(problem) de la resolucion (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:56-65).
- **Orden / concurrencia / ciclo de vida**: Segunda pasada tras library-extensions.
- **Dependencias**: filas: AC-EXT; AC-CFG. Externas: Typesafe Config (getStringList), DynamicAccess/reflexion JVM, CountDownLatch/ConcurrentHashMap.
- **Equivalente Rust propuesto**: Registro explicito con politica "skip" solo para errores de resolucion de nombre. **Diferencias observables**: Sin carga por nombre, el caso "skip" desaparece salvo que el registro acepte nombres opcionales.
- **Aserciones**:
  - A1. extensions por configuracion cargan DummyExtension1 y SlowExtension — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:121-134
  - A2. extension que falla al crearse hace fallar el sistema (no se omite) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:136-153
  - A3. ExtensionSetup sustituye DummyExtension1 por DummyExtension1ViaSetup — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:236-252
  - A4. el caso "skip" por clase ausente no esta afirmado por spec (derivada de actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:56-65)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh (incluyendo el caso de clase ausente en extensions, a validar). Procedencia: jar oficial.
- **Decisiones**: Q-X-2;Q-T-EXT-1

## AT-EXT-002.3: Carga de extensiones tipadas desde el sistema clasico (LoadTypedExtensions) (public)

- **Contrato observable**: Contractual: el sistema clasico lista pekko.library-extensions += LoadTypedExtensions, que fuerza la creacion de la ActorSystemAdapter y la carga de las extensiones tipadas, de modo que las extensiones registradas se crean eagerly aun cuando el sistema se cree como clasico y solo despues se acceda con toTyped.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorSystemAdapter.scala:143-176; actor-typed/src/main/resources/reference.conf:35-36
- **Condiciones**: before == beforeCreation + 1 y after == before (actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:213-216).
- **Orden / concurrencia / ciclo de vida**: Eager en arranque.
- **Dependencias**: filas: AT-ADP-008; AC-EXT. Externas: Typesafe Config (getStringList), DynamicAccess/reflexion JVM, CountDownLatch/ConcurrentHashMap.
- **Equivalente Rust propuesto**: n/a si no hay sistema clasico (Q-X-8 b). **Diferencias observables**: Solo relevante con interop clasico.
- **Aserciones**:
  - A1. la extension listada se crea durante la construccion del sistema clasico y el acceso posterior no la recrea — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:204-220
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-X-8

## AT-EXT-003: ExtensionSetup / AbstractExtensionSetup (override en tests) (public)

- **Contrato observable**: Contractual: un ExtensionSetup(extId, createExtension) incluido en ActorSystemSetup reemplaza la fabrica de la extension: createExtensionInstance busca el primer setup cuyo extId == ext (igualdad de referencia) y lo usa en vez de ext.createExtension.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/Extensions.scala:170-187; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:96-101
- **Condiciones**: collectFirst sobre settings.setup.setups (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:97-100).
- **Orden / concurrencia / ciclo de vida**: En la creacion de la extension.
- **Dependencias**: filas: AC-EXT. Externas: Typesafe Config (getStringList), DynamicAccess/reflexion JVM, CountDownLatch/ConcurrentHashMap.
- **Equivalente Rust propuesto**: Builder del sistema acepta ExtensionOverride(id, factory). **Diferencias observables**: Ninguna.
- **Aserciones**:
  - A1. extension sustituida por setup: extension(...) es DummyExtension1ViaSetup y DummyExtension1(sys) devuelve la misma instancia — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:236-252
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

- **Contrato observable**: Contractual: ActorTags(tag, otros...) / create / apply(Set) agrega tags a Props; solo el primer ActorTags de la lista se aplica (PropsAdapter usa firstOrElse); los tags quedan en Deploy.tags del actor clasico y se publican como MDC pekkoTags separados por coma; tags null lanza IllegalArgumentException "Tags must not be null".
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/Props.scala:238-286; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/PropsAdapter.scala:53-55; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorMdc.scala:36-38
- **Condiciones**: tags vacios no agregan MDC. El orden de los tags en pekkoTags depende de Set (no determinista).
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: filas: AT-LOG-002. Externas: Dispatchers/Mailboxes clasicos y Typesafe Config.
- **Equivalente Rust propuesto**: ActorTags(Vec<String>) en Props. **Diferencias observables**: Q-T-LOG-1.
- **Aserciones**:
  - A1. pass tags from props to MDC — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.scala:270-290 (no leido a fondo)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-T-LOG-1

## AT-DSP-005: Dispatchers.lookup y ids (DefaultDispatcherId, InternalDispatcherId) (public)

- **Contrato observable**: Contractual: Dispatchers.lookup(selector) devuelve un ExecutionContextExecutor: default y sameAsParent -> system.dispatcher; fromConfig(path) -> system.dispatchers.lookup(path); selector desconocido -> RuntimeException "Unsupported dispatcher selector: <x>"; shutdown() no hace nada; ids "pekko.actor.default-dispatcher" y "pekko.actor.internal-dispatcher".
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/Dispatchers.scala:20-40; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorSystemAdapter.scala:95-103
- **Condiciones**: Ver contrato.
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: filas: AC-DSP. Externas: Dispatchers/Mailboxes clasicos y Typesafe Config.
- **Equivalente Rust propuesto**: Dispatchers::lookup(selector)->Executor. **Diferencias observables**: Q-X-4.
- **Aserciones**:
  - A1. derivada del codigo actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorSystemAdapter.scala:95-103 (no afirmada por spec leido)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-X-4

## AT-DLV-001: ProducerController (punto a punto, RequestNext, RegisterConsumer, MessageWithConfirmation) (api-may-change)

- **Contrato observable**: Contractual (Scaladoc, ApiMayChange): el productor recibe RequestNext(sendNextTo, askNextTo, currentSeqNr, confirmedSeqNr) y envia un mensaje por demanda; el flujo lo controla el consumidor; los no confirmados se guardan en memoria; con DurableProducerQueue se reenvian tras reinicio; askNextTo con MessageWithConfirmation responde SeqNr al confirmar o almacenar; RegisterConsumer vincula un ConsumerController; Start(producer).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/ProducerController.scala:66-311; actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/internal/ProducerControllerImpl.scala (855 lineas, no leido)
- **Condiciones**: Sin leer (ver notes).
- **Orden / concurrencia / ciclo de vida**: Sin leer.
- **Dependencias**: filas: AT-CTX-*; AT-TMR-*; AT-STASH-*; AT-RCP-*. Externas: Typesafe Config (getBytes/getDuration), serializacion (AC-SER) para chunking, scheduler/timers.
- **Equivalente Rust propuesto**: Behavior ProducerController<A> con mensajes Start/RequestNext/RegisterConsumer/MessageWithConfirmation; solo variante local. **Diferencias observables**: Q-T-DLV-1.
- **Aserciones**:
  - A1. contrato por Scaladoc actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/ProducerController.scala:66-311; sin cuerpos de test leidos
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh (pares ProducerController/ConsumerController locales). Procedencia: jar oficial.
- **Decisiones**: Q-T-DLV-1

## AT-DLV-002: ConsumerController (Start, Delivery, Confirmed, ventana de flujo, deteccion de perdidas, only-flow-control, DeliverThenStop) (api-may-change)

- **Contrato observable**: Contractual: entrega de a uno y espera Confirmed; pide ventana (flow-control-window=50, pide mas al usar la mitad) y reenvios (resend-interval-min/max); deduplica y detecta perdidas por seqNr; only-flow-control=true no reenvia perdidos; DeliverThenStop entrega lo pendiente y para; RegisterToProducerController; SequencedMessage.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/ConsumerController.scala:65-349; actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/internal/ConsumerControllerImpl.scala (743 lineas, no leido)
- **Condiciones**: Sin leer.
- **Orden / concurrencia / ciclo de vida**: Sin leer.
- **Dependencias**: filas: AT-CTX-*; AT-TMR-*; AT-STASH-*. Externas: Typesafe Config (getBytes/getDuration), serializacion (AC-SER) para chunking, scheduler/timers.
- **Equivalente Rust propuesto**: Behavior ConsumerController<A>. **Diferencias observables**: Q-T-DLV-1.
- **Aserciones**:
  - A1. contrato por Scaladoc actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/ConsumerController.scala:65-349; sin cuerpos de test leidos
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-T-DLV-1

## AT-DLV-003: WorkPullingProducerController (via Receptionist ServiceKey) (api-may-change)

- **Contrato observable**: Contractual: distribuye trabajo a workers registrados con una ServiceKey segun demanda (cada worker tiene su ConsumerController); reenvia no confirmados a otro worker si uno muere; GetWorkerStats(replyTo) -> WorkerStats(numberOfWorkers); Settings con bufferSize e internalAskTimeout; chunking no soportado (Settings lanza IllegalArgumentException "Chunked messages not implemented for work-pulling yet." si chunkLargeMessagesBytes > 0).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/WorkPullingProducerController.scala:114-258; actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/internal/WorkPullingProducerControllerImpl.scala (704 lineas, no leido)
- **Condiciones**: El unico error textual leido: Settings (actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/WorkPullingProducerController.scala:190-192).
- **Orden / concurrencia / ciclo de vida**: Sin leer.
- **Dependencias**: filas: AT-RCP-*; AT-DLV-001; AT-DLV-002. Externas: Typesafe Config (getBytes/getDuration), serializacion (AC-SER) para chunking, scheduler/timers.
- **Equivalente Rust propuesto**: Behavior WorkPullingProducerController<A>. **Diferencias observables**: Q-T-DLV-1.
- **Aserciones**:
  - A1. Settings con chunking > 0 -> IllegalArgumentException "Chunked messages not implemented for work-pulling yet." — actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/WorkPullingProducerController.scala:190-192 (no afirmada por spec)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-T-DLV-1

## AT-DLV-004: DurableProducerQueue (protocolo) (api-may-change)

- **Contrato observable**: Contractual: protocolo LoadState(replyTo)/StoreMessageSent(sent, replyTo -> StoreMessageSentAck(storedSeqNr))/StoreMessageConfirmed(seqNr, confirmationQualifier, timestampMillis); State(currentSeqNr, highestConfirmedSeqNr, confirmedSeqNr, unconfirmed): empty = State(1, 0, Map.empty, Vector.empty); addMessageSent(sent) -> currentSeqNr = sent.seqNr + 1 y agrega a unconfirmed; confirmed(seqNr, qualifier, ts) quita de unconfirmed los de ese qualifier con seqNr <= seqNr, sube highestConfirmedSeqNr (max) y registra (seqNr, ts) para el qualifier; cleanup(qualifiers) quita entradas; cleanupPartialChunkedMessages descarta chunks incompletos y recalcula currentSeqNr; la implementacion durable vive en persistence-typed (OUT).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/DurableProducerQueue.scala:41-246
- **Condiciones**: confirmed: filtra solo por (qualifier, seqNr <=); un mensaje con otro qualifier permanece. cleanupPartialChunkedMessages: caso primer+ultimo chunk = mensaje completo; una secuencia sin chunk final se descarta y currentSeqNr = ultimo completo + 1 (o highestConfirmedSeqNr + 1 si ninguno).
- **Orden / concurrencia / ciclo de vida**: Las operaciones de State son puras. Los comandos se envian al actor durable y se responden de forma asincrona.
- **Dependencias**: filas: AT-DLV-001; AT-DLV-005. Externas: Typesafe Config (getBytes/getDuration), serializacion (AC-SER) para chunking, scheduler/timers.
- **Equivalente Rust propuesto**: trait DurableProducerQueue<A>; struct DurableState<A>; implementacion en memoria opcional (Q-T-DLV-2). **Diferencias observables**: Serializacion de eventos (DeliverySerializable) solo relevante con persistencia.
- **Aserciones**:
  - A1. State.empty.addMessageSent(MessageSent(1,"a",...)): unconfirmed.head.message == "a" y currentSeqNr == 2; segundo: size 2 y currentSeqNr == 3 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/DurableProducerQueueSpec.scala:29-39
  - A2. confirmed(1,"",0): queda "b" y currentSeqNr 3 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/DurableProducerQueueSpec.scala:41-49
  - A3. cleanupPartialChunkedMessages descarta chunks sin ultimo: quedan "a", currentSeqNr 2; con chunk final queda a,d,e y currentSeqNr 4; con reinicio de seqNr 4 ("h") quedan a,d,e,h y currentSeqNr 5 — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/DurableProducerQueueSpec.scala:51-99
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh (State puro). Procedencia: jar oficial.
- **Decisiones**: Q-T-DLV-2

## AT-DLV-005: Chunking de mensajes grandes (api-may-change)

- **Contrato observable**: Contractual: con pekko.reliable-delivery.producer-controller.chunk-large-messages > 0 (bytes) el mensaje se serializa y se parte en ChunkedMessage(serialized, firstChunk, lastChunk, serializerId, manifest); el consumidor reensambla; no soportado en work-pulling (ver DLV-003); "off" = 0; valores > Int.MaxValue lanzan "Too large chunk-large-messages.".
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/internal/ChunkedMessage.scala; actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/ProducerController.scala:167-178,234; actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/internal/ProducerControllerImpl.scala (no leido)
- **Condiciones**: Settings: toRootLowerCase(chunk-large-messages) == "off" -> 0; otro -> getBytes con requiring(<= Int.MaxValue, "Too large chunk-large-messages.") (actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/ProducerController.scala:168-172).
- **Orden / concurrencia / ciclo de vida**: Sin leer.
- **Dependencias**: filas: AC-SER; AT-DLV-001; AT-DLV-004. Externas: Typesafe Config (getBytes/getDuration), serializacion (AC-SER) para chunking, scheduler/timers.
- **Equivalente Rust propuesto**: Requiere serializacion de mensajes (Q-T-SER-1). **Diferencias observables**: Q-T-SER-1.
- **Aserciones**:
  - A1. lectura de configuracion — actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/ProducerController.scala:167-178 (no afirmada por spec leido)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-T-SER-1

## AT-DLV-006: Configuracion pekko.reliable-delivery.* y Settings (api-may-change)

- **Contrato observable**: Contractual: claves y defaults de reference.conf: producer-controller.chunk-large-messages=off; durable-queue.request-timeout=3s, retry-attempts=10, resend-first-interval=1s; consumer-controller.flow-control-window=50, resend-interval-min=2s, resend-interval-max=30s, only-flow-control=false; work-pulling.producer-controller hereda de producer-controller con buffer-size=1000, internal-ask-timeout=60s, chunk-large-messages=off. Settings(system|config) lee esas rutas; withX devuelve copia; WorkPulling Settings valida chunking.
- **Fuente**: actor-typed/src/main/resources/reference.conf:70-131; actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/ProducerController.scala:154-248; actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/ConsumerController.scala:201-290; actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/WorkPullingProducerController.scala:151-212
- **Condiciones**: ConsumerController.Settings.toString imprime solo (flowControlWindow, resendIntervalMin, onlyFlowControl) (actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/ConsumerController.scala:288 aprox). work-pulling.producer-controller = ${pekko.reliable-delivery.producer-controller} y se sobreescribe.
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: filas: AC-CFG. Externas: Typesafe Config (getBytes/getDuration), serializacion (AC-SER) para chunking, scheduler/timers.
- **Equivalente Rust propuesto**: structs ProducerSettings/ConsumerSettings/WorkPullingSettings con Default y from_config. **Diferencias observables**: Parser HOCON propio (Q-X-1) debe soportar substitution ${...} como en work-pulling.
- **Aserciones**:
  - A1. defaults — actor-typed/src/main/resources/reference.conf:70-131 (no afirmada por spec)
  - A2. Settings lee las rutas indicadas — actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/ProducerController.scala:167-178; actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/ConsumerController.scala:214-221; actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/WorkPullingProducerController.scala:164-170
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh (valores efectivos de Settings). Procedencia: jar oficial.
- **Decisiones**: Q-X-1

## AT-DLV-007: Flight recorder (JFR) de actor y delivery (internal-accidental)

- **Contrato observable**: Interno: ActorFlightRecorder/DeliveryFlightRecorder emiten eventos JFR (solo JDK 9+, via scala-jdk-9/jfr/Events.scala y JFRActorFlightRecorder); noop si JFR no esta disponible.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorFlightRecorder.scala; actor-typed/src/main/scala-jdk-9/org/apache/pekko/actor/typed/internal/jfr/Events.scala; actor-typed/src/main/scala-jdk-9/org/apache/pekko/actor/typed/internal/jfr/JFRActorFlightRecorder.scala
- **Condiciones**: Sin leer.
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: filas: -. Externas: Typesafe Config (getBytes/getDuration), serializacion (AC-SER) para chunking, scheduler/timers.
- **Equivalente Rust propuesto**: Eventos tracing equivalentes (Q-T-DLV-3). **Diferencias observables**: Q-T-DLV-3.
- **Aserciones**:
  - A1. sin spec
- **Oraculo**: No aplica (JFR).
- **Decisiones**: Q-T-DLV-3

## AT-CS-001: CoordinatedShutdown en el ActorSystem tipado (reutiliza el clasico) (public)

- **Contrato observable**: Contractual: el ActorSystem tipado no define CoordinatedShutdown propio: terminate() ejecuta CoordinatedShutdown clasico con ActorSystemTerminateReason (si pekko.coordinated-shutdown.run-by-actor-system-terminate=off no lo corre pero igual termina el sistema); detener el guardian de usuario termina el sistema via GuardianStopInterceptor -> system.terminate() (asi corre CoordinatedShutdown); getWhenTerminated/whenTerminated se completan al terminar; terminate inmediato antes de arrancar el guardian es valido (el guardian no recibe PostStop).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorSystem.scala:111-160; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/GuardianStartupBehavior.scala:56-90; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorSystemAdapter.scala:117-126
- **Condiciones**: close-actor-system-timeout: terminar bloqueando (Await) lanza TimeoutException (docs actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorSystem.scala:149-156).
- **Orden / concurrencia / ciclo de vida**: terminate() es asincrono; whenTerminated se completa al final de la secuencia.
- **Dependencias**: filas: AT-SYS-*; AC-CS. Externas: CoordinatedShutdown clasico (AC-CS).
- **Equivalente Rust propuesto**: Delega en el CoordinatedShutdown portado (AC-CS). **Diferencias observables**: Ninguna si AC-CS se porta.
- **Aserciones**:
  - A1. guardian que devuelve Behaviors.stopped termina el sistema y CoordinatedShutdown(sys).shutdownReason() == Some(ActorSystemTerminateReason) — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.scala:74-89
  - A2. guardian detenido de inmediato: whenTerminated se completa — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.scala:91-101
  - A3. terminate(): guardian recibe PostStop ("done") y shutdownReason es ActorSystemTerminateReason — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.scala:103-124
  - A4. terminate antes de arrancar: no hay PostStop y es valido — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.scala:126-135
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

## AT-JAPI-001: javadsl.Behaviors (espejo Java de los factories) (public)

- **Contrato observable**: Contractual: misma semantica que scaladsl.Behaviors con tipos de japi.function y Class[T] en lugar de ClassTag.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/Behaviors.scala:1-422
- **Condiciones**: Sin leer.
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: filas: AT-BEH-*. Externas: JVM: japi.function, Optional, CompletionStage.
- **Equivalente Rust propuesto**: Sin fachada separada: una sola API Rust cubre la semantica (propuesta Q-T-JAPI-1); sin clases Java. **Diferencias observables**: Q-T-JAPI-1: no hay clases Java ni lambdas japi; la semantica coincide con las filas scaladsl equivalentes.
- **Aserciones**:
  - A1. mismas aserciones que las filas equivalentes de scaladsl; cuerpos Java no leidos
- **Oraculo**: No aplica como golden propio (la semantica se compara en las filas scaladsl).
- **Decisiones**: Q-T-JAPI-1

## AT-JAPI-002: javadsl.AbstractBehavior + Receive + ReceiveBuilder (public)

- **Contrato observable**: Contractual: estilo OO Java con createReceive y builder de casos por clase.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/AbstractBehavior.scala:1-94; actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/Receive.scala:1-68; actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/ReceiveBuilder.scala:1-232
- **Condiciones**: Sin leer.
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: filas: AT-BEH-*. Externas: JVM: japi.function, Optional, CompletionStage.
- **Equivalente Rust propuesto**: Sin fachada separada: una sola API Rust cubre la semantica (propuesta Q-T-JAPI-1); sin clases Java. **Diferencias observables**: Q-T-JAPI-1: no hay clases Java ni lambdas japi; la semantica coincide con las filas scaladsl equivalentes.
- **Aserciones**:
  - A1. mismas aserciones que las filas equivalentes de scaladsl; cuerpos Java no leidos
- **Oraculo**: No aplica como golden propio (la semantica se compara en las filas scaladsl).
- **Decisiones**: Q-T-JAPI-1

## AT-JAPI-003: javadsl.BehaviorBuilder (public)

- **Contrato observable**: Contractual: builder funcional Java de mensajes y senales por clase y predicado.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/BehaviorBuilder.scala:1-214
- **Condiciones**: Sin leer.
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: filas: AT-BEH-*. Externas: JVM: japi.function, Optional, CompletionStage.
- **Equivalente Rust propuesto**: Sin fachada separada: una sola API Rust cubre la semantica (propuesta Q-T-JAPI-1); sin clases Java. **Diferencias observables**: Q-T-JAPI-1: no hay clases Java ni lambdas japi; la semantica coincide con las filas scaladsl equivalentes.
- **Aserciones**:
  - A1. mismas aserciones que las filas equivalentes de scaladsl; cuerpos Java no leidos
- **Oraculo**: No aplica como golden propio (la semantica se compara en las filas scaladsl).
- **Decisiones**: Q-T-JAPI-1

## AT-JAPI-004: javadsl.AbstractMatchingBehavior (public)

- **Contrato observable**: Contractual: variante Java de menor overhead con onMessage/onSignal por pattern matching de Java.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/AbstractMatchingBehavior.scala:1-90
- **Condiciones**: Sin leer.
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: filas: AT-BEH-*. Externas: JVM: japi.function, Optional, CompletionStage.
- **Equivalente Rust propuesto**: Sin fachada separada: una sola API Rust cubre la semantica (propuesta Q-T-JAPI-1); sin clases Java. **Diferencias observables**: Q-T-JAPI-1: no hay clases Java ni lambdas japi; la semantica coincide con las filas scaladsl equivalentes.
- **Aserciones**:
  - A1. mismas aserciones que las filas equivalentes de scaladsl; cuerpos Java no leidos
- **Oraculo**: No aplica como golden propio (la semantica se compara en las filas scaladsl).
- **Decisiones**: Q-T-JAPI-1

## AT-JAPI-005: javadsl.ActorContext (getSelf, getLog, Optional getChild, CompletionStage ask/pipeToSelf) (public)

- **Contrato observable**: Contractual: espejo Java del contexto (Optional/List en lugar de Option/Seq, CompletionStage en lugar de Future).
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/ActorContext.scala:1-345; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorContextImpl.scala:133-158
- **Condiciones**: Sin leer.
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: filas: AT-CTX-*. Externas: JVM: japi.function, Optional, CompletionStage.
- **Equivalente Rust propuesto**: Sin fachada separada: una sola API Rust cubre la semantica (propuesta Q-T-JAPI-1); sin clases Java. **Diferencias observables**: Q-T-JAPI-1: no hay clases Java ni lambdas japi; la semantica coincide con las filas scaladsl equivalentes.
- **Aserciones**:
  - A1. mismas aserciones que las filas equivalentes de scaladsl; cuerpos Java no leidos
- **Oraculo**: No aplica como golden propio (la semantica se compara en las filas scaladsl).
- **Decisiones**: Q-T-JAPI-1

## AT-JAPI-006: Otras fachadas Java: TimerScheduler, StashBuffer, Routers, AskPattern, Adapter (public)

- **Contrato observable**: Contractual: espejos Java con Duration y CompletionStage; Adapter.* equivale a scaladsl.adapter.
- **Fuente**: actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/TimerScheduler.scala:1-275; actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/StashBuffer.scala:1-160; actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/Routers.scala:1-203; actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/AskPattern.scala:1-67; actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/Adapter.scala:1-183
- **Condiciones**: Sin leer.
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: filas: AT-TMR-*; AT-STASH-*; AT-RT-*; AT-ASK-*; AT-ADP-*. Externas: JVM: japi.function, Optional, CompletionStage.
- **Equivalente Rust propuesto**: Sin fachada separada: una sola API Rust cubre la semantica (propuesta Q-T-JAPI-1); sin clases Java. **Diferencias observables**: Q-T-JAPI-1: no hay clases Java ni lambdas japi; la semantica coincide con las filas scaladsl equivalentes.
- **Aserciones**:
  - A1. mismas aserciones que las filas equivalentes de scaladsl; cuerpos Java no leidos
- **Oraculo**: No aplica como golden propio (la semantica se compara en las filas scaladsl).
- **Decisiones**: Q-T-JAPI-1

## AT-TK-001.1: ActorTestKit: construccion, nombre del sistema y configuracion (public)

- **Contrato observable**: Contractual: ActorTestKit() crea un ActorSystem de test con guardian propio; el nombre sale de la clase de test (testNameFromCallStack) o del nombre dado normalizado (scrubActorSystemName: caracteres invalidos -> "_", paquete eliminado: "Vector"); por defecto carga application-test.conf (ConfigFactory.load("application-test")) y NO application.conf; con Config explicita no carga application-test; ActorTestKit(system) agrega un guardian "test", "test-2", ... bajo systemActor; settings opcionales.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKit.scala:48-180,194-218; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestKitUtils.scala:76-91
- **Condiciones**: Caracteres validos de nombre: letras, digitos, "-", "_" (test "validActorSystemNameChars" conserva todos). testkit con Config explicita: hasPath("test.from-application-test") == false.
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: filas: AT-SYS-*; AC-CFG. Externas: Runtime de actores, scheduler clasico (ExplicitlyTriggeredScheduler), Logback (LoggingTestKit), ScalaTest/JUnit.
- **Equivalente Rust propuesto**: ActorTestKit::new()/with_name/with_config; nombre via macro/type_name. **Diferencias observables**: Rust: nombre del sistema por type_name/module_path del test o explicito.
- **Aserciones**:
  - A1. ScalaTestWithActorTestKit usa el nombre de la clase de test ("ActorTestKitSpec") — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitSpec.scala:36-38
  - A2. ActorTestKit() usa el nombre de la clase — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitSpec.scala:50-55
  - A3. ActorTestKit(classOf[Vector[_]].getName) -> "Vector" — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitSpec.scala:57-63
  - A4. nombre con caracteres invalidos -> "actor__sys-tem_Name_1__with_invalid_ch_racter___" — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitSpec.scala:65-71
  - A5. nombre valido se conserva — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitSpec.scala:73-80
  - A6. ActorTestKit(system) expone el mismo sistema — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitSpec.scala:40-48
  - A7. application-test.conf se carga por defecto (test.from-application-test="yes", sin test.from-application) — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitSpec.scala:109-112
  - A8. Config explicita no carga application-test.conf — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitSpec.scala:114-124
  - A9. nombres de probes unicos respecto del TestKit clasico — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitSpec.scala:126-131
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh (requiere pekko-actor-testkit-typed_2.13 1.7.1, ausente en fetch-jars.sh). Procedencia: jar oficial pekko-actor-testkit-typed_2.13 1.7.1, que NO esta en tools/oracle/fetch-jars.sh (hay que agregarlo con su SHA-1).
- **Decisiones**: Q-T-TK-3

## AT-TK-001.2: ActorTestKit: spawn, stop y falla de spawn (public)

- **Contrato observable**: Contractual: spawn(behavior[, name][, props]) pide al guardian del testkit (ask con timeout=DefaultTimeout dilatado) crear el hijo y devuelve el ref; stop(ref, max) hace watchWith+stop y espera Ack, si vence lanza AssertionError "timeout (<max>) during stop() waiting for actor [<path>] to stop"; si el spawn falla (p.ej. dispatcher desconocido) el guardian loguea error "Spawn failed, props [<props>]" y responde con un ref a un actor detenido.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKit.scala:231-272; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestKitUtils.scala:31-71
- **Condiciones**: stop: guardian hace context.watchWith(ref, ActorStopped(reply)) y context.stop(ref) (actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestKitUtils.scala:53-56). Las excepciones NonFatal de spawn se capturan.
- **Orden / concurrencia / ciclo de vida**: spawn bloquea con Await.result hasta la respuesta del guardian.
- **Dependencias**: filas: AT-CTX-002; AT-ASK-001. Externas: Runtime de actores, scheduler clasico (ExplicitlyTriggeredScheduler), Logback (LoggingTestKit), ScalaTest/JUnit.
- **Equivalente Rust propuesto**: spawn/stop asincronos o bloqueantes (Q-T-TK-1) sobre un guardian de testkit. **Diferencias observables**: Rust: async fn o bloqueante sobre runtime.
- **Aserciones**:
  - A1. spawn de comportamiento anonimo corre el setup — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitSpec.scala:82-90
  - A2. spawn con nombre -> path.name == "name" — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitSpec.scala:92-100
  - A3. dispatcher desconocido -> error "Spawn failed" y ref terminado — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/DispatcherSelectorSpec.scala:87-93
  - A4. timeout de stop -> AssertionError con el texto indicado (no afirmada por spec; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKit.scala:264-272)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-T-TK-1

## AT-TK-001.3: ActorTestKit: shutdown (public)

- **Contrato observable**: Contractual: shutdownTestKit()/ActorTestKit.shutdown(system[, timeout, throwIfShutdownFails]) llama system.terminate() y espera whenTerminated hasta el timeout (DefaultActorSystemShutdownTimeout=10s dilatado); al vencer lanza RuntimeException "Failed to stop [<nombre>] within [<timeout>] \n<printTree>" si throw-on-shutdown-timeout=true, si no imprime el mensaje por stdout.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKit.scala:160-172,220-225; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestKitUtils.scala:93-102
- **Condiciones**: Con throw=false el mensaje de error va a println y no se lanza.
- **Orden / concurrencia / ciclo de vida**: Bloqueante hasta terminar o vencer.
- **Dependencias**: filas: AT-CS-001. Externas: Runtime de actores, scheduler clasico (ExplicitlyTriggeredScheduler), Logback (LoggingTestKit), ScalaTest/JUnit.
- **Equivalente Rust propuesto**: shutdown(timeout, throw) sobre el sistema. **Diferencias observables**: Idem Q-T-TK-1.
- **Aserciones**:
  - A1. tras shutdownTestKit el sistema esta terminado (Done) — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitSpec.scala:102-107
  - A2. timeout de shutdown (no afirmada por spec; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestKitUtils.scala:93-101)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-T-TK-1

## AT-TK-002.1: TestProbe: expectMessage, expectMessageType, expectNoMessage, receiveMessage(s) (public)

- **Contrato observable**: Contractual: la probe es un actor de sistema con cola LinkedBlockingDeque; expectMessage(obj) espera un mensaje (timeout remainingOrDefault = single-expect-default 3s dilatado o el resto de within) y compara igualdad: otro mensaje -> AssertionError "expected <obj>, found <m>[: hint]", timeout -> AssertionError "timeout (<max>) during expectMessage while waiting for <obj>[: hint]"; esperar una Signal lanza IllegalArgumentException "<clase> is a signal, expecting signals with a TestProbe is not possible"; expectMessageType[T] (boxed type): "Expected <c>, found <clase> (<m>)" / "Timeout (<max>) during expectMessageClass waiting for <c>"; receiveMessage: "Timeout (<max>) during receiveMessage while waiting for message."; receiveMessages(n): "timeout (<max>) while expecting <n> messages (got <k>)"; expectNoMessage([max]) falla "Received unexpected message <m>" si llega algo (default expect-no-message-default 100ms); max = 0 hace poll sin bloquear.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestProbeImpl.scala:50-101,160-252,254-276; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProbe.scala
- **Condiciones**: expectNoMessage marca lastWasNoMessage = true (para within); cualquier otro receive lo reinicia.
- **Orden / concurrencia / ciclo de vida**: Los mensajes se encolan en orden de llegada; expect* consumen la cabeza.
- **Dependencias**: filas: AT-TK-001.1; AT-TK-010. Externas: Runtime de actores, scheduler clasico (ExplicitlyTriggeredScheduler), Logback (LoggingTestKit), ScalaTest/JUnit.
- **Equivalente Rust propuesto**: TestProbe<M> con expect_message/expect_message_type/expect_no_message/receive_message(s) (async o bloqueante, Q-T-TK-1). **Diferencias observables**: Excepciones -> panic/AssertionError equivalente en Rust (mensajes a conservar).
- **Aserciones**:
  - A1. receiveMessages(3) devuelve List("one","two","three") — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProbeSpec.scala:135-145
  - A2. receiveMessages(3, 50ms) con un solo mensaje -> AssertionError — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProbeSpec.scala:147-155
  - A3. receiveMessage devuelve cada EventT y luego expectNoMessage — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProbeSpec.scala:157-165
  - A4. receiveMessage(100ms) sin mensaje -> AssertionError — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProbeSpec.scala:167-169
  - A5. con default-timeout 100ms receiveMessage() sin mensaje falla — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProbeSpec.scala:201-207
  - A6. textos de error de expectMessage/expectMessageType/expectNoMessage (no afirmadas por spec; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestProbeImpl.scala:173-251)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh (textos de AssertionError). Procedencia: jar oficial.
- **Decisiones**: Q-T-TK-1

## AT-TK-002.2: TestProbe: fishForMessage y FishingOutcome (public)

- **Contrato observable**: Contractual: fishForMessage(max[, hint])(fisher) recibe mensajes y aplica fisher: Continue (conserva), ContinueAndIgnore (descarta), Complete (devuelve los acumulados incluido el actual) o Fail(error) (AssertionError "<error>, hint: <hint>"); un MatchError del fisher -> AssertionError "Unexpected message <m> while fishing for messages, seen messages <lista>, hint: <hint>"; vencer max -> AssertionError "timeout (<max>) during fishForMessage, seen messages <lista>, hint: <hint>"; cada iteracion descuenta el tiempo ya gastado.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestProbeImpl.scala:278-332; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/FishingOutcome.scala:1-34
- **Condiciones**: fishForMessagePF delega en fishForMessage.
- **Orden / concurrencia / ciclo de vida**: Consume mensajes en orden.
- **Dependencias**: filas: AT-TK-002.1. Externas: Runtime de actores, scheduler clasico (ExplicitlyTriggeredScheduler), Logback (LoggingTestKit), ScalaTest/JUnit.
- **Equivalente Rust propuesto**: fish_for_message(max, |m| FishingOutcome). **Diferencias observables**: Rust: closure total (sin MatchError).
- **Aserciones**:
  - A1. continue/complete devuelve List("one","two") — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProbeSpec.scala:69-82
  - A2. FishingOutcomes.fail -> AssertionError — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProbeSpec.scala:84-97
  - A3. fisher lento: AssertionError por timeout — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProbeSpec.scala:99-108
  - A4. mensaje desconocido en fishForMessagePF -> AssertionError — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProbeSpec.scala:110-121
  - A5. timeout esperando mas mensajes — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProbeSpec.scala:123-133
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-T-TK-1

## AT-TK-002.3: TestProbe: expectTerminated y stop (public)

- **Contrato observable**: Contractual: expectTerminated(ref[, max]) pide a la probe observar (watch) el actor y espera su Terminated: timeout -> AssertionError "timeout (<max>) during expectTerminated waiting for actor [<path>] to stop"; otro actor -> AssertionError "expected [<path>] to stop, but saw [<path2>] stop"; max infinito usa takeFirst; max cero hace poll; sirve aunque el actor ya haya terminado (watch de actor muerto entrega Terminated); probe.stop() envia Stop al actor de la probe, que se detiene (y es observable desde otra probe).
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestProbeImpl.scala:51-74,334-355,412-414
- **Condiciones**: La probe usa Behaviors.receiveSignal para encolar Terminated en una cola aparte (terminations).
- **Orden / concurrencia / ciclo de vida**: Terminated llega por signal al actor de la probe.
- **Dependencias**: filas: AC-DW; AT-CTX-006. Externas: Runtime de actores, scheduler clasico (ExplicitlyTriggeredScheduler), Logback (LoggingTestKit), ScalaTest/JUnit.
- **Equivalente Rust propuesto**: expect_terminated(&ActorRef, max); probe.stop(). **Diferencias observables**: Ninguna con DeathWatch correcto.
- **Aserciones**:
  - A1. expectTerminated sobre actor ya detenido — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProbeSpec.scala:49-53
  - A2. expectTerminated cuando aun no se detuvo (500ms) — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProbeSpec.scala:55-67
  - A3. probe1.stop() y probe2.expectTerminated(probe1.ref) — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProbeSpec.scala:171-176
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-T-TK-1

## AT-TK-002.4: TestProbe: awaitAssert, within, remaining y dilatacion de tiempo (public)

- **Contrato observable**: Contractual: awaitAssert(a[, max, interval]) reintenta cada interval (100ms por defecto) hasta que a no lance (NonFatal) o venza max (relanza la ultima excepcion); within(min,max)(f) fija el fin del bloque (end) y verifica que el bloque dure >= min y <= max (salvo que la ultima operacion fuese expectNoMessage), con "required min time <min> not possible, only <resto> left", "block took <d>, should at least have been <min>" y "block took <d>, exceeding <max>"; remaining/remainingOr/remainingOrDefault; todos los plazos con .dilated multiplican por timefactor.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestProbeImpl.scala:103-158,357-403; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProbe.scala:1-273
- **Condiciones**: remaining fuera de within lanza AssertionError "`remaining` may not be called outside of `within`".
- **Orden / concurrencia / ciclo de vida**: Usa Thread.sleep(t) entre intentos.
- **Dependencias**: filas: AT-TK-010. Externas: Runtime de actores, scheduler clasico (ExplicitlyTriggeredScheduler), Logback (LoggingTestKit), ScalaTest/JUnit.
- **Equivalente Rust propuesto**: await_assert(f, max, interval) y within(min,max,f). **Diferencias observables**: Rust: sleep async y reloj inyectable (para ManualTime).
- **Aserciones**:
  - A1. awaitAssert en uso real — actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/pubsub/LocalPubSubSpec.scala:44-52
  - A2. within y textos (no afirmada por spec; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestProbeImpl.scala:136-158)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial.
- **Decisiones**: Q-T-TK-1

## AT-TK-003: BehaviorTestKit (test sincronico: run, signal, effects, selfInbox, childInbox, ask reificado, timers) (public)

- **Contrato observable**: Contractual: ejecuta un Behavior sin ActorSystem real (StubbedActorContext/EffectfulActorContext sobre ActorSystemStub), registra Effects en una cola, expone run(msg), signal(sig), retrieveEffect/retrieveAllEffects/expectEffect/expectEffectType/expectEffectPF/hasEffects, selfInbox/childInbox/childTestKit, logEntries/clearLog, returnedBehavior, currentBehavior, isAlive, address por defecto; los timers se simulan (ver AT-TK-004.2).
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKit.scala:1-187; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/BehaviorTestKitImpl.scala:1-247; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/StubbedActorContext.scala:1-280; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/EffectfulActorContext.scala:37-212
- **Condiciones**: Sin leer salvo lo citado.
- **Orden / concurrencia / ciclo de vida**: Sincronico: run() ejecuta el Behavior en el hilo llamante.
- **Dependencias**: filas: AT-CTX-*; AT-TMR-*. Externas: Runtime de actores, scheduler clasico (ExplicitlyTriggeredScheduler), Logback (LoggingTestKit), ScalaTest/JUnit.
- **Equivalente Rust propuesto**: BehaviorTestKit<M> sincronico; requiere interprete de Behavior sin runtime (Q-T-TK-4). **Diferencias observables**: Q-T-TK-4.
- **Aserciones**:
  - A1. ask reificado: ver AT-TK-004.1
  - A2. resto: titulos en actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:207-530 (no leidos)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh (jar testkit-typed). Procedencia: jar oficial pekko-actor-testkit-typed_2.13 1.7.1 (a agregar a fetch-jars.sh).
- **Decisiones**: Q-T-TK-4

## AT-TK-004.1: Effects: AskInitiated (ask reificado) (public)

- **Contrato observable**: Contractual: context.ask en BehaviorTestKit no envia un ask real: registra Effect.AskInitiated(target, responseTimeout, responseClass)(askMessage, forwardResponse, mapResponse), envia askMessage al target (replyTo = ignoreRef), y permite completar con respondWith(resp) o timeout() (corre el mapResponse en el Behavior via AdaptMessage y deja logs/estado); solo se puede completar una vez: segunda llamada (respondWith o timeout) -> IllegalStateException "Can only complete the ask once"; sin forwardResponse -> IllegalStateException "Can only complete and ask from a BehaviorTestKit-emitted effect"; adaptResponse(resp)/adaptTimeout[(msg)] devuelven el mensaje adaptado sin correr el behavior; texto del timeout: "Ask timed out on [<target>] after [<ms> ms]. Message of type [<clase>]. A typical reason for `AskTimeoutException` is that the recipient actor didn't send a reply."; timeout <= 0 -> IllegalArgumentException "requirement failed: Timeout length must be positive, question not sent to [<target>]" (require de Scala, actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/EffectfulActorContext.scala:51).
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/Effect.scala:53-92; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/EffectfulActorContext.scala:46-101
- **Condiciones**: sendResponse esta sincronizado (synchronized). El efecto no se vuelve a registrar al completar (hasEffects == false tras completar, test 640).
- **Orden / concurrencia / ciclo de vida**: Completar corre el Behavior sincronamente.
- **Dependencias**: filas: AT-CTX-*; AT-ASK-001. Externas: Runtime de actores, scheduler clasico (ExplicitlyTriggeredScheduler), Logback (LoggingTestKit), ScalaTest/JUnit.
- **Equivalente Rust propuesto**: Effect::AskInitiated<Req,Res,M> con respond_with/timeout/adapt_*. **Diferencias observables**: Ninguna.
- **Aserciones**:
  - A1. expectEffectType[AskInitiated] == Effects.askInitiated(cdInbox.ref, 10.seconds, classOf[CookiesForYou]); cdInbox recibe effect.askMessage; adaptResponse da Log "Got 10 cookies..."; adaptTimeout da Log que empieza "Failed to get cookies: Ask timed out on [" — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:532-554
  - A2. respondWith completa y deja un log "Got <n> cookies from distributor" sin mensajes en selfInbox — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:556-582
  - A3. timeout() deja un log "Failed to get cookies: Ask timed out on [" — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:584-609
  - A4. completar dos veces (respondWith o timeout) -> IllegalStateException; solo el primer completado deja log — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:611-642
  - A5. tras timeout, completar o repetir timeout -> IllegalStateException y no hay logs ni efectos nuevos — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:644-676
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh (jar testkit-typed). Procedencia: jar oficial pekko-actor-testkit-typed_2.13 1.7.1 (a agregar a fetch-jars.sh).
- **Decisiones**: Q-T-TK-4

## AT-TK-004.2: Effects: Spawned, SpawnedAnonymous, Stopped, Watched, WatchedWith, Unwatched, ReceiveTimeoutSet/Cancelled, Scheduled, TimerScheduled/Cancelled, MessageAdapter, NoEffects (public)

- **Contrato observable**: Contractual: EffectfulActorContext registra un Effect por operacion del contexto: spawn -> Spawned(behavior, name, props), spawnAnonymous -> SpawnedAnonymous, spawnMessageAdapter -> SpawnedAdapter/SpawnedAnonymousAdapter (internos), messageAdapter -> MessageAdapter(clase, f), stop(child) -> Stopped(child.path.name), watch/watchWith/unwatch -> Watched/WatchedWith/Unwatched, setReceiveTimeout -> ReceiveTimeoutSet(d,msg), cancelReceiveTimeout -> ReceiveTimeoutCancelled, scheduleOnce -> Scheduled(delay,target,msg), timers -> TimerScheduled(key,msg,delay,mode,overriding)(send)/TimerCancelled(key) con modos FixedRate/FixedDelay (con o sin initialDelay) y Single; cancelAll cancela activos; startPeriodicTimer = FixedRateMode; los timers de Single se quitan al dispararse.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/Effect.scala:94-313; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/EffectfulActorContext.scala:103-212; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/Effects.scala
- **Condiciones**: Spawned igualdad por (behavior,childName,props) (actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/Effect.scala:97-116).
- **Orden / concurrencia / ciclo de vida**: Los efectos se encolan en el orden de las llamadas.
- **Dependencias**: filas: AT-CTX-*; AT-TMR-*. Externas: Runtime de actores, scheduler clasico (ExplicitlyTriggeredScheduler), Logback (LoggingTestKit), ScalaTest/JUnit.
- **Equivalente Rust propuesto**: enum Effect y EffectfulContext. **Diferencias observables**: Q-T-TK-4.
- **Aserciones**:
  - A1. catalogo derivado del codigo actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/EffectfulActorContext.scala:103-211 (no afirmada por spec leido)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh (jar testkit-typed). Procedencia: jar oficial pekko-actor-testkit-typed_2.13 1.7.1 (a agregar a fetch-jars.sh).
- **Decisiones**: Q-T-TK-4

## AT-TK-005: TestInbox / ReplyInbox / StatusReplyInbox (public)

- **Contrato observable**: Contractual: buzones sincronicos para BehaviorTestKit: TestInbox con ref, receiveMessage, receiveAll, hasMessages, expectMessage; ReplyInbox/StatusReplyInbox para respuestas de ask.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestInbox.scala:1-170; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestInboxImpl.scala:1-184
- **Condiciones**: Sin leer.
- **Orden / concurrencia / ciclo de vida**: Sin leer.
- **Dependencias**: filas: AT-TK-003. Externas: Runtime de actores, scheduler clasico (ExplicitlyTriggeredScheduler), Logback (LoggingTestKit), ScalaTest/JUnit.
- **Equivalente Rust propuesto**: TestInbox<M>. **Diferencias observables**: Q-T-TK-4.
- **Aserciones**:
  - A1. sin cuerpos leidos
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh. Procedencia: jar oficial pekko-actor-testkit-typed_2.13 1.7.1 (a agregar a fetch-jars.sh).
- **Decisiones**: Q-T-TK-4

## AT-TK-006: LoggingTestKit y LogCapturing (filtros por nivel, mensaje, causa, MDC, ocurrencias) (public)

- **Contrato observable**: Contractual: espera/intercepta eventos de log con filtros (nivel, mensaje exacto/patron, causa, nombre de logger, MDC, custom, ocurrencias, filter-leeway) dentro de un bloque expect; withOccurrences(0) comprueba ausencia; LogCapturing muestra los logs capturados solo si el test falla.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/LoggingTestKit.scala:1-225; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/LoggingTestKitImpl.scala:1-176; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestAppender.scala; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/CapturingAppender.scala; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/LogCapturing.scala
- **Condiciones**: Sin leer.
- **Orden / concurrencia / ciclo de vida**: Sin leer.
- **Dependencias**: filas: AT-LOG-001; AT-LOG-002. Externas: Runtime de actores, scheduler clasico (ExplicitlyTriggeredScheduler), Logback (LoggingTestKit), ScalaTest/JUnit.
- **Equivalente Rust propuesto**: Layer de tracing que captura eventos con filtros (Q-T-LOG-1). **Diferencias observables**: Q-T-TK-2.
- **Aserciones**:
  - A1. titulos de spec en actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/LoggingEventFilterSpec.scala:69-136 y actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestAppenderSpec.scala:38-117; cuerpos sin leer
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh solo si hay logback en el jar de pruebas. Procedencia: jar oficial pekko-actor-testkit-typed_2.13 1.7.1 (a agregar a fetch-jars.sh).
- **Decisiones**: Q-T-TK-2

## AT-TK-007: ManualTime (scheduler disparado explicitamente) (public)

- **Contrato observable**: Contractual: ManualTime.config fija pekko.scheduler.implementation = ExplicitlyTriggeredScheduler; ManualTime() exige que el sistema use ese scheduler (IllegalArgumentException "ActorSystem not configured with explicitly triggered scheduler, make sure to include org.apache.pekko.actor.testkit.typed.scaladsl.ManualTime.config() when setting up the test" o "ActorSystem.scheduler is not a classic SchedulerAdapter but a <clase>, this is not supported"); timePasses(amount) avanza el tiempo; expectNoMessageFor(duration, probes*) avanza y exige expectNoMessage(0) en cada probe.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ManualTime.scala:31-81
- **Condiciones**: Errores textuales por codigo.
- **Orden / concurrencia / ciclo de vida**: timePasses ejecuta tareas vencidas en el hilo llamante (ExplicitlyTriggeredScheduler, classic).
- **Dependencias**: filas: AC-SCH; AT-TMR-*. Externas: Runtime de actores, scheduler clasico (ExplicitlyTriggeredScheduler), Logback (LoggingTestKit), ScalaTest/JUnit.
- **Equivalente Rust propuesto**: Reloj inyectable (clock trait) en el scheduler; ManualTime::time_passes. **Diferencias observables**: Requiere scheduler con reloj inyectable (AC-SCH).
- **Aserciones**:
  - A1. derivada del codigo actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ManualTime.scala:44-79 (no afirmada por spec leido)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh (jshell con ExplicitlyTriggeredScheduler). Procedencia: jar oficial.
- **Decisiones**: Q-X-4

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

## AT-TK-009: Integraciones de framework (ScalaTestWithActorTestKit, JUnit TestKitJunitResource, JUnit5), SerializationTestKit (public)

- **Contrato observable**: Contractual: ScalaTestWithActorTestKit/ActorTestKitBase crean y cierran un ActorTestKit por suite; TestKitJunitResource (JUnit4), TestKitJUnit5Extension y JUnit5TestKitBuilder para Java; LogCapturingExtension; SerializationTestKit con round trip de serializacion de mensajes.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ScalaTestWithActorTestKit.scala:1-105; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKitBase.scala:1-138; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/javadsl/TestKitJunitResource.scala:1-195; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/javadsl/TestKitJUnit5Extension.scala; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/javadsl/JUnit5TestKitBuilder.scala; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/SerializationTestKit.scala:1-70; actor-testkit-typed/src/main/java/org/apache/pekko/actor/testkit/typed/annotations/JUnit5TestKit.java
- **Condiciones**: Sin leer.
- **Orden / concurrencia / ciclo de vida**: Sin leer.
- **Dependencias**: filas: AT-TK-001.1; AC-SER. Externas: Runtime de actores, scheduler clasico (ExplicitlyTriggeredScheduler), Logback (LoggingTestKit), ScalaTest/JUnit.
- **Equivalente Rust propuesto**: Helper RAII/macro para #[tokio::test]; sin JUnit. **Diferencias observables**: Q-T-TK-3.
- **Aserciones**:
  - A1. sin cuerpos leidos
- **Oraculo**: No aplica (integraciones JVM).
- **Decisiones**: Q-T-TK-3;Q-T-SER-1

## AT-TK-010: TestKitSettings y pekko.actor.testkit.typed.* (timefactor, timeouts) (public)

- **Contrato observable**: Contractual: TestKitSettings(system|config) lee pekko.actor.testkit.typed: timefactor=1.0 (requiere finito > 0: "timefactor must be positive finite double"), single-expect-default=3s, expect-no-message-default=100ms, default-timeout=5s (implicit Timeout), system-shutdown-default=10s, throw-on-shutdown-timeout=true, filter-leeway=3s; todos los plazos (menos los booleanos) se dilatan: dilated(d) = d*timefactor+0.5 redondeado en nanos.
- **Fuente**: actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/TestKitSettings.scala:27-101; actor-testkit-typed/src/main/resources/reference.conf:9-39
- **Condiciones**: Config via extension privada Ext (actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/TestKitSettings.scala:55-62).
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: filas: AC-CFG. Externas: Runtime de actores, scheduler clasico (ExplicitlyTriggeredScheduler), Logback (LoggingTestKit), ScalaTest/JUnit.
- **Equivalente Rust propuesto**: struct TestKitSettings con dilated(). **Diferencias observables**: Parser HOCON propio con duraciones (Q-X-1).
- **Aserciones**:
  - A1. default-timeout=100ms hace fallar receiveMessage() a los 100ms — actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProbeSpec.scala:194-207
  - A2. valores por defecto de reference.conf (no afirmada por spec; actor-testkit-typed/src/main/resources/reference.conf:9-39)
  - A3. dilated(d) = (d.toNanos*timefactor+0.5).toLong (no afirmada por spec; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/TestKitSettings.scala:93-95)
- **Oraculo**: Comparable con jar oficial pekko-actor-typed_2.13 1.7.1 (tools/oracle/fetch-jars.sh) via jshell/tools/oracle/run.sh (valores por defecto efectivos). Procedencia: jar oficial pekko-actor-testkit-typed_2.13 1.7.1 (a agregar a fetch-jars.sh).
- **Decisiones**: Q-X-1

## Descomposiciones

Los padres desaparecen del TSV; los hijos heredan `impl`/`validation` = pending (ninguna fila de este grupo estaba implementada).

- AT-RCP-005 -> AT-RCP-005.1 (Find), .2 (Subscribe y Listing inicial/incremental), .3 (accesores y chequeo de clave del Listing).
- AT-RT-002 -> AT-RT-002.1 (round robin), .2 (random), .3 (consistent hashing): tres logicas con contratos y errores distintos.
- AT-ADP-006 -> AT-ADP-006.1 (despacho de mensajes), .2 (resultado del Behavior), .3 (hooks de ciclo de vida), .4 (supervision de hijos y ChildFailed).
- AT-PS-001 -> AT-PS-001.1 (Subscribe/Unsubscribe/terminacion), .2 (Publish), .3 (GetTopicStats).
- AT-EXT-001 -> AT-EXT-001.1 (identidad y concurrencia), .2 (fallo cacheado), .3 (extension() sin registro / hasExtension).
- AT-EXT-002 -> AT-EXT-002.1 (library-extensions), .2 (extensions), .3 (LoadTypedExtensions desde clasico).
- AT-TK-001 -> AT-TK-001.1 (construccion, nombre, config), .2 (spawn/stop), .3 (shutdown).
- AT-TK-002 -> AT-TK-002.1 (expect/receive), .2 (fishForMessage), .3 (expectTerminated/stop), .4 (awaitAssert/within/dilatacion).
- AT-TK-004 -> AT-TK-004.1 (AskInitiated), .2 (resto del catalogo de Effects).

## Omisiones agregadas

- AT-RCP-008: extractores `ServiceKey.Listing` / `ServiceKey.Registered` (`actor-typed/src/main/scala/org/apache/pekko/actor/typed/receptionist/Receptionist.scala:77-95`), usados por Topic y routers.
- AT-RT-006: validacion `pool size must be positive, was <n>` y construccion inmutable de los builders (`actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/PoolRouterImpl.scala:38`).
- AT-ADP-008: singleton `AdapterExtension`/`ActorSystemAdapter` y carga de extensiones tipadas desde el clasico (`internal/adapter/AdapterExtension.scala`, `actor-typed/src/main/resources/reference.conf:35-36`; archivo sin cita en el inventario).
- AT-ADP-009: mapeo de mensajes de sistema y rechazo de `null` en `ActorRefAdapter` (`actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorRefAdapter.scala:35-38,70-81`).
- AT-ADP-010: `GuardianStartupBehavior` y `GuardianStopInterceptor` (`actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/GuardianStartupBehavior.scala:31-90`), que alimentan AT-CS-001.
- AT-TK-010: `TestKitSettings` y claves `pekko.actor.testkit.typed.*` (`actor-testkit-typed/src/main/resources/reference.conf:9-39`, `actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/TestKitSettings.scala:27-101`).
- Sin fila nueva por no definir comportamiento observable: ver Cobertura (Platform.scala, DeliverySerializable.scala, TestException.scala).

## Contradicciones halladas y corregidas

- AT-EXT-002 (inventario viejo): "extensions loguea y sigue". Codigo (`actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:56-65`) y test (`actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:136-153`): solo los fallos de CARGA (clase ausente, no es ExtensionId) se omiten con log en `extensions`; un fallo de CREACION hace fallar el arranque (RuntimeException, repetible). Corregido en AT-EXT-002.2 y AT-EXT-001.2.
- AT-ASK-001 (inventario viejo): "timeout <= 0 es IllegalArgumentException". El codigo (`actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/AskPattern.scala:152-157`) devuelve `Future.failed(IllegalArgumentException)`, no la lanza, y ademas igual se envia el mensaje a dead letters (`actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/AskPattern.scala:170-181`). Sin test que lo afirme. Corregido en AT-ASK-001.
- AT-DSP-003 (inventario viejo): default = SingleConsumerOnlyUnboundedMailbox. PropsAdapter fija `mailbox == "pekko.actor.typed.default-mailbox"` (`actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/adpater/PropsAdapterSpec.scala:28-32`) y `MailboxSelector.default()` devuelve `fromConfig` (`actor-typed/src/main/scala/org/apache/pekko/actor/typed/Props.scala:213`); el tipo de cola sale de `actor-typed/src/main/resources/reference.conf:29-32`. Corregido en AT-DSP-003.
- AT-SPV-001: el Scaladoc dice que el formato del sufijo es "undefined" (`actor-typed/src/main/scala/org/apache/pekko/actor/typed/SpawnProtocol.scala:49-52`) pero el spec lo fija en "-1","-2",... (`actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SpawnProtocolSpec.scala:89-111`). Se sigue el codigo y el spec; ver contratos.
- AT-RT-002 (inventario viejo): "sin routees lanza". Solo ConsistentHashingLogic lo afirma (IllegalStateException con texto fijo, `actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/routing/RoutingLogicSpec.scala:170-178`); round robin y random fallan de otra forma y los routers nunca llaman sin routees. Corregido en AT-RT-002.1-3 (Q-T-RT-2).
- AT-EVT-004 (inventario viejo): "ciertos descartes se publican como Dropped" sin distinguir camino. Supervision usa el eventStream clasico, el router de grupo `EventStream.Publish` tipado, y Topic envia `Dropped` a `deadLetters`, que `DeadLetterActorRef` envuelve en `DeadLetter` (`actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:736-743`): probable diferencia observable; pendiente de oraculo (Q-T-EVT-1).
- AT-RCP-005 (inventario viejo) omite que no existe `Unsubscribe` en el protocolo y que Deregister/Register repetidos notifican a suscriptores aunque el estado no cambie (`actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionist.scala:195-218`). Registrado en AT-RCP-004/005.2 y Q-T-RCP-3.

## Filas que siguen listed

- AT-ADP-006.1: Q-X-8; implementacion leida, pero los tests que la afirman (WatchSpec, TimerSpec, ActorContextSpec, SupervisionSpec) son de otros grupos y no se releyeron
- AT-ADP-006.3: Q-X-8; implementacion leida, pero los tests que la afirman (WatchSpec, TimerSpec, ActorContextSpec, SupervisionSpec) son de otros grupos y no se releyeron
- AT-ADP-010: Cuerpo de GuardianStartupSpec no leido; el sistema SYS pertenece a otro grupo, esta fila solo documenta el hook que alimenta AT-CS-001
- AT-LOG-002: Falta leer el cuerpo de actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.scala:338-510 (MDC con withMdc/behavior changes) y WithMdcBehaviorInterceptor; implementacion de ActorMdc/LoggingContext si leida
- AT-LOG-004: Falta leer actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/package.scala (LoggerOps) y el cuerpo de LoggerOpsSpec; Q-T-LOG-4 propone subsumirlo (no excluirlo)
- AT-DSP-004: Falta leer el cuerpo de actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.scala:270-290 y el orden de los tags (Set.mkString: orden no definido para Set)
- AT-DSP-005: No se leyo ningun test que afirme lookup directo; derivada de actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorSystemAdapter.scala:95-103
- AT-DLV-001: No se leyeron ProducerControllerImpl ni los cuerpos de los specs; contrato solo por Scaladoc. Candidato a descomponer en: demanda/RequestNext, secuenciacion y confirmacion, reenvio/timers, MessageWithConfirmation, RegisterConsumer cuando falta. Q-T-DLV-1
- AT-DLV-002: No se leyeron ConsumerControllerImpl ni los 19 tests; Q-T-DLV-1
- AT-DLV-003: Implementacion y specs sin leer; solo Settings y mensajes leidos
- AT-DLV-005: Implementacion del chunking y specs sin leer; solo la lectura de configuracion verificada. Q-T-SER-1
- AT-DLV-007: Archivos no leidos; JVM-only. Q-T-DLV-3 propone equivalente (eventos tracing), no exclusion
- AT-JAPI-001: Fuente leida solo por tamano/nombre; no se leyeron los cuerpos Java. Q-T-JAPI-1 propone no portar la fachada Java y reutilizar las aserciones de las filas Scala
- AT-JAPI-002: Fuente leida solo por tamano/nombre; no se leyeron los cuerpos Java. Q-T-JAPI-1 propone no portar la fachada Java y reutilizar las aserciones de las filas Scala
- AT-JAPI-003: Fuente leida solo por tamano/nombre; no se leyeron los cuerpos Java. Q-T-JAPI-1 propone no portar la fachada Java y reutilizar las aserciones de las filas Scala
- AT-JAPI-004: Fuente leida solo por tamano/nombre; no se leyeron los cuerpos Java. Q-T-JAPI-1 propone no portar la fachada Java y reutilizar las aserciones de las filas Scala
- AT-JAPI-005: Fuente leida solo por tamano/nombre; no se leyeron los cuerpos Java. Q-T-JAPI-1 propone no portar la fachada Java y reutilizar las aserciones de las filas Scala
- AT-JAPI-006: Fuente leida solo por tamano/nombre; no se leyeron los cuerpos Java. Q-T-JAPI-1 propone no portar la fachada Java y reutilizar las aserciones de las filas Scala
- AT-TK-002.4: Sin spec leido que afirme within/dilatacion (solo uso indirecto); derivada del codigo
- AT-TK-003: Solo se leyeron las secciones de ask (actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala:531-677) y EffectfulActorContext; BehaviorTestKitImpl/StubbedActorContext y los demas tests (207-530) sin leer. Q-T-TK-4
- AT-TK-004.2: Implementacion de EffectfulActorContext leida; los cuerpos de los tests de effects/timers (209-530) no. Q-T-TK-4
- AT-TK-005: No leidos TestInboxImpl ni tests propios; Q-T-TK-4
- AT-TK-006: Implementacion (Logback-especifica: LogbackUtil, TestAppender) y specs sin leer a fondo; Q-T-LOG-1/Q-T-TK-2
- AT-TK-007: Test que use ManualTime sin leer (TimerSpec/ManualTimerExample); derivada del codigo
- AT-TK-009: Sin leer; Q-T-TK-3 propone sustituir integraciones JVM por helper de Rust y Q-T-SER-1 para SerializationTestKit

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
- `delivery/internal/{ProducerControllerImpl,ConsumerControllerImpl,WorkPullingProducerControllerImpl,ChunkedMessage}.scala` -> AT-DLV-001..003, 005 (no leidos: las filas siguen listed).
- `delivery/internal/DeliverySerializable.scala` -> sin comportamiento observable: trait marcador para los serializadores de persistencia/cluster (OUT).
- `internal/ActorFlightRecorder.scala; scala-jdk-9/internal/jfr/{Events,JFRActorFlightRecorder}.scala` -> AT-DLV-007 (JVM-only, listed).
- `internal/MiscMessageSerializer.scala; internal/receptionist/ServiceKeySerializer.scala; ActorRef.scala (SerializedActorRef)` -> AT-SER-001..003.
- `resources/reference.conf (typed)` -> extensions/library-extensions: AT-EXT-002; default-mailbox: AT-DSP-003; restart-stash-capacity: unidad SUP (otro grupo); serializers: AT-SER-001/002; use-slf4j: AT-LOG-003; reliable-delivery: AT-DLV-006.
- `javadsl/{Behaviors,AbstractBehavior,AbstractMatchingBehavior,ActorContext,Adapter,AskPattern,BehaviorBuilder,Receive,ReceiveBuilder,Routers,StashBuffer,TimerScheduler}.scala` -> AT-JAPI-001..006 (listed).
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
- `actor-typed-tests: delivery/*Spec.scala y Test*.scala (2673 lineas de specs + helpers), receptionist/ReceptionistApiSpec.scala, scaladsl/{LoggerOpsSpec,adapter/GuardianStartupSpec}, OrElseSpec` -> no leidos a fondo: filas AT-DLV-001..003/005, AT-LOG-004 y ADP-010 siguen listed; OrElseSpec prueba una composicion escrita en el propio test (sin fila, ver actor-typed.md).
- `actor-typed-tests: docs/** y jdocs/** (ejemplos de documentacion, ~40 archivos Scala/Java, incl. RouterSpec, SpawnProtocolDocSpec, MailboxDocSpec, DispatchersDocSpec, ExtensionDocSpec, PubSubExample, LoggingDocExamples, coexistence)` -> ejemplos que ilustran el uso de las APIs de las filas; no leidos: no definen comportamiento adicional salvo compilacion (a confirmar al implementar cada unidad).
- `actor-typed-tests: src/test/java/**, resources/mailbox-config-sample.conf, scala-2.13+/ActorSystemScala213PlusSpec.scala, scala-3/**` -> espejos Java y variantes de version de Scala; sin leer; cubiertos por las filas Scala/JAPI.
- actor-testkit-typed/src/test (Scala y Java), resources/application*.conf -> ActorTestKitSpec, TestProbeSpec y BehaviorTestKitSpec(ask) leidos; el resto (LoggingEventFilterSpec, TestAppenderSpec, JUnit5TestKitBuilderSpec, ejemplos jdocs/docs, Java) sin leer.


## Omisiones agregadas en la auditoría final de cobertura

Estas tres filas no tienen todavía el bloque de contrato completo: se agregaron al detectar cinco archivos de `actor-testkit-typed/src/main` sin ninguna cita. Siguen `listed`; no pueden ejecutarse hasta extraer su contrato.

### AT-TK-011 — ActorSystemStub: ActorSystem mínimo sin runtime real que usa BehaviorTestKit (scheduler, dispatchers, logging y settings de pega)

- **Contrato observable:** [contractual interno] ActorSystemStub es el ActorSystem[T] sobre el que corre BehaviorTestKit; sus operaciones no soportadas fallan. El contrato exacto (qué lanza UnsupportedOperationException, qué devuelve) está por extraer.
- **Fuente:** actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/ActorSystemStub.scala:1-151
- **Condiciones:** por extraer (n/a hasta leer el cuerpo completo).
- **Orden / concurrencia / ciclo de vida:** por extraer.
- **Dependencias:** AT-TK-002.1
- **Equivalente Rust propuesto:** stub de sistema que cumple el trait de sistema tipado con operaciones no soportadas explícitas
- **Aserciones:** por extraer; la fila no se marca `extracted` hasta tenerlas.
- **Oráculo:** el jar `pekko-actor-testkit-typed` no está en `tools/oracle/fetch-jars.sh`.
- **Decisiones:** Q-X-7, Q-X-8.

### AT-TK-012 — ControlledExecutor y DebugRef: ejecutor manual (cola de tareas que se corre a pedido) y ref de depuración usados por BehaviorTestKit

- **Contrato observable:** [contractual interno] ControlledExecutor acumula Runnable en una lista y los corre en el orden de llegada cuando se lo pide (runOne/runAll); DebugRef es el ActorRef que registra sus mensajes. Detalle por extraer.
- **Fuente:** actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/ControlledExecutor.scala:1-42 ; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/DebugRef.scala:1-80
- **Condiciones:** por extraer (n/a hasta leer el cuerpo completo).
- **Orden / concurrencia / ciclo de vida:** por extraer.
- **Dependencias:** AT-TK-002.1
- **Equivalente Rust propuesto:** ejecutor FIFO manual + ref que registra mensajes recibidos
- **Aserciones:** por extraer; la fila no se marca `extracted` hasta tenerlas.
- **Oráculo:** el jar `pekko-actor-testkit-typed` no está en `tools/oracle/fetch-jars.sh`.
- **Decisiones:** Q-X-7, Q-X-8.

### AT-TK-013 — LogbackUtil y LogCapturingExtension: utilidades de captura de logs sobre Logback para los tests (JUnit 5)

- **Contrato observable:** [contractual JVM] LogbackUtil manipula el logger raíz de Logback y LogCapturingExtension (JUnit 5) activa la captura por test. Su equivalente depende de Q-X-7 (backend de logging). Detalle por extraer.
- **Fuente:** actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/LogbackUtil.scala:1-59 ; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/javadsl/LogCapturingExtension.scala:1-53
- **Condiciones:** por extraer (n/a hasta leer el cuerpo completo).
- **Orden / concurrencia / ciclo de vida:** por extraer.
- **Dependencias:** AT-TK-008
- **Equivalente Rust propuesto:** captura de eventos de log por test sobre el backend elegido en Q-X-7
- **Aserciones:** por extraer; la fila no se marca `extracted` hasta tenerlas.
- **Oráculo:** el jar `pekko-actor-testkit-typed` no está en `tools/oracle/fetch-jars.sh`.
- **Decisiones:** Q-X-7, Q-X-8.

