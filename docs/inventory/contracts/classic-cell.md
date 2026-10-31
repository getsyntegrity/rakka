# Contratos verificables: classic-cell (REF, CELL, SUP, DW)

Fuente: Apache Pekko 1.7.1 (`upstream/pekko` @ 9a54d1294c3ac7821bbed3c940769c3d5027d1ff). Citas `path:line` relativas a `upstream/pekko/`. Las aserciones marcadas *no afirmada por spec* se derivan del código y deben cubrirse con tests propios o con el oráculo.

### AC-REF-001.1: Identidad de ActorRef: equals y hashCode

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: equals es verdadero sólo si path y uid coinciden; hashCode = uid, o hash del path si uid == 0 (undefinedUid). Una referencia a un actor terminado no es igual a la de otro actor recreado con el mismo path.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:109-112,175-186 ; actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:388-398
- **Condiciones** (normales, límites, errores, efectos): Normal: refs de la misma encarnación son iguales. Límite: uid 0 (refs sin encarnación, p. ej. deadLetters) usa el hash del path. Errores: ninguno. Efectos: ninguno.
- **Orden / concurrencia / ciclo de vida**: sin garantías de orden; operación pura.
- **Dependencias**: AC-PATH-014
- **Equivalente Rust propuesto**: `ActorRef` con `PartialEq/Eq/Hash` por (path, uid); `ActorIncarnationUid` no nulo para actores vivos
- **Diferencias observables**: Pekko compara con `==` sobre `Any` (false para no-ActorRef); en Rust sólo existe `PartialEq<ActorRef>`.
- **Aserciones**:
  - A1: Dos incarnations con igual path no son iguales y su toString difiere — ActorSelectionSpec.scala:139-143 (a2.path === a1.path; a2 should not be a1; a2.toString should not be a1.toString).
  - A2: Una ref a la incarnation terminada no resuelve como el actor: identify(a1.path) === None tras terminar — ActorSelectionSpec.scala:132.
  - A3: hashCode = path.uid si uid != 0, si no path.hashCode — derivada del código ActorRef.scala:175-178 (no afirmada por spec).
  - A4: newUid nunca devuelve 0 (reintenta) — derivada de ActorCell.scala:392-398 (no afirmada por spec).
- **Oráculo**: Pekko compilado de upstream/pekko con jshell: comparar equals/hashCode para dos incarnations (tools/oracle, escenario nuevo). Procedencia: jars oficiales 1.7.1 vía fetch-jars.sh.
- **Decisiones**: Q-REF-6
- **Notas**: Hijo de AC-REF-001. Ver Q-REF-6 para accesores de estado (isTerminated).

### AC-REF-001.2: Identidad de ActorRef: compareTo

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: compareTo ordena primero por path (ActorPath.compareTo) y, si empatan, por uid con resultado -1, 0 o 1.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:125-133
- **Condiciones** (normales, límites, errores, efectos): Normal: path distinto → resultado del compareTo del path. Límite: path igual y uid igual → 0. Errores: ninguno.
- **Orden / concurrencia / ciclo de vida**: sin concurrencia; puro.
- **Dependencias**: AC-PATH-014
- **Equivalente Rust propuesto**: `impl Ord for ActorRef`: orden por `ActorPath` y luego `uid`
- **Diferencias observables**: Pekko devuelve el entero del path.compareTo (cualquier magnitud); Rust devuelve `Ordering`. No observable salvo el signo.
- **Aserciones**:
  - A1: path igual y uid menor → -1; igual → 0; mayor → 1 — derivada de ActorRef.scala:129-133 (no afirmada por spec).
  - A2: path distinto → signo del ActorPath.compareTo — derivada de ActorRef.scala:129-131 (no afirmada por spec).
- **Oráculo**: no aplica: sin diferencia observable más que el signo; se puede comparar con jshell sobre Pekko 1.7.1 si se agrega escenario (jars oficiales).
- **Decisiones**: Q-REF-6
- **Notas**: Hijo de AC-REF-001. Ningún spec upstream de actor-tests lo afirma (búsqueda de compareTo en ActorRefSpec/ActorSelectionSpec sin resultado); aserciones derivadas del código.

### AC-REF-001.3: Identidad de ActorRef: toString

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: toString = "Actor[<path>]" si uid == 0, "Actor[<path>#<uid>]" en otro caso.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:188-190 ; actor/src/main/scala/org/apache/pekko/actor/ActorSelection.scala:143-151
- **Condiciones** (normales, límites, errores, efectos): Normal: actor vivo (uid != 0) incluye "#uid". Límite: deadLetters (uid 0) sin "#". Errores: ninguno.
- **Orden / concurrencia / ciclo de vida**: puro.
- **Dependencias**: AC-PATH-014
- **Equivalente Rust propuesto**: `impl Display for ActorRef`
- **Diferencias observables**: Ninguna; el formato de uid es decimal con signo (Int de 32 bits, puede ser negativo).
- **Aserciones**:
  - A1: uid presente en la representación de una selección: "ActorSelection[Anchor(pekko://ActorSelectionSpec/user/c2/c21#<uid>), Path(/../*/hello)]" — ActorSelectionSpec.scala:346-349.
  - A2: deadLetters no lleva uid: el mensaje de warning contiene "Actor[<deadLetters path>]" — LocalActorRefProviderSpec.scala:77 (EventFilter con `s"unhandled message from Actor[$deadLettersPath]: $message"`).
  - A3: ref de dos incarnations difieren en toString — ActorSelectionSpec.scala:143.
- **Oráculo**: jshell contra Pekko 1.7.1: imprimir ref de actor y de deadLetters (jars oficiales).
- **Decisiones**: -
- **Notas**: Contradicción corregida: el inventario viejo decía siempre "Actor[path#uid]"; el código omite "#uid" cuando uid == 0 (ActorRef.scala:188-190).

### AC-REF-002: tell / `!` (fire-and-forget) y Envelope

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: Envío asíncrono sin esperar al receptor. Un mensaje null lanza InvalidMessageException. Un sender ausente se sustituye por deadLetters al crear el Envelope. Un fallo al encolar no se propaga: se loguea como Error.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:141,157,457-458,684-691 ; actor/src/main/scala/org/apache/pekko/dispatch/AbstractDispatcher.scala:35-52 ; actor/src/main/scala/org/apache/pekko/dispatch/Dispatcher.scala:72-76 ; actor/src/main/scala/org/apache/pekko/actor/dungeon/Dispatch.scala:137-148,171-178
- **Condiciones** (normales, límites, errores, efectos): Mensaje null: con sender ausente "Message is null."; con sender "Message sent from [<sender>] is null." (AbstractDispatcher.scala:46-50). Envío a EmptyLocalActorRef/DeadLetterActorRef con null: InvalidMessageException("Message is null") (ActorRef.scala:685,737). Excepción durante el envío (p. ej. mailbox acotado lleno con timeout): se publica Error "swallowing exception during message send" (o con ": <mensaje>" si NoStackTrace) y NO se propaga (Dispatch.scala:137-148). Con pekko.actor.serialize-messages=on el mensaje se serializa y deserializa (Dispatch.scala:171-178; ver AC-CELL-028).
- **Orden / concurrencia / ciclo de vida**: encolar → registerForExecution con hasMessageHint (Dispatcher.scala:72-76). Orden FIFO por par emisor-receptor (AC-REF-003). No bloquea al emisor salvo mailbox acotado.
- **Dependencias**: AC-MBX-001
- **Equivalente Rust propuesto**: `fn tell(&self, msg: Message, sender: Option<&ActorRef>)`; el Envelope guarda `sender: ActorRef` ya resuelto a deadLetters
- **Diferencias observables**: Pekko acepta cualquier objeto; Rust usa el tipo de mensaje de Q-X-3. `null` y `InvalidMessageException` no existen en Rust (diferencia a favor).
- **Aserciones**:
  - A1: Un tell sin sender entrega sender()==deadLetters — ActorSelectionSpec.scala:281-285 (GetSender(testActor) → expectMsg(system.deadLetters)).
  - A2: Responder vía sender() llega al emisor original — ActorRefSpec.scala:390-413.
  - A3: Tell a un actor muerto publica DeadLetter(msg, sender, recipient) — DeadLetterSupressionSpec.scala:54 y 61.
  - A4: null → InvalidMessageException con los mensajes exactos — derivada de AbstractDispatcher.scala:44-50 (no afirmada por spec).
  - A5: Excepción al encolar se traga y se loguea — derivada de Dispatch.scala:137-148 (no afirmada por spec).
- **Oráculo**: jshell contra Pekko 1.7.1: tell sin sender y verificar sender()==deadLetters (jars oficiales).
- **Decisiones**: Q-X-3, Q-REF-1
- **Notas**: Q-X-3 (mensajes dinámicos). Q-REF-1: representación de "sin sender". `null` como mensaje desaparece en Rust; se documenta como no aplica.

### AC-REF-003: Garantías de entrega: at-most-once y orden por par emisor–receptor

- **Estado de análisis**: listed. Visibilidad: public.
- **Contrato observable**: Para un par (emisor, receptor) los mensajes de usuario llegan en orden de envío si llegan; sin garantía entre emisores ni reintentos; los mensajes de sistema (fallo de hijo) no se ordenan respecto de los de usuario.
- **Fuente**: docs/src/main/paradox/general/message-delivery-reliability.md:160-175 ; actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala:230-280
- **Condiciones** (normales, límites, errores, efectos): Normal: FIFO por emisor. Error: pérdida posible (at-most-once). Límites: mailbox cerrado → deadLetters.
- **Orden / concurrencia / ciclo de vida**: ver AC-CELL-013.1 para el orden entre mensajes de sistema y de usuario.
- **Dependencias**: AC-MBX-005
- **Equivalente Rust propuesto**: cola MPSC FIFO por actor
- **Diferencias observables**: Ninguna prevista.
- **Aserciones**:
  - A1: Un solo emisor: Ping, Die, Ping se observan en ese orden — SupervisorSpec.scala:389-399 (expectMsg PingMessage, ExceptionMessage, PingMessage).
  - A2: Fallo de hijo comunicado por mensaje de sistema sin orden respecto a los de usuario — docs message-delivery-reliability.md:171-175 (no afirmada por spec ejecutable).
- **Oráculo**: no aplica: depende de la planificación; se verifica con pruebas de orden en Rust, no con jshell.
- **Decisiones**: -
- **Notas**: Sigue listed: no leí message-delivery-reliability.md:41-42,108-159 ni MailboxConfigSpec.scala:77; ActorModelSpec.scala:311,331 NO afirman orden (sólo exclusión mutua y conteo, líneas 311-344). Los mensajes de sistema no siguen el orden respecto de los de usuario (docs :160-175).

### AC-REF-004: forward (preserva el sender original)

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: forward(msg) = tell(msg, context.sender()); sirve para tell, ask y ActorSelection.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:164 ; actor/src/main/scala/org/apache/pekko/actor/ActorSelection.scala:67
- **Condiciones** (normales, límites, errores, efectos): Normal: el sender del mensaje en curso se reenvía. Límite: sin sender (deadLetters) reenvía deadLetters. Errores: ninguno.
- **Orden / concurrencia / ciclo de vida**: igual que tell.
- **Dependencias**: AC-REF-005
- **Equivalente Rust propuesto**: `fn forward(&self, msg, ctx: &Context)`
- **Diferencias observables**: Ninguna.
- **Aserciones**:
  - A1: Cadena de 3 forwarders con tell desde `replyTo`: el mensaje llega al destino con el sender original — ForwardActorSpec.scala:45-54.
  - A2: Con ask: la respuesta completa el Future original — ForwardActorSpec.scala:56-60.
  - A3: forward sobre ActorSelection preserva sender: el destino responde al testActor y lastSender es c21 — ActorSelectionSpec.scala:435-439.
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales): cadena de forward y sender observado.
- **Decisiones**: -

### AC-REF-005: sender()

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: sender() devuelve el sender del mensaje en proceso; si no hay mensaje en curso o el mensaje no tiene sender, devuelve deadLetters.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:596-600 ; actor/src/main/scala/org/apache/pekko/actor/Actor.scala:526 ; actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:559
- **Condiciones** (normales, límites, errores, efectos): Normal: sender del Envelope. Límite: fuera de receive tras clearActorFields (restart) o antes del primer mensaje → deadLetters. Errores: ninguno.
- **Orden / concurrencia / ciclo de vida**: sólo válido durante el procesamiento del mensaje (AC-CELL-019).
- **Dependencias**: AC-CELL-003
- **Equivalente Rust propuesto**: `ctx.sender() -> ActorRef` (deadLetters por defecto)
- **Diferencias observables**: en Rust el sender se pasa por préstamo; no hay estado mutable capturable.
- **Aserciones**:
  - A1: tell sin sender → sender()==deadLetters — ActorSelectionSpec.scala:281-285.
  - A2: reply por sender() llega — ActorRefSpec.scala:390-413.
  - A3: tras fallo y Resume sender() conserva el del mensaje fallido — derivada de ActorCell.scala:559 (no afirmada por spec).
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales).
- **Decisiones**: Q-REF-1
- **Notas**: Dato nuevo: currentMessage sólo se pone a null tras un procesamiento exitoso (ActorCell.scala:559) y en clearActorFields; tras un fallo con Resume sender() sigue devolviendo el sender del mensaje fallido.

### AC-REF-006: Los actores sólo se crean vía actorOf

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: Crear un Actor con `new` fuera de actorOf lanza ActorInitializationException; dentro de un actor ocurre lo mismo si se instancia un actor anidado con `new`.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/Actor.scala:495-506 ; actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:622-640
- **Condiciones** (normales, límites, errores, efectos): Mensaje exacto (sin path, actor == null): "You cannot create an instance of [<clase>] explicitly using the constructor (new). You have to use one of the 'actorOf' factory methods to create a new actor. See the documentation." (Actor.scala:496-501). La pila de contextos queda vacía tras el error (ActorRefSpec.scala:155).
- **Orden / concurrencia / ciclo de vida**: el contexto se obtiene del thread-local contextStack durante newActor (ActorCell.scala:622-640).
- **Dependencias**: AC-CELL-002
- **Equivalente Rust propuesto**: la restricción es estructural: `Actor` no tiene constructor con acceso a `Context`; `Props` construye
- **Diferencias observables**: en Rust no hay forma de construir el contexto fuera de la celda (compile-time); el test no se porta.
- **Aserciones**:
  - A1: `new Actor` fuera de actorOf → ActorInitializationException — ActorRefSpec.scala:150-152.
  - A2: actor anidado creado con new dentro de actorOf: se loguea 1 ActorInitializationException y contextStack queda vacío — ActorRefSpec.scala:154-165 (EventFilter occurrences=1 y contextStackMustBeEmpty).
- **Oráculo**: no aplica: comportamiento ligado a la JVM (thread-local); no hay equivalente en Rust.
- **Decisiones**: Q-REF-4
- **Notas**: Q-REF-4: el caso de error no existe en Rust; se propone marcarlo no aplicable con justificación (decisión pendiente del Owner).

### AC-REF-007.1: Props: formas de creación, valores por defecto y validación

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: Props es inmutable; Props.empty crea un actor sin conducta (todo unhandled); Props.default usa defaultCreator, que lanza UnsupportedOperationException("No actor creator specified!"); formas: Props[T](), Props(creator), Props(clazz, args*), Props.create(...).
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/Props.scala:32-124 ; actor/src/main/scala/org/apache/pekko/actor/Props.scala:183-204
- **Condiciones** (normales, límites, errores, efectos): Normal: actorOf(props). Errores: creator por defecto → UnsupportedOperationException("No actor creator specified!"); firma de constructor inválida → IllegalArgumentException al construir Props (validación del producer). Defaults: dispatcher "pekko.actor.default-dispatcher" (Dispatchers.DefaultDispatcherId) y mailbox default si deploy tiene NoDispatcherGiven/NoMailboxGiven (Props.scala dispatcher/mailbox).
- **Orden / concurrencia / ciclo de vida**: la fábrica corre dentro de la celda durante Create (AC-CELL-002.1), no en el hilo del llamador de actorOf.
- **Dependencias**: AC-CFG-013
- **Equivalente Rust propuesto**: `Props<A>` con fábrica `Arc<dyn Fn() -> A + Send + Sync>`; `Props::empty()`, `Props::default_props()`
- **Diferencias observables**: sin reflexión (clase + args): se reemplaza por closures (Q-REF-2).
- **Aserciones**:
  - A1: Props con creator, con classOf, con 1 y 2 params null y la Java API crean actores sin error con serialize-creators=on — PropsCreationSpec.scala:54-80 (system.actorOf(p) sin excepción).
  - A2: defaultCreator lanza UnsupportedOperationException("No actor creator specified!") — derivada de Props.scala (final val defaultCreator) (no afirmada por spec).
- **Oráculo**: jshell contra Pekko 1.7.1: Props.empty / Props.default().newActor (jars oficiales).
- **Decisiones**: Q-REF-2, Q-X-2
- **Notas**: Hijo de AC-REF-007. Q-REF-2 (reflexión). Los tests PropsCreationSpec sólo afirman que actorOf no lanza con serialize-creators=on.

### AC-REF-007.2: Props: modificadores withDispatcher/withMailbox/withRouter/withDeploy/withActorTags

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: Cada withX devuelve una copia; withDispatcher/withMailbox devuelven `this` si el valor ya es el mismo; withDeploy hace d.withFallback(deploy); withActorTags agrega tags al deploy.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/Props.scala:183-204
- **Condiciones** (normales, límites, errores, efectos): dispatcher cuando deploy.dispatcher == NoDispatcherGiven → DefaultDispatcherId; mailbox análogo. withRouter reemplaza routerConfig del deploy.
- **Orden / concurrencia / ciclo de vida**: inmutable, seguro entre hilos.
- **Dependencias**: AC-CFG-013
- **Equivalente Rust propuesto**: `Props::with_dispatcher(&self, id) -> Props` (copia; Arc compartido)
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: withDispatcher(d) con deploy sin dispatcher devuelve copia con d — derivada de Props.scala (withDispatcher) (no afirmada por spec).
  - A2: withDispatcher(x) con el mismo x devuelve la misma instancia — derivada del código (no afirmada por spec).
  - A3: withDeploy(d) = d.withFallback(deploy) — derivada del código (no afirmada por spec).
- **Oráculo**: jshell contra Pekko 1.7.1: encadenar withX y leer dispatcher/mailbox/deploy (jars oficiales).
- **Decisiones**: Q-REF-2
- **Notas**: Hijo de AC-REF-007. No hay spec en actor-tests que afirme withX (PropsCreationSpec no los usa); aserciones derivadas del código.

### AC-REF-008: IndirectActorProducer y constructores por reflexión

- **Estado de análisis**: listed. Visibilidad: internal-required.
- **Contrato observable**: El producer se elige según la forma de Props (creator, clase sin args, clase con args, Creator Java).
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/IndirectActorProducer.scala:30-122
- **Condiciones** (normales, límites, errores, efectos): pendiente de lectura de IndirectActorProducer.scala.
- **Orden / concurrencia / ciclo de vida**: n/a: pendiente.
- **Dependencias**: AC-REF-007.1
- **Equivalente Rust propuesto**: trait `ActorFactory` / closure
- **Diferencias observables**: mapeo a closure (Q-REF-2).
- **Aserciones**:
  - A1: Props con classOf + 1 y 2 params null funcionan — PropsCreationSpec.scala:58-70.
- **Oráculo**: no aplica: reflexión de JVM.
- **Decisiones**: Q-REF-2
- **Notas**: Sigue listed: no leí IndirectActorProducer.scala (sólo PropsCreationSpec). Q-REF-2: excluir la parte reflexiva (propuesta; la fila sigue en alcance).

### AC-REF-009.1: actorOf: validación del nombre

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: actorOf(props, name) con name null → InvalidActorNameException("actor name must not be null"); vacío → "actor name must not be empty"; otro valor pasa por ActorPath.validatePathElement.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/Children.scala:58-64,269-277 ; actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:230,246
- **Condiciones** (normales, límites, errores, efectos): Mensajes que deben contener: null→"null"; vacío→"empty"; "$hallo"→"not start with `$`"; "a%","%3","%xx","%0G","%gg","%","%1t","a?"→"Invalid actor path element"; "üß"→"include only ASCII"; `he"llo`→`["] at position: 2`; `$hello`→`[$] at position: 0`; `hell>o`→`[>] at position: 4`. Los nombres generados por el sistema empiezan por "$" y son válidos sólo internamente.
- **Orden / concurrencia / ciclo de vida**: la validación ocurre en el hilo del llamador, antes de reservar el nombre.
- **Dependencias**: AC-PATH-008
- **Equivalente Rust propuesto**: `ctx.actor_of(props, name) -> Result<ActorRef, InvalidActorName>`
- **Diferencias observables**: ninguna; el mensaje de ActorPath se decide en AC-PATH-008.
- **Aserciones**:
  - A1: null/"" lanzan InvalidActorNameException con "null" / "empty" en el mensaje — LocalActorRefProviderSpec.scala:169-170.
  - A2: caracteres inválidos y posición en el mensaje — LocalActorRefProviderSpec.scala:171-197.
- **Oráculo**: jshell contra Pekko 1.7.1: system.actorOf(Props.empty, name) con los mismos nombres (jars oficiales).
- **Decisiones**: -
- **Notas**: Hijo de AC-REF-009. La validación del path element la define AC-PATH-008 (ActorPath.validatePathElement); aquí sólo null/vacío y delegación.

### AC-REF-009.2: actorOf: unicidad y reserva atómica del nombre

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: El nombre se reserva (ChildNameReserved) antes de crear el hijo; si ya existe (vivo o reservado, incluido uno parándose) lanza InvalidActorNameException("actor name [<n>] is not unique!"). Si la creación falla se libera la reserva.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/ChildrenContainer.scala:142-145,205-211 ; actor/src/main/scala/org/apache/pekko/actor/dungeon/Children.scala:161-170,279-345
- **Condiciones** (normales, límites, errores, efectos): Normal: único gana en concurrencia. Error: segundo → InvalidActorNameException. Reutilizar un nombre sólo es posible tras procesar la terminación del hijo (Terminated). Durante Termination de la celda, reservar lanza IllegalStateException("cannot reserve actor name '<n>': terminating"); en TerminatedChildrenContainer: "already terminated" (ChildrenContainer.scala:205-211 y TerminatedChildrenContainer).
- **Orden / concurrencia / ciclo de vida**: tras context.stop(kid) el nombre sigue reservado hasta que el padre procesa la DeathWatchNotification del hijo (AC-DW-008).
- **Dependencias**: AC-CELL-015.1
- **Equivalente Rust propuesto**: reserva atómica del nombre en el contenedor de hijos (CAS o lock)
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: 4 actorOf concurrentes con el mismo nombre → un éxito y fallos InvalidActorNameException (x100) — LocalActorRefProviderSpec.scala:140-153.
  - A2: dos actorOf con el mismo nombre en la misma invocación → InvalidActorNameException logueada 1 vez — LocalActorRefProviderSpec.scala:156-165.
  - A3: stop(kid) y actorOf mismo nombre inmediato → InvalidActorNameException — SupervisorMiscSpec.scala:126-141.
  - A4: tras Terminated(kid) se puede crear nuevo hijo con el mismo nombre, otra instancia, mismo path — SupervisorMiscSpec.scala:102-124.
- **Oráculo**: jshell contra Pekko 1.7.1: carrera de actorOf con el mismo nombre (jars oficiales; resultado no determinista en orden, sí en conteo).
- **Decisiones**: -
- **Notas**: Hijo de AC-REF-009.

