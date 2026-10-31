# Contratos verificables: grupo classic-io-ser (SER, IO, JAPI)

Revisión del inventario de `pekko-actor` 1.7.1 (commit `9a54d129…`) para las unidades **SER** (serialización local), **IO** (`pekko.io`: TCP, UDP, UDP conectado, DNS) y **JAPI** (API Java). Solo documentación: no hay código Rust. Todas las filas siguen en alcance con `impl=pending` y `validation=pending`; las exclusiones son decisiones `proposed` (ver `classic-io-ser.decisions.tsv`). Las citas `path:línea` son relativas a `upstream/pekko`; las de rangos se contrastaron con `sed -n` sobre los archivos fijados.

Convenciones: "derivada del código" = ninguna spec afirma el comportamiento (no afirmada por spec); "oráculo" = comparación posible contra los jars oficiales con `tools/oracle/run.sh` (ningún escenario nuevo se escribió ni se ejecutó en esta revisión). Los IDs `Q-X-n` son los transversales del orquestador y no se redefinen aquí.

Resumen: 77 filas (62 `extracted`, 15 `listed`). Decisiones: ver al final.


### AC-SER-001.1 — Resolución de serializador por clase: serializerFor/findSerializerFor, bindings ordenados de más específico a menos, desempate, null -> NullSerializer

**Contrato observable.** Dado un tipo de mensaje, devuelve siempre el mismo serializador (resultado cacheado en serializerMap). Se elige entre los bindings cuya clase configurada es supertipo (isAssignableFrom) del tipo pedido; los bindings se ordenan con subtipos antes que supertipos, y entre no relacionados se respeta el orden de configuración. Si hay varios candidatos: si el primero es subtipo de todos o todos usan la misma instancia de serializador, gana el primero; si no, se descartan JavaSerializer/DisabledJavaSerializer y gana el primero restante, con un warning "Multiple serializers found" si aún no son unívocos. findSerializerFor(null) devuelve NullSerializer.

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:314-315 (findSerializerFor) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:326-390 (serializerFor (unique 330-336, filtro 338-343, desempate 344-365, cache 367-390)) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:507-520 (sort); doc pública docs/src/main/paradox/serialization.md:36-60 ; reference.conf actor/src/main/resources/reference.conf:810-831

**Condiciones.** Normal: caché hit tras la primera resolución. Límite: dos marcadores no relacionados -> warning y se elige el primero configurado. Error: sin candidatos -> java.io.NotSerializableException("No configured serialization-bindings for class [<FQCN>]") (Serialization.scala:342); solo JavaSerializer entre varios y todos descartados -> NotSerializableException("More than one JavaSerializer configured for class [<FQCN>]") (:356, "shouldn't happen"). Efectos: log warning (marker Security) "Using the Java serializer for class [..]..." si aplica (ver AC-SER-004.2); debug "Using serializer [..] for message [..]".

**Orden / concurrencia / ciclo de vida.** Resolución thread-safe: ConcurrentHashMap.putIfAbsent; dos hilos pueden calcular a la vez pero solo el que inserta primero loguea; ambos devuelven la misma instancia. Sin mensajes de sistema.

**Dependencias.** AC-SER-001.2 (bindings), AC-SER-004.2 (avisos Java), AC-EVT (log con marker), Q-X-6

**Equivalente Rust propuesto.** `trait MessageType` registrado por `TypeKey`; `SerializerRegistry` guarda `Vec<(TypeKey, Arc<dyn Serializer>)>` ya ordenado con la relación declarada `supertypes_of(TypeKey)` y una caché `DashMap<TypeKey, Arc<dyn Serializer>>`; error `NotSerializableError { message }` con el texto exacto.

**Diferencias observables.** Sin herencia/interfaces JVM, "isAssignableFrom" se reemplaza por supertipos declarados al registrar; un tipo Rust no registrado nunca coincide con "java.io.Serializable" ni con nombres de clases JVM (los nombres FQCN de reference.conf no tienen equivalente). Mensajes citan el nombre de tipo Rust, no el FQCN JVM.

**Aserciones.**
- A1. serializerFor(SimpleMessage) -> NoopSerializer (binding por interfaz Marker) — SerializeSpec.scala:191-193
- A2. serializerFor(ExtendedSimpleMessage) -> NoopSerializer (interfaz heredada de la superclase) — SerializeSpec.scala:195-197
- A3. serializerFor(AnotherMessage) y ExtendedAnotherMessage -> NoopSerializer (interfaz indirecta) — SerializeSpec.scala:199-205
- A4. serializerFor(PlainMessage) y ExtendedPlainMessage -> NoopSerializer — SerializeSpec.scala:207-213
- A5. JavaSerializer pierde ante otro serializador cuando hay varios bindings (BothTestSerializableAndJavaSerializable -> Noop) — SerializeSpec.scala:215-217
- A6. Varios serializadores aplicables y no unívocos -> exactamente 1 warning que empieza con "Multiple serializers found"; el resultado es Noop o Noop2 — SerializeSpec.scala:219-224
- A7. Orden de bindings: A -> DisabledJavaSerializer, B -> Noop, C (extiende B y A) -> Noop — SerializeSpec.scala:226-231
- A8. Más específico primero: A -> Disabled, D (extiende A) -> Noop, E (extiende D) -> Noop — SerializeSpec.scala:233-237
- A9. Clase sin binding (Actor) -> java.io.NotSerializableException — SerializeSpec.scala:239-243
- A10. findSerializerFor(null) -> NullSerializer — derivada del código Serialization.scala:314-315; no afirmada por spec
- A11. Texto "No configured serialization-bindings for class [..]" y "More than one JavaSerializer configured.." — derivada del código Serialization.scala:342,356; no afirmada por spec (los tests solo verifican el tipo de excepción)

**Oráculo.** Jar oficial pekko-actor 1.7.1 con jshell (tools/oracle/run.sh): se puede crear un ActorSystem con bindings propios y consultar serializerFor sobre jerarquías Java/Scala para fijar el orden. Escenario NO existe aún ni se ejecutó en esta revisión. La parte "hay herencia JVM" no es comparable con Rust.

**Decisiones.** Q-SER-1, Q-X-2, Q-X-6

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-SER-001.2 — Carga de serializers y bindings desde config y SerializationSetup: alias -> instancia, "none", ids duplicados, construcción por system/vacío/bindingName

**Contrato observable.** Al construir la extensión se leen pekko.actor.serializers (alias -> FQCN) y pekko.actor.serialization-bindings (FQCN de clase -> alias). Cada serializador se crea probando tres constructores en orden: (ExtendedActorSystem), (), (ExtendedActorSystem, bindingName); la variante sin argumentos de la API pública serializerOf(fqcn) re-lanza NoSuchMethodException. Un binding con valor "none" se omite. Los serializadores programáticos (SerializationSetup) se suman a los de config y sus useFor añaden bindings. Los ids se indexan en serializerByIdentity (más NullSerializer id 0).

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:53-64 (Settings) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:397-427 (serializerOf (3 constructores 405-424)) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:431-446 (serializerDetails) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:448-454 (serializers) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:459-473 (bindings ("none" y protobuf :462)) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:475-479 (ensureOnlyAllowedSerializers) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:498-505 (checkGoogleProtobuf) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:528-545 (serializerByIdentity (mensaje id duplicado 535-538)); actor/src/main/resources/reference.conf:799-848

**Condiciones.** Normal: alias -> serializador; cualquier alias con valor "none" se ignora (Serialization.scala:462). Errores: id duplicado -> IllegalArgumentException("Serializer identifier [<id>] of [<clase>] is not unique. It is also used by [<clase>].") (:535-538); clase de serializador o binding inexistente -> ClassNotFoundException (propaga del dynamicAccess; spec SerializationSetupSpec.scala:123-135); alias de binding que no existe en serializers -> NoSuchElementException de Map (serializers(alias), :463, derivado del código); allow-java-serialization=off y alguna instancia de JavaSerializer -> IllegalArgumentException("requirement failed: Disallowed JavaSerializer binding.") (:477, ver AC-SER-004.2). Bindings cuyo className empieza por "com.google.protobuf" solo se aceptan si la clase carga (:498-505).

**Orden / concurrencia / ciclo de vida.** Todo ocurre en el constructor de Serialization, durante registerExtension de SerializationExtension (ver AC-SER-014); un serializador no puede pedir SerializationExtension en su constructor.

**Dependencias.** AC-SER-014 (extensión), AC-SER-005 (setup programático), AC-CFG-006/016 (DynamicAccess y config), AC-SER-004.2

**Equivalente Rust propuesto.** `SerializerFactory = Arc<dyn Fn(&SerializerContext) -> Result<Arc<dyn Serializer>>>` registrado por nombre; el alias de config referencia el nombre registrado; `SerializationConfigError::{UnknownSerializer(name), DuplicateIdentifier{id, first, second}, ..}`.

**Diferencias observables.** No hay reflexión: "john.is.not.here" produce un error de config con nombre, no ClassNotFoundException (Q-X-2). Los tres constructores se reducen a una fábrica que recibe contexto (sistema + bindingName). Los bindings por FQCN JVM ("java.lang.String", "[B", "scala.Long") se convierten en entradas por TypeKey (Q-SER-1).

**Aserciones.**
- A1. Dos serializadores con el mismo id (9999) hacen fallar ActorSystem con IllegalArgumentException cuyo mensaje cumple regex "Serializer identifier \[9999\].*is not unique" — SerializeSpec.scala:283-304
- A2. Serializador con clase inexistente en pekko.actor.serializers -> ClassNotFoundException al crear el sistema — SerializationSetupSpec.scala:123-135
- A3. ser.bindings contiene (Address -> DisabledJavaSerializer) y (PlainMessage -> NoopSerializer) con la config de prueba — SerializeSpec.scala:171-176
- A4. Orden de constructores (system, vacío, system+bindingName) y alias "none" — derivada del código Serialization.scala:405-424,462; no afirmada por spec
- A5. Binding a alias inexistente falla con NoSuchElementException de Map — derivada del código Serialization.scala:463; no afirmada por spec

**Oráculo.** Jar oficial vía jshell: crear ActorSystem con serializers/bindings inválidos y capturar la excepción (tipo+mensaje). Escenario no existe ni se ejecutó.

**Decisiones.** Q-SER-1, Q-X-2

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-SER-001.3 — Serialization.serialize/deserialize: por id+manifest, por clase y por ByteBuffer; cache de manifest; errores de id/manifest desconocidos

**Contrato observable.** serialize(o) = findSerializerFor(o).toBinary(o) envuelto en Try y con la información de transporte fijada. deserialize(bytes, serializerId, manifest): busca el serializador por id (array rápido para ids 0..1023, mapa para el resto); si es SerializerWithStringManifest usa el manifest como String; si no, con manifest "" llama fromBinary(bytes, None) y con manifest no vacío resuelve la clase por nombre (cacheada en manifestCache) y llama fromBinary(bytes, Some(clase)). deserializeByteBuffer usa ByteBufferSerializer si el serializador lo implementa, si no copia a array. deserialize(bytes, clazz) usa serializerFor(clazz).

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:170-177 (withTransportInformation) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:183-187 (serialize) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:194-209 (deserialize(clazz opt, deprecado)) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:216-227 (deserialize(id, manifest)) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:229-262 (deserializeByteArray (manifest 241-258)) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:269-296 (deserializeByteBuffer) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:302-306 (deserialize(clazz)) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:547-566 (tabla rápida/getSerializerById)

**Condiciones.** Errores (NotSerializableException salvo indicación): "Cannot find serializer with id [<id>] (manifest [<manifest>]). The most probable reason is that the configuration entry pekko.actor.serializers is not in sync between the two systems." (:222-224) y en deserializeByteBuffer el mismo texto con "is not in synch" (typo real, :276-277); variante deprecada: "Cannot find serializer with id [<id>] (class [<FQCN>]). The most probable..." (:201-204); manifest-clase inexistente: "Cannot find manifest class [<manifest>] for serializer with id [<id>]." (:254-255). Límite: nesting depth ver AC-SER-009. deserialize/serialize devuelven Failure en lugar de lanzar (Try); deserializeByteBuffer lanza.

**Orden / concurrencia / ciclo de vida.** manifestCache es un AtomicReference con CAS reintentado; sin orden entre hilos. withTransportInformation restaura el valor anterior en finally.

**Dependencias.** AC-SER-001.1, AC-SER-002 (Serializers.manifestFor), AC-SER-007.2, AC-SER-009

**Equivalente Rust propuesto.** `fn deserialize(&self, bytes: &[u8], serializer_id: i32, manifest: &str) -> Result<Box<dyn Any + Send>, SerializationError>`; `SerializationError::NotSerializable(String)` conserva el texto.

**Diferencias observables.** Try -> Result. El manifest de clase (serializadores sin manifest-string) se resuelve por registro nombre -> TypeKey (Q-SER-1) en lugar de Class.forName. El typo "synch" se puede conservar o corregir: decisión de texto, marcado en Q-SER-1.

**Aserciones.**
- A1. serialize -> findSerializerFor -> Serializers.manifestFor -> deserialize reproduce el objeto (ProgrammaticDummy con FakeSerializer) — SerializationSetupSpec.scala:103-109,195-196
- A2. Deserializar con id de JavaSerializer en un sistema con Java deshabilitado lanza DisabledJavaSerializer.JavaSerializationException — SerializationSetupSpec.scala:199-212
- A3. serialize(f) y deserialize(bytes, serializer.identifier, "") para una función con JavaSerializer devuelve f2(3)==4 — SerializeSpec.scala:437-444
- A4. Textos exactos de "Cannot find serializer with id" y "Cannot find manifest class" — derivada del código Serialization.scala:201-204,222-224,254-255,276-277; no afirmada por spec
- A5. Caché de manifest y tabla rápida 0..1023 — derivada del código Serialization.scala:229-262,547-566; no afirmada por spec

**Oráculo.** Jar oficial: serialize/deserialize con ids/manifests inválidos en jshell para fijar textos. Escenario no existe ni se ejecutó.

**Decisiones.** Q-SER-1, Q-SER-3

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-SER-002 — SPI Serializer (identifier, toBinary, fromBinary, includeManifest), SerializerWithStringManifest, BaseSerializer, ByteBufferSerializer, Serializers.manifestFor

**Contrato observable.** Serializer: identifier, toBinary(AnyRef), includeManifest, fromBinary(bytes, Option[Class]) (lanza NotSerializableException si no entiende). SerializerWithStringManifest: includeManifest siempre true, manifest(o): String, fromBinary(bytes, manifest: String); la versión con Option[Class] pasa c.getName o "". ByteBufferSerializer añade toBinary(o, buf) y fromBinary(buf, manifest). BaseSerializer exige system y su identifier sale de config pekko.actor.serialization-identifiers."<FQCN o alias>". Serializers.manifestFor(s, msg): manifest() si es StringManifest; si no, msg.getClass.getName cuando includeManifest, si no "".

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/Serializer.scala:52-89 (Serializer) ; actor/src/main/scala/org/apache/pekko/serialization/Serializer.scala:91-99 (Serializers.manifestFor) ; actor/src/main/scala/org/apache/pekko/serialization/Serializer.scala:128-174 (SerializerWithStringManifest (includeManifest 136, adaptación 166-172)) ; actor/src/main/scala/org/apache/pekko/serialization/Serializer.scala:211-225 (ByteBufferSerializer) ; actor/src/main/scala/org/apache/pekko/serialization/Serializer.scala:232-262 (BaseSerializer) ; actor/src/main/scala/org/apache/pekko/serialization/Serializer.scala:263-284 (BaseSerializer.identifierFromConfig) ; actor/src/main/resources/reference.conf:833-848

**Condiciones.** Error: identifier ausente en config -> ConfigException.Missing al construir (getInt, :278 y :283, derivado del código). fromBinary con manifest desconocido: recomendado lanzar NotSerializableException (Scaladoc :152-162): remoting lo trata como problema transitorio (se loguea y se descarta el mensaje) y otras excepciones cierran la conexión TCP.

**Orden / concurrencia / ciclo de vida.** Los arrays devueltos por toBinary no deben mutarse después (Scaladoc :63). Sin concurrencia propia: los serializadores deben ser thread-safe porque se comparten.

**Dependencias.** AC-SER-001.2, AC-CFG (getInt de identifiers)

**Equivalente Rust propuesto.** Traits `Serializer`, `ManifestSerializer` (manifest: &str) y `BufferSerializer` (con `bytes::BufMut`); helper `Serializers::manifest_for`; `BaseSerializer::identifier_from_config(system, name)`.

**Diferencias observables.** fromBinary(Option[Class]) usa un TypeKey registrado (Q-SER-1). ByteBuffer -> BufMut/Buf. Los ids reservados 0-40 se mantienen para compatibilidad de cable con Pekko si se quisiera interoperar (no exigido).

**Aserciones.**
- A1. ByteArraySerializer.toBinary(a) y fromBinary devuelven la MISMA instancia para "foo".getBytes, null y Array vacío — SerializeSpec.scala:245-250
- A2. ByteArraySerializer.toBinary("pigdog") -> IllegalArgumentException con mensaje "<FQCN ByteArraySerializer> only serializes byte arrays, not [java.lang.String]" — SerializeSpec.scala:252-255
- A3. ByteBuffer little-endian: toBinary(payload, buf) avanza position hasta payload.length; fromBinary(buf,"") consume todo (remaining==0) y devuelve los mismos bytes; String en toBinary(.., buf) lanza el mismo IllegalArgumentException — SerializeSpec.scala:258-275
- A4. Para tipos primitivos, el formato en Array y en ByteBuffer (LITTLE_ENDIAN) es idéntico — PrimitivesSerializationSpec.scala:48-62
- A5. Serializers.manifestFor / SerializerWithStringManifest.fromBinary(Option[Class]) — derivada del código Serializer.scala:94-97,166-172; no afirmada por spec salvo uso indirecto en SerializationSetupSpec.scala:105-108

**Oráculo.** Jar oficial: la salida de toBinary de cada serializador incluido (bytes) se puede volcar con jshell y fijar como golden; provenance jar pekko-actor 1.7.1. Escenario no existe ni se ejecutó.

**Decisiones.** Q-X-3, Q-SER-1

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-SER-003.1 — NullSerializer (id 0) y ByteArraySerializer (id 4, binding "[B")

**Contrato observable.** NullSerializer: identifier 0, toBinary = array vacío compartido, fromBinary = null. ByteArraySerializer: toBinary(null)=null, toBinary(Array)=mismo array, otro tipo -> IllegalArgumentException; fromBinary devuelve los mismos bytes; soporta ByteBuffer.

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/Serializer.scala:420-430 (NullSerializer) ; actor/src/main/scala/org/apache/pekko/serialization/Serializer.scala:436-465 (ByteArraySerializer); actor/src/main/resources/reference.conf:802,818,841

**Condiciones.** toBinary(otro) -> IllegalArgumentException("<FQCN> only serializes byte arrays, not [<clase>]") (:443-444 y :455-456). toBinary(null) devuelve null (no array vacío) en ByteArraySerializer (:440).

**Orden / concurrencia / ciclo de vida.** n/a: sin estado.

**Dependencias.** AC-SER-002

**Equivalente Rust propuesto.** `NullSerializer` (id 0) y `BytesSerializer` (id 4, tipo `Vec<u8>`/`Bytes`)

**Diferencias observables.** "Misma instancia" (theSameInstanceAs) se traduce a "mismo buffer sin copia" si el tipo es `Bytes`; con `Vec<u8>` movido no hay copia. El nombre de clase en el mensaje cambia al nombre Rust.

**Aserciones.**
- A1. serializerFor(Array[Byte]) es ByteArraySerializer; roundtrip devuelve la misma instancia para bytes, null y vacío — SerializeSpec.scala:245-250
- A2. Mensaje de error con String — SerializeSpec.scala:252-255
- A3. Defaults de reference.conf: Array[Byte] -> ByteArraySerializer — SerializeSpec.scala:376-378
- A4. NullSerializer.fromBinary devuelve null; id 0 — derivada del código Serializer.scala:426-429; no afirmada por spec

**Oráculo.** Jar oficial: volcar bytes/ids. Escenario no existe ni se ejecutó.

**Decisiones.** -

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-SER-003.2 — LongSerializer (id 18) e IntSerializer (id 19): formato little-endian de ancho fijo

**Contrato observable.** toBinary(Long): array de 8 bytes con los bytes bajos primero (little-endian); fromBinary(bytes) reconstruye desde bytes(7) a bytes(0). toBinary(buf) usa buf.putLong con el orden del buffer (el spec fija LITTLE_ENDIAN). Int análogo con 4 bytes. Binding: java.lang.Long y scala.Long -> primitive-long; java.lang.Integer y scala.Int -> primitive-int.

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/PrimitiveSerializers.scala:27-64 (LongSerializer (toBinary 42-52, fromBinary 54-63)) ; actor/src/main/scala/org/apache/pekko/serialization/PrimitiveSerializers.scala:69-102 (IntSerializer); actor/src/main/resources/reference.conf:803-804,825-828,843-844

**Condiciones.** fromBinary con menos de 8 (4) bytes -> ArrayIndexOutOfBoundsException (derivado del código :57-60; no afirmado). toBinary de un objeto no Long -> ClassCastException/NullPointerException de Long.unbox (derivado).

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AC-SER-002, AC-SER-003.5

**Equivalente Rust propuesto.** `I64Serializer { id: 18 }`, `I32Serializer { id: 19 }` con `to_le_bytes`/`from_le_bytes` y variantes sobre `BufMut`.

**Diferencias observables.** Se pierde la distinción java.lang.Long vs scala.Long (un solo i64). Entradas mal tipadas devuelven `Err` en lugar de ClassCastException/AIOOBE.

**Aserciones.**
- A1. Para cada valor de {0,1,-1,Long.MinValue,Long.MinValue+1,Long.MaxValue,Long.MaxValue-1}: serializerFor(Long) es LongSerializer; fromBinary(toBinary(v))==v; mismos bytes Array/ByteBuffer LE y roundtrip por ByteBuffer — PrimitivesSerializationSpec.scala:64-79
- A2. Idem para Int con {0,1,-1,MinValue,MinValue+1,MaxValue,MaxValue-1} — PrimitivesSerializationSpec.scala:88-102
- A3. Id 18 y 19 por reference.conf — derivada del código actor/src/main/resources/reference.conf:843-844; la aserción del spec (PrimitivesSerializationSpec.scala:81-84,104-107) no es una aserción efectiva: usa === sin should/assert, y consulta el id de Long
- A4. Bytes LE concretos (p. ej. 1L -> 01 00 00 00 00 00 00 00) — derivada del código PrimitiveSerializers.scala:42-52; no afirmada por spec salvo igualdad Array/ByteBuffer

**Oráculo.** Jar oficial: volcar toBinary de los valores del spec; golden de bytes. Escenario no existe ni se ejecutó.

**Decisiones.** Q-SER-1

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-SER-003.3 — StringSerializer (id 20) y ByteStringSerializer (id 21)

**Contrato observable.** String: toBinary = bytes UTF-8; fromBinary = new String(bytes, UTF-8). ByteString: toBinary copia a array; fromBinary = ByteString(bytes) (copia); toBinary(buf) copia al buffer y lanza BufferOverflowException si no cabe.

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/PrimitiveSerializers.scala:107-127 (StringSerializer) ; actor/src/main/scala/org/apache/pekko/serialization/PrimitiveSerializers.scala:132-161 (ByteStringSerializer (overflow 143-144)); actor/src/main/resources/reference.conf:805-806,821-824

**Condiciones.** Buffer sin espacio suficiente -> java.nio.BufferOverflowException (:143-144; derivado, no afirmado). Bytes inválidos UTF-8 se reemplazan por U+FFFD (comportamiento del constructor String de JVM; derivado, no afirmado).

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AC-SER-002, AC-UTIL ByteString

**Equivalente Rust propuesto.** `StringSerializer` (id 20), `ByteStringSerializer` (id 21) sobre `bytes::Bytes` o el `ByteString` portado en AC-UTIL.

**Diferencias observables.** UTF-8 inválido: String::from_utf8_lossy reproduce el reemplazo U+FFFD; decisión: usar lossy para igualar. BufferOverflowException -> Err(BufferOverflow).

**Aserciones.**
- A1. Para "", "hello", "árvíztűrőütvefúrógép" y un String aleatorio: serializerFor(String)==StringSerializer y roundtrip Array y ByteBuffer idéntico — PrimitivesSerializationSpec.scala:132-148
- A2. Para ByteString vacío, simple, concatenado, slice y largo(2000 bytes) concatenado: serializerFor(clase concreta)==ByteStringSerializer y roundtrip — PrimitivesSerializationSpec.scala:157-178
- A3. Ids 20 y 21 — derivada del código actor/src/main/resources/reference.conf:845-846; la aserción del spec (PrimitivesSerializationSpec.scala:150-153,180-183) no es efectiva (=== sin assert)
- A4. BufferOverflowException en ByteString.toBinary(buf) sin espacio — derivada del código PrimitiveSerializers.scala:143-144; no afirmada por spec

**Oráculo.** Jar oficial: golden de bytes UTF-8 y ByteString. Escenario no existe ni se ejecutó.

**Decisiones.** Q-SER-1

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-SER-003.4 — BooleanSerializer (id 35): 1 byte, 0=false 1=true, rechaza otros valores

**Contrato observable.** toBinary: TRUE->1, FALSE->0, otro objeto -> IllegalArgumentException("Non boolean flag: <o>"); fromBinary: 1 -> TRUE, 0 -> FALSE, otro -> IllegalArgumentException("Non boolean flag byte: <b>") (se lee solo bytes(0); un array vacío falla con AIOOBE).

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/PrimitiveSerializers.scala:166-214 ((toBinary 196-205, fromBinary 207-213, buffer 179-194)); actor/src/main/resources/reference.conf:807,829-830,847

**Condiciones.** Mensajes exactos: "Non boolean flag: <objeto>" (:183,:200) y "Non boolean flag byte: <byte>" (:192,:211) (derivado; los tests no los comprueban).

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AC-SER-002

**Equivalente Rust propuesto.** `BoolSerializer { id: 35 }`

**Diferencias observables.** scala.Boolean y java.lang.Boolean se unifican en bool.

**Aserciones.**
- A1. Para false,true,Boolean.FALSE,Boolean.TRUE: serializerFor es BooleanSerializer; roundtrip Array y ByteBuffer — PrimitivesSerializationSpec.scala:110-124
- A2. Id 35 — derivada del código actor/src/main/resources/reference.conf:847; aserción del spec (PrimitivesSerializationSpec.scala:126-129) no efectiva
- A3. Mensajes "Non boolean flag" / "Non boolean flag byte" — derivada del código PrimitiveSerializers.scala:183,192,200,211; no afirmada por spec

**Oráculo.** Jar oficial: volcar bytes y errores. Escenario no existe ni se ejecutó.

**Decisiones.** -

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-SER-003.5 — Tabla por defecto de reference.conf: serializers, serialization-bindings y serialization-identifiers

**Contrato observable.** serializers: java, bytes, primitive-long, primitive-int, primitive-string, primitive-bytestring, primitive-boolean. serialization-bindings: "[B"=bytes, "java.io.Serializable"=java, java.lang.String, ByteString$ByteString1C/ByteString1/ByteStrings, java.lang.Long/scala.Long, java.lang.Integer/scala.Int, java.lang.Boolean/scala.Boolean. serialization-identifiers: JavaSerializer=1, ByteArraySerializer=4, y por alias primitive-*: 18,19,20,21,35 (reservados 0-40).

**Fuente.** actor/src/main/resources/reference.conf:799-848

**Condiciones.** Con allow-java-serialization=off (por defecto), la clase Serializable usa DisabledJavaSerializer con el id de JavaSerializer (1). Object (sin binding) -> NotSerializableException.

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AC-SER-001.2, AC-SER-004

**Equivalente Rust propuesto.** Tabla `const`/embebida; `TypeKey` de i64, i32, String, bool, ByteString, Vec<u8>.

**Diferencias observables.** "java.io.Serializable" no tiene sentido en Rust (ver Q-SER-2); las claves FQCN JVM se reemplazan por TypeKey. Los ids se conservan.

**Aserciones.**
- A1. Serializable usa DisabledJavaSerializer con reference.conf — SerializeSpec.scala:372-374
- A2. Array[Byte] -> ByteArraySerializer — SerializeSpec.scala:376-378
- A3. Long, Integer, String, ByteString1/1C/ByteStrings -> LongSerializer, IntSerializer, StringSerializer, ByteStringSerializer — SerializeSpec.scala:380-388
- A4. Object sin binding -> NotSerializableException — SerializeSpec.scala:390-392
- A5. Una función (lambda) con Java deshabilitado -> DisabledJavaSerializer — SerializeSpec.scala:394-397
- A6. Con allow-java-serialization=on: Serializable -> JavaSerializer, mismos primitivos — SerializeSpec.scala:416-435

**Oráculo.** Jar oficial: listar ser.bindings y serializerByIdentity con config por defecto y volcar. Escenario no existe ni se ejecutó.

**Decisiones.** Q-SER-1, Q-SER-2

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-SER-004.1 — DisabledJavaSerializer: reemplaza JavaSerializer con allow-java-serialization=off; warnings Security; excepciones fijas