### AC-REF-009.3: actorOf anónimo: nombres generados

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: Sin nombre, el hijo se llama "$" + base64(contador) donde el contador es por celda y empieza en 0 ("$a","$b",…,"$~","$ab",…).
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/Children.scala:124-131 ; actor/src/main/scala/org/apache/pekko/util/Helpers.scala:115-123
- **Condiciones** (normales, límites, errores, efectos): Normal: contador atómico por padre. Límite: tras 64 nombres pasa a dos caracteres ("$ab"). Los FunctionRef usan prefijo "$$".
- **Orden / concurrencia / ciclo de vida**: el contador se incrementa en cada actorOf anónimo incluso si luego falla.
- **Dependencias**: AC-PATH-008
- **Equivalente Rust propuesto**: contador por celda → nombre base64 con prefijo `$`
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: primer nombre anónimo es "$a" — derivada de Helpers.scala:118-123 y Children.scala randomName (no afirmada por spec).
  - A2: tras 64 nombres, el 65º es "$ab" — derivada de Helpers.scala:118-123 (no afirmada por spec).
- **Oráculo**: jshell contra Pekko 1.7.1: crear 70 hijos anónimos y listar nombres (jars oficiales).
- **Decisiones**: -
- **Notas**: Hijo de AC-REF-009. Ningún spec upstream afirma la secuencia exacta; derivada del código. El alfabeto es "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+~".

### AC-REF-010: children, child(name), getChild, findChild

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: children devuelve los hijos vivos (no reservados) en orden de nombre; child(name) devuelve Option; getChild devuelve null; findChild devuelve Optional.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/Children.scala:44-56 ; actor/src/main/scala/org/apache/pekko/actor/dungeon/ChildrenContainer.scala:79,120-136
- **Condiciones** (normales, límites, errores, efectos): Normal: hijo existente. Límite: nombre reservado pero no iniciado → None. Errores: ninguno.
- **Orden / concurrencia / ciclo de vida**: lectura racy fuera del actor.
- **Dependencias**: AC-CELL-015.1
- **Equivalente Rust propuesto**: `ctx.children() -> impl Iterator<Item=ActorRef>`, `ctx.child(&str) -> Option<ActorRef>`
- **Diferencias observables**: orden de iteración: nombre ascendente (TreeMap) — Rust debe usar BTreeMap para igualar.
- **Aserciones**:
  - A1: child("child") definido, child("whatnot") vacío — ActorRefSpec.scala:465-477 (isDefined true/false).
  - A2: orden alfabético de children — derivada de ChildrenContainer (TreeMap) (no afirmada por spec).
- **Oráculo**: jshell contra Pekko 1.7.1: crear hijos "b","a","c" y listar children (jars oficiales).
- **Decisiones**: -
- **Notas**: Detalle nuevo: el contenedor es TreeMap por nombre → children se itera en orden alfabético por nombre; las reservas (ChildNameReserved) no aparecen.

### AC-REF-011: ActorSelection: actorSelection(path|ActorPath), relativo, `..`, comodines * y ?; tell/forward

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: actorSelection(String|ActorPath) construye una selección sobre el árbol evaluada en el momento de entrega; los elementos se resuelven por nombre, ".." (padre) o patrón con '*' y '?'; mensajes a elementos inexistentes se publican como DeadLetter; con comodines se entrega a todos los que coinciden.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/ActorSelection.scala:43-70,211-232,239-302 ; actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:255-274
- **Condiciones** (normales, límites, errores, efectos): Parseo: path relativo vacío "" → ancla deadLetters; que empieza por "/" → ancla rootGuardian; otro relativo → ancla lookupRoot (el actor consultado, ActorRefProvider.scala:255-271); ruta absoluta con address ajena → rootGuardianAt(address)==deadLetters; string no parseable → ancla deadLetters (ActorRefProvider.scala:255-274). Elementos vacíos se descartan (`user/` ≡ `user`) (ActorSelection.scala:221-226). ".." sobre el root devuelve el root. Strings ya probados: "/user","/system","/","","..","user","system","user/","system/" resuelven a su guardián (ActorSelectionSpec.scala:163-176); "a/b/c","pekko://all-systems/Nobody","pekko://all-systems/user","foo://user","/deadLetters","deadLetters","deadLetters/" → ActorIdentity(None) (:180-188).
- **Orden / concurrencia / ciclo de vida**: la evaluación ocurre al entregar (deliverSelection); la entrega a hijos con comodín se hace en orden de iteración del contenedor de hijos (alfabético).
- **Dependencias**: AC-REF-013 ; AC-REF-014
- **Equivalente Rust propuesto**: `ActorSelection { anchor, elements: Vec<SelectionElement> }` con `SelectionElement::{Name, Pattern, Parent}`
- **Diferencias observables**: las selecciones contra address ajena resuelven a deadLetters igual que Pekko local (sin remote).
- **Aserciones**:
  - A1: Selección por ActorPath/String/relativo/absoluta/Java API encuentra los actores — ActorSelectionSpec.scala:100-122,150-160.
  - A2: Incarnation vieja no se encuentra: Identify → None — ActorSelectionSpec.scala:124-148 (identify(a1.path) === None).
  - A3: Guardianes y raíz seleccionables — ActorSelectionSpec.scala:162-177; no existentes → None — :179-189.
  - A4: Desde un ActorContext: relativo, "..", "../..", más arriba que root se queda en root — ActorSelectionSpec.scala:230-243.
  - A5: Selección con comodín envía a todos con el sender correcto: ActorSelection(c21,"../../*") desde c1 llega a c1 y c2 — ActorSelectionSpec.scala:299-307.
  - A6: Mensajes que no se pueden entregar se descartan (sólo llega a c21 con "../../*/c21") — ActorSelectionSpec.scala:309-317.
  - A7: Selección a actor ausente: DeadLetter(msg="boom", sender=testActor, recipient.path="/user/missing") — ActorSelectionSpec.scala:361-369.
  - A8: tell directo a selección con ancla y path "" llega al ancla con sender deadLetters — ActorSelectionSpec.scala:281-285.
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales): parseo de strings y resolución con Identify; mismas tablas de A3.
- **Decisiones**: Q-X-3
- **Notas**: Amplía el inventario viejo: reglas de parseo (split por "/+", elementos vacíos descartados, ".." → SelectParent, '?' o '*' → patrón) y el ancla (""→deadLetters). Ver AC-REF-026 (equals/toString) y AC-REF-014 (entrega).

### AC-REF-012: resolveOne(timeout) y ActorNotFound

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: resolveOne envía Identify(None) por ask y completa con el ref si ActorIdentity(_, Some(ref)); en cualquier otro resultado (None, timeout, fallo) falla con ActorNotFound(selection).
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/ActorSelection.scala:79-141,387
- **Condiciones** (normales, límites, errores, efectos): Mensaje de ActorNotFound: "Actor not found for: " + selection.toString (ActorSelection.scala:387). Variantes: resolveOne() con timeout implícito, resolveOne(FiniteDuration), resolveOne(java.time.Duration) → CompletionStage; resolveOneCS (deprecated).
- **Orden / concurrencia / ciclo de vida**: la ejecución del callback usa un contexto paralelo/parasitic.
- **Dependencias**: AC-ASK-001 ; AC-REF-013
- **Equivalente Rust propuesto**: `async fn resolve_one(&self, timeout) -> Result<ActorRef, ActorNotFound>`
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: resolveOne con timeout explícito e implícito devuelve c2 — ActorSelectionSpec.scala:319-329.
  - A2: path inexistente → ActorNotFound — ActorSelectionSpec.scala:331-335.
  - A3: resolveOne concuerda con Identify manual (None ⇒ falla) — ActorSelectionSpec.scala:81-82.
  - A4: timeout también produce ActorNotFound — derivada de ActorSelection.scala:84 (no afirmada por spec).
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales): resolveOne sobre inexistente y sobre actor ocupado sin responder.
- **Decisiones**: -
- **Notas**: Contradicción corregida: el inventario decía "falla con ActorNotFound" sólo si no existe; el código usa `case _ => p.failure(ActorNotFound(this))`, o sea que también un timeout/fallo del ask produce ActorNotFound (no AskTimeoutException).

### AC-REF-013: Identify / ActorIdentity

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: Todo actor responde Identify(id) con ActorIdentity(id, Some(self)) sin pasar por receive; deadLetters y EmptyLocalActorRef responden ActorIdentity(id, None). Identify no reinicia el receive-timeout.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/Actor.scala:76-111 ; actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:578 ; actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:693-704,736-738
- **Condiciones** (normales, límites, errores, efectos): ActorIdentity(c, Some(null)) → IllegalArgumentException (mensaje "ActorIdentity created with ref = Some(null) is not allowed, this could happen when serializing with Scala 2.12 and deserializing with Scala 2.11 which is not supported."). Selección con comodín: sólo responden los existentes; si ninguno coincide responde None una vez (ActorSelectionSpec.scala:408-409).
- **Orden / concurrencia / ciclo de vida**: se maneja como AutoReceivedMessage (ActorCell.scala:549-582).
- **Dependencias**: AC-CELL-003.1
- **Equivalente Rust propuesto**: mensaje manejado por la celda antes del receive de usuario
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: Identify de selección a ref existente devuelve Some(ref), a inexistente None — ActorSelectionSpec.scala:70-85,179-189.
  - A2: Identify con "/user/a/*" devuelve un ActorIdentity por hijo (b1,b2) y nada más — ActorSelectionSpec.scala:398-406.
  - A3: "/user/a/b1/*" (sin hijos) → un solo ActorIdentity(None) — ActorSelectionSpec.scala:408-409.
  - A4: "/user/a/*/c/d/e" sin coincidencias en fan-out → ninguna respuesta — ActorSelectionSpec.scala:431-432.
  - A5: Identify no reinicia el receive timeout: llega ReceiveTimeout aunque lleguen Identify cada 100ms — ReceiveTimeoutSpec.scala:151-171.
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales): Identify directo y por selección.
- **Decisiones**: -
- **Notas**: Identify es NotInfluenceReceiveTimeout (Actor.scala:81). ActorIdentity(corr, Some(null)) lanza IllegalArgumentException (Actor.scala:92-96).

### AC-REF-014: Entrega de selección con comodines (ActorSelectionMessage, SelectChildName/Pattern/Parent, deliverSelection)

- **Estado de análisis**: listed. Visibilidad: internal-required.
- **Contrato observable**: deliverSelection recorre el árbol: nombre → hijo; ".." → padre; patrón → fan-out sobre children que coinciden; en fan-out (wildcardFanOut=true) no se envía a EmptyLocalActorRef; al llegar al final entrega el mensaje al/los destinos con el sender original; elementos no encontrados fuera de fan-out envían el ActorSelectionMessage a un EmptyLocalActorRef (que responde None a Identify o publica DeadLetter).
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/ActorSelection.scala:239-302,322-380 ; actor/src/main/scala/org/apache/pekko/util/Glob.scala
- **Condiciones** (normales, límites, errores, efectos): wildcardFanOut = true si el patrón coincide con más de 1 hijo (`matchingChildren.size > 1`). Patrón sin coincidencia en hoja y sin fan-out → emptyRef.tell(sel). Ref foráneo (no ActorRefWithCell) recibe el ActorSelectionMessage con los elementos restantes.
- **Orden / concurrencia / ciclo de vida**: la evaluación del patrón y la entrega ocurren en el hilo que llama tell (no en el actor).
- **Dependencias**: AC-REF-011
- **Equivalente Rust propuesto**: matcher glob propio (sin regex)
- **Diferencias observables**: patrones sin backtracking exponencial (Glob).
- **Aserciones**:
  - A1: Patrón "*a"×12+"*b" sobre nombre de 36 'a' responde ActorIdentity(4,None) en <3s — ActorSelectionSpec.scala:371-386.
  - A2: Casos de Identify con comodín (*/c, b2/*/d, */*/d, */c/*, b2/*/d/e) — ActorSelectionSpec.scala:411-429.
  - A3: Fan-out sin coincidencias no responde (wildcardFanOut) — ActorSelectionSpec.scala:431-432.
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales): mismas selecciones sobre árbol fijo.
- **Decisiones**: -
- **Notas**: Sigue listed: no leí util/Glob.scala ni GlobSpec.scala:33; lo demás (deliverSelection) está leído. El patrón se compara con Glob.matches(patternStr, name) y no con regex (ActorSelection.scala:369).

### AC-REF-015: EmptyLocalActorRef (ref a actor inexistente)

- **Estado de análisis**: extracted. Visibilidad: internal-required.
- **Contrato observable**: Un EmptyLocalActorRef publica DeadLetter por cada mensaje; Watch(watchee==this, watcher!=this) → envía DeathWatchNotification(existenceConfirmed=false, addressTerminated=false) al watcher; Unwatch se ignora; Identify → ActorIdentity(None); mensaje DeadLetterSuppression → SuppressedDeadLetter; ActorSelectionMessage con Identify responde None salvo fan-out; sin Identify publica DeadLetter con el mensaje interior.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:671-728
- **Condiciones** (normales, límites, errores, efectos): mensaje null → InvalidMessageException("Message is null"); DeadLetter recibido se "desenvuelve" (specialHandle(d.message, d.sender)) para evitar bucles; sender noSender → deadLetters como remitente del DeadLetter. isTerminated = true.
- **Orden / concurrencia / ciclo de vida**: sin orden especial.
- **Dependencias**: AC-EVT-008
- **Equivalente Rust propuesto**: variante `RefInner::EmptyLocal`
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: Deserializar un ref de actor ya parado produce un EmptyLocalActorRef igual a uno nuevo con el mismo path — ActorRefSpec.scala:341-364 (readObject should ===(new EmptyLocalActorRef(provider, ref.path, eventStream))).
  - A2: ask sobre ref vacío falla de inmediato con AskTimeoutException — AskSpec.scala:50-59.
  - A3: mensaje normal → DeadLetter y suprimible → SuppressedDeadLetter (también por ActorSelection) — DeadLetterSupressionSpec.scala:41-72.
  - A4: Watch → DWN existenceConfirmed=false — derivada de ActorRef.scala:693-698 (no afirmada por spec ejecutable; ver AC-DW-002).
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales): watch sobre resolveActorRef de path inexistente.
- **Decisiones**: -
- **Notas**: Dato nuevo: sendSystemMessage de un EmptyLocalActorRef redirige a deadLetters con specialHandle.

### AC-REF-016: deadLetters (DeadLetterActorRef)

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: Todo mensaje a deadLetters se publica como DeadLetter(msg, sender|deadLetters, deadLetters); un DeadLetter recibido se re-publica tal cual; Identify responde None; mensajes suprimibles generan SuppressedDeadLetter.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:733-760 ; actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:414-417
- **Condiciones** (normales, límites, errores, efectos): sender por defecto = deadLetters (no noSender). Con DeadLetterSuppression: no DeadLetter pero sí SuppressedDeadLetter y AllDeadLetters (DeadLetterSupressionSpec.scala:87-92).
- **Orden / concurrencia / ciclo de vida**: publicación síncrona en EventStream.
- **Dependencias**: AC-EVT-008
- **Equivalente Rust propuesto**: ref especial que publica en EventStream
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: normal → DeadLetter(NormalMsg, testActor, deadLetters) y suprimible → SuppressedDeadLetter, ambos en AllDeadLetters — DeadLetterSupressionSpec.scala:84-93.
  - A2: sin duplicados tras 200ms — DeadLetterSupressionSpec.scala:94-97.
  - A3: vía ActorSelection se desempaqueta igual — DeadLetterSupressionSpec.scala:99-105.
  - A4: el logging de dead letters (INFO "…not delivered", WARNING "received dead letter from …") pertenece a AC-EVT; aquí sólo se publica — ActorSystemSpec.scala:164-186 (referencia).
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales): suscribir a DeadLetter y enviar.
- **Decisiones**: -
- **Notas**: Diferencia con EmptyLocalActorRef: el sender implícito de `!` es `this` (no noSender) y Watch sólo responde DWN si watchee y watcher no son deadLetters (ActorRef.scala:746-752). Path "/deadLetters" -> Identify devuelve None (ActorSelectionSpec.scala:186-188).

### AC-REF-017: resolveActorRef(path) del proveedor

- **Estado de análisis**: extracted. Visibilidad: internal-required.
- **Contrato observable**: resolveActorRef(path: String) → ActorPathExtractor con la address local; si no, deadLetters. resolveActorRef(ActorPath) → si path.root != rootPath deadLetters. Elementos vacíos → deadLetters. getChild == Nobody → EmptyLocalActorRef con ruta ref.path / elementos; si existe, el ref.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:622-657
- **Condiciones** (normales, límites, errores, efectos): Se loguea Debug (logDeser) en cada caso de fallback.
- **Orden / concurrencia / ciclo de vida**: lectura racy del árbol.
- **Dependencias**: AC-SYS-018
- **Equivalente Rust propuesto**: `provider.resolve_actor_ref(&ActorPath) -> ActorRef`
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: hijo con nombre URL-encoded se resuelve con provider.resolveActorRef(child.path) y no está terminado — LocalActorRefProviderSpec.scala:49-65.
  - A2: ref serializado de actor parado se resuelve a EmptyLocalActorRef — ActorRefSpec.scala:341-364.
  - A3: address ajena/ruta vacía → deadLetters — derivada de ActorRefProvider.scala:630-653 (no afirmada por spec).
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales): resolveActorRef con las tres formas.
- **Decisiones**: -
- **Notas**: Contradicción corregida: el inventario decía "address ajena o inválida → deadLetters; inexistente → EmptyLocalActorRef". Verdad: además un path vacío (sólo raíz) → deadLetters (ActorRefProvider.scala:645-648).

### AC-REF-018: Jerarquía interna InternalActorRef / MinimalActorRef / ActorRefWithCell / Nobody / LocalRef / RepointableRef

- **Estado de análisis**: extracted. Visibilidad: internal-required.
- **Contrato observable**: Operaciones internas start/suspend/resume/restart/stop/sendSystemMessage/getParent/getChild/isLocal/provider; MinimalActorRef las implementa como no-ops; isTerminated por defecto false.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:223-330,503-522
- **Condiciones** (normales, límites, errores, efectos): Errores: Nobody.provider → UnsupportedOperationException("Nobody does not provide"). getParent de Minimal → Nobody.
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: AC-CELL-013.1
- **Equivalente Rust propuesto**: trait sellado `RefInner` con variantes; métodos internos `pub(crate)`
- **Diferencias observables**: isLocal y RepointableRef son hooks para remote/cluster (fuera de alcance).
- **Aserciones**:
  - A1: MinimalActorRef.getChild(names) = this si todos vacíos, si no Nobody — derivada de ActorRef.scala:506 (no afirmada por spec).
  - A2: Nobody.provider lanza UnsupportedOperationException — derivada de ActorRef.scala:329 (no afirmada por spec).
- **Oráculo**: no aplica: tipos internos.
- **Decisiones**: -
- **Notas**: Sin spec. Nobody.path = Address("pekko","all-systems") + "/Nobody"; Nobody.provider lanza UnsupportedOperationException("Nobody does not provide") (ActorRef.scala:327-329). Dato nuevo: MinimalActorRef.getChild devuelve this si todos los elementos son vacíos, si no Nobody (ActorRef.scala:506).

### AC-REF-019: LocalActorRef: getChild por nombres (incluye FunctionRefs) e isTerminated monotónico

- **Estado de análisis**: extracted. Visibilidad: internal-required.
- **Contrato observable**: isTerminated = mailbox.isClosed; una vez true nunca vuelve a false; puede ser false aunque el actor ya esté muriendo (racy). getChild resuelve hijos y function refs por nombre, con uid opcional ("name#uid").
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:166-175,350-460 ; actor/src/main/scala/org/apache/pekko/actor/dungeon/Children.scala:230-245 ; actor/src/main/scala/org/apache/pekko/actor/dungeon/Dispatch.scala:72
- **Condiciones** (normales, límites, errores, efectos): nombre con '#' y uid distinto → Nobody. getChild(".."): padre; getChild(""): el mismo.
- **Orden / concurrencia / ciclo de vida**: el mailbox pasa a Closed en detach (finishTerminate).
- **Dependencias**: AC-CELL-008.2
- **Equivalente Rust propuesto**: `AtomicBool` de mailbox cerrado
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: tras Kill hasta el fin de max retries, terminal.isTerminated === true — DeathWatchSpec.scala:210.
  - A2: FunctionRef parado: isTerminated cuando _watchedBy vacío — derivada de ActorRef.scala:924 (no afirmada por spec).
- **Oráculo**: no aplica.
- **Decisiones**: Q-REF-6
- **Notas**: Q-REF-6 (visibilidad de isTerminated). getChild resuelve "..", "" (se queda), nombre con "#uid" (verifica uid), luego FunctionRefs (Children.scala:230-245). Si el uid no coincide devuelve Nobody.

### AC-REF-020: RepointableActorRef / UnstartedCell (creación asíncrona)

- **Estado de análisis**: listed. Visibilidad: internal-required.
- **Contrato observable**: Con async=true la creación del ref es inmediata con una UnstartedCell que encola mensajes de usuario y de sistema; el guardián, al procesar Supervise(child, async=true), llama point() que crea la celda real y entrega primero los mensajes de sistema encolados y luego los de usuario en orden.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/RepointableActorRef.scala:40-202,204-326 ; actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:659-719 ; actor/src/main/scala/org/apache/pekko/actor/dungeon/Children.scala:42-60 ; actor/src/main/resources/reference.conf:145
- **Condiciones** (normales, límites, errores, efectos): sendMessage espera el lock hasta unstarted-push-timeout (10s) y si falla publica Warning "dropping message of type … due to lock timeout" y envía a deadLetters; cola llena → Warning "dropping message … due to enqueue failure". Si newCell falla con catchFailures: celda con dispatcher global por defecto y Create(ActorInitializationException("failure while creating ActorCell")). children/getSingleChild de UnstartedCell → vacío/Nobody.
- **Orden / concurrencia / ciclo de vida**: replaceWith drena primero sysmsgQueue (en orden de envío, usando reverse) y luego la cola de usuario, vaciando sysmsgQueue después de cada mensaje (RepointableActorRef.scala:225-251).
- **Dependencias**: AC-CELL-016
- **Equivalente Rust propuesto**: cola previa en el ref hasta que la celda está lista
- **Diferencias observables**: Q-REF-5: Rust puede crear la celda de forma síncrona siempre que preserve estas observables.
- **Aserciones**:
  - A1: Los refs creados por actorOf asíncrono terminan siendo ya no-Unstarted o terminados — ActorSystemSpec.scala:323-325.
  - A2: Orden de drenaje: sistema antes que usuario — derivada de RepointableActorRef.scala:241-247 (no afirmada por spec).
  - A3: timeout de lock descarta a deadLetters con Warning — derivada de RepointableActorRef.scala:283-290 (no afirmada por spec).
- **Oráculo**: no aplica directamente (timing); se puede probar con jshell un envío inmediato tras actorOf (jars oficiales) para orden.
- **Decisiones**: Q-REF-5
- **Notas**: Sigue listed: no leí RoutingSpec.scala:206 (cita del inventario); ActorSystemSpec.scala:323-325 sólo afirma que los refs creados están terminados o ya no son UnstartedCell. Hallazgo nuevo: system.actorOf usa attachChild(async=true) → RepointableActorRef; context.actorOf usa async=false → LocalActorRef (ActorRefProvider.scala:711-713). Q-REF-5.

### AC-REF-021: FunctionRef (ref respaldado por una función, hijo virtual del actor)

- **Estado de análisis**: extracted. Visibilidad: internal-required.
- **Contrato observable**: Reenvía cada mensaje a la función con el sender; puede ser vigilado (Watch/Unwatch) y vigilar a otros (watch/unwatch propios); muere cuando muere el padre (stopFunctionRefs); no aparece entre children; al pararse reemplaza la función por deadLetterMessageHandler (evita retener memoria y bucles con DeadLetter).
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:861-1000 ; actor/src/main/scala/org/apache/pekko/actor/dungeon/Children.scala:68-122
- **Condiciones** (normales, límites, errores, efectos): Terminated recibido por un FunctionRef que vigila: se entrega a la función con existenceConfirmed=true y NO se auto-limpia: hay que llamar unwatch (FunctionRefSpec.scala:83-86). removeFunctionRef(ref) exige ref.path.parent eq self.path (require); FunctionRef no registrado no se encuentra: al serializar con serialize-messages=on falla (SerializationCheckFailedException "Failed to serialize and deserialize message of type org.apache.pekko.actor.FunctionRefSpec…") (FunctionRefSpec.scala:108-120).
- **Orden / concurrencia / ciclo de vida**: stop() → sendTerminated() notifica a watchers remotos primero y luego locales (ActorRef.scala:927-956). AddressTerminated se procesa en `!` (ActorRef.scala:897-898).
- **Dependencias**: AC-DW-001
- **Equivalente Rust propuesto**: `FunctionRef` con closure `Fn(ActorRef sender, Msg)`
- **Diferencias observables**: sin serialización Java (AC-REF-023).
- **Aserciones**:
  - A1: reenvía mensajes (Forwarded("hello", testActor)) — FunctionRefSpec.scala:62-65.
  - A2: es vigilable: tras removeFunctionRef el watcher recibe Terminated — FunctionRefSpec.scala:67-73.
  - A3: puede vigilar: forwarder.watch(f) y al caer f recibe Forwarded(Terminated(f)(true,false), f); isWatching permanece true hasta unwatch — FunctionRefSpec.scala:75-87.
  - A4: termina cuando su padre termina — FunctionRefSpec.scala:89-93.
  - A5: creado por top-level y por no top-level — FunctionRefSpec.scala:98-106.
  - A6: no registrado → no se encuentra — FunctionRefSpec.scala:108-120.
- **Oráculo**: jshell contra Pekko 1.7.1: FunctionRef es private[pekko]; se puede usar via ActorCell.addFunctionRef desde un jshell en paquete org.apache.pekko (jars oficiales).
- **Decisiones**: -
- **Notas**: Usado por typed (adaptadores) y streams. addFunctionRef nombra "$$<n>[-name]" y se registra en un mapa aparte (no en children).

### AC-REF-022: IgnoreActorRef

- **Estado de análisis**: extracted. Visibilidad: internal-required.
- **Contrato observable**: Descarta todo mensaje (hereda `!` = ()), sendSystemMessage no hace nada (no genera DeathWatchNotification); su path es fijo.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:529-562,567-570 ; actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:419
- **Condiciones** (normales, límites, errores, efectos): Watch a ignoreRef nunca se confirma (no hay Terminated). Errores: ninguno.
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: AC-PATH-015
- **Equivalente Rust propuesto**: variante `RefInner::Ignore`
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: ignoreRef ! m no publica DeadLetter — derivada de ActorRef.scala:515 (no afirmada por spec).
  - A2: path == RootActorPath(Address("pekko","local")) / "ignore" — derivada de ActorRef.scala:543-546 (no afirmada por spec).
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales): system.provider.ignoreRef.path.
- **Decisiones**: -
- **Notas**: Sin spec en actor-tests. Path: "pekko://local/ignore" (Address("pekko","local") / "ignore"); isIgnoreRefPath(String|ActorPath). Serializa a un SerializedIgnore singleton.

### AC-REF-023: Serialización Java de ActorRef (SerializedActorRef, writeReplace/readResolve)

- **Estado de análisis**: extracted. Visibilidad: internal-required.
- **Contrato observable**: Se serializa como la ruta de serialización (Serialization.serializedActorPath); deserializar requiere un sistema en JavaSerializer.currentSystem; si no existe localmente el ref → EmptyLocalActorRef.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:471-496 ; actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:521
- **Condiciones** (normales, límites, errores, efectos): Sin sistema en scope → IllegalStateException("Trying to deserialize a serialized ActorRef without an ActorSystem in scope. Use 'org.apache.pekko.serialization.JavaSerializer.currentSystem.withValue(system) { ... }'") (ActorRef.scala:481-483). Con sistema: la misma instancia (eq) para un actor local vivo.
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: AC-SER-007
- **Equivalente Rust propuesto**: `serialized_actor_path(&ActorRef) -> String` + `provider.resolve_actor_ref`
- **Diferencias observables**: sin Java serialization (Q-REF-3).
- **Aserciones**:
  - A1: round-trip local: readA eq a — ActorRefSpec.scala:288-317.
  - A2: sin sistema → IllegalStateException con el mensaje exacto — ActorRefSpec.scala:319-339.
  - A3: ref de actor parado → EmptyLocalActorRef — ActorRefSpec.scala:341-365.
- **Oráculo**: no aplica: serialización Java de la JVM.
- **Decisiones**: Q-REF-3
- **Notas**: Q-REF-3: excluir Java serialization (propuesta); se conserva el contrato path→ref.

### AC-REF-024: noSender (null) y sender implícito

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: El sender implícito en un actor es self; sin contexto, noSender (null) → el receptor ve deadLetters.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/Actor.scala:416 ; actor/src/main/scala/org/apache/pekko/actor/Actor.scala:481-533 ; actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:157,215
- **Condiciones** (normales, límites, errores, efectos): n/a en Rust: sin implícitos.
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: AC-REF-002
- **Equivalente Rust propuesto**: `Option<ActorRef>` explícito
- **Diferencias observables**: Rust expone `ctx.tell(to, msg)` que usa self como sender y `tell(msg, None)` para noSender.
- **Aserciones**:
  - A1: tell sin sender → sender()==deadLetters — ActorSelectionSpec.scala:281-285.
  - A2: reply via sender — ActorRefSpec.scala:390-413.
- **Oráculo**: no aplica.
- **Decisiones**: Q-REF-1
- **Notas**: Q-REF-1. Actor.noSender = null; dentro de un actor el sender implícito es self (implicit val self); `!` tiene `sender = Actor.noSender` por defecto (ActorRef.scala:157), pero DeadLetterActorRef usa `this`.

### AC-REF-025: ClassicActorSystemProvider / ClassicActorContextProvider

- **Estado de análisis**: listed. Visibilidad: public.
- **Contrato observable**: classicSystem devuelve el ActorSystem classic; classicActorContext devuelve el ActorContext classic (private[pekko]).
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/ClassicActorSystemProvider.scala:20-45
- **Condiciones** (normales, límites, errores, efectos): pendiente.
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: AC-SYS-012
- **Equivalente Rust propuesto**: traits `AsClassicSystem`, `AsClassicContext`
- **Diferencias observables**: pendiente de Q-X-8.
- **Aserciones**:
  - A1: ActorCell.classicActorContext = this — derivada de ActorCell.scala:446 (no afirmada por spec).
- **Oráculo**: no aplica.
- **Decisiones**: Q-X-8
- **Notas**: Sigue listed: sólo leí los trait (classicSystem; classicActorContext private[pekko]); falta ver el uso desde typed. Necesario para pekko-actor-typed (ver Q-X-8).

### AC-REF-026: ActorSelection: equals, hashCode, toString, toSerializationFormat, anchorPath, pathString

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: Igualdad por ancla (ref con uid) y path de elementos; toString "ActorSelection[Anchor(<path>[#uid]), Path(/<elems>)]"; toSerializationFormat concatena la ruta del ancla (con address por defecto si es ActorRefWithCell) y los elementos.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/ActorSelection.scala:143-194
- **Condiciones** (normales, límites, errores, efectos): serialización: ancla terminada en "/" no duplica la barra; path vacío sólo imprime el ancla.
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: AC-REF-011
- **Equivalente Rust propuesto**: `impl Eq/Hash/Display for ActorSelection`; `to_serialization_format()`
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: igualdad y hash coinciden y difieren si cambia ancla o path — ActorSelectionSpec.scala:337-344.
  - A2: toString exacto "ActorSelection[Anchor(pekko://ActorSelectionSpec/user/c2/c21#<uid>), Path(/../*/hello)]" — ActorSelectionSpec.scala:346-349.
  - A3: toSerializationFormat: "pekko://ActorSelectionSpec/user/c2", ".../c2/c21", "ActorSelection(c2,"/")" → ".../c2", "../*/hello" → ".../c2/../*/hello" — ActorSelectionSpec.scala:351-359.
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales): imprimir toString/toSerializationFormat.
- **Decisiones**: -
- **Notas**: Omisión agregada: no estaba en el inventario. Evidencia: ActorSelectionSpec.scala:337-359.

### AC-REF-027: VirtualPathContainer y /temp (refs virtuales de nivel raíz)

- **Estado de análisis**: listed. Visibilidad: internal-required.
- **Contrato observable**: /temp es un contenedor virtual donde se registran los PromiseActorRef de ask; sólo soporta SelectChildName en ActorSelectionMessage.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:763-860 ; actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:539-575
- **Condiciones** (normales, límites, errores, efectos): pendiente de lectura completa.
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: AC-ASK-001 ; AC-SYS-018
- **Equivalente Rust propuesto**: contenedor de hijos virtual (mapa concurrente nombre→ref)
- **Diferencias observables**: pendiente.
- **Aserciones**:
  - A1: rootGuardian.getSingleChild("temp") = tempContainer; "deadLetters" = deadLetters — derivada de ActorRefProvider.scala:547-552 (no afirmada por spec).
- **Oráculo**: no aplica.
- **Decisiones**: -
- **Notas**: Omisión agregada: no estaba. Sigue listed: leí la firma y la raíz (getSingleChild "temp"/"deadLetters"/extraNames, registerTempActor/unregisterTempActor con assert) pero no el cuerpo de VirtualPathContainer (ActorRef.scala:763-860). Usado por ask (AC-ASK).

### AC-CELL-001: Trait Actor: receive (PartialFunction), self, context; Receive; emptyBehavior; ignoringBehavior

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: receive devuelve la conducta inicial (PartialFunction[Any, Unit]); emptyBehavior no matchea ningún mensaje (todo unhandled); ignoringBehavior consume todo; self es un val final del Actor; context es un val implícito.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/Actor.scala:384-416,481-533,545-556
- **Condiciones** (normales, límites, errores, efectos): Normal: receive se evalúa al instanciar el actor (la conducta es la cabeza del behaviorStack). Borde: become en el constructor reemplaza receive antes del primer mensaje. Error: instanciar sin contexto → ActorInitializationException (AC-REF-006).
- **Orden / concurrencia / ciclo de vida**: aroundReceive aplica receive.applyOrElse(msg, notHandledFun) y si no estaba definido llama unhandled(msg) (Actor.scala:545-556).
- **Dependencias**: -
- **Equivalente Rust propuesto**: `trait Actor: Send + 'static { fn receive(&mut self, ctx: &mut Context, msg: AnyMsg) -> Handled }` con `Handled::{Handled, Unhandled}`
- **Diferencias observables**: PartialFunction → función que devuelve Handled/Unhandled (Q-CELL-1).
- **Aserciones**:
  - A1: become múltiples veces en el constructor: gana la última (respuesta "4:pigdog") — HotSwapSpec.scala:35-42.
  - A2: become en el constructor reemplaza receive ("pigdog") — HotSwapSpec.scala:26-33.
  - A3: emptyBehavior + PoisonPill: el actor termina (postStop lanza y se loguea) — ActorLifeCycleSpec.scala:132-140.
  - A4: emptyBehavior.isDefinedAt = false; ignoringBehavior.isDefinedAt = true — derivada de Actor.scala:399,408 (no afirmada por spec).
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales): actor con emptyBehavior → UnhandledMessage.
- **Decisiones**: Q-CELL-1, Q-X-3
- **Notas**: Q-CELL-1 (forma del receive). Detalle: `emptyBehavior.apply` lanza UnsupportedOperationException("Empty behavior apply()") (Actor.scala:400); en Pekko `receive` se evalúa UNA vez al construir la celda (behaviorStack inicial = instance.receive).

### AC-CELL-002.1: Creación: Create como primer mensaje de sistema, instanciación y preStart

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: init crea el mailbox, encola Create(None) en la cola de sistema ANTES de cualquier mensaje y, si sendSupervise, envía Supervise(self, async=false) al padre; start() hace dispatcher.attach. Al procesar Create: newActor() (constructor) → aroundPreStart() → checkReceiveTimeout(reschedule=true) → Debug "started (<actor>)" si debug.lifecycle.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/Dispatch.scala:79-116 ; actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:622-683
- **Condiciones** (normales, límites, errores, efectos): Normal: preStart corre una vez por incarnation. Instancia null → ActorInitializationException(self, "Actor instance passed to actorOf can't be 'null'") (ActorCell.scala:628-629). Con un fallo, ver AC-CELL-002.2.
- **Orden / concurrencia / ciclo de vida**: Create se encola antes de cualquier mensaje de usuario porque el actor no está adjunto al dispatcher hasta start(); el hijo creado con context.actorOf tiene Supervise enviado al padre en init (síncrono para el padre), mientras que system.actorOf pasa por Supervise(async=true) al guardián (AC-REF-020).
- **Dependencias**: AC-SUP-004
- **Equivalente Rust propuesto**: fábrica de Props ejecutada en la tarea del actor durante Create; `pre_start` tras construir
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: preStart se reporta como ("preStart", id, 0) al crear — ActorLifeCycleSpec.scala:61.
  - A2: hijos creados con nombre en el constructor del padre existen tras el arranque; al reiniciar el padre se crean de nuevo sin colisión (preStart se informa dos veces) — SupervisorMiscSpec.scala:87-100.
  - A3: Create se procesa antes que cualquier mensaje de usuario — derivada de Dispatch.scala:105-107 (no afirmada por spec).
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales): orden preStart vs primer mensaje.
- **Decisiones**: -
- **Notas**: Hijo de AC-CELL-002. Detalle nuevo: con un mailbox que exige un tipo de cola (RequiresMessageQueue) y no coincide, Create lleva Some(ActorInitializationException("Actor [<ref>] requires mailbox type [<req>] got [<got>]")) (Dispatch.scala:84-100): ver AC-CELL-026.

### AC-CELL-002.2: Creación: fallos del constructor o de preStart → ActorInitializationException

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: Si el constructor o preStart lanzan, la celda hace setFailed(actor.self) (si hay instancia), borra la instancia y lanza ActorInitializationException(self, "exception during creation, root cause message: [<mensaje de la causa raíz o 'No message available'>]", causa), que se maneja con handleInvokeFailure → Failed al padre.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:642-683,686-691 ; actor/src/main/scala/org/apache/pekko/actor/Actor.scala:198-211
- **Condiciones** (normales, límites, errores, efectos): InstantiationException: mensaje "exception during creation, this problem is likely to occur because the class of the Actor you tried to create is either, a non-static inner class (in which case make it a static inner class or use Props(new ...) or Props( new Creator ... ) or is missing an appropriate, reachable no-args constructor." (ActorCell.scala:667-674; sin sentido en Rust). InterruptedException: setFailedFatally y "interruption during creation". El mensaje de la excepción lleva el path del actor como prefijo: "<path>: <mensaje>" (Actor.scala:198-211; ActorRefSpec.scala:277-286 con pattern "/user/failingActor:").
- **Orden / concurrencia / ciclo de vida**: tras el fallo el actor queda suspendido (mailbox) hasta que el padre decida: Stop (default), Restart → faultRecreate con actor==null → Error "changing Recreate into Create after <causa>" + faultCreate, Resume → "changing Resume into Create after <causa>" (AC-SUP-011).
- **Dependencias**: AC-SUP-004 ; AC-SUP-011
- **Equivalente Rust propuesto**: panic/Err en la fábrica o en pre_start → `ActorInitializationError { actor, message, cause }`
- **Diferencias observables**: root cause: en Rust usar source() hasta el último; sin InstantiationException.
- **Aserciones**:
  - A1: constructor que falla en top-level cuyo mailbox no puede crearse: se loguea 1 ActorInitializationException y el ref termina — SupervisorSpec.scala:510-514.
  - A2: hijo con preStart que lanza → Restart del supervisor: reintenta hasta maxNrOfRetries=3 (preStart 0..3 observados) y luego no más — ActorLifeCycleSpec.scala:165-190.
  - A3: el mensaje de ActorInitializationException incluye el path del actor — ActorRefSpec.scala:277-286.
  - A4: ActorInitializationException custom Restart: segundo intento (cnt=1) funciona y responde PongMessage — SupervisorSpec.scala:546-569.
  - A5: error de configuración de mailbox en hijo → el padre ve ConfigurationException cuya causa es la original — SupervisorSpec.scala:516-525.
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales): actor que lanza en constructor y preStart; capturar el mensaje de la excepción en un decider.
- **Decisiones**: Q-X-5
- **Notas**: Hijo de AC-CELL-002. Contradicción corregida: SupervisorSpec.scala:546 prueba que una estrategia CUSTOM que responde Restart a ActorInitializationException reinicia el hijo (Child(cnt==0) falla una vez); NO prueba "el decider por defecto lo detiene" como decía el inventario. La parada por defecto se afirma en SupervisorSpec.scala:510-514 (top-level cuyo mailbox falla al crearse: expectTerminated).

### AC-CELL-003: Procesamiento de mensajes uno a la vez; AutoReceivedMessage antes del receive de usuario; aroundReceive

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: Nunca dos mensajes del mismo actor se procesan concurrentemente; Terminated, PoisonPill, Kill, Identify, ActorSelectionMessage y AddressTerminated son AutoReceivedMessage y los trata la celda (no el receive del usuario salvo Terminated).
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:549-590 ; actor/src/main/scala/org/apache/pekko/actor/Actor.scala:545-556
- **Condiciones** (normales, límites, errores, efectos): invoke: cancelReceiveTimeoutIfNeeded(msg) → currentMessage = envelope → autoReceiveMessage o receiveMessage (aroundReceive) → currentMessage = null (sólo si no falló) → checkReceiveTimeoutIfNeeded en finally. Excepción no fatal o Interrupted → handleInvokeFailure(Nil, e). Debug "received AutoReceiveMessage <envelope>" si debug.autoreceive (ActorCell.scala:569-570).
- **Orden / concurrencia / ciclo de vida**: Memoria: el mailbox da la sincronización entre mensajes consecutivos. Los mensajes de sistema se tratan antes y entre mensajes de usuario (AC-CELL-013.1).
- **Dependencias**: AC-MBX-002
- **Equivalente Rust propuesto**: tarea por actor que drena el mailbox; `&mut self` garantiza exclusión
- **Diferencias observables**: ninguna; la visibilidad la da la cola en Rust.
- **Aserciones**:
  - A1: un solo mensaje a la vez: si hubiera violación habría reinicio en vez de cuenta atrás — ActorModelSpec.scala:311-329.
  - A2: cada actor ve los pasos 0..9999 en orden consecutivo sin perder atomicidad (left+right==42) — ConsistencySpec.scala:41-60,69-81.
  - A3: Identify es manejado por la celda y no reinicia el receive timeout — ReceiveTimeoutSpec.scala:151-171.
  - A4: AutoReceived en orden: PoisonPill se loguea con "received AutoReceiveMessage Envelope(PoisonPill" — LoggingReceiveSpec.scala:186-203.
- **Oráculo**: no aplica: concurrencia; tests de propiedad en Rust.
- **Decisiones**: Q-X-4
- **Notas**: Contradicción corregida: el inventario citaba ActorModelSpec.scala:331 como afirmación de exclusión, pero :331-344 prueba encolado multihilo (200 mensajes procesados); la exclusión mutua se afirma en :311-329 y ConsistencySpec.scala:70-80 (visibilidad y orden de pasos).

### AC-CELL-004: unhandled(): Terminated → DeathPactException; resto → UnhandledMessage en EventStream

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: Mensaje no manejado: Terminated(dead) → lanza DeathPactException(dead); cualquier otro → publica UnhandledMessage(msg, sender(), self) en el EventStream del sistema.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/Actor.scala:292-302,545-556,650-662
- **Condiciones** (normales, límites, errores, efectos): DeathPactException(dead) mensaje exacto "Monitored actor [<dead>] terminated"; es NoStackTrace (Actor.scala:282-285, ver AC-SUP-017). Usa sender() actual.
- **Orden / concurrencia / ciclo de vida**: ocurre dentro del procesamiento del mensaje, en la propia tarea del actor.
- **Dependencias**: AC-EVT-010 ; AC-DW-007
- **Equivalente Rust propuesto**: resultado `Unhandled` → la celda llama `actor.unhandled(msg)`
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: mensajes enviados a los guardianes se publican como UnhandledMessage (log Warning "unhandled message from Actor[<deadLetters path>]: <msg>") — LocalActorRefProviderSpec.scala:70-107.
  - A2: un watcher que no maneja Terminated falla con DeathPactException(failed) y su supervisor lo recibe vía decider — DeathWatchSpec.scala:216-245.
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales): suscribirse a UnhandledMessage.
- **Decisiones**: -
- **Notas**: Contradicción corregida: EventStreamSpec.scala:115 (citada en el inventario) prueba el reenviador de debug.unhandled, no unhandled() (AC-CELL-025). La publicación de UnhandledMessage se observa en LocalActorRefProviderSpec.scala:77.

### AC-CELL-005: become / unbecome (pila de conductas)

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: become(b) con discardOld=true reemplaza el tope; con false apila; unbecome desapila y, con pila de 0 o 1 elementos, deja `actor.receive` como única conducta; la pila se limpia en restart (clearActorFields pone emptyBehaviorStack).
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:106-137,602-615,622-640 ; docs/src/main/paradox/actors.md:1055
- **Condiciones** (normales, límites, errores, efectos): En el constructor: la pila empieza vacía (newActor); si el constructor hizo become, el receive NO se añade (ActorCell.scala:632); si no, se añade receive. become con pila vacía y discardOld=true deja [b].
- **Orden / concurrencia / ciclo de vida**: la pila sólo se toca en el hilo del actor.
- **Dependencias**: AC-CELL-001
- **Equivalente Rust propuesto**: `Vec<Box<dyn Behavior>>` en la celda
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: become reemplaza: tras "swap", "swapped" se responde con la nueva conducta — HotSwapSpec.scala:70-83.
  - A2: unbecome revierte a receive — HotSwapSpec.scala:85-107.
  - A3: become con apilado en constructor y unbecome en cada paso: respuestas "4:pigdog","3:pigdog","2:pigdog","1:pigdog" — HotSwapSpec.scala:55-68.
  - A4: la última conducta apilada revierte a receive: "pigdog:pigdog" luego "badass:badass" — HotSwapSpec.scala:44-53.
  - A5: restart vuelve a la conducta inicial — HotSwapSpec.scala:109-132.
  - A6: tras Become(discardOld=false) y fallo, el actor responde 42 (conducta original) — ActorLifeCycleSpec.scala:142-163.
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales): mismos escenarios.
- **Decisiones**: Q-CELL-1

### AC-CELL-006: Hooks de ciclo de vida: preStart, postStop, preRestart (default: unwatch+stop hijos y postStop), postRestart (default: preStart)

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: preStart y postStop vacíos; preRestart(reason, message) por defecto desvigila y detiene a todos los hijos y llama postStop(); postRestart(reason) por defecto llama preStart(). preRestart recibe la causa y el mensaje opcional; postRestart la causa.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/Actor.scala:596-648 ; docs/src/main/paradox/actors.md:455-461,466-497
- **Condiciones** (normales, límites, errores, efectos): Los hooks pueden lanzar: ver AC-CELL-007.4 (preRestart/postRestart) y AC-CELL-009 (postStop). preStart se llama exactamente una vez por incarnation (restart incluido por el default de postRestart).
- **Orden / concurrencia / ciclo de vida**: Con defaults el orden observable es: postStop(old) → preStart(new) (ActorLifeCycleSpec.scala:93-96).
- **Dependencias**: AC-CELL-007.1
- **Equivalente Rust propuesto**: métodos por defecto del trait Actor
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: con hooks custom: preRestart(id,0), postRestart(id,1), generación incrementa; tras maxRetries=3 sólo postStop — ActorLifeCycleSpec.scala:49-82.
  - A2: defaults: postStop(old) y luego preStart(new) por cada restart — ActorLifeCycleSpec.scala:84-114.
  - A3: sin restart no se invocan preRestart ni postRestart; stop → postStop — ActorLifeCycleSpec.scala:116-130.
  - A4: con la misma instancia devuelta por la fábrica: preRestartN/postRestartN y postStop1 al agotar — SupervisorSpec.scala:240-290.
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales): secuencia de hooks en un restart.
- **Decisiones**: -
- **Notas**: Detalle: preRestart default NO llama postStop si el usuario lo sobrescribe; el default ejecuta context.children.foreach { unwatch; stop } y luego postStop() (Actor.scala:625-632). actors.md:466-497 describe la secuencia.