**Contrato observable.** DisabledJavaSerializer(system) usa el id de JavaSerializer. toBinary: warning "Outgoing message attempted to use Java Serialization even though `pekko.actor.allow-java-serialization = off` was set! Message type was: [<clase>]" y lanza IllegalSerialization. fromBinary (array o ByteBuffer): warning "Incoming message attempted to use Java Serialization even though `pekko.actor.allow-java-serialization = off` was set!" y lanza IllegalDeserialization. new JavaSerializer(system) con la opción off lanza JavaSerializationException("Attempted creation of `JavaSerializer` while `pekko.actor.allow-java-serialization = off` was set!").

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/Serializer.scala:340-345 (JavaSerializer ctor guard) ; actor/src/main/scala/org/apache/pekko/serialization/Serializer.scala:367-410 (DisabledJavaSerializer (toBinary 379-386, fromBinary 388-403)) ; actor/src/main/scala/org/apache/pekko/serialization/Serializer.scala:412-418 (excepciones); actor/src/main/resources/reference.conf:761-778

**Condiciones.** Excepciones (RuntimeException con NoStackTrace): IllegalSerialization = "Attempted to serialize message using Java serialization while `pekko.actor.allow-java-serialization` was disabled. Check WARNING logs for more details." ; IllegalDeserialization = "Attempted to deserialize message using Java serialization while `pekko.actor.allow-java-serialization` was disabled. Check WARNING logs for more details." (Serializer.scala:414-417). Los warnings llevan marker Security.

**Orden / concurrencia / ciclo de vida.** n/a: sin estado; los warnings se emiten en el hilo que serializa.

**Dependencias.** AC-EVT (marker Security), AC-SER-004.2, Q-X-7

**Equivalente Rust propuesto.** `DisabledJavaSerializer` que devuelve `Err(JavaSerializationError::{Serialize,Deserialize})` y emite `tracing`/log con marker Security.

**Diferencias observables.** En Rust no existe Java serialization; este serializador existe solo para (a) mantener el id 1 reservado, (b) reproducir los avisos si un peer envía id=1 (compat de cable con peers JVM), (c) casos de test que verifican el warning. Q-SER-2.

**Aserciones.**
- A1. Mensaje no verificable (List("a")) con serialize-messages=on y allow-java=off -> exactamente 1 warning que empieza "Outgoing message attempted to use Java Serialization" — DisabledJavaSerializerWarningSpec.scala:42-49
- A2. Mensaje Msg del paquete org.apache.pekko no genera el warning y llega intacto — DisabledJavaSerializerWarningSpec.scala:51-57
- A3. DisabledJavaSerializer(esys).fromBinary(ByteBuffer, "") loguea warning "Incoming message attempted to use Java Serialization" (1 vez) y lanza JavaSerializationException — DisabledJavaSerializerWarningSpec.scala:59-67
- A4. new JavaSerializer(sys con off) lanza JavaSerializationException con mensaje que incluye "pekko.actor.allow-java-serialization = off"; findSerializerFor(...).toBinary lanza lo mismo desde un setup programático — SerializationSetupSpec.scala:172-182
- A5. Un JavaSerializer programático en sistema con off se reemplaza por DisabledJavaSerializer con el mismo identifier; el resto sigue funcionando — SerializationSetupSpec.scala:184-197
- A6. Deserializar con el id de Java en un sistema off lanza JavaSerializationException; ser2.serializerByIdentity(serId) es el DisabledJavaSerializer — SerializationSetupSpec.scala:199-212
- A7. Textos exactos de las dos excepciones — derivada del código Serializer.scala:414-417; no afirmada por spec

**Oráculo.** Jar oficial: reproducir warnings/excepciones y comparar texto. Escenario no existe ni se ejecutó.

**Decisiones.** Q-SER-2

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-SER-004.2 — allow-java-serialization, warn-about-java-serializer-usage, warn-on-no-serialization-verification: reglas de aviso y reemplazo

**Contrato observable.** shouldWarnAboutJavaSerializer = warn-about-java-serializer-usage && serializador es Java/Disabled && la clase no empieza por "org.apache.pekko." ni "java.lang." && !(warn-on-no-serialization-verification=off && clase extiende NoSerializationVerificationNeeded). Mensaje: "Using the Java serializer for class [<clase>] which is not recommended because of performance implications. Use another serializer or disable this warning using the setting 'pekko.actor.warn-about-java-serializer-usage'" (marker Security). También warning "Using serializer [..] for message [..]. Note that this serializer is not implemented by Apache Pekko. It's not recommended to replace serializers for messages provided by Apache Pekko." cuando una clase org.apache.pekko.* usa un serializador no-pekko. Con allow-java-serialization=off, todo JavaSerializer configurado o programático se reemplaza por DisabledJavaSerializer (debug "Replacing JavaSerializer with DisabledJavaSerializer, due to `pekko.actor.allow-java-serialization = off`.").

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:367-390 (aviso al resolver) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:405-411 (y 431-446 reemplazo) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:475-479 (requisito) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:481-496 (warnUnexpectedNonPekkoSerializer) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:567-595 (flags y shouldWarnAboutJavaSerializer) ; actor/src/main/resources/reference.conf:761-797

**Condiciones.** Valores por defecto: allow-java-serialization=off, warn-about-java-serializer-usage=on, warn-on-no-serialization-verification=on. Con off y un JavaSerializer en el mapa final: IllegalArgumentException("requirement failed: Disallowed JavaSerializer binding.") (:477).

**Orden / concurrencia / ciclo de vida.** Los avisos salen una sola vez por clase (la primera resolución inserta en serializerMap).

**Dependencias.** AC-SER-001.1, AC-SER-004.1, Q-X-7

**Equivalente Rust propuesto.** `SerializationSettings { allow_java_serialization: bool, warn_about_java_serializer_usage: bool, warn_on_no_serialization_verification: bool }` y la lógica `should_warn_about_java_serializer(type_name)`.

**Diferencias observables.** Sin Java serialization los avisos sobre JavaSerializer solo aplican a bindings heredados; el aviso de "serializador no-pekko para mensaje pekko" sí es portable usando el prefijo de módulo del tipo.

**Aserciones.**
- A1. Con allow-java=on y warn=on: serializerFor(BigDecimal) loguea exactamente 1 warning que empieza "Using the Java serializer for class" — SerializeSpec.scala:446-450
- A2. Sin warning para java.lang.String — SerializeSpec.scala:452-456
- A3. warn-on-no-serialization-verification=on: warning para BigDecimal (1) y también para NoVerification (1) — SerializeSpec.scala:513-537
- A4. warn-on-no-serialization-verification=off: warning para BigDecimal (1) y ninguno para NoVerification — SerializeSpec.scala:539-563
- A5. Serializador fuera de org.apache.pekko para mensaje org.apache.pekko -> 1 warning ".*not implemented by Apache Pekko.*" — SerializeSpec.scala:277-281
- A6. Disallowed JavaSerializer binding — derivada del código Serialization.scala:475-479; no afirmada por spec

**Oráculo.** Jar oficial: ejecutar los escenarios y volcar logs. Escenario no existe ni se ejecutó.

**Decisiones.** Q-SER-2, Q-X-7

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-SER-005 — SerializationSetup/SerializerDetails (registro programático de serializadores y bindings)

**Contrato observable.** SerializationSetup(createSerializers: ExtendedActorSystem => Seq[SerializerDetails]) es un Setup del ActorSystemSetup. SerializerDetails(alias, serializer, useFor: Seq[Class]). Los details se suman a los de config; sus useFor crean bindings; un binding de config que nombra el alias "test" usa el serializador programático.

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/SerializationSetup.scala:24-48 (SerializationSetup) ; actor/src/main/scala/org/apache/pekko/serialization/SerializationSetup.scala:51-82 (SerializerDetails) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:431-446 (serializerDetails) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:448-454 (serializers) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:459-473 (bindings)

**Condiciones.** JavaSerializer programático con off -> reemplazado por Disabled (ver AC-SER-004.1). Misma clase en config y programático: los bindings programáticos se agregan tras los de config antes de ordenar.

**Orden / concurrencia / ciclo de vida.** createSerializers se invoca una vez durante la construcción de la extensión.

**Dependencias.** AC-CFG-015 (ActorSystemSetup), AC-SER-001.2

**Equivalente Rust propuesto.** `SerializationSetup` implementando el trait `Setup`; `SerializerDetails { alias, serializer: Arc<dyn Serializer>, use_for: Vec<TypeKey> }`.

**Diferencias observables.** La API Java (create con java.util.List) no se porta (AC-JAPI-008). `useFor` usa TypeKey.

**Aserciones.**
- A1. findSerializerFor(ProgrammaticDummy) devuelve la instancia programática — SerializationSetupSpec.scala:113-116
- A2. Un binding de config "…ConfigurationDummy"=test usa el serializador programático de alias "test" — SerializationSetupSpec.scala:118-121
- A3. Serializer inexistente en config -> ClassNotFoundException al crear el sistema — SerializationSetupSpec.scala:123-135

**Oráculo.** No aplica por separado: cubierto por el escenario de AC-SER-001.2 (jar oficial).

**Decisiones.** Q-X-2

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-SER-006 — AsyncSerializer, AsyncSerializerWithStringManifest y AsyncSerializerWithStringManifestCS

**Contrato observable.** AsyncSerializer define toBinaryAsync(o): Future[Array[Byte]] y fromBinaryAsync(bytes, manifest): Future[AnyRef]. AsyncSerializerWithStringManifest los une a SerializerWithStringManifest; sus toBinary/fromBinary síncronos loguean warning ("Async serializer called synchronously. This will block. Async serializers should only be used for pekko persistence plugins that support them. Class: {}" / "..for Pekko persistence plugins that support them. Manifest: [{}]") y hacen Await.result(.., Duration.Inf). La variante CS adapta CompletionStage.

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/AsyncSerializer.scala:33-46 (trait) ; actor/src/main/scala/org/apache/pekko/serialization/AsyncSerializer.scala:52-71 (AsyncSerializerWithStringManifest (warning 59-61 y 66-68, Await 62 y 69)) ; actor/src/main/scala/org/apache/pekko/serialization/AsyncSerializer.scala:77-96 (variante CS)

**Condiciones.** Los mensajes de warning difieren en mayúsculas ("pekko persistence" vs "Pekko persistence"): son literales distintos en :60-61 y :67-68. Await sin timeout (Duration.Inf).

**Orden / concurrencia / ciclo de vida.** Los métodos async no bloquean; los síncronos bloquean el hilo llamante indefinidamente.

**Dependencias.** Q-X-4 (runtime), AC-SER-002

**Equivalente Rust propuesto.** `trait AsyncSerializer`; adaptador `SyncFallback` que loguea el warning y ejecuta el future con `block_in_place`/hilo dedicado.

**Diferencias observables.** Bloquear dentro de un runtime async puede interbloquear; el equivalente debe usar un hilo/block_in_place (Q-SER-5). CompletionStage no se porta (AC-JAPI-008).

**Aserciones.**
- A1. findSerializerFor(Message1) es TestAsyncSerializer; toBinaryAsync/fromBinaryAsync(.., manifest) devuelven el mismo objeto — AsyncSerializeSpec.scala:111-119
- A2. ser.serialize(Message1) loguea exactamente 1 warning que empieza "Async serializer called synchronously" — AsyncSerializeSpec.scala:121-125
- A3. Variante CS: serialize + deserialize síncronos loguean 2 warnings y reproducen el mensaje — AsyncSerializeSpec.scala:127-137

**Oráculo.** Jar oficial: logs del warning. Escenario no existe ni se ejecutó.

**Decisiones.** Q-SER-5, Q-X-4

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-SER-007.1 — Serialization.serializedActorPath(actorRef): formato con la dirección de transporte actual

**Contrato observable.** Sin Information vigente: si la ref es local con celda usa path.toSerializationFormatWithAddress(provider.getDefaultAddress) (si lanza no-fatal, path.toSerializationFormat); si no, path.toSerializationFormat. Con Information(address, system): si la ref pertenece a ese system (o no tiene sistema) usa la address de la Information; si pertenece a otro, provider.getExternalAddressFor(address) o la default.

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:66-95 (serializedActorPath) ; actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:470-496 (SerializedActorRef (path con serializedActorPath :475)) ; LocalActorRefProvider.getExternalAddressFor actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:759

**Condiciones.** Local provider: getExternalAddressFor(addr) = Some(addr) si addr == rootPath.address, si no None (ActorRefProvider.scala:759).

**Orden / concurrencia / ciclo de vida.** Depende de la variable thread-local currentTransportInformation (ver AC-SER-007.2).

**Dependencias.** AC-PATH, AC-REF-023, AC-SYS-018

**Equivalente Rust propuesto.** `fn serialized_actor_path(actor_ref:&ActorRef, ctx:Option<&TransportInformation>)->String`

**Diferencias observables.** Remote queda fuera del módulo (límite del inventario): solo se necesita el caso local.

**Aserciones.**
- A1. Formato "path con address por defecto" para refs locales — derivada del código Serialization.scala:78-85; no afirmada por spec
- A2. Formato con otra Information/address — derivada del código Serialization.scala:86-93; no afirmada por spec

**Oráculo.** Jar oficial: ActorRef local -> Serialization.serializedActorPath fuera y dentro de withTransportInformation. Escenario no existe ni se ejecutó.

**Decisiones.** Q-SER-3

**Estado.** analysis=listed; impl=pending; validation=pending.

### AC-SER-007.2 — Información de transporte (Serialization.Information, currentTransportInformation, withTransportInformation, getCurrentTransportInformation)

**Contrato observable.** currentTransportInformation es una DynamicVariable (thread-local) inicialmente null. serialize/deserialize fijan provider.serializationInformation si no hay valor y restauran el anterior en finally. Serialization.withTransportInformation(system)(f) fija el valor (no re-fija si ya es el mismo objeto). serializationInformation del provider local = Information(rootPath.address, system) cacheada; "Too early access of serializationInformation" si system es null.

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:51 (variable) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:102 (Information) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:116-124 (withTransportInformation) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:134-141 (getCurrentTransportInformation) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:170-177 (helper privado) ; actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:764-780 (serializationInformation)

**Condiciones.** getCurrentTransportInformation sin valor -> IllegalStateException("currentTransportInformation is not set, use Serialization.withTransportInformation") (Serialization.scala:137-138). serializationInformation con system null -> IllegalStateException("Too early access of serializationInformation") (ActorRefProvider.scala:771).

**Orden / concurrencia / ciclo de vida.** Thread-local: se pierde al cambiar de hilo; los serializadores async no la heredan.

**Dependencias.** AC-SYS-018 (provider), AC-SER-001.3

**Equivalente Rust propuesto.** `tokio::task_local!`/`thread_local!` con `TransportInformation`, o parámetro explícito `&SerializationContext` pasado a `Serializer` (Q-SER-3).

**Diferencias observables.** Variable dinámica -> en Rust el contexto debe viajar con la tarea (task_local) o explícitamente; el cambio de hilo del runtime puede perder un thread_local.

**Aserciones.**
- A1. getCurrentTransportInformation sin valor -> IllegalStateException con el texto exacto — derivada del código Serialization.scala:136-139; no afirmada por spec
- A2. serialize/deserialize fijan y restauran la información (anidados respetan el valor externo) — derivada del código Serialization.scala:170-177,282-295; no afirmada por spec
- A3. serializationInformation cacheada y excepción "Too early access" — derivada del código ActorRefProvider.scala:764-780; no afirmada por spec

**Oráculo.** Jar oficial: jshell llamando withTransportInformation/getCurrentTransportInformation. Escenario no existe ni se ejecutó.

**Decisiones.** Q-SER-3

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-SER-008.1 — pekko.actor.serialize-messages: verifica que cada mensaje sobreviva a serialize+deserialize (SerializationCheckFailedException)

**Contrato observable.** sendMessage: si SerializeAllMessages, serializeAndDeserialize(envelope): desenvuelve DeadLetter; salta si el mensaje es NoSerializationVerificationNeeded o su clase empieza por algún prefijo de no-serialization-verification-needed-class-prefix (por defecto "org.apache.pekko."); si no, serializa con el serializador de la clase y deserializa con id y manifest; con DisabledJavaSerializer y sin motivo de aviso se omite (mensajes "locales conocidos"). Cualquier excepción no fatal se envuelve en SerializationCheckFailedException.

**Fuente.** actor/src/main/scala/org/apache/pekko/actor/dungeon/Dispatch.scala:36-41 (excepción) ; actor/src/main/scala/org/apache/pekko/actor/dungeon/Dispatch.scala:171-178 (sendMessage) ; actor/src/main/scala/org/apache/pekko/actor/dungeon/Dispatch.scala:180-206 (serializeAndDeserialize) ; actor/src/main/scala/org/apache/pekko/actor/dungeon/Dispatch.scala:208-223 (payload) ; actor/src/main/resources/reference.conf:129-140 (serialize-messages, no-serialization-verification-needed-class-prefix)

**Condiciones.** SerializationCheckFailedException(msg, cause) extiende PekkoException con mensaje: "Failed to serialize and deserialize message of type <FQCN> for testing. To avoid this error, either disable 'pekko.actor.serialize-messages', mark the message with 'org.apache.pekko.actor.NoSerializationVerificationNeeded', or configure serialization to support this message" (Dispatch.scala:37-41). Se loguea como error del actor emisor/receptor según el dispatcher (handleException).

**Orden / concurrencia / ciclo de vida.** Ocurre en el hilo del emisor, antes de encolar en el buzón.

**Dependencias.** AC-CELL (sendMessage), AC-SER-001, AC-SER-004.2, AC-SER-007.2

**Equivalente Rust propuesto.** Hook opcional en `ActorCell::send_message` que usa `Serialization` y falla con `SerializationCheckFailed { message_type, cause }`.

**Diferencias observables.** Con mensajes `Box<dyn Any>` solo se puede verificar lo registrado en el serializer registry; lo no registrado falla igual que en Pekko (NotSerializableException), no hay equivalente a "Serializable por herencia" (Q-SER-4).

**Aserciones.**
- A1. serialize-messages=on y SerializeAllCreators=on se reflejan en settings — SerializeSpec.scala:311-314
- A2. Un Object (java.lang.Object) enviado a un actor genera 1 evento SerializationCheckFailedException con "Failed to serialize and deserialize message of type java.lang.Object" — SerializeSpec.scala:339-349
- A3. Un mensaje del sistema (ActorSystem) o con prefijo org.apache.pekko. no se verifica y se entrega igual; un ask de String vuelve intacto — SerializeSpec.scala:340-342,351-361
- A4. Un error de serialización se loguea con un mensaje que menciona NoSerializationVerificationNeeded (EventFilter patrón ".*NoSerializationVerificationNeeded.*", 1 ocurrencia) — DispatchSpec.scala:36-41

**Oráculo.** Jar oficial: jshell con ActorSystem y serialize-messages=on para mensaje serializable y no serializable. Escenario no existe ni se ejecutó.

**Decisiones.** Q-SER-4

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-SER-008.2 — pekko.actor.serialize-creators: verifica argumentos de Props al crear un actor

**Contrato observable.** En makeChild: si SerializeAllCreators, no es systemService y props.deploy.scope != LocalScope, cada arg (no null, no NoSerializationVerificationNeeded, sin prefijo exento) se serializa y deserializa; la comprobación devuelve true si el resultado != null; cualquier excepción no fatal -> IllegalArgumentException.

**Fuente.** actor/src/main/scala/org/apache/pekko/actor/dungeon/Children.scala:280-309 ; actor/src/main/resources/reference.conf:133-140

**Condiciones.** Error: IllegalArgumentException("pre-creation serialization check failed at [<path del padre>/<nombre>]", cause) (Children.scala:305). Nota: forall corta en el primer argumento que devuelva false (resultado null) sin error (derivado).

**Orden / concurrencia / ciclo de vida.** Ocurre síncronamente en actorOf del padre (hilo llamante).

**Dependencias.** AC-CELL, AC-REF-007, AC-SER-001

**Equivalente Rust propuesto.** Hook en la creación de hijos; los `Props` de Rust (closure) no son serializables, por lo que el equivalente verifica solo argumentos declarados (`Props::with_serializable_args`).

**Diferencias observables.** Props de Rust usan closures; no hay args reflectivos. Q-SER-4 propone aplicar la verificación solo a "args declarados".

**Aserciones.**
- A1. Props[FooActor] y Props(new FooAbstractActor) se crean sin error con serialize-creators=on — SerializeSpec.scala:316-321
- A2. Props(classOf[NonSerializableActor], new AnyRef) -> IllegalArgumentException — SerializeSpec.scala:323-325
- A3. Un arg que empieza por org.apache.pekko. (ActorSystem) no se verifica y no se loguea nada — SerializeSpec.scala:329-336
- A4. Texto "pre-creation serialization check failed at [..]" — derivada del código Children.scala:305; no afirmada por spec

**Oráculo.** Jar oficial: jshell. Escenario no existe ni se ejecutó.

**Decisiones.** Q-SER-4

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-SER-009 — Límite de profundidad de deserialización anidada (pekko.actor.serialization-max-nesting-depth)

**Contrato observable.** Contador thread-local por hilo: atNextLevel(max)(body) incrementa, lanza si pasa de max y siempre decrementa en finally. deserializeByteArray envuelve cada deserialización con atNextLevel(maxNestingDepth).

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/NestedDeserialization.scala:43-45 (contador) ; actor/src/main/scala/org/apache/pekko/serialization/NestedDeserialization.scala:51-61 (atNextLevel) ; actor/src/main/scala/org/apache/pekko/serialization/NestedDeserialization.scala:64 (currentDepth) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:163-164, (236-237) ; actor/src/main/resources/reference.conf:780-785

**Condiciones.** Error: NotSerializableException("Message exceeds the maximum deserialization nesting depth of [<max>]. Configure with 'pekko.actor.serialization-max-nesting-depth'.") (NestedDeserialization.scala:56-58). Con max=32 el nivel 33 falla.

**Orden / concurrencia / ciclo de vida.** Estado thread-local; no cruza hilos ni futuros.

**Dependencias.** AC-SER-001.3, AC-CFG

**Equivalente Rust propuesto.** `NestedDeserializationGuard` con contador thread_local (o parámetro de contexto) y `SerializationSettings::max_nesting_depth`.

**Diferencias observables.** Rust no tiene StackOverflowError recuperable; el límite sigue siendo necesario para evitar recursión sin cota.

**Aserciones.**
- A1. Nivel 33 con max=32 lanza NotSerializableException con el texto exacto; el contador vuelve a 0 después (también tras la excepción) — derivada del código NestedDeserialization.scala:51-61; no afirmada por spec
- A2. Valor por defecto 32 y documentación de qué cuenta como anidado (Some, Optional, Status.Failure, StatusReply, causa de Throwable, ActorSelectionMessage) — derivada del código actor/src/main/resources/reference.conf:780-785; no afirmada por spec

**Oráculo.** Jar oficial: jshell llamando NestedDeserialization (private[pekko], accesible en bytecode). Escenario no existe ni se ejecutó.

**Decisiones.** -

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-SER-010 — Límite de descompresión gzip (pekko.serialization.max-decompressed-size) y Decompression.gunzip

**Contrato observable.** maxDecompressedSize(system): lee la cadena de pekko.serialization.max-decompressed-size; "unlimited" -> -1; entero negativo -> ese valor; si no, config.getBytes (p. ej. "16 KiB" = 16384). gunzip lee en bloques de 4096 acumulando total; si maxDecompressedSize >= 0 y total > max lanza antes de seguir leyendo.

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/Decompression.scala:44-55 (maxDecompressedSize) ; actor/src/main/scala/org/apache/pekko/serialization/Decompression.scala:62-81 (gunzip (bloque 37, chequeo 71-75)) ; actor/src/main/resources/reference.conf:852-863

**Condiciones.** Error: NotSerializableException("Compressed message expands to more than the maximum decompressed size of [<max>] bytes. Configure with 'pekko.serialization.max-decompressed-size'.") (Decompression.scala:72-75). La comparación es total > max: exactamente max se acepta. Aplica a serializadores de cluster/ddata/metrics (no a Jackson). Gzip inválido -> IOException/ZipException de GZIPInputStream (derivado, no afirmado).

**Orden / concurrencia / ciclo de vida.** n/a (función pura).

**Dependencias.** AC-CFG (getBytes), AC-IO-012

**Equivalente Rust propuesto.** `fn gunzip(bytes:&[u8], max_decompressed_size: Option<u64>) -> Result<Vec<u8>, NotSerializableError>` con `flate2::read::GzDecoder` leyendo en bloques de 4096.

**Diferencias observables.** Los errores de gzip malformado cambian de tipo (flate2 error); el mensaje límite se conserva.

**Aserciones.**
- A1. Un payload de 8 KiB con límite 1 MiB vuelve idéntico — DecompressionSpec.scala:41-44
- A2. Un payload de exactamente 1024 bytes con límite 1024 se acepta — DecompressionSpec.scala:46-49
- A3. Un payload de 1025 bytes con límite 1024 -> NotSerializableException — DecompressionSpec.scala:51-56
- A4. El mensaje incluye "pekko.serialization.max-decompressed-size" — DecompressionSpec.scala:58-62
- A5. Una bomba (64 MiB de ceros, <1 MiB comprimidos) se rechaza con límite 1024 sin descomprimir todo — DecompressionSpec.scala:64-72
- A6. Límite negativo = sin límite (1025 bytes pasan) — DecompressionSpec.scala:74-78
- A7. Config por defecto "unlimited" -> -1; "16 KiB" -> 16384; "-1" -> -1 — DecompressionSpec.scala:80-102

**Oráculo.** Jar oficial (Decompression es private[pekko] pero público en bytecode): jshell para fijar textos. Escenario no existe ni se ejecutó.

**Decisiones.** Q-X-1

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-SER-011 — WireConfig.parseString: HOCON de origen remoto sin resolver include

**Contrato observable.** Usa un includer propio que implementa las 4 interfaces (ConfigIncluder, File, URL, Classpath) y devuelve ConfigObject vacío; el parser delega include file()/url()/classpath() a los métodos tipados y solo el include plano a include().

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/WireConfig.scala:51-82 ((NoIncludes 61-74) ; actor/src/main/scala/org/apache/pekko/serialization/WireConfig.scala:parseOptions 76 ; actor/src/main/scala/org/apache/pekko/serialization/WireConfig.scala:parseString 81)

**Condiciones.** Un include nunca lee archivos ni hace requests; no falla aunque sea required(...). Otros errores de sintaxis siguen siendo ConfigException.Parse (derivado).

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AC-CFG-017 (parser HOCON con includes desactivables), Q-X-1

**Equivalente Rust propuesto.** `HoconParseOptions { includes: IncludePolicy::Disabled }` consumido por `WireConfigParser`.

**Diferencias observables.** Si se usa un crate HOCON externo, debe permitir desactivar includes (Q-X-1); si no, parser propio.

**Aserciones.**
- A1. a = 1 y b { c = two } se leen normalmente — WireConfigSpec.scala:44-48
- A2. La salida de ConfigRenderOptions.concise de roots se re-parsea (lista de strings) — WireConfigSpec.scala:50-55
- A3. include file("...") no lee el archivo y a=1 sigue — WireConfigSpec.scala:57-63
- A4. include required(file("...")) tampoco — WireConfigSpec.scala:65-70
- A5. include url("file:...") no se resuelve — WireConfigSpec.scala:72-78
- A6. include classpath("reference.conf") no importa pekko.version — WireConfigSpec.scala:80-83
- A7. Premisa: ConfigFactory.parseString sí resuelve los tres includes — WireConfigSpec.scala:85-91

**Oráculo.** Jar oficial + config-1.4.9.jar: ya existe tools/oracle/config-hocon.jsh para HOCON; el modo sin includes es escenario nuevo. No ejecutado.

**Decisiones.** Q-X-1

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-SER-012 — pekko.serialization.protobuf.allowed-classes / whitelist-class (solo reference.conf; sin consumidor en pekko-actor)

**Contrato observable.** Lista deprecada whitelist-class (4 FQCN de protobuf/scalapb) y allowed-classes = ${whitelist-class}: clases permitidas aunque no estén en serialization-bindings (una clase exacta o su superclase/interfaces de un nivel).

**Fuente.** actor/src/main/resources/reference.conf:865-880

**Condiciones.** Sin comportamiento en pekko-actor. Debe seguir parseando como HOCON y sustituir ${pekko.serialization.protobuf.whitelist-class}.

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AC-CFG (sustituciones), módulos fuera de alcance (remote/cluster)

**Equivalente Rust propuesto.** Clave aceptada en `reference.conf` embebido; sin tipo.

**Diferencias observables.** Sin diferencia observable en el módulo.

**Aserciones.**
- A1. El reference.conf embebido define ambas claves y la sustitución se resuelve — derivada del archivo actor/src/main/resources/reference.conf:865-880; no afirmada por spec

**Oráculo.** No aplica: sin comportamiento en pekko-actor.

**Decisiones.** Q-SER-6

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-SER-013 — JavaSerializer habilitado: toBinary/fromBinary con ObjectStream y JavaSerializer.currentSystem; serialización Java de ActorRef (SerializedActorRef)