### AC-CELL-007.1: Restart 1/4: Recreate, suspensión y preRestart de la instancia vieja

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: faultRecreate(cause): si actor==null → Error "changing Recreate into Create after <cause>" y faultCreate; si el contenedor de hijos no es normal (isNormal=false: Recreation/Creation/Termination) → faultResume(null) para equilibrar el contador de suspensión; si no → Debug "restarting"; optionalMessage = currentMessage.message o None; si !isFailedFatally → aroundPreRestart(cause, optionalMessage) sobre la instancia vieja; en finally clearActorFields(recreate=true) (currentMessage=null, behaviorStack vacío). El mailbox debe estar suspendido (assert).
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala:49-87,89-118 ; actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:526 ; actor/src/main/scala/org/apache/pekko/dispatch/sysmsg/SystemMessage.scala:231 ; docs/src/main/paradox/actors.md:466-497
- **Condiciones** (normales, límites, errores, efectos): Si preRestart lanza una excepción no fatal: se publica Error(PreRestartException(self, e, cause, optionalMessage), …, e.getMessage) y el restart continúa. PreRestartException.getMessage = "<path>: exception in preRestart(<clase de originalCause o null>, <clase del mensaje o None>)" (Actor.scala:225-241).
- **Orden / concurrencia / ciclo de vida**: durante el restart el mailbox está suspendido: los mensajes de usuario permanecen encolados, no se pierden ni se reprocesan; el mensaje que falló se pasa a preRestart y se descarta (AC-CELL-020). Recreate es StashWhenWaitingForChildren (SystemMessage.scala:231): si ya se espera a hijos, se difiere (AC-CELL-013.2).
- **Dependencias**: AC-SUP-001 ; AC-CELL-006
- **Equivalente Rust propuesto**: estado de celda `Restarting{cause}`; `fault_recreate(cause)`
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: preRestart(id,0) llega antes de postRestart(id,1) — ActorLifeCycleSpec.scala:63-64.
  - A2: preRestart puede responder a sender() (el sender del mensaje fallido) — Ticket669Spec.scala:36-47 (expectMsg "failure1").
  - A3: un preRestart que lanza produce un Error PreRestartException y el actor igual se reinicia (inits==3) — SupervisorSpec.scala:419-458.
  - A4: ante Recreate sin instancia → Error "changing Recreate into Create after" — SupervisorHierarchySpec.scala:861-912 (EventFilter.error(start="changing Recreate into Create")).
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales): preRestart lanzando excepción y verificar Error en EventStream.
- **Decisiones**: Q-X-5
- **Notas**: Hijo de AC-CELL-007. Secuencia exacta: (1) el padre aplica Restart → `child.restart(cause)` encola Recreate(cause) en la cola de sistema del hijo (Dispatch.scala:160-162); (2) systemInvoke(Recreate) → faultRecreate.

### AC-CELL-007.2: Restart 2/4: espera a la parada de los hijos (Recreation)

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: Tras preRestart, si hay hijos pendientes de parada (por context.stop en preRestart, p. ej. el default), setChildrenTerminationReason(Recreation(cause)) devuelve true y la celda ESPERA: no crea la instancia hasta procesar la DeathWatchNotification del último hijo en toDie; entonces handleChildTerminated devuelve Some(Recreation(cause)) y se ejecuta finishRecreate(cause). Si no hay hijos por parar, finishRecreate corre de inmediato.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala:114,327-351 ; actor/src/main/scala/org/apache/pekko/actor/dungeon/Children.scala:183-190,246-266 ; actor/src/main/scala/org/apache/pekko/actor/dungeon/ChildrenContainer.scala:25-226
- **Condiciones** (normales, límites, errores, efectos): Mientras espera, el contenedor sigue aceptando hijos nuevos (reserve sólo falla en Termination); Recreate/Suspend/Resume que llegan se difieren (AC-CELL-013.2).
- **Orden / concurrencia / ciclo de vida**: la espera se resuelve por mensajes de sistema DeathWatchNotification (no por tiempo).
- **Dependencias**: AC-CELL-015.2 ; AC-DW-008
- **Equivalente Rust propuesto**: contenedor de hijos `Terminating{ reason: Recreation(cause) }`
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: constructor que crea hijo con nombre fijo ("bob") no choca tras reiniciar: preStart se informa 2 veces y el actor sigue vivo — SupervisorMiscSpec.scala:87-100 (default preRestart detiene los hijos y se espera su parada).
  - A2: preRestart sobrescrito que NO detiene hijos: hijos sobreviven (ver AC-CELL-007.3) — SupervisorTreeSpec.scala:27-45.
  - A3: espera a hijos en creación: "failChild" con hijos "workingChild" intenta 6 veces sin choque de nombres — SupervisorHierarchySpec.scala:861-912.
- **Oráculo**: no aplica (timing de hilos); verificable por orden de eventos con jshell (jars oficiales).
- **Decisiones**: -
- **Notas**: Hijo de AC-CELL-007.

### AC-CELL-007.3: Restart 3/4: nueva instancia, postRestart y reinicio de hijos sobrevivientes

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: finishRecreate: snapshot de children (survivors); resumeNonRecursive (en finally clearFailed); freshActor = newActor() (nueva incarnation con constructor); aroundPostRestart(cause) (default preStart); checkReceiveTimeout(reschedule=true); Debug "restarted"; sólo ENTONCES cada sobreviviente recibe restart(cause) (en orden de iteración del contenedor = nombre).
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala:278-303 ; actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:622-640
- **Condiciones** (normales, límites, errores, efectos): Si restart de un hijo lanza: Error "restarting <child>" y sigue con los demás. El uid del actor NO cambia en el restart (misma incarnation de ref).
- **Orden / concurrencia / ciclo de vida**: orden: [padre: preRestart(old)] → [hijos stop si default] → constructor(new) → postRestart(new) → [hijos sobrevivientes: Recreate → su preRestart → su constructor → su postRestart]. Mensajes de usuario encolados se procesan DESPUÉS de postRestart (docs actors.md:487-491).
- **Dependencias**: AC-SUP-015 ; AC-CELL-017
- **Equivalente Rust propuesto**: `finish_recreate(cause)`
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: restart de middle: llega primero preRestart de middle y luego el de last — SupervisorTreeSpec.scala:36-41.
  - A2: tras postRestart el actor responde "status" en la nueva generación — ActorLifeCycleSpec.scala:65-66.
  - A3: ReceiveTimeout fijado en preStart (invocado desde postRestart) se agenda tras restart — ReceiveTimeoutSpec.scala:300-311.
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales): árbol de 3 niveles, matar el del medio, orden de hooks.
- **Decisiones**: -
- **Notas**: Hijo de AC-CELL-007.

### AC-CELL-007.4: Restart 4/4: fallo en postRestart o constructor → PostRestartException

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: Si newActor() o aroundPostRestart() o el reinicio de hijos lanzan: setFailedFatally(); clearActorFields(actor, recreate=false); handleInvokeFailure(survivors, PostRestartException(self, e, cause)) — se suspende, se notifica Failed(self, PostRestartException) al padre sin suspender a los sobrevivientes.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala:278-303 ; actor/src/main/scala/org/apache/pekko/actor/Actor.scala:247-267
- **Condiciones** (normales, límites, errores, efectos): PostRestartException mensaje: "<path>: exception post restart (<clase de originalCause o null>)". OriginalRestartException.unapply recorre cadenas de PostRestartException hasta la causa original (Actor.scala:260-267).
- **Orden / concurrencia / ciclo de vida**: el padre decide: default decider → Stop (ActorInitializationException); decider custom Exception → Restart → nuevo Recreate sin preRestart.
- **Dependencias**: AC-SUP-001 ; AC-SUP-019
- **Equivalente Rust propuesto**: `PostRestartError { actor, cause, original_cause }`
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: constructor que lanza en init%3==1 y preRestart que lanza en init%3==0: se loguean exactamente 1 RuntimeException, 1 PreRestartException y 1 PostRestartException, el actor termina vivo y responde Ping; inits==3 — SupervisorSpec.scala:419-458.
  - A2: Recreate tras fallo fatal no invoca preRestart — derivada de FaultHandling.scala:104 (no afirmada por spec).
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales): mismo escenario con contadores.
- **Decisiones**: Q-X-5
- **Notas**: Hijo de AC-CELL-007. Caso límite: tras un fallo en postRestart, setFailedFatally: el siguiente Recreate NO vuelve a llamar preRestart (isFailedFatally) y Resume con causa se convierte en Restart (AC-SUP-019).

### AC-CELL-008.1: Stop 1/2: terminate() — cancelación, unwatch y parada de hijos

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: terminate(): cancelReceiveTimeout() (= setReceiveTimeout(Duration.Undefined), AbstractActor.scala:200) y cancelReceiveTimeoutTask(); unwatchWatchedActors (para evitar DeadLetter(Terminated)); detiene a todos los hijos; si systemImpl.aborting reenvía DWN(existenceConfirmed=true) por los hijos no locales; si hay hijos pendientes: suspende el mailbox (sin recursión), setFailed(self) ("no propagar fallos durante shutdown") y Debug "stopping" (sólo la 1ª vez); si no hay hijos: setTerminated() + finishTerminate().
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala:180-212 ; docs/src/main/paradox/actors.md:440-444
- **Condiciones** (normales, límites, errores, efectos): Mensajes de usuario en el mailbox no se procesan mientras espera hijos (suspendido); mensajes de sistema Failed/Suspend/Resume/Recreate se difieren según estado (AC-CELL-013.2). Crear hijos durante Termination lanza IllegalStateException("cannot create children while terminating or terminated") (Children.scala:314) o "cannot reserve actor name '<n>': terminating".
- **Orden / concurrencia / ciclo de vida**: orden de eventos de stop con hijos: Terminate(a) → Unwatch a watched → Terminate a hijos → (esperar DWN de hijos) → finishTerminate (AC-CELL-008.2). PoisonPill procesada en orden de mensaje (AC-CELL-011).
- **Dependencias**: AC-DW-009 ; AC-CELL-015.2
- **Equivalente Rust propuesto**: estado `Terminating` + conjunto de hijos pendientes
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: PoisonPill: ask previos se responden (five/null) antes de parar; luego Terminated — ActorRefSpec.scala:422-440.
  - A2: stop sin restart previo: postStop(id,0) y nada más — ActorLifeCycleSpec.scala:116-130.
  - A3: Terminate durante espera de hijos no repite "stopping" — derivada de FaultHandling.scala:198-207 (wasTerminating) (no afirmada por spec).
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales): árbol a→b→c y orden de postStop.
- **Decisiones**: -
- **Notas**: Hijo de AC-CELL-008. La parada se inicia por Terminate() (mensaje de sistema; DeadLetterSuppression).

### AC-CELL-008.2: Stop 2/2: finishTerminate — postStop, detach y notificaciones

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: Orden estricto de finishTerminate (cada paso en try/finally, un fallo no impide los siguientes): (1) aroundPostStop (excepción no fatal → Error con e.getMessage); (2) stopFunctionRefs; (3) dispatcher.detach (mailbox → deadLetterMailbox, closed, cleanUp: pendientes a deadLetters); (4) parent.sendSystemMessage(DWN(self, existenceConfirmed=true)); (5) tellWatchersWeDied (watchers distintos del padre); (6) unwatchWatchedActors; (7) Debug "stopped"; (8) clearActorFields y clearFieldsForTermination (unstashAll, props = terminatedProps, actor = null).
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala:247-276 ; actor/src/main/scala/org/apache/pekko/dispatch/AbstractDispatcher.scala:240-246 ; actor/src/main/scala/org/apache/pekko/dispatch/Mailboxes.scala:54-72
- **Condiciones** (normales, límites, errores, efectos): tras terminar, `props` es terminatedProps, cuya fábrica lanza IllegalActorStateException("This Actor has been terminated") (ActorCell.scala:388); el contexto sigue siendo no nulo (no se anula) — ActorLifeCycleSpec.scala:193-230. Mensajes enviados después van a deadLetters (mailbox swap).
- **Orden / concurrencia / ciclo de vida**: DeathWatchNotification al padre ANTES que a los watchers: un padre que también vigila recibe primero la limpieza del hijo y luego el Terminated (AC-DW-008).
- **Dependencias**: AC-DW-010 ; AC-MBX-003
- **Equivalente Rust propuesto**: `finish_terminate()` con orden fijo
- **Diferencias observables**: sin diferencias.
- **Aserciones**:
  - A1: postStop que lanza: se loguea Error("hurrah") 1 vez al recibir PoisonPill — ActorLifeCycleSpec.scala:132-140.
  - A2: contexto no nulo tras terminar y respuesta "pong" tras context.stop(self) — ActorLifeCycleSpec.scala:193-230.
  - A3: tras terminar, el cell no retiene Props originales (terminatedProps) — LocalActorRefProviderSpec.scala:109-129.
  - A4: 3 watchers: exactamente un Terminated cada uno — DeathWatchSpec.scala:154-167.
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales): orden postStop → Terminated en watchers.
- **Decisiones**: -
- **Notas**: Hijo de AC-CELL-008. El comentario del código: "The following order is crucial… if a parent is also a watcher then ChildTerminated and Terminated must be processed in this specific order" (FaultHandling.scala:249-252).

### AC-CELL-009: Fallo en postStop se loguea y no impide la terminación

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: Una excepción no fatal en postStop se publica como Error(e, path, clase, e.getMessage) y el resto de finishTerminate continúa.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala:247-258
- **Condiciones** (normales, límites, errores, efectos): Excepciones fatales no se capturan (handleNonFatalOrInterruptedException). InterruptedException: re-marca la interrupción.
- **Orden / concurrencia / ciclo de vida**: dentro de finishTerminate (AC-CELL-008.2).
- **Dependencias**: AC-CELL-008.2
- **Equivalente Rust propuesto**: capturar panic/Err del hook y publicar Error
- **Diferencias observables**: panic fatal en Rust: ver Q-X-5.
- **Aserciones**:
  - A1: postStop lanza Exception("hurrah"): 1 log y el actor termina — ActorLifeCycleSpec.scala:132-140.
  - A2: postStop puede responder a sender() ("failure2") — Ticket669Spec.scala:49-59.
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales): postStop lanzando.
- **Decisiones**: Q-X-5

### AC-CELL-010: stop(ref): context.stop y system.stop (asíncrono)

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: stop es asíncrono; context.stop(hijo) marca shallDie y envía Terminate; context.stop(self) equivale a self.stop(); system.stop(ref): top-level bajo /user o /system pasa por StopChild al guardián, en otro caso llama ref.stop().
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/Children.scala:133-148 ; actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:936-949
- **Condiciones** (normales, límites, errores, efectos): stop de un ref no hijo → sólo InternalActorRef.stop(). stop de un ref ya muerto: no-op (mailbox cerrado: Terminate a deadLetters, DeadLetterSuppression).
- **Orden / concurrencia / ciclo de vida**: el nombre queda reservado hasta Terminated del hijo (AC-REF-009.2).
- **Dependencias**: AC-CELL-008.1
- **Equivalente Rust propuesto**: `ctx.stop(&ActorRef)`, `system.stop(&ActorRef)`
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: tras context.stop(kid) no se puede reusar el nombre inmediatamente (InvalidActorNameException) — SupervisorMiscSpec.scala:126-141.
  - A2: stop de FunctionRef (removeFunctionRef) → watcher recibe Terminated — FunctionRefSpec.scala:67-73.
  - A3: system.stop de top-level usa StopChild al guardián — derivada de ActorSystem.scala:939-945 (no afirmada por spec).
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales).
- **Decisiones**: -
- **Notas**: Contradicción corregida: ActorRefSpec.scala:465 (cita vieja) prueba child(name), no stop; stop se observa en :422-440 (PoisonPill) y SupervisorMiscSpec.scala:102-142. Detalle: ActorContext.stop(ref) para un hijo propio marca shallDie (contenedor → Terminating UserRequest) y luego envía Terminate; para un RepointableRef no iniciado no marca shallDie.

### AC-CELL-011: PoisonPill

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: Mensaje ordinario (entra por la cola de usuario, en orden) que al procesarse llama self.stop(); es AutoReceivedMessage, PossiblyHarmful y DeadLetterSuppression.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/Actor.scala:46-55 ; actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:576 ; docs/src/main/paradox/actors.md:977
- **Condiciones** (normales, límites, errores, efectos): Normal: los mensajes anteriores se procesan. Después de PoisonPill el actor sigue procesando hasta que Terminate llega por la cola de sistema (los mensajes de usuario ya dequeued en ese ciclo de throughput pueden procesarse; ver AC-CELL-013.1).
- **Orden / concurrencia / ciclo de vida**: Terminate se encola al sistema: efecto tras el mensaje actual y antes del siguiente mensaje de usuario (Mailbox.processMailbox llama processAllSystemMessages tras cada mensaje).
- **Dependencias**: AC-CELL-008.1
- **Equivalente Rust propuesto**: mensaje de control interceptado por la celda
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: ask previos respondidos antes de parar, luego Terminated — ActorRefSpec.scala:422-440.
  - A2: terminal ! PoisonPill → watchers reciben un Terminated — DeathWatchSpec.scala:135-143.
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales).
- **Decisiones**: -
- **Notas**: Docs actors.md:977 no releído.

### AC-CELL-012: Kill

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: Al procesarse lanza ActorKilledException("Kill") (NoStackTrace) → se maneja como fallo del actor; el decider por defecto detiene al actor.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/Actor.scala:57-70,177-180 ; actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:575
- **Condiciones** (normales, límites, errores, efectos): Mensaje de la excepción: "Kill" (ActorCell.scala:575, Actor.scala:177). Log Error: Logging.Error(ActorKilledException, path, _, "Kill").
- **Orden / concurrencia / ciclo de vida**: como cualquier fallo: suspende, notifica a su padre (AC-SUP-001).
- **Dependencias**: AC-SUP-004
- **Equivalente Rust propuesto**: ídem PoisonPill
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: Kill con decider Throwable → Restart: preRestart y postRestart llaman latch — ActorRefSpec.scala:442-463.
  - A2: Kill bajo defaultStrategy: el hijo se detiene (Failed con ActorKilledException, lastSender es el hijo) — DeathWatchSpec.scala:237-245.
  - A3: log Error con causa ActorKilledException y mensaje "Kill" — LoggingReceiveSpec.scala:281-284.
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales).
- **Decisiones**: -
- **Notas**: Contradicción corregida: ActorRefSpec.scala:442-463 usa un decider custom (List(classOf[Throwable]) → Restart) y comprueba el reinicio; la parada por defecto ante Kill se afirma en DeathWatchSpec.scala:237-245 (Failed(_, ActorKilledException) con lastSender == failed y luego el watcher recibe Terminated).

### AC-CELL-013.1: Prioridad de mensajes de sistema sobre los de usuario (bucle del mailbox)

- **Estado de análisis**: extracted. Visibilidad: internal-required.
- **Contrato observable**: Mailbox.run: si no está cerrado, processAllSystemMessages(); luego processMailbox(): por cada mensaje de usuario (hasta throughput y deadline): dequeue → invoke → processAllSystemMessages(). Nunca se ejecuta un mensaje de usuario con un mensaje de sistema pendiente. Un mailbox suspendido (suspendCount>0) NO procesa mensajes de usuario pero sí de sistema.
- **Fuente**: actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala:230-280,289-333 ; actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:481-546
- **Condiciones** (normales, límites, errores, efectos): Si el mailbox se cierra mientras se procesan sistema, los restantes se envían al deadLetterMailbox (DeathWatch lo exige). Una interrupción durante el procesamiento se relanza como InterruptedException al terminar el drenaje. Throughput: max(throughput,1) mensajes por turno.
- **Orden / concurrencia / ciclo de vida**: dentro de systemInvoke, `invokeAll` procesa en orden de envío (EarliestFirst), y tras cada mensaje recalcula el estado (calculateState) para desestacar (AC-CELL-013.2); si isTerminated el resto se manda a deadLetters.
- **Dependencias**: AC-MBX-002
- **Equivalente Rust propuesto**: dos colas por actor (sistema, usuario) y bucle: drenar sistema → un mensaje de usuario → drenar sistema
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: un mensaje de sistema que lanza (OHNOES en decider) no pierde mensajes de sistema: parent restarted, child terminated, y siguen vivos — SupervisorSpec.scala:460-506.
  - A2: un hijo suspendido por fallo no procesa mensajes hasta Resume (worker no responde 2s, luego "pong") — SupervisorHierarchySpec.scala:832-859.
  - A3: processAllSystemMessages se invoca antes del primer mensaje y entre mensajes — derivada de Mailbox.scala:233-234,276 (no afirmada por spec).
- **Oráculo**: no aplica: concurrencia; test determinista con dispatcher de un hilo (CallingThread) en Rust.
- **Decisiones**: Q-X-4
- **Notas**: Hijo de AC-CELL-013.

### AC-CELL-013.2: Diferimiento (stash) de mensajes de sistema según el estado de la celda

- **Estado de análisis**: extracted. Visibilidad: internal-required.
- **Contrato observable**: Estado = SuspendedWaitForChildren si el contenedor espera Recreation/Creation; Suspended si el mailbox está suspendido; Default si no. shouldStash(m, state): Default → nunca; Suspended → m es StashWhenFailed (Failed); SuspendedWaitForChildren → m es StashWhenWaitingForChildren. Diferidos se guardan en sysmsgStash (LatestFirst) y se reponen al "subir" de estado (newState < currentState) con rest.reversePrepend(unstashAll()).
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:452-545 ; actor/src/main/scala/org/apache/pekko/dispatch/sysmsg/SystemMessage.scala:215-278 ; actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:405-407
- **Condiciones** (normales, límites, errores, efectos): Terminate durante espera de hijos NO se difiere: pasa a Termination (AC-CELL-008.1). clearFieldsForTermination hace unstashAll() y descarta.
- **Orden / concurrencia / ciclo de vida**: reponer: los diferidos se ejecutan antes del resto pendiente y en orden de llegada original.
- **Dependencias**: AC-CELL-007.2 ; AC-CELL-008.1
- **Equivalente Rust propuesto**: `enum CellState { Default, Suspended, WaitingForChildren }` + cola de sistema diferida
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: Recreate/Resume mientras la celda aún crea hijos: se convierten en Create (6 intentos de creación, preStart 1 vez) — SupervisorHierarchySpec.scala:861-912.
  - A2: tabla de estados y conjuntos de mensajes diferidos — derivada de ActorCell.scala:492-511 y SystemMessage.scala:215-278 (no afirmada por spec).
- **Oráculo**: no aplica: estado interno.
- **Decisiones**: -
- **Notas**: Hijo de AC-CELL-013. Datos: StashWhenFailed = {Failed}; StashWhenWaitingForChildren = {Failed, Recreate, Suspend, Resume}. DeathWatchNotification, Create, Terminate, Supervise, Watch, Unwatch NUNCA se difieren.

### AC-CELL-013.3: Mensajes de sistema a una celda terminada → deadLetters

- **Estado de análisis**: extracted. Visibilidad: internal-required.
- **Contrato observable**: Tras detach cualquier mensaje de usuario va a deadLetters (DeadLetter(msg, sender, receiver), sin bucle para DeadLetter); cualquier mensaje de sistema se vuelve DeadLetter(sysmsg, receiver, receiver). cleanUp traslada pendientes (sistema y usuario) al mailbox muerto.
- **Fuente**: actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala:289-333,340-354 ; actor/src/main/scala/org/apache/pekko/dispatch/Mailboxes.scala:54-72 ; actor/src/main/scala/org/apache/pekko/dispatch/AbstractDispatcher.scala:240-246
- **Condiciones** (normales, límites, errores, efectos): onDeadLetterMailboxEnqueue ignora DeadLetter (actor suscrito a DeadLetter) para evitar bucles. Los Terminate a deadLetters son DeadLetterSuppression.
- **Orden / concurrencia / ciclo de vida**: ocurre en unregister (detach) en finishTerminate paso 3.
- **Dependencias**: AC-DW-002 ; AC-REF-016
- **Equivalente Rust propuesto**: mailbox cerrado sustituido por un mailbox "dead letter" que reenvía a deadLetters
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: watch sobre actor ya muerto produce Terminated (vía este mecanismo) — DeathWatchSpec.scala:145-152.
  - A2: sysmsg a mailbox muerto = DeadLetter(sysmsg, receiver, receiver) — derivada de Mailboxes.scala:62-63 (no afirmada por spec).
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales): suscribir a DeadLetter y enviar tras parar.
- **Decisiones**: -
- **Notas**: Hijo de AC-CELL-013. Hallazgo nuevo: detach cambia el mailbox por deadLetterMailbox; su systemEnqueue publica DeadLetter(sysmsg, receiver, receiver) a deadLetters, y DeadLetterActorRef.specialHandle convierte un Watch en DWN(existenceConfirmed=false) (ActorRef.scala:746-752).

### AC-CELL-014: Familia SystemMessage (Create, Recreate, Suspend, Resume, Terminate, Supervise, Watch, Unwatch, Failed, DeathWatchNotification, NoMessage) y listas intrusivas

- **Estado de análisis**: extracted. Visibilidad: internal-required.
- **Contrato observable**: Mensajes de sistema: Create(failure: Option[ActorInitializationException]), Recreate(cause), Suspend(), Resume(causedByFailure), Terminate(), Supervise(child, async), Watch(watchee, watcher), Unwatch(watchee, watcher), NoMessage, Failed(child, cause, uid), DeathWatchNotification(actor, existenceConfirmed, addressTerminated). Un mismo objeto de mensaje de sistema nunca se envía a dos actores. Terminate y DeathWatchNotification son DeadLetterSuppression.
- **Fuente**: actor/src/main/scala/org/apache/pekko/dispatch/sysmsg/SystemMessage.scala:29-285
- **Condiciones** (normales, límites, errores, efectos): NoMessage sólo marca terminación en la cola.
- **Orden / concurrencia / ciclo de vida**: LatestFirst/EarliestFirst: encolar = prepend; drenar invierte para procesar en orden de envío.
- **Dependencias**: -
- **Equivalente Rust propuesto**: `enum SystemMessage` + `VecDeque`
- **Diferencias observables**: sin listas intrusivas.
- **Aserciones**:
  - A1: lista vacía: head null, isEmpty, reverse == ENil — SystemMessageListSpec.scala:28-32.
  - A2: `::` antepone y enlaza next — SystemMessageListSpec.scala:34-45.
  - A3: head/tail y size — SystemMessageListSpec.scala:47-75.
- **Oráculo**: no aplica.
- **Decisiones**: -
- **Notas**: Listas intrusivas son optimización JVM; en Rust basta enum + cola. Sólo leí SystemMessageListSpec.scala:26-75 de 28..103: el resto (sublistas/reverse) queda por leer pero las operaciones son estructurales.

### AC-CELL-015.1: Contenedor de hijos: estados Normal/Terminating y reserva de nombres

- **Estado de análisis**: extracted. Visibilidad: internal-required.
- **Contrato observable**: Estados: Empty, Normal (TreeMap nombre→ChildStats), Terminating(c, toDie, reason ∈ {UserRequest, Recreation(cause), Creation, Termination}), Terminated. Un nombre queda reservado (ChildNameReserved) hasta initChild o unreserve; sólo se libera tras procesar la DWN del hijo.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/ChildrenContainer.scala:25-226 ; actor/src/main/scala/org/apache/pekko/actor/dungeon/Children.scala:161-190,246-266,312-345
- **Condiciones** (normales, límites, errores, efectos): reserve en Normal/Terminating(no Termination) con nombre existente → InvalidActorNameException("actor name [<n>] is not unique!"); Terminating(Termination) → IllegalStateException("cannot reserve actor name '<n>': terminating"); Terminated → IllegalStateException("cannot reserve actor name '<n>': already terminated"). remove: si el último de toDie se va y reason == Termination → Terminated, si no → Normal. isNormal sólo con UserRequest o Normal.
- **Orden / concurrencia / ciclo de vida**: transiciones por CAS (swapChildrenRefs).
- **Dependencias**: AC-REF-009.2
- **Equivalente Rust propuesto**: `enum ChildrenState` + `BTreeMap<String, ChildStats>`
- **Diferencias observables**: Rust puede usar un lock sobre el contenedor en vez de CAS.
- **Aserciones**:
  - A1: stop + actorOf mismo nombre → InvalidActorNameException — SupervisorMiscSpec.scala:126-141.
  - A2: tras Terminated(kid) el nombre se puede reusar y la nueva instancia es distinta — SupervisorMiscSpec.scala:102-124.
  - A3: en handleChildTerminated (estrategia) se puede crear hijo del mismo nombre — SupervisorMiscSpec.scala:144-159.
  - A4: padre que observa a su hijo con mismo nombre dos veces ("kid") — DeathWatchSpec.scala:248-255.
- **Oráculo**: no aplica: estado interno.
- **Decisiones**: -
- **Notas**: Hijo de AC-CELL-015. Contradicción corregida: "crear hijo mientras se termina → IllegalStateException" aplica sólo cuando reason == Termination (isTerminating): el mensaje es "cannot create children while terminating or terminated" (Children.scala:314); durante Recreation/Creation/UserRequest SÍ se pueden crear.

### AC-CELL-015.2: Contenedor de hijos: razones de espera y transición al terminar el último hijo

- **Estado de análisis**: extracted. Visibilidad: internal-required.
- **Contrato observable**: removeChildAndGetStateChange: si el contenedor es Terminating y tras remove ya no lo es, devuelve la razón (Some(reason)) y la celda continúa: Recreation → finishRecreate(cause), Creation → finishCreate, Termination → finishTerminate; UserRequest no hace nada.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/ChildrenContainer.scala:150-226 ; actor/src/main/scala/org/apache/pekko/actor/dungeon/Children.scala:183-190,246-266 ; actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala:327-351
- **Condiciones** (normales, límites, errores, efectos): si remove deja aún hijos en toDie, no hay cambio de estado (None).
- **Orden / concurrencia / ciclo de vida**: el resultado se calcula ANTES de quitar (la coincidencia se hace sobre el contenedor previo).
- **Dependencias**: AC-CELL-007.2 ; AC-CELL-008.1
- **Equivalente Rust propuesto**: `ChildrenState::Terminating { to_die, reason }`
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: creación de hijo fallida y reintentos con Resume/Restart (reason=Creation) terminan con 6 intentos — SupervisorHierarchySpec.scala:861-912.
  - A2: hijo creado dentro de handleChildTerminated mientras el padre se detiene produce IllegalStateException logueada "handleChildTerminated failed" — SupervisorMiscSpec.scala:160-162.
- **Oráculo**: no aplica.
- **Decisiones**: -
- **Notas**: Hijo de AC-CELL-015. handleChildTerminated llama a la estrategia (handleChildTerminated) aunque la celda esté reiniciando; su excepción se loguea ("handleChildTerminated failed") y se propaga como fallo (AC-SUP-020).

### AC-CELL-016: Arranque de un hijo: reserva, uid nuevo, hereda suspensiones del padre, Supervise al padre, start

- **Estado de análisis**: extracted. Visibilidad: internal-required.
- **Contrato observable**: makeChild: validar serializabilidad de args si serialize-creators (AC-CELL-029) → comprobar !terminating → reserveChild(name) → provider.actorOf(childPath con newUid, systemService, async) → heredar suspensiones → initChild → start. Debug "now supervising <child>" al procesar Supervise.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/Children.scala:279-345 ; actor/src/main/scala/org/apache/pekko/actor/dungeon/Dispatch.scala:79-116 ; actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:693-714
- **Condiciones** (normales, límites, errores, efectos): Supervise ignorado si la celda está en Termination (isTerminating). Para hijos RepointableActorRef con async, handleSupervise llama point(catchFailures=true).
- **Orden / concurrencia / ciclo de vida**: el hijo hereda "N" Suspend del padre para no procesar mensajes mientras el padre falla.
- **Dependencias**: AC-CELL-015.1 ; AC-REF-020
- **Equivalente Rust propuesto**: `make_child` con reserva atómica y suspensión heredada
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: hijos creados en el constructor del padre existen tras arranque y reinicio sin colisión — SupervisorMiscSpec.scala:87-100.
  - A2: decider recibe sender()==hijo que falló (path.parent == parent.path, name "child") — SupervisorMiscSpec.scala:165-180.
  - A3: log "now supervising TestActor[<path>]" al arrancar un hijo — LoggingReceiveSpec.scala:276 (msg == s"now supervising TestActor[$aname]").
- **Oráculo**: no aplica.
- **Decisiones**: -
- **Notas**: Datos nuevos: makeChild lanza IllegalStateException("cannot create children while terminating or terminated") si la celda termina; en un fallo de provider.actorOf libera la reserva y propaga; si mailbox no nulo hace actor.suspend() tantas veces como suspendCount del padre; luego initChild(actor) y actor.start(). Supervise es el primer mensaje de sistema que el padre recibe del hijo; si el hijo no está reservado: Error "received Supervise from unregistered child <c>, this will not end well" (ActorCell.scala:702-706).

### AC-CELL-017: ReceiveTimeout

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: setReceiveTimeout(d) programa el envío de ReceiveTimeout tras d sin mensajes; cada mensaje que no sea NotInfluenceReceiveTimeout cancela y reprograma el temporizador; Duration.Undefined/Inf lo apaga; se cancela al parar (terminate y faultCreate) y se reprograma tras restart.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/ReceiveTimeout.scala:14-84 ; actor/src/main/scala/org/apache/pekko/actor/Actor.scala:146-164 ; actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:76-104 ; actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala:162-163,181-182,289 ; actor/src/main/scala/org/apache/pekko/actor/AbstractActor.scala:200
- **Condiciones** (normales, límites, errores, efectos): ReceiveTimeout llega como mensaje normal (sender deadLetters, programado con system.scheduler.scheduleOnce(f, self, ReceiveTimeout)); puede llegar justo tras otro mensaje (no garantiza inactividad previa). Si el usuario cambia el timeout durante un mensaje NotInfluence se reprograma igualmente (receiveTimeoutChanged). Identify es NotInfluence.
- **Orden / concurrencia / ciclo de vida**: antes de procesar: cancelReceiveTimeoutIfNeeded(msg) sólo si hay datos y el mensaje influye; después (finally): checkReceiveTimeoutIfNeeded. En restart: finishRecreate llama checkReceiveTimeout(reschedule=true) tras postRestart.
- **Dependencias**: AC-SCH-001
- **Equivalente Rust propuesto**: timer por actor en la celda (tarea cancelable) con `ReceiveTimeoutDeadline`
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: se dispara tras 500ms — ReceiveTimeoutSpec.scala:82-95.
  - A2: se reprograma tras un mensaje normal — ReceiveTimeoutSpec.scala:97-113.
  - A3: desactivar con Duration.Undefined desde el handler → count==1 — ReceiveTimeoutSpec.scala:115-136.
  - A4: sin configurar no llega — ReceiveTimeoutSpec.scala:138-149.
  - A5: con NotInfluence (TransparentTick, Identify, Timers) sigue disparando — ReceiveTimeoutSpec.scala:151-213.
  - A6: activar/desactivar/cambiar el timeout dentro de un handler NotInfluence — ReceiveTimeoutSpec.scala:215-297.
  - A7: tras restart se reprograma inmediato — ReceiveTimeoutSpec.scala:300-311.
  - A8: al pararse se cancela: no hay DeadLetter de ReceiveTimeout — ReceiveTimeoutSpec.scala:313-320.
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales) para ordenes con tiempo; el resto con reloj simulado en Rust.
- **Decisiones**: Q-X-4
- **Notas**: Contradicción corregida: Duration.Inf también apaga el timeout (checkReceiveTimeout: sólo FiniteDuration programa; cualquier otro cancela) — ReceiveTimeoutSpec.scala:231-247. Muchos tests son TimingTest: portar con tiempo controlado. Mínimo soportado 1 ms (ActorCell.scala:88).

### AC-CELL-018: API de ActorContext: self, props, system, dispatcher, parent, getters Java

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: ActorContext expone self, props, receiveTimeout/setReceiveTimeout, become/unbecome, sender, children/child, dispatcher, system, parent, watch/watchWith/unwatch, actorOf/actorSelection/stop; la Java API ofrece getSelf/getProps/getParent/getSystem/getDispatcher/getChildren/getChild/findChild.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:58-250,435-464
- **Condiciones** (normales, límites, errores, efectos): self y props son thread-safe (ActorCell.scala:60-74); receiveTimeout/setReceiveTimeout no lo son (:76-104). dispatcher es un ExecutionContextExecutor del actor.
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: AC-DSP-016
- **Equivalente Rust propuesto**: `Context` con métodos equivalentes
- **Diferencias observables**: dispatcher como ExecutionContext no existe en Rust: se usa el runtime (Q-X-4).
- **Aserciones**:
  - A1: actorOf anidados devuelve refs distintos — ActorRefSpec.scala:367-377.
  - A2: el path del hijo tiene como padre el path del padre — SupervisorMiscSpec.scala:177-180.
- **Oráculo**: no aplica.
- **Decisiones**: Q-X-4, Q-X-8
- **Notas**: Sin spec que recorra toda la API; las aserciones son por uso. Q-X-8 para ActorContext typed vs classic.

### AC-CELL-019: El contexto sólo es válido dentro del procesamiento del actor

- **Estado de análisis**: listed. Visibilidad: public.
- **Contrato observable**: self y props pueden usarse desde otros hilos; el resto del contexto no es thread-safe; usarlo desde Futures es indefinido.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:58-110
- **Condiciones** (normales, límites, errores, efectos): tras terminar el actor, el contexto sigue siendo no nulo.
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: AC-CELL-018
- **Equivalente Rust propuesto**: `&mut Context` prestado sólo durante el handler
- **Diferencias observables**: en Rust el préstamo lo impide por tipos; qué hacer con un ctx capturado tras terminar (Q-CELL-3).
- **Aserciones**:
  - A1: context no nulo tras context.stop(self) y la respuesta "pong" llega — ActorLifeCycleSpec.scala:193-230.
- **Oráculo**: no aplica.
- **Decisiones**: Q-CELL-3
- **Notas**: Sigue listed: sólo leí los comentarios de self/props/receiveTimeout (ActorCell.scala:60-104); falta el resto (ActorCell.scala:106-250). Contradicción corregida: el contexto NO se anula tras terminar (ActorLifeCycleSpec.scala:210-214: comentario "Previously, the context was nulled…"). Q-CELL-3.

### AC-CELL-020: El mensaje que causó el fallo no se reprocesa tras restart

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: El mensaje durante el cual falló se entrega a preRestart como Option y no se reprocesa en la nueva instancia; el sender de ese mensaje es accesible en preRestart.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala:99-104 ; actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:559 ; docs/src/main/paradox/general/supervision.md:59-79 ; docs/src/main/paradox/actors.md:471-497
- **Condiciones** (normales, límites, errores, efectos): optionalMessage = Some(currentMessage.message) si currentMessage != null, si no None.
- **Orden / concurrencia / ciclo de vida**: ver AC-CELL-007.1.
- **Dependencias**: AC-CELL-007.1
- **Equivalente Rust propuesto**: -
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: preRestart puede responder al sender del mensaje que falló — Ticket669Spec.scala:36-47.
  - A2: documento: "the message during which the failure occurred is not re-processed" — supervision.md:76-79 (doc).
  - A3: nueva instancia no ve el mensaje — derivada de FaultHandling.scala:99-112 (no afirmada por spec).
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales).
- **Decisiones**: -
- **Notas**: Dato nuevo: currentMessage sólo se limpia tras éxito; en un restart provocado por un hermano (AllForOne) o por el padre, optionalMessage puede ser Some(mensaje anterior fallido) o None según haya currentMessage; docs actors.md:471-476 dicen None si el restart no fue causado por un mensaje.

### AC-CELL-021: Interrupción de hilo: InterruptedException → ActorInterruptedException al padre

- **Estado de análisis**: extracted. Visibilidad: internal-required.
- **Contrato observable**: InterruptedException durante receive se reporta al padre envuelta en ActorInterruptedException(cause) y se re-marca el flag de interrupción del hilo; durante la creación: ActorInitializationException(self, "interruption during creation", e) y setFailedFatally.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala:226-234,353-359 ; actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:658-662
- **Condiciones** (normales, límites, errores, efectos): ActorInterruptedException.getMessage = cause.getMessage. El resto de mensajes sigue procesándose.
- **Orden / concurrencia / ciclo de vida**: ver AC-SUP-001.
- **Dependencias**: AC-SUP-001
- **Equivalente Rust propuesto**: N/A (Rust no interrumpe hilos)
- **Diferencias observables**: sin equivalente (Q-CELL-2).
- **Aserciones**:
  - A1: dos Interrupt con falla → ActorInterruptedException con getCause.getMessage "Ping!", y los demás mensajes siguen — ActorModelSpec.scala:422-455.
  - A2: InterruptNicely no produce fallo — ActorModelSpec.scala:457-477.
- **Oráculo**: no aplica: específico de JVM.
- **Decisiones**: Q-CELL-2
- **Notas**: Q-CELL-2: excluir; en Rust la cancelación cooperativa de tareas no equivale (propuesta, la fila sigue en alcance).

### AC-CELL-022: Hooks de instrumentación aroundReceive/aroundPreStart/aroundPostStop/aroundPreRestart/aroundPostRestart

- **Estado de análisis**: extracted. Visibilidad: internal-required.
- **Contrato observable**: aroundX envuelve al hook real: aroundReceive aplica la conducta y llama unhandled si no matchea; aroundPreStart → preStart; aroundPostStop → postStop; aroundPreRestart → preRestart; aroundPostRestart → postRestart. Pueden sobrescribirse (protected[pekko]).
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/Actor.scala:545-584 ; actor/src/main/scala/org/apache/pekko/actor/Actor.scala:370-382
- **Condiciones** (normales, límites, errores, efectos): n/a.
- **Orden / concurrencia / ciclo de vida**: la celda siempre llama a los around*, nunca directo al hook.
- **Dependencias**: AC-CELL-006
- **Equivalente Rust propuesto**: trait `Interceptor` o métodos `pub(crate)` con implementación por defecto
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: aroundReceive llama unhandled(msg) cuando applyOrElse devuelve NotHandled — derivada de Actor.scala:545-551 (no afirmada por spec).
  - A2: los around* delegan en los hooks — derivada de Actor.scala:553-580 (no afirmada por spec).
- **Oráculo**: no aplica.
- **Decisiones**: -
- **Notas**: Sin spec. Ejemplo de uso en Pekko: DiagnosticActorLogging.aroundReceive pone el MDC antes de super.aroundReceive y lo limpia en finally (Actor.scala:375-381).

### AC-CELL-023: Eventos Debug de ciclo de vida y AutoReceive ("started", "restarting", "restarted", "stopping", "stopped", "now supervising", "now watched by")

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: Con debug.lifecycle=on se publican eventos Debug (source = path del actor, clase = clase del actor): "started (<actor>)", "restarting", "restarted", "stopping", "stopped", "now supervising <child>" (source = path del supervisor), "now watched by <watcher>", "no longer watched by <watcher>". Con debug.autoreceive=on: "received AutoReceiveMessage <envelope>".
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:569-570,655-656,699-700 ; actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala:99,206,271,290 ; actor/src/main/scala/org/apache/pekko/actor/dungeon/DeathWatch.scala:189,207
- **Condiciones** (normales, límites, errores, efectos): "stopping" sólo se emite la primera vez que se espera a los hijos (!wasTerminating). Los textos deben incluir exactamente el toString de actor/envelope/ref de Pekko.
- **Orden / concurrencia / ciclo de vida**: "started" tras preStart; "restarting" antes de preRestart; "restarted" tras postRestart; "stopping" al iniciar la espera de hijos; "stopped" al final de finishTerminate.
- **Dependencias**: AC-CFG-011
- **Equivalente Rust propuesto**: publicar `LogEvent::Debug` con los mismos textos
- **Diferencias observables**: toString del actor/envelope (Q-CELL-4).
- **Aserciones**:
  - A1: AutoReceive: Debug "received AutoReceiveMessage Envelope(PoisonPill" — LoggingReceiveSpec.scala:196-199.
  - A2: "now supervising" para guardián y supervisor — LoggingReceiveSpec.scala:215-224.
  - A3: "now watched by" / "no longer watched by" — LoggingReceiveSpec.scala:230-252.
  - A4: started(…)+now supervising, restarting, restarted, y Error(ActorKilledException, aname, _, "Kill"), y al parar: "stopped" del hijo y "stopping"/"stopped" del supervisor — LoggingReceiveSpec.scala:263-292.
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales): suscribir a Debug con debug.lifecycle=on y listar textos.
- **Decisiones**: Q-CELL-4
- **Notas**: Contradicción corregida: las líneas del inventario viejo (FaultHandling.scala:98,203,291) estaban corridas; los textos reales están en 99 ("restarting"), 206 ("stopping"), 271 ("stopped"), 290 ("restarted"). Faltaba "no longer watched by <watcher>" (DeathWatch.scala:207). Config: pekko.actor.debug.lifecycle y debug.autoreceive (reference.conf:736-746). Q-CELL-4.

### AC-CELL-024: Limpieza de campos al terminar (props=terminatedProps, instancia nula, unstashAll)

- **Estado de análisis**: extracted. Visibilidad: internal-required.
- **Contrato observable**: Tras terminar: _props = terminatedProps (su fábrica lanza IllegalActorStateException("This Actor has been terminated")), _actor = null, sysmsgStash descartado, currentMessage = null, behaviorStack vacío.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:386-388,716-726
- **Condiciones** (normales, límites, errores, efectos): la limpieza ocurre DESPUÉS de enviar los Terminated (se verifica "durante un tiempo razonable").
- **Orden / concurrencia / ciclo de vida**: último paso de finishTerminate.
- **Dependencias**: AC-CELL-008.2
- **Equivalente Rust propuesto**: la celda libera la fábrica y la instancia al terminar
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: tras Terminated, el hijo expone Props == terminatedProps (misma instancia) — LocalActorRefProviderSpec.scala:109-129.
- **Oráculo**: no aplica.
- **Decisiones**: -
- **Notas**: Omisión agregada: no estaba en el inventario. Evidencia: ActorCell.terminatedProps y clearFieldsForTermination.

### AC-CELL-025: pekko.actor.debug.unhandled: reenviador de UnhandledMessage a Debug

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: Con debug.unhandled=on, al iniciar loggers se crea el actor de sistema "UnhandledMessageForwarder" suscrito a UnhandledMessage que publica Debug(rcp.path.toString, rcp.getClass, "unhandled message from " + sender + ": " + msg).
- **Fuente**: actor/src/main/resources/reference.conf:754-755 ; actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:460 ; actor/src/main/scala/org/apache/pekko/event/Logging.scala:154-170
- **Condiciones** (normales, límites, errores, efectos): Si el nombre ya existe (InvalidActorNameException) se ignora.
- **Orden / concurrencia / ciclo de vida**: depende del arranque de loggers.
- **Dependencias**: AC-EVT-010 ; AC-CELL-004
- **Equivalente Rust propuesto**: suscriptor interno de `UnhandledMessage` que publica un Debug
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: se publica el UnhandledMessage original y además Logging.Debug(deadLetters.path, deadLetters.getClass, "unhandled message from Actor[…deadLetters]: 42") — EventStreamSpec.scala:119-126.
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales).
- **Decisiones**: -
- **Notas**: Omisión agregada (clave de reference.conf sin fila). Evidencia: reference.conf debug.unhandled y Logging.scala. Compartida con AC-EVT-010.

### AC-CELL-026: Validación del tipo de mailbox exigido por el actor en Create