**Contrato observable.** JavaSerializer(system): includeManifest=false; toBinary usa ObjectOutputStream con JavaSerializer.currentSystem fijada; fromBinary usa ClassLoaderObjectInputStream sobre UnsynchronizedByteArrayInputStream. ActorRef (LocalActorRef) se serializa vía writeReplace a SerializedActorRef(path serializedActorPath) y readResolve lo resuelve con currentSystem.value.provider.resolveActorRef(path); sin currentSystem lanza IllegalStateException.

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/Serializer.scala:306-335 (JavaSerializer.currentSystem) ; actor/src/main/scala/org/apache/pekko/serialization/Serializer.scala:340-362 (JavaSerializer) ; actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:462-496

**Condiciones.** Error al deserializar una ref sin sistema: IllegalStateException("Trying to deserialize a serialized ActorRef without an ActorSystem in scope. Use 'org.apache.pekko.serialization.JavaSerializer.currentSystem.withValue(system) { ... }'") (ActorRef.scala:481-483).

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AC-SER-004, AC-REF-023

**Equivalente Rust propuesto.** Sin equivalente (Q-SER-2).

**Diferencias observables.** No hay Java serialization en Rust. Observable: ninguno si se excluye (propuesto); las refs se serializan con serializedActorPath mediante un serializador propio de ref.

**Aserciones.**
- A1. Con allow-java=on: Serializable->JavaSerializer; Address, Person y Record hacen roundtrip — SerializeSpec.scala:416-418,465-476
- A2. Una función se serializa y deserializa con JavaSerializer (f2(3)==4) — SerializeSpec.scala:437-444
- A3. Un ActorCell no es serializable (NotSerializableException) — SerializeSpec.scala:478-489
- A4. DeadLetterActorRef se deserializa a la misma instancia a.deadLetters — SerializeSpec.scala:491-508
- A5. Un LocalActorRef serializado y leído con currentSystem devuelve la misma ref, también vía JavaSerializer — ActorRefSpec.scala:288-317

**Oráculo.** Jar oficial: jshell con allow-java-serialization=on. Escenario no existe ni se ejecutó.

**Decisiones.** Q-SER-2

**Estado.** analysis=listed; impl=pending; validation=pending.

### AC-SER-014 — SerializationExtension: registro como library-extension, acceso y prohibición de pedirla desde el constructor de un serializador

**Contrato observable.** SerializationExtension es un ExtensionId[Serialization] cargado por pekko.library-extensions. Mientras se construye, la extensión figura "en registro" (CountDownLatch); otro hilo/llamada que pida la extensión espera hasta creation-timeout y falla.

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/SerializationExtension.scala:24-29 ; actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1167-1192 (findExtension (mensaje 1182-1186)) ; actor/src/main/resources/reference.conf:78

**Condiciones.** IllegalStateException("Initialization of [SerializationExtension] took more than [<ms> ms]. A serializer must not access the SerializationExtension from its constructor. Use lazy init.") (ActorSystem.scala:1182-1186; el spec solo comprueba que el mensaje incluye "SerializationExtension from its constructor").

**Orden / concurrencia / ciclo de vida.** La espera bloquea el hilo; el sistema no llega a crearse: ActorSystem(...) lanza.

**Dependencias.** AC-EXT-001, AC-SYS-001

**Equivalente Rust propuesto.** `SerializationExtension` registrada vía `ExtensionRegistry` con detección de ciclo (`ExtensionInitializationError::Cycle`).

**Diferencias observables.** En Rust se puede detectar el ciclo de inmediato (sin esperar creation-timeout) y devolver el mismo texto; diferencia de latencia. Documentado como decisión Q-SER-6.

**Aserciones.**
- A1. ActorSystem con un serializador que llama SerializationExtension(system) en su constructor lanza IllegalStateException cuyo mensaje incluye "SerializationExtension from its constructor" (creation-timeout=1s) — SerializeSpec.scala:565-587

**Oráculo.** Jar oficial: jshell. Escenario no existe ni se ejecutó.

**Decisiones.** Q-SER-6, Q-X-4

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-001 — IO(ext) punto de entrada y IO.Extension (actor manager por extensión: IO-TCP, IO-UDP-FF, IO-UDP-CONN, IO-DNS)

**Contrato observable.** IO.Extension exige def manager: ActorRef. IO.apply[T <: Extension](key)(implicit system) = key(system).manager. Cada extensión crea su manager con system.systemActorOf(Props(...).withDispatcher(management-dispatcher).withDeploy(Deploy.local), name): Tcp "IO-TCP", Udp "IO-UDP-FF", UdpConnected "IO-UDP-CONN", Dns "IO-DNS". La API Java expone getManager.

**Fuente.** actor/src/main/scala/org/apache/pekko/io/IO.scala:24-38 ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:654-663 ; actor/src/main/scala/org/apache/pekko/io/Udp.scala:236-245 ; actor/src/main/scala/org/apache/pekko/io/UdpConnected.scala:174-185 ; actor/src/main/scala/org/apache/pekko/io/Dns.scala:213, (242-249)

**Condiciones.** Los managers nacen al crearse la extensión (acceso lazy a IO(Tcp)). Nombres de actor fijos. Un fallo de configuración (p. ej. nr-of-selectors<=0) falla la creación de la extensión (ver AC-IO-003).

**Orden / concurrencia / ciclo de vida.** Los comandos se envían al manager con tell; las respuestas llegan al sender (CommandFailed/Bound/Connected...).

**Dependencias.** AC-EXT-001, AC-SYS (systemActorOf), AC-IO-003, AC-IO-004.1

**Equivalente Rust propuesto.** `trait IoExtension: Extension { fn manager(&self) -> ActorRef<dyn Any> }`; `Tcp::get(system)`; `fn io(system: &ActorSystem, ext: impl ExtensionId<..>) -> ActorRef`.

**Diferencias observables.** Nombres de actor y paths /system/IO-TCP etc. conservados si el sistema expone systemActorOf; con Q-IO-1 los internos pueden ser tareas.

**Aserciones.**
- A1. IO(Tcp) ! Bind(handler, endpoint, options) responde Bound al commander; Connect responde Connected tras aceptar — TcpIntegrationSpec.scala:43-52; TcpIntegrationSpecSupport.scala:48-58
- A2. IO(Udp) ! Bind responde Bound y IO(Udp) ! SimpleSender responde SimpleSenderReady — UdpIntegrationSpec.scala:31-49
- A3. Nombres IO-TCP / IO-UDP-FF / IO-UDP-CONN / IO-DNS — derivada del código Tcp.scala:657, Udp.scala:239, UdpConnected.scala:179, Dns.scala:213; no afirmada por spec

**Oráculo.** Jar oficial: jshell crea ActorSystem y lista /system hijos. Escenario no existe ni se ejecutó.

**Decisiones.** Q-IO-1

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-002.1 — Tcp: jerarquía de mensajes (Command/Event/CloseCommand/ConnectionClosed/NoAck/WriteCommand/CompoundWrite), requisitos de construcción y CommandFailed.cause

**Contrato observable.** Command (con failureMessage = CommandFailed(this)): Connect, Bind, Register, Unbind, CloseCommand{Close, ConfirmedClose, Abort}, WriteCommand{Write, WriteFile(deprecado), WritePath, CompoundWrite}, ResumeWriting, SuspendReading, ResumeReading, ResumeAccepting. Event: Received, Connected, CommandFailed, WritingResumed, Bound, Unbound, ConnectionClosed{Closed, Aborted, ConfirmedClosed, PeerClosed, ErrorClosed(cause)}, NoAck. SimpleWriteCommand exige ack != null; wantsAck = ack no es NoAck. Write.empty = Write(ByteString.empty, NoAck); Write.apply(data) devuelve Write.empty si data vacío. +: y ++: arman CompoundWrite; WriteCommand(writes) = writes ++: Write.empty.

**Fuente.** actor/src/main/scala/org/apache/pekko/io/Tcp.scala:111-120 (Message/Command) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:132-139 (Connect) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:160-167 (Bind) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:187-188 (Register) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:200-253 (CloseCommand/Close/ConfirmedClose/Abort) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:261-267 (NoAck) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:272-316 (WriteCommand) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:321-339 (SimpleWriteCommand) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:351-367 (Write) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:373-391 (WriteFile/WritePath) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:401-416 (CompoundWrite) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:475-508 (CommandFailed) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:538-603 (ConnectionClosed y subtipos)

**Condiciones.** IllegalArgumentException("requirement failed: ack must be non-null. Use NoAck if you don't want acks.") (Tcp.scala:322; require de Scala antepone "requirement failed: "); WriteFile/WritePath: "requirement failed: WriteFile.position must be >= 0" y "requirement failed: WriteFile.count must be > 0" (:374-375 y :389-390; WritePath reutiliza el texto "WriteFile."). CommandFailed.toString = "CommandFailed(<cmd>)" + " because of <clase>: <msg>[, caused by: ..]" hasta 2 niveles de causa (:492-507). NoAck.token puede ser null (NoAck por defecto). ErrorClosed.getErrorCause = cause; isErrorClosed=true. Close/Abort/ConfirmedClose extienden DeadLetterSuppression.

**Orden / concurrencia / ciclo de vida.** CompoundWrite se procesa como una sola escritura atómica: los acks de cada sub-escritura se envían a medida que cada una se completa, en orden (ver AC-IO-002.5).

**Dependencias.** AC-UTIL ByteString, AC-DW (DeadLetterSuppression), Q-X-3

**Equivalente Rust propuesto.** `enum TcpCommand { Connect{..}, Bind{..}, Register{..}, Unbind, Close, ConfirmedClose, Abort, Write(Write), WritePath{..}, Compound(Vec<SimpleWrite>), ResumeWriting, SuspendReading, ResumeReading, ResumeAccepting{batch_size} }` y `enum TcpEvent { Received(ByteString), Connected{..}, CommandFailed{cmd, cause}, WritingResumed, Bound(..), Unbound, Closed, Aborted, ConfirmedClosed, PeerClosed, ErrorClosed(String), NoAck(Option<Token>) }`

**Diferencias observables.** Los comandos Rust son enums cerrados (en Pekko el trait Command no es sealed); Q-X-3 define si viajan como `Box<dyn Any>`. WriteFile (deprecado) se mapea a WritePath. El mensaje require pasa a un error de construcción (`Result`) con el mismo texto.

**Aserciones.**
- A1. Write.empty con Ack se acusa y sin ack/NoAck(42) no genera respuesta — TcpConnectionSpec.scala:235-251
- A2. CompoundWrite(Write("test1",Ack(1)), Write("test2"), Write(empty,Ack(3)), Write("test4",Ack(4))) escribe "test1test2test4" y responde Ack(1), Ack(3), Ack(4) en ese orden — TcpConnectionSpec.scala:277-298
- A3. Un Write con ack desconocido no responde (Write(ByteString(42)) sin ack no responde) — TcpConnectionSpec.scala:227-233
- A4. Textos de require y toString de CommandFailed — derivada del código Tcp.scala:322,374-375,389-390,492-507; no afirmada por spec

**Oráculo.** Jar oficial: construir mensajes y volcar toString/excepciones con jshell. Escenario no existe ni se ejecutó.

**Decisiones.** Q-X-3, Q-IO-3

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-002.2 — Tcp.Bind y TcpListener: Bound, backlog, puerto 0, batch-accept, pull mode (ResumeAccepting), Unbind/Unbound, fallo de bind

**Contrato observable.** Bind(handler, localAddress, backlog=100, options, pullMode=false). El listener vigila (death pact) al handler, abre ServerSocketChannel no bloqueante, aplica beforeServerSocketBind de las opciones, hace bind(local, backlog), registra OP_ACCEPT (0 si pullMode) y tras afterBind (SocketOptionV2) y recibir la ChannelRegistration envía Bound(localAddress real) al comandante. En ChannelAcceptable acepta hasta acceptLimit conexiones (BatchAcceptLimit; 0 en pull mode hasta ResumeAccepting), por cada una pide a los selectores un TcpIncomingConnection con handler=bind.handler; si queda cupo reactiva OP_ACCEPT. ResumeAccepting(n) fija acceptLimit=n y reactiva OP_ACCEPT. FailedRegisterIncoming -> warning y cierre del canal. Unbind -> cancelAndClose, responde Unbound al solicitante y detiene el listener.

**Fuente.** actor/src/main/scala/org/apache/pekko/io/TcpListener.scala:59 (death pact) ; actor/src/main/scala/org/apache/pekko/io/TcpListener.scala:61-94 (bind (catch 82-94)) ; actor/src/main/scala/org/apache/pekko/io/TcpListener.scala:98-102 (receive/Bound) ; actor/src/main/scala/org/apache/pekko/io/TcpListener.scala:104-127 (bound) ; actor/src/main/scala/org/apache/pekko/io/TcpListener.scala:128-133 (unregistering) ; actor/src/main/scala/org/apache/pekko/io/TcpListener.scala:135-152 (acceptAllPending) ; actor/src/main/scala/org/apache/pekko/io/TcpListener.scala:154-163 (postStop) ; actor/src/main/scala/org/apache/pekko/io/TcpManager.scala:61-69

**Condiciones.** Fallo de bind: CommandFailed(bind).withCause(e) al comandante; si e es java.net.BindException se re-crea con mensaje "[<localAddress>] <mensaje original>" (TcpListener.scala:84-90); log error "Bind failed for TCP channel on endpoint [{}]"; el listener se detiene. Dirección local no InetSocketAddress -> IllegalArgumentException("bound to unknown SocketAddress [..]"). Fallo de accept no fatal: log error y se corta el lote. Si el selector está lleno: warning "Could not register incoming connection since selector capacity limit is reached, closing connection".

**Orden / concurrencia / ciclo de vida.** El estado Bound solo se alcanza al recibir la ChannelRegistration (asíncrono). Las conexiones aceptadas se crean como hijos del selector vía WorkerForCommand(RegisterIncoming). Death pact: si el handler muere el listener se detiene con DeathPactException.

**Dependencias.** AC-IO-004.1/4.2 (selectores), AC-IO-002.4, AC-DW

**Equivalente Rust propuesto.** `TcpListener` actor con `accept_limit` y `TcpListenerCommand::ResumeAccepting(batch)`; bind con `socket2`/tokio y reporte `CommandFailed{cause: BindError("[addr] msg")}`.

**Diferencias observables.** En Rust el aviso de aceptación puede venir de tokio::net::TcpListener::accept; el modelo "registro con selector + interest ops" desaparece (Q-IO-1). El texto del BindException cambia al mensaje del SO.

**Aserciones.**
- A1. El listener registra su ServerSocketChannel con su selector al crearse — TcpListenerSpec.scala:38
- A2. Tras la ChannelRegistration el comandante recibe Bound — TcpListenerSpec.scala:40-47
- A3. Con batch-accept-limit=2 y 3 conexiones pendientes, un ChannelAcceptable crea 2 workers y re-pide OP_ACCEPT; el siguiente ChannelAcceptable crea el tercero — TcpListenerSpec.scala:49-67
- A4. Tras un accept sigue aceptando en el siguiente aviso — TcpListenerSpec.scala:69-83
- A5. En pull mode no acepta hasta ResumeAccepting(1); con batchSize 2 acepta 2 y luego deja de pedir OP_ACCEPT — TcpListenerSpec.scala:85-117
- A6. Unbind responde Unbound al solicitante y el listener termina — TcpListenerSpec.scala:119-127
- A7. FailedRegisterIncoming(channel) -> 1 warning ".*selector capacity limit.*" y el canal se cierra — TcpListenerSpec.scala:129-141
- A8. Bind sobre servidor completo responde CommandFailed con el Bind original — CapacityLimitSpec.scala:27-53 (ver AC-IO-004.2)
- A9. Mensaje del BindException con prefijo "[addr]" — derivada del código TcpListener.scala:84-90; no afirmada por spec

**Oráculo.** Jar oficial: jshell con Bind en puerto 0 y en puerto ocupado para volcar tipo y mensaje de la causa. Escenario no existe ni se ejecutó; mensaje dependiente del SO.

**Decisiones.** Q-IO-1, Q-IO-2

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-002.3 — Tcp.Connect y TcpOutgoingConnection: resolución DNS, timeout de conexión, finishConnect con reintentos, fallos como CommandFailed

**Contrato observable.** Connect(remoteAddress, localAddress=None, options=Nil, timeout=None, pullMode=false). El actor vigila al comandante, aplica beforeConnect de las opciones, bind local opcional y se registra con OP_CONNECT (initialOps 0 y luego enableInterest). Si la dirección no está resuelta pide a Dns.resolve(DnsProtocol.Resolve(host)); si hay caché conecta; si no, pasa a "resolving" y espera DnsProtocol.Resolved (o Failure del async-dns). connect() inmediato -> completeConnect; si no, espera ChannelConnectable y finishConnect (hasta FinishConnectRetries reintentos, reprogramando OP_CONNECT cada 1 ms). Si el timeout está definido arma ReceiveTimeout que, al vencer, falla. completeConnect aplica TCP_NODELAY=true (salvo SocketException: debug), afterConnect de opciones, envía Connected(remote, local) al comandante, arma ReceiveTimeout(RegisterTimeout) y pasa a esperar Register.

**Fuente.** actor/src/main/scala/org/apache/pekko/io/TcpOutgoingConnection.scala:51-56 (inicio) ; actor/src/main/scala/org/apache/pekko/io/TcpOutgoingConnection.scala:58-69 (stop/reportConnectFailure) ; actor/src/main/scala/org/apache/pekko/io/TcpOutgoingConnection.scala:71-89 (receive y DNS) ; actor/src/main/scala/org/apache/pekko/io/TcpOutgoingConnection.scala:91-103 (resolving) ; actor/src/main/scala/org/apache/pekko/io/TcpOutgoingConnection.scala:105-115 (register) ; actor/src/main/scala/org/apache/pekko/io/TcpOutgoingConnection.scala:117-142 (connecting) ; actor/src/main/scala/org/apache/pekko/io/TcpOutgoingConnection.scala:144-148 (timeout) ; actor/src/main/scala/org/apache/pekko/io/TcpOutgoingConnection.scala:153-157 (excepciones) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:224-250 (completeConnect)

**Condiciones.** Fallos -> stopWith(CloseInformation(Set(commander), CommandFailed(connect).withCause(cause)), shouldAbort=true). Causas: ConnectException("Connect timeout of <Option[FiniteDuration]> expired") (p. ej. "Connect timeout of Some(100 milliseconds) expired", TcpOutgoingConnection.scala:156-157); ConnectException("Could not establish connection because finishConnect never returned true") tras agotar finish-connect-retries (:153-154); RuntimeException(ex) si el async-dns responde Failure(ex) (:98-102); cualquier excepción no fatal de connect/DNS. Un host sin resolver (notthere.local) termina en CommandFailed(Connect(address)) dentro de 15 s.

**Orden / concurrencia / ciclo de vida.** El comandante recibe Connected o CommandFailed del propio actor de conexión (sender()). El resto de la fase de registro: ver AC-IO-002.7.

**Dependencias.** AC-IO-008.1 (Dns.resolve), AC-IO-002.4/7, AC-IO-005 (opciones)

**Equivalente Rust propuesto.** `TcpOutgoingConnection` actor: `connect_with_timeout`, `resolve_host(Dns)`, mapea errores a `CommandFailed{ cause }`.

**Diferencias observables.** Mensajes de ConnectException del SO/JVM no se igualan (Q-IO-3). finish-connect-retries (workaround Android) puede ser inocuo en Rust (Q-IO-3).

**Aserciones.**
- A1. Las opciones beforeConnect se aplican antes de conectar: ReuseAddress(true) visible en el socket — TcpConnectionSpec.scala:101-107
- A2. Destino no accesible (sin listener): CommandFailed(Connect(UnboundAddress)) al handler y el actor termina — TcpConnectionSpec.scala:630-650
- A3. Host no resoluble ("notthere.local"): CommandFailed(Connect(address)) en <=15 s — TcpConnectionSpec.scala:652-660
- A4. Con timeout=100ms y destino no accesible: CommandFailed(Connect(.., timeout=Some(100ms))) y el actor termina — TcpConnectionSpec.scala:662-672
- A5. Endpoint 192.0.2.1:23825 (no responde): no se recibe Connected (o CommandFailed) durante 1 s — TcpIntegrationSpec.scala:177-185
- A6. Connected(remoteAddress == endpoint) llega al connectCommander en un Connect normal — TcpIntegrationSpec.scala:187-194
- A7. Textos de causa: "Connect timeout of <timeout> expired" — derivada del código TcpOutgoingConnection.scala:156-157; no afirmada por spec (el spec solo compara el CommandFailed sin causa)

**Oráculo.** Jar oficial + sockets locales: Connect a puerto cerrado, a loopback, con timeout; el texto de la causa depende del SO. Escenario no existe ni se ejecutó.

**Decisiones.** Q-IO-3, Q-IO-4

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-002.4 — Register y flujo de lectura: Received, Suspend/ResumeReading, pull mode, max-received-message-size, direct-buffer-size

**Contrato observable.** Register(handler, keepOpenOnPeerClosed=false, useResumeWriting=true) lleva la conexión de "esperando registro" a "conectada": cambia el death pact del comandante al handler, cancela el ReceiveTimeout, y si no es pullMode (o ya se reanudó) empieza a leer. doRead lee en un ByteBuffer del pool hasta min(DirectBufferSize, ReceivedMessageSizeLimit restante) bytes, envía Received(ByteString) al handler por cada lectura con datos y, si la lectura llenó el buffer, en push mode sigue (hasta el límite por mensaje) y se auto-envía ChannelReadable; en pull mode devuelve MoreDataWaiting sin leer más (el handler debe enviar ResumeReading). SuspendReading deshabilita OP_READ; un ChannelReadable ya en vuelo se ignora mientras readingSuspended. ResumeReading rehabilita y lee.

**Fuente.** actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:76-92 (Register) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:94-98 (Resume/Suspend antes de Register) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:112-118 (connected) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:252-259 (suspend/resume) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:266-312 (doRead)

**Condiciones.** Datos que llegan antes de Register se entregan tras Register (spec: "receive data directly when the connection is established"). Lectura = -1 (EOF): ver AC-IO-002.6. IOException al leer: ErrorClosed (AC-IO-002.6). Un lote mayor que el buffer produce Received consecutivos: bufferSize y el resto (DataSize = bufferSize+1500 -> mensajes de 128 KiB y 1500 bytes con la config por defecto).

**Orden / concurrencia / ciclo de vida.** Orden de Received = orden de bytes del stream. En pull mode: un Received por ResumeReading (spec) y no hay lectura espontánea. Antes de Register, ResumeReading/SuspendReading solo cambian readingSuspended.

**Dependencias.** AC-IO-002.3, AC-IO-010 (pool), AC-IO-003 (settings)

**Equivalente Rust propuesto.** Conexión actor con `read_buf: BytesMut` limitado a `direct_buffer_size` y `max_received_message_size`, mensaje `TcpEvent::Received(ByteString)`.

**Diferencias observables.** Tamaños de lectura: Pekko lee del socket con búfer directo del tamaño configurado; con tokio puede leerse menos por llamada, pero el spec exige Received de exactamente bufferSize y luego el resto cuando hay datos disponibles; hay que fijar la lectura a "llenar hasta bufferSize si hay datos" para igualar.

**Aserciones.**
- A1. Los datos "testdata" llegan como Received; dos paquetes en vuelo llegan unidos como "testdata2testdata3" — TcpConnectionSpec.scala:122-134
- A2. Con DataSize = bufferSize+1500 escrito de una vez, el handler recibe Received(bufferSize) y luego Received(1500) tras un ChannelReadable — TcpConnectionSpec.scala:136-155
- A3. Datos enviados por el servidor justo tras accept llegan al handler tras Register (OP_READ pedido y Received "immediatedata") — TcpConnectionSpec.scala:158-176
- A4. SuspendReading deshabilita OP_READ; un ChannelReadable en vuelo no entrega datos; ResumeReading los entrega — TcpConnectionSpec.scala:362-384
- A5. Pull mode con buffer 1k: nada hasta ResumeReading; 1024 "t" luego 512 "u" en mensajes separados; un nuevo ResumeReading entrega lo nuevo — TcpConnectionSpec.scala:386-420

**Oráculo.** Jar oficial: servidor y cliente locales, tamaños de Received observados. Escenario no existe ni se ejecutó.

**Decisiones.** Q-IO-1

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-002.5 — Escritura: Write/WritePath/CompoundWrite, ack y NoAck, rechazo con CommandFailed, ResumeWriting/WritingResumed, useResumeWriting

**Contrato observable.** Con escritura pendiente nula y writingSuspended=false, un WriteCommand crea el PendingWrite del comandante (PendingBufferWrite o PendingWriteFile) e intenta escribir de inmediato; el ack (si wantsAck) se envía al comandante cuando el último byte de esa escritura se entregó al SO. Si ya hay una pendiente: CommandFailed(write).withCause(DroppingWriteBecauseQueueIsFullException) y, si useResumeWriting, writingSuspended=true; mientras writingSuspended todo write (incluso Write.empty) se rechaza con DroppingWriteBecauseWritingIsSuspendedException. ResumeWriting: pone writingSuspended=false; si hay escritura pendiente guarda al solicitante (el primero) y, cuando se vacía, le envía WritingResumed (un segundo solicitante recibe CommandFailed(ResumeWriting)); si no hay pendiente responde WritingResumed al instante. WritePath usa FileChannel.transferTo en el file-io-dispatcher con lotes de TransferToLimit bytes y notifica con UpdatePendingWriteAndThen. Una escritura vacía con ack se acusa al instante (sin tocar el socket); con NoAck no responde.

**Fuente.** actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:161-209 (handleWriteMessages) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:437-453 (PendingWrite) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:455-505 (PendingBufferWrite) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:507-549 (PendingWriteFile) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:597-601 (excepciones) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:341-367 (Write) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:388-391 (WritePath) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:425 (ResumeWriting) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:517-518 (WritingResumed)

**Condiciones.** Causas fijas: IOException("Dropping write because writing is suspended") y IOException("Dropping write because queue is full") (NoStackTrace, TcpConnection.scala:597-601). IOException durante write -> ErrorClosed al handler y al comandante de la escritura (ver AC-IO-002.6). Si useResumeWriting=false no hay suspensión: tras vaciar la cola las escrituras vuelven a funcionar. Con CompoundWrite: acks de sub-escrituras vacías con ack se envían al construir el PendingWrite (TcpConnection.scala:448-450).

**Orden / concurrencia / ciclo de vida.** El ack llega después de que los datos entran en el buffer del SO; una escritura grande se completa en varios ChannelWritable. Archivo: los lotes se encadenan vía mensajes al propio actor (UpdatePendingWriteAndThen) para no bloquear el selector.

**Dependencias.** AC-IO-002.1, AC-IO-010, AC-DSP (file-io-dispatcher)

**Equivalente Rust propuesto.** `TcpConnection` mantiene `pending_write: Option<PendingWrite>`; WritePath con `tokio::fs`/`spawn_blocking` y `sendfile` cuando exista.

**Diferencias observables.** Los ack significan "aceptado por el SO" tanto en Pekko como con tokio `write_all`; la capacidad de cola (cuántas escrituras caben antes del primer NACK) depende del buffer del SO y no es portable; los specs "backed up" llenan hasta el primer rechazo, por lo que se adaptan sin números fijos.

**Aserciones.**
- A1. Un Write con Ack responde Ack al comandante y los 8 bytes llegan; un Write sin ack no genera respuesta y los 10 bytes llegan — TcpConnectionSpec.scala:179-203
- A2. Un buffer de 512 KiB llega íntegro — TcpConnectionSpec.scala:205-225
- A3. Write.empty con Ack responde Ack; con NoAck / NoAck(42) no responde — TcpConnectionSpec.scala:235-251
- A4. WritePath de 10 MB responde Ack y los bytes llegan — TcpConnectionSpec.scala:253-275
- A5. CompoundWrite: bytes en orden y Ack(1), Ack(3), Ack(4) — TcpConnectionSpec.scala:277-298
- A6. ResumeWriting (backed up): tras NACK las escrituras (también Write.empty) fallan; ResumeWriting no responde hasta vaciar; luego WritingResumed y las escrituras funcionan — TcpConnectionSpec.scala:710-746
- A7. ResumeWriting (queue flushed): tras vaciar, las escrituras aún fallan hasta ResumeWriting, que responde WritingResumed al instante — TcpConnectionSpec.scala:748-782
- A8. useResumeWriting=false: tras NACK las escrituras fallan mientras haya cola; al vaciar vuelven a funcionar sin ResumeWriting — TcpConnectionSpec.scala:784-841
- A9. Escribir con el peer reseteado produce ErrorClosed al escritor y al handler — TcpConnectionSpec.scala:614-626
- A10. Ciclo request/response con acks Aye y Yes entre cliente y servidor reales — TcpIntegrationSpec.scala:134-154
- A11. Backpressure con buffers pequeños: 100000 bytes con Ack llegan — TcpIntegrationSpec.scala:156-175
- A12. "stop writing in cases of backpressure and resume afterwards": NO afirma (pending) — TcpConnectionSpec.scala:312-360 (marcada pending en :319)

**Oráculo.** Jar oficial: se pueden comparar secuencias de mensajes (Ack/CommandFailed/WritingResumed) en loopback; los tamaños de cola dependen del SO. Escenario no existe ni se ejecutó.