- **Estado de análisis**: extracted. Visibilidad: internal-required.
- **Contrato observable**: Si el mailboxType produce ProducesMessageQueue y el actor exige un tipo, init compara; si no coincide Create lleva Some(ActorInitializationException(self, "Actor [<self>] requires mailbox type [<req>] got [<got>]")) (got = "null" si no hay cola).
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/Dispatch.scala:79-116
- **Condiciones** (normales, límites, errores, efectos): el fallo se difiere al procesamiento de Create para tratarse como fallo normal de creación.
- **Orden / concurrencia / ciclo de vida**: Create sigue siendo el primer mensaje de sistema.
- **Dependencias**: AC-MBX-016 ; AC-CELL-002.2
- **Equivalente Rust propuesto**: comprobación de `RequiresMessageQueue` contra el tipo de cola del mailbox
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: tipo exigido distinto del real → ActorInitializationException con ese texto — derivada de Dispatch.scala:84-100 (no afirmada por spec).
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales): actor con Stash y mailbox no deque.
- **Decisiones**: -
- **Notas**: Omisión agregada. Sin spec en actor-tests (búsqueda de "requires mailbox type" sin resultados).

### AC-CELL-027: Fallo al crear la celda (initWithFailure) y punto de creación diferida

- **Estado de análisis**: extracted. Visibilidad: internal-required.
- **Contrato observable**: Si newCell falla con catchFailures=true se crea un ActorCell con dispatcher global por defecto y UnboundedMailbox y Create(Some(ActorInitializationException(self, "failure while creating ActorCell", cause))). Un dispatcher no configurado lanza ConfigurationException("Dispatcher [<id>] not configured for path <path>"); un problema de configuración al crear: ConfigurationException("configuration problem while creating [<path>] with dispatcher [<d>] and mailbox [<m>]", e).
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/Dispatch.scala:118-129 ; actor/src/main/scala/org/apache/pekko/actor/RepointableActorRef.scala:112-121 ; actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:704-719
- **Condiciones** (normales, límites, errores, efectos): top-level cuyo mailbox falla: 1 ActorInitializationException logueada y el ref termina; hijo normal: el padre recibe (self, sender, ConfigurationException) cuya causa es el error original.
- **Orden / concurrencia / ciclo de vida**: el error aparece como fallo de creación normal (AC-CELL-002.2).
- **Dependencias**: AC-REF-020
- **Equivalente Rust propuesto**: celda "fallida" que sólo entrega ActorInitializationError
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: top-level con mailbox que lanza → ActorInitializationException logueada y Terminated — SupervisorSpec.scala:510-514.
  - A2: hijo con mailbox que lanza → ConfigurationException con causa — SupervisorSpec.scala:516-525.
  - A3: router top-level con mailbox que lanza → terminado — SupervisorSpec.scala:527-531.
- **Oráculo**: no aplica.
- **Decisiones**: -
- **Notas**: Omisión agregada. Cuando falla la creación del mailbox, la ref termina; el error se informa al supervisor.

### AC-CELL-028: pekko.actor.serialize-messages: ida y vuelta de mensajes en modo test

- **Estado de análisis**: listed. Visibilidad: internal-required.
- **Contrato observable**: Con serialize-messages=on, sendMessage serializa y deserializa el mensaje (desenvolviendo DeadLetter) salvo NoSerializationVerificationNeeded o clases con prefijo en no-serialization-verification-needed-class-prefix (default ["org.apache.pekko."]); un fallo lanza SerializationCheckFailedException(msg, e) que se traga y se loguea como Error.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/Dispatch.scala:171-225 ; actor/src/main/resources/reference.conf:129-140
- **Condiciones** (normales, límites, errores, efectos): pendiente de la forma exacta del mensaje de SerializationCheckFailedException.
- **Orden / concurrencia / ciclo de vida**: antes de encolar.
- **Dependencias**: AC-SER-001
- **Equivalente Rust propuesto**: hook de test que serializa y deserializa cada mensaje enviado (feature opcional)
- **Diferencias observables**: Q-CELL-5.
- **Aserciones**:
  - A1: un FunctionRef no registrado no sobrevive a la ida y vuelta: EventFilter[SerializationCheckFailedException] start "Failed to serialize and deserialize message of type org.apache.pekko.actor.FunctionRefSpec" — FunctionRefSpec.scala:108-120.
- **Oráculo**: no aplica.
- **Decisiones**: Q-CELL-5
- **Notas**: Omisión agregada. Sigue listed: no leí SerializationCheckFailedException ni la ruta de SerializationExtension (unidad SER). Q-CELL-5.

### AC-CELL-029: pekko.actor.serialize-creators: verificación de argumentos de Props al crear hijos

- **Estado de análisis**: extracted. Visibilidad: internal-required.
- **Contrato observable**: Con serialize-creators=on y deploy.scope != LocalScope, makeChild serializa/deserializa cada arg de Props salvo null, NoSerializationVerificationNeeded o prefijos exentos; falla con IllegalArgumentException("pre-creation serialization check failed at [<path>/<name>]", causa).
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/Children.scala:279-306 ; actor/src/main/resources/reference.conf:133-140
- **Condiciones** (normales, límites, errores, efectos): los systemService quedan exentos; deploy local queda exento.
- **Orden / concurrencia / ciclo de vida**: ocurre antes de reservar el nombre.
- **Dependencias**: AC-SER-001
- **Equivalente Rust propuesto**: hook de test que serializa los argumentos de Props (feature opcional)
- **Diferencias observables**: Q-CELL-5.
- **Aserciones**:
  - A1: con la opción activa, Props con args null funcionan — PropsCreationSpec.scala:54-80.
  - A2: mensaje de error de verificación — derivada de Children.scala:299-301 (no afirmada por spec).
- **Oráculo**: no aplica.
- **Decisiones**: Q-CELL-5
- **Notas**: Omisión agregada. Los tests existentes sólo comprueban que, con la opción activa, crear actores con args null no falla (el caso de error no se afirma). Q-CELL-5.

### AC-CELL-030: Errores al enviar (suspend/resume/restart/stop/sendMessage/sendSystemMessage) se tragan y se loguean

- **Estado de análisis**: extracted. Visibilidad: internal-required.
- **Contrato observable**: Las operaciones de ciclo de vida y envío no propagan excepciones: InterruptedException → Error "interrupted during message send" y se re-marca; otra no fatal → Error "swallowing exception during message send" (con ": <mensaje>" si es NoStackTrace).
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/Dispatch.scala:137-170,225-228
- **Condiciones** (normales, límites, errores, efectos): nunca se pierde un mensaje de sistema por esta vía en condiciones normales.
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: AC-REF-002
- **Equivalente Rust propuesto**: `catch_unwind`/Result en el envío con log Error
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: excepción durante sendMessage se publica como Error con ese texto — derivada de Dispatch.scala:137-148 (no afirmada por spec).
- **Oráculo**: no aplica.
- **Decisiones**: -
- **Notas**: Omisión agregada. Sin spec. Comentario del código: "this function is marked noinline because Kamon…" (instrumentación JVM, no aplica).

### AC-SUP-001: Propagación de fallos: excepción en receive → suspender actor e hijos → Failed(child, cause, uid) al padre

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: Excepción no fatal en invoke/systemInvoke → handleInvokeFailure: suspende su mailbox (suspendNonRecursive), marca setFailed(child | self), suspende a todos sus hijos excepto el hijo cuyo Failed está procesando y childrenNotToSuspend, y envía Failed(self, causa, uid) al padre (InterruptedException se envuelve en ActorInterruptedException). El actor deja de procesar mensajes de usuario.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala:49-87,214-245 ; actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:549-566 ; docs/src/main/paradox/general/supervision.md:59-79
- **Condiciones** (normales, límites, errores, efectos): Si falla el propio manejo (p. ej. enviar al padre): parada de emergencia (AC-SUP-016). uid = self.path.uid.
- **Orden / concurrencia / ciclo de vida**: el padre recibe Failed por cola de sistema (no ordenado con mensajes de usuario). "NEVER SEND THE SAME SYSTEM MESSAGE OBJECT TO TWO ACTORS" (se crea un Failed nuevo por envío).
- **Dependencias**: AC-CELL-013.1
- **Equivalente Rust propuesto**: handler devuelve `Err`/panic capturado con `catch_unwind` → `SystemMessage::Failed`
- **Diferencias observables**: Q-X-5.
- **Aserciones**:
  - A1: mientras el padre decide, el hijo suspendido no procesa mensajes de usuario (worker no responde durante 2s y luego "pong") — SupervisorHierarchySpec.scala:832-859.
  - A2: one-way Ping, Die, Ping se observan en orden y el actor sigue operativo — SupervisorSpec.scala:389-399.
  - A3: call-kill-call mantiene el servicio tras el restart — SupervisorSpec.scala:311-316.
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales).
- **Decisiones**: Q-X-5
- **Notas**: Q-X-5. Detalle: si el actor ya está fallado (isFailed) no vuelve a suspender hijos ni a fallar de nuevo ("while currentlyFailed() they do not fail again"; salvo fallo en constructor/preStart de restart).

### AC-SUP-002: El padre sólo actúa ante Failed de un hijo actual con uid coincidente

- **Estado de análisis**: extracted. Visibilidad: internal-required.
- **Contrato observable**: handleFailure: currentMessage = Envelope(f, f.child); busca al hijo por ref; si es hijo actual y stats.uid == f.uid → delega en supervisorStrategy.handleFailure(this, f.child, f.cause, stats, allChildStats) y si devuelve false relanza f.cause; si el hijo existe con otro uid → Debug "dropping Failed(<cause>) from old child <child> (uid=<stats.uid> != <f.uid>)"; si no es hijo → Debug "dropping Failed(<cause>) from unknown child <child>".
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala:305-325
- **Condiciones** (normales, límites, errores, efectos): ninguna de las dos descartadas altera el estado.
- **Orden / concurrencia / ciclo de vida**: sender() dentro del decider es f.child (currentMessage.sender).
- **Dependencias**: AC-PATH-011
- **Equivalente Rust propuesto**: comparar uid en `ChildStats`
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: Failed de incarnation vieja se descarta con ese Debug — derivada de FaultHandling.scala:315-320 (no afirmada por spec).
  - A2: Failed de hijo desconocido se descarta con ese Debug — derivada de FaultHandling.scala:321-323 (no afirmada por spec).
  - A3: Terminated de un hijo viejo detenido en preRestart no llega tras el restart (DeathWatch) — UidClashTest.scala:103-111.
- **Oráculo**: no aplica.
- **Decisiones**: -
- **Notas**: Contradicción corregida: UidClashTest.scala:103-111 prueba DeathWatch (un Terminated de un hijo viejo no llega tras el restart: unwatch + stop en preRestart y watch de un ref con path colisionado), no el descarte de Failed por uid. Sin spec que afirme los mensajes Debug de descarte (grep "dropping Failed" en actor-tests sin resultados).

### AC-SUP-003: Directivas Resume/Restart/Stop/Escalate y su nivel de log

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: Resume se loguea en Warning, Restart y Stop en Error, Escalate no se loguea; se pueden fijar niveles propios con resume(level), restart(level), stop(level).
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/FaultHandling.scala:114-201,395-430
- **Condiciones** (normales, límites, errores, efectos): loggingEnabled=false desactiva el log; ver AC-SUP-018.
- **Orden / concurrencia / ciclo de vida**: el log ocurre antes de aplicar la directiva.
- **Dependencias**: AC-EVT-012
- **Equivalente Rust propuesto**: `enum Directive { Resume(LogLevel), Restart(LogLevel), Stop(LogLevel), Escalate }`
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: Resume → Warning "expected" 1 vez — SupervisorHierarchySpec.scala:815-830.
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales): niveles de log por directiva.
- **Decisiones**: -
- **Notas**: Las variantes con LogLevel propio: SupervisorStrategy.resume(logLevel)/restart(logLevel)/stop(logLevel) (FaultHandling.scala:175,189,201). Escalate.logLevel lanza IllegalStateException("Escalate is not logged").

### AC-SUP-004: defaultDecider

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: ActorInitializationException, ActorKilledException y DeathPactException → Stop; cualquier otra Exception → Restart; Throwable que no es Exception: el decider no está definido y handleFailure usa escalateDefault → Escalate.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/FaultHandling.scala:210-223,363-376
- **Condiciones** (normales, límites, errores, efectos): decider.applyOrElse(cause, escalateDefault).
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: AC-SUP-003
- **Equivalente Rust propuesto**: función `default_decider(&Failure) -> Directive`
- **Diferencias observables**: Q-SUP-1 (modelo de errores).
- **Aserciones**:
  - A1: Kill → Failed(_, ActorKilledException) y el hijo se detiene — DeathWatchSpec.scala:237-245 (FF(Failed(_, _: ActorKilledException, _)) y lastSender eq failed).
  - A2: watcher sin Terminated → DeathPactException(failed) → Stop y el supervisor lo ve — DeathWatchSpec.scala:240-244.
  - A3: top-level cuyo creator falla: se detiene — SupervisorSpec.scala:510-514.
  - A4: Throwable no Exception → Escalate — derivada de FaultHandling.scala:218-223 y 376 (no afirmada por spec).
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales).
- **Decisiones**: Q-SUP-1, Q-X-5
- **Notas**: Contradicción corregida: ActorRefSpec.scala:442-463 y SupervisorSpec.scala:546 usan deciders custom; la parada por defecto de ActorKilledException/DeathPactException se afirma en DeathWatchSpec.scala:216-245 y la de ActorInitializationException en SupervisorSpec.scala:510-514. Q-SUP-1.

### AC-SUP-005: defaultStrategy y stoppingStrategy

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: defaultStrategy = OneForOneStrategy()(defaultDecider) (sin límite de reintentos); stoppingStrategy = OneForOneStrategy()({ case _: Exception => Stop }); los configuradores DefaultSupervisorStrategy / StoppingSupervisorStrategy las devuelven.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/FaultHandling.scala:225-244,96-107
- **Condiciones** (normales, límites, errores, efectos): stoppingStrategy deja Throwable no Exception sin directiva → Escalate.
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: AC-SUP-006.1
- **Equivalente Rust propuesto**: constantes/constructores
- **Diferencias observables**: registro de configuradores por nombre (Q-X-2).
- **Aserciones**:
  - A1: guardián con StoppingSupervisorStrategy: hijo que lanza Exception("hello") → Terminated(a)(true,false) — ActorSystemSpec.scala:336-357.
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales).
- **Decisiones**: Q-X-2
- **Notas**: Contradicción corregida: SupervisorSpec.scala:292 (cita vieja) prueba "not restart temporary actor" (AllForOne maxNrOfRetries=0), no stoppingStrategy. DefaultSupervisorStrategy y StoppingSupervisorStrategy son SupervisorStrategyConfigurator (create()).

### AC-SUP-006.1: OneForOneStrategy: aplicación al hijo que falló y límite de reintentos (maxNrOfRetries)

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: OneForOne aplica la directiva sólo al hijo que falló. Sin ventana: maxNrOfRetries<0 → ilimitado; =0 → ningún reinicio (la primera falla detiene); N>0 → se permiten N reinicios, la N+1-ésima falla detiene (stats.maxNrOfRetriesCount += 1 <= N). Al denegarse el permiso: context.stop(child).
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/FaultHandling.scala:49-78,582-678
- **Condiciones** (normales, límites, errores, efectos): requestRestartPermission((Some(r), _)) con r<1 → false; (Some(r), None) → contador++ y contador<=r; (None, _) → true.
- **Orden / concurrencia / ciclo de vida**: restartChild(child, cause, suspendFirst=false) (el hijo ya está suspendido por su fallo).
- **Dependencias**: AC-SUP-003
- **Equivalente Rust propuesto**: `ChildRestartStats { retries_count, window_start }`; `OneForOneStrategy { max_retries, within, decider, logging }`
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: maxNrOfRetries=0: el hijo se detiene al primer fallo y el supervisor ve Terminated — SupervisorSpec.scala:232-238.
  - A2: maxNrOfRetries=3: 3 restarts (preRestart/postRestart informados) y el 4º fallo detiene (postStop1) — SupervisorSpec.scala:240-290.
  - A3: maxNrOfRetries=2 sin ventana: 2 reinicios y el 3er Crash detiene — RestartStrategySpec.scala:173-222.
  - A4: sin límites: 100 Crash → 100 postRestart y el empleado sigue vivo — RestartStrategySpec.scala:91-113.
  - A5: -1 y Duration.Inf: 7 kills sin detenerse — SupervisorSpec.scala:572-587.
  - A6: maxNrOfRetries=3 y Duration.Inf: 3 reinicios y el 4º detiene — SupervisorSpec.scala:601-612.
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales).
- **Decisiones**: Q-SUP-2
- **Notas**: Hijo de AC-SUP-006. Q-SUP-2.

### AC-SUP-006.2: OneForOneStrategy: ventana withinTimeRange

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: Con ventana finita (>=0) y N=-1 se usa N=1; la ventana se abre al primer permiso solicitado (restartTimeWindowStartNanos==0 ⇒ now) y no se desliza: dentro de la ventana (now-start <= window) cuenta+1 y se permite si cuenta<=N; fuera de la ventana: cuenta=1, nueva ventana desde now, permitido. Duration.Inf equivale a "sin ventana".
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/FaultHandling.scala:49-90,303-309,550,660-662
- **Condiciones** (normales, límites, errores, efectos): withinTimeRange infinito o negativo → None (sin ventana); ventana 0 → Some(0). Cada hijo tiene su propio contador (ChildRestartStats).
- **Orden / concurrencia / ciclo de vida**: el contador se actualiza al solicitar permiso, aunque el reinicio luego se descarte.
- **Dependencias**: AC-SUP-006.1
- **Equivalente Rust propuesto**: `RestartWindow { start: Option<Instant>, count }` con `Instant` monótono
- **Diferencias observables**: Q-SUP-2.
- **Aserciones**:
  - A1: 2 restarts en 1s: 3er Crash detiene (stopLatch) — RestartStrategySpec.scala:43-89.
  - A2: 2 reinicios en 500ms, espera de 700ms, otro Crash → vuelve a reiniciar — RestartStrategySpec.scala:115-171.
  - A3: sin maxNrOfRetries (None) y ventana de 1s: el segundo fallo dentro de la ventana detiene al hijo (equivale a N=1) — RestartStrategySpec.scala:224-277.
  - A4: -1 con ventana de 10s: 1 reinicio y el siguiente fallo detiene — SupervisorSpec.scala:589-599.
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales): secuencias con sleep; mejor con reloj simulado en Rust.
- **Decisiones**: Q-SUP-2
- **Notas**: Hijo de AC-SUP-006. Q-SUP-2: withinTimeRange.toMillis.toInt (truncamiento a Int); la ventana usa System.nanoTime (Instant en Rust).

### AC-SUP-007: AllForOneStrategy

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: La directiva se aplica a todos los hijos (stats de todos los ChildRestartStats): Restart sólo si todos permiten (forall); si no, se detiene a todos. Los hermanos se reinician con suspendFirst = (hermano != hijo fallido), o sea se suspenden antes de reiniciar.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/FaultHandling.scala:468-568
- **Condiciones** (normales, límites, errores, efectos): children vacío → no hace nada. Resume y Stop siguen la rama de handleFailure (Resume sólo resume al hijo que falló; Stop se aplica a todos por processFailure(restart=false)).
- **Orden / concurrencia / ciclo de vida**: el orden de reinicio sigue el orden de iteración del contenedor de hijos (nombre).
- **Dependencias**: AC-SUP-006.2
- **Equivalente Rust propuesto**: igual que OneForOne sobre el conjunto
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: AllForOne kill de uno de 3 → dos mensajes de excepción más (los otros reinician) — SupervisorSpec.scala:354-362.
  - A2: call-kill-call con 3 actores y todos siguen vivos — SupervisorSpec.scala:364-380.
  - A3: AllForOne anidado — SupervisorSpec.scala:401-417.
  - A4: manager con AllForOneStrategy()(List()) escala; boss lo reinicia y los workers también: 4 postRestart — SupervisorHierarchySpec.scala:767-788.
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales).
- **Decisiones**: -
- **Notas**: Detalle: `children.forall(_.requestRestartPermission(...))` se corta en el primer false, por lo que no todos los contadores se incrementan en esa decisión (derivado del código; fuente de diferencias si se calcula con map).

### AC-SUP-008: Escalate: el padre relanza la causa sin envolver hacia su propio supervisor

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: handleFailure devuelve false para Escalate; la celda hace `throw f.cause` dentro del manejo de Failed, que se convierte en un fallo del propio padre con la MISMA causa (sin envolver), enviado a su supervisor.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/FaultHandling.scala:352-393 ; actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala:305-314 ; actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala:214-225
- **Condiciones** (normales, límites, errores, efectos): logFailure no loguea Escalate.
- **Orden / concurrencia / ciclo de vida**: el hijo escalador queda suspendido hasta que alguien decida sobre él.
- **Dependencias**: AC-SUP-001
- **Equivalente Rust propuesto**: propagar `Failure` original
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: manager con trapExit vacío escala: boss lo reinicia y reinician los workers — SupervisorHierarchySpec.scala:767-788.
  - A2: /user escala → la raíz detiene /user y el sistema se apaga (whenTerminated) — ActorSystemSpec.scala:359-375.
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales).
- **Decisiones**: -
- **Notas**: Contradicción corregida: SupervisorHierarchySpec.scala:790 (cita vieja) prueba el Terminated tras max reintentos, no Escalate. Detalle: al procesar Failed del hijo, currentMessage = Envelope(f, child) → handleInvokeFailure no vuelve a suspender a ese hijo ya suspendido (skip).

### AC-SUP-009: Resume: el hijo conserva estado y se reanudan sus hijos

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: Resume reanuda al actor sin recrearlo (conserva estado); resumeChildren(causedByFailure, perp): cada hijo recibe resume(causa) sólo si es el perpetrador, los demás resume(null); sólo el perpetrador limpia su marca de fallo (clearFailed).
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala:130-153 ; actor/src/main/scala/org/apache/pekko/actor/dungeon/Children.scala:216-228
- **Condiciones** (normales, límites, errores, efectos): si el actor no llegó a crearse → Create (AC-SUP-011); si fallo fatal previo y hay causa → Restart (AC-SUP-019).
- **Orden / concurrencia / ciclo de vida**: suspend/resume en pares (AC-SUP-010).
- **Dependencias**: AC-SUP-003
- **Equivalente Rust propuesto**: -
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: tras Resume el padre y el worker responden (estado conservado) — SupervisorHierarchySpec.scala:815-830.
  - A2: mientras el supervisor decide Resume, el hijo suspendido no procesa; luego responde — SupervisorHierarchySpec.scala:832-859.
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales).
- **Decisiones**: -

### AC-SUP-010: Suspensión recursiva de hijos durante el fallo y contadores balanceados

- **Estado de análisis**: extracted. Visibilidad: internal-required.
- **Contrato observable**: Cada fallo propio o recursivo incrementa suspendCount del mailbox (suspend() devuelve true si estaba en 0); resume() decrementa y el actor vuelve a procesar al llegar a 0 (dispatcher.resume reprograma). Cerrado no cambia. Un actor fallado no se suspende de nuevo por cascada propia.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala:49-87,120-130,214-245 ; actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala:121-170 ; actor/src/main/scala/org/apache/pekko/dispatch/AbstractDispatcher.scala:277-293
- **Condiciones** (normales, límites, errores, efectos): faultSuspend siempre suspende (equilibrio); un hijo hereda tantas suspensiones como tenga el padre al crearse (AC-CELL-016).
- **Orden / concurrencia / ciclo de vida**: Suspend se difiere si la celda espera hijos (AC-CELL-013.2).
- **Dependencias**: AC-MBX-001
- **Equivalente Rust propuesto**: contador de suspensión en mailbox (entero) + reanudar al llegar a 0
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: el worker nunca procesa mientras algún ancestro está fallado (mailbox.isSuspended y sin respuesta durante 2s) — SupervisorHierarchySpec.scala:845-855.
  - A2: suspend/resume son pares; suspendCount = status / suspendUnit — derivada de Mailbox.scala:126-170 (no afirmada por spec).