**Decisiones.** Q-IO-3

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-002.6 — Cierre: Close, ConfirmedClose, Abort, PeerClosed, ErrorClosed, keepOpenOnPeerClosed y quién recibe la notificación

**Contrato observable.** handleClose(info, closeCommander, evento): Aborted -> cierra de inmediato con RST (SO_LINGER 0). PeerClosed con keepOpenOnPeerClosed -> envía PeerClosed al handler, marca peerClosed y pasa a peerSentEOF (siguen las escrituras; Close/ConfirmedClose posteriores cierran). Con escritura pendiente -> suelta el death pact, pasa a closingWithPendingWrite y cierra al vaciar. ConfirmedClosed: shutdownOutput; si el peer ya cerró (o shutdownOutput falla) cierra ya, si no pasa a "closing" esperando EOF del peer (entonces ConfirmedClosed). Otro cierre: cierra ya. En lectura EOF: si la salida ya estaba cerrada -> ConfirmedClosed, si no PeerClosed. El cierre efectivo es stopWith: cancelAndClose del registro y, al parar el actor (postStop), se notifica el evento de cierre al handler y al comandante del Close (Set -> si son el mismo actor, una sola vez) y a quien tenía una escritura pendiente. IOException -> ErrorClosed(mensaje, tomado de la primera causa con mensaje no vacío) a handler (y al escritor pendiente).

**Fuente.** actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:120-125 (peerSentEOF) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:128-150 (closingWithPendingWrite) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:153-159 (closing) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:320-349 (handleClose) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:351-358 (doCloseConnection/handleError) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:360-375 (safeShutdownOutput/extractMsg) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:377-407 (prepareAbort/stopWith) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:409-432 (postStop)

**Condiciones.** Mensajes: ErrorClosed(cause) con cause = mensaje del IOException (p. ej. "Connection reset by peer" en Linux, según plataforma). Aborted -> setSoLinger(true,0): si falla, se ignora con debug. postRestart lanza IllegalStateException("Restarting not supported for connection actors.") (:434-435). Conexión ya cerrada por SO: se notifica directamente.

**Orden / concurrencia / ciclo de vida.** La notificación sale en postStop (después de cancelAndClose si el canal sigue abierto). Close con escritura pendiente: Ack de la escritura, luego Closed. Un Close del handler produce exactamente un Closed. PoisonPill al actor de conexión equivale a Abort (el peer ve ErrorClosed).

**Dependencias.** AC-IO-002.5, AC-IO-002.7, AC-IO-004.4, AC-IO-005

**Equivalente Rust propuesto.** `TcpConnection::handle_close(info, closer, ClosedEvent)` con `ClosedEvent::{Closed, Aborted, ConfirmedClosed, PeerClosed, ErrorClosed(String)}`; RST con `SO_LINGER=0` vía socket2.

**Diferencias observables.** RST y mensaje de error por plataforma: el cliente que recibe RST ve ErrorClosed con texto del SO; los specs usan constantes por plataforma (ConnectionResetByPeerMessage); en Rust el texto proviene de `std::io::Error` y diferirá.

**Aserciones.**
- A1. Close con escritura pendiente (buffer chico): Ack, Closed al handler y al comandante del Close, actor terminado, el servidor lee EOF (-1) — TcpConnectionSpec.scala:422-447
- A2. Close del handler genera un solo Closed (sin duplicado en 500 ms) — TcpConnectionSpec.scala:449-456
- A3. Abort responde Aborted, el actor termina y el servidor ve IOException con el mensaje de reset — TcpConnectionSpec.scala:458-473
- A4. ConfirmedClose (simplificado): Ack de escritura, EOF en el servidor, ConfirmedClosed tras cerrar el servidor — TcpConnectionSpec.scala:487-516
- A5. ConfirmedClose completo: nada antes de Ack, luego ConfirmedClosed tras que el servidor cierre — TcpConnectionSpec.scala:518-551
- A6. Peer cierra -> PeerClosed al handler y el actor termina — TcpConnectionSpec.scala:553-562
- A7. keepOpenOnPeerClosed: PeerClosed y aún se puede escribir (Ack) y cerrar con Close -> Closed o ConfirmedClose -> ConfirmedClosed — TcpConnectionSpec.scala:564-598
- A8. Peer aborta -> ErrorClosed con cause == ConnectionResetByPeerMessage; sin más mensajes — TcpConnectionSpec.scala:600-612
- A9. Integración: Close del cliente -> Closed al cliente y PeerClosed al servidor; Abort -> Aborted y ErrorClosed al otro lado; PoisonPill -> el otro lado recibe ErrorClosed; ambos actores terminan — TcpIntegrationSpec.scala:45-132

**Oráculo.** Jar oficial en loopback: secuencias de eventos de cierre; textos de ErrorClosed por plataforma. Escenario no existe ni se ejecutó.

**Decisiones.** Q-IO-3, Q-IO-1

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-002.7 — Ciclo de vida de conexión: death pact con comandante y handler, register-timeout, estado "unregistering" y manejo de muerte del actor

**Contrato observable.** signDeathPact(actor) hace context.watch del comandante (al crear) y luego del handler (al Register, si es distinto). Si el vigilado termina sin que la conexión haya cerrado, llega un Terminated no manejado y el actor falla con DeathPactException; el supervisor (connectionSupervisorStrategy) detiene el hijo y registra a nivel debug "Closed after handler termination" en vez de error. Al cerrar normalmente se retira el pact. En waitingForRegistration el ReceiveTimeout (RegisterTimeout) hace stop con debug "Configured registration timeout of [..] expired, stopping"; un cierre pedido antes de Register usa comandante como handler. Si el actor muere sin pasar por stopWith y el canal sigue abierto, postStop hace prepareAbort (RST).

**Fuente.** actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:58-71 (death pact) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:76-109 (waitingForRegistration (timeout 104-108)) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:211-218 (unregistering) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:393-435 (stopWith/postStop) ; actor/src/main/scala/org/apache/pekko/io/SelectionHandler.scala:136-149 (connectionSupervisorStrategy)

**Condiciones.** register-timeout = infinite -> sin timeout (Duration.Undefined). Si el handler muere con el peer conectado: se espera un único DeathPactException (EventFilter) y Terminated de ambos. Si se detiene el actor antes del Register (PoisonPill): el canal se cierra (RST; el servidor lee IOException).

**Orden / concurrencia / ciclo de vida.** Los eventos de cierre se notifican en postStop. El registro con el selector se cancela antes de notificar (cancelAndClose + Unregistered).

**Dependencias.** AC-DW (DeathWatch), AC-SUP, AC-IO-004.3

**Equivalente Rust propuesto.** Actor con `watch` y `receive_timeout(register_timeout)`; supervisor de conexiones con directiva Stop y log debug para DeathPact.

**Diferencias observables.** Ninguna esperada si el runtime de actores porta DeathWatch y ReceiveTimeout; DeathPactException hereda de AC-DW.

**Aserciones.**
- A1. Si Connected no se responde con Register, el actor termina (register-timeout) — TcpConnectionSpec.scala:674-685
- A2. Handler muere mientras se conecta: 1 DeathPactException y el actor de conexión termina — TcpConnectionSpec.scala:687-696
- A3. Handler muere conectado: 1 DeathPactException; terminan handler y conexión — TcpConnectionSpec.scala:698-708
- A4. PoisonPill a la conexión antes de Register: termina y el servidor ve IOException al leer — TcpConnectionSpec.scala:843-869
- A5. PoisonPill a la conexión establecida: el servidor lee -1 o IOException y el actor termina — TcpIntegrationSpec.scala:187-204
- A6. Texto debug "Configured registration timeout of [..] expired, stopping" y "Closed after handler termination" — derivada del código TcpConnection.scala:107; SelectionHandler.scala:145; no afirmada por spec

**Oráculo.** Jar oficial: loopback; se puede comparar secuencia de Terminated/eventos. Escenario no existe ni se ejecutó.

**Decisiones.** Q-IO-1

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-003 — Configuración pekko.io.tcp: TcpExt.Settings y SelectionHandlerSettings (claves, validaciones y mensajes)

**Contrato observable.** Defaults: nr-of-selectors=1, max-channels=256000 ("unlimited" -> -1), selector-association-retries=10, batch-accept-limit=10, direct-buffer-size=128 KiB, direct-buffer-pool-limit=1000, register-timeout=5s ("infinite" -> Duration.Undefined), max-received-message-size=unlimited (-> Int.MaxValue), trace-logging=off, selector-dispatcher=pekko.io.pinned-dispatcher, worker-dispatcher y management-dispatcher=pekko.actor.internal-dispatcher, file-io-dispatcher=pekko.actor.default-blocking-io-dispatcher, file-io-transferTo-limit=512 KiB ("unlimited" -> Int.MaxValue), finish-connect-retries=5, windows-connection-abort-workaround-enabled=off (on|off|auto; auto = Helpers.isWindows). MaxChannelsPerSelector = max(MaxChannels / NrOfSelectors, 1) (o -1).

**Fuente.** actor/src/main/scala/org/apache/pekko/io/Tcp.scala:606-650 (Settings (nr-of-selectors 614, batch 616, buffers 617-618, register-timeout 619-622, max-received 623-626, transferTo 629-632, per-selector 634, retries 635-636, windows 638-642, getIntBytes 644-649)) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:654-667 (manager/bufferPool/fileIoDispatcher) ; actor/src/main/scala/org/apache/pekko/io/SelectionHandler.scala:39-54 ; actor/src/main/resources/reference.conf:940-1026

**Condiciones.** IllegalArgumentException con el texto indicado: "nr-of-selectors must be > 0" (Tcp.scala:614), "batch-accept-limit must be > 0" (:616), "finish-connect-retries must be > 0" (:636), "<ruta> must be < 2 GiB" y "<ruta> must be non-negative" (getIntBytes :646-647, p. ej. direct-buffer-size, max-received-message-size, file-io-transferTo-limit), "max-channels must be > 0 or 'unlimited'" (SelectionHandler.scala:44), "selector-association-retries must be >= 0" (:47). Un valor no numérico -> ConfigException.WrongType (derivado).

**Orden / concurrencia / ciclo de vida.** Settings se evalúan al construir TcpExt (acceso a IO(Tcp)).

**Dependencias.** AC-CFG-006 (getBytes, duraciones), AC-DSP (dispatchers nombrados), Q-X-1

**Equivalente Rust propuesto.** `TcpSettings { nr_of_selectors, max_channels: Option<usize>, selector_association_retries, batch_accept_limit, direct_buffer_size, direct_buffer_pool_limit, register_timeout: Option<Duration>, max_received_message_size: usize, trace_logging, .. }`

**Diferencias observables.** selector/worker/management/file-io dispatcher son nombres de dispatcher JVM; el valor se acepta pero su efecto depende del modelo de ejecución (Q-IO-1). windows-connection-abort-workaround y finish-connect-retries no tienen efecto probable (Q-IO-3).

**Aserciones.**
- A1. Con max-channels=4 (config del spec) un Bind y un Connect que excedan la capacidad reciben CommandFailed con el comando original — CapacityLimitSpec.scala:27-53
- A2. batch-accept-limit=2 (config de TcpListenerSpec) limita el lote de accept a 2 — TcpListenerSpec.scala:49-67
- A3. Los defaults y mensajes de validación listados — derivada del código Tcp.scala:609-649 y reference.conf; no afirmada por spec

**Oráculo.** Jar oficial: instanciar TcpExt con configs inválidas y volcar mensajes. Escenario no existe ni se ejecutó.

**Decisiones.** Q-X-1, Q-IO-1, Q-IO-3

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-004.1 — TcpManager/UdpManager/UdpConnectedManager y SelectorBasedManager: pool de selectores "selectors" y enrutamiento de comandos

**Contrato observable.** workerForCommandHandler(pf) convierte cada comando HasFailureMessage definido en pf en un WorkerForCommand(cmd, sender, childProps) enviado a selectorPool, un RandomPool(nrOfSelectors) con nombre hijo "selectors" y deploy local. TcpManager: Connect -> TcpOutgoingConnection, Bind -> TcpListener(selectorPool,...). UdpManager: Bind -> UdpListener, SimpleSender -> UdpSender. UdpConnectedManager: Connect -> UdpConnection. Comandos fuera de pf quedan sin manejar.

**Fuente.** actor/src/main/scala/org/apache/pekko/io/SelectionHandler.scala:111-130 ; actor/src/main/scala/org/apache/pekko/io/TcpManager.scala:61-69 ; actor/src/main/scala/org/apache/pekko/io/UdpManager.scala:59-67 ; actor/src/main/scala/org/apache/pekko/io/UdpConnectedManager.scala:26-30

**Condiciones.** Comando desconocido -> unhandled (evento UnhandledMessage); el commander se captura con sender() antes del callback asíncrono.

**Orden / concurrencia / ciclo de vida.** El pool elige un selector al azar; cada selector es un actor con su propio bucle (ver AC-IO-004.4).

**Dependencias.** AC-RT (RandomPool), AC-IO-004.2, AC-IO-004.3

**Equivalente Rust propuesto.** `SelectorPoolManager` con `Vec<SelectorHandle>` y elección aleatoria; o runtime tokio con un solo reactor (Q-IO-1).

**Diferencias observables.** Con tokio, nr-of-selectors pierde significado observable (Q-IO-1).

**Aserciones.**
- A1. Reparto de Connect/Bind al pool y nombre "selectors" — derivada del código SelectionHandler.scala:116-129; TcpManager.scala:61-69; no afirmada por spec

**Oráculo.** Jar oficial: listar hijos de /system/IO-TCP. Escenario no existe ni se ejecutó.

**Decisiones.** Q-IO-1

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-004.2 — Protección de capacidad del selector: max-channels, selector-association-retries, Retry y CommandFailed

**Contrato observable.** SelectionHandler cuenta hijos (childCount). Con MaxChannelsPerSelector == -1 o childCount < límite crea un hijo con nombre secuencial (sequenceNumber.toString), dispatcher WorkerDispatcher y Deploy.local; solo vigila hijos si hay límite. Si está lleno y quedan reintentos: debug "Rejecting [..] with [n] retries left, retrying..." y reenvía Retry(cmd, n-1) al padre (el pool); sin reintentos: warning "Rejecting [..] with no retries left, aborting..." y envía cmd.apiCommand.failureMessage (CommandFailed(cmd)) al comandante.

**Fuente.** actor/src/main/scala/org/apache/pekko/io/SelectionHandler.scala:96-104 (WorkerForCommand/Retry) ; actor/src/main/scala/org/apache/pekko/io/SelectionHandler.scala:313-335 (receive) ; actor/src/main/scala/org/apache/pekko/io/SelectionHandler.scala:367-386 (spawnChildWithCapacityProtection)

**Condiciones.** Retry exige retriesLeft >= 0 (require). El límite es "soft": puede aceptar algo más o menos por reparto aleatorio.

**Orden / concurrencia / ciclo de vida.** Terminated de un hijo decrementa childCount (solo si se vigila).

**Dependencias.** AC-IO-003, AC-IO-004.1

**Equivalente Rust propuesto.** Contador atómico de canales por selector y `CommandFailed` tras N reintentos.

**Diferencias observables.** En un runtime sin selectores el límite de canales es una cuenta global; los reintentos entre selectores pierden sentido (Q-IO-1).

**Aserciones.**
- A1. Con max-channels=4: tras listener+conexión servidor+conexión cliente y un Bind extra, otro Bind recibe CommandFailed(bind) (misma instancia) y un Connect también — CapacityLimitSpec.scala:27-53
- A2. Textos de log "Rejecting [..] with [..] retries left" / "no retries left" — derivada del código SelectionHandler.scala:379-383; no afirmada por spec

**Oráculo.** Jar oficial: loopback con max-channels pequeño. Escenario no existe ni se ejecutó.

**Decisiones.** Q-IO-1

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-004.3 — Estrategias de supervisión de IO: connectionSupervisorStrategy y estrategia de SelectionHandler (stop en toda Exception, log a debug)

**Contrato observable.** connectionSupervisorStrategy = OneForOneStrategy con el decider de stoppingStrategy; logFailure publica Logging.Debug "Closed after handler termination" si la causa es DeathPactException, o deja el log por defecto. SelectionHandler: OneForOne con decider {case _: Exception => Stop} y logFailure que publica Debug con el mensaje de ActorInitializationException.cause / simpleName / getMessage.

**Fuente.** actor/src/main/scala/org/apache/pekko/io/SelectionHandler.scala:136-149 ; actor/src/main/scala/org/apache/pekko/io/SelectionHandler.scala:341-365

**Condiciones.** Solo Exception se detiene; Error/Throwable siguen el decider por defecto de la estrategia (escalan) en connectionSupervisorStrategy; ver AC-SUP.

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AC-SUP-004, Q-X-5, AC-EVT (Debug)

**Equivalente Rust propuesto.** Estrategia `Stop` para hijos IO con un filtro de log.

**Diferencias observables.** Depende de Q-X-5 (Err/panic).

**Aserciones.**
- A1. Todo hijo con excepción se detiene sin reinicio; DeathPact a nivel debug — derivada del código SelectionHandler.scala:136-149,341-365; no afirmada por spec salvo el efecto DeathPact en TcpConnectionSpec.scala:687-708

**Oráculo.** Jar oficial: provocar fallo en hijo IO y observar eventos. Escenario no existe ni se ejecutó.

**Decisiones.** Q-X-5

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-004.4 — Bucle de selección y ChannelRegistry: interest ops, MaxSelectMillis=10000, cancelAndClose y mensajes ChannelReadable/Writable/Acceptable/Connectable

**Contrato observable.** ChannelRegistry.register(channel, initialOps)(channelActor) registra en el selector y devuelve al actor un ChannelRegistration (enableInterest/disableInterest/cancelAndClose(andThen)). El bucle select(10000 ms) despacha por readyOps: OP_READ -> ChannelReadable, OP_WRITE -> ChannelWritable, ambos -> ChannelWritable luego ChannelReadable, OP_ACCEPT -> ChannelAcceptable, OP_CONNECT -> ChannelConnectable; antes de enviar borra el interés del key (evita re-selección). Las operaciones de interés se ejecutan siempre en el hilo del selector. cancelAndClose cancela la key, cierra el canal y ejecuta "andThen" tras el siguiente select.

**Fuente.** actor/src/main/scala/org/apache/pekko/io/SelectionHandler.scala:59-82 (interfaces) ; actor/src/main/scala/org/apache/pekko/io/SelectionHandler.scala:90 (MaxSelectMillis) ; actor/src/main/scala/org/apache/pekko/io/SelectionHandler.scala:106-109 (mensajes) ; actor/src/main/scala/org/apache/pekko/io/SelectionHandler.scala:161-198 (bucle) ; actor/src/main/scala/org/apache/pekko/io/SelectionHandler.scala:202-223 (register) ; actor/src/main/scala/org/apache/pekko/io/SelectionHandler.scala:225-239 (shutdown) ; actor/src/main/scala/org/apache/pekko/io/SelectionHandler.scala:241-291 (interest/cancelAndClose) ; actor/src/main/scala/org/apache/pekko/io/SelectionHandler.scala:293-309 (Task) ; actor-tests/src/test/scala/org/apache/pekko/io/TcpConnectionSpec.scala:883-960 (LocalServerTest (ChannelRegistry simulado))

**Condiciones.** readyOps inválido -> warning "Invalid readyOps: [..]". CancelledKeyException se ignora. Errores de tarea: log error "Error during selector management task: [..]".

**Orden / concurrencia / ciclo de vida.** Un único hilo por selector (pinned-dispatcher); MaxSelectMillis fuerza despertar cada 10 s para liberar referencias a actores muertos.

**Dependencias.** AC-DSP-011 (pinned-dispatcher), AC-IO-004.1

**Equivalente Rust propuesto.** Reactor sobre `mio::Poll` con un tipo `ChannelRegistration` que expone enable/disable de interés; o tokio (Q-IO-1).

**Diferencias observables.** Con tokio el modelo edge-triggered de readiness difiere; los ChannelReadable/Writable no existen en el API público.

**Aserciones.**
- A1. Traducción readyOps -> mensajes y limpieza de interés — derivada del código SelectionHandler.scala:161-198; no afirmada por spec (los specs inyectan los mensajes a mano)

**Oráculo.** No aplica: interno de NIO.

**Decisiones.** Q-IO-1, Q-IO-2

**Estado.** analysis=listed; impl=pending; validation=pending.

### AC-IO-005 — Opciones de socket: Inet.SocketOption(+V2), Inet.SO.*, Tcp.SO.*, Udp.SO.Broadcast, DatagramChannelCreator, forwarders Scala y fábricas Java

**Contrato observable.** SocketOption define 4 hooks vacíos (beforeDatagramBind, beforeServerSocketBind, beforeConnect, afterConnect); SocketOptionV2 añade afterBind (DatagramSocket, ServerSocket) y afterConnect(DatagramSocket). SO.ReceiveBufferSize(size>0): antes de bind de servidor/datagrama y antes de connect; SO.ReuseAddress(on): en los tres; SO.SendBufferSize(size>0): afterConnect; SO.TrafficClass(0..255): afterConnect. Tcp.SO.KeepAlive/OOBInline/TcpNoDelay: afterConnect. Udp.SO.Broadcast: beforeDatagramBind. DatagramChannelCreator.create() abre el DatagramChannel (la primera opción de ese tipo gana). Forwarders: val ReceiveBufferSize = SO.ReceiveBufferSize, etc.

**Fuente.** actor/src/main/scala/org/apache/pekko/io/Inet.scala:30-51 (SocketOption) ; actor/src/main/scala/org/apache/pekko/io/Inet.scala:59-76 (V2) ; actor/src/main/scala/org/apache/pekko/io/Inet.scala:86-101 (DatagramChannelCreator) ; actor/src/main/scala/org/apache/pekko/io/Inet.scala:103-152 (SO) ; actor/src/main/scala/org/apache/pekko/io/Inet.scala:154-185 (SoForwarders) ; actor/src/main/scala/org/apache/pekko/io/Inet.scala:187-219 (SoJavaFactories) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:68-106 (Tcp.SO) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:672-700 (TcpSO) ; actor/src/main/scala/org/apache/pekko/io/Udp.scala:197-208 (Broadcast)

**Condiciones.** IllegalArgumentException("requirement failed: ReceiveBufferSize must be > 0") (Inet.scala:111), "requirement failed: SendBufferSize must be > 0" (:136), "requirement failed: TrafficClass needs to be in the interval [0, 255]" (:148). TCP_NODELAY está habilitado por defecto en completeConnect antes de las opciones del usuario (TcpConnection.scala:231). El spec "set socket options after connecting" está `ignore`.

**Orden / concurrencia / ciclo de vida.** beforeX corre antes de bind/connect; afterConnect tras conectar y antes de enviar Connected; afterBind tras recibir bind exitoso.

**Dependencias.** AC-IO-002.2/3, AC-IO-006.1/2

**Equivalente Rust propuesto.** `trait SocketOption { fn before_bind(&self,&Socket); fn before_connect(..); fn after_connect(..); fn after_bind(..) }` con `socket2`; `DatagramSocketFactory`.

**Diferencias observables.** java.net.Socket vs socket2::Socket: semántica de ciertas opciones (OOBINLINE, TrafficClass) depende del SO. DatagramChannelCreator pasa a un trait factoría (Q-IO-3).

**Aserciones.**
- A1. ReuseAddress(true) está activo en el socket cliente tras crear el actor de conexión (beforeConnect) — TcpConnectionSpec.scala:101-107
- A2. SO.KeepAlive tras conectar: test `ignore` (no afirma) — TcpConnectionSpec.scala:109-120
- A3. beforeDatagramBind se llama una vez antes del bind (ds no está bound) — UdpIntegrationSpec.scala:108-114,134-141
- A4. afterBind se llama una vez tras el bind exitoso (s.isBound) — UdpIntegrationSpec.scala:116-122,143-152
- A5. DatagramChannelCreator.create se llama una vez al abrir el canal — UdpIntegrationSpec.scala:124-130,154-163
- A6. Textos de require de las opciones — derivada del código Inet.scala:111,136,148; no afirmada por spec

**Oráculo.** Jar oficial: jshell con opciones y lectura del socket subyacente (solo en tests internos). Escenario no existe ni se ejecutó.

**Decisiones.** Q-IO-3

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-006.1 — Udp.Bind y UdpListener: Bound, recepción por lotes (receive-throughput), SuspendReading/ResumeReading, Unbind/Unbound, fallo de bind

**Contrato observable.** Bind(handler, localAddress, options): el listener vigila al handler, crea el canal con DatagramChannelCreator (opción) no bloqueante, beforeDatagramBind, bind, registra OP_READ y tras afterBind recibe ChannelRegistration y envía Bound(local real). ChannelReadable lee hasta BatchReceiveLimit+1 datagramas por aviso (innerReceive con readsLeft) y los envía como Received(ByteString, sender) al handler, luego rehabilita OP_READ. SuspendReading/ResumeReading deshabilitan/habilitan OP_READ. Unbind -> cancelAndClose, Unbound al solicitante, stop. Además atiende Send (ver AC-IO-006.2).

**Fuente.** actor/src/main/scala/org/apache/pekko/io/UdpListener.scala:45-76 (inicio y bind) ; actor/src/main/scala/org/apache/pekko/io/UdpListener.scala:78-82 (receive) ; actor/src/main/scala/org/apache/pekko/io/UdpListener.scala:84-93 (readHandlers) ; actor/src/main/scala/org/apache/pekko/io/UdpListener.scala:95-100 (unregistering) ; actor/src/main/scala/org/apache/pekko/io/UdpListener.scala:102-124 (doReceive) ; actor/src/main/scala/org/apache/pekko/io/UdpListener.scala:126-134 (postStop) ; actor/src/main/scala/org/apache/pekko/io/Udp.scala:113-125 (Bind/Unbind) ; actor/src/main/scala/org/apache/pekko/io/Udp.scala:147-153 (Suspend/Resume) ; actor/src/main/scala/org/apache/pekko/io/Udp.scala:164-190 (eventos)

**Condiciones.** Fallo de bind: CommandFailed(bind) SIN causa (a diferencia de TCP) + log error "Failed to bind UDP channel to endpoint [{}]" y stop (UdpListener.scala:72-76). Datagrama más grande que direct-buffer-size se trunca (derivado de ByteBuffer.limit en :105).

**Orden / concurrencia / ciclo de vida.** Los datagramas se entregan en orden de recepción del socket. Un listener atiende lectura y envío; Unbind pendiente durante un envío no se especifica.

**Dependencias.** AC-IO-004.1, AC-IO-005, AC-IO-006.2

**Equivalente Rust propuesto.** `UdpListener` con `recv_from` en bucle limitado por `receive_throughput` y `Received{data, sender}`.

**Diferencias observables.** Orden/lote: tokio `recv_from` no tiene noción de "readsLeft"; se debe ceder tras BatchReceiveLimit+1 lecturas para igualar la equidad (Q-IO-1).

**Aserciones.**
- A1. Bind a 127.0.0.1:0 responde Bound con el puerto real — UdpIntegrationSpec.scala:31-35,53-60
- A2. Bind sobre una dirección concreta responde Bound(address) — UdpIntegrationSpec.scala:37-42,75-106
- A3. Cliente y servidor bindeados intercambian 20 paquetes en cada dirección y 20 alternados; Received lleva la dirección del emisor — UdpIntegrationSpec.scala:75-106
- A4. Unbind responde Unbound y se puede volver a hacer bind en la misma dirección — UdpConnectedIntegrationSpec.scala:116-136
- A5. CommandFailed(bind) sin causa en fallo — derivada del código UdpListener.scala:72-76; no afirmada por spec

**Oráculo.** Jar oficial en loopback. Escenario no existe ni se ejecutó.

**Decisiones.** Q-IO-1

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-006.2 — Udp.Send y SimpleSender (WithUdpSend/UdpSender): ack, cola de una posición, reintento por OP_WRITE, resolución de nombres

**Contrato observable.** Send(payload, target, ack=NoAck) exige ack != null. Con una Send pendiente (hasWritePending) responde CommandFailed(send); con payload vacío solo acusa (ack si wantsAck) sin tocar el socket; si no, la guarda y, si target.isUnresolved, usa Dns.resolve: con caché hit reemplaza el target y envía; con miss responde CommandFailed(send) de inmediato (aunque Dns.resolve ya envió la solicitud, cuya respuesta Resolved llegará al listener/sender como mensaje no manejado). doSend copia el payload a un buffer del pool, channel.send; si escribe 0 bytes: primer intento -> enableInterest(OP_WRITE) y retriedSend=true; segundo -> CommandFailed(pendingSend); si escribe >0: ack si wantsAck. SimpleSender(options): el UdpSender abre un DatagramChannel (con creator opcional), aplica beforeDatagramBind, registra con initialOps=0, aplica afterConnect de SocketOptionV2 y responde SimpleSenderReady al comandante.

**Fuente.** actor/src/main/scala/org/apache/pekko/io/WithUdpSend.scala:46-84 (sendHandlers) ; actor/src/main/scala/org/apache/pekko/io/WithUdpSend.scala:86-115 (doSend) ; actor/src/main/scala/org/apache/pekko/io/UdpSender.scala:40-63 ; actor/src/main/scala/org/apache/pekko/io/UdpSender.scala:65-71 (postStop) ; actor/src/main/scala/org/apache/pekko/io/Udp.scala:98-105 (Send) ; actor/src/main/scala/org/apache/pekko/io/Udp.scala:137-139 (SimpleSender) ; actor/src/main/scala/org/apache/pekko/io/Udp.scala:182-183 (SimpleSenderReady)

**Condiciones.** Send: IllegalArgumentException("requirement failed: ack must be non-null. Use NoAck if you don't want acks.") (Udp.scala:99). Dirección sin resolver: CommandFailed(send) y debug "Name resolution failed for remote address [{}]" (WithUdpSend.scala:73-77). Excepciones no fatales al enviar tras resolver -> CommandFailed(send) y debug "Failure while sending UDP datagram to remote address [{}]: {}" (:64-70); sin más estado para el resto (pendingSend=null). Un datagrama mayor al buffer directo se trunca silenciosamente (derivado).

**Orden / concurrencia / ciclo de vida.** Una Send a la vez por actor; el spec verifica que tras un CommandFailed por resolución se pueden enviar más datagramas.

**Dependencias.** AC-IO-008.1, AC-IO-010

**Equivalente Rust propuesto.** `UdpSender` con `send_to`; resolución previa con `DnsResolver::cached`.

**Diferencias observables.** Q-IO-5: el primer Send a un host sin caché siempre falla en Pekko, incluso con DNS válido. Replicar o corregir es decisión del Owner.

**Aserciones.**
- A1. SimpleSender responde SimpleSenderReady y Send(data, serverAddress) llega como Received al servidor — UdpIntegrationSpec.scala:44-60
- A2. Un Send a host irresoluble responde CommandFailed(cmd) y un Send posterior al servidor real llega — UdpIntegrationSpec.scala:62-73
- A3. Primer intento de envío con 0 bytes escritos: reintento por OP_WRITE y luego CommandFailed — derivada del código WithUdpSend.scala:96-111; no afirmada por spec
- A4. Send con payload vacío solo acusa — derivada del código WithUdpSend.scala:51-53; no afirmada por spec

**Oráculo.** Jar oficial en loopback; el comportamiento de "primer Send a nombre" se puede fijar con `localhost` por nombre no resuelto. Escenario no existe ni se ejecutó.

**Decisiones.** Q-IO-5, Q-IO-1

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-006.3 — Configuración pekko.io.udp y UdpSettings (nr-of-selectors, max-channels=4096, select-timeout, receive-throughput, buffers, dispatchers)

**Contrato observable.** Defaults: nr-of-selectors=1, max-channels=4096, select-timeout=infinite (clave NO leída por UdpSettings), selector-association-retries=10, receive-throughput=3, direct-buffer-size=128 KiB, direct-buffer-pool-limit=1000, trace-logging=off, dispatchers como TCP. BatchReceiveLimit = receive-throughput.

**Fuente.** actor/src/main/scala/org/apache/pekko/io/Udp.scala:210-227 (UdpSettings) ; actor/src/main/scala/org/apache/pekko/io/Udp.scala:230-252 (UdpExt) ; actor/src/main/resources/reference.conf:1028-1082

**Condiciones.** IllegalArgumentException "nr-of-selectors must be > 0" (Udp.scala:213) y "<ruta> must be < 2 GiB" (:224); a diferencia de TCP no valida non-negative. max-channels/selector-association-retries heredan validaciones de AC-IO-003.

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AC-IO-003

**Equivalente Rust propuesto.** `UdpSettings { nr_of_selectors, max_channels, receive_throughput, direct_buffer_size, direct_buffer_pool_limit, .. }`

**Diferencias observables.** select-timeout se acepta sin efecto (igual que Pekko); queda como clave documentada.

**Aserciones.**
- A1. Defaults y mensajes de validación — derivada del código Udp.scala:213-226 y reference.conf; no afirmada por spec

**Oráculo.** Jar oficial: instanciar UdpExt con configs inválidas. Escenario no existe ni se ejecutó.

**Decisiones.** Q-IO-3

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-007 — UDP conectado: UdpConnected (Connect/Send/Disconnect/SuspendReading/ResumeReading, Connected/Received/Disconnected/CommandFailed) y UdpConnection

**Contrato observable.** Connect(handler, remoteAddress, localAddress=None, options): vigila al handler; si remoteAddress.isUnresolved usa Dns.resolve (cache hit: doConnect; miss: espera Resolved/Failure); doConnect abre DatagramChannel no bloqueante, beforeDatagramBind, bind local opcional, channel.connect(remoteAddress), registra OP_READ. Con ChannelRegistration: afterConnect de V2 y Connected al comandante. connected: Send con pendiente -> CommandFailed; payload vacío -> ack sin socket; si no, guarda y habilita OP_WRITE; ChannelWritable -> doWrite (0 bytes escritos -> CommandFailed(send); si no ack si wantsAck). Lectura como Received(data) por datagrama hasta que read devuelve 0: innerRead decrementa readsLeft pero nunca lo comprueba (UdpConnection.scala:132-141), así que receive-throughput NO limita el lote en UdpConnected (a diferencia de UdpListener); PortUnreachableException al leer se ignora (debug). Disconnect: cierra el canal, responde Disconnected y detiene el actor. Fallo de conexión -> CommandFailed(connect) al comandante y stop.

**Fuente.** actor/src/main/scala/org/apache/pekko/io/UdpConnection.scala:52-92 (resolución y doConnect) ; actor/src/main/scala/org/apache/pekko/io/UdpConnection.scala:94-129 (connected) ; actor/src/main/scala/org/apache/pekko/io/UdpConnection.scala:131-151 (doRead) ; actor/src/main/scala/org/apache/pekko/io/UdpConnection.scala:153-170 (doWrite) ; actor/src/main/scala/org/apache/pekko/io/UdpConnection.scala:172-194 (postStop/reportConnectFailure) ; actor/src/main/scala/org/apache/pekko/io/UdpConnected.scala:89-99 (Send) ; actor/src/main/scala/org/apache/pekko/io/UdpConnected.scala:107-113 (Connect) ; actor/src/main/scala/org/apache/pekko/io/UdpConnected.scala:120 (Disconnect) ; actor/src/main/scala/org/apache/pekko/io/UdpConnected.scala:145-166 (eventos) ; actor/src/main/scala/org/apache/pekko/io/UdpConnected.scala:170-189 (UdpConnectedExt)

**Condiciones.** IMPORTANTE: doConnect(address) recibe la dirección resuelta pero llama channel.connect(remoteAddress) (UdpConnection.scala:82-88), es decir, con la dirección original SIN resolver: conectar a un InetSocketAddress sin resolver lanza UnresolvedAddressException -> CommandFailed(connect). Efecto: un Connect con host por nombre siempre falla en 1.7.1 aunque DNS resuelva (derivado del código; el spec solo prueba el fallo con un host que no existe). Send: IllegalArgumentException("requirement failed: ack must be non-null...") (UdpConnected.scala:90-93, formato con salto de línea pero mismo texto).

**Orden / concurrencia / ciclo de vida.** El Send no se escribe hasta ChannelWritable; una segunda Send antes de eso se rechaza.

**Dependencias.** AC-IO-006.1, AC-IO-008.1, AC-IO-005

**Equivalente Rust propuesto.** `UdpConnectedActor` con `UdpSocket::connect(resolved)` (ver Q-IO-5 para el caso con nombre).

**Diferencias observables.** Q-IO-5: elegir entre replicar el fallo de nombres o conectar a la dirección resuelta.

**Aserciones.**
- A1. Connect a "doesnotexist.local" sin resolver (con async-dns) responde CommandFailed(command) en <=10 s; repetido (caché) en <=6 s — UdpConnectedIntegrationSpec.scala:61-77
- A2. Enviar sin bind local: el servidor recibe Udp.Received con los datos y responde con Udp.Send y el cliente recibe UdpConnected.Received — UdpConnectedIntegrationSpec.scala:79-95
- A3. Con bind local: el servidor ve Received(d, clientAddress) — UdpConnectedIntegrationSpec.scala:97-114
- A4. Tras Unbind del servidor y nuevo bind en la misma dirección el cliente puede seguir enviando — UdpConnectedIntegrationSpec.scala:116-136
- A5. Cuando el servidor desaparece el actor sobrevive y los Send con ack (1,2,3) se acusan; al volver un servidor recibe Received — UdpConnectedIntegrationSpec.scala:138-174
- A6. Connect con host por nombre que SÍ resuelve falla por doConnect — derivada del código UdpConnection.scala:55-63,82-92; no afirmada por spec y NO verificada contra el jar
- A7. El lote de lectura no se limita por receive-throughput — derivada del código UdpConnection.scala:132-141; no afirmada por spec

**Oráculo.** Jar oficial en loopback: el caso "Connect con nombre localhost" debe ejecutarse para confirmar el defecto (propuesto en Q-IO-5). Escenario no existe ni se ejecutó.

**Decisiones.** Q-IO-5, Q-IO-1

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-008.1 — Extensión Dns: DnsExt, Dns.resolve/cached (nueva y deprecada), selección de resolver, IpVersionSelector

**Contrato observable.** Dns.resolve(request, system, sender): ret = cached(request); si ret.isEmpty hace IO(Dns)(system).tell(request, sender); devuelve ret. Dns.cached/resolve(name) y Dns.Resolve/Resolved(name, ipv4, ipv6) son la API deprecada (cached por defecto None). DnsExt construye un DnsProvider por reflexión (provider-object del resolver configurado), expone cache = provider.cache y crea el manager "IO-DNS" con dispatcher pekko.io.dns.dispatcher. loadAsyncDns(managerName) crea bajo demanda un manager async-dns con nombre propio (uno por nombre, caché en ConcurrentHashMap). IpVersionSelector.getInetAddress(ipv4, ipv6): prefiere IPv6 solo si la propiedad JVM java.net.preferIPv6Addresses == "true"; si no, IPv4 primero.

**Fuente.** actor/src/main/scala/org/apache/pekko/io/Dns.scala:44-74 (Dns) ; actor/src/main/scala/org/apache/pekko/io/Dns.scala:76-160 (object Dns) ; actor/src/main/scala/org/apache/pekko/io/Dns.scala:162-251 (DnsExt (loadAsyncDns 165-202) ; actor/src/main/scala/org/apache/pekko/io/Dns.scala:Settings 215-228 ; actor/src/main/scala/org/apache/pekko/io/Dns.scala:provider 235-236 ; actor/src/main/scala/org/apache/pekko/io/Dns.scala:cache 239 ; actor/src/main/scala/org/apache/pekko/io/Dns.scala:manager 242-246) ; actor/src/main/scala/org/apache/pekko/io/Dns.scala:257-267 (IpVersionSelector) ; actor/src/main/scala/org/apache/pekko/io/DnsProvider.scala:27-47 ; actor/src/main/resources/reference.conf:1140-1170

**Condiciones.** Resolved.addr lanza UnknownHostException(name) si no hay dirección (Dns.scala:92). Provider desconocido -> excepción de createInstanceFor (ClassNotFoundException) al construir DnsExt (get con .get, Dns.scala:236). DnsProvider está @deprecated: "Overriding the DNS implementation will be removed in future versions of Apache Pekko".

**Orden / concurrencia / ciclo de vida.** Dns.resolve no bloquea: con miss la respuesta llega al `sender` como DnsProtocol.Resolved (o Status.Failure en async-dns).

**Dependencias.** AC-IO-008.4 (cache), AC-IO-008.5, AC-CFG, Q-X-2

**Equivalente Rust propuesto.** `trait Dns { fn cached(&self,&Resolve)->Option<Resolved>; fn resolve(&self,&Resolve,&ActorSystem,&ActorRef)->Option<Resolved> }` y `DnsProvider` enum {InetAddress, Async}

**Diferencias observables.** El selector de IPv6 por propiedad JVM no existe (Q-IO-4). La carga por nombre de provider usa el registro (Q-X-2).

**Aserciones.**
- A1. Un Connect TCP a un host sin resolver acaba en CommandFailed (usa Dns.resolve internamente) — TcpConnectionSpec.scala:652-660
- A2. Con async-dns, Connect UDP a host inexistente -> CommandFailed(command) — UdpConnectedIntegrationSpec.scala:61-77
- A3. AsyncDnsManager adapta el protocolo antiguo Dns.Resolve a Dns.Resolved y devuelve el SimpleDnsCache con GetCache — AsyncDnsManagerSpec.scala:42-65
- A4. IpVersionSelector y semántica de Dns.resolve (cached + tell) — derivada del código Dns.scala:68-73,257-267; no afirmada por spec

**Oráculo.** Jar oficial: jshell con resolve de localhost. Escenario no existe ni se ejecutó.

**Decisiones.** Q-IO-4, Q-X-2

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-008.2 — DnsProtocol (Resolve/Resolved, Ip/Srv), ResourceRecord (A, AAAA, CNAME, SRV, Unknown) y CachePolicy (Never/Forever/Ttl)

**Contrato observable.** Resolve(name, requestType); requestType = Ip(ipv4=true, ipv6=true) | Srv; Resolve(name) = Ip(). Resolved(name, records, additionalRecords) con address(): primera A (Inet4Address) y primera AAAA elegidas con IpVersionSelector; si no hay -> UnknownHostException(name). Registros: ARecord(name, ttl, ip), AAAARecord, CNameRecord(name, ttl, canonicalName), SRVRecord(name, ttl, priority, weight, port, target), UnknownRecord(name, ttl, recType, recClass, data). CachePolicy: Never, Forever, Ttl(value >= 0); Ttl.fromPositive(v) exige v > 0; Ttl.never = Ttl(0s); Ttl.effectivelyForever = fromPositive(Int.MaxValue.seconds); toTll(policy). Ttl se ordena por duración. RecordType: A=1 NS=2 MD=3 MF=4 CNAME=5 SOA=6 MB=7 MG=8 MR=9 NULL=10 WKS=11 PTR=12 HINFO=13 MINFO=14 MX=15 TXT=16 AAAA=28 SRV=33 AXFR=252 MAILB=253 MAILA=254 WILDCARD=255; RecordClass: IN=1 CS=2 CH=3 HS=4 WILDCARD=255.

**Fuente.** actor/src/main/scala/org/apache/pekko/io/dns/DnsProtocol.scala:38-127 ; actor/src/main/scala/org/apache/pekko/io/dns/DnsResourceRecords.scala:33-144 (tipos y parseBody) ; actor/src/main/scala/org/apache/pekko/io/dns/DnsResourceRecords.scala:156-177 (parse) ; actor/src/main/scala/org/apache/pekko/io/dns/CachePolicy.scala:22-76 ; actor/src/main/scala/org/apache/pekko/io/dns/RecordType.scala:21-106 ; actor/src/main/scala/org/apache/pekko/io/dns/RecordClass.scala:20-27

**Condiciones.** Ttl(negativo) -> IllegalArgumentException("TTL values must be a positive value (zero included)."); fromPositive(<=0) -> IllegalArgumentException("Positive TTL values must be a strictly positive value. Use Ttl.never for zero.") (CachePolicy.scala:30,48-49). Resolved.address sin A/AAAA -> UnknownHostException(<name>) (DnsProtocol.scala:116). RecordClass.parse desconocida -> RuntimeException("Unexpected record class <n>") (RecordClassSerializer.scala:28); RecordType desconocido -> IllegalArgumentException("Illegal id [<n>] for DnsRecordType") (RecordTypeSerializer.scala:35). El TTL cero de un RR se mapea a Ttl.never, otro a Ttl.fromPositive(n.seconds) (DnsResourceRecords.scala:163-166).

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AC-IO-008.1, AC-IO-009.5

**Equivalente Rust propuesto.** `enum RequestType { Ip{ipv4,ipv6}, Srv }`, `struct Ttl(Duration)` con constructores validados, `enum CachePolicy { Never, Forever, Ttl(Ttl) }`.

**Diferencias observables.** InetAddress -> std::net::IpAddr (Ipv4Addr/Ipv6Addr); la API Java (getRecords) no se porta.

**Aserciones.**
- A1. Resolved("cats.com", Seq(rec)) igualdad estructural con registros — AsyncDnsResolverSpec.scala:56-65,132-154
- A2. SRV con registros adicionales: Resolved("cats.com", srvRecs, aRecs) igual al esperado y en segunda consulta desde caché — AsyncDnsResolverSpec.scala:191-206
- A3. Literal IP responde inmediatamente ARecord(name, Ttl.effectivelyForever, ip) — AsyncDnsResolverSpec.scala:167-174
- A4. Ttl.fromPositive(5000ms) usado por la caché — SimpleDnsCacheSpec.scala:33-40
- A5. Textos de excepciones de Ttl, RecordType, RecordClass — derivada del código CachePolicy.scala:30,48-49; RecordTypeSerializer.scala:35; RecordClassSerializer.scala:28; no afirmada por spec

**Oráculo.** Jar oficial: igualdad y toString de los mensajes. Escenario no existe ni se ejecutó.

**Decisiones.** -

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-008.3 — InetAddressDnsResolver (resolver "inet-address"): positive-ttl/negative-ttl, propiedades JVM, formas de respuesta, SRV no soportado

**Contrato observable.** Positivo "default": valor de Security networkaddress.cache.ttl (si no vacío), si no sun.net.inetaddr.ttl, si no 30 s; entero: 0 -> Never, <0 -> Forever, n -> Ttl(n s); no entero -> warning "Caching TTL misconfigured. Using default value Ttl(30 seconds)." y 30 s. Negativo "default": networkaddress.cache.negative.ttl / sun.net.inetaddr.negative.ttl, por defecto Never; mismos casos y warning "Negative caching TTL misconfigured. Using default value Never.". "forever"/"never"/duración >0. Resolve(_, Srv) -> Status.Failure(IllegalArgumentException("SRV request sent to InetResolver. SRV requests are only supported by async-dns resolver.")). Resolve(name, Ip(v4,v6)): si cached -> respuesta; si no getAllByName(name), registros A/AAAA filtrados por v4/v6 con TTL positivo; guarda en caché con clave (name, Ip()) (¡siempre Ip(), aunque el pedido tenga flags distintos!) si positivo != Never; UnknownHostException -> Resolved(name, vacío) y, si negativo != Never, se guarda con clave (name, ip) tal cual el pedido. También atiende el Dns.Resolve deprecado.

**Fuente.** actor/src/main/scala/org/apache/pekko/io/InetAddressDnsResolver.scala:49-59 (propiedades) ; actor/src/main/scala/org/apache/pekko/io/InetAddressDnsResolver.scala:61-95 (políticas por defecto) ; actor/src/main/scala/org/apache/pekko/io/InetAddressDnsResolver.scala:97-111 (getTtl y políticas) ; actor/src/main/scala/org/apache/pekko/io/InetAddressDnsResolver.scala:113-123 (toLongTtl) ; actor/src/main/scala/org/apache/pekko/io/InetAddressDnsResolver.scala:125-151 (Resolve Ip/Srv) ; actor/src/main/scala/org/apache/pekko/io/InetAddressDnsResolver.scala:152-174 (Dns.Resolve) ; actor/src/main/scala/org/apache/pekko/io/InetAddressDnsResolver.scala:177-186 (addressToRecords) ; actor/src/main/resources/reference.conf:1152-1170

**Condiciones.** positive-ttl/negative-ttl inválido (duración <=0): IllegalArgumentException("pekko.io.dns.<path> must be 'default', 'forever', 'never' or positive duration") (InetAddressDnsResolver.scala:105). Derivado y no afirmado: con ipv6=false la respuesta cacheada bajo Ip() contiene solo A, de modo que una consulta posterior Ip() lee una respuesta filtrada (inconsistencia de clave).