- **Oráculo**: no aplica.
- **Decisiones**: Q-X-4
- **Notas**: Sólo leí el inicio del test de estrés (SupervisorHierarchySpec.scala:914-935), no su cuerpo completo.

### AC-SUP-011: Fallo durante la creación: Recreate/Resume se convierten en Create

- **Estado de análisis**: extracted. Visibilidad: internal-required.
- **Contrato observable**: Si el actor no llegó a crearse (actor == null): faultRecreate publica Error "changing Recreate into Create after <causa>"; faultResume publica Error "changing Resume into Create after <causa>"; ambos → faultCreate: cancela receive timeout, detiene los hijos (children.foreach(stop)) y, tras su parada (reason Creation), finishCreate: resume + clearFailed + create(None) (nuevo constructor y preStart).
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala:89-96,130-143,155-178
- **Condiciones** (normales, límites, errores, efectos): assert(perpetrator == self) en faultCreate (falla es del propio actor).
- **Orden / concurrencia / ciclo de vida**: Create corre sólo cuando no quedan hijos.
- **Dependencias**: AC-CELL-002.2
- **Equivalente Rust propuesto**: -
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: constructor que falla 5 veces con Resume/Restart alternados: 6 intentos, preStart 1 vez, postRestart 0 — SupervisorHierarchySpec.scala:861-912.
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales).
- **Decisiones**: -

### AC-SUP-012: Helpers makeDecider (Seq de excepciones → Restart si no Escalate; lista CauseDirective; JDecider)

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: makeDecider(Seq[Class]) → Restart si el error es instancia de alguna clase, si no Escalate; makeDecider(Iterable[CauseDirective]) ordena (sort inserta cada par antes de cualquier supertipo ya presente) y devuelve la primera directiva cuyo tipo coincide, si no Escalate; makeDecider(JDecider) delega; conversiones implícitas seqThrowable2Decider y seqCauseDirective2Decider.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/FaultHandling.scala:246-309
- **Condiciones** (normales, límites, errores, efectos): lista vacía → todo Escalate.
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: AC-SUP-004
- **Equivalente Rust propuesto**: closures; matching por tipo de error con `downcast_ref`
- **Diferencias observables**: Q-SUP-1.
- **Aserciones**:
  - A1: List(classOf[Exception]) reinicia Exception — SupervisorSpec.scala:240-290.
  - A2: List() escala todo — SupervisorHierarchySpec.scala:775-788.
  - A3: orden de CauseDirective: subtipos antes que supertipos — derivada de FaultHandling.scala:287-301 (no afirmada por spec).
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales).
- **Decisiones**: Q-SUP-1
- **Notas**: Depende de cómo se tipan los fallos (Q-SUP-1). Los tests sólo usan List(classOf[Exception]) y List() (que escalan todo).

### AC-SUP-013: Puntos de extensión de SupervisorStrategy: decider, handleChildTerminated, processFailure, handleFailure, logFailure, loggingEnabled, resumeChild, restartChild

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: Estrategias custom pueden redefinir decider, handleChildTerminated(context, child, children), processFailure(context, restart, child, cause, stats, children), handleFailure (devuelve true si manejó), logFailure, loggingEnabled; resumeChild(child, cause) y restartChild(child, cause, suspendFirst) son final. En el decider, sender() es el hijo que falló.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/FaultHandling.scala:316-466 ; actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala:305-351
- **Condiciones** (normales, límites, errores, efectos): handleChildTerminated se invoca tras quitar al hijo del contenedor; sus excepciones se loguean (AC-SUP-020). Las excepciones no durante terminación no hacen fallar al actor.
- **Orden / concurrencia / ciclo de vida**: handleFailure corre dentro del procesamiento de Failed.
- **Dependencias**: AC-SUP-006.1
- **Equivalente Rust propuesto**: trait `SupervisorStrategy` con métodos por defecto
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: handleChildTerminated puede crear hijo con el mismo nombre (otro ref, mismo path) — SupervisorMiscSpec.scala:144-159.
  - A2: sender() en el decider es el hijo con path.parent == parent.path y name "child" — SupervisorMiscSpec.scala:165-180.
  - A3: handleFailure sobrescrito se invoca con child==sender — DeathWatchSpec.scala:218-228.
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales).
- **Decisiones**: -

### AC-SUP-014: Estrategias de guardianes: raíz para ante cualquier fallo; /user configurable (Default/Stopping configurators); /system default; guardianes no pierden hijos al reiniciar

- **Estado de análisis**: extracted. Visibilidad: internal-required.
- **Contrato observable**: rootGuardianStrategy = OneForOne { case ex => log.error(ex, "guardian failed, shutting down system"); Stop }; /user usa guardianStrategy (configurador de pekko.actor.guardian-supervisor-strategy, default DefaultSupervisorStrategy); /system usa defaultStrategy; los guardianes no detienen a sus hijos en preRestart (override preRestart vacío). La cadena de vigilancia: systemGuardian vigila a /user y root vigila a systemGuardian; la muerte de /user detiene el sistema.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:312-326,518-575,613-620 ; actor/src/main/resources/reference.conf:119-123
- **Condiciones** (normales, límites, errores, efectos): ActorSystem con guardianProps custom: actorOf desde fuera lanza UnsupportedOperationException.
- **Orden / concurrencia / ciclo de vida**: init del provider envía Watch(guardian, systemGuardian) y Watch(systemGuardian, rootGuardian).
- **Dependencias**: AC-SYS-003 ; AC-CFG-013
- **Equivalente Rust propuesto**: estrategias fijas + registro de configuradores
- **Diferencias observables**: Q-X-2.
- **Aserciones**:
  - A1: Kill a /user detiene el sistema — ActorSystemSpec.scala:328-334.
  - A2: StoppingSupervisorStrategy configurado: el hijo que falla termina — ActorSystemSpec.scala:336-357.
  - A3: configurador que escala: el sistema termina — ActorSystemSpec.scala:359-375.
  - A4: mensajes a los guardianes terminan en UnhandledMessage — LocalActorRefProviderSpec.scala:70-107.
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales).
- **Decisiones**: Q-X-2
- **Notas**: Q-X-2 para resolver la clase del configurador por nombre. Detalle: Guardian.receive: Terminated(_) → context.stop(self); StopChild(child) → context.stop(child). Mensajes de otro tipo a los guardianes → unhandled → Warning "unhandled message from Actor[deadLetters]: …".

### AC-SUP-015: Reinicio de un padre reinicia a sus hijos sobrevivientes

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: Tras postRestart del padre, los hijos que no fueron detenidos en preRestart reciben restart(cause) con la misma causa.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala:278-298
- **Condiciones** (normales, límites, errores, efectos): si el default preRestart los detiene, no sobreviven.
- **Orden / concurrencia / ciclo de vida**: orden: postRestart del padre antes que el preRestart del hijo (AC-CELL-007.3).
- **Dependencias**: AC-CELL-007.3
- **Equivalente Rust propuesto**: -
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: la restauración de middle y luego de last en ese orden — SupervisorTreeSpec.scala:36-41.
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales).
- **Decisiones**: -

### AC-SUP-016: Parada de emergencia ante excepción en el manejo del fallo

- **Estado de análisis**: extracted. Visibilidad: internal-accidental.
- **Contrato observable**: Si falla el propio handleInvokeFailure (p. ej. enviar al padre), se publica Error(e, path, clase, "emergency stop: exception in failure handling for <clase de t><stackTrace>"), se detiene a todos los hijos y se ejecuta finishTerminate.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala:235-244
- **Condiciones** (normales, límites, errores, efectos): n/a.
- **Orden / concurrencia / ciclo de vida**: finishTerminate completo (AC-CELL-008.2).
- **Dependencias**: AC-SUP-001
- **Equivalente Rust propuesto**: -
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: derivada de FaultHandling.scala:235-244 (no afirmada por spec).
- **Oráculo**: no aplica.
- **Decisiones**: Q-X-5
- **Notas**: Sin spec. Comportamiento accidental: se conserva sólo por compatibilidad si se decide (Q-X-5).

### AC-SUP-017: Catálogo de excepciones del ciclo de vida: ActorInitializationException (getActor, path en mensaje), PreRestartException, PostRestartException, OriginalRestartException, DeathPactException, ActorKilledException, InvalidActorNameException, InvalidMessageException, IllegalActorStateException, ActorInterruptedException

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: IllegalActorStateException(message); ActorKilledException(message) NoStackTrace; InvalidActorNameException(message); ActorInitializationException(actor, message, cause) con mensaje "<actor.path>: <message>" si actor != null (getActor); PreRestartException(actor, cause, originalCause, messageOption) mensaje "exception in preRestart(<clase de originalCause o null>, <clase del mensaje o None>)"; PostRestartException(actor, cause, originalCause) mensaje "exception post restart (<clase originalCause o null>)"; OriginalRestartException.unapply recorre PostRestartException anidadas; InvalidMessageException(message); DeathPactException(dead) mensaje "Monitored actor [<dead>] terminated" NoStackTrace; ActorInterruptedException(cause) mensaje = cause.getMessage.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/Actor.scala:166-295
- **Condiciones** (normales, límites, errores, efectos): PreRestartException/PostRestartException son subtipos de ActorInitializationException (el defaultDecider las detiene).
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: -
- **Equivalente Rust propuesto**: `enum ActorError` o structs con `source()`
- **Diferencias observables**: nombres de clase en mensajes (clases de Java) → nombres de tipos Rust (diferencia observable en los mensajes de PreRestart/PostRestart).
- **Aserciones**:
  - A1: mensaje con path "/user/failingActor:" — ActorRefSpec.scala:277-286.
  - A2: PreRestartException y PostRestartException con mensaje "Don't wanna!" logueadas 1 vez cada una — SupervisorSpec.scala:443-446.
  - A3: DeathPactException(`failed`) se extrae del Failed — DeathWatchSpec.scala:240.
  - A4: ActorInterruptedException con getCause.getMessage "Ping!" — ActorModelSpec.scala:449-451.
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales): imprimir getMessage de cada una.
- **Decisiones**: Q-SUP-1
- **Notas**: Q-SUP-1.

### AC-SUP-018: logFailure: texto y destino del log de fallos de hijos

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: Si loggingEnabled: mensaje = si cause es ActorInitializationException con causa: si la causa es InvocationTargetException con causa → ex.getCause.getMessage, si no ex.getMessage; en otro caso cause.getMessage. Escalate no se loguea. Error → publish(Error(cause, child.path.toString, getClass (clase de la estrategia), mensaje)); otros niveles → LogEvent(nivel, child.path.toString, getClass, mensaje). Si el publish falla se ignora.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/FaultHandling.scala:395-433
- **Condiciones** (normales, límites, errores, efectos): source = path del hijo; logClass = clase de la estrategia.
- **Orden / concurrencia / ciclo de vida**: antes de resume/processFailure.
- **Dependencias**: AC-SUP-003 ; AC-EVT-012
- **Equivalente Rust propuesto**: `fn log_failure(&self, ctx, child, cause, directive)`
- **Diferencias observables**: la clase de la estrategia en Rust (nombre de tipo).
- **Aserciones**:
  - A1: Resume loguea Warning con el mensaje de la excepción ("expected") 1 vez — SupervisorHierarchySpec.scala:821-826.
  - A2: ActorInitializationException con causa usa el mensaje de la causa — derivada de FaultHandling.scala:406-413 (no afirmada por spec).
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales): suscribir a Logging.Error.
- **Decisiones**: -
- **Notas**: Omisión agregada. El inventario viejo no incluía el algoritmo de selección del mensaje.

### AC-SUP-019: Fallo fatal en restart: Resume con causa se convierte en Restart y no se repite preRestart

- **Estado de análisis**: extracted. Visibilidad: internal-required.
- **Contrato observable**: Tras un fallo en postRestart (setFailedFatally), isFailedFatally es verdadero: faultResume con causedByFailure != null publica Error "changing Resume into Restart after <causa>" y ejecuta faultRecreate; y faultRecreate omite aroundPreRestart.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala:72-87,104,136-153
- **Condiciones** (normales, límites, errores, efectos): FailedFatally no se limpia con clearFailed (sólo FailedRef).
- **Orden / concurrencia / ciclo de vida**: ver AC-CELL-007.4.
- **Dependencias**: AC-CELL-007.4
- **Equivalente Rust propuesto**: bandera `failed_fatally` en la celda
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: derivada de FaultHandling.scala:136-145 (no afirmada por spec).
  - A2: preRestart omitido si isFailedFatally — derivada de FaultHandling.scala:104 (no afirmada por spec).
- **Oráculo**: no aplica.
- **Decisiones**: -
- **Notas**: Omisión agregada. Sin spec que lo afirme: SupervisorHierarchySpec.scala:922-925 sólo mutea el log "changing Resume into Restart" en el test de estrés.

### AC-SUP-020: Fallo en handleChildTerminated de la estrategia

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: Si supervisorStrategy.handleChildTerminated lanza una excepción no fatal se publica Error(e, path, clase, "handleChildTerminated failed") y el actor falla (handleInvokeFailure(Nil, e)); el cambio de estado del contenedor (finishRecreate/finishCreate/finishTerminate) se ejecuta igualmente tras el intento. Si actor == null no se llama.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala:327-351
- **Condiciones** (normales, límites, errores, efectos): durante terminación, la excepción se loguea y se sigue.
- **Orden / concurrencia / ciclo de vida**: handleChildTerminated corre antes de continuar la acción diferida.
- **Dependencias**: AC-SUP-013 ; AC-DW-008
- **Equivalente Rust propuesto**: -
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: crear un hijo en handleChildTerminated mientras el padre se detiene lanza IllegalStateException, se loguea "handleChildTerminated failed" 1 vez — SupervisorMiscSpec.scala:160-162.
- **Oráculo**: no aplica.
- **Decisiones**: -
- **Notas**: Omisión agregada.

### AC-DW-001: watch: un único Terminated(ref) cuando el vigilado se detiene

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: context.watch(ref) envía Watch(ref, self) al vigilado y lo registra una vez; se recibe exactamente un Terminated aunque se vigile varias veces; watch(self) no hace nada y devuelve el ref.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/DeathWatch.scala:29-49 ; docs/src/main/paradox/actors.md:440-444
- **Condiciones** (normales, límites, errores, efectos): ref no InternalActorRef → IllegalArgumentException("ActorRef is not internal: <ref>") (no representable en Rust). Vigilar dos veces con el mismo modo → no repite el Watch (checkWatchingSame). Vigilar con distinto modo → AC-DW-005.
- **Orden / concurrencia / ciclo de vida**: Watch y Unwatch son mensajes de sistema: asíncronos respecto del mensaje que los origina ("watch and unwatch are asynchronous", DeathWatchSpec.scala:183).
- **Dependencias**: AC-CELL-013.3
- **Equivalente Rust propuesto**: `ctx.watch(&ActorRef) -> ActorRef`
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: un Terminated cuando el actor se detiene — DeathWatchSpec.scala:135-143.
  - A2: tres monitores, cada uno con un Terminated — DeathWatchSpec.scala:154-167.
  - A3: vigilar dos veces envía un único Watch — derivada de DeathWatch.scala:37-45 (no afirmada por spec).
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales).
- **Decisiones**: -
- **Notas**: Dato: el estado de watching es un Map[ActorRef, Option[Any]] (None=Terminated, Some(msg)=watchWith); watch no es observable inmediatamente (asíncrono, el Watch es mensaje de sistema).

### AC-DW-002: watch sobre un actor ya muerto → Terminated inmediato

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: Vigilar un actor terminado produce Terminated(actor) con existenceConfirmed=false si la ref ya estaba muerta (EmptyLocalActorRef o mailbox muerto); true si la parada ocurrió después del watch.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:693-698,745-752 ; actor/src/main/scala/org/apache/pekko/dispatch/Mailboxes.scala:54-72 ; docs/src/main/paradox/actors.md:430-440
- **Condiciones** (normales, límites, errores, efectos): si watcher == watchee == deadLetters no se envía nada.
- **Orden / concurrencia / ciclo de vida**: asíncrono: el Terminated llega más tarde por cola de sistema.
- **Dependencias**: AC-REF-015 ; AC-CELL-013.3
- **Equivalente Rust propuesto**: Watch a ref terminado responde DWN
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: tras PoisonPill y esperar, startWatching → Terminated(terminal) — DeathWatchSpec.scala:145-152.
  - A2: existenceConfirmed=false en ese caso — derivada de ActorRef.scala:746-752 y Mailboxes.scala:62-63 (no afirmada por spec).
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales): imprimir existenceConfirmed.
- **Decisiones**: Q-DW-1
- **Notas**: Hallazgo nuevo: si el actor murió, su mailbox es el deadLetterMailbox; el Watch se publica como DeadLetter y DeadLetterActorRef lo convierte en DWN(existenceConfirmed=false) (sólo si watchee y watcher != deadLetters). Por eso Terminated llega con existenceConfirmed=false para un LocalActorRef ya muerto, a diferencia del caso normal (true).

### AC-DW-003: Restart no genera Terminated

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: Sólo la parada definitiva (finishTerminate) notifica a los watchers; un restart conserva la misma incarnation de ref y sus watchedBy.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala:247-276
- **Condiciones** (normales, límites, errores, efectos): watchedBy se limpia en tellWatchersWeDied (sólo al terminar).
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: AC-CELL-007.1
- **Equivalente Rust propuesto**: -
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: Kill, Kill (restarts), ask "foo" ok, Kill (stop): un único Terminated y terminal.isTerminated — DeathWatchSpec.scala:195-214.
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales).
- **Decisiones**: -

### AC-DW-004: unwatch y descarte de Terminated ya encolado

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: Tras unwatch no se entrega Terminated aunque la notificación ya estuviera en camino o en el mailbox: unwatch envía Unwatch al vigilado (si estaba en watching), quita de watching y de terminatedQueued.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/DeathWatch.scala:67-97,117-119 ; docs/src/main/paradox/actors.md:440-444
- **Condiciones** (normales, límites, errores, efectos): unwatch de un ref no vigilado es inofensivo: igual limpia terminatedQueued; unwatch(self) no envía Unwatch.
- **Orden / concurrencia / ciclo de vida**: DWN → Terminated encolado → unwatch → Terminated recibido: la celda lo descarta (receivedTerminated sólo entrega si está en terminatedQueued).
- **Dependencias**: AC-DW-001
- **Equivalente Rust propuesto**: set `terminated_queued` en la celda
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: watch+unwatch antes de PoisonPill: sólo los monitores vigentes reciben Terminated — DeathWatchSpec.scala:169-193.
  - A2: unwatch entre DWN y procesamiento: el actor no muere (DeathPact) y sigue respondiendo Identify — DeathWatchSpec.scala:268-291.
- **Oráculo**: no aplica (carrera): se reproduce con un dispatcher determinista en Rust.
- **Decisiones**: -
- **Notas**: Contradicción corregida: DeathWatchSpec.scala:257 (cita vieja) es "only notify when watching" (DWN a no-watcher se ignora, ver AC-DW-008), el test de unwatch es :268-291.

### AC-DW-005: watchWith(ref, msg) con mensaje custom

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: watchWith(ref, msg) entrega msg (a través de receive, no como Terminated) cuando el vigilado termina. Re-vigilar con otro modo/mensaje sin unwatch lanza IllegalStateException.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/DeathWatch.scala:51-65,121-131
- **Condiciones** (normales, límites, errores, efectos): Mensaje exacto: "Watch(<self>, <ref>) termination message was not overwritten from [<previous>] to [<newMessage>]. If this was intended, unwatch first before using `watch` / `watchWith` with another message." donde previous/newMessage son Option (None / Some(msg)) (DeathWatch.scala:127-130). watch tras watchWith (o al revés) también lanza porque cambian Some/None. Mismo mensaje (igualdad) no lanza.
- **Orden / concurrencia / ciclo de vida**: al entregar, currentMessage se reemplaza por el mensaje custom (para que stash guarde el custom).
- **Dependencias**: AC-DW-001
- **Equivalente Rust propuesto**: `ctx.watch_with(&ActorRef, msg)`
- **Diferencias observables**: igualdad de mensajes custom (== de Scala) → PartialEq en Rust.
- **Aserciones**:
  - A1: watchWith entrega CustomWatchMsg(subject) — DeathWatchSpec.scala:293-302.
  - A2: el mensaje custom estachado se entrega correctamente (#29101) — DeathWatchSpec.scala:305-316.
  - A3: IllegalStateException con el texto exacto — derivada de DeathWatch.scala:127-130 (no afirmada por spec).
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales): capturar el mensaje de la excepción.
- **Decisiones**: -

### AC-DW-006: Mensaje Terminated (actor, existenceConfirmed, addressTerminated)

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: Terminated(actor)(existenceConfirmed, addressTerminated) es AutoReceivedMessage; sólo llega al receive si sigue en terminatedQueued; su sender es el actor muerto. existenceConfirmed=false si no vino directamente del actor; addressTerminated=true si se derivó de la caída de un nodo.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/Actor.scala:112-141 ; actor/src/main/scala/org/apache/pekko/actor/dungeon/DeathWatch.scala:81-115
- **Condiciones** (normales, límites, errores, efectos): watchedActorTerminated: si no se vigila, ignora; si vigila y no isTerminating: self.tell(Terminated(actor)(…), actor) y terminatedQueuedFor(actor, optionalMessage). Si la celda se está deteniendo no se envía.
- **Orden / concurrencia / ciclo de vida**: ver AC-CELL-008.2 para el orden respecto del padre.
- **Dependencias**: AC-DW-001
- **Equivalente Rust propuesto**: `struct Terminated { actor, existence_confirmed, address_terminated }` con `PartialEq` sólo por `actor`
- **Diferencias observables**: Q-DW-1.
- **Aserciones**:
  - A1: expectTerminationOf recibe WrappedTerminated(Terminated(ref)) — DeathWatchSpec.scala:130-133.
  - A2: Terminated(a)(existenceConfirmed=true, addressTerminated=false) para un hijo parado — ActorSystemSpec.scala:353-355.
  - A3: whenTerminated: Terminated(rootGuardian) con addressTerminated=true y existenceConfirmed=true — ActorSystemSpec.scala:250-261.
  - A4: Terminated(f)(true,false) a un FunctionRef que vigila — FunctionRefSpec.scala:81.
  - A5: equals sólo compara actor — derivada de Actor.scala:123-131 (no afirmada por spec).
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales): Terminated(a)(true,false) == Terminated(a)(false,true).
- **Decisiones**: Q-DW-1
- **Notas**: Q-DW-1: la igualdad de Terminated compara sólo `actor` (segunda lista de parámetros fuera de equals/hashCode de la case class). Terminated es private[pekko] en el constructor y está marcada DeadLetterSuppression, PossiblyHarmful y NoSerializationVerificationNeeded.

### AC-DW-007: Death pact: Terminated no manejado → DeathPactException → Stop

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: Un watcher que no maneja Terminated falla con DeathPactException(dead) ("Monitored actor [<dead>] terminated") y el decider por defecto lo detiene.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/Actor.scala:276-288,650-662
- **Condiciones** (normales, límites, errores, efectos): sender() es el actor muerto.
- **Orden / concurrencia / ciclo de vida**: el DeathPactException aparece como Failed del watcher y luego su supervisor lo detiene y llega Terminated(watcher) a quienes lo vigilan.
- **Dependencias**: AC-CELL-004 ; AC-SUP-004
- **Equivalente Rust propuesto**: -
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: orden observado: Failed(ActorKilledException) por el vigilado, Failed(DeathPactException(failed)) por el hermano, y finalmente Terminated(brother) — DeathWatchSpec.scala:237-245.
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales).
- **Decisiones**: -

### AC-DW-008: El padre siempre recibe la terminación del hijo (DeathWatchNotification) aunque no lo vigile

- **Estado de análisis**: extracted. Visibilidad: internal-required.
- **Contrato observable**: finishTerminate envía DWN(self, true, false) al padre aunque no lo vigile; el padre, en watchedActorTerminated, actualiza el contenedor (handleChildTerminated) si es hijo; si además lo vigila, el Terminated se envía antes (self.tell) y se entrega después.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala:262-266,327-351 ; actor/src/main/scala/org/apache/pekko/actor/dungeon/DeathWatch.scala:99-115
- **Condiciones** (normales, límites, errores, efectos): la terminación libera el nombre; si el contenedor cambia de estado se continúa la acción diferida.
- **Orden / concurrencia / ciclo de vida**: tellWatchersWeDied omite al padre (ya fue notificado por DWN directa).
- **Dependencias**: AC-CELL-015.2
- **Equivalente Rust propuesto**: -
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: recrear hijo "kid" tras recibir Terminated: GREEN dos veces — DeathWatchSpec.scala:248-255.
  - A2: DWN a un actor que no vigila no genera Terminated — DeathWatchSpec.scala:257-266.
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales).
- **Decisiones**: -
- **Notas**: Contradicción corregida: DeathWatchSpec.scala:257-266 afirma que una DWN a un actor que NO vigila no produce Terminated (watchedActorTerminated ignora si watching.get == None), pero aún así actualiza el contenedor si es hijo.

### AC-DW-009: Al pararse, un actor deja de vigilar a todos (sin Terminated a deadLetters)

- **Estado de análisis**: extracted. Visibilidad: internal-required.
- **Contrato observable**: terminate() llama unwatchWatchedActors (envía Unwatch a cada vigilado, vacía watching y terminatedQueued) antes de detener hijos; finishTerminate lo repite al final (por si hubo parada de emergencia).
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala:184-185,268 ; actor/src/main/scala/org/apache/pekko/actor/dungeon/DeathWatch.scala:165-180
- **Condiciones** (normales, límites, errores, efectos): un watchee no InternalActorRef → IllegalStateException("Expected InternalActorRef, but got […]") (no representable).
- **Orden / concurrencia / ciclo de vida**: antes de detener hijos (AC-CELL-008.1).
- **Dependencias**: AC-CELL-008.1
- **Equivalente Rust propuesto**: -
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: derivada de FaultHandling.scala:184-185 (no afirmada por spec).
- **Oráculo**: no aplica.
- **Decisiones**: -
- **Notas**: Sin spec directa. El comentario del código dice "prevent Deadletter(Terminated) messages".

### AC-DW-010: Notificación a watchers al morir (remotos primero, luego locales; el padre se omite)

- **Estado de análisis**: extracted. Visibilidad: internal-required.
- **Contrato observable**: tellWatchersWeDied envía DWN(self, existenceConfirmed=true, addressTerminated=false) a cada watcher (remotos primero, luego locales) salvo al padre; luego vacía watchedBy (en finally) y mantiene la suscripción a AddressTerminated.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/DeathWatch.scala:133-163
- **Condiciones** (normales, límites, errores, efectos): si watchedBy está vacío no hace nada.
- **Orden / concurrencia / ciclo de vida**: remotos antes que locales para que el RemoteDaemon no se cierre antes de sus Terminated.
- **Dependencias**: AC-DW-001
- **Equivalente Rust propuesto**: -
- **Diferencias observables**: sin remote el orden local/remoto es trivial.
- **Aserciones**:
  - A1: 3 watchers, cada uno con un Terminated — DeathWatchSpec.scala:154-167.
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales).
- **Decisiones**: Q-DW-2
- **Notas**: El orden remoto→local es hook para remote (fuera de alcance; Q-DW-2).

### AC-DW-011: Protocolo Watch/Unwatch de sistema (addWatcher/remWatcher)

- **Estado de análisis**: extracted. Visibilidad: internal-required.
- **Contrato observable**: Watch(watchee, watcher) válido sólo si exactamente una parte es self: watchee==self → añade a watchedBy (Debug "now watched by <watcher>"); watcher==self → ejecuta watch(watchee); Unwatch análogo (Debug "no longer watched by <watcher>"). Otras combinaciones → Warning "BUG".
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/DeathWatch.scala:182-215
- **Condiciones** (normales, límites, errores, efectos): añadir un watcher ya presente no repite el Debug.
- **Orden / concurrencia / ciclo de vida**: mensajes de sistema; ver AC-CELL-013.2 (nunca diferidos).
- **Dependencias**: AC-CELL-014
- **Equivalente Rust propuesto**: -
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: Debug "now watched by" y "no longer watched by" — LoggingReceiveSpec.scala:240-251.
- **Oráculo**: no aplica.
- **Decisiones**: -
- **Notas**: Contradicción corregida: el inventario decía que las combinaciones ilegales "se loguean BUG"; el nivel es Warning con textos "BUG: illegal Watch(<watchee>,<watcher>) for <self>" y "BUG: illegal Unwatch(<watchee>,<watcher>) for <self>". Faltaba el Debug "no longer watched by".

### AC-DW-012: AddressTerminated y AddressTerminatedTopic (watch de refs remotos)

- **Estado de análisis**: listed. Visibilidad: internal-required.
- **Contrato observable**: Si hay refs no locales vigilados o vigilantes, la celda se suscribe al topic AddressTerminated; al caer una address: limpia watchedBy con esa address y envía a sí misma DWN(a, existenceConfirmed = (a es hijo), addressTerminated=true) por cada vigilado de esa address. El topic publica AddressTerminated a todos los suscriptores con noSender.
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/dungeon/DeathWatch.scala:217-264 ; actor/src/main/scala/org/apache/pekko/event/AddressTerminatedTopic.scala:30-71
- **Condiciones** (normales, límites, errores, efectos): la suscripción se activa sólo al pasar de 0 a >0 refs no locales y se retira cuando pasa a 0 (maintainAddressTerminatedSubscription).
- **Orden / concurrencia / ciclo de vida**: n/a.
- **Dependencias**: AC-EXT-001
- **Equivalente Rust propuesto**: hook en el trait de ref (`is_local`) + topic opcional
- **Diferencias observables**: Q-DW-2.
- **Aserciones**:
  - A1: topic: subscribe/unsubscribe por CAS; publish tell a todos — derivada de AddressTerminatedTopic.scala:38-71 (no afirmada por spec).
- **Oráculo**: no aplica.
- **Decisiones**: Q-DW-2
- **Notas**: Sigue listed: no leí AddressTerminatedTopicBenchSpec.scala:44 (es un benchmark, no un spec funcional). Fuera de alcance funcional (sin remote) pero conviene dejar el hook. Q-DW-2.

### AC-DW-013: Interacción con Stash: Terminated desestachado vuelve a ser entregable

- **Estado de análisis**: extracted. Visibilidad: public.
- **Contrato observable**: unstashAll devuelve los mensajes al principio del mailbox; si el mensaje es Terminated(ref) se re-registra con terminatedQueuedFor(ref, None) para que se pueda recibir otra vez; un watchWith estachado entrega el mensaje custom (currentMessage reemplazado).
- **Fuente**: actor/src/main/scala/org/apache/pekko/actor/Stash.scala:266-277 ; actor/src/main/scala/org/apache/pekko/actor/dungeon/DeathWatch.scala:81-93
- **Condiciones** (normales, límites, errores, efectos): el mensaje se entrega una vez por unstash.
- **Orden / concurrencia / ciclo de vida**: enqueueFirst: va delante de los demás mensajes del mailbox.
- **Dependencias**: AC-MBX-016
- **Equivalente Rust propuesto**: -
- **Diferencias observables**: ninguna.
- **Aserciones**:
  - A1: actor que estacha Terminated y hace unstashAll lo recibe dos veces ("terminated" ×2) — ActorWithStashSpec.scala:186-190.
  - A2: watchWith estachado entrega CustomWatchMsg — DeathWatchSpec.scala:305-316.
- **Oráculo**: jshell contra Pekko 1.7.1 (jars oficiales).
- **Decisiones**: -

## Descomposiciones

El padre desaparece; los hijos heredan `impl`/`validation` = pending (ninguna de estas filas estaba implementada).

| Padre | Hijos | Motivo |
|---|---|---|
| AC-REF-001 | AC-REF-001.1 (equals/hashCode), .2 (compareTo), .3 (toString) | Tres contratos distintos con fuentes y aserciones distintas; sólo equals/toString tienen spec. |
| AC-REF-007 | AC-REF-007.1 (formas y defaults), .2 (modificadores withX) | withX no tiene ningún spec; las formas de creación sí. |
| AC-REF-009 | AC-REF-009.1 (validación del nombre), .2 (unicidad y reserva), .3 (nombres anónimos) | Tres comportamientos con errores y mensajes distintos. |
| AC-CELL-002 | AC-CELL-002.1 (Create/preStart), .2 (fallos de creación) | El contrato de fallos tiene mensajes y decisiones propios. |
| AC-CELL-007 | AC-CELL-007.1 (Recreate+preRestart), .2 (espera de hijos), .3 (nueva instancia/postRestart/sobrevivientes), .4 (fallo en postRestart) | Secuencia de restart con cuatro fases y estados distintos. |
| AC-CELL-008 | AC-CELL-008.1 (terminate), .2 (finishTerminate) | Dos fases separadas por la espera a los hijos. |
| AC-CELL-013 | AC-CELL-013.1 (prioridad sistema/usuario), .2 (diferimiento por estado), .3 (celda terminada → deadLetters) | Tres reglas independientes del mailbox. |
| AC-CELL-015 | AC-CELL-015.1 (estados y reserva), .2 (transición al terminar el último hijo) | Dos comportamientos con aserciones distintas. |
| AC-SUP-006 | AC-SUP-006.1 (conteo de reintentos), .2 (ventana) | Dos algoritmos con tests propios. |

## Omisiones agregadas

| ID | Qué | Evidencia (archivo/clave de Pekko no cubierto) |
|---|---|---|
| AC-REF-026 | equals/hashCode/toString/toSerializationFormat de ActorSelection | `ActorSelection.scala:143-194`; `ActorSelectionSpec.scala:337-359` |
| AC-REF-027 | VirtualPathContainer y /temp | `ActorRef.scala:763-860`; `ActorRefProvider.scala:539-575` |
| AC-CELL-024 | Limpieza al terminar (terminatedProps, instancia nula) | `ActorCell.scala:388,716-726`; `LocalActorRefProviderSpec.scala:109-129` |
| AC-CELL-025 | `pekko.actor.debug.unhandled` y el reenviador | `reference.conf:754-755`; `Logging.scala:154-170`; `EventStreamSpec.scala:115-131` |
| AC-CELL-026 | Validación del tipo de mailbox exigido en Create | `Dispatch.scala:79-116` |
| AC-CELL-027 | initWithFailure y creación diferida con fallo | `Dispatch.scala:118-129`; `RepointableActorRef.scala:112-121`; `SupervisorSpec.scala:508-531` |
| AC-CELL-028 | `pekko.actor.serialize-messages` | `reference.conf:129-131,138-140`; `Dispatch.scala:171-225` |
| AC-CELL-029 | `pekko.actor.serialize-creators` | `reference.conf:133-136`; `Children.scala:279-306` |
| AC-CELL-030 | Errores al enviar se tragan y se loguean | `Dispatch.scala:137-170,225-228` |
| AC-SUP-018 | Algoritmo de logFailure | `FaultHandling.scala:395-433` |
| AC-SUP-019 | Fallo fatal en restart (Resume→Restart, sin preRestart) | `dungeon/FaultHandling.scala:72-87,104,136-153` |
| AC-SUP-020 | Fallo en handleChildTerminated de la estrategia | `dungeon/FaultHandling.scala:327-351`; `SupervisorMiscSpec.scala:144-163` |

## Contradicciones halladas y corregidas

| Fila | Inventario viejo | Código / tests |
|---|---|---|
| AC-REF-001.3 | toString siempre "Actor[path#uid]" | Sin "#uid" cuando uid == 0 (`ActorRef.scala:188-190`; deadLetters en `LocalActorRefProviderSpec.scala:77`) |
| AC-REF-003 | `ActorModelSpec.scala:311,331` afirman orden por par | :311-344 afirman exclusión mutua y conteo, no orden; el orden sólo se observa en `SupervisorSpec.scala:389-399` |
| AC-REF-012 | ActorNotFound sólo si no existe | `case _ => p.failure(ActorNotFound(this))` incluye timeout y fallos (`ActorSelection.scala:84`) |
| AC-REF-017 | Dos casos (ajena/inexistente) | Un path vacío también → deadLetters (`ActorRefProvider.scala:643-648`) |
| AC-REF-020 | Cita a `RoutingSpec.scala:206` | No leída; el test de `ActorSystemSpec.scala:323-325` es el verificable. Hallazgo: `system.actorOf` es asíncrono (RepointableActorRef) y `context.actorOf` síncrono (`ActorRefProvider.scala:711-713`) |
| AC-CELL-002.2 / AC-SUP-004 | `SupervisorSpec.scala:546` prueba "el decider por defecto detiene" | Ese test usa un decider custom con Restart; la parada por defecto se afirma en `SupervisorSpec.scala:510-514` y `DeathWatchSpec.scala:237-245` |
| AC-CELL-003 | `ActorModelSpec.scala:331` prueba exclusión | :331-344 prueba encolado multihilo; la exclusión está en :311-329 |
| AC-CELL-004 | `EventStreamSpec.scala:115` prueba unhandled() | Prueba el reenviador de `debug.unhandled`; la publicación de UnhandledMessage se observa en `LocalActorRefProviderSpec.scala:70-107` |
| AC-CELL-010 | `ActorRefSpec.scala:465` prueba stop | Prueba `child(name)`; stop está en :422-440 y `SupervisorMiscSpec.scala:102-142` |
| AC-CELL-012 | `ActorRefSpec.scala:442` prueba la parada por defecto | Usa decider custom; la parada por defecto está en `DeathWatchSpec.scala:237-245` |
| AC-CELL-015.1 | "Crear hijo mientras se termina → IllegalStateException" | Sólo con reason == Termination (`Children.scala:314`); durante Recreation/Creation se puede crear |
| AC-CELL-017 | Sólo Undefined apaga | También Duration.Inf (`ReceiveTimeout.scala:46-59`, `ReceiveTimeoutSpec.scala:231-247`) |
| AC-CELL-019 | El contexto es inválido tras terminar | El contexto no se anula tras terminar (`ActorLifeCycleSpec.scala:193-230`) |
| AC-CELL-023 | Líneas FaultHandling.scala:98,203,291 | Las líneas reales son 99, 206, 271, 290; faltaba "no longer watched by" (`DeathWatch.scala:207`) |
| AC-SUP-002 | `UidClashTest.scala:104` prueba el descarte de Failed por uid | Prueba que un Terminated de un hijo viejo no llega (DeathWatch) (`UidClashTest.scala:103-111`) |
| AC-SUP-005 | `SupervisorSpec.scala:292` prueba stoppingStrategy | Prueba AllForOne sin reintentos; stopping está en `ActorSystemSpec.scala:336-357` |
| AC-SUP-008 | `SupervisorHierarchySpec.scala:790` prueba Escalate | Prueba Terminated tras max reintentos; Escalate está en :767-788 y `ActorSystemSpec.scala:359-375` |
| AC-DW-001 | "exactamente un Terminated aunque vigile varias veces" afirmado por spec | Ningún spec lo afirma; se deriva de `DeathWatch.scala:37-45` |
| AC-DW-002 | existenceConfirmed=false sólo para EmptyLocal/deadLetters | También para un LocalActorRef muerto, vía deadLetterMailbox (`Mailboxes.scala:54-72`, `ActorRef.scala:746-752`) |
| AC-DW-004 | `DeathWatchSpec.scala:257` prueba unwatch | :257 es "only notify when watching"; unwatch está en :268-291 |
| AC-DW-005 | El spec afirma IllegalStateException al re-vigilar | Ningún spec lo afirma; mensaje derivado de `DeathWatch.scala:127-130` |
| AC-DW-011 | Combinaciones ilegales "se loguean BUG" | Nivel Warning con texto "BUG: illegal Watch/Unwatch(…)" (`DeathWatch.scala:194-195,212-213`) |

## Filas que siguen listed

| Fila | Motivo |
|---|---|
| AC-REF-003 | No leí `message-delivery-reliability.md:41-42,108-159` ni `MailboxConfigSpec.scala:77`; el orden por par sólo lo observo en `SupervisorSpec.scala:389-399`. |
| AC-REF-008 | No leí `IndirectActorProducer.scala`; la validación de firma de constructor (`Props.validate`, `AbstractProps.scala`) tampoco. |
| AC-REF-014 | No leí `util/Glob.scala` ni `GlobSpec.scala:33` (unidad UTIL); la parte de deliverSelection está leída. |
| AC-REF-020 | No leí `RoutingSpec.scala:206`; sólo hay un test indirecto. |
| AC-REF-025 | Sólo leí los trait; no el uso desde typed (Q-X-8). |
| AC-REF-027 | No leí el cuerpo de `VirtualPathContainer` (`ActorRef.scala:763-860`). |
| AC-CELL-019 | Sólo leí los comentarios de `self`, `props` y `receiveTimeout` del ActorContext (`ActorCell.scala:60-104`). |
| AC-CELL-028 | No leí `SerializationCheckFailedException` ni la extensión de serialización (unidad SER). |
| AC-DW-012 | `AddressTerminatedTopicBenchSpec.scala:44` es un benchmark que no leí; el código sí. |

## Cobertura

Archivos de Pekko del área classic-cell y filas que los cubren. "Leído" = cuerpo completo o rangos citados; el resto se declara.

| Archivo (relativo a `upstream/pekko/`) | Filas |
|---|---|
| `actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala` | AC-REF-001.1-.3, 002, 004, 005, 015, 016, 018, 019, 021, 022, 023, 024, 027 |
| `actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala` | AC-CELL-001..005, 016, 017, 018, 019, 023, 024; AC-REF-005 |
| `actor/src/main/scala/org/apache/pekko/actor/Actor.scala` | AC-CELL-001, 004, 006, 011, 012, 022; AC-REF-006, 013, 024; AC-SUP-017; AC-DW-006, 007 |
| `actor/src/main/scala/org/apache/pekko/actor/FaultHandling.scala` | AC-SUP-003..008, 012, 013, 018; AC-SUP-014 |
| `actor/src/main/scala/org/apache/pekko/actor/Props.scala` | AC-REF-007.1, 007.2 |
| `actor/src/main/scala/org/apache/pekko/actor/AbstractProps.scala` | no leído (Props.create y Props.validate): ver AC-REF-007.1/008 |
| `actor/src/main/scala/org/apache/pekko/actor/IndirectActorProducer.scala` | AC-REF-008 (no leído) |
| `actor/src/main/scala/org/apache/pekko/actor/ActorSelection.scala` | AC-REF-011, 012, 013, 014, 026 |
| `actor/src/main/scala/org/apache/pekko/actor/RepointableActorRef.scala` | AC-REF-020; AC-CELL-027 |
| `actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala` | AC-REF-009.1, 011, 016, 017, 022, 027; AC-SUP-014; AC-CELL-027 (creación de actores y RouterConfig: unidad RT para routers) |
| `actor/src/main/scala/org/apache/pekko/actor/ClassicActorSystemProvider.scala` | AC-REF-025 |
| `actor/src/main/scala/org/apache/pekko/actor/Stash.scala` | AC-DW-013 (el resto: unidad MBX) |
| `actor/src/main/scala/org/apache/pekko/actor/dungeon/Children.scala` | AC-REF-009.1-.3, 010, 021; AC-CELL-010, 015.1, 015.2, 016, 029 |
| `actor/src/main/scala/org/apache/pekko/actor/dungeon/ChildrenContainer.scala` | AC-REF-009.2, 010; AC-CELL-007.2, 015.1, 015.2 |
| `actor/src/main/scala/org/apache/pekko/actor/dungeon/DeathWatch.scala` | AC-DW-001..012 |
| `actor/src/main/scala/org/apache/pekko/actor/dungeon/Dispatch.scala` | AC-REF-002; AC-CELL-002.1, 026, 027, 028, 030 |
| `actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala` | AC-CELL-007.1-.4, 008.1-.2, 009, 020; AC-SUP-001, 002, 008..011, 015, 016, 019, 020 |
| `actor/src/main/scala/org/apache/pekko/actor/dungeon/ReceiveTimeout.scala` | AC-CELL-017 |
| `actor/src/main/scala/org/apache/pekko/actor/dungeon/TimerSchedulerImpl.scala` | no define comportamiento de este grupo: pertenece a Timers (unidad SCH) |
| `actor/src/main/scala/org/apache/pekko/dispatch/sysmsg/SystemMessage.scala` | AC-CELL-013.2, 014 |
| `actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala` (partes: run, processAllSystemMessages, suspend/resume, cleanUp) | AC-CELL-013.1, 013.3; AC-SUP-010 (el resto: unidad MBX) |
| `actor/src/main/scala/org/apache/pekko/dispatch/Mailboxes.scala` (deadLetterMailbox) | AC-CELL-013.3; AC-DW-002 |
| `actor/src/main/scala/org/apache/pekko/dispatch/AbstractDispatcher.scala` (Envelope, unregister, suspend/resume) | AC-REF-002; AC-CELL-008.2; AC-SUP-010 (el resto: unidad DSP) |
| `actor/src/main/scala/org/apache/pekko/event/AddressTerminatedTopic.scala` | AC-DW-012 |
| `actor/src/main/java/org/apache/pekko/actor/AbstractActorRef.java` | no define comportamiento observable: sólo offsets de `Unsafe` para los campos `_cellDoNotCallMeDirectly`/`_lookupDoNotCallMeDirectly` de RepointableActorRef (boilerplate de JVM) |
| `actor/src/main/java/org/apache/pekko/actor/dungeon/AbstractActorCell.java` | no define comportamiento observable: offsets de `Unsafe` para children/functionRefs/nextName/mailbox (boilerplate de JVM; no leído, inferido por los accesos en `Children.scala`) |
| `actor/src/main/scala-2.12/**`, `scala-2.13/**`, `scala-3/**` (compat) | no define comportamiento observable de este grupo: shims de compilación |
| `actor/src/main/resources/reference.conf` claves `pekko.actor.provider`, `guardian-supervisor-strategy`, `creation-timeout`, `serialize-messages`, `serialize-creators`, `no-serialization-verification-needed-class-prefix`, `unstarted-push-timeout`, `debug.lifecycle`, `debug.autoreceive`, `debug.unhandled` | AC-SUP-014 (guardian-supervisor-strategy), AC-CELL-028 (serialize-messages), AC-CELL-029 (serialize-creators), AC-REF-020 (unstarted-push-timeout), AC-CELL-023 (debug.lifecycle, debug.autoreceive), AC-CELL-025 (debug.unhandled). `provider` y `creation-timeout` pertenecen a SYS/CFG y no definen comportamiento propio de este grupo. `debug.receive`, `fsm`, `event-stream` y `router-misconfiguration` pertenecen a EVT, FSM y RT. |
| Tests citados (`actor-tests/src/test/scala/org/apache/pekko/actor/*Spec.scala`) | Leídos: ActorLifeCycleSpec, DeathWatchSpec, HotSwapSpec, Ticket669Spec, SupervisorMiscSpec, SupervisorTreeSpec, RestartStrategySpec, FunctionRefSpec, UidClashTest, DeadLetterSupressionSpec, ForwardActorSpec, ReceiveTimeoutSpec, ActorSelectionSpec, SupervisorSpec (completo), ActorRefSpec (:140-175,270-530), LocalActorRefProviderSpec, ActorSystemSpec (:155-261,320-375), ConsistencySpec, PropsCreationSpec, SupervisorHierarchySpec (:760-935), LoggingReceiveSpec (:180-300), EventStreamSpec (:105-135), ActorWithStashSpec (:83-98,170-192), AskSpec (:45-66), SystemMessageListSpec (:24-75), ActorModelSpec (:305-345,418-480). No leídos: RoutingSpec, MailboxConfigSpec, GlobSpec, AddressTerminatedTopicBenchSpec. |