**Orden / concurrencia / ciclo de vida.** Ejecuta getAllByName bloqueante: por eso el deployment /IO-DNS/inet-address/* usa default-blocking-io-dispatcher (AC-IO-011).

**Dependencias.** AC-IO-008.4, AC-IO-011, Q-IO-4

**Equivalente Rust propuesto.** Resolver que llama `std::net::ToSocketAddrs`/getaddrinfo en `spawn_blocking`; `CachePolicy` configurable; las propiedades JVM no existen.

**Diferencias observables.** networkaddress.cache.ttl y sun.net.inetaddr.* no existen: "default" debe fijarse a 30 s / Never (Q-IO-4). La clave de caché inconsistente (Ip()) es comportamiento observable: decisión de replicarla o corregirla en Q-IO-4.

**Aserciones.**
- A1. Por defecto positivo=30 s (30000 ms) y negativo=0 — InetAddressDnsResolverSpec.scala:32-47
- A2. Propiedad de seguridad networkaddress.cache.ttl=42 gana a sun.net.inetaddr.ttl=2000 (positivo) y el análogo negativo (43) — InetAddressDnsResolverSpec.scala:48-65
- A3. Sin propiedad de seguridad, se usa la de sistema (42/43) — InetAddressDnsResolverSpec.scala:66-83
- A4. Valor <0 -> Forever (Long.MaxValue); 0 -> Never (0) — InetAddressDnsResolverSpec.scala:84-97
- A5. Config forever -> Long.MaxValue; never -> 0 — InetAddressDnsResolverSpec.scala:140-149
- A6. Respuesta SRV con error exacto y claves de caché — derivada del código InetAddressDnsResolver.scala:126-150; no afirmada por spec

**Oráculo.** Jar oficial: resolver localhost y un nombre inexistente; propiedades JVM no comparables con Rust. Escenario no existe ni se ejecutó.

**Decisiones.** Q-IO-4

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-008.4 — SimpleDnsCache y PeriodicCacheCleanup: expiración exacta, limpieza periódica, API antigua cached(name)

**Contrato observable.** Cache inmutable en un AtomicReference con CAS: mapa (name, RequestType) -> entrada(answer, until) y SortedSet de expiraciones. put con Forever -> until=Long.MaxValue, Never -> clock-1 (ya vencida), Ttl -> clock+ttl.ms. get devuelve la entrada solo si clock < until. cleanup() elimina entradas vencidas y sus marcas. El reloj usa System.nanoTime relativo al arranque en ms. cached(name) deprecado combina las entradas (name, Ip(v6=false)), (name, Ip(v4=false)) y (name, Ip()), recolecta IPs y devuelve Dns.Resolved o None si vacío.

**Fuente.** actor/src/main/scala/org/apache/pekko/io/SimpleDnsCache.scala:38-67 (clase y cached(name)) ; actor/src/main/scala/org/apache/pekko/io/SimpleDnsCache.scala:69-70 (cached(Resolve)) ; actor/src/main/scala/org/apache/pekko/io/SimpleDnsCache.scala:72-77 (clock) ; actor/src/main/scala/org/apache/pekko/io/SimpleDnsCache.scala:83-92 (get/put) ; actor/src/main/scala/org/apache/pekko/io/SimpleDnsCache.scala:94-99 (cleanup) ; actor/src/main/scala/org/apache/pekko/io/SimpleDnsCache.scala:108-142 (Cache) ; actor/src/main/scala/org/apache/pekko/io/SimpleDnsCache.scala:144-165 (entradas) ; actor/src/main/scala/org/apache/pekko/io/SimpleDnsManager.scala:39-49 (limpieza periódica)

**Condiciones.** Entrada con until == clock ya no es válida (clock < until estricto). Dos puts con la misma clave: el último gana, y su ExpiryEntry se añade al set (el viejo puede quedar hasta la limpieza).

**Orden / concurrencia / ciclo de vida.** Thread-safe lock-free (CAS). La limpieza la ejecuta el manager en un timer con retraso fijo cache-cleanup-interval (120 s por defecto).

**Dependencias.** AC-IO-008.2, AC-IO-008.5

**Equivalente Rust propuesto.** `SimpleDnsCache` con `ArcSwap<Cache>` o `Mutex<BTreeMap>`; reloj `trait Clock`.

**Diferencias observables.** El reloj monotónico se abstrae para tests (como el override de clock() del spec).

**Aserciones.**
- A1. Con TTL 5000 ms: válida en t=0 y t=4999; ya no en t=5000 aunque no se haya barrido — SimpleDnsCacheSpec.scala:33-50
- A2. cleanup() elimina la entrada vencida; reponer el reloj a 0 después de cleanup no la resucita — SimpleDnsCacheSpec.scala:51-76
- A3. Conversión desde el protocolo antiguo (cached(name)) — derivada del código SimpleDnsCache.scala:56-67; no afirmada por spec (TODO en :77)

**Oráculo.** Jar oficial: SimpleDnsCache con reloj subclaseado en jshell. Escenario no existe ni se ejecutó.

**Decisiones.** -

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-008.5 — SimpleDnsManager, DnsProvider/InetAddressDnsProvider/AsyncDnsProvider: manager de resolver con router y limpieza periódica

**Contrato observable.** SimpleDnsManager(ext) crea un hijo con FromConfig.props(Props(providerActorClass, cache, resolverConfig).withDeploy(local).withDispatcher(dns.dispatcher)) bajo el nombre del resolver ("inet-address"), un timer scheduleWithFixedDelay(interval, interval, CacheCleanup) si la caché es PeriodicCacheCleanup, y reenvía DnsProtocol.Resolve (y el deprecado Dns.Resolve) con forward. InetAddressDnsProvider: cache=SimpleDnsCache, actorClass=InetAddressDnsResolver, managerClass=SimpleDnsManager. AsyncDnsProvider: cache=SimpleDnsCache, actorClass=AsyncDnsResolver, managerClass=AsyncDnsManager.

**Fuente.** actor/src/main/scala/org/apache/pekko/io/SimpleDnsManager.scala:25-75 ; actor/src/main/scala/org/apache/pekko/io/InetAddressDnsProvider.scala:25-29 ; actor/src/main/scala/org/apache/pekko/io/dns/internal/AsyncDnsProvider.scala:25-31 ; actor/src/main/scala/org/apache/pekko/io/DnsProvider.scala:27-47

**Condiciones.** Cancela el timer en postStop. Mensajes desconocidos -> unhandled.

**Orden / concurrencia / ciclo de vida.** Los Resolve se reenvían sin transformar; el router consistent-hashing por nombre (ConsistentHashable) asegura que el mismo nombre va al mismo resolver (AC-IO-011).

**Dependencias.** AC-IO-011 (deployments), AC-RT, AC-SCH

**Equivalente Rust propuesto.** `DnsManager` con pool de resolvers y `Scheduler::schedule_with_fixed_delay`.

**Diferencias observables.** El pool (consistent-hashing) puede ser un único resolver async si el runtime sirve getaddrinfo con su propio pool (Q-IO-1).

**Aserciones.**
- A1. Reenvío de Resolve, limpieza periódica y nombres de hijos — derivada del código SimpleDnsManager.scala:32-66; no afirmada por spec

**Oráculo.** Jar oficial: observar /system/IO-DNS/inet-address. Escenario no existe ni se ejecutó.

**Decisiones.** Q-X-2

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-009.1 — DnsSettings (async-dns): nameservers, resolve-timeout, positive/negative-ttl, search-domains, ndots, id-generator-policy; parseNameserverAddress

**Contrato observable.** nameservers: cadena "default" -> servidores del SO (JNDI y, de reserva, sun.net.dns.ResolverConfiguration por reflexión) o IllegalStateException si no hay; cadena IP[:puerto] -> uno; lista no vacía -> varios; otro tipo -> IllegalArgumentException. resolve-timeout: duración. positive-ttl/negative-ttl: "forever"/"never"/duración no negativa (Ttl.fromPositive). id-generator-policy: thread-local-random | secure-random | enhanced-double-hash-random (vacío = enhanced). search-domains: "default" -> de /etc/resolv.conf (salvo Windows) o Nil; cadena -> una; lista. ndots: "default" -> de /etc/resolv.conf o 1; número >= 0. parseNameserverAddress usa la regex (.*?)(?::(\d+))? y puerto 53 por defecto.

**Fuente.** actor/src/main/scala/org/apache/pekko/io/dns/DnsSettings.scala:62-80 (nameservers) ; actor/src/main/scala/org/apache/pekko/io/dns/DnsSettings.scala:82-85 (timeout/ttl) ; actor/src/main/scala/org/apache/pekko/io/dns/DnsSettings.scala:87-97 (id policy) ; actor/src/main/scala/org/apache/pekko/io/dns/DnsSettings.scala:99-108 (getTtl) ; actor/src/main/scala/org/apache/pekko/io/dns/DnsSettings.scala:110-127 (resolvConf) ; actor/src/main/scala/org/apache/pekko/io/dns/DnsSettings.scala:129-140 (search-domains) ; actor/src/main/scala/org/apache/pekko/io/dns/DnsSettings.scala:142-159 (ndots) ; actor/src/main/scala/org/apache/pekko/io/dns/DnsSettings.scala:163-166 (error) ; actor/src/main/scala/org/apache/pekko/io/dns/DnsSettings.scala:172-184 (parseNameserverAddress) ; actor/src/main/scala/org/apache/pekko/io/dns/DnsSettings.scala:195-253 (nameservers por defecto) ; actor/src/main/resources/reference.conf:1172-1231

**Condiciones.** Textos: IllegalStateException("Unable to obtain default nameservers from JNDI or via reflection. Please set `pekko.io.dns.async-dns.nameservers` explicitly in order to be able to resolve domain names. ") (DnsSettings.scala:164-166); "Invalid type for nameservers. Must be a string or string list" (:78); "nameservers can not be empty" (:76, require -> prefijo "requirement failed: "); "Invalid type for search-domains. Must be a string or string list." (:138); "Invalid value for ndots. Must be the string 'default' or an integer." (:148,:157); "Invalid value for ndots, ndots must not be negative." (:153); "Invalid value for id-generator-policy, id-generator-policy must be 'thread-local-random', 'secure-random' or`enhanced-double-hash-random`" (:93-95, sin espacio antes de la comilla invertida: concatenación literal); positive-ttl/negative-ttl con duración negativa: "pekko.io.dns.<path> must be 'default', 'forever', 'never' or positive duration" (:106), y duración cero lanza el texto de Ttl.fromPositive (AC-IO-008.2). Un error al parsear /etc/resolv.conf se registra (log.error "Error parsing /etc/resolv.conf, ignoring.") y se ignora.

**Orden / concurrencia / ciclo de vida.** resolvConf es lazy y se evalúa una vez. NameServers se calcula en construcción.

**Dependencias.** AC-IO-009.6 (ResolvConfParser), AC-IO-009.7, AC-CFG

**Equivalente Rust propuesto.** `AsyncDnsSettings { nameservers: Vec<SocketAddr>, resolve_timeout, positive_ttl, negative_ttl, search_domains, ndots, id_generator_policy }`

**Diferencias observables.** Nameservers por defecto: JNDI/reflexión JVM no existen; Q-IO-6 propone leer nameserver de /etc/resolv.conf (Pekko no lo hace). En Windows search/ndots no se leen.

**Aserciones.**
- A1. nameservers="default" no lanza (valor depende del SO) — DnsSettingsSpec.scala:42-48
- A2. nameservers="127.0.0.1" -> [127.0.0.1]; lista -> [127.0.0.1, 127.0.0.2] — DnsSettingsSpec.scala:49-65
- A3. search-domains="default" no devuelve ["default"] — DnsSettingsSpec.scala:66-77
- A4. search-domains="example.com" -> ["example.com"]; lista -> ["example.com","example.net"] — DnsSettingsSpec.scala:78-99
- A5. ndots="default" no lanza; ndots=5 -> 5 — DnsSettingsSpec.scala:100-124
- A6. positive-ttl forever / negative never; "10 s" y "10 d" -> Ttl 10s y 10d — DnsSettingsSpec.scala:125-141
- A7. id-generator-policy=thread-local-random -> ThreadLocalRandom — DnsSettingsSpec.scala:142-146
- A8. "8.8.8.8:153" y "[2001:4860:4860::8888]:153" con puerto; sin puerto -> 53 — NameserverAddressParserSpec.scala:23-37
- A9. Textos de error de ndots/search-domains/nameservers/id-generator-policy — derivada del código DnsSettings.scala:76-95,138-166; no afirmada por spec

**Oráculo.** Jar oficial: instanciar DnsSettings (private[dns], público en bytecode) con configs inválidas. Escenario no existe ni se ejecutó.

**Decisiones.** Q-IO-6, Q-X-1

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-009.2 — AsyncDnsResolver: caché con localhost precargado, TTL mínimo, literales IP inmediatos, búsqueda con search-domains/ndots, reintento entre nameservers, A y AAAA, SRV

**Contrato observable.** Al crearse precarga "localhost" en la caché para Ip(), Ip(v4 only) e Ip(v6 only) con TTL effectivelyForever (loopback, 127.0.0.1, ::1). Resolve(name, mode): caché hit -> responde; si no resolveWithResolvers: si name es literal IPv4/IPv6 responde ARecord/AAAARecord(name, effectivelyForever, ip) sin consulta; si no, prueba cada cliente en orden con resolveWithSearch; ante AskTimeoutException (resolve-timeout) o fallo, log info y siguiente; sin clientes -> Failure(ResolveFailedException("Failed to resolve <name> with nameservers: <lista>")). resolveWithSearch: sin search-domains resuelve el nombre; con ellos construye name+"."+sd; si el nombre tiene >= ndots puntos prueba primero el nombre y luego los dominios, si no los dominios primero y el nombre al final; se queda con la primera respuesta con registros. resolve: nombre en minúsculas (Helpers.toRootLowerCase); Ip(v4,v6) envía Question4 y/o Question6 (ids de IdGenerator) y concatena A+AAAA y adicionales; Srv envía SrvQuestion. DuplicateId -> reintenta con id nuevo. Fallo/otro resultado -> DropRequest al cliente. Resultado: si hay registros, cache.put con min(positiveCachePolicy, mín ttl de los registros); si no hay y negative != Never, cache.put con negativeCachePolicy.

**Fuente.** actor/src/main/scala/org/apache/pekko/io/dns/internal/AsyncDnsResolver.scala:61-82 (localhost) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/AsyncDnsResolver.scala:85-97 ; actor/src/main/scala/org/apache/pekko/io/dns/internal/AsyncDnsResolver.scala:101-119 (receive y caché) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/AsyncDnsResolver.scala:121-153 (resolveWithResolvers) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/AsyncDnsResolver.scala:155-169 (sendQuestion) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/AsyncDnsResolver.scala:171-194 (search) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/AsyncDnsResolver.scala:196-212 (resolveFirst) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/AsyncDnsResolver.scala:214-241 (resolve) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/AsyncDnsResolver.scala:251-265 (detección de literales IP) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/AsyncDnsResolver.scala:270 (ResolveFailedException)

**Condiciones.** ResolveFailedException(msg) extiende Exception con el mensaje "Failed to resolve <name> with nameservers: <List[InetSocketAddress]>" (AsyncDnsResolver.scala:140,270). Literal IPv6 por regex con zona opcional "%.+". Respuesta con registros: el TTL mínimo de caché incluye la política positiva (si positive-ttl=never, Never gana: el min con Ttl(0) = 0).

**Orden / concurrencia / ciclo de vida.** Las consultas A y AAAA se emiten a la vez (los dos val ipv4Recs/ipv6Recs se evalúan antes del for-comprehension, AsyncDnsResolver.scala:219-229) y llegan al cliente en ese orden: Question4 y luego Question6; el for solo espera ambas respuestas. El resultado se envía con pipeTo al sender original.

**Dependencias.** AC-IO-009.3, AC-IO-009.7, AC-IO-008.4, AC-UTIL (Helpers.toRootLowerCase)

**Equivalente Rust propuesto.** `AsyncDnsResolver` con `Vec<ActorRef<DnsClientCmd>>`, `ask` con `resolve_timeout`, búsqueda por `search_domains`.

**Diferencias observables.** Ninguna esperada si el runtime ofrece ask+timeout; la regex de IPv6 debe portarse tal cual o reemplazarse por `Ipv6Addr::from_str` (con la zona %); diferencia posible en formas no estándar: validar con casos.

**Aserciones.**
- A1. Usa los clientes en orden: solo el primero recibe la pregunta si responde; respuesta Resolved("cats.com", vacío) — AsyncDnsResolverSpec.scala:56-65
- A2. Si el primero responde Failure pasa al segundo con otro id — AsyncDnsResolverSpec.scala:67-81
- A3. Si el primero no responde (timeout) pasa al segundo — AsyncDnsResolverSpec.scala:83-96
- A4. DuplicateId reintenta con otro id en el mismo cliente sin pasar al segundo — AsyncDnsResolverSpec.scala:98-112
- A5. Con Ip(v4,v6): Question4 luego Question6 y Resolved(name, Seq(A, AAAA)) — AsyncDnsResolverSpec.scala:114-130
- A6. Si todos fallan o expiran -> Failure(ResolveFailedException(_)) — AsyncDnsResolverSpec.scala:132-154
- A7. Srv envía SrvQuestion y no consulta al segundo cliente — AsyncDnsResolverSpec.scala:156-165
- A8. IPv4 e IPv6 literales responden de inmediato sin consultar (IPv6: toString "/1:2:3:0:0:0:0:0") — AsyncDnsResolverSpec.scala:167-189
- A9. Registros adicionales de SRV se devuelven y la segunda consulta sale de la caché — AsyncDnsResolverSpec.scala:191-206
- A10. negative-ttl=5s: la respuesta vacía se cachea y no se consulta de nuevo — AsyncDnsResolverSpec.scala:208-225
- A11. Los registros se cachean hasta expirar su TTL — AsyncDnsResolverSpec.scala:227-244
- A12. positive-ttl=never: siempre consulta — AsyncDnsResolverSpec.scala:246-270
- A13. positive-ttl=200ms: cachea 200ms y luego vuelve a consultar — AsyncDnsResolverSpec.scala:272-298
- A14. Orden ndots/search-domains — derivada del código AsyncDnsResolver.scala:171-194; no afirmada por spec (los specs usan search-domains=[])
- A15. Precarga de localhost en caché — derivada del código AsyncDnsResolver.scala:61-82; no afirmada por spec

**Oráculo.** Jar oficial con clientes simulados (como el spec) y/o un servidor DNS local; el servicio Docker de AsyncDnsResolverIntegrationSpec no se usa. Escenario no existe ni se ejecutó.

**Decisiones.** Q-IO-6

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-009.3 — DnsClient (UDP): Bind efímero, preguntas A/AAAA/SRV, ids en vuelo, DuplicateId, verificación de la pregunta, caída a TCP si el mensaje está truncado

**Contrato observable.** preStart: Udp.Bind(self, 0.0.0.0:0). Hasta Bound, las preguntas (Question4/Question6/SrvQuestion) se guardan en el stash; luego pasan a ready(socket). Cada pregunta crea un Message(id, MessageFlags(), [Question(name, A|AAAA|SRV, IN)]) y, si el id no estaba en vuelo, se registra y se envía como Udp.Send al nameserver; si el id estaba en vuelo responde DuplicateId(id). Udp.Received: parsea; si flags.isTruncated y el id está en vuelo, reenvía el mensaje original al tcpDnsClient (lazy, BackoffSupervisor con minBackoff 10 ms, maxBackoff 20 s, randomFactor 0.1); si no, compara preguntas (isSameQuestion ordenadas por nombre; ignora punto final) y si coinciden responde Answer(id, answerRecs, additionalRecs) (vacíos si responseCode != SUCCESS); si no coinciden, warning "Unexpected DNS response id {} question [{}] does not match question asked [{}]" y se descarta; id desconocido -> warning "Unexpected DNS response invalid id {}". DropRequest(question) elimina el id en vuelo solo si el nombre coincide, si no warning "Cannot drop inflight DNS request the question [{}] does not match [{}]". Udp.CommandFailed(Send): busca el id y, si la pregunta coincide, envía Failure(RuntimeException("Send failed to nameserver")) al solicitante.

**Fuente.** actor/src/main/scala/org/apache/pekko/io/dns/internal/DnsClient.scala:33-51 (mensajes) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/DnsClient.scala:57-71 (init) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/DnsClient.scala:73-84 (stash) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/DnsClient.scala:94-195 (ready) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/DnsClient.scala:197-205 (newInflightRequests) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/DnsClient.scala:207-218 (isSameQuestion) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/DnsClient.scala:220-230 (tcp client) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/Question.scala:25-51 (Question.isSame)

**Condiciones.** Texto de fallo de envío: RuntimeException("Send failed to nameserver") (DnsClient.scala:139). Truncado sin id en vuelo: debug "Client for id {} not found. Discarding unsuccessful response.". Udp.Unbind -> reenvía; Unbound -> stop.

**Orden / concurrencia / ciclo de vida.** El id es la única correlación: dos preguntas con el mismo id y distinta pregunta -> DuplicateId. El cliente TCP es perezoso (no se crea si no hay truncado).

**Dependencias.** AC-IO-006.1, AC-IO-009.4, AC-IO-009.5, AC-SUP/AC-ASK (BackoffSupervisor)

**Equivalente Rust propuesto.** `DnsClient` actor con `inflight: HashMap<u16, (ActorRef, Message)>` y socket UDP del módulo IO.

**Diferencias observables.** Ninguna esperada. Binding a 0.0.0.0 efímero IPv4 (InetAddress.getByAddress(Array.ofDim(4))).

**Aserciones.**
- A1. No crea el cliente TCP al responder por UDP: tcpClientCreated queda en false — DnsClientSpec.scala:36-60
- A2. Respuesta truncada por UDP -> el mensaje original va al cliente TCP y la respuesta TCP vuelve al solicitante — DnsClientSpec.scala:62-84
- A3. Id duplicado: el segundo emisor recibe DuplicateId(id) y el primero su respuesta; no se reenvía por UDP — DnsClientSpec.scala:86-118
- A4. Una respuesta con pregunta distinta (DNS poisoning) se ignora y el id sigue en vuelo — DnsClientSpec.scala:120-159
- A5. DropRequest con pregunta distinta no elimina el id en vuelo — DnsClientSpec.scala:161-199
- A6. Udp.CommandFailed con pregunta distinta no falla al solicitante; con la misma sí (Failure) — DnsClientSpec.scala:201-244
- A7. Textos de warning de respuesta inesperada — derivada del código DnsClient.scala:166-182; no afirmada por spec

**Oráculo.** Jar oficial con servidor UDP falso. Escenario no existe ni se ejecutó.

**Decisiones.** -

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-009.4 — TcpDnsClient: DNS sobre TCP con prefijo de longitud de 2 bytes, reconexión, fragmentación y concatenación

**Contrato observable.** Estado idle: al recibir un Message lo guarda en stash, pide Tcp.Connect(ns) y pasa a connecting. En Connected hace Register(self), watch de la conexión y desestasha. ready: cada Message se escribe como Tcp.Write(longitud(2 bytes)+bytes). Tcp.Received acumula en buffer; cuando hay 2 bytes lee la longitud; si hay payload completo parsea Message y envía Answer(id, recs, additional) al recipient (vacíos si responseCode != SUCCESS), y procesa el resto reinyectándolo como Received. Tcp.PeerClosed -> unwatch y vuelve a idle (reconecta con el siguiente Message). Terminated de la conexión -> falla.

**Fuente.** actor/src/main/scala/org/apache/pekko/io/dns/internal/TcpDnsClient.scala:36-42 (idle) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/TcpDnsClient.scala:44-56 (connecting) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/TcpDnsClient.scala:58-87 (ready) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/TcpDnsClient.scala:89-98 (parseResponse) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/TcpDnsClient.scala:101-113 (encode/decode length/throwFailure)

**Condiciones.** Fallos lanzan PekkoException (el BackoffSupervisor del DnsClient reinicia con backoff): "Failed to connect to TCP DNS server at [<ns>]" (+causa), "Write failed", "TCP connection terminated without closing (register timeout?)" (TcpDnsClient.scala:46,63,86). Respuesta truncada por TCP: warning "TCP DNS response truncated".

**Orden / concurrencia / ciclo de vida.** encodeLength = (len/256, len) bytes; decodeLength = ((b0+256)%256)*256+((b1+256)%256). Respuestas en orden de llegada.

**Dependencias.** AC-IO-002 (Tcp), AC-IO-009.5

**Equivalente Rust propuesto.** `TcpDnsClient` con buffer de recepción y helpers `encode_length/decode_length`.

**Diferencias observables.** Ninguna esperada.

**Aserciones.**
- A1. Reconecta (segundo Tcp.Connect) cuando el servidor cierra la conexión y entrega Answer(42, Nil) — TcpDnsClientSpec.scala:38-62
- A2. Acepta una respuesta fragmentada en varios Received — TcpDnsClientSpec.scala:64-83
- A3. Acepta respuestas pegadas: Answer(42, Nil) y Answer(43, Nil) tras dos Write — TcpDnsClientSpec.scala:85-109
- A4. Falla cuando la conexión simplemente termina — TcpDnsClientSpec.scala:111-130

**Oráculo.** Jar oficial con servidor TCP falso. Escenario no existe ni se ejecutó.

**Decisiones.** -

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-009.5 — Códec DNS: Message/MessageFlags/OpCode/ResponseCode, Question, DomainName (punteros de compresión, límites), RecordTypeSerializer/RecordClassSerializer, orden de bytes big-endian

**Contrato observable.** MessageFlags(Short): bit 15 respuesta, bits 14-11 opcode (QUERY=0, IQUERY=1, STATUS=2), bit 10 AA, bit 9 TC, bit 8 RD, bit 7 RA, bits 3-0 ResponseCode (SUCCESS=0 FORMAT_ERROR=1 SERVER_FAILURE=2 NAME_ERROR=3 NOT_IMPLEMENTED=4 REFUSED=5); por defecto RD=true. Message.write emite id, flags, qdcount y tres ceros (solo escribe las preguntas, no registros). Message.parse lee la cabecera y cuenta de cada sección; si el mensaje está truncado (TC) descarta silenciosamente las preguntas/registros que fallen al parsear; si no, propaga la excepción. DomainName.write: etiquetas separadas por "."; longitud de etiqueta como byte y chars como bytes, termina en 0 (no valida >63 ni caracteres no ASCII). DomainName.parse: sigue punteros (0xC0) con máximo 16 saltos, rechaza etiquetas con tipo reservado (bits altos 01/10) y nombres > 255 caracteres. ResourceRecord.parse: name, tipo, clase, ttl (0 -> Ttl.never), rdlength; tipos 1=A, 5=CNAME, 28=AAAA, 33=SRV; otros -> UnknownRecord. Todo en BIG_ENDIAN (package.scala).

**Fuente.** actor/src/main/scala/org/apache/pekko/io/dns/internal/DnsMessage.scala:31-48 (OpCode/ResponseCode) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/DnsMessage.scala:54-108 (MessageFlags) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/DnsMessage.scala:114-139 (Message.write) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/DnsMessage.scala:145-181 (Message.parse (TC 170-177)) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/DomainName.scala:32-33 (límites) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/DomainName.scala:39-47 (write) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/DomainName.scala:49-82 (parse) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/Question.scala:25-51 ; actor/src/main/scala/org/apache/pekko/io/dns/DnsResourceRecords.scala:149-177 (parse) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/package.scala:networkByteOrder

**Condiciones.** Errores de DomainName.parse: IllegalArgumentException("Unable to parse domain name: more than 16 compression pointers, probable pointer loop") (DomainName.scala:63-64); "Unable to parse domain name: unsupported label type [<n>]" (:69-70); "Unable to parse domain name: name longer than 255 characters" (:77-78); nombre truncado -> NoSuchElementException (ByteIterator, spec). Question.isSame ignora un punto final (el comentario dice que parse "adds a '.' to the end", pero parse no lo añade: comentario obsoleto, Question.scala:31-33 vs DomainName.scala:49-82).

**Orden / concurrencia / ciclo de vida.** n/a (funciones puras).

**Dependencias.** AC-IO-008.2, AC-IO-009.3, AC-IO-009.4

**Equivalente Rust propuesto.** `struct DnsMessage { id, flags, questions, answers, authority, additional }` y `DnsWireError::{PointerLoop, UnsupportedLabel(n), NameTooLong, Truncated}`

**Diferencias observables.** NoSuchElementException de ByteIterator -> `DnsWireError::Truncated`. Los chars se escriben como bytes (c.toByte): nombres no ASCII se truncan a 8 bits en Pekko; en Rust conviene replicar o rechazar (decisión menor, anotada aquí).

**Aserciones.**
- A1. parse de un nombre plano -> "www.example.com" — DomainNameSpec.scala:34-37
- A2. Sigue un puntero de compresión hacia atrás -> "www.example.com" — DomainNameSpec.scala:39-43
- A3. Rechaza un puntero que apunta a sí mismo (IllegalArgumentException) — DomainNameSpec.scala:45-49
- A4. Rechaza un ciclo de punteros — DomainNameSpec.scala:51-55
- A5. Rechaza tipos de etiqueta reservados — DomainNameSpec.scala:57-60
- A6. Falla (NoSuchElementException) con un nombre truncado, no entra en bucle — DomainNameSpec.scala:62-65
- A7. Parsea una respuesta truncada a mitad del mensaje: id=4, TC=true, 1 pregunta ("many.bzzt.net", AAAA, IN), 0 respuestas/autoridad/adicionales — MessageSpec.scala:24-36
- A8. Textos exactos de las excepciones de DomainName — derivada del código DomainName.scala:63-78; no afirmada por spec (los tests solo comprueban el tipo)

**Oráculo.** Jar oficial (clases private[dns]/[pekko], públicas en bytecode): jshell para volcar write(Message) y parse de fixtures. Escenario no existe ni se ejecutó.

**Decisiones.** -

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-009.6 — ResolvConfParser: search/domain/options ndots de /etc/resolv.conf

**Contrato observable.** parseLines recorta cada línea, descarta vacías y las que empiezan por ";" o "#"; separa etiqueta y argumentos por el primer espacio en blanco; "domain X" -> search=[X]; "search a b c" -> lista dividida por espacios; "options ..." -> para cada opción que empiece por "ndots:" fija ndots=Int. Todo lo demás se ignora. Valores iniciales: search=Nil, ndots=1. parseFile lee el archivo con Files.lines dentro de Try.

**Fuente.** actor/src/main/scala/org/apache/pekko/io/dns/internal/ResolvConfParser.scala:24-30 (etiquetas) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/ResolvConfParser.scala:36-40 (parseFile) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/ResolvConfParser.scala:42-76 (parseLines)

**Condiciones.** "ndots:abc" -> NumberFormatException (toInt) que hace fallar el parseo completo y DnsSettings lo registra y lo ignora (derivado, no afirmado). Líneas con tabulaciones/espacios al principio o fin se toleran.

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AC-IO-009.1

**Equivalente Rust propuesto.** `ResolvConf { search: Vec<String>, ndots: u32 }` y `parse_lines(impl Iterator<Item=&str>)`.

**Diferencias observables.** Windows: no se lee archivo (DnsSettings).

**Aserciones.**
- A1. Config de Kubernetes: search de 3 dominios y ndots 5 — ResolvConfParserSpec.scala:27-34
- A2. Ignora comentarios con # y con ; — ResolvConfParserSpec.scala:35-44
- A3. El último search encontrado gana; domain equivale a search de un elemento; el último domain gana — ResolvConfParserSpec.scala:45-59
- A4. Ignora opciones que no son ndots y tolera tabs/espacios — ResolvConfParserSpec.scala:60-67
- A5. ndots por defecto 1 — ResolvConfParserSpec.scala:68-70

**Oráculo.** No aplica: puro; se prueba con las mismas cadenas.

**Decisiones.** Q-IO-6

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-009.7 — IdGenerator: políticas thread-local-random, secure-random, enhanced-double-hash-random (ids DNS de 16 bits)

**Contrato observable.** random(rand): (rand.nextInt(65536) + Short.MinValue).toShort. EnhancedDoubleHashGenerator(seed SecureRandom): estado index, increment y count; nextId = (0xFFFFFFFF & index).asInstanceOf[Short] sincronizado; luego index -= increment; increment -= count++ (doble hash mejorado). apply(policy): ThreadLocalRandom -> random(ThreadLocalRandom.current), SecureRandom -> random(new SecureRandom), EnhancedDoubleHashRandom -> EnhancedDoubleHashGenerator(new SecureRandom); apply() = ThreadLocalRandom.

**Fuente.** actor/src/main/scala/org/apache/pekko/io/dns/IdGenerator.scala:54-60 (apply) ; actor/src/main/scala/org/apache/pekko/io/dns/IdGenerator.scala:65-66 (random) ; actor/src/main/scala/org/apache/pekko/io/dns/IdGenerator.scala:68-92 (EnhancedDoubleHashGenerator)

**Condiciones.** Sin errores. La secuencia de enhanced-double-hash está determinada por la semilla (SecureRandom): no reproducible; solo se puede probar rango y no repetición a corto plazo (propiedad, no afirmada).

**Orden / concurrencia / ciclo de vida.** nextId es thread-safe (synchronized en enhanced).

**Dependencias.** AC-IO-009.1

**Equivalente Rust propuesto.** `trait DnsIdGenerator { fn next_id(&self) -> u16 }` con `rand`/`getrandom`.

**Diferencias observables.** Short con signo -> u16 en Rust; el cable es el mismo.

**Aserciones.**
- A1. ThreadLocalRandom y SecureRandom devuelven valores dentro de [Short.MinValue, Short.MaxValue] — IdGeneratorSpec.scala:25-37
- A2. Política enhanced-double-hash-random (algoritmo) — derivada del código IdGenerator.scala:68-92; no afirmada por spec

**Oráculo.** No aplica: aleatorio.

**Decisiones.** -

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-009.8 — AsyncDnsManager y adaptación del protocolo antiguo Dns.Resolve a Dns.Resolved; GetCache; limpieza periódica

**Contrato observable.** AsyncDnsManager(name, system, resolverConfig, cache, dispatcher, provider) crea DnsSettings, un hijo FromConfig.props(Props(providerActorClass, settings, cache, factory de DnsClient por nameserver)) con nombre del resolver y un timer CacheCleanup de periodo cache-cleanup-interval si la caché es PeriodicCacheCleanup. Resolve -> forward. Dns.Resolve(name) -> resolver ? DnsProtocol.Resolve(name) (timeout resolve-timeout), mapTo Resolved, y responde Dns.Resolved(name, ips de A y AAAA) con pipeTo. GetCache -> responde la caché.

**Fuente.** actor/src/main/scala/org/apache/pekko/io/dns/internal/AsyncDnsManager.scala:51-80 (init) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/AsyncDnsManager.scala:82-89 (resolver) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/AsyncDnsManager.scala:91-102 (limpieza) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/AsyncDnsManager.scala:106-131 (receive)

**Condiciones.** Si el mapTo falla (tipo inesperado) o hay timeout: Status.Failure al emisor vía pipeTo (derivado, no afirmado).

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AC-IO-009.2, AC-IO-008.2

**Equivalente Rust propuesto.** `AsyncDnsManager` con adaptación de respuestas del protocolo antiguo.

**Diferencias observables.** La API antigua (Dns.Resolved ipv4/ipv6) se mantiene solo si Q-IO-4 decide portar la API deprecada.

**Aserciones.**
- A1. Dns.Resolve(name) recibe Dns.Resolved equivalente al protocolo nuevo — AsyncDnsManagerSpec.scala:42-46
- A2. Soporta IPv6 con el protocolo nuevo y con el antiguo (ipv4 vacío, ipv6 de longitud 1) — AsyncDnsManagerSpec.scala:48-61
- A3. GetCache responde el mismo SimpleDnsCache que Dns(system).cache — AsyncDnsManagerSpec.scala:63-66

**Oráculo.** Jar oficial. Escenario no existe ni se ejecutó.

**Decisiones.** Q-IO-4

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-010 — BufferPool / DirectByteBufferPool: pool de buffers directos con límite

**Contrato observable.** BufferPool: acquire(): ByteBuffer, release(buf). DirectByteBufferPool(defaultBufferSize, maxPoolEntries): arreglo de tamaño maxPoolEntries con synchronized. acquire: pop del pool y clear(), o allocateDirect(defaultBufferSize). release: si hay sitio lo apila; si no, tryCleanDirectByteBuffer fuera del lock (ByteBufferCleaner.clean si es directo y soportado; errores no fatales ignorados).

**Fuente.** actor/src/main/scala/org/apache/pekko/io/DirectByteBufferPool.scala:20-23 (trait) ; actor/src/main/scala/org/apache/pekko/io/DirectByteBufferPool.scala:38-86 (clase) ; actor/src/main/scala/org/apache/pekko/io/DirectByteBufferPool.scala:89-106 (objeto)

**Condiciones.** Sin errores públicos. Un buffer devuelto de tamaño distinto se acepta igual (no se valida el tamaño).

**Orden / concurrencia / ciclo de vida.** acquire/release son seguros entre hilos (lock sobre el arreglo).

**Dependencias.** AC-IO-012/013

**Equivalente Rust propuesto.** `BufferPool { default_size, max_entries }` con `Mutex<Vec<BytesMut>>`.

**Diferencias observables.** No hay buffers "directos" ni limpieza manual; el pool se limita a reutilizar memoria. direct-buffer-size/pool-limit siguen en config.

**Aserciones.**
- A1. acquire reutiliza y limpia; release no excede maxPoolEntries — derivada del código DirectByteBufferPool.scala:51-82; no afirmada por spec

**Oráculo.** No aplica.

**Decisiones.** Q-IO-3

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-011 — pekko.io.pinned-dispatcher y deployments /IO-DNS/inet-address, /IO-DNS/inet-address/*, /IO-DNS/async-dns

**Contrato observable.** pinned-dispatcher: type=PinnedDispatcher, executor=thread-pool-executor, thread-pool-executor.allow-core-timeout=off. Deployments: "/IO-DNS/inet-address" mailbox "unbounded", router consistent-hashing-pool, nr-of-instances 4; "/IO-DNS/inet-address/*" dispatcher default-blocking-io-dispatcher; "/IO-DNS/async-dns" mailbox unbounded, router round-robin-pool, nr-of-instances 1.

**Fuente.** actor/src/main/resources/reference.conf:349-362 deployments ; actor/src/main/resources/reference.conf:930-938 pinned-dispatcher

**Condiciones.** Valores por defecto; el nombre de ruta determina el deployment para los hijos creados con FromConfig.

**Orden / concurrencia / ciclo de vida.** El router consistent-hashing usa ConsistentHashable.consistentHashKey (= name) de Resolve.

**Dependencias.** AC-DSP-011, AC-RT, AC-IO-008.5

**Equivalente Rust propuesto.** Entradas de config embebidas; `PinnedDispatcher` solo si Q-IO-1 conserva selectores dedicados.

**Diferencias observables.** Con tokio los dispatchers no se usan: se acepta la config sin efecto (Q-IO-1).

**Aserciones.**
- A1. Valores de deployments y pinned-dispatcher — derivada del archivo actor/src/main/resources/reference.conf:349-362,930-938; no afirmada por spec

**Oráculo.** Jar oficial: leer la config efectiva. No ejecutado.

**Decisiones.** Q-IO-1

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-012 — UnsynchronizedByteArrayInputStream (java io): InputStream sobre byte[] sin sincronización, mark/reset/skip

**Contrato observable.** Constructores (data) y (data, offset, length). available = max(eod-offset, 0). mark(readLimit) guarda el offset ignorando el límite; reset vuelve a ese offset; markSupported = true. read() devuelve el byte sin signo o -1 al final. read(dest) y read(dest, off, len) copian min(len, disponible) bytes, devuelven 0 si len==0 y -1 si no queda nada. skip(n): n<0 -> IllegalArgumentException; devuelve min(n, disponible) pero fija offset = offset + n (puede quedar más allá del fin; el siguiente read da -1). close hereda el no-op de InputStream.

**Fuente.** actor/src/main/java/org/apache/pekko/io/UnsynchronizedByteArrayInputStream.java:46-51 (requireNonNegative) ; actor/src/main/java/org/apache/pekko/io/UnsynchronizedByteArrayInputStream.java:53-58 (checkFromIndexSize) ; actor/src/main/java/org/apache/pekko/io/UnsynchronizedByteArrayInputStream.java:82-104 (constructores) ; actor/src/main/java/org/apache/pekko/io/UnsynchronizedByteArrayInputStream.java:106-109 (available) ; actor/src/main/java/org/apache/pekko/io/UnsynchronizedByteArrayInputStream.java:111-120 (mark/markSupported) ; actor/src/main/java/org/apache/pekko/io/UnsynchronizedByteArrayInputStream.java:122-158 (read y readLocal) ; actor/src/main/java/org/apache/pekko/io/UnsynchronizedByteArrayInputStream.java:160-164 (reset) ; actor/src/main/java/org/apache/pekko/io/UnsynchronizedByteArrayInputStream.java:166-179 (skip)

**Condiciones.** Errores: IllegalArgumentException("offset cannot be negative") / ("length cannot be negative") (:98-99 vía :46-51); NullPointerException("data"/"dest"/"byte array"); IndexOutOfBoundsException("Range [off, off + len) out of bounds for length n") en read por rango (:53-58, formato String.format con %s); IllegalArgumentException("Skipping backward is not supported") (:169); skip con n > Int.MaxValue -> ArithmeticException (Math.toIntExact, :177; derivado, no afirmado).

**Orden / concurrencia / ciclo de vida.** No es thread-safe (por diseño).

**Dependencias.** AC-SER-010, AC-SER-013

**Equivalente Rust propuesto.** `std::io::Cursor` cubre lectura y seek; mark/reset se emula guardando la posición; no hace falta tipo propio.

**Diferencias observables.** En Rust no hay InputStream; el quirk de skip no se reproduce a menos que un test lo exija.

**Aserciones.**
- A1. markSupported es true; read, mark(1), read, reset, read repite el segundo byte (el límite de mark se ignora) — UnsynchronizedByteArrayInputStreamSpec.scala:30-38
- A2. skip(1) devuelve 1 y el siguiente read es el segundo byte ("b") — UnsynchronizedByteArrayInputStreamSpec.scala:40-44
- A3. skip(50) sobre 3 bytes devuelve 3 y read -> -1 — UnsynchronizedByteArrayInputStreamSpec.scala:46-51
- A4. Textos de excepciones y offset tras skip — derivada del código UnsynchronizedByteArrayInputStream.java:46-58,166-179; no afirmada por spec

**Oráculo.** No aplica.

**Decisiones.** -

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-013 — ByteBufferCleaner (java io): liberación explícita de ByteBuffer directo (Java8Cleaner / Java9Cleaner) e isSupported

**Contrato observable.** Al cargar la clase se elige un cleaner: Java8Cleaner (reflexión sun.misc.Cleaner / sun.nio.ch.DirectBuffer.cleaner) y, si falla, Java9Cleaner (sun.misc.Unsafe.invokeCleaner con MethodHandle). Si hay cleaner, lo prueba limpiando un buffer directo de 1 byte; si eso falla lo deshabilita. isSupported = INSTANCE != null. clean(buffer) delega en INSTANCE y envuelve cualquier Throwable en IllegalStateException("Failed to clean direct buffer.", t).

**Fuente.** actor/src/main/java/org/apache/pekko/io/ByteBufferCleaner.java:45-47 (interfaz) ; actor/src/main/java/org/apache/pekko/io/ByteBufferCleaner.java:49-66 (Java8Cleaner) ; actor/src/main/java/org/apache/pekko/io/ByteBufferCleaner.java:68-88 (Java9Cleaner) ; actor/src/main/java/org/apache/pekko/io/ByteBufferCleaner.java:90 (INSTANCE) ; actor/src/main/java/org/apache/pekko/io/ByteBufferCleaner.java:98-104 (clean) ; actor/src/main/java/org/apache/pekko/io/ByteBufferCleaner.java:106-134 (getCleaner) ; actor/src/main/java/org/apache/pekko/io/ByteBufferCleaner.java:143-145 (isSupported)

**Condiciones.** stderr: "WARNING: Failed to initialize a ByteBuffer Cleaner. This means direct ByteBuffers will only be cleaned upon garbage collection. Reason: <e1>" (:114-117) y "WARNING: ByteBuffer Cleaner failed to clean a test buffer. ByteBuffer Cleaner has been disabled. This means direct ByteBuffers will only be cleaned upon garbage collection. Reason: <t>" (:126-130). clean sin cleaner soportado -> IllegalStateException("Failed to clean direct buffer.") (NullPointerException envuelto, :98-103; derivado).

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AC-IO-010

**Equivalente Rust propuesto.** Sin equivalente: Rust libera al soltar el buffer.

**Diferencias observables.** Sin diferencia observable si se excluye (propuesto en Q-IO-3).

**Aserciones.**
- A1. isSupported es true y clean(ByteBuffer.allocateDirect(1)) no lanza (JVM con cleaner) — ByteBufferCleanerSpec.scala:28-33
- A2. Textos de los avisos por stderr y de IllegalStateException — derivada del código ByteBufferCleaner.java:98-130; no afirmada por spec

**Oráculo.** No aplica.

**Decisiones.** Q-IO-3

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-JAPI-001 — AbstractActor + createReceive/ReceiveBuilder y AbstractActor.ActorContext (getters Java, become(Receive), receive timeout con java.time.Duration)

**Contrato observable.** AbstractActor extiende Actor: receive = createReceive().onMessage (:317-318); receiveBuilder() = ReceiveBuilder.create() (:324). Receive(onMessage: PartialFunction[Any, BoxedUnit]).orElse(other) compone; emptyBehavior no coincide con ningún mensaje (:42-55). AbstractActor.ActorContext añade getSelf, getProps, getSender, getChildren (Iterable), findChild(name): Optional, getParent, getSystem, getDispatcher, become(Receive[, discardOld]), getReceiveTimeout/setReceiveTimeout(java.time.Duration)/cancelReceiveTimeout (= setReceiveTimeout(Duration.Undefined)). Hooks preStart, postStop, preRestart(reason, Optional), postRestart, supervisorStrategy delegan en Actor.

**Fuente.** actor/src/main/scala/org/apache/pekko/actor/AbstractActor.scala:42-55 (Receive) ; actor/src/main/scala/org/apache/pekko/actor/AbstractActor.scala:64-200 (ActorContext) ; actor/src/main/scala/org/apache/pekko/actor/AbstractActor.scala:229-325 (AbstractActor) ; actor/src/main/scala/org/apache/pekko/actor/AbstractActor.scala:141-159 (become) ; actor/src/main/scala/org/apache/pekko/actor/AbstractActor.scala:192-200 (receive timeout)

**Condiciones.** createReceive() debe devolver un Receive; el ReceiveBuilder.build() construye la PF con la lógica de AC-JAPI-004.1 (primer caso que coincida gana; sin coincidencia -> unhandled).

**Orden / concurrencia / ciclo de vida.** createReceive() se invoca al construir el actor (receive es def; ver AC-CELL-001).

**Dependencias.** AC-CELL-001 (Actor), AC-JAPI-004.1

**Equivalente Rust propuesto.** No hay clase base Java: el equivalente es `impl Actor for T` con `fn receive(&mut self, ctx, msg) -> Handled`; `ReceiveBuilder` solo si Q-JAPI-2 lo adopta.

**Diferencias observables.** La API Rust es una sola; no se porta la superficie Java (Q-JAPI-1). Observable: ninguno si la semántica de AC-CELL-001 se porta.

**Aserciones.**
- A1. Props.create(JavaAPITestActor.class) crea un ActorRef no nulo — JavaAPI.java:66-69
- A2. Actor con parámetros de constructor "a","b",17,18 responde "a-b-17-18" (también con boxed/unboxed y null) — JavaAPI.java:99-143
- A3. receive = createReceive().onMessage; orElse; emptyBehavior — derivada del código AbstractActor.scala:42-55,317-318; no afirmada por spec leída

**Oráculo.** Jar oficial desde Java/jshell: no aplicable al port Rust (API distinta).

**Decisiones.** Q-JAPI-1, Q-JAPI-2

**Estado.** analysis=listed; impl=pending; validation=pending.

### AC-JAPI-002 — UntypedAbstractActor, AbstractLoggingActor, AbstractActorWithStash/UnboundedStash/UnrestrictedStash, AbstractActorWithTimers, UntypedAbstractActorWithTimers

**Contrato observable.** UntypedAbstractActor: createReceive() lanza UnsupportedOperationException("createReceive should not be used by UntypedAbstractActor") (:338-339); receive = { case msg => onReceive(msg) } (:341); onReceive(Any) obligatorio (@throws Throwable); unhandled delegable. AbstractLoggingActor = AbstractActor with ActorLogging. AbstractActorWithStash/UnboundedStash/UnrestrictedStash mezclan Stash/UnboundedStash/UnrestrictedStash (:410,:419,:427).

**Fuente.** actor/src/main/scala/org/apache/pekko/actor/AbstractActor.scala:336-358 (UntypedAbstractActor) ; actor/src/main/scala/org/apache/pekko/actor/AbstractActor.scala:365 (AbstractLoggingActor) ; actor/src/main/scala/org/apache/pekko/actor/AbstractActor.scala:410 (WithStash) ; actor/src/main/scala/org/apache/pekko/actor/AbstractActor.scala:419 (WithUnboundedStash) ; actor/src/main/scala/org/apache/pekko/actor/AbstractActor.scala:427 (WithUnrestrictedStash) ; actor/src/main/scala/org/apache/pekko/actor/Timers.scala:78,94

**Condiciones.** UntypedAbstractActor.createReceive -> UnsupportedOperationException con el texto fijo (:338-339). Stash: reglas de AC-MBX/AC-STASH.

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AC-JAPI-001, AC-MBX

**Equivalente Rust propuesto.** La API Rust es una sola; no se porta la superficie Java (Q-JAPI-1).

**Diferencias observables.** La API Rust es una sola; no se porta la superficie Java (Q-JAPI-1).

**Aserciones.**
- A1. createReceive de UntypedAbstractActor lanza UnsupportedOperationException con el texto exacto — derivada del código AbstractActor.scala:338-339; no afirmada por spec leída

**Oráculo.** No aplicable.

**Decisiones.** Q-JAPI-1

**Estado.** analysis=listed; impl=pending; validation=pending.

### AC-JAPI-003 — AbstractFSM / AbstractLoggingFSM / AbstractFSMWithStash y builders FSM (FSMStateFunctionBuilder, FSMTransitionHandlerBuilder, FSMStopBuilder)

**Contrato observable.** AbstractFSM[S, D] extiende FSM[S, D]: when(stateName, stateFunctionBuilder|timeout), startWith(state, data[, timeout]), onTransition(builder|UnitApply2), whenUnhandled(builder), onTermination(stopBuilder), matchEvent* (por clase, predicado, lista de tipos, igualdad, cualquiera), matchState(from, to, apply), matchStop(reason|class), matchData, goTo, startTimerWithFixedDelay/AtFixedRate/startSingleTimer/setTimer con java.time.Duration. AbstractFSM.NullFunction = FSM.NullFunction. AbstractLoggingFSM = AbstractFSM with LoggingFSM; AbstractFSMWithStash mezcla Stash y reexpone postStop.

**Fuente.** actor/src/main/scala/org/apache/pekko/actor/AbstractFSM.scala:26-45 (objeto) ; actor/src/main/scala/org/apache/pekko/actor/AbstractFSM.scala:86-130 (when) ; actor/src/main/scala/org/apache/pekko/actor/AbstractFSM.scala:140-170 (startWith) ; actor/src/main/scala/org/apache/pekko/actor/AbstractFSM.scala:175-206 (handlers) ; actor/src/main/scala/org/apache/pekko/actor/AbstractFSM.scala:217-348 (matchEvent*) ; actor/src/main/scala/org/apache/pekko/actor/AbstractFSM.scala:361-448 (matchState/Stop/Data) ; actor/src/main/scala/org/apache/pekko/actor/AbstractFSM.scala:448 (goTo) ; actor/src/main/scala/org/apache/pekko/actor/AbstractFSM.scala:466-560 (timers) ; actor/src/main/scala/org/apache/pekko/actor/AbstractFSM.scala:568-586 (variantes)

**Condiciones.** Mismas condiciones que FSM (AC-FSM): estado inicial exigido por initialize(); timers con nombre.

**Orden / concurrencia / ciclo de vida.** Igual que FSM.

**Dependencias.** AC-FSM-001, AC-JAPI-004

**Equivalente Rust propuesto.** La API Rust es una sola; no se porta la superficie Java (Q-JAPI-1).

**Diferencias observables.** La API Rust es una sola; no se porta la superficie Java (Q-JAPI-1).

**Aserciones.**
- A1. MyFSM (AbstractFSM) emite "Transitioning from start to start." al iniciar y "Transitioning from start to next." al recibir "next" — AbstractFSMActorTest.java:51-61

**Oráculo.** No aplicable.

**Decisiones.** Q-JAPI-1

**Estado.** analysis=listed; impl=pending; validation=pending.

### AC-JAPI-004.1 — japi.pf Match/PFBuilder/AbstractMatch/AbstractPFBuilder/CaseStatement: PartialFunction construible por casos (tipo, tipo+predicado, igualdad, cualquiera)

**Contrato observable.** PFBuilder<I,R>.match(Class<P>, Apply<P,R>) añade CaseStatement con predicado type.isInstance(o); match(Class, TypedPredicate, Apply) exige además el predicado; matchEquals(obj, apply) usa obj.equals(o); matchAny(apply) siempre; matchUnchecked evita el chequeo de genéricos. addStatement encadena con orElse. build() devuelve statements.orElse(empty) y resetea statements=null; sin casos devuelve la PF vacía. Match.create(builder) congela la PF; match(i) aplica y lanza MatchError sin coincidencia; asPF() devuelve la PF. Match.match(...) estático crea un PFBuilder con el primer caso.

**Fuente.** actor/src/main/java/org/apache/pekko/japi/pf/AbstractPFBuilder.java:26-46 ; actor/src/main/java/org/apache/pekko/japi/pf/AbstractMatch.java:26-41 ; actor/src/main/java/org/apache/pekko/japi/pf/PFBuilder.java:43-148 ; actor/src/main/java/org/apache/pekko/japi/pf/Match.java:35-133 ; actor/src/main/scala/org/apache/pekko/japi/pf/CaseStatements.scala:25-30 (CaseStatement)

**Condiciones.** Sin coincidencia: scala.MatchError(<objeto>) (Match.java:131-132). matchEquals(null, ...) -> NullPointerException al evaluar object.equals(o) (PFBuilder.java:123; derivado, no afirmado). El builder es mutable y no thread-safe; tras build() queda vacío.

**Orden / concurrencia / ciclo de vida.** El orden de los casos es el orden de alta: Integer antes que Number en el spec.

**Dependencias.** AC-CELL-001 (receive como PartialFunction)

**Equivalente Rust propuesto.** `struct ReceiveBuilder<R> { cases: Vec<Case<R>> }` con `match_type::<T>(|t| ..)`, `match_type_if`, `match_equals`, `match_any`; `build() -> impl Fn(&dyn Any) -> Option<R>`; `MatchError` como valor `Err`.

**Diferencias observables.** `type.isInstance` incluye subtipos JVM; en Rust se compara TypeId exacto (Q-X-6/Q-JAPI-2): un caso `Number` no atraparía un `Integer`. Se reemplaza MatchError (excepción) por `Err(MatchError)` o `None`.

**Aserciones.**
- A1. Con casos Integer(*10) y Number(*-10): 4711 -> 47110.0; Double 4711 -> -47110.0; String "4711" -> MatchError — MatchBuilderTest.java:28-58
- A2. matchUnchecked(GenericClass.class, ..) extrae el valor "A" de GenericClass<String> — MatchBuilderTest.java:68-85
- A3. Con predicado (val no vacío) y valor "" -> MatchError — MatchBuilderTest.java:87-110
- A4. matchEquals("hello") + matchAny: isDefinedAt("hello") y isDefinedAt("42") son true; apply("42")==42 — PFBuilderTest.java:25-36
- A5. build() resetea el builder y sin casos devuelve PF vacía — derivada del código AbstractPFBuilder.java:39-46; no afirmada por spec

**Oráculo.** No aplicable (API Java).

**Decisiones.** Q-JAPI-2

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-JAPI-004.2 — japi.pf UnitMatch/UnitPFBuilder (variante con efecto, sin valor de retorno)

**Contrato observable.** UnitMatch.create(UnitPFBuilder).match(i) aplica el primer caso y devuelve void; lanza MatchError sin coincidencia (UnitMatch.java:142). UnitPFBuilder tiene los mismos matchers (match con/sin predicado, matchUnchecked, matchEquals con/sin predicado, matchAny).

**Fuente.** actor/src/main/java/org/apache/pekko/japi/pf/UnitMatch.java:61-142 ; actor/src/main/scala/org/apache/pekko/japi/pf/CaseStatements.scala:33-38

**Condiciones.** MatchError sin coincidencia.

**Orden / concurrencia / ciclo de vida.** Primer caso gana.

**Dependencias.** AC-JAPI-004.1

**Equivalente Rust propuesto.** Misma estructura que AC-JAPI-004.1 con acción `FnMut(&T)`.

**Diferencias observables.** La API Rust es una sola; no se porta la superficie Java (Q-JAPI-1).

**Aserciones.**
- A1. Semántica igual a Match con retorno void — derivada del código UnitMatch.java:142; no afirmada por spec

**Oráculo.** No aplicable.

**Decisiones.** Q-JAPI-2

**Estado.** analysis=listed; impl=pending; validation=pending.

### AC-JAPI-004.3 — japi.pf FI: interfaces funcionales (Apply, UnitApply, Predicate, TypedPredicate, Apply2, UnitApply2/3, UnitApplyVoid, ...)

**Contrato observable.** Contrato: cada interfaz declara un único método (apply/defined) y puede lanzar Exception (Apply) según el caso.

**Fuente.** actor/src/main/java/org/apache/pekko/japi/pf/FI.java:14-232 (no leído en detalle)

**Condiciones.** n/a: no leído.

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AC-JAPI-004.1

**Equivalente Rust propuesto.** Closures.

**Diferencias observables.** La API Rust es una sola; no se porta la superficie Java (Q-JAPI-1).

**Aserciones.**
- A1. Sin aserciones: no leído — n/a

**Oráculo.** No aplicable.

**Decisiones.** Q-JAPI-1

**Estado.** analysis=listed; impl=pending; validation=pending.

### AC-JAPI-004.4 — japi.pf DeciderBuilder: decider de supervisión por tipo de Throwable

**Contrato observable.** Clase final con constructor privado; match(Class<P extends Throwable>, Apply<P, Directive>), match(Class, TypedPredicate<P>, Apply), matchAny(Apply<Throwable, Directive>) delegan en Match.match/matchAny.

**Fuente.** actor/src/main/java/org/apache/pekko/japi/pf/DeciderBuilder.java:20-78

**Condiciones.** Sin coincidencia en el decider: la estrategia aplica su comportamiento por defecto (AC-SUP-001).

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AC-SUP-001, AC-JAPI-004.1

**Equivalente Rust propuesto.** `DeciderBuilder::new().on::<ErrorType>(Directive::Restart)`.

**Diferencias observables.** Depende de Q-X-5: los tipos de fallo en Rust son `Err` y panics.

**Aserciones.**
- A1. Delega en Match (misma semántica que AC-JAPI-004.1) — derivada del código DeciderBuilder.java; no afirmada por spec

**Oráculo.** No aplicable.

**Decisiones.** Q-X-5, Q-JAPI-2

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-JAPI-005.1 — japi.function: Function, Function2, Procedure, Effect, Predicate (+ identity), y japi.Creator/Pair/Option/Util

**Contrato observable.** Function[-T,+R].apply(T): R (puede lanzar Exception); Function.identity; Function2, Procedure, Effect, Predicate.test. japi.Pair(first, second).toScala; Pair.create. japi.Option (some/none/option(v)/fromScalaOption, get/getOrElse/isEmpty/asScala/iterator). japi.Util: classTag, immutableSeq(arr|iterable), immutableSingletonSeq, javaArrayList, immutableIndexedSeq, option(Optional). JavaPartialFunction con noMatch.

**Fuente.** actor/src/main/scala/org/apache/pekko/japi/function/Function.scala:28-108 ; actor/src/main/scala/org/apache/pekko/japi/JavaAPI.scala:36-117 (interfaces y Pair) ; actor/src/main/scala/org/apache/pekko/japi/JavaAPI.scala:120-168 (JavaPartialFunction) ; actor/src/main/scala/org/apache/pekko/japi/JavaAPI.scala:187-252 (Option) ; actor/src/main/scala/org/apache/pekko/japi/JavaAPI.scala:255-311 (Util)

**Condiciones.** Util.option(Optional.ofNullable(null)).isEmpty es true; con valor isDefined (JavaAPI.java:155-161).

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** -

**Equivalente Rust propuesto.** La API Rust es una sola; no se porta la superficie Java (Q-JAPI-1).

**Diferencias observables.** La API Rust es una sola; no se porta la superficie Java (Q-JAPI-1).

**Aserciones.**
- A1. Pair.create(1,"2"), Tuple4.create, Tuple22.create compilan — JavaAPI.java:147-151
- A2. Util.option(Optional.ofNullable(null)) está vacío; con "hello" está definido — JavaAPI.java:153-161

**Oráculo.** No aplicable.

**Decisiones.** Q-JAPI-1

**Estado.** analysis=listed; impl=pending; validation=pending.

### AC-JAPI-005.2 — Funciones y tuplas generadas (japi.function.Functions, japi.tuple.Tuple1..22) desde boilerplate

**Contrato observable.** Generación de Function3..22 y Tuple1..22 con create y accesores; solo compilación.

**Fuente.** actor/src/main/boilerplate/.../Functions.scala.template y Tuples.scala.template (no leídos)

**Condiciones.** n/a

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** -

**Equivalente Rust propuesto.** Tuplas de Rust (hasta 12 en traits estándar).

**Diferencias observables.** La API Rust es una sola; no se porta la superficie Java (Q-JAPI-1).

**Aserciones.**
- A1. Tuple4.create y Tuple22.create compilan — JavaAPI.java:147-151

**Oráculo.** No aplicable.

**Decisiones.** Q-JAPI-1

**Estado.** analysis=listed; impl=pending; validation=pending.

### AC-JAPI-005.3 — japi.Throwables: isFatal / isNonFatal con la semántica de scala.util.control.NonFatal

**Contrato observable.** isNonFatal delega en scala.util.control.NonFatal; isFatal es su negación. Scaladoc :20-24 enumera fatales.

**Fuente.** actor/src/main/scala/org/apache/pekko/japi/Throwables.scala:20-52

**Condiciones.** Sin errores.

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** Q-X-5

**Equivalente Rust propuesto.** Función `is_fatal` sobre el tipo de fallo; depende de qué se considere fatal en Rust (OOM, stack overflow, `abort`).

**Diferencias observables.** Los tipos JVM no existen: la lista de fatales se reinterpreta (Q-X-5).

**Aserciones.**
- A1. isNonFatal(IllegalArgumentException) es true — ThrowablesTest.java:20-23
- A2. isFatal es true para StackOverflowError, ThreadDeath, InterruptedException y LinkageError — ThrowablesTest.java:25-31

**Oráculo.** No aplicable.

**Decisiones.** Q-X-5

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-JAPI-005.4 — japi.JAPI (deprecado) y utilidades Java de colecciones/conversión

**Contrato observable.** Clase pública @Deprecated con el único método estático seq(T... ts) que devuelve scala.collection.Seq vía Util.immutableSeq.

**Fuente.** actor/src/main/java/org/apache/pekko/japi/JAPI.java:21-30

**Condiciones.** n/a

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AC-JAPI-005.1

**Equivalente Rust propuesto.** Sin equivalente.

**Diferencias observables.** La API Rust es una sola; no se porta la superficie Java (Q-JAPI-1).

**Aserciones.**
- A1. seq(ts) devuelve Util.immutableSeq(ts) — derivada del código JAPI.java:21-30; no afirmada por spec

**Oráculo.** No aplicable.

**Decisiones.** Q-JAPI-1

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-JAPI-006 — Props Java (Props.create con clase/args o Creator) y AbstractProps: validaciones y mensajes

**Contrato observable.** create(clazz, args...) = new Props(defaultDeploy, clazz, args.toList). validate(clazz): abstracta -> IllegalArgumentException; no subclase de Actor ni IndirectActorProducer -> IllegalArgumentException. create(Creator) (deprecado): infiere la clase del parámetro genérico del Creator; Creator erasado (lambda) -> IllegalArgumentException. create(actorClass, creator) = Props(CreatorConsumer, actorClass, creator). checkCreatorClosingOver rechaza Creators locales no estáticos.

**Fuente.** actor/src/main/scala/org/apache/pekko/actor/AbstractProps.scala:33-41 (validate) ; actor/src/main/scala/org/apache/pekko/actor/AbstractProps.scala:46-48 (create(clazz, args)) ; actor/src/main/scala/org/apache/pekko/actor/AbstractProps.scala:58-81 (create(creator)) ; actor/src/main/scala/org/apache/pekko/actor/AbstractProps.scala:87-90 (create(class, creator)) ; actor/src/main/scala/org/apache/pekko/actor/AbstractProps.scala:92-137 (checkCreatorClosingOver)

**Condiciones.** Mensajes: "Actor class [<FQCN>] must not be abstract" (:35); "Actor class [<FQCN>] must be subClass of org.apache.pekko.actor.Actor or org.apache.pekko.actor.IndirectActorProducer." (:38-39); "erased Creator types (e.g. lambdas) are unsupported, use Props.create(actorClass, creator) instead" (:75-76); "unsupported type found in Creator argument [<x>]" (:72); "unexpected type: <t>" (:78); "cannot use non-static local Creator to create actors; make it static (e.g. local to a static method) or top-level" (:136).

**Orden / concurrencia / ciclo de vida.** Todas las validaciones son síncronas al construir el Props (antes de actorOf).

**Dependencias.** AC-REF-007 (Props), AC-CELL

**Equivalente Rust propuesto.** `Props::new<A: Actor>(factory: impl Fn() -> A + Send + Sync + 'static)`; las validaciones de clase abstracta/closing-over no tienen equivalente (el sistema de tipos las impide).

**Diferencias observables.** No hay clase abstracta ni Creator erasado en Rust: esos errores no existen (Q-JAPI-1).

**Aserciones.**
- A1. Props.create(creator) y Props.create(classOf[Actor], creator) crean actores — PropsCreationSpec.scala:72-82
- A2. Props(Actor), Props(classOf, null) y Props(classOf, null, null) crean actores — PropsCreationSpec.scala:53-70
- A3. Props.create(JavaAPITestActor.class) / con Creator lambda / con Creator erasado crean actores — JavaAPI.java:66-96
- A4. Textos de validación y de closing-over — derivada del código AbstractProps.scala:35,38-39,72,75-78,136; no afirmada por spec

**Oráculo.** Jar oficial: jshell con Props.create inválidos y volcar mensajes. Escenario no existe ni se ejecutó.

**Decisiones.** Q-JAPI-1

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-JAPI-007 — EventBus Java (EventBus, LookupEventBus, SubchannelEventBus, ScanningEventBus, ManagedActorEventBus)

**Contrato observable.** EventBus[E,S,C]: subscribe(subscriber, to): Boolean, unsubscribe(subscriber, from): Boolean, unsubscribe(subscriber): Unit, publish(event). LookupEventBus: mapSize, compareSubscribers, classify, publish(event, subscriber). SubchannelEventBus: subclassification, classify, publish. ScanningEventBus: compareClassifiers, compareSubscribers, matches, publish. ManagedActorEventBus(system): mapSize, classify -> ActorRef.

**Fuente.** actor/src/main/scala/org/apache/pekko/event/japi/EventBusJavaAPI.scala:26-230 (firmas)

**Condiciones.** n/a (no leído)

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AC-EVT-001

**Equivalente Rust propuesto.** La API Rust es una sola; no se porta la superficie Java (Q-JAPI-1).

**Diferencias observables.** La API Rust es una sola; no se porta la superficie Java (Q-JAPI-1).

**Aserciones.**
- A1. Sin aserciones: no leído — n/a

**Oráculo.** No aplicable.

**Decisiones.** Q-JAPI-1

**Estado.** analysis=listed; impl=pending; validation=pending.

### AC-JAPI-008 — Sobrecargas con java.time.Duration, Optional y CompletionStage en Scheduler, ActorSelection, ActorSystem, etc.

**Contrato observable.** Cada sobrecarga convierte java.time.Duration a FiniteDuration y delega en la versión Scala; devuelve Optional/CompletionStage donde Scala usa Option/Future.

**Fuente.** citas de la columna evidence (no leídas)

**Condiciones.** n/a

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AC-SCH, AC-SEL, AC-SYS

**Equivalente Rust propuesto.** `std::time::Duration` + `Option` + `async fn`: una sola API.

**Diferencias observables.** La API Rust es una sola; no se porta la superficie Java (Q-JAPI-1).

**Aserciones.**
- A1. Sin aserciones: no leído — n/a

**Oráculo.** No aplicable.

**Decisiones.** Q-JAPI-1

**Estado.** analysis=listed; impl=pending; validation=pending.

### AC-JAPI-009.1 — TypedActor (clásico, deprecado): proxies JDK; semántica de invocación (void, Future, Option/JOption, valor)

**Contrato observable.** TypedActorInvocationHandler.invoke: toString/equals/hashCode se resuelven localmente contra el ActorRef; otros métodos: MethodCall(method,args): isOneWay (retorno void) -> actor ! m y null; returnsFuture -> ask(actor,m)(timeout).map(NullResponse -> null); returnsJOption/returnsOption -> Await.ready(ask, timeout) y, ante None/NullResponse/AskTimeoutException/TimeoutException, devuelve JOption.none/None; otro -> Await.result(ask,..) y NullResponse -> null. Las excepciones del método se propagan al llamante (reenvía Status.Failure) y el actor también falla (se aplica supervisión).

**Fuente.** actor/src/main/scala/org/apache/pekko/actor/TypedActor.scala:148-167 (MethodCall) ; actor/src/main/scala/org/apache/pekko/actor/TypedActor.scala:226-228 (NullResponse) ; actor/src/main/scala/org/apache/pekko/actor/TypedActor.scala:344-372 (receive del actor) ; actor/src/main/scala/org/apache/pekko/actor/TypedActor.scala:466-499 (invoke)

**Condiciones.** Excepciones del método (IllegalStateException("expected") en el spec) llegan al llamante y el estado del actor no se reinicia si la supervisión usa Resume (spec). Tiempos: ask con el timeout de TypedProps.

**Orden / concurrencia / ciclo de vida.** Las llamadas se encolan en el buzón del actor: un mismo proxy procesa un método a la vez. Futures completan fuera del hilo del actor.

**Dependencias.** AC-ASK-001 (ask), AC-CELL, AC-SUP

**Equivalente Rust propuesto.** Exclusión propuesta (Q-JAPI-3). Alternativa: trait objeto generado con macro que traduce cada método en un mensaje y respeta las 4 formas de retorno.

**Diferencias observables.** No existen proxies dinámicos en Rust; observable: el API no existe. Con macro, equivalente funcional.

**Aserciones.**
- A1. t.incr(); t.read()==1; t.incr(); t.read()==2; t.read()==2 (void unidireccional) — TypedActorSpec.scala:340-348
- A2. pigdog() devuelve "Pigdog"; nullOption/nullJOption/nullReturn/nullFuture devuelven None/none/null/null — TypedActorSpec.scala:350-362
- A3. futurePigdog(200ms) no está completo al llamar y resuelve "Pigdog"; 20 llamadas concurrentes resuelven "Pigdog"+i — TypedActorSpec.scala:364-379
- A4. joptionPigdog(100ms).get=="Pigdog" y con 2s y timeout 1s -> none; AskTimeoutException se trata como none; optionPigdog análogo (None) — TypedActorSpec.scala:381-399
- A5. Composición de futures sin bloqueo: "PIGDOG" — TypedActorSpec.scala:401-408
- A6. Excepciones: IllegalStateException("expected") llega al llamante en void, Future, JOption y Option; el estado (read()==1) se mantiene con Resume — TypedActorSpec.scala:410-441
- A7. toString/equals/hashCode del proxy coinciden con los del ActorRef — TypedActorSpec.scala:321-338

**Oráculo.** Jar oficial desde jshell/Scala: no aplicable al port si se excluye.

**Decisiones.** Q-JAPI-3, Q-X-5

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-JAPI-009.2 — TypedActor: self/context, hooks de ciclo de vida (PreStart, PostStop, PreRestart, PostRestart, Supervisor, Receiver), reinicio, stop/poisonPill, serialización de MethodCall

**Contrato observable.** TypedActor.self[T]: proxy del actor en curso (ThreadLocal) o IllegalStateException; context: ActorContext o IllegalStateException. El actor interno envuelve cada hook y cada método en withContext (fija self y context). Traits: Supervisor (supervisorStrategy()), PreStart, PostStop, PreRestart(reason, Option[message]), PostRestart(reason), Receiver (onReceive para mensajes no MethodCall). preRestart sin hook detiene a los hijos (no llama postStop). postStop apunta el proxy a deadLetters. stop(proxy)/poisonPill(proxy) devuelven false si no es proxy. MethodCall serializa vía SerializedMethodCall (parámetros con su serializador y manifest).

**Fuente.** actor/src/main/scala/org/apache/pekko/actor/TypedActor.scala:43-113 (TypedActorFactory) ; actor/src/main/scala/org/apache/pekko/actor/TypedActor.scala:148-224 (MethodCall/Serialized) ; actor/src/main/scala/org/apache/pekko/actor/TypedActor.scala:249-262 (self/context) ; actor/src/main/scala/org/apache/pekko/actor/TypedActor.scala:274-372 (TypedActor actor) ; actor/src/main/scala/org/apache/pekko/actor/TypedActor.scala:376-424 (traits)

**Condiciones.** IllegalStateException("Calling TypedActor.self outside of a TypedActor implementation method!") (:251); IllegalStateException("Calling TypedActor.context outside of a TypedActor implementation method!") (:260); SerializedMethodCall.readResolve sin sistema: IllegalStateException("Trying to deserialize a SerializedMethodCall without an ActorSystem in scope. Use org.apache.pekko.serialization.JavaSerializer.currentSystem.withValue(system) { ... }") (:203-205); SerializedTypedActorInvocationHandler: IllegalStateException("SerializedTypedActorInvocationHandler.readResolve requires that JavaSerializer.currentSystem.value is set to a non-null value") (:513-514).

**Orden / concurrencia / ciclo de vida.** Los hooks y métodos corren en el hilo del actor con self/context fijados; los hooks de ciclo de vida siguen el de Actor.

**Dependencias.** AC-CELL, AC-SUP, AC-SER-013 (Java serialization)

**Equivalente Rust propuesto.** Q-JAPI-3.

**Diferencias observables.** Ver AC-JAPI-009.1.

**Aserciones.**
- A1. isTypedActor(t) es true; stop(null) devuelve false — TypedActorSpec.scala:292-305
- A2. TypedActor.self fuera de ámbito -> IllegalStateException con el mensaje exacto — TypedActorSpec.scala:307-313
- A3. t.self === t dentro de una llamada — TypedActorSpec.scala:315-319
- A4. Reinicio tras fallo: la llamada siguiente funciona — TypedActorSpec.scala:443-456
- A5. Callbacks de ciclo de vida y Receiver ("pigdog" -> "dogpig"); latch de 16 llamadas — TypedActorSpec.scala:567-586
- A6. MethodCall, sus parámetros y el proxy se serializan y deserializan — TypedActorSpec.scala:493-565
- A7. Dispatcher balanceado con 60 actores y 120 llamadas — TypedActorSpec.scala:482-491

**Oráculo.** No aplicable al port si se excluye.

**Decisiones.** Q-JAPI-3

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-JAPI-009.3 — TypedProps, TypedActorExtension y createActorRefProxy (creación de proxies, interfaces, timeout, dispatcher)

**Contrato observable.** typedActorOf crea Props(new TypedActor(proxyVar, creator(), interfaces)).withDeploy(props.actorProps().deploy) y el proxy con createActorRefProxy; cada TypedProps trae interfaces, creator, timeout, dispatcher.

**Fuente.** actor/src/main/scala/org/apache/pekko/actor/TypedActor.scala:87-111 (typedActorOf (3 variantes))

**Condiciones.** n/a: no leído.

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AC-JAPI-009.1

**Equivalente Rust propuesto.** Q-JAPI-3.

**Diferencias observables.** Ver AC-JAPI-009.1.

**Aserciones.**
- A1. Se crea un proxy válido (isTypedActor) — TypedActorSpec.scala:292-296

**Oráculo.** No aplicable.

**Decisiones.** Q-JAPI-3

**Estado.** analysis=listed; impl=pending; validation=pending.

### AC-JAPI-010 — Fábricas Java de mensajes IO: TcpMessage, UdpMessage, UdpConnectedMessage, TcpSO, UdpSO y SoJavaFactories

**Contrato observable.** UdpMessage: noAck(token), noAck, send(payload, target[, ack]), bind(handler, endpoint[, options]), unbind, simpleSender([options]), suspendReading, resumeReading. UdpConnectedMessage: connect(handler, remote[, local][, options]), send(data[, ack]), disconnect, noAck, suspendReading, resumeReading. TcpSO/UdpSO: keepAlive, oobInline, tcpNoDelay, broadcast + SoJavaFactories (receiveBufferSize, reuseAddress, sendBufferSize, trafficClass).

**Fuente.** actor/src/main/scala/org/apache/pekko/io/Udp.scala:257-363 ; actor/src/main/scala/org/apache/pekko/io/UdpConnected.scala:194-275 ; actor/src/main/scala/org/apache/pekko/io/Inet.scala:187-219

**Condiciones.** n/a (delegan en los constructores Scala; ver filas IO).

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AC-IO-002.1, AC-IO-005

**Equivalente Rust propuesto.** La API Rust es una sola; no se porta la superficie Java (Q-JAPI-1).

**Diferencias observables.** La API Rust es una sola; no se porta la superficie Java (Q-JAPI-1).

**Aserciones.**
- A1. Cada fábrica construye el mensaje equivalente — derivada del código Udp.scala:257-352; no afirmada por spec

**Oráculo.** No aplicable.

**Decisiones.** Q-JAPI-1

**Estado.** analysis=listed; impl=pending; validation=pending.

### AC-JAPI-011 — Clases base Java del núcleo: AbstractScheduler, AbstractActorRef, AbstractPromiseActorRef, AbstractCircuitBreaker, JSerializer, annotation/* (@InternalApi, @DoNotInherit, @ApiMayChange)

**Contrato observable.** AbstractScheduler extiende AbstractSchedulerBase y expone schedule(initialDelay, interval, runnable, executor) (deprecado, FIXME #26910) con variantes java.time.Duration que convierten a FiniteDuration. Las clases Abstract* de actor/ y pattern/ declaran campos con offsets Unsafe para operaciones atómicas (sin API pública propia).

**Fuente.** actor/src/main/java/org/apache/pekko/actor/AbstractScheduler.java:14-90 ((lectura parcial))

**Condiciones.** n/a

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AC-SCH, AC-ASK, AC-CB

**Equivalente Rust propuesto.** Sin equivalente: los atómicos se expresan con `std::sync::atomic`.

**Diferencias observables.** La API Rust es una sola; no se porta la superficie Java (Q-JAPI-1).

**Aserciones.**
- A1. Sin aserciones: no leído — n/a

**Oráculo.** No aplicable.

**Decisiones.** Q-JAPI-1

**Estado.** analysis=listed; impl=pending; validation=pending.

## Descomposiciones

El padre desaparece del TSV; los hijos heredan `impl=pending`/`validation=pending` (ninguna de estas filas estaba implementada).

- AC-SER-001 -> AC-SER-001.1, AC-SER-001.2, AC-SER-001.3
- AC-SER-003 -> AC-SER-003.1, AC-SER-003.2, AC-SER-003.3, AC-SER-003.4, AC-SER-003.5
- AC-SER-004 -> AC-SER-004.1, AC-SER-004.2
- AC-SER-007 -> AC-SER-007.1, AC-SER-007.2
- AC-SER-008 -> AC-SER-008.1, AC-SER-008.2
- AC-IO-002 -> AC-IO-002.1, AC-IO-002.2, AC-IO-002.3, AC-IO-002.4, AC-IO-002.5, AC-IO-002.6, AC-IO-002.7
- AC-IO-004 -> AC-IO-004.1, AC-IO-004.2, AC-IO-004.3, AC-IO-004.4
- AC-IO-006 -> AC-IO-006.1, AC-IO-006.2, AC-IO-006.3
- AC-IO-008 -> AC-IO-008.1, AC-IO-008.2, AC-IO-008.3, AC-IO-008.4, AC-IO-008.5
- AC-IO-009 -> AC-IO-009.1, AC-IO-009.2, AC-IO-009.3, AC-IO-009.4, AC-IO-009.5, AC-IO-009.6, AC-IO-009.7, AC-IO-009.8
- AC-JAPI-004 -> AC-JAPI-004.1, AC-JAPI-004.2, AC-JAPI-004.3, AC-JAPI-004.4
- AC-JAPI-005 -> AC-JAPI-005.1, AC-JAPI-005.2, AC-JAPI-005.3, AC-JAPI-005.4
- AC-JAPI-009 -> AC-JAPI-009.1, AC-JAPI-009.2, AC-JAPI-009.3

## Omisiones agregadas

Cada una con la evidencia del archivo o clave de Pekko que el inventario no cubría (mismo criterio de ID: siguiente libre de la unidad).

- AC-SER-013 — JavaSerializer habilitado y serialización Java de ActorRef: `serialization/Serializer.scala:306-362`, `actor/ActorRef.scala:462-496`, `SerializeSpec.scala:402-511` (el inventario solo cubría la variante deshabilitada, AC-SER-004).
- AC-SER-014 — `SerializationExtension` y su registro como library-extension: `serialization/SerializationExtension.scala:24-29`, `actor/ActorSystem.scala:1167-1192`, `reference.conf:78`, `SerializeSpec.scala:565-587` (SerializerDeadlockSpec).
- AC-IO-012 — `io/UnsynchronizedByteArrayInputStream.java` + `UnsynchronizedByteArrayInputStreamSpec.scala`.
- AC-IO-013 — `io/ByteBufferCleaner.java` + `ByteBufferCleanerSpec.scala`.
- AC-JAPI-010 — fábricas Java de IO (`TcpMessage` en `Tcp.scala:702-`, `UdpMessage` `Udp.scala:257-352`, `UdpConnectedMessage` `UdpConnected.scala:194-275`, `TcpSO`/`UdpSO`, `SoJavaFactories` `Inet.scala:187-219`).
- AC-JAPI-011 — `actor/AbstractScheduler.java`, `AbstractActorRef.java`, `pattern/AbstractPromiseActorRef.java`, `pattern/AbstractCircuitBreaker.java`, `annotation/*.java`.

Además, la descomposición hizo explícitos comportamientos que el padre ocultaba (no son omisiones del inventario, pero no tenían fila propia): `NestedDeserialization` ya existía (AC-SER-009); nuevos hijos para UdpSettings (AC-IO-006.3), bucle de selección (AC-IO-004.4), TcpDnsClient (AC-IO-009.4), códec DNS (AC-IO-009.5), ResolvConfParser (AC-IO-009.6), IdGenerator (AC-IO-009.7) y AsyncDnsManager (AC-IO-009.8), cuyos archivos aparecían en `uncited-classic-main.txt` (`dns/RecordClass`, `dns/RecordType`, `dns/internal/AsyncDnsManager`, `AsyncDnsProvider`, `DnsMessage`, `DomainName`, `Question`, `RecordClassSerializer`, `RecordTypeSerializer`, `io/DnsProvider`, `InetAddressDnsProvider`, `TcpIncomingConnection`, `TcpOutgoingConnection`, `WithUdpSend`, `japi/pf/AbstractMatch`, `AbstractPFBuilder`, `UnitMatch`, `UnitPFBuilder`). Tests sin citar de `uncited-classic-tests.txt` ahora citados: `IdGeneratorSpec`, `NameserverAddressParserSpec`, `AsyncDnsManagerSpec`, `DomainNameSpec`, `MessageSpec`, `TcpDnsClientSpec`, `TcpIntegrationSpecSupport`, `PFBuilderTest`, `NoVerification.scala` (clase auxiliar de `NoVerificationWarningSpec`, AC-SER-004.2). No se leyeron `AsyncDnsResolverIntegrationSpec`/`DockerBindDnsService` (requieren Docker).

## Contradicciones halladas y corregidas

Inventario viejo vs. código/tests de 1.7.1.

1. **AC-IO-002** citaba `TcpConnectionSpec.scala:96` como spec de "backpressure por ack"; el test "stop writing in cases of backpressure and resume afterwards" está marcado `pending` (`TcpConnectionSpec.scala:316-320`) y no afirma nada; el backpressure se afirma con los tests ResumeWriting/useResumeWriting (`:710-841`). Corregido en AC-IO-002.5.
2. **AC-IO-010** citaba `CapacityLimitSpec.scala:27` como test del pool de buffers; ese spec prueba `max-channels` (AC-IO-004.2). El pool no tiene spec. Corregido.
3. **AC-SER-003** daba por afirmados los ids de los serializadores primitivos; las aserciones "have right serializer id" de `PrimitivesSerializationSpec.scala:81-84,104-107,126-129,150-153,180-183` usan `===` sin `should`/`assert` (no fallan nunca) y todas consultan el id de `Long`. Los ids salen de `reference.conf:843-847`, sin spec efectivo. Corregido en AC-SER-003.2-.4.
4. **AC-SER-007** citaba `ActorRefSpec.scala:288` como test de `serializedActorPath`; ese test prueba la serialización Java de un `LocalActorRef` (`ActorRef.scala:462-496`), no el formato del path. Movido a AC-SER-013; AC-SER-007.1 queda `listed`.
5. **AC-SER-001** evidencia `Serialization.scala:149-400`: el cuerpo real llega a la línea 595 (carga de serializadores y bindings en `:397-566`). Corregido en AC-SER-001.1/.2.
6. **AC-SER-012** daba consumidor en remote/cluster; en `pekko-actor` no hay lectura de `allowed-classes`/`whitelist-class` (grep sin resultados en `actor/src/main/{scala,java}`). Se mantiene como clave aceptada sin efecto.
7. **AC-IO-006 / AC-IO-007**: el inventario sugiere que `pekko.io.udp` se consume completa; `select-timeout` figura en `reference.conf:1046,1102` pero `UdpSettings` no lo lee (`Udp.scala:210-227`). Además `UdpConnection.doConnect(address)` ignora `address` y conecta a `remoteAddress` (`UdpConnection.scala:82-92`), así que conectar por nombre falla aunque el DNS resuelva; los specs solo prueban el fallo por host inexistente (`UdpConnectedIntegrationSpec.scala:61-77`). Registrado como Q-IO-5 (no verificado contra el jar). Además `UdpConnection.doRead` decrementa `readsLeft` sin comprobarlo (`UdpConnection.scala:132-141`): `receive-throughput` no limita el lote en UDP conectado (AC-IO-007).
8. **AC-JAPI-009** citaba `TypedActorSpec.scala:36`; los tests de TypedActor empiezan en `:290`. Corregido.
9. **Comentario obsoleto** en `Question.scala:31-33` ("DomainName.parse adds a '.' to the end") frente a `DomainName.scala:49-82`, que no añade punto final; `isSame` tolera el punto de todos modos (AC-IO-009.5).
10. **Mensajes con diferencias reales de texto**: `Serialization.scala:277` escribe "not in synch" y `:224` "not in sync"; `AsyncSerializer.scala:60-61` y `:67-68` difieren en "pekko persistence"/"Pekko persistence"; `DnsSettings.scala:94-95` pega "or" y la comilla invertida sin espacio. Se conservan como texto exacto en las filas.

## Filas que siguen listed

- AC-SER-007.1: no se leyó `toSerializationFormatWithAddress` (AC-PATH) ni el provider remoto; ningún spec afirma el formato.
- AC-SER-013: no se leyó `ClassLoaderObjectInputStream`, `resolveActorRef` ni `ActorRefSpec.scala:319-340`.
- AC-IO-004.4: no se leyó el soporte de `TcpConnectionSpec.scala:883-1142` que simula el registry.
- AC-JAPI-001: faltan `ReceiveBuilder.java` y `ActorCreationTest.java`.
- AC-JAPI-002: faltan `StashJavaAPI.java`, `Timers.scala:78-94`.
- AC-JAPI-003: solo firmas de `AbstractFSM` y un test; faltan los builders y `AbstractFSMWithStashActorTest.java`.
- AC-JAPI-004.2: solo firmas de `UnitMatch`; sin spec propio.
- AC-JAPI-004.3: `FI.java` no leído.
- AC-JAPI-005.1: se leyeron firmas, no cuerpos.
- AC-JAPI-005.2: plantillas de boilerplate no leídas.
- AC-JAPI-007: solo firmas, sin tests.
- AC-JAPI-008: sobrecargas no leídas.
- AC-JAPI-009.3: `TypedProps`/`TypedActorExtension` no leídos.
- AC-JAPI-010: `TcpMessage` no leído (sí `UdpMessage`/`UdpConnectedMessage` por firmas).
- AC-JAPI-011: solo `AbstractScheduler.java` parcialmente.

## Cobertura

Archivos del árbol de Pekko de esta área y las filas que los cubren.

**Serialización** (`actor/src/main/scala/org/apache/pekko/serialization/`): `AsyncSerializer.scala` -> AC-SER-006; `Decompression.scala` -> AC-SER-010; `NestedDeserialization.scala` -> AC-SER-009; `PrimitiveSerializers.scala` -> AC-SER-003.2/.3/.4; `Serialization.scala` -> AC-SER-001.1/.2/.3, AC-SER-004.2, AC-SER-007.1/.2; `SerializationExtension.scala` -> AC-SER-014; `SerializationSetup.scala` -> AC-SER-005; `Serializer.scala` -> AC-SER-002, AC-SER-003.1, AC-SER-004.1, AC-SER-013; `WireConfig.scala` -> AC-SER-011. Fuera de `serialization/`: `actor/dungeon/Dispatch.scala:36-41,171-223` -> AC-SER-008.1; `actor/dungeon/Children.scala:280-309` -> AC-SER-008.2; `actor/ActorRef.scala:462-496` -> AC-SER-013; `actor/ActorSystem.scala:1167-1192` -> AC-SER-014; `actor/Actor.scala:44` (`NoSerializationVerificationNeeded`) -> AC-SER-008.1/.2. `reference.conf`: `:78` AC-SER-014; `:129-140` AC-SER-008; `:761-797` AC-SER-004; `:780-785` AC-SER-009; `:799-848` AC-SER-003.5/AC-SER-001.2; `:852-863` AC-SER-010; `:865-880` AC-SER-012. Tests: `SerializeSpec`, `SerializationSetupSpec`, `DisabledJavaSerializerWarningSpec`, `PrimitivesSerializationSpec`, `AsyncSerializeSpec`, `DecompressionSpec`, `WireConfigSpec`, `NoVerification.scala` (clase auxiliar, sin comportamiento propio).

**IO** (`actor/src/main/scala/org/apache/pekko/io/`): `IO.scala` -> AC-IO-001; `Tcp.scala` -> AC-IO-002.1, AC-IO-003, AC-IO-005 (SO), AC-JAPI-010 (TcpMessage, TcpSO); `TcpManager.scala` -> AC-IO-004.1; `TcpListener.scala`, `TcpIncomingConnection.scala` -> AC-IO-002.2; `TcpOutgoingConnection.scala` -> AC-IO-002.3; `TcpConnection.scala` -> AC-IO-002.4/.5/.6/.7; `SelectionHandler.scala` -> AC-IO-003 (settings), AC-IO-004.1/.2/.3/.4; `Inet.scala` -> AC-IO-005; `Udp.scala` -> AC-IO-006.1/.2/.3; `UdpManager.scala` -> AC-IO-004.1; `UdpListener.scala` -> AC-IO-006.1; `UdpSender.scala`, `WithUdpSend.scala` -> AC-IO-006.2; `UdpConnected.scala`, `UdpConnectedManager.scala`, `UdpConnection.scala` -> AC-IO-007, AC-IO-004.1; `Dns.scala`, `DnsProvider.scala` -> AC-IO-008.1; `InetAddressDnsProvider.scala`, `SimpleDnsManager.scala` -> AC-IO-008.5; `InetAddressDnsResolver.scala` -> AC-IO-008.3; `SimpleDnsCache.scala` -> AC-IO-008.4; `DirectByteBufferPool.scala` -> AC-IO-010; Java: `UnsynchronizedByteArrayInputStream.java` -> AC-IO-012, `ByteBufferCleaner.java` -> AC-IO-013. `io/dns/`: `DnsProtocol.scala`, `DnsResourceRecords.scala`, `CachePolicy.scala`, `RecordType.scala`, `RecordClass.scala` -> AC-IO-008.2; `DnsSettings.scala` -> AC-IO-009.1; `IdGenerator.scala` -> AC-IO-009.7; `internal/AsyncDnsResolver.scala` -> AC-IO-009.2; `internal/DnsClient.scala` -> AC-IO-009.3; `internal/TcpDnsClient.scala` -> AC-IO-009.4; `internal/DnsMessage.scala`, `DomainName.scala`, `Question.scala`, `RecordTypeSerializer.scala`, `RecordClassSerializer.scala`, `package.scala` -> AC-IO-009.5; `internal/ResolvConfParser.scala` -> AC-IO-009.6; `internal/AsyncDnsManager.scala`, `AsyncDnsProvider.scala` -> AC-IO-009.8 / AC-IO-008.5. `reference.conf`: `:345-364` y `:930-938` AC-IO-011; `:940-1026` AC-IO-003; `:1028-1082` AC-IO-006.3; `:1084-1138` AC-IO-007; `:1140-1233` AC-IO-008.x / AC-IO-009.x. Tests: `TcpConnectionSpec`, `TcpIntegrationSpec`, `TcpIntegrationSpecSupport`, `TcpListenerSpec`, `CapacityLimitSpec`, `UdpIntegrationSpec`, `UdpConnectedIntegrationSpec`, `InetAddressDnsResolverSpec`, `SimpleDnsCacheSpec`, `UnsynchronizedByteArrayInputStreamSpec`, `ByteBufferCleanerSpec`, `dns/DnsSettingsSpec`, `dns/IdGeneratorSpec`, `dns/NameserverAddressParserSpec`, `dns/internal/*Spec` (AsyncDnsResolverSpec, DnsClientSpec, TcpDnsClientSpec, AsyncDnsManagerSpec, DomainNameSpec, MessageSpec, ResolvConfParserSpec). Sin leer: `dns/AsyncDnsResolverIntegrationSpec.scala`, `dns/DockerBindDnsService.scala` (requieren Docker).

**JAPI**: `actor/AbstractActor.scala` -> AC-JAPI-001/002; `actor/AbstractFSM.scala` y `japi/pf/FSM*Builder.java` -> AC-JAPI-003; `japi/pf/{Match,PFBuilder,AbstractMatch,AbstractPFBuilder}.java`, `pf/CaseStatements.scala` -> AC-JAPI-004.1; `UnitMatch.java`, `UnitPFBuilder.java` -> AC-JAPI-004.2; `FI.java` -> AC-JAPI-004.3; `DeciderBuilder.java` -> AC-JAPI-004.4; `japi/function/Function.scala`, `japi/JavaAPI.scala` -> AC-JAPI-005.1; `boilerplate/*.template` -> AC-JAPI-005.2; `japi/Throwables.scala` -> AC-JAPI-005.3; `japi/JAPI.java` -> AC-JAPI-005.4; `actor/AbstractProps.scala` -> AC-JAPI-006; `event/japi/EventBusJavaAPI.scala` -> AC-JAPI-007; `Scheduler.scala`, `ActorSelection.scala`, `ActorSystem.scala` sobrecargas -> AC-JAPI-008; `actor/TypedActor.scala` -> AC-JAPI-009.1/.2/.3; `actor/AbstractScheduler.java`, `AbstractActorRef.java`, `pattern/Abstract*.java`, `annotation/*.java` -> AC-JAPI-011. `dispatch/Abstract*.java`, `dispatch/affinity/` y `util/Unsafe.java`, `util/JavaVersion.scala`: pertenecen a MBX/DSP/UTIL y no son cubiertos por este grupo (sin comportamiento propio del módulo de IO/serialización/API Java; implementación interna de colas atómicas). Los archivos `annotation/ApiMayChange.java`, `DoNotInherit.java`, `InternalApi.java`, `InternalStableApi.java` son anotaciones sin comportamiento en tiempo de ejecución (solo metadatos de compilación). `actor/src/main/mima-filters` y `scala-2.13`/`scala-3` (compatibilidad de compilación) no definen comportamiento observable.
