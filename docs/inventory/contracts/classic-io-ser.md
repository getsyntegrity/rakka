# Contratos verificables: grupo classic-io-ser (SER, IO, JAPI)

Revisión del inventario de `pekko-actor` 1.7.1 (commit `9a54d129…`) para las unidades **SER** (serialización local), **IO** (`pekko.io`: TCP, UDP, UDP conectado, DNS) y **JAPI** (API Java). Solo documentación: no hay código Rust. Todas las filas siguen en alcance con `impl=pending` y `validation=pending`; las exclusiones son decisiones `proposed` (ver `classic-io-ser.decisions.tsv`). Las citas `path:línea` son relativas a `upstream/pekko`; las de rangos se contrastaron con `sed -n` sobre los archivos fijados.

Convenciones: "derivada del código" = ninguna spec afirma el comportamiento (no afirmada por spec); "oráculo" = comparación posible contra los jars oficiales con `tools/oracle/run.sh` (ningún escenario nuevo se escribió ni se ejecutó en esta revisión). Los IDs `Q-X-n` son los transversales del orquestador y no se redefinen aquí.

Resumen (cierre documental 2026-10-31): 79 filas, todas `extracted` (las 15 `listed` pasaron a `extracted` tras leer cuerpos de implementación y tests; se añadieron AC-JAPI-012 y AC-JAPI-013). Decisiones: ver al final.


### AC-SER-001.1 — Resolución de serializador por clase: serializerFor/findSerializerFor, bindings ordenados de más específico a menos, desempate, null -> NullSerializer

**Contrato observable.** Dado un tipo de mensaje, devuelve siempre el mismo serializador (resultado cacheado en serializerMap). Se elige entre los bindings cuya clase configurada es supertipo (isAssignableFrom) del tipo pedido; los bindings se ordenan con subtipos antes que supertipos, y entre no relacionados se respeta el orden de configuración. Si hay varios candidatos: si el primero es subtipo de todos o todos usan la misma instancia de serializador, gana el primero; si no, se descartan JavaSerializer/DisabledJavaSerializer y gana el primero restante, con un warning "Multiple serializers found" si aún no son unívocos. findSerializerFor(null) devuelve NullSerializer.

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:293-294 (findSerializerFor) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:305-369 (serializerFor (unique 330-336, filtro 338-343, desempate 344-365, cache 367-390)) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:513-526 (sort); doc pública docs/src/main/paradox/serialization.md:36-60 ; reference.conf actor/src/main/resources/reference.conf:857-879

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

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:50-61 (Settings) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:376-430 (serializerOf (3 constructores 405-424)) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:434-449 (serializerDetails) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:451-457 (serializers) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:462-476 (bindings ("none" y protobuf :462)) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:478-482 (ensureOnlyAllowedSerializers) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:501-511 (checkGoogleProtobuf) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:534-551 (serializerByIdentity (mensaje id duplicado 535-538)); actor/src/main/resources/reference.conf:846-896

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

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:167-174 (withTransportInformation) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:180-184 (serialize) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:185 (deserialize(clazz opt, deprecado)) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:191-202 (deserialize(id, manifest)) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:204-241 (deserializeByteArray (manifest 241-258)) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:248-275 (deserializeByteBuffer) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:281-285 (deserialize(clazz)) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:553-572 (tabla rápida/getSerializerById)

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

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/Serializer.scala:56-93 (Serializer) ; actor/src/main/scala/org/apache/pekko/serialization/Serializer.scala:95-103 (Serializers.manifestFor) ; actor/src/main/scala/org/apache/pekko/serialization/Serializer.scala:140-186 (SerializerWithStringManifest (includeManifest 136, adaptación 166-172)) ; actor/src/main/scala/org/apache/pekko/serialization/Serializer.scala:223-237 (ByteBufferSerializer) ; actor/src/main/scala/org/apache/pekko/serialization/Serializer.scala:244-274 (BaseSerializer) ; actor/src/main/scala/org/apache/pekko/serialization/Serializer.scala:275-296 (BaseSerializer.identifierFromConfig) ; actor/src/main/resources/reference.conf:881-896

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

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/Serializer.scala:432-442 (NullSerializer) ; actor/src/main/scala/org/apache/pekko/serialization/Serializer.scala:448-477 (ByteArraySerializer); actor/src/main/resources/reference.conf:849,865,889

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

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/PrimitiveSerializers.scala:27-64 (LongSerializer (toBinary 42-52, fromBinary 54-63)) ; actor/src/main/scala/org/apache/pekko/serialization/PrimitiveSerializers.scala:69-102 (IntSerializer); actor/src/main/resources/reference.conf:850-851,873-876,891-892

**Condiciones.** fromBinary con menos de 8 (4) bytes -> ArrayIndexOutOfBoundsException (derivado del código :57-60; no afirmado). toBinary de un objeto no Long -> ClassCastException/NullPointerException de Long.unbox (derivado).

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AC-SER-002, AC-SER-003.5

**Equivalente Rust propuesto.** `I64Serializer { id: 18 }`, `I32Serializer { id: 19 }` con `to_le_bytes`/`from_le_bytes` y variantes sobre `BufMut`.

**Diferencias observables.** Se pierde la distinción java.lang.Long vs scala.Long (un solo i64). Entradas mal tipadas devuelven `Err` en lugar de ClassCastException/AIOOBE.

**Aserciones.**
- A1. Para cada valor de {0,1,-1,Long.MinValue,Long.MinValue+1,Long.MaxValue,Long.MaxValue-1}: serializerFor(Long) es LongSerializer; fromBinary(toBinary(v))==v; mismos bytes Array/ByteBuffer LE y roundtrip por ByteBuffer — PrimitivesSerializationSpec.scala:64-79
- A2. Idem para Int con {0,1,-1,MinValue,MinValue+1,MaxValue,MaxValue-1} — PrimitivesSerializationSpec.scala:88-102
- A3. Id 18 y 19 por reference.conf — derivada del código actor/src/main/resources/reference.conf:891-892; la aserción del spec (PrimitivesSerializationSpec.scala:81-84,104-107) no es una aserción efectiva: usa === sin should/assert, y consulta el id de Long
- A4. Bytes LE concretos (p. ej. 1L -> 01 00 00 00 00 00 00 00) — derivada del código PrimitiveSerializers.scala:42-52; no afirmada por spec salvo igualdad Array/ByteBuffer

**Oráculo.** Jar oficial: volcar toBinary de los valores del spec; golden de bytes. Escenario no existe ni se ejecutó.

**Decisiones.** Q-SER-1

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-SER-003.3 — StringSerializer (id 20) y ByteStringSerializer (id 21)

**Contrato observable.** String: toBinary = bytes UTF-8; fromBinary = new String(bytes, UTF-8). ByteString: toBinary copia a array; fromBinary = ByteString(bytes) (copia); toBinary(buf) copia al buffer y lanza BufferOverflowException si no cabe.

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/PrimitiveSerializers.scala:107-127 (StringSerializer) ; actor/src/main/scala/org/apache/pekko/serialization/PrimitiveSerializers.scala:132-161 (ByteStringSerializer (overflow 143-144)); actor/src/main/resources/reference.conf:852-853,868-872

**Condiciones.** Buffer sin espacio suficiente -> java.nio.BufferOverflowException (:143-144; derivado, no afirmado). Bytes inválidos UTF-8 se reemplazan por U+FFFD (comportamiento del constructor String de JVM; derivado, no afirmado).

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AC-SER-002, AC-UTIL ByteString

**Equivalente Rust propuesto.** `StringSerializer` (id 20), `ByteStringSerializer` (id 21) sobre `bytes::Bytes` o el `ByteString` portado en AC-UTIL.

**Diferencias observables.** UTF-8 inválido: String::from_utf8_lossy reproduce el reemplazo U+FFFD; decisión: usar lossy para igualar. BufferOverflowException -> Err(BufferOverflow).

**Aserciones.**
- A1. Para "", "hello", "árvíztűrőütvefúrógép" y un String aleatorio: serializerFor(String)==StringSerializer y roundtrip Array y ByteBuffer idéntico — PrimitivesSerializationSpec.scala:132-148
- A2. Para ByteString vacío, simple, concatenado, slice y largo(2000 bytes) concatenado: serializerFor(clase concreta)==ByteStringSerializer y roundtrip — PrimitivesSerializationSpec.scala:157-178
- A3. Ids 20 y 21 — derivada del código actor/src/main/resources/reference.conf:893-894; la aserción del spec (PrimitivesSerializationSpec.scala:150-153,180-183) no es efectiva (=== sin assert)
- A4. BufferOverflowException en ByteString.toBinary(buf) sin espacio — derivada del código PrimitiveSerializers.scala:143-144; no afirmada por spec

**Oráculo.** Jar oficial: golden de bytes UTF-8 y ByteString. Escenario no existe ni se ejecutó.

**Decisiones.** Q-SER-1

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-SER-003.4 — BooleanSerializer (id 35): 1 byte, 0=false 1=true, rechaza otros valores

**Contrato observable.** toBinary: TRUE->1, FALSE->0, otro objeto -> IllegalArgumentException("Non boolean flag: <o>"); fromBinary: 1 -> TRUE, 0 -> FALSE, otro -> IllegalArgumentException("Non boolean flag byte: <b>") (se lee solo bytes(0); un array vacío falla con AIOOBE).

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/PrimitiveSerializers.scala:166-214 ((toBinary 196-205, fromBinary 207-213, buffer 179-194)); actor/src/main/resources/reference.conf:854,877-878,895

**Condiciones.** Mensajes exactos: "Non boolean flag: <objeto>" (:183,:200) y "Non boolean flag byte: <byte>" (:192,:211) (derivado; los tests no los comprueban).

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AC-SER-002

**Equivalente Rust propuesto.** `BoolSerializer { id: 35 }`

**Diferencias observables.** scala.Boolean y java.lang.Boolean se unifican en bool.

**Aserciones.**
- A1. Para false,true,Boolean.FALSE,Boolean.TRUE: serializerFor es BooleanSerializer; roundtrip Array y ByteBuffer — PrimitivesSerializationSpec.scala:110-124
- A2. Id 35 — derivada del código actor/src/main/resources/reference.conf:895; aserción del spec (PrimitivesSerializationSpec.scala:126-129) no efectiva
- A3. Mensajes "Non boolean flag" / "Non boolean flag byte" — derivada del código PrimitiveSerializers.scala:183,192,200,211; no afirmada por spec

**Oráculo.** Jar oficial: volcar bytes y errores. Escenario no existe ni se ejecutó.

**Decisiones.** -

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-SER-003.5 — Tabla por defecto de reference.conf: serializers, serialization-bindings y serialization-identifiers

**Contrato observable.** serializers: java, bytes, primitive-long, primitive-int, primitive-string, primitive-bytestring, primitive-boolean. serialization-bindings: "[B"=bytes, "java.io.Serializable"=java, java.lang.String, ByteString$ByteString1C/ByteString1/ByteStrings, java.lang.Long/scala.Long, java.lang.Integer/scala.Int, java.lang.Boolean/scala.Boolean. serialization-identifiers: JavaSerializer=1, ByteArraySerializer=4, y por alias primitive-*: 18,19,20,21,35 (reservados 0-40).

**Fuente.** actor/src/main/resources/reference.conf:846-896

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

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/Serializer.scala:352-357 (JavaSerializer ctor guard) ; actor/src/main/scala/org/apache/pekko/serialization/Serializer.scala:379-422 (DisabledJavaSerializer (toBinary 379-386, fromBinary 388-403)) ; actor/src/main/scala/org/apache/pekko/serialization/Serializer.scala:424-430 (excepciones); actor/src/main/resources/reference.conf:808-825

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

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:346-369 (aviso al resolver) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:384-390 (y 431-446 reemplazo) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:478-482 (requisito) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:484-499 (warnUnexpectedNonPekkoSerializer) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:573-601 (flags y shouldWarnAboutJavaSerializer) ; actor/src/main/resources/reference.conf:808-844

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

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/SerializationSetup.scala:22-45 (SerializationSetup) ; actor/src/main/scala/org/apache/pekko/serialization/SerializationSetup.scala:48-79 (SerializerDetails) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:434-449 (serializerDetails) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:451-457 (serializers) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:462-476 (bindings)

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

**Contrato observable.** `Serialization.serializedActorPath(actorRef)` produce la cadena con la que se serializa una ref. Sin `Information` vigente (thread-local nula): si la ref es `ActorRefWithCell` usa `path.toSerializationFormatWithAddress(system.provider.getDefaultAddress)` y, si eso lanza una excepción no fatal, `path.toSerializationFormat`; si no es `ActorRefWithCell` (p. ej. `DeadLetterActorRef`, `EmptyLocalActorRef`, refs de promesa) usa `path.toSerializationFormat`. Con `Information(address, system)`: si la ref no tiene celda o su sistema es el mismo (`==`) que el de la Information usa `toSerializationFormatWithAddress(address)`; si pertenece a otro sistema usa `provider.getExternalAddressFor(address).getOrElse(provider.getDefaultAddress)` de ese otro sistema.

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:63-92 (serializedActorPath) ; actor/src/main/scala/org/apache/pekko/actor/ActorPath.scala:276-285 (contrato de los dos formatos) ; actor/src/main/scala/org/apache/pekko/actor/ActorPath.scala:325-333 (RootActorPath) ; actor/src/main/scala/org/apache/pekko/actor/ActorPath.scala:402-406,426-443,477-480 (ChildActorPath, fragmento `#uid`) ; actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:443-448 (único consumidor: SerializedActorRef) ; actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:766-768 (getExternalAddressFor/getDefaultAddress del provider local) ; actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:513-532 (IgnoreActorRef.path).

**Precondiciones.** La ref pertenece a un sistema en marcha del que se pueda leer `provider`. Para la rama con `Information` la variable thread-local debe haberse fijado (`serialize`/`deserialize`/`withTransportInformation`, AC-SER-007.2).

**Comportamiento.** (1) `RootActorPath.toSerializationFormat` es `address.toString + name`; `toSerializationFormatWithAddress(addr)` reemplaza la dirección sólo si el root no tiene host (`address.host.isDefined` -> conserva la propia; si no usa `addr`) (ActorPath.scala:297-305). (2) `ChildActorPath.toSerializationFormat` añade `#<uid>` cuando `uid != undefinedUid(0)` (`appendUidFragment`, :449-452); `toSerializationFormatWithAddress` aplica la misma regla de reemplazo de dirección (sólo si el root no tiene host, `addressStringLengthDiff` :410-414) y añade el fragmento uid. (3) Excepción: para el path de `IgnoreActorRef` (`pekko://local/ignore`) nunca se cambia la dirección ni se añade uid: devuelve `this.toString` (:399-402). (4) Con provider local, `getExternalAddressFor(addr)` es `Some(addr)` sólo si `addr == rootPath.address`, si no `None` (-> `getDefaultAddress` = `rootPath.address`). Consecuencia para un sistema local sin remote: toda serialización de una ref local da `pekko://<sistema>/<ruta>#<uid>` con la dirección raíz del sistema (sin host/puerto); el resultado sólo difiere entre ramas si hay `Information` con una dirección distinta, caso que en `pekko-actor` sólo ocurre con otro provider (remote/cluster, fuera del módulo).

**Errores.** No lanza en la rama sin Information para refs con celda: cualquier `NonFatal` de `getDefaultAddress` degrada a `toSerializationFormat` (Serialization.scala:83-84). Sin celda y sin Information no hay camino de error. Con Information el cálculo no está protegido: una excepción de `getExternalAddressFor` se propaga al serializador.

**Orden / concurrencia.** Lee la variable thread-local `currentTransportInformation` en el hilo que serializa (AC-SER-007.2); no hay estado propio. `getDefaultAddress` es `val`-like y estable.

**Dependencias.** AC-PATH (ActorPath/Address, `#uid`), AC-REF-023 (SerializedActorRef/writeReplace), AC-SER-007.2, AC-SYS-018 (provider).

**Equivalente Rust propuesto.** `fn serialized_actor_path(actor_ref: &ActorRef, ctx: Option<&TransportInformation>) -> String`. Viable y exacto para el caso local: el path ya conoce su dirección raíz y uid. Se conserva: formato `<dirección-raíz><ruta>#<uid>` (sin `#` si uid=0), excepción de `IgnoreActorRef`, degradación a la forma sin dirección. Límite: las ramas "otra Information" y `getExternalAddressFor` distinta de la default sólo tienen sentido con un provider no local (remote/cluster), fuera de esta fase; se implementan con un `TransportInformation` explícito y pruebas con un provider simulado (Q-SER-3).

**Aserciones.**
- A1. Ref local con celda, sin Information: devuelve `<path con dirección raíz>#<uid>`; la ref de un actor `/user/a` de un sistema `S` da `pekko://S/user/a#<uid>` — *no afirmada por spec*. Verificada contra el jar publicado 1.7.1 (`ORACLE=jars tools/oracle/run.sh` con un script jshell local no versionado, 2026-10-31; es el cross-check de jars, no el build desde fuente): un actor de usuario en el sistema `O1` da `pekko://O1/user/$b#1313291330`; el root guardian da `pekko://O1/` (sin uid). Caracterización propuesta: crear el actor, leer `ref.path.uid` y comparar la cadena. Respaldo indirecto de la forma de `toSerializationFormat`: ActorSelectionSpec.scala:352-357 afirma `ActorSelection.toSerializationFormat` (`pekko://ActorSelectionSpec/user/c2`), que es otra API.
- A2. Ref sin celda (DeadLetterActorRef) sin Information: `path.toSerializationFormat` — *no afirmada por spec*. Verificada contra el jar publicado 1.7.1 (`ORACLE=jars tools/oracle/run.sh` con un script jshell local no versionado, 2026-10-31; es el cross-check de jars, no el build desde fuente): `deadLetters` da `pekko://O1/deadLetters` (uid 0, sin fragmento). Caracterización propuesta: serializar `deadLetters`. (ActorRefSpec.scala:288-317 y SerializeSpec.scala:491-508 usan este formato sólo por ida y vuelta Java, no comparan la cadena.)
- A3. Con `Information(addr, mismoSistema)` el root sin host se reescribe con `addr` — *no afirmada por spec*. Respaldo parcial: ActorPathSpec.scala:84-107 afirma `toStringWithAddress` (mismo reemplazo de dirección, incluido que un root con host conserva la suya, :102-105), pero no `toSerializationFormatWithAddress` ni el `#uid`.
- A4. Con `Information` de otro sistema usa `getExternalAddressFor(addr)` o la default — *no afirmada por spec*; necesita provider simulado.
- A5. `IgnoreActorRef` conserva `pekko://local/ignore` sin uid ni cambio de dirección — *no afirmada por spec*. Verificada contra el jar publicado 1.7.1 (`ORACLE=jars tools/oracle/run.sh` con un script jshell local no versionado, 2026-10-31; es el cross-check de jars, no el build desde fuente): `system.provider.ignoreRef` da `pekko://local/ignore`.

**Validación prevista.** Diferencial con el jar oficial (escenario nuevo `tools/oracle/ser-actor-path.jsh` a crear en implementación: sistema local, actor hijo, `deadLetters`, con y sin `withTransportInformation`; A1, A2 y A5 ya medidos a mano) para A1-A3 y A5, y test de caracterización en Rust para A4 con provider simulado. Justificación: ningún spec de Pekko afirma el formato; el oráculo es la única fuente de verdad ejecutable.

**Decisiones.** Q-SER-3 (cómo viaja la Information: parámetro explícito o thread/task-local). Efecto: no cambia el formato; sólo la superficie de `serialized_actor_path`. Puede avanzar el caso local sin esperar al Owner.

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-SER-007.2 — Información de transporte (Serialization.Information, currentTransportInformation, withTransportInformation, getCurrentTransportInformation)

**Contrato observable.** currentTransportInformation es una DynamicVariable (thread-local) inicialmente null. serialize/deserialize fijan provider.serializationInformation si no hay valor y restauran el anterior en finally. Serialization.withTransportInformation(system)(f) fija el valor (no re-fija si ya es el mismo objeto). serializationInformation del provider local = Information(rootPath.address, system) cacheada; "Too early access of serializationInformation" si system es null.

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:48 (variable) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:99 (Information) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:113-121 (withTransportInformation) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:131-138 (getCurrentTransportInformation) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:167-174 (helper privado) ; actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:771-787 (serializationInformation)

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

**Fuente.** actor/src/main/scala/org/apache/pekko/actor/dungeon/Dispatch.scala:34-39 (excepción) ; actor/src/main/scala/org/apache/pekko/actor/dungeon/Dispatch.scala:172-179 (sendMessage) ; actor/src/main/scala/org/apache/pekko/actor/dungeon/Dispatch.scala:181-207 (serializeAndDeserialize) ; actor/src/main/scala/org/apache/pekko/actor/dungeon/Dispatch.scala:209-224 (payload) ; actor/src/main/resources/reference.conf:129-140 (serialize-messages, no-serialization-verification-needed-class-prefix)

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

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/NestedDeserialization.scala:43-45 (contador) ; actor/src/main/scala/org/apache/pekko/serialization/NestedDeserialization.scala:51-61 (atNextLevel) ; actor/src/main/scala/org/apache/pekko/serialization/NestedDeserialization.scala:64 (currentDepth) ; actor/src/main/scala/org/apache/pekko/serialization/Serialization.scala:160-161, (236-237) ; actor/src/main/resources/reference.conf:827-832

**Condiciones.** Error: NotSerializableException("Message exceeds the maximum deserialization nesting depth of [<max>]. Configure with 'pekko.actor.serialization-max-nesting-depth'.") (NestedDeserialization.scala:56-58). Con max=32 el nivel 33 falla.

**Orden / concurrencia / ciclo de vida.** Estado thread-local; no cruza hilos ni futuros.

**Dependencias.** AC-SER-001.3, AC-CFG

**Equivalente Rust propuesto.** `NestedDeserializationGuard` con contador thread_local (o parámetro de contexto) y `SerializationSettings::max_nesting_depth`.

**Diferencias observables.** Rust no tiene StackOverflowError recuperable; el límite sigue siendo necesario para evitar recursión sin cota.

**Aserciones.**
- A1. Nivel 33 con max=32 lanza NotSerializableException con el texto exacto; el contador vuelve a 0 después (también tras la excepción) — derivada del código NestedDeserialization.scala:51-61; no afirmada por spec
- A2. Valor por defecto 32 y documentación de qué cuenta como anidado (Some, Optional, Status.Failure, StatusReply, causa de Throwable, ActorSelectionMessage) — derivada del código actor/src/main/resources/reference.conf:827-832; no afirmada por spec

**Oráculo.** Jar oficial: jshell llamando NestedDeserialization (private[pekko], accesible en bytecode). Escenario no existe ni se ejecutó.

**Decisiones.** -

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-SER-010 — Límite de descompresión gzip (pekko.serialization.max-decompressed-size) y Decompression.gunzip

**Contrato observable.** maxDecompressedSize(system): lee la cadena de pekko.serialization.max-decompressed-size; "unlimited" -> -1; entero negativo -> ese valor; si no, config.getBytes (p. ej. "16 KiB" = 16384). gunzip lee en bloques de 4096 acumulando total; si maxDecompressedSize >= 0 y total > max lanza antes de seguir leyendo.

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/Decompression.scala:42-53 (maxDecompressedSize) ; actor/src/main/scala/org/apache/pekko/serialization/Decompression.scala:60-79 (gunzip (bloque 37, chequeo 71-75)) ; actor/src/main/resources/reference.conf:900-911

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

**Fuente.** actor/src/main/resources/reference.conf:913-928

**Condiciones.** Sin comportamiento en pekko-actor. Debe seguir parseando como HOCON y sustituir ${pekko.serialization.protobuf.whitelist-class}.

**Orden / concurrencia / ciclo de vida.** n/a

**Dependencias.** AC-CFG (sustituciones), módulos fuera de alcance (remote/cluster)

**Equivalente Rust propuesto.** Clave aceptada en `reference.conf` embebido; sin tipo.

**Diferencias observables.** Sin diferencia observable en el módulo.

**Aserciones.**
- A1. El reference.conf embebido define ambas claves y la sustitución se resuelve — derivada del archivo actor/src/main/resources/reference.conf:913-928; no afirmada por spec

**Oráculo.** No aplica: sin comportamiento en pekko-actor.

**Decisiones.** Q-SER-6

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-SER-013 — JavaSerializer habilitado: toBinary/fromBinary con ObjectStream y JavaSerializer.currentSystem; serialización Java de ActorRef (SerializedActorRef)

**Contrato observable.** Con `pekko.actor.allow-java-serialization = on` el binding `java.io.Serializable` resuelve a `JavaSerializer` (en lugar de `DisabledJavaSerializer`, AC-SER-004.1). `JavaSerializer(system)`: `includeManifest=false`; `toBinary` serializa con `ObjectOutputStream` dentro de `JavaSerializer.currentSystem.withValue(system)`; `fromBinary` lee con `ClassLoaderObjectInputStream(system.dynamicAccess.classLoader, UnsynchronizedByteArrayInputStream(bytes))` dentro del mismo `withValue`. Una ref a actor (`LocalActorRef`, `RepointableActorRef`, `MinimalActorRef` y derivados) define `writeReplace()` -> `SerializedActorRef(path)` con `path = Serialization.serializedActorPath(ref)` (AC-SER-007.1); `readResolve()` resuelve con `currentSystem.value.provider.resolveActorRef(path)`.

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/Serializer.scala:318-347 (JavaSerializer.currentSystem) ; actor/src/main/scala/org/apache/pekko/serialization/Serializer.scala:352-374 (JavaSerializer) ; actor/src/main/scala/org/apache/pekko/util/ClassLoaderObjectInputStream.scala:25-31 ; actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:434-468 (writeReplace de ActorRefWithCell, SerializedActorRef) ; actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:492-493 (MinimalActorRef.writeReplace) ; actor/src/main/scala/org/apache/pekko/actor/RepointableActorRef.scala:200 ; actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:629-664 (resolveActorRef) ; actor/src/main/resources/reference.conf:808-825,846-879 (allow-java-serialization, bindings).

**Precondiciones.** `allow-java-serialization = on`; si está `off`, construir `JavaSerializer` lanza `DisabledJavaSerializer.JavaSerializationException("Attempted creation of `JavaSerializer` while `pekko.actor.allow-java-serialization = off` was set!")` (Serializer.scala:341-343). Para deserializar refs debe haber `currentSystem` fijada.

**Comportamiento.** (1) Ida y vuelta de objetos `Serializable` arbitrarios (case classes anidadas, records, lambdas serializables) por el formato estándar de Java. (2) `fromBinary` carga clases con el classloader del sistema y cae al resolver estándar si no la encuentra (ClassLoaderObjectInputStream.scala:26-30). (3) Una ref viva se deserializa a la MISMA instancia (`eq`) si el path resuelve a un actor vivo del mismo sistema. (4) Si el actor ya no existe (`getChild` -> `Nobody`), devuelve `EmptyLocalActorRef(provider, ref.path / elems, eventStream)` y registra un debug (ActorRefProvider.scala:648-656). (5) Path inválido o de otra dirección -> `deadLetters` con debug "Resolve (deserialization) of unknown (invalid) path [..], using deadLetters." (:622-627); path vacío -> `deadLetters` (:644-646). (6) `DeadLetterActorRef` (un `MinimalActorRef`) se deserializa a `system.deadLetters` del sistema en `currentSystem`. (7) `ActorCell` no es serializable (`NotSerializableException`).

**Errores.** Sin `currentSystem` al leer una ref: `IllegalStateException("Trying to deserialize a serialized ActorRef without an ActorSystem in scope. Use 'org.apache.pekko.serialization.JavaSerializer.currentSystem.withValue(system) { ... }'")` (ActorRef.scala:479-483), texto exacto afirmado por ActorRefSpec.scala:334-338. Objeto no serializable: `java.io.NotSerializableException` del flujo Java. Clase no encontrada al leer: `ClassNotFoundException` del flujo.

**Orden / concurrencia.** `currentSystem` es una `DynamicVariable` (thread-local heredable por `InheritableThreadLocal`); válida sólo durante el `withValue`. No hay estado compartido en el serializador.

**Dependencias.** AC-SER-004.1/.2, AC-SER-007.1, AC-REF-023, AC-IO-012 (UnsynchronizedByteArrayInputStream), AC-PATH (resolución de paths).

**Equivalente Rust propuesto.** Sin equivalente para el formato Java (Q-SER-2: exclusión propuesta). Lo que SÍ es contrato transferible y se mantiene en alcance: la resolución de una ref serializada como path (`resolve_actor_ref(path)`: misma ref viva / `EmptyLocalActorRef` / `deadLetters` con sus mensajes de debug) y el error de "sin sistema en alcance", expuestos mediante un serializador propio de refs (`ActorRefSerializer`) que use `serialized_actor_path`. Diferencia: la identidad `eq` de la ref se sustituye por igualdad de path+uid.

**Aserciones.**
- A1. Con allow-java=on, `Serializable` -> `JavaSerializer` — SerializeSpec.scala:418-420 (test "declare Serializable classes to be use JavaSerializer").
- A2. `Address`, `Person` y `Record` hacen ida y vuelta por `ser.serialize`/`deserialize` — SerializeSpec.scala:465-476 (`===` sobre el valor original).
- A3. Una función `Int => Int` se serializa con `JavaSerializer` y `f2(3) == 4` — SerializeSpec.scala:437-444.
- A4. Un `ActorCell` no es serializable (`NotSerializableException` capturada en el actor) — SerializeSpec.scala:478-489.
- A5. `DeadLetterActorRef` se deserializa a la misma instancia `a.deadLetters` bajo `currentSystem.withValue` — SerializeSpec.scala:491-508.
- A6. Un `LocalActorRef` escrito con `ObjectOutputStream` y leído con `currentSystem` vuelve como `ActorRefWithCell` y `eq` a la original, tanto por `ObjectInputStream` como por `new JavaSerializer(esys).fromBinary(bytes, None)` — ActorRefSpec.scala:288-317.
- A7. Leer una ref sin `currentSystem` lanza `IllegalStateException` con el texto exacto de arriba — ActorRefSpec.scala:319-339.
- A8. Si el actor ya murió (PoisonPill + `expectTerminated`), la lectura devuelve `EmptyLocalActorRef(provider, ref.path, eventStream)` (`===`) — ActorRefSpec.scala:341-365.
- A9. Construir `JavaSerializer` con allow-java=off lanza `JavaSerializationException` con el texto citado — *no afirmada por spec* (el comportamiento de `off` se afirma sólo por el reemplazo con `DisabledJavaSerializer`, SerializeSpec.scala:372-374). Caracterización propuesta sólo si el Owner decide no excluir (Q-SER-2).
- A10. Path inválido/foráneo -> `deadLetters` con debug — *no afirmada por spec* (derivada de ActorRefProvider.scala:622-638); caracterización propuesta con un path de otra dirección.

**Validación prevista.** Si Q-SER-2 acepta la exclusión: A1-A5, A9 no se portan (JVM-específicos) y se documentan como excluidos; A6-A8 y A10 se portan como pruebas del serializador de refs de Rust (misma secuencia: serializar, parar o no, deserializar, comparar), más un diferencial con el jar para A10. Si no se acepta: no hay forma razonable de emular el formato binario de Java; se registraría como divergencia funcional para el Owner. Justificación: los specs de ActorRefSpec y SerializeSpec afirman comportamientos de resolución que sobreviven a la exclusión del formato.

**Decisiones.** Q-SER-2 (excluir Java serialization). Efecto: A1-A5 y A9 quedan excluidos con la decisión; A6-A8 se reformulan sobre el serializador de refs. La fila sigue en alcance y `impl=pending`. Bloquea sólo la parte JVM; el serializador de refs puede avanzar.

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-SER-014 — SerializationExtension: registro como library-extension, acceso y prohibición de pedirla desde el constructor de un serializador

**Contrato observable.** SerializationExtension es un ExtensionId[Serialization] cargado por pekko.library-extensions. Mientras se construye, la extensión figura "en registro" (CountDownLatch); otro hilo/llamada que pida la extensión espera hasta creation-timeout y falla.

**Fuente.** actor/src/main/scala/org/apache/pekko/serialization/SerializationExtension.scala:24-29 ; actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1198-1223 (findExtension (mensaje 1182-1186)) ; actor/src/main/resources/reference.conf:78

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

**Fuente.** actor/src/main/scala/org/apache/pekko/io/IO.scala:24-38 ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:656-665 ; actor/src/main/scala/org/apache/pekko/io/Udp.scala:238-247 ; actor/src/main/scala/org/apache/pekko/io/UdpConnected.scala:176-187 ; actor/src/main/scala/org/apache/pekko/io/Dns.scala:129, (242-249)

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

**Fuente.** actor/src/main/scala/org/apache/pekko/io/Tcp.scala:112-121 (Message/Command) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:132-139 (Connect) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:159-166 (Bind) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:186-187 (Register) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:201-254 (CloseCommand/Close/ConfirmedClose/Abort) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:262-268 (NoAck) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:275-319 (WriteCommand) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:326-344 (SimpleWriteCommand) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:356-372 (Write) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:373-387 (WriteFile/WritePath) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:397-412 (CompoundWrite) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:471-504 (CommandFailed) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:540-605 (ConnectionClosed y subtipos)

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

**Fuente.** actor/src/main/scala/org/apache/pekko/io/TcpOutgoingConnection.scala:52-57 (inicio) ; actor/src/main/scala/org/apache/pekko/io/TcpOutgoingConnection.scala:61-72 (stop/reportConnectFailure) ; actor/src/main/scala/org/apache/pekko/io/TcpOutgoingConnection.scala:74-92 (receive y DNS) ; actor/src/main/scala/org/apache/pekko/io/TcpOutgoingConnection.scala:94-106 (resolving) ; actor/src/main/scala/org/apache/pekko/io/TcpOutgoingConnection.scala:108-118 (register) ; actor/src/main/scala/org/apache/pekko/io/TcpOutgoingConnection.scala:120-147 (connecting) ; actor/src/main/scala/org/apache/pekko/io/TcpOutgoingConnection.scala:149-153 (timeout) ; actor/src/main/scala/org/apache/pekko/io/TcpOutgoingConnection.scala:158-164 (excepciones) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:221-247 (completeConnect)

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

**Fuente.** actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:74-90 (Register) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:92-96 (Resume/Suspend antes de Register) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:110-116 (connected) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:249-256 (suspend/resume) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:263-307 (doRead)

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

**Fuente.** actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:159-207 (handleWriteMessages) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:438-452 (PendingWrite) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:454-504 (PendingBufferWrite) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:506-548 (PendingWriteFile) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:596-600 (excepciones) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:346-372 (Write) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:384-387 (WritePath) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:421 (ResumeWriting) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:515-516 (WritingResumed)

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

**Fuente.** actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:118-123 (peerSentEOF) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:126-148 (closingWithPendingWrite) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:151-157 (closing) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:315-344 (handleClose) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:346-353 (doCloseConnection/handleError) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:355-370 (safeShutdownOutput/extractMsg) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:372-402 (prepareAbort/stopWith) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:404-434 (postStop)

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

**Fuente.** actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:56-69 (death pact) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:74-107 (waitingForRegistration (timeout 104-108)) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:209-216 (unregistering) ; actor/src/main/scala/org/apache/pekko/io/TcpConnection.scala:388-437 (stopWith/postStop) ; actor/src/main/scala/org/apache/pekko/io/SelectionHandler.scala:136-149 (connectionSupervisorStrategy)

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

**Fuente.** actor/src/main/scala/org/apache/pekko/io/Tcp.scala:608-652 (Settings (nr-of-selectors 614, batch 616, buffers 617-618, register-timeout 619-622, max-received 623-626, transferTo 629-632, per-selector 634, retries 635-636, windows 638-642, getIntBytes 644-649)) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:656-669 (manager/bufferPool/fileIoDispatcher) ; actor/src/main/scala/org/apache/pekko/io/SelectionHandler.scala:39-54 ; actor/src/main/resources/reference.conf:992-1078

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

**Contrato observable.** Costura interna entre los actores de canal (TcpConnection, TcpListener, UdpListener, UdpSender, UdpConnection) y el selector NIO. `ChannelRegistry.register(channel, initialOps)(implicit channelActor)` registra el canal en el selector del `SelectionHandler` y entrega al `channelActor` un `ChannelRegistration` con `enableInterest(op)`, `disableInterest(op)` y `cancelAndClose(andThen)`. Un bucle de select (`MaxSelectMillis = 10000` ms) traduce disponibilidad en mensajes al actor del canal: `OP_READ` -> `ChannelReadable`; `OP_WRITE` -> `ChannelWritable`; ambos -> `ChannelWritable` y luego `ChannelReadable`; `OP_ACCEPT` -> `ChannelAcceptable`; `OP_CONNECT` -> `ChannelConnectable`. Antes de enviar, borra de `interestOps` los ops listos (evita re-selección inmediata); el actor debe volver a habilitar el interés.

**Fuente.** actor/src/main/scala/org/apache/pekko/io/SelectionHandler.scala:59-82 (ChannelRegistry, ChannelRegistration) ; actor/src/main/scala/org/apache/pekko/io/SelectionHandler.scala:84-110 (MaxSelectMillis 90, mensajes 106-109) ; actor/src/main/scala/org/apache/pekko/io/SelectionHandler.scala:151-310 (ChannelRegistryImpl: bucle 161-198, register 202-223, shutdown 225-239, interest/cancelAndClose 241-291, Task 293-309) ; actor/src/main/scala/org/apache/pekko/io/SelectionHandler.scala:313-337 (el SelectionHandler crea el registry con `SerializedSuspendableExecutionContext(dispatcher.throughput)(selector-dispatcher)` y lo apaga en `postStop`) ; actor/src/main/resources/reference.conf:982-990,1016-1052 (selector-dispatcher; las claves `select-timeout` de UDP, :1046 y :1102, no las consume el bucle: `MaxSelectMillis` es constante) ; actor-tests/src/test/scala/org/apache/pekko/io/TcpConnectionSpec.scala:930-1187 (registry simulado) ; actor-tests/src/test/scala/org/apache/pekko/io/TcpListenerSpec.scala:36-142,163,187 ; actor-tests/src/test/scala/org/apache/pekko/io/CapacityLimitSpec.scala:22-52 (único spec con selector real).

**Precondiciones.** Un `SelectionHandler` vivo con su dispatcher de selector. Los canales deben ser no bloqueantes (los crea cada actor de canal).

**Comportamiento.** (1) Bucle: `selector.select(MaxSelectMillis) > 0` -> recorre `selectedKeys`; por cada key válida lee `readyOps`, ejecuta `key.interestOps(interestOps & ~readyOps)` y envía el mensaje al `attachment` (el actor); al final `keys.clear()`; `wakeUp.set(false)`. (2) Tras cada vuelta el Task se re-encola en el contexto de ejecución, detrás de las tareas ya en cola (reprogramación). (3) `register`: tarea en el hilo del selector que hace `channel.register(selector, initialOps, channelActor)` y envía un `ChannelRegistration` al actor; `ClosedChannelException` se ignora (carrera con cierre). (4) `enableInterest`/`disableInterest` encolan una tarea que calcula `newOps` y sólo llama a `key.interestOps` si cambia; siempre en el hilo del selector (comentario "benchmarks show ... lock contention"). (5) `cancelAndClose(andThen)`: tarea que hace `key.cancel()` y `channel.close()` (ambos dentro de `Try`) y encola `andThen` como otra tarea, de modo que corra tras el siguiente `select` (necesario en JDK 11/Windows para liberar el canal a nivel de SO). (6) `shutdown`: cierra todos los canales de `selector.keys` y luego el selector. (7) `execute(task)` encola y, si `wakeUp.compareAndSet(false,true)`, llama a `selector.wakeup()`. (8) Todo el estado del selector se toca sólo desde un contexto serializado.

**Errores.** `readyOps` no reconocido: `log.warning("Invalid readyOps: [{}]", x)`. `CancelledKeyException` dentro del bucle o de una tarea se ignora (el canal se cerró desde otro hilo en `postStop` de la conexión). Cualquier otra `NonFatal` en una tarea: `log.error(e, "Error during selector management task: [{}]", e)` (el comentario FIXME :299 reconoce que no se avisa a nadie). `selector.isOpen == false` detiene la reprogramación del bucle (:195-198).

**Orden / concurrencia.** Un único hilo lógico por selector (dispatcher del selector, pinned-dispatcher por defecto, AC-IO-011); las tareas se ejecutan en orden de encolado. Orden de entrega observable para un canal listo para leer y escribir: `ChannelWritable` ANTES de `ChannelReadable`. El interés pedido por el actor se hace efectivo asíncronamente: `enableInterest` retorna antes de que la key lo tenga; por eso los actores de canal toleran un `ChannelReadable` en vuelo tras `disableInterest` (TcpConnectionSpec.scala:362-381).

**Dependencias.** AC-IO-004.1 (el `SelectionHandler` y el pool `selectors`), AC-DSP-011 (pinned-dispatcher), AC-IO-003 (`TraceLogging`, `SelectorDispatcher`), AC-IO-002.4/.5/.7 y AC-IO-006.x/AC-IO-007 (consumen la costura).

**Equivalente Rust propuesto.** `trait ChannelRegistry { fn register(&self, channel, initial_ops, channel_actor) }` + `ChannelRegistration { enable_interest, disable_interest, cancel_and_close }` como costura interna. Sobre ella, dos implementaciones: reactor sobre `mio::Poll` (un hilo por selector, con el mismo orden Writable-antes-de-Readable y limpieza de interés) o adaptador sobre tokio donde el "interés" se traduce a tareas de lectura/escritura (Q-IO-1 opción c). Se conserva: protocolo de mensajes interno, orden Writable->Readable, `cancelAndClose(andThen)` ejecutando `andThen` después de que el canal esté cerrado, y robustez ante `Readable` tardío. No se conserva: `OP_*` de NIO como API pública ni la semántica level-triggered de `select`.

**Aserciones.**
- A1. El registry NO se prueba por sí mismo: ningún spec toca `ChannelRegistryImpl`. Los specs de conexión lo SUSTITUYEN por `LocalServerTest extends ChannelRegistry` (TcpConnectionSpec.scala:883-947): `register` reenvía `Registration(channel, initialOps)` a `registerCallReceiver`; `ChannelRegistration` falso reenvía `op` a `interestCallReceiver` en `enableInterest` y `-op` en `disableInterest` (:925-930); `cancelAndClose(andThen)` por defecto ejecuta `andThen` de inmediato, y en `EstablishedConnectionTest` cierra el canal cliente, hace `selectNow()` y luego `Try(andThen())` (:1049-1056). Esto fija el CONTRATO que los actores esperan de la costura: registran con `initialOps=0` (:960 `Registration(clientSideChannel, 0)`), piden `OP_CONNECT` al conectar (:998), `OP_READ` tras `Register` salvo pull mode (:1005), y `-OP_READ` en `SuspendReading` (:368).
- A2. Los mensajes `ChannelConnectable/Readable/Writable/Acceptable` se inyectan a mano (`selector.send(connectionActor, ChannelReadable)`, TcpConnectionSpec.scala:116,151,166,353,372,503-603,643,678; `listener ! ChannelAcceptable`, TcpListenerSpec.scala:57,65,73); un `ChannelReadable` tardío tras `SuspendReading` se ignora aunque haya datos pendientes (TcpConnectionSpec.scala:362-381).
- A3. Con el selector real y `max-channels = 4`: tras listener + conexión servidor + conexión cliente (3 canales) un `Bind` adicional responde `Bound` (4.º canal), y el siguiente `Bind` y un `Connect` responden `CommandFailed` con el mismo comando (`theSameInstanceAs`) (el conteo es del `SelectionHandler`, AC-IO-004.2) — CapacityLimitSpec.scala:27-45. Es la única prueba de integración que atraviesa el registry real, sin afirmar nada sobre los mensajes de readiness.
- A4. Traducción `readyOps` -> mensajes, borrado de interés, `OP_READ|OP_WRITE` -> Writable antes que Readable — *no afirmada por spec*; derivada de SelectionHandler.scala:161-198. Caracterización propuesta: con la costura `ChannelRegistry` de Rust, test con canal loopback que fuerza ambos readiness y comprueba el orden de entrega y que el interés queda a 0 hasta el siguiente `enable_interest`.
- A5. `cancelAndClose(andThen)` ejecuta `andThen` después de cerrar el canal y de un `select` — *no afirmada por spec* (sólo comentada, SelectionHandler.scala:260-268). Caracterización propuesta: `andThen` observa el canal cerrado.
- A6. `readyOps` inválido -> warning "Invalid readyOps: [..]"; error de tarea -> log "Error during selector management task" — *no afirmada por spec*; textos derivados de SelectionHandler.scala:192,306 (sólo diagnóstico, sin contrato de formato más allá del texto).

**Validación prevista.** (a) Costura `ChannelRegistry` falsa en Rust que reproduce `LocalServerTest` para portar TcpConnectionSpec/TcpListenerSpec (AC-IO-002.x); es el uso principal. (b) Pruebas de caracterización propias del reactor (A4, A5) en loopback; (c) CapacityLimitSpec portado como integración (A3). Sin oráculo: es NIO interno, no observable por la API pública; el diferencial queda en TcpIntegrationSpec (AC-IO-002.x).

**Decisiones.** Q-IO-1 (modelo de ejecución) y Q-IO-2 (estrategia de port de specs). Efecto: si Q-IO-1 adopta la costura (c) este contrato se implementa tal cual como interfaz interna; si adopta tokio puro (b) A4/A5 se reinterpretan sobre tareas por conexión y la costura se reduce a un `trait` de pruebas. Puede avanzar la definición del trait y del test de contrato sin esperar al Owner.

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-005 — Opciones de socket: Inet.SocketOption(+V2), Inet.SO.*, Tcp.SO.*, Udp.SO.Broadcast, DatagramChannelCreator, forwarders Scala y fábricas Java

**Contrato observable.** SocketOption define 4 hooks vacíos (beforeDatagramBind, beforeServerSocketBind, beforeConnect, afterConnect); SocketOptionV2 añade afterBind (DatagramSocket, ServerSocket) y afterConnect(DatagramSocket). SO.ReceiveBufferSize(size>0): antes de bind de servidor/datagrama y antes de connect; SO.ReuseAddress(on): en los tres; SO.SendBufferSize(size>0): afterConnect; SO.TrafficClass(0..255): afterConnect. Tcp.SO.KeepAlive/OOBInline/TcpNoDelay: afterConnect. Udp.SO.Broadcast: beforeDatagramBind. DatagramChannelCreator.create() abre el DatagramChannel (la primera opción de ese tipo gana). Forwarders: val ReceiveBufferSize = SO.ReceiveBufferSize, etc.

**Fuente.** actor/src/main/scala/org/apache/pekko/io/Inet.scala:37-58 (SocketOption) ; actor/src/main/scala/org/apache/pekko/io/Inet.scala:66-83 (V2) ; actor/src/main/scala/org/apache/pekko/io/Inet.scala:93-108 (DatagramChannelCreator) ; actor/src/main/scala/org/apache/pekko/io/Inet.scala:110-172 (SO) ; actor/src/main/scala/org/apache/pekko/io/Inet.scala:174-214 (SoForwarders) ; actor/src/main/scala/org/apache/pekko/io/Inet.scala:216-257 (SoJavaFactories) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:67-105 (Tcp.SO) ; actor/src/main/scala/org/apache/pekko/io/Tcp.scala:674-702 (TcpSO) ; actor/src/main/scala/org/apache/pekko/io/Udp.scala:199-210 (Broadcast)

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

**Fuente.** actor/src/main/scala/org/apache/pekko/io/UdpListener.scala:45-76 (inicio y bind) ; actor/src/main/scala/org/apache/pekko/io/UdpListener.scala:78-82 (receive) ; actor/src/main/scala/org/apache/pekko/io/UdpListener.scala:84-93 (readHandlers) ; actor/src/main/scala/org/apache/pekko/io/UdpListener.scala:95-100 (unregistering) ; actor/src/main/scala/org/apache/pekko/io/UdpListener.scala:102-121 (doReceive) ; actor/src/main/scala/org/apache/pekko/io/UdpListener.scala:123-131 (postStop) ; actor/src/main/scala/org/apache/pekko/io/Udp.scala:112-124 (Bind/Unbind) ; actor/src/main/scala/org/apache/pekko/io/Udp.scala:145-151 (Suspend/Resume) ; actor/src/main/scala/org/apache/pekko/io/Udp.scala:162-192 (eventos)

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

**Fuente.** actor/src/main/scala/org/apache/pekko/io/WithUdpSend.scala:46-84 (sendHandlers) ; actor/src/main/scala/org/apache/pekko/io/WithUdpSend.scala:86-115 (doSend) ; actor/src/main/scala/org/apache/pekko/io/UdpSender.scala:43-81 ; actor/src/main/scala/org/apache/pekko/io/UdpSender.scala:83-89 (postStop) ; actor/src/main/scala/org/apache/pekko/io/Udp.scala:98-105 (Send) ; actor/src/main/scala/org/apache/pekko/io/Udp.scala:136-137 (SimpleSender) ; actor/src/main/scala/org/apache/pekko/io/Udp.scala:182-183 (SimpleSenderReady)

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

**Fuente.** actor/src/main/scala/org/apache/pekko/io/Udp.scala:212-229 (UdpSettings) ; actor/src/main/scala/org/apache/pekko/io/Udp.scala:232-254 (UdpExt) ; actor/src/main/resources/reference.conf:1080-1134

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

**Fuente.** actor/src/main/scala/org/apache/pekko/io/UdpConnection.scala:52-92 (resolución y doConnect) ; actor/src/main/scala/org/apache/pekko/io/UdpConnection.scala:94-129 (connected) ; actor/src/main/scala/org/apache/pekko/io/UdpConnection.scala:131-148 (doRead) ; actor/src/main/scala/org/apache/pekko/io/UdpConnection.scala:150-167 (doWrite) ; actor/src/main/scala/org/apache/pekko/io/UdpConnection.scala:169-191 (postStop/reportConnectFailure) ; actor/src/main/scala/org/apache/pekko/io/UdpConnected.scala:88-98 (Send) ; actor/src/main/scala/org/apache/pekko/io/UdpConnected.scala:105-111 (Connect) ; actor/src/main/scala/org/apache/pekko/io/UdpConnected.scala:118 (Disconnect) ; actor/src/main/scala/org/apache/pekko/io/UdpConnected.scala:143-168 (eventos) ; actor/src/main/scala/org/apache/pekko/io/UdpConnected.scala:172-191 (UdpConnectedExt)

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
- A6. Connect con host por nombre que SÍ resuelve falla por doConnect — derivada del código UdpConnection.scala:55-63,82-92; no afirmada por spec. Verificada contra el jar publicado 1.7.1 (`ORACLE=jars tools/oracle/run.sh` con un script jshell local no versionado, 2026-10-31): `UdpConnectedMessage.connect(handler, InetSocketAddress.createUnresolved("localhost", 40999))` responde `CommandFailed(Connect(.. localhost:40999 ..))` aunque `localhost` resuelve, mientras que `new InetSocketAddress("127.0.0.1", 40999)` (ya resuelta) responde `Connected` y un nombre inexistente (`doesnotexist.local`) también `CommandFailed`. Defecto latente confirmado de 1.7.1 (Q-IO-5)
- A7. El lote de lectura no se limita por receive-throughput — derivada del código UdpConnection.scala:132-141; no afirmada por spec

**Oráculo.** Jar oficial en loopback: el caso "Connect con nombre localhost" debe ejecutarse para confirmar el defecto (propuesto en Q-IO-5). Escenario no existe ni se ejecutó.

**Decisiones.** Q-IO-5, Q-IO-1

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-008.1 — Extensión Dns: DnsExt, Dns.resolve/cached (nueva y deprecada), selección de resolver, IpVersionSelector

**Contrato observable.** Dns.resolve(request, system, sender): ret = cached(request); si ret.isEmpty hace IO(Dns)(system).tell(request, sender); devuelve ret. Dns.cached/resolve(name) y Dns.Resolve/Resolved(name, ipv4, ipv6) son la API deprecada (cached por defecto None). DnsExt construye un DnsProvider por reflexión (provider-object del resolver configurado), expone cache = provider.cache y crea el manager "IO-DNS" con dispatcher pekko.io.dns.dispatcher. loadAsyncDns(managerName) crea bajo demanda un manager async-dns con nombre propio (uno por nombre, caché en ConcurrentHashMap). IpVersionSelector.getInetAddress(ipv4, ipv6): prefiere IPv6 solo si la propiedad JVM java.net.preferIPv6Addresses == "true"; si no, IPv4 primero.

**Fuente.** actor/src/main/scala/org/apache/pekko/io/Dns.scala:36-46 (Dns) ; actor/src/main/scala/org/apache/pekko/io/Dns.scala:48-78 (object Dns) ; actor/src/main/scala/org/apache/pekko/io/Dns.scala:80-167 (DnsExt (loadAsyncDns 165-202) ; actor/src/main/scala/org/apache/pekko/io/Dns.scala:Settings 215-228 ; actor/src/main/scala/org/apache/pekko/io/Dns.scala:provider 235-236 ; actor/src/main/scala/org/apache/pekko/io/Dns.scala:cache 239 ; actor/src/main/scala/org/apache/pekko/io/Dns.scala:manager 242-246) ; actor/src/main/scala/org/apache/pekko/io/Dns.scala:173-183 (IpVersionSelector) ; actor/src/main/scala/org/apache/pekko/io/DnsProvider.scala:26-46 ; actor/src/main/resources/reference.conf:1192-1222

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

**Fuente.** actor/src/main/scala/org/apache/pekko/io/dns/DnsProtocol.scala:39-129 ; actor/src/main/scala/org/apache/pekko/io/dns/DnsResourceRecords.scala:34-147 (tipos y parseBody) ; actor/src/main/scala/org/apache/pekko/io/dns/DnsResourceRecords.scala:159-180 (parse) ; actor/src/main/scala/org/apache/pekko/io/dns/CachePolicy.scala:22-77 ; actor/src/main/scala/org/apache/pekko/io/dns/RecordType.scala:22-107 ; actor/src/main/scala/org/apache/pekko/io/dns/RecordClass.scala:20-27

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

**Fuente.** actor/src/main/scala/org/apache/pekko/io/InetAddressDnsResolver.scala:46-56 (propiedades) ; actor/src/main/scala/org/apache/pekko/io/InetAddressDnsResolver.scala:58-92 (políticas por defecto) ; actor/src/main/scala/org/apache/pekko/io/InetAddressDnsResolver.scala:94-108 (getTtl y políticas) ; actor/src/main/scala/org/apache/pekko/io/InetAddressDnsResolver.scala:108 (toLongTtl) ; actor/src/main/scala/org/apache/pekko/io/InetAddressDnsResolver.scala:110-136 (Resolve Ip/Srv) ; actor/src/main/scala/org/apache/pekko/io/InetAddressDnsResolver.scala:136 (Dns.Resolve) ; actor/src/main/scala/org/apache/pekko/io/InetAddressDnsResolver.scala:139-148 (addressToRecords) ; actor/src/main/resources/reference.conf:1204-1222

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

**Fuente.** actor/src/main/scala/org/apache/pekko/io/SimpleDnsCache.scala:35-46 (clase y cached(name)) ; actor/src/main/scala/org/apache/pekko/io/SimpleDnsCache.scala:47-48 (cached(Resolve)) ; actor/src/main/scala/org/apache/pekko/io/SimpleDnsCache.scala:50-55 (clock) ; actor/src/main/scala/org/apache/pekko/io/SimpleDnsCache.scala:61-70 (get/put) ; actor/src/main/scala/org/apache/pekko/io/SimpleDnsCache.scala:72-77 (cleanup) ; actor/src/main/scala/org/apache/pekko/io/SimpleDnsCache.scala:86-120 (Cache) ; actor/src/main/scala/org/apache/pekko/io/SimpleDnsCache.scala:122-140 (entradas) ; actor/src/main/scala/org/apache/pekko/io/SimpleDnsManager.scala:39-49 (limpieza periódica)

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

**Fuente.** actor/src/main/scala/org/apache/pekko/io/SimpleDnsManager.scala:25-68 ; actor/src/main/scala/org/apache/pekko/io/InetAddressDnsProvider.scala:26-30 ; actor/src/main/scala/org/apache/pekko/io/dns/internal/AsyncDnsProvider.scala:25-31 ; actor/src/main/scala/org/apache/pekko/io/DnsProvider.scala:26-46

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

**Fuente.** actor/src/main/scala/org/apache/pekko/io/dns/DnsSettings.scala:61-79 (nameservers) ; actor/src/main/scala/org/apache/pekko/io/dns/DnsSettings.scala:81-84 (timeout/ttl) ; actor/src/main/scala/org/apache/pekko/io/dns/DnsSettings.scala:86-96 (id policy) ; actor/src/main/scala/org/apache/pekko/io/dns/DnsSettings.scala:98-107 (getTtl) ; actor/src/main/scala/org/apache/pekko/io/dns/DnsSettings.scala:109-126 (resolvConf) ; actor/src/main/scala/org/apache/pekko/io/dns/DnsSettings.scala:128-139 (search-domains) ; actor/src/main/scala/org/apache/pekko/io/dns/DnsSettings.scala:141-158 (ndots) ; actor/src/main/scala/org/apache/pekko/io/dns/DnsSettings.scala:162-166 (error) ; actor/src/main/scala/org/apache/pekko/io/dns/DnsSettings.scala:172-187 (parseNameserverAddress) ; actor/src/main/scala/org/apache/pekko/io/dns/DnsSettings.scala:199-270 (nameservers por defecto) ; actor/src/main/resources/reference.conf:1224-1283

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

**Fuente.** actor/src/main/scala/org/apache/pekko/io/dns/internal/AsyncDnsResolver.scala:57-78 (localhost) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/AsyncDnsResolver.scala:79-90 ; actor/src/main/scala/org/apache/pekko/io/dns/internal/AsyncDnsResolver.scala:98-111 (receive y caché) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/AsyncDnsResolver.scala:111-133 (resolveWithResolvers) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/AsyncDnsResolver.scala:135-147 (sendQuestion) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/AsyncDnsResolver.scala:147 (search) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/AsyncDnsResolver.scala:147-150 (resolveFirst) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/AsyncDnsResolver.scala:150-151 (resolve) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/AsyncDnsResolver.scala:161-175 (detección de literales IP) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/AsyncDnsResolver.scala:183 (ResolveFailedException)

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

**Fuente.** actor/src/main/scala/org/apache/pekko/io/dns/internal/DnsClient.scala:36-54 (mensajes) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/DnsClient.scala:61-77 (init) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/DnsClient.scala:79-90 (stash) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/DnsClient.scala:100-211 (ready) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/DnsClient.scala:213-221 (newInflightRequests) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/DnsClient.scala:223-234 (isSameQuestion) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/DnsClient.scala:251-261 (tcp client) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/Question.scala:25-51 (Question.isSame)

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

**Fuente.** actor/src/main/scala/org/apache/pekko/io/dns/internal/TcpDnsClient.scala:37-43 (idle) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/TcpDnsClient.scala:45-57 (connecting) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/TcpDnsClient.scala:59-88 (ready) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/TcpDnsClient.scala:90-99 (parseResponse) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/TcpDnsClient.scala:107-118 (encode/decode length/throwFailure)

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

**Fuente.** actor/src/main/scala/org/apache/pekko/io/dns/internal/DnsMessage.scala:28-45 (OpCode/ResponseCode) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/DnsMessage.scala:51-105 (MessageFlags) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/DnsMessage.scala:111-136 (Message.write) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/DnsMessage.scala:142-177 (Message.parse (TC 170-177)) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/DomainName.scala:32-33 (límites) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/DomainName.scala:39-47 (write) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/DomainName.scala:49-82 (parse) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/Question.scala:25-51 ; actor/src/main/scala/org/apache/pekko/io/dns/DnsResourceRecords.scala:152-180 (parse) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/package.scala:networkByteOrder

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

**Fuente.** actor/src/main/scala/org/apache/pekko/io/dns/internal/ResolvConfParser.scala:22-28 (etiquetas) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/ResolvConfParser.scala:34-41 (parseFile) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/ResolvConfParser.scala:43-77 (parseLines)

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

**Fuente.** actor/src/main/scala/org/apache/pekko/io/dns/IdGenerator.scala:55-61 (apply) ; actor/src/main/scala/org/apache/pekko/io/dns/IdGenerator.scala:66-67 (random) ; actor/src/main/scala/org/apache/pekko/io/dns/IdGenerator.scala:69-93 (EnhancedDoubleHashGenerator)

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

**Fuente.** actor/src/main/scala/org/apache/pekko/io/dns/internal/AsyncDnsManager.scala:49-76 (init) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/AsyncDnsManager.scala:78-85 (resolver) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/AsyncDnsManager.scala:87-98 (limpieza) ; actor/src/main/scala/org/apache/pekko/io/dns/internal/AsyncDnsManager.scala:100-111 (receive)

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

**Fuente.** actor/src/main/resources/reference.conf:350-363 deployments ; actor/src/main/resources/reference.conf:982-990 pinned-dispatcher

**Condiciones.** Valores por defecto; el nombre de ruta determina el deployment para los hijos creados con FromConfig.

**Orden / concurrencia / ciclo de vida.** El router consistent-hashing usa ConsistentHashable.consistentHashKey (= name) de Resolve.

**Dependencias.** AC-DSP-011, AC-RT, AC-IO-008.5

**Equivalente Rust propuesto.** Entradas de config embebidas; `PinnedDispatcher` solo si Q-IO-1 conserva selectores dedicados.

**Diferencias observables.** Con tokio los dispatchers no se usan: se acepta la config sin efecto (Q-IO-1).

**Aserciones.**
- A1. Valores de deployments y pinned-dispatcher — derivada del archivo actor/src/main/resources/reference.conf:350-363,982-990; no afirmada por spec

**Oráculo.** Jar oficial: leer la config efectiva. No ejecutado.

**Decisiones.** Q-IO-1

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-IO-012 — UnsynchronizedByteArrayInputStream (java io): InputStream sobre byte[] sin sincronización, mark/reset/skip

**Contrato observable.** Constructores (data) y (data, offset, length). available = max(eod-offset, 0). mark(readLimit) guarda el offset ignorando el límite; reset vuelve a ese offset; markSupported = true. read() devuelve el byte sin signo o -1 al final. read(dest) y read(dest, off, len) copian min(len, disponible) bytes, devuelven 0 si len==0 y -1 si no queda nada. skip(n): n<0 -> IllegalArgumentException; devuelve min(n, disponible) pero fija offset = offset + n (puede quedar más allá del fin; el siguiente read da -1). close hereda el no-op de InputStream.

**Fuente.** actor/src/main/java/org/apache/pekko/io/UnsynchronizedByteArrayInputStream.java:49-54 (requireNonNegative) ; actor/src/main/java/org/apache/pekko/io/UnsynchronizedByteArrayInputStream.java:55 (checkFromIndexSize) ; actor/src/main/java/org/apache/pekko/io/UnsynchronizedByteArrayInputStream.java:78-102 (constructores) ; actor/src/main/java/org/apache/pekko/io/UnsynchronizedByteArrayInputStream.java:104-107 (available) ; actor/src/main/java/org/apache/pekko/io/UnsynchronizedByteArrayInputStream.java:109-118 (mark/markSupported) ; actor/src/main/java/org/apache/pekko/io/UnsynchronizedByteArrayInputStream.java:120-148 (read y readLocal) ; actor/src/main/java/org/apache/pekko/io/UnsynchronizedByteArrayInputStream.java:175-179 (reset) ; actor/src/main/java/org/apache/pekko/io/UnsynchronizedByteArrayInputStream.java:181-211 (skip)

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

**Fuente.** actor/src/main/java/org/apache/pekko/io/ByteBufferCleaner.java:44-46 (interfaz) ; actor/src/main/java/org/apache/pekko/io/ByteBufferCleaner.java:47 (Java8Cleaner) ; actor/src/main/java/org/apache/pekko/io/ByteBufferCleaner.java:48-69 (Java9Cleaner) ; actor/src/main/java/org/apache/pekko/io/ByteBufferCleaner.java:71 (INSTANCE) ; actor/src/main/java/org/apache/pekko/io/ByteBufferCleaner.java:79-85 (clean) ; actor/src/main/java/org/apache/pekko/io/ByteBufferCleaner.java:87-111 (getCleaner) ; actor/src/main/java/org/apache/pekko/io/ByteBufferCleaner.java:120-122 (isSupported)

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

### AC-JAPI-001 — AbstractActor + createReceive y AbstractActor.ActorContext (getters Java, become(Receive), receive timeout con java.time.Duration)

**Contrato observable.** Clase base Java del actor clásico. `createReceive()` (abstracto) devuelve un `AbstractActor.Receive` (envoltorio de `PartialFunction[Any, BoxedUnit]`); `AbstractActor.receive` es `createReceive().onMessage` y es lo que el runtime usa como comportamiento inicial. `Receive.orElse(other)` compone las funciones; `AbstractActor.emptyBehavior` no coincide con ningún mensaje. `getContext()`/`getSelf()`/`getSender()` exponen `context`/`self`/`sender()`. `AbstractActor.ActorContext` añade a `ActorContext`: `getSelf`, `getProps`, `getSender`, `getChildren` (Iterable), `findChild(name): Optional`, `getParent`, `getSystem`, `getDispatcher`, `become(Receive[, discardOld])`, `getReceiveTimeout`/`setReceiveTimeout(java.time.Duration)`/`cancelReceiveTimeout`. La construcción de comportamientos con `receiveBuilder()` se contrata aparte en AC-JAPI-012.

**Fuente.** actor/src/main/scala/org/apache/pekko/actor/AbstractActor.scala:44-57 (Receive, emptyBehavior) ; actor/src/main/scala/org/apache/pekko/actor/AbstractActor.scala:66-206 (ActorContext Java) ; actor/src/main/scala/org/apache/pekko/actor/AbstractActor.scala:235-327 (AbstractActor: getContext 236, getSelf 243, getSender 255, supervisorStrategy 261, preStart 271, postStop 280, preRestart 286 y 298, postRestart 309, createReceive 315, receive 317, receiveBuilder 324) ; actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:423-463 (ActorCell implementa AbstractActor.ActorContext y los getters Java) ; actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:621-628,651 (become, Procedure y receive inicial) ; actor/src/main/scala/org/apache/pekko/actor/dungeon/Children.scala:46-57 (getChildren, findChild) ; actor/src/main/scala/org/apache/pekko/actor/dungeon/ReceiveTimeout.scala:25-39 (estado por defecto Undefined) ; actor/src/main/scala-2/org/apache/pekko/util/JavaDurationConverters.scala (asScala = Duration.fromNanos(toNanos); asJava = ofNanos(toNanos)).

**Precondiciones.** El actor se crea por `Props` con una clase concreta (AC-JAPI-006). `createReceive()` se invoca UNA vez al instanciar (ActorCell.scala:632 usa `instance.receive` sólo si la pila de comportamientos está vacía) y debe devolver un `Receive` no nulo.

**Comportamiento.** (1) Clasificación de fachada: `AbstractActor extends Actor`; `getContext()` es `context.asInstanceOf[AbstractActor.ActorContext]` y la celda real (`ActorCell`) YA implementa `AbstractActor.ActorContext` (ActorCell.scala:426), de modo que `getSelf/getParent/getSystem/getDispatcher/getProps/getChildren/findChild` son alias de `self/parent/system/dispatcher/props/children/getChild` de la MISMA celda; `getSender()` = `sender()`; `become(Receive)` = `become(behavior.onMessage, discardOld = true)`; `setReceiveTimeout(java.time.Duration)` = `setReceiveTimeout(timeout.asScala)`; `cancelReceiveTimeout()` = `setReceiveTimeout(Duration.Undefined)`. No hay lógica propia de ejecución: todo delega en la implementación Scala (AC-CELL, AC-FSM...). (2) Comportamiento diferencial propio (sólo conversiones y valores por defecto): (a) `become(Receive)` descarta el comportamiento anterior por defecto (`discardOld=true`), igual que la forma Scala; (b) `findChild` devuelve `Optional.ofNullable(getChild(name))` (vacío si no existe); (c) `getChildren()` es una vista Java de la colección inmutable; (d) las conversiones de `java.time.Duration` pasan por nanosegundos (`toNanos`): duraciones que no caben en `Long` nanos (~292 años) desbordan con `ArithmeticException` de Java; (e) `getReceiveTimeout()` convierte con `receiveTimeout.asJava`; mientras no se haya fijado un timeout el valor es `Duration.Undefined` (ReceiveTimeout.scala:25) y `Undefined.toNanos` lanza `IllegalArgumentException` en scala.concurrent.duration (derivado del tipo; ver A8); (f) `preRestart(reason, Optional)` es el gancho que se sobrescribe en Java: la versión Scala `preRestart(reason, Option)` está deprecada y delega en la versión `Optional` (`message.toJava`), que a su vez llama a `super.preRestart(reason, message.toScala)` (AbstractActor.scala:286-300); (g) los demás ganchos (`preStart`, `postStop`, `postRestart`, `supervisorStrategy`) son re-declaraciones con `@throws(Exception)` sólo para que Java pueda sobrescribirlos.

**Errores.** `createReceive` no definido -> error de compilación en Java (no existe en Rust). `UntypedAbstractActor.createReceive` lanza `UnsupportedOperationException("createReceive should not be used by UntypedAbstractActor")` (AC-JAPI-002). Un mensaje que ninguna función del `Receive` acepta pasa a `unhandled` (UnhandledMessage al EventStream, AC-CELL), no es un error del builder. Excepciones de `createReceive` durante la creación -> `ActorInitializationException` (AC-CELL).

**Orden / concurrencia.** `getContext()` y el contexto no son seguros entre hilos (documentado, AbstractActor.scala:230-235): sólo dentro del actor. `become` entra en vigor para el siguiente mensaje.

**Dependencias.** AC-CELL-001 (celda y comportamiento), AC-CELL (become/unbecome, receive-timeout), AC-JAPI-012 (ReceiveBuilder), AC-JAPI-004.1/.3, AC-UTIL-018.2 (JavaDurationConverters).

**Equivalente Rust propuesto.** El trait `Actor` de Rust con su `receive`/`Context` ya expone self, sender, parent, system, children, become y receive-timeout con `std::time::Duration`; no hace falta una clase base ni `Receive` nuevo. Se conserva todo el efecto observable de las filas CELL; lo que se pierde es sólo el nombre `getX()`, `Optional` (-> `Option`) y la vista `Iterable` (-> iterador). Diferencias a documentar: `getReceiveTimeout()` sin timeout (A8) en Rust devolverá `Option::None` en lugar de lanzar. Q-JAPI-1 decide si se ofrecen los alias.

**Aserciones.**
- A1. Un actor Java cuya `createReceive` devuelve `receiveBuilder().build()` vacío se crea y su clase queda en `Props.actorClass()` — ActorCreationTest.java:83-106 y 321-325 (afirma `p.actorClass()`, no comportamiento de mensajes).
- A2. `UntypedAbstractActor.onReceive` recibe los mensajes y `getSender()/getSelf()` responden al remitente: el actor responde "a-b-17-18" a "get" — JavaAPI.java:98-106 y 163-209 (`getSender().tell(reply, getSelf())`).
- A3. `createReceive` con `receiveBuilder().match(Object.class, ...)` procesa mensajes, `getSender()` y `getSelf()` — StashJavaAPITestActors.java:43-56 (con stash; responde `5` a "Hello", StashJavaAPI.java:31-39).
- A4. `receive` inicial = `createReceive().onMessage`; `become(Receive)` reemplaza descartando el anterior — *no afirmada por spec Java*. Respaldo en Scala del mecanismo subyacente: ActorCell become/unbecome (spec de AC-CELL). Caracterización propuesta: actor que hace `become` y comprueba que el comportamiento viejo ya no responde.
- A5. `findChild(name)` vacío para hijo inexistente / presente para existente — *no afirmada por spec* (Children.scala:56). Caracterización propuesta.
- A6. `Receive.orElse` compone en orden; `emptyBehavior` no cubre nada — *no afirmada por spec*. Caracterización propuesta con `isDefinedAt`.
- A7. `setReceiveTimeout(Duration.ofMillis(n))`/`cancelReceiveTimeout()` equivalen a las formas Scala — *no afirmada por spec Java*; la semántica del timeout la afirman los specs Scala de ReceiveTimeout (AC-CELL). Caracterización propuesta.
- A8. `getReceiveTimeout()` sin timeout fijado lanza `IllegalArgumentException` — *no afirmada por spec*; derivada del código AbstractActor.scala:168-171 + ReceiveTimeout.scala:25. Verificada contra el jar publicado 1.7.1 (`ORACLE=jars tools/oracle/run.sh` con un script jshell local no versionado, 2026-10-31; es el cross-check de jars, no el build desde fuente): un actor `AbstractActor` que llama `getContext().getReceiveTimeout()` obtiene `java.lang.IllegalArgumentException: toNanos not allowed on infinite Durations`. Es un defecto/artefacto de la fachada Java; en Rust se propone `Option::None` (divergencia a registrar si se porta).
- A9. `preRestart(reason, Optional)` recibe el mensaje que causó el fallo como `Optional` (vacío si no hubo) — *no afirmada por spec Java*; respaldo Scala en los specs de reinicio (AC-SUP). Caracterización propuesta.

**Validación prevista.** (1) Para la fachada: la semántica la cubren las filas CELL/SUP/TMR con sus specs Scala portados; no se duplican tests por cada alias. (2) Diferencial con oráculo sólo para A8 (único comportamiento no derivable con seguridad). (3) Si Q-JAPI-1 acepta la exclusión, esta fila se cierra con la justificación de fachada (ActorCell implementa la interfaz) y sin tests propios.

**Decisiones.** Q-JAPI-1 (portar o no la fachada Java). Efecto: sin fachada, ninguna aserción A4-A9 se implementa como API separada (se prueban en CELL); con fachada, se implementan como alias sin lógica propia más A8. No bloquea CELL.

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-JAPI-002 — UntypedAbstractActor, AbstractLoggingActor, AbstractActorWithStash/UnboundedStash/UnrestrictedStash, AbstractActorWithTimers, UntypedAbstractActorWithTimers

**Contrato observable.** Variantes de `AbstractActor` que mezclan un trait Scala: `UntypedAbstractActor` (estilo `onReceive(Object)`, `createReceive` prohibido), `AbstractLoggingActor` (+ `ActorLogging`), `AbstractActorWithStash` (+ `Stash`, cola de doble extremo con mailbox acotado por defecto), `AbstractActorWithUnboundedStash` (+ `UnboundedStash`), `AbstractActorWithUnrestrictedStash` (+ `UnrestrictedStash`, sin mailbox impuesto), `AbstractActorWithTimers` y `UntypedAbstractActorWithTimers` (+ `Timers`, `getTimers()`).

**Fuente.** actor/src/main/scala/org/apache/pekko/actor/AbstractActor.scala:339-360 (UntypedAbstractActor), 365 (AbstractLoggingActor), 410 (AbstractActorWithStash), 419 (UnboundedStash), 427 (UnrestrictedStash) ; actor/src/main/scala/org/apache/pekko/actor/Timers.scala:77-82 (AbstractActorWithTimers, `getTimers`), 94-99 (UntypedAbstractActorWithTimers, @since 1.1.0) ; actor/src/main/scala/org/apache/pekko/actor/Stash.scala:70-75,81 (Stash, UnboundedStash, UnrestrictedStash y sus RequiresMessageQueue).

**Precondiciones.** Para `*WithStash`/`UnboundedStash` el mailbox debe ser deque: lo exige el marker `RequiresMessageQueue[DequeBasedMessageQueueSemantics]` (el sistema selecciona `unbounded-deque-based` o el acotado según el trait; `UnrestrictedStash` exige que el usuario configure un mailbox compatible, p. ej. `.withMailbox("pekko.actor.mailbox.unbounded-deque-based")`).

**Comportamiento.** Todas son composiciones de `AbstractActor` con un trait existente; no añaden lógica. `UntypedAbstractActor`: `receive = { case msg => onReceive(msg) }` (comodín: todo mensaje va a `onReceive`), `createReceive` final y lanza `UnsupportedOperationException("createReceive should not be used by UntypedAbstractActor")`, `unhandled` re-declarado. `AbstractActorWithTimers.getTimers` = `timers` (el `TimerScheduler` del trait `Timers`). Stash: `stash()`, `unstashAll()` se heredan del trait; `postStop`/`preRestart` del trait devuelven los mensajes apilados al mailbox (AC-MBX/AC-STASH).

**Errores.** `createReceive()` en `UntypedAbstractActor` -> `UnsupportedOperationException` (el runtime nunca lo llama porque `receive` está sobrescrito). Mailbox no compatible con deque en `WithStash` -> error de creación del actor (AC-MBX-RequiresMessageQueue). Desborde del stash acotado -> `StashOverflowException` (AC-STASH).

**Orden / concurrencia.** Igual que Scala: stash y timers sólo se usan dentro del actor; los timers se cancelan al reiniciar/parar.

**Dependencias.** AC-JAPI-001, AC-MBX (Stash, deque), AC-TIM (Timers), AC-LOG (ActorLogging).

**Equivalente Rust propuesto.** Los traits `Stash`, `Timers` y el logging por contexto de Rust (AC-TIM/AC-STASH/AC-LOG). No hay herencia: la composición es opcional por capacidad. Efecto observable que se preserva: lo especificado por las filas Scala. No se necesita `UntypedAbstractActor` (en Rust un único `receive`).

**Aserciones.**
- A1. Las tres variantes de stash funcionan de forma equivalente: tras 2 mensajes apilados con `stash()` y un tercero que dispara `unstashAll()`, el actor responde `5` (longitud de "Hello") al probe — StashJavaAPI.java:31-39 (helper), 40-55 (tres tests: WithStash, WithUnboundedStash, WithUnrestrictedStash con `withMailbox("pekko.actor.mailbox.unbounded-deque-based")`), con los actores de StashJavaAPITestActors.java:24-87. El assert (`expectMsg(5)`) cubre sólo la primera respuesta; no verifica el orden ni el contenido tras el segundo mensaje.
- A2. `UntypedAbstractActor`: `onReceive` recibe y `getSender().tell(.., getSelf())` responde — JavaAPI.java:98-106,163-209.
- A3. `AbstractFSMWithStash` y las clases del stash compilan desde Java sin "inherits unrelated defaults" — AbstractFSMWithStashActorTest.java:27-81 (ver AC-JAPI-003; es un contrato de compilación JVM/Scala 3, sin equivalente en Rust).
- A4. `AbstractActorWithTimers.getTimers()` devuelve el mismo `TimerScheduler` que `timers` y los timers se cancelan al reiniciar — *no afirmada por spec Java*; la semántica de los timers la afirman los specs Scala (AC-TIM; Timers.scala:78-99 sólo delega). Caracterización: no necesaria en JAPI.
- A5. `AbstractLoggingActor.log` — *no afirmada por spec Java*; es `ActorLogging` (AC-LOG).
- A6. `createReceive` de `UntypedAbstractActor` lanza `UnsupportedOperationException` con ese texto — *no afirmada por spec*; derivada de AbstractActor.scala:338-339. Sin equivalente (no hay `createReceive` en Rust).

**Validación prevista.** Por fachada: cubierta por las filas AC-STASH, AC-TIM, AC-LOG. No se duplica test salvo, si Q-JAPI-1 acepta la fachada, A1 portado como prueba de integración de la clase base (un actor con stash apilando y reentregando). Justificación: la única lógica diferencial es la selección del mailbox por trait (verificable en MBX).

**Decisiones.** Q-JAPI-1. Efecto: sin fachada, la fila se cierra por equivalencia con STASH/TIM/LOG; con fachada, se añaden alias de tipo sin lógica. No bloquea.

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-JAPI-003 — AbstractFSM / AbstractLoggingFSM / AbstractFSMWithStash y builders FSM (FSMStateFunctionBuilder, FSMTransitionHandlerBuilder, FSMStopBuilder)

**Contrato observable.** Versión Java del FSM. `AbstractFSM[S, D] extends FSM[S, D]`: `when(state, builder)`, `when(state, timeout, builder)` (con `FiniteDuration` o `java.time.Duration`), `startWith(state, data[, timeout])`, `onTransition(builder | UnitApply2)`, `whenUnhandled(builder)`, `onTermination(builder)`, constructores de builder `matchEvent` (7 formas), `matchEventEquals` (2), `matchAnyEvent`, `matchState` (2), `matchStop` (3), `matchData` (2), `goTo`, y temporizadores `startTimerWithFixedDelay/AtFixedRate/startSingleTimer(name, msg, java.time.Duration)` y `setTimer` (deprecados). `AbstractLoggingFSM` añade `LoggingFSM`; `AbstractFSMWithStash` añade `Stash`.

**Fuente.** actor/src/main/scala/org/apache/pekko/actor/AbstractFSM.scala:26-547 (when 86-130, startWith 140-166, onTransition 175-188, whenUnhandled 196, onTermination 203, matchEvent 217-309, matchEventEquals 322-338, matchAnyEvent 348, matchState 361-375, matchStop 386-415, matchData 424-439, goTo 448, timers 466-549, Normal/Shutdown 554-560, LoggingFSM 568, WithStash 575-585) ; actor/src/main/java/org/apache/pekko/japi/pf/FSMStateFunctionBuilder.java:28-274 ; actor/src/main/java/org/apache/pekko/japi/pf/FSMStopBuilder.java:31-133 ; actor/src/main/java/org/apache/pekko/japi/pf/FSMTransitionHandlerBuilder.java:28-102 ; actor/src/main/java/org/apache/pekko/japi/pf/PFBuilder.java:25-144 (base de los builders).

**Precondiciones.** Se llama desde el constructor de la subclase antes de `initialize()`, como el FSM Scala (AC-FSM). Argumentos de builder no nulos salvo los comodines indicados.

**Comportamiento.** (1) Fachada: cada método de `AbstractFSM` delega en el FSM Scala (`super.when`, `super.startWith`, `super.onTransition`, `super.whenUnhandled`, `super.onTermination`, `startTimer*`/`setTimer` con `.asScala`) y los builders producen `PartialFunction` que se pasan a esos métodos; la máquina de estados, los timeouts de estado, `StateTimeout`, `stay`, `stop`, `using`, `replying`, notificaciones y el orden de los handlers de transición son los de AC-FSM. (2) Comportamiento propio de los builders (el único diferencial): `FSMStateFunctionBuilder.erasedEvent` evalúa, por este orden, evento (clase -> `isInstance`; valor -> `equals`; null -> cualquiera), dato de estado (clase -> `isInstance`; valor -> `equals`; null -> cualquiera) y por último el predicado (FSMStateFunctionBuilder.java:57-80); la variante con lista `eventMatches` comprueba primero el tipo de dato y luego, por cada elemento de la lista, `isInstance` si es `Class` o `event.equals(em)` si no, cortando en el primero que coincide (:188-203). `FSMStopBuilder.stop(reason, ..)` compara con `reason.equals(e.reason())`; `stop(reasonType, ..)` usa `isInstance` y predicado opcional. `FSMTransitionHandlerBuilder.state(from, to, ..)` trata `null` como comodín en `from` y `to` y compara con `equals`; DEVUELVE un PF de pares (from,to) y, como en Scala, TODOS los handlers instalados con `onTransition` se ejecutan. (3) `build()` de los tres builders delega en `PFBuilder.build`/`UnitPFBuilder.build`, que SÍ resetea el estado (a diferencia de `ReceiveBuilder.build`, AC-JAPI-012). (4) `matchEvent(eventType, dataType, ..)` con ambos tipos exige que el dato sea de ese tipo; sin predicado ni tipo de dato el caso aplica a todo dato. (5) `when(state, builder)` sin timeout pasa `null: FiniteDuration` (sin timeout de estado por defecto).

**Errores.** Evento no cubierto por la función de estado ni por `whenUnhandled` -> warning "unhandled event .. in state .." y `stay` (AC-FSM). `eventMatches` con elemento null o `event` null -> `NullPointerException` en la evaluación (`event.equals(em)`). Casts de `stateData` a `Q` no verificados: tipo de dato incorrecto -> `ClassCastException` en el cuerpo del usuario. `startWith` sin `initialize()` -> el FSM no arranca (AC-FSM).

**Orden / concurrencia.** El orden de `when(...)` repetidos para un mismo estado es el de AC-FSM (se encadenan con `orElse`: el primero que cubre gana); `whenUnhandled`/`onTermination` sobrescriben (la última llamada gana: "Calling this method again will overwrite", AbstractFSM.scala:191-203). Los handlers de transición se acumulan.

**Dependencias.** AC-FSM-001 (y las filas FSM de estado/timers/stash), AC-JAPI-004.1/.2/.3, AC-JAPI-002 (Stash), AC-TIM, AC-UTIL-018.2.

**Equivalente Rust propuesto.** Una sola API de FSM en Rust (AC-FSM): función de estado como `match` sobre `(evento, dato)`; `matchEvent*`/`matchState`/`matchStop` se expresan con patrones nativos y no necesitan builder. Se conservan todos los efectos observables del FSM; se pierden `isInstance` por clase (TypeId exacto), `null` como comodín (-> `_`) y los nombres. Los builders sólo son viables como utilidad opcional (Q-JAPI-1/Q-JAPI-2).

**Aserciones.**
- A1. FSM Java con `onTransition(this::logTransition)`, `startWith("start","data")`, `when("start", matchEventEquals("next", (s,d) -> goTo(s)))`, `when("next", AbstractFSM.NullFunction())`, `initialize()`: al arrancar emite "Transitioning from start to start." y al recibir "next" emite "Transitioning from start to next." — AbstractFSMActorTest.java:27-62 (test canCreateFSM, :52-62). Cubre `onTransition(UnitApply2)`, `matchEventEquals(event, apply)`, `goTo`, `NullFunction`.
- A2. `AbstractFSMWithStash` con `matchEvent(String.class, (event, data) -> ..)`, `stash()`, `unstashAll()` y `goTo("next")`: "work" se apila en `start`, "go" desapila y transita, y en `next` el probe recibe "work" — AbstractFSMWithStashActorTest.java:34-81 (test :73-81). El propósito declarado es de COMPILACIÓN con Scala 3 (cabecera :27-33: javac rechaza subclases si la clase base no declara un override real de `postStop`); sin equivalente en Rust.
- A3. Orden de evaluación evento -> dato -> predicado; lista `eventMatches` con clases e instancias; `null` comodín — *no afirmada por spec*; derivada de FSMStateFunctionBuilder.java:57-80,188-203. Caracterización propuesta en el FSM de Rust (patrones con guarda).
- A4. `matchStop(reason, ..)` por igualdad y `matchStop(Class, ..)` por tipo con predicado; `matchState(from, to, ..)` con `null` comodín — *no afirmada por spec*; derivada de FSMStopBuilder.java:40-124 y FSMTransitionHandlerBuilder.java:36-91.
- A5. `startTimer*`/`setTimer` con `java.time.Duration` convierten con `asScala` y delegan — *no afirmada por spec Java*; semántica de timers en AC-FSM (FSMTimersSpec).
- A6. `whenUnhandled`/`onTermination` sobrescriben; `onTransition` acumula — *no afirmada por spec Java*; semántica documentada en AbstractFSM.scala:168-204 y afirmada para la forma Scala por FSMTransitionSpec/FSMActorSpec (AC-FSM).

**Validación prevista.** Por fachada, la semántica FSM la validan los specs Scala portados (AC-FSM). Sólo si Q-JAPI-1 acepta la fachada: A1 portado como prueba de integración y A3/A4 como caracterización de los builders. A2 se descarta como contrato JVM. Sin oráculo propio (el oráculo FSM de AC-FSM cubre el núcleo).

**Decisiones.** Q-JAPI-1 (fachada), Q-JAPI-2 (isInstance en builders). Efecto: sin fachada, la fila se cierra por equivalencia con AC-FSM y A3/A4 se cierran como "no aplicable (patrones nativos)". No bloquea AC-FSM.

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-JAPI-004.1 — japi.pf Match/PFBuilder/AbstractMatch/AbstractPFBuilder/CaseStatement: PartialFunction construible por casos (tipo, tipo+predicado, igualdad, cualquiera)

**Contrato observable.** PFBuilder<I,R>.match(Class<P>, Apply<P,R>) añade CaseStatement con predicado type.isInstance(o); match(Class, TypedPredicate, Apply) exige además el predicado; matchEquals(obj, apply) usa obj.equals(o); matchAny(apply) siempre; matchUnchecked evita el chequeo de genéricos. addStatement encadena con orElse. build() devuelve statements.orElse(empty) y resetea statements=null; sin casos devuelve la PF vacía. Match.create(builder) congela la PF; match(i) aplica y lanza MatchError sin coincidencia; asPF() devuelve la PF. Match.match(...) estático crea un PFBuilder con el primer caso.

**Fuente.** actor/src/main/java/org/apache/pekko/japi/pf/AbstractPFBuilder.java:26-46 ; actor/src/main/java/org/apache/pekko/japi/pf/AbstractMatch.java:26-41 ; actor/src/main/java/org/apache/pekko/japi/pf/PFBuilder.java:46-143 ; actor/src/main/java/org/apache/pekko/japi/pf/Match.java:37-135 ; actor/src/main/scala/org/apache/pekko/japi/pf/CaseStatements.scala:24-29 (CaseStatement)

**Condiciones.** Sin coincidencia: scala.MatchError(<objeto>) (Match.java:131-132). matchEquals(null, ...) -> NullPointerException al evaluar object.equals(o) (PFBuilder.java:123; derivado, no afirmado). El builder es mutable y no thread-safe; tras build() queda vacío.

**Orden / concurrencia / ciclo de vida.** El orden de los casos es el orden de alta: Integer antes que Number en el spec.

**Dependencias.** AC-CELL-001 (receive como PartialFunction)

**Equivalente Rust propuesto.** `struct ReceiveBuilder<R> { cases: Vec<Case<R>> }` con `match_type::<T>(|t| ..)`, `match_type_if`, `match_equals`, `match_any`; `build() -> impl Fn(&dyn Any) -> Option<R>`; `MatchError` como valor `Err`.

**Diferencias observables.** `type.isInstance` incluye subtipos JVM; en Rust se compara TypeId exacto (Q-X-6/Q-JAPI-2): un caso `Number` no atraparía un `Integer`. Se reemplaza MatchError (excepción) por `Err(MatchError)` o `None`.

**Aserciones.**
- A1. Con casos Integer(*10) y Number(*-10): 4711 -> 47110.0; Double 4711 -> -47110.0; String "4711" -> MatchError — MatchBuilderTest.java:28-66
- A2. matchUnchecked(GenericClass.class, ..) extrae el valor "A" de GenericClass<String> — MatchBuilderTest.java:68-85
- A3. Con predicado (val no vacío) y valor "" -> MatchError — MatchBuilderTest.java:87-110
- A4. matchEquals("hello") + matchAny: isDefinedAt("hello") y isDefinedAt("42") son true; apply("42")==42 — PFBuilderTest.java:25-36
- A5. build() resetea el builder y sin casos devuelve PF vacía — derivada del código AbstractPFBuilder.java:39-46; no afirmada por spec

**Oráculo.** No aplicable (API Java).

**Decisiones.** Q-JAPI-2

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-JAPI-004.2 — japi.pf UnitMatch/UnitPFBuilder (variante con efecto, sin valor de retorno)

**Contrato observable.** Gemelos de `Match`/`PFBuilder` para acciones sin resultado: `UnitPFBuilder<I>` (final, ctor público) ofrece `match(Class, UnitApply)`, `matchUnchecked(Class, UnitApply)`, `match(Class, TypedPredicate, UnitApply)`, `matchUnchecked(Class, TypedPredicate, UnitApply)`, `matchEquals(obj, UnitApply)`, `matchEquals(obj, TypedPredicate, UnitApply)` y `matchAny(UnitApply)`. `UnitMatch<I>` estático crea un `UnitPFBuilder` con el primer caso (`match`, `matchUnchecked`, `matchEquals`, `matchAny`), `UnitMatch.create(builder)` congela la PF y `match(i)` la aplica lanzando `MatchError` si ningún caso aplica. La PF interna es `UnitCaseStatement` (`PartialFunction[F, Unit]`).

**Fuente.** actor/src/main/java/org/apache/pekko/japi/pf/UnitPFBuilder.java:26-179 ; actor/src/main/java/org/apache/pekko/japi/pf/UnitMatch.java:29-147 (match 37-45, create 120, match(i) 142) ; actor/src/main/java/org/apache/pekko/japi/pf/AbstractPFBuilder.java:26-46 ; actor/src/main/java/org/apache/pekko/japi/pf/AbstractMatch.java:26-41 ; actor/src/main/scala/org/apache/pekko/japi/pf/CaseStatements.scala:31-37 (UnitCaseStatement) ; consumidores: actor/src/main/scala/org/apache/pekko/actor/AbstractFSM.scala:427-442 (matchData) y FSMStopBuilder/FSMTransitionHandlerBuilder.

**Precondiciones.** Mismas que AC-JAPI-004.1 (no nulos).

**Comportamiento.** Idéntico a PFBuilder/Match salvo el tipo de resultado: `isDefinedAt` = predicado; `apply` ejecuta la acción y devuelve `BoxedUnit`. Orden de alta, `type.isInstance`, `equals` y `matchAny` como en 004.1. Diferencias de superficie con `PFBuilder`: `UnitPFBuilder` SÍ tiene `matchEquals(obj, TypedPredicate, UnitApply)` (UnitPFBuilder.java:138-160) y `PFBuilder`/`Match` NO tienen la variante con predicado; `UnitMatch` expone ambas formas de `matchUnchecked` (47, 72); `matchAny` de `UnitPFBuilder` usa un predicado siempre verdadero. `build()` hereda el reset de `AbstractPFBuilder`.

**Errores.** `UnitMatch.match(i)` sin coincidencia -> `scala.MatchError(i)`. `matchEquals(null, ..)` -> NPE en la evaluación. Una excepción del `UnitApply` se propaga a quien llama a `match`.

**Orden / concurrencia.** Orden de alta; builder mutable no seguro entre hilos; `UnitMatch` tras `create` es inmutable.

**Dependencias.** AC-JAPI-004.1 (misma mecánica), AC-JAPI-004.3 (FI.UnitApply), AC-JAPI-003 (consumidor).

**Equivalente Rust propuesto.** `ReceiveBuilder<()>`/`CaseBuilder<()>` de AC-JAPI-012 o `match` nativo. Se preserva el orden de casos y el MatchError como `Err`. Es la misma implementación que 004.1 con otro parámetro de tipo: en Rust ambos se unifican con un genérico `R` (con `R = ()`), por lo que NO requiere tipo separado.

**Aserciones.**
- A1. Ningún test de Pekko ejercita `UnitMatch` ni `UnitPFBuilder` directamente: se usan de forma indirecta en `AbstractFSM.matchData`, `FSMStopBuilder` y `FSMTransitionHandlerBuilder`, que ejerce AbstractFSMActorTest.java:52-62 (transición por `UnitApply2`; no pasa por `UnitPFBuilder` sino por `pekko.compat.PartialFunction.fromFunction`, AbstractFSM.scala:185-188) — por lo que la mecánica `UnitPFBuilder` queda *no afirmada por spec*.
- A2. `UnitMatch.match` con coincidencia ejecuta la acción; sin coincidencia lanza `MatchError` — *no afirmada por spec*; derivada de UnitMatch.java:142-144 y CaseStatements.scala:32-38. Verificada contra el jar publicado 1.7.1 (`ORACLE=jars tools/oracle/run.sh` con un script jshell local no versionado, 2026-10-31; es el cross-check de jars, no el build desde fuente): `UnitMatch.create(UnitMatch.match(String.class, ..)).match(1)` -> `scala.MatchError: 1 (of class java.lang.Integer)`; y `PFBuilder.build()` resetea (la segunda PF sólo acepta lo añadido después). Caracterización propuesta (reutilizando los tests de AC-JAPI-004.1 con `R = ()`).
- A3. `UnitPFBuilder.matchEquals(obj, TypedPredicate, UnitApply)` exige `equals` y luego el predicado — *no afirmada por spec*; derivada de UnitPFBuilder.java:138-160.

**Validación prevista.** Caracterización en Rust compartida con 004.1 (parametrizada por `R`); sin oráculo (semántica determinista de Java). Justificación: no hay spec; el único riesgo es duplicar lógica.

**Decisiones.** Q-JAPI-2 (builders por TypeId exacto), Q-JAPI-1. Efecto: con (c) de Q-JAPI-2 la fila se cierra como "no aplicable".

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-JAPI-004.3 — japi.pf FI: interfaces funcionales (Apply, UnitApply, Predicate, TypedPredicate, Apply2, UnitApply2/3/4, UnitApplyVoid, TypedPredicate2)

**Contrato observable.** `FI` es una clase final con constructor privado que agrupa interfaces funcionales (todas `@FunctionalInterface` y `@Deprecated`, "Will be removed in 2.0.0", con recomendación de usar `org.apache.pekko.japi.function.Function`): `Apply<I,R>` (`R apply(I) throws Exception`), `Apply2<I1,I2,R>`, `TypedPredicate<T>` (`boolean defined(T)`), `TypedPredicate2<T,U>`, `UnitApply<I>` (`void apply(I) throws Exception`), `UnitApply2<I1,I2>`, `UnitApply3`, `UnitApply4`, `UnitApplyVoid` (`void apply() throws Exception`), y el privado de paquete `Predicate` (`boolean defined(Object)`).

**Fuente.** actor/src/main/java/org/apache/pekko/japi/pf/FI.java:14-232 (1.7.1) (Apply 36-45, Apply2 60-70, TypedPredicate 82-91, TypedPredicate2 102-111, UnitApply 124-131, UnitApply2 142-150, UnitApply3 162-171, UnitApply4 184-194, UnitApplyVoid 206-209, Predicate 222-230) ; usos: ReceiveBuilder.java, PFBuilder.java, UnitPFBuilder.java, FSM*Builder.java, AbstractFSM.scala:50-51.

**Precondiciones.** n/a (tipos).

**Comportamiento.** Sin lógica: contratos de firma. Las que devuelven valor o ejecutan acción PUEDEN lanzar `Exception` comprobada (`throws Exception`), a diferencia de `java.util.function.*`; los predicados (`defined`) no lanzan comprobadas. Se usan como parámetro de los builders.

**Errores.** Una excepción lanzada por una acción se propaga al invocador (el runtime del actor, que la trata como fallo del actor; para `Apply` dentro de un `Match`, a quien llama `match`).

**Orden / concurrencia.** n/a.

**Dependencias.** AC-JAPI-004.1/.2, AC-JAPI-003, AC-JAPI-012.

**Equivalente Rust propuesto.** Closures `Fn`/`FnMut`/`FnOnce` (`Box<dyn FnMut(&T) -> Result<R, Error>>`). Efecto preservado: acciones falibles. Sin tipo propio. Es duplicado exacto de `japi.function.*` salvo el nombre del método (`defined` vs `test`).

**Aserciones.**
- A1. Sin aserciones de Pekko: no hay test de `FI`; las interfaces se usan en lambdas de ReceiveBuilderTest.java (p. ej. :98,176-195), PFBuilderTest.java:25-36 y MatchBuilderTest.java:29-110 (clases anónimas `FI.Apply`/`FI.TypedPredicate` con `@SuppressWarnings("deprecation")` en :27). Esos tests no afirman propiedades de `FI` sino de los builders.
- A2. Las acciones pueden lanzar `Exception` comprobada y los predicados no — *no afirmada por spec*; derivada de FI.java:45,70,131,150,171,194,209 (`throws Exception`) y :91,111,230 (sin throws). Sin test posible en Rust (todo closure puede devolver `Result`).

**Validación prevista.** Ninguna propia: se valida por los tests de los builders (004.1, 004.2, 012). Justificación: sólo firmas.

**Decisiones.** Q-JAPI-1 (fachada). Efecto: sin fachada, se cierra por equivalencia con closures.

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-JAPI-004.4 — japi.pf DeciderBuilder: decider de supervisión por tipo de Throwable

**Contrato observable.** Clase pública (no `final`, `DeciderBuilder.java:20`) con constructor privado; match(Class<P extends Throwable>, Apply<P, Directive>), match(Class, TypedPredicate<P>, Apply), matchAny(Apply<Throwable, Directive>) delegan en Match.match/matchAny.

**Fuente.** actor/src/main/java/org/apache/pekko/japi/pf/DeciderBuilder.java:23-81

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

### AC-JAPI-005.1 — japi.function (Function, Function2, Procedure, Effect, Predicate, Creator, identity) y japi.Function/Procedure/Effect/Predicate/Creator/Pair/Option/Util deprecados

**Contrato observable.** Dos familias de interfaces SAM para Java: (a) `org.apache.pekko.japi.function.*` (vigente, `Serializable`, `@SerialVersionUID(1L)`, con varianza): `Function[-T,+R]`, `Function2`, `Procedure[-T]`, `Effect`, `Predicate[-T]` (`test`), `Creator[+T]` (`create()`), con `Function.identity` (`@since 1.2.0`); `apply`/`create` pueden lanzar `Exception`. (b) `org.apache.pekko.japi.*` (deprecado desde 1.2.0, "Will be removed ... in Pekko 2.0.0"): `Function`, `Function2`, `Procedure`, `Effect`, `Predicate`, `Creator[T]` sin varianza y no `Serializable` salvo `Creator`. Utilidades en `JavaAPI.scala`: `Pair[A,B]` (case class `Serializable`, `toScala`, `Pair.create`), `Option[A]` (deprecado; `some`, `none`, `option(v)`, `fromScalaOption`, `get`, `getOrElse`, `isEmpty`, `isDefined`, `asScala`, `iterator`, `Iterable`) y `Util` (`classTag`, `immutableSeq` x3, `immutableSingletonSeq`, `javaArrayList`, `immutableIndexedSeq`, `option(Optional)` deprecado desde 1.2.1). `JavaPartialFunction` se contrata en AC-JAPI-013.

**Fuente.** actor/src/main/scala/org/apache/pekko/japi/function/Function.scala:29-155 (Function 28-30, object Function.identity 33-39, Function2 50, Procedure 63, Effect 76, Predicate 90, Creator 101-108) ; actor/src/main/scala/org/apache/pekko/japi/JavaAPI.scala:24-38 (versiones deprecadas) ; actor/src/main/scala/org/apache/pekko/japi/JavaAPI.scala:25-37 (Pair) ; actor/src/main/scala/org/apache/pekko/japi/JavaAPI.scala:100 (Option) ; actor/src/main/scala/org/apache/pekko/japi/JavaAPI.scala:101-154 (Util) ; actor/src/main/scala/org/apache/pekko/util/ConstantFun.scala:38 (javaIdentityFunction).

**Precondiciones.** n/a (tipos y utilidades puras).

**Comportamiento.** Interfaces sin lógica (firmas). Lógica real, toda trivial y determinista: `Option.some(v)`=Some(v) (permite null), `Option.none`=singleton (`assertSame`), `Option.option(v)`=`none` si `v==null` si no `some(v)`, `Option.Some.get`=v, `None.get` lanza `NoSuchElementException("None.get")`, `getOrElse(d)`=v o d, `asScala`, iteración con 0 ó 1 elemento (`iterator`), conversiones implícitas Scala<->Java. `Pair(first, second)`, `toScala`=(first, second). `Util.immutableSeq(array)`: copia a `IndexedSeq`, `Nil` si el array es null o vacío; `immutableSeq(Iterable)`: devuelve la misma si ya es `immutable.Seq`, si no copia con `VectorBuilder` y `EmptyImmutableSeq` si está vacía; `immutableSingletonSeq(v)`=`v :: Nil`; `javaArrayList(seq)` copia a `ArrayList`; `immutableIndexedSeq` = `immutableSeq(..).toVector`; `Util.option(Optional)` = `Option(jOption.orElse(null))` (NOTA: un `Optional.of(null)` no existe; `Optional.empty` -> None).

**Errores.** `None.get` -> `NoSuchElementException("None.get")`. `Util.option(null)` -> `NullPointerException`. `Creator.create()` puede lanzar `Exception` y debe devolver una instancia distinta en cada llamada (contrato de documentación, no verificado).

**Orden / concurrencia.** Inmutables; sin estado.

**Dependencias.** AC-JAPI-006 (Creator en Props), AC-JAPI-004.3 (FI es su predecesor), AC-UTIL (ConstantFun).

**Equivalente Rust propuesto.** Closures, `Option<T>`, tuplas `(A, B)`, `Vec<T>`/`Arc<[T]>` y `Result`. Efecto preservado: construcción y consulta de opciones/pares. No se portan: varianza, `Serializable` (capturar lambdas para línea de número es un artefacto de Java 8 documentado en los comentarios) ni el tipo `japi.Option` (duplicado de `java.util.Optional`).

**Aserciones.**
- A1. `Option.some("abc")`: `isEmpty` falso, `isDefined` verdadero, `get`="abc", `getOrElse("other")`="abc" — JavaAPITestBase.java:30-37 (test shouldCreateSomeString).
- A2. `Option.none()`: vacío, no definido, `getOrElse("other")`="other" — JavaAPITestBase.java:39-50.
- A3. Una `Option` presente entra en un `for` Java (itera una vez); una vacía no itera — JavaAPITestBase.java:52-65.
- A4. `Option.none()` es singleton (`assertSame`) — JavaAPITestBase.java:67-70.
- A5. `Pair.create(1, "2")`, `Tuple4.create(1,"2",3,4L)`, `Tuple22.create(...)` compilan (sólo compilación) — JavaAPI.java:145-151.
- A6. `Util.option(Optional.ofNullable(null))` está vacío y `Util.option(Optional.ofNullable("hello"))` está definido — JavaAPI.java:153-161.
- A7. `None.get` lanza `NoSuchElementException("None.get")`, `Option.option(null)` = none, `Util.immutableSeq` de array nulo/vacío = `Nil`, de `Iterable` ya `immutable.Seq` devuelve la misma instancia, `javaArrayList` copia en orden — *no afirmada por spec*; derivada de JavaAPI.scala:211-249,255-311. Medidos con el jar (ver AC-JAPI-004.2): `Option.none().get()` -> `NoSuchElementException: None.get`; `Option.option(null).isEmpty()` verdadero; `Util.immutableSeq(new String[0])` = `List()`; `japi.function.Function.identity().apply("z")` = "z". Caracterización propuesta con tests propios de las funciones de Rust equivalentes si Q-JAPI-1 acepta la fachada; si no, no aplica.
- A8. `Function.identity` devuelve su entrada — *no afirmada por spec Java*; derivada de Function.scala:39 y ConstantFun.scala:38. Trivial.
- A9. Los `japi.function.*` son `Serializable` y los `japi.*` deprecados no lo son (salvo `Creator`) — *no afirmada por spec*; propiedad JVM sin equivalente.

**Validación prevista.** Ninguna prueba específica de Rust si se excluye la fachada (closures y `Option` nativos cumplen por tipos). Con fachada: A1-A4, A6 se portan como tests de `option`/`some`/`none` y A7 como caracterización. Sin oráculo (funciones puras).

**Decisiones.** Q-JAPI-1. Efecto: sin fachada se cierra por equivalencia con `Option`/closures/tuplas; deprecado en 1.2.0 con retirada anunciada para Pekko 2.0.0 (evidencia para el Owner: lo deprecado no es parte del núcleo de largo plazo).

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-JAPI-005.2 — Funciones y tuplas generadas (japi.function.Function3..22, Procedure2..22, japi.tuple.Tuple3..22) desde boilerplate

**Contrato observable.** El generador de plantillas del build produce: `japi.function.Function3..Function22` (`trait FunctionN[-T1.., +R] extends Serializable { def apply(arg1: T1, ...): R }`), `Procedure2..Procedure22` (`apply(...)`: Unit; el valor `[2..#` del rango incluye 2 hasta el máximo) y `japi.tuple.Tuple3..Tuple22` (companion `Tuple<N>.create(t1..tN)` y `final case class Tuple<N>(t1..tN)` con `val toScala`). Las aridades 1 y 2 de las tuplas son `Pair` (JavaAPI.scala) y las de función 1 y 2 son `Function`/`Function2` (AC-JAPI-005.1).

**Fuente.** actor/src/main/boilerplate/org/apache/pekko/japi/function/Functions.scala.template:14-45 (`[3..22#` Function, `[2..#` Procedure) ; actor/src/main/boilerplate/org/apache/pekko/japi/tuple/Tuples.scala.template:14-31 (`[3..22#` Tuple y companion) ; plugin project/plugins.sbt:21 (sbt-boilerplate 0.7.0: el rango y `##1L` se expanden al compilar; las plantillas son la única fuente).

**Precondiciones.** n/a.

**Comportamiento.** Puramente declarativo: `Function<N>` lanza `Exception` permitida (`@throws(classOf[Exception])`), es `Serializable` con `@SerialVersionUID(1L)` y contravariante en los argumentos / covariante en `R`; `Tuple<N>` es case class (igualdad estructural, `toString`, `hashCode`, `copy` de Scala), `create` equivale al constructor, `toScala` es la tupla de Scala `(t1, .., tN)`. `Tuple<N>` NO es `scala.Product<N>` ni `scala.Tuple<N>`: es un tipo distinto hecho para Java. (El rango de Procedure en la plantilla es `[2..#` sin límite superior explícito: sbt-boilerplate 0.7.0, project/plugins.sbt:21, aplica su aridad máxima por defecto, 22; detalle del build, sin comportamiento en tiempo de ejecución.)

**Errores.** Ninguno (tipos).

**Orden / concurrencia.** n/a. Inmutables.

**Dependencias.** AC-JAPI-005.1.

**Equivalente Rust propuesto.** Tuplas nativas de Rust (hasta 12 elementos con `Debug`/`Eq`/`Hash` derivados) y closures. Efecto preservado: agrupar N valores y compararlos estructuralmente. Diferencias: Rust limita las tuplas con traits derivados a 12; Pekko llega a 22. Si se exigiera paridad de aridad se usaría un macro; recomendación: no portar.

**Aserciones.**
- A1. `Tuple4.create(1,"2",3,4L)` y `Tuple22.create(1..22)` compilan y existen en `japi.tuple` — JavaAPI.java:145-151 (`mustCompileTupleCreation`; sólo compila: no hay `assert`).
- A2. Igualdad estructural, `toScala` y `create` de `Tuple<N>`; aridad máxima 22; `Function<N>` serializables — *no afirmada por spec*; derivada de Tuples.scala.template:14-31 y Functions.scala.template:14-45. Caracterización: sin test en Rust (se confía en las tuplas nativas).

**Validación prevista.** Ninguna para Rust si se excluye la fachada. Justificación: sólo tipos generados; el único test es de compilación Java.

**Decisiones.** Q-JAPI-1. Efecto: exclusión propuesta apoyada en que lo generado es equivalente a tuplas y closures nativos.

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-JAPI-005.3 — japi.Throwables: isFatal / isNonFatal con la semántica de scala.util.control.NonFatal

**Contrato observable.** isNonFatal delega en scala.util.control.NonFatal; isFatal es su negación. Scaladoc :20-24 enumera fatales.

**Fuente.** actor/src/main/scala/org/apache/pekko/japi/Throwables.scala:20-63

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

**Fuente.** actor/src/main/java/org/apache/pekko/japi/JAPI.java:21-30 (1.7.1)

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

**Fuente.** actor/src/main/scala/org/apache/pekko/actor/AbstractProps.scala:33-41 (validate) ; actor/src/main/scala/org/apache/pekko/actor/AbstractProps.scala:46-48 (create(clazz, args)) ; actor/src/main/scala/org/apache/pekko/actor/AbstractProps.scala:52 (create(creator)) ; actor/src/main/scala/org/apache/pekko/actor/AbstractProps.scala:53-56 (create(class, creator)) ; actor/src/main/scala/org/apache/pekko/actor/AbstractProps.scala:58-103 (checkCreatorClosingOver)

**Condiciones.** Mensajes: "Actor class [<FQCN>] must not be abstract" (:35); "Actor class [<FQCN>] must be subClass of org.apache.pekko.actor.Actor or org.apache.pekko.actor.IndirectActorProducer." (:38-39); "erased Creator types (e.g. lambdas) are unsupported, use Props.create(actorClass, creator) instead" (:75-76); "unsupported type found in Creator argument [<x>]" (:72); "unexpected type: <t>" (:78); "cannot use non-static local Creator to create actors; make it static (e.g. local to a static method) or top-level" (:136).

**Orden / concurrencia / ciclo de vida.** Todas las validaciones son síncronas al construir el Props (antes de actorOf).

**Dependencias.** AC-REF-007 (Props), AC-CELL

**Equivalente Rust propuesto.** `Props::new<A: Actor>(factory: impl Fn() -> A + Send + Sync + 'static)`; las validaciones de clase abstracta/closing-over no tienen equivalente (el sistema de tipos las impide).

**Diferencias observables.** No hay clase abstracta ni Creator erasado en Rust: esos errores no existen (Q-JAPI-1).

**Aserciones.**
- A1. Props.create(creator) y Props.create(classOf[Actor], creator) crean actores — PropsCreationSpec.scala:72-82
- A2. Props(Actor), Props(classOf, null) y Props(classOf, null, null) crean actores — PropsCreationSpec.scala:53-70
- A3. Props.create(JavaAPITestActor.class) / con Creator lambda / con Creator erasado crean actores — JavaAPI.java:66-96
- A4. Los textos de validación SÍ están afirmados por spec (corrige "derivada"): closing-over de Creator anónimo/local no estático: "cannot use non-static local Creator to create actors; make it static (e.g. local to a static method) or top-level" — ActorCreationTest.java:186-203 (`Props.create(Actor.class, new Creator<Actor>(){..})`), 224-237 (`new C() {}`), 302-319 (`new P("hello") {}`); Creator erasado (`G`, lambda sin clase): "erased Creator types (e.g. lambdas) are unsupported, use Props.create(actorClass, creator) instead" — ActorCreationTest.java:205-215 y 327-341; clase abstracta: `String.format("Actor class [%s] must not be abstract", H.class.getName())` — ActorCreationTest.java:267-274.
- A5. Creator estático/top-level/paramétrico/acotado/superinterfaz infiere `p.actorClass()` (AbstractActor, Actor, UntypedAbstractActor...) y Creators anónimos creados en método estático se aceptan — ActorCreationTest.java:217-222, 239-265, 286-300; con clase explícita: `Props.create(TestActor.class, () -> new TestActor(17))` y Creator anónimo — 321-325, 343-353; Issue 20537 (creator reutilizado en varios Props) — 355-369.
- A6. Los textos "Actor class [..] must be subClass of org.apache.pekko.actor.Actor or org.apache.pekko.actor.IndirectActorProducer.", "unsupported type found in Creator argument [..]" y "unexpected type: .." — derivada del código AbstractProps.scala:38-39,72,78; no afirmada por spec (ActorCreationTest no los ejercita).
- A7. Constructores con parámetros: `Props.create(Class, "a","b",Integer.valueOf(17),18)` y variantes boxed/unboxed y `null` crean el actor y la respuesta refleja los argumentos ("a-b-17-18", "a-null-null-18", "null-undefined-0-0", "a-null-0-0") — JavaAPI.java:98-143 (resolución de constructor por reflexión Scala: concordancia con null y primitivas; sin equivalente en Rust).

**Oráculo.** Jar oficial: jshell con Props.create inválidos y volcar mensajes. Escenario no existe ni se ejecutó.

**Decisiones.** Q-JAPI-1

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-JAPI-007 — EventBus Java (EventBus, LookupEventBus, SubchannelEventBus, ScanningEventBus, ManagedActorEventBus)

**Contrato observable.** Adaptadores abstractos para implementar los buses de eventos desde Java sin tipos abstractos de Scala. `japi.EventBus[E,S,C]`: `subscribe(S, C): Boolean`, `unsubscribe(S, C): Boolean`, `unsubscribe(S): Unit`, `publish(E): Unit`. `LookupEventBus` (hooks protegidos `mapSize`, `compareSubscribers`, `classify`, `publish(E,S)`), `SubchannelEventBus` (`subclassification` público, `classify`, `publish(E,S)`), `ScanningEventBus` (`compareClassifiers`, `compareSubscribers`, `matches`, `publish(E,S)`), `ManagedActorEventBus[E](system)` (suscriptor y clasificador son `ActorRef`; hooks `mapSize`, `classify`).

**Fuente.** actor/src/main/scala/org/apache/pekko/event/japi/EventBusJavaAPI.scala:26-49 (EventBus), 57-100 (LookupEventBus), 108-144 (SubchannelEventBus), 152-195 (ScanningEventBus), 203-230 (ManagedActorEventBus) ; actor/src/main/scala/org/apache/pekko/event/EventBus.scala:92,133,216,272 (los traits Scala delegados) ; ejemplos/tests Java del módulo docs: docs/src/test/java/jdocs/event/EventBusDocTest.java:119-268 (implementaciones) y 277-349 (tests).

**Precondiciones.** Hooks implementados por la subclase; `ManagedActorEventBus` exige un `ActorSystem` en el constructor. Los hooks Java pueden ensancharse de `protected` a `public` (los ejemplos lo hacen).

**Comportamiento.** (1) Clasificación de fachada: cada clase crea internamente (campo `private val bus`) una instancia anónima del bus Scala correspondiente (`pekko.event.EventBus with LookupClassification`, `SubchannelClassification`, `ScanningClassification`, `ActorEventBus with ManagedActorClassification with ActorClassifier`) cuyos hooks reenvían a los métodos abstractos de la clase Java; `subscribe`, `unsubscribe` (2 formas) y `publish` delegan 1:1 en ese `bus`. No hay lógica propia: la semántica (orden de suscriptores, dedup, subclasificación, desuscripción automática al morir el suscriptor, BoundedBuffer...) es la de AC-EVT-002..005. (2) Único detalle propio: `ManagedActorEventBus` fija `override val system = ManagedActorEventBus.this.system` y su `mapSize` Java reenvía `ManagedActorEventBus.this.mapSize()`.

**Errores.** Los de los traits Scala delegados (AC-EVT-002..005): p. ej. `compareSubscribers` inconsistente rompe el orden; sin validación propia. Un hook que lanza propaga la excepción a quien llama `publish`.

**Orden / concurrencia.** Hereda el modelo de AC-EVT-002..005 (thread-safe; `publish` síncrono en el hilo del llamador). El campo `bus` se inicializa en la construcción de la clase base abstracta, ANTES del constructor de la subclase: los hooks no deben depender de campos de la subclase durante `subscribe` desde el constructor (derivado del orden de inicialización; no afirmado).

**Dependencias.** AC-EVT-001..005 (comportamiento), AC-EVT-022 (fila que cubre el mismo archivo desde EVT: DUPLICADA con ésta; coordinar para mantener un único contrato), Q-X-6 (Subclassification).

**Equivalente Rust propuesto.** Los mismos traits de bus de AC-EVT (`Lookup`, `Subchannel`, `Scanning`, `ManagedActor`), sin distinción Java/Scala. Se preserva el efecto observable completo de las filas EVT. No se ofrece una segunda API.

**Aserciones.**
- A1. `LookupEventBus`: tras `subscribe(testActor, "greetings")`, publicar `MsgEnvelope("time", ..)` no entrega y `MsgEnvelope("greetings","hello")` entrega "hello" — docs/src/test/java/jdocs/event/EventBusDocTest.java:277-290.
- A2. `SubchannelEventBus` con subclasificación "empieza por": suscripción a "abc"; "xyzabc" y "bcdef" no entregan, "abc" entrega "c" y "abcdef" entrega "d" — EventBusDocTest.java:292-308 (con `StartsWithSubclassification`, :152-162). Nota: el spec sitúa la semántica `isSubclass(x,y)=x.startsWith(y)` y espera que "abcdef" (subclase de "abc") llegue.
- A3. `ScanningEventBus` con clasificador numérico 3 y `matches = event.length() <= classifier`: "xyzabc" no entrega, "ab" entrega "ab", "abc" entrega "abc" — EventBusDocTest.java:310-325.
- A4. `ManagedActorEventBus`: subscriber1 y subscriber2 a observer1, subscriber2 a observer2; publicar n1 (clasificador observer1) llega a ambos; n2 (observer2) sólo a subscriber2 y probe1 no recibe nada en 500 ms — EventBusDocTest.java:327-349.
- A5. Los tests anteriores viven en el módulo `docs` (ejemplos de documentación), NO en `actor-tests`; en `actor-tests` no hay ningún test Java de estas clases. La semántica de los buses está afirmada por los specs Scala EventBusSpec.scala (LookupEventBusSpec :353, ScanningEventBusSpec :317, y los de Subchannel y ManagedActor del mismo archivo; AC-EVT-002..005).
- A6. `unsubscribe(S)` y `unsubscribe(S,C)` de las fachadas reenvían al bus Scala — *no afirmada por spec Java*; respaldo en EventBusSpec (Scala). Caracterización no necesaria.

**Validación prevista.** Ninguna propia: cubierta por los tests de AC-EVT-002..005 portados. Si Q-JAPI-1 acepta una fachada, A1-A4 (ejemplos de documentación) se portan como pruebas de integración de la fachada. Justificación: delegación 1:1 demostrable por lectura (EventBusJavaAPI.scala:95-98,139-142,191-194,226-229).

**Decisiones.** Q-JAPI-1; duplicidad con AC-EVT-022 (Q-EVT-5 en classic-system). Efecto: se recomienda cerrar JAPI-007 como "Duplicado/fachada equivalente de AC-EVT-002..005" y mantener AC-EVT-022 como registro de la exclusión/inclusión.

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-JAPI-008 — Sobrecargas Java con java.time.Duration, Optional y CompletionStage (Scheduler, ActorSelection, ActorSystem, Actor, Address)

**Contrato observable.** Métodos Java-friendly que convierten tipos de Java a los de Scala y delegan. Inventario verificado en `pekko-actor` (excluyendo CoordinatedShutdown, que cubren AC-CS-*): `Scheduler` (8 sobrecargas con `java.time.Duration` + `ExecutionContext`/`sender` explícitos): `scheduleWithFixedDelay(Duration, Duration, Runnable, EC)` (Scheduler.scala:129-135), `scheduleWithFixedDelay(Duration, Duration, receiver, message, EC, sender)` (:186-195), `scheduleAtFixedRate` análogas (:264-270, :332-342), `schedule` deprecadas (:373-383, :413-417), `scheduleOnce(Duration, receiver, message, EC, sender)` (:446-452) y `scheduleOnce(Duration, Runnable)(implicit EC)` (:490-493). `ActorSelection.resolveOne(java.time.Duration): CompletionStage[ActorRef]` (ActorSelection.scala:137-141) y `resolveOneCS(FiniteDuration|Duration)` deprecados (:110-126). `ActorSystem.getWhenTerminated: CompletionStage[Terminated]` (ActorSystem.scala:721 declaración, :1003 implementación `whenTerminated.asJava`). `ActorIdentity.getActorRef: Optional[ActorRef]` (Actor.scala:102-105), `Address.getHost(): Optional[String]` y `getPort(): Optional[Integer]` (Address.scala:57-62), `AbstractActor.preRestart(Throwable, Optional[Any])` (AbstractActor.scala:298), `AbstractActor.ActorContext.findChild: Optional` y los receive-timeout con Duration (AC-JAPI-001). Las sobrecargas de `AbstractFSM` (timers y `when/startWith` con Duration) se contratan en AC-JAPI-003.

**Fuente.** (las citadas arriba) ; actor/src/main/scala-2/org/apache/pekko/util/JavaDurationConverters.scala (Scala 2.13) y scala-3 equivalente: `asScala = Duration.fromNanos(self.toNanos)`, `asJava = JDuration.ofNanos(self.toNanos)` ; actor/src/main/scala/org/apache/pekko/util/FutureConverters.scala (CompletionStage <-> Future, AC-UTIL-018.1) ; actor/src/main/scala/org/apache/pekko/util/OptionConverters.scala (Option <-> Optional, AC-UTIL-018.2).

**Precondiciones.** Duraciones y argumentos no nulos (`null` -> `NullPointerException` en `toNanos`/`asScala`). Las de planificación exigen un `ExecutionContext` explícito (en Java no hay implícitos).

**Comportamiento.** Todas convierten `java.time.Duration` por nanosegundos y reenvían a la sobrecarga Scala equivalente; las que devuelven `CompletionStage` aplican `FutureConverters.asJava` al `Future` de la versión Scala (el `CompletionStage` se completa/falla exactamente como el `Future`); las que devuelven `Optional` aplican `toJava` al `Option`. Consecuencias propias: (a) duración de Java que no cabe en `Long` nanosegundos -> `ArithmeticException` de `Duration.toNanos` ANTES de planificar; (b) duración negativa o cero pasa tal cual a la validación del scheduler (AC-SCH-001..004, "must be > 0"/IllegalArgumentException en las periódicas, `scheduleOnce` acepta cero/negativo según AC-SCH-001), sin validación adicional; (c) `Address.getPort` devuelve `Optional<Integer>` (boxed) por un `asInstanceOf` explícito; (d) `resolveOne(Duration)` es la forma NO deprecada para Java; `resolveOneCS` quedó como alias deprecado.

**Errores.** `resolveOne`: `ActorNotFound` (el `CompletionStage` falla) si no existe o no responde en el timeout; las de `Scheduler` lanzan `IllegalArgumentException`/`IllegalStateException` como las Scala (AC-SCH-009 tras `close`). `Optional`: nunca null; vacío en lugar de ausencia.

**Orden / concurrencia.** Idéntica a las versiones Scala. El `CompletionStage` se completa en el dispatcher/hilo del `Future` subyacente.

**Dependencias.** AC-SCH-001..004, AC-REF-011 (ActorSelection.resolveOne), AC-SYS-004 (`whenTerminated`), AC-REF-013 (ActorIdentity), AC-PATH-001 (Address), AC-UTIL-018.1/.2, AC-JAPI-001/003.

**Equivalente Rust propuesto.** `std::time::Duration`, `Option<T>`, `Future` (tokio) y `JoinHandle`/`CancellationToken`; una sola API: las sobrecargas Java desaparecen. Se preserva el efecto observable de la implementación Scala; no se necesitan conversiones (`Duration` de Rust ya es nanosegundos de `u64`/`u32`: el límite de 292 años de Java desaparece, lo que es una diferencia benigna a documentar).

**Aserciones.**
- A1. `selection.resolveOne(Duration.ofMillis(10))` sobre un actor existente completa con esa misma ref antes de 3 s — ActorSelectionTest.java:36-47 (test :37-47).
- A2. `system.getWhenTerminated()` se completa (≤ 2 s) tras `system.terminate()` y NO está completo antes — ActorSystemTest.java:42-52 (`testGetWhenTerminated`, `testGetWhenTerminatedWithoutTermination`); el try-with-resources cierra el sistema y su `getWhenTerminated` completa — ActorSystemTest.java:54-62 (`testTryWithResources`).
- A3. `scheduleWithFixedDelay`, `scheduleAtFixedRate`, `scheduleOnce` con `java.time.Duration` — *no afirmada por spec Java* (ninguno de los tests Java del módulo `actor-tests` llama a estas sobrecargas; los specs Scala SchedulerSpec cubren la semántica, AC-SCH-001..004). Caracterización propuesta: la conversión Duration -> nanos produce el mismo calendario que la forma Scala.
- A4. `Address.getHost/getPort`, `ActorIdentity.getActorRef`, `AbstractActor.preRestart(.., Optional)` devuelven `Optional` vacío/presente según el `Option` — *no afirmada por spec Java*; derivada de Address.scala:57-62, Actor.scala:102-105. Trivial.
- A5. `ArithmeticException` con `Duration` mayor que ~292 años y `NullPointerException` con `null` — *no afirmada por spec*; derivada de la API JDK (`Duration.toNanos`). Sin equivalente en Rust.

**Validación prevista.** A1 y A2 portados como pruebas de integración de `resolve_one(timeout)` y `when_terminated()` (que ya existen en las filas REF/SYS en su forma Scala: el test Java es el mismo caso con otra firma). El resto no necesita test propio: las filas AC-SCH ya prueban el calendario. Justificación: delegación 1:1 por lectura.

**Decisiones.** Q-JAPI-1. Efecto: sin fachada, todas las sobrecargas se cierran por equivalencia con la API única de Rust. Se añade la nota de que las deprecadas (`schedule`, `resolveOneCS`, `setTimer`) siguen la decisión de sus filas Scala (AC-SCH-004).

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-JAPI-009.1 — TypedActor (clásico, deprecado): proxies JDK; semántica de invocación (void, Future, Option/JOption, valor)

**Contrato observable.** TypedActorInvocationHandler.invoke: toString/equals/hashCode se resuelven localmente contra el ActorRef; otros métodos: MethodCall(method,args): isOneWay (retorno void) -> actor ! m y null; returnsFuture -> ask(actor,m)(timeout).map(NullResponse -> null); returnsJOption/returnsOption -> Await.ready(ask, timeout) y, ante None/NullResponse/AskTimeoutException/TimeoutException, devuelve JOption.none/None; otro -> Await.result(ask,..) y NullResponse -> null. Las excepciones del método se propagan al llamante (reenvía Status.Failure) y el actor también falla (se aplica supervisión).

**Fuente.** actor/src/main/scala/org/apache/pekko/actor/TypedActor.scala:148-167 (1.7.1) (MethodCall) ; actor/src/main/scala/org/apache/pekko/actor/TypedActor.scala:226-228 (1.7.1) (NullResponse) ; actor/src/main/scala/org/apache/pekko/actor/TypedActor.scala:344-372 (1.7.1) (receive del actor) ; actor/src/main/scala/org/apache/pekko/actor/TypedActor.scala:466-499 (1.7.1) (invoke)

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

**Fuente.** actor/src/main/scala/org/apache/pekko/actor/TypedActor.scala:43-113 (1.7.1) (TypedActorFactory) ; actor/src/main/scala/org/apache/pekko/actor/TypedActor.scala:148-224 (1.7.1) (MethodCall/Serialized) ; actor/src/main/scala/org/apache/pekko/actor/TypedActor.scala:249-262 (1.7.1) (self/context) ; actor/src/main/scala/org/apache/pekko/actor/TypedActor.scala:274-372 (1.7.1) (TypedActor actor) ; actor/src/main/scala/org/apache/pekko/actor/TypedActor.scala:376-424 (1.7.1) (traits)

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

### AC-JAPI-009.3 — TypedProps, TypedActorExtension, TypedActorFactory y createActorRefProxy (creación de proxies, interfaces, timeout, dispatcher)

**Contrato observable.** (TypedActor clásico, deprecado desde Akka 2.6.0, `@deprecated("Use 'org.apache.pekko.actor.typed' API.")`.) `TypedProps[T]` es la configuración inmutable y compartible de un TypedActor: `interfaces`, `creator`, `dispatcher` (por defecto `Dispatchers.DefaultDispatcherId`), `deploy`, `timeout: Option[Timeout]` (por defecto `None`), `loader: Option[ClassLoader]`. Fábricas Scala (`apply[T](implementation: Class[T])`, `apply(interface, implementation)`, `apply(interface, creator: => T)`, `apply[T: ClassTag]()`) y constructores Java (clase; `Creator`; interfaz + clase). Modificadores `withDispatcher`, `withDeploy`, `withLoader` (Java con null -> `None`, y Scala con `Option`), `withTimeout` (Java con null -> `None`), `withInterface`, `withoutInterface`, `actorProps()`. `TypedProps.extractInterfaces(clazz)` = `[clazz]` si es interfaz, si no `clazz.getInterfaces`. `TypedActorFactory` (`typedActorOf(props)`, `typedActorOf(props, name)`, `typedActorOf(props, actorRef)`, `stop(proxy)`, `poisonPill(proxy)`, `isTypedActor`, `getActorRefFor`), `TypedActor(system)` extensión (`TypedActorExtension`: `DefaultReturnTimeout` desde `pekko.actor.typed.timeout`, `getActorRefFor`, `isTypedActor`, `createActorRefProxy`, `invocationHandlerFor`) y `TypedActor(context)` / `TypedActor.get(context)` -> `ContextualTypedActorFactory` (los typed actors creados son hijos del contexto).

**Fuente.** actor/src/main/scala/org/apache/pekko/actor/TypedActor.scala:43-113 (1.7.1) (TypedActorFactory) ; actor/src/main/scala/org/apache/pekko/actor/TypedActor.scala:115-143 (1.7.1) (extensión y contexto) ; actor/src/main/scala/org/apache/pekko/actor/TypedActor.scala:526-689 (1.7.1) (TypedProps) ; actor/src/main/scala/org/apache/pekko/actor/TypedActor.scala:691-700 (1.7.1) (ContextualTypedActorFactory) ; actor/src/main/scala/org/apache/pekko/actor/TypedActor.scala:702-774 (1.7.1) (TypedActorExtension, createActorRefProxy 734-757, invocationHandlerFor 762-773) ; actor/src/main/resources/reference.conf:146 (`pekko.actor.typed.timeout = 5s`).

**Precondiciones.** Al menos una interfaz en `props.interfaces` (el proxy JDK se crea con `Proxy.newProxyInstance(loader, interfaces)`; si el `loader` no está definido toma el del primer interfaz); `T <: R` y `R` debe ser una interfaz de las proxiadas. `typedActorOf(props, actorRef)` no crea actor.

**Comportamiento.** `typedActorOf(props[, name])`: crea un `AtomVar` para resolver el huevo-gallina proxy/actor, cachea `creator` e `interfaces` para no cerrar sobre `props`, crea el actor con `Props(new TypedActor.TypedActor(proxyVar, c(), i)).withDeploy(props.actorProps().deploy)` (el dispatcher de `TypedProps` SÍ llega al actor: `actorProps()` hace `Props.default.withDispatcher(d).withDeploy(deploy)` y el dispatcher de un `Props` vive en su `deploy`; ver A8) y devuelve el proxy; el orden de las sentencias en `createActorRefProxy` es relevante (comentario "do not change order"): primero `proxyVar.set(proxy)` y después `actorVar.set(actorRef)` para que el actor pueda leer `TypedActor.self` y el proxy ya apunte al actor. `typedActorOf(props, actorRef)`: sólo proxy; las llamadas se envían como `MethodCall` al `actorRef` dado (p. ej. un router). `stop(proxy)`: `false` si no es un proxy; si lo es, `InternalActorRef.stop()` y `true`. `poisonPill(proxy)`: `false` si no es proxy; si no `ref ! PoisonPill` y `true`. `getActorRefFor(proxy)`: `null` si no es proxy. `isTypedActor`: verdadero sólo si es una clase proxy JDK cuyo `InvocationHandler` es `TypedActorInvocationHandler`. `timeout` efectivo = `props.timeout.getOrElse(DefaultReturnTimeout)` (5 s por defecto), aplicado a métodos con retorno no-void (AC-JAPI-009.1).

**Errores.** `stop(null)`/`poisonPill(null)` devuelven `false` (invocationHandlerFor(null) = null). Interfaz inexistente / proxy no creable -> excepción de `Proxy.newProxyInstance` (IllegalArgumentException). Nombre de actor duplicado -> `InvalidActorNameException` (AC-CELL). `TypedActor.self` fuera de contexto -> `IllegalStateException` (AC-JAPI-009.2).

**Orden / concurrencia.** `TypedProps` inmutable y thread-safe. El contexto devuelto por `TypedActor(context)` no debe escapar del actor (documentado, TypedActor.scala:126-130).

**Dependencias.** AC-JAPI-009.1/.2, AC-CELL (creación de actores, Props/Deploy), AC-EXT (ExtensionId), AC-ASK (Timeout), AC-SYS-004.

**Equivalente Rust propuesto.** Sin equivalente dinámico: Rust no tiene proxies por reflexión. Opciones (Q-JAPI-3): excluir, o una macro `#[typed_actor] trait` que genere un proxy que traduzca llamadas a mensajes con las cuatro formas de retorno. Viabilidad alta pero con diseño propio; se preservaría: ciclo crear/parar/poisonPill, timeout por defecto 5 s configurable por `pekko.actor.typed.timeout`, proxy contra un `ActorRef` externo (router). Se perdería: múltiples interfaces y `loader`.

**Aserciones.**
- A1. `typedActorOf(TypedProps[Bar](classOf[Foo], classOf[Bar]).withTimeout(..))` crea un proxy que `isTypedActor` (true) y se detiene con `stop` (true) — TypedActorSpec.scala:292-296 (`be able to instantiate`), 298-301 (`be able to stop`).
- A2. `stop(null)` devuelve false (no detiene no iniciados) — TypedActorSpec.scala:303-305.
- A3. `TypedProps[StackedImpl](classOf[Stacked], classOf[StackedImpl])` soporta interfaces apiladas: `notOverriddenStacked`="foobar", `stacked`="FOOBAR" — TypedActorSpec.scala:458-463.
- A4. `typedActorOf(TypedProps[Bar]())` (sólo implementación; `extractInterfaces(Bar)`): `futurePigdog(200 ms)` y `futurePigdog(Zero)` no están completados al volver y producen el mismo resultado — TypedActorSpec.scala:465-473.
- A5. `TypedProps[StackedImpl]()` con interfaces complejas: `stackable1`="foo", `stackable2`="bar" — TypedActorSpec.scala:475-480.
- A6. `typedActorOf(TypedProps[Foo](), router)` (proxy sobre un `RoundRobinGroup` de typed actors): se pueden invocar `optionPigdog()` repetidamente; `getActorRefFor(t).path.toStringWithoutAddress` da las rutas de los routees; los cuatro `stop` devuelven true — TypedActorSpec.scala:606-636 (el test sólo escribe `info(..)` y no hace asserts sobre el valor de `optionPigdog`; las aserciones son `isTypedActor` de t1 y t2 y `mustStop`).
- A7. `pekko.actor.typed.timeout = 5s` es el timeout por defecto — reference.conf:147-152; consumo en TypedActor.scala:715 (`DefaultReturnTimeout`) — *no afirmada por spec* en su valor. El valor 5 s queda confirmado por `DefaultReturnTimeout` (ver A8); el efecto sobre una llamada concreta sigue siendo caracterización propuesta (AC-JAPI-009.1).
- A8. `props.dispatcher` se aplica al actor creado por `typedActorOf` — *no afirmada por spec* (hipótesis inicial de que no se aplicaba, REFUTADA). Verificada contra el jar publicado 1.7.1 (`ORACLE=jars tools/oracle/run.sh` con un script jshell local no versionado, 2026-10-31; es el cross-check de jars, no el build desde fuente): con `withDispatcher("my-disp")` el método del proxy corre en un hilo `O2-my-disp-5`; sin dispatcher en `O2-pekko.actor.default-dispatcher-4`; con `pekko.actor.internal-dispatcher` en `O2-pekko.actor.internal-dispatcher-2`. Además `isTypedActor(proxy)`=true, `stop(null)`=false y `DefaultReturnTimeout`=`Timeout(5000 milliseconds)` (confirma A7 y A2). El spec "be able to use balancing dispatcher" (TypedActorSpec.scala:482-491; `newFooBar("pooled-dispatcher", ..)` construye con `.withDispatcher`, :278-281) sólo comprueba que 120 llamadas devuelven "Pigdog"+i, no en qué dispatcher corren. No se declara como defecto: pendiente de verificación con oráculo (propuesta: crear typed actor con `withDispatcher("pooled-dispatcher")` y leer `getContext().getDispatcher()` desde dentro).
- A9. `TypedProps.withTimeout(null)` -> `None`; `withLoader(null)` -> `None`; `withoutInterface` quita las interfaces extraídas — *no afirmada por spec*; derivada de TypedActor.scala:645,659,679-680.

**Validación prevista.** Si Q-JAPI-3 acepta (b): portar A1-A6 contra el proxy macro-generado más oráculo para A7-A8. Si se excluye: se documenta la exclusión sin tests. A7/A8 ya fueron medidas con el jar (ver A8). Justificación: deprecado, único test de integración es TypedActorSpec.

**Decisiones.** Q-JAPI-3 (excluir o macro). Efecto: A7 sólo importa si se porta. Bloquea únicamente AC-JAPI-009.1-.3; nada más depende.

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-JAPI-010 — Fábricas Java de mensajes IO: TcpMessage, UdpMessage, UdpConnectedMessage, TcpSO, UdpSO y SoJavaFactories

**Contrato observable.** Objetos estáticos para Java que construyen comandos de IO y opciones de socket. `TcpMessage`: `connect(remote, local, options, timeout: FiniteDuration | java.time.Duration, pullMode)`, `connect(remote)`, `bind(handler, endpoint, backlog, options, pullMode)`, `bind(handler, endpoint, backlog)`, `register(handler, keepOpenOnPeerClosed, useResumeWriting)`, `register(handler)`, `unbind`, `close`, `confirmedClose`, `abort`, `noAck(token)`, `noAck`, `write(data, ack)`, `write(data)`, `writeFile(String, pos, count, ack)`, `writePath(Path, pos, count, ack)`, `resumeWriting`, `suspendReading`, `resumeReading`, `resumeAccepting(batchSize)`. `UdpMessage`: `noAck`, `send` (2), `bind` (2), `unbind`, `simpleSender` (2), `suspendReading`, `resumeReading`. `UdpConnectedMessage`: `connect` (3), `send` (2), `disconnect`, `noAck`, `suspendReading`, `resumeReading`. `TcpSO` (`keepAlive`, `oobInline`, `tcpNoDelay`), `UdpSO` (`broadcast`), `SoJavaFactories` (`receiveBufferSize`, `reuseAddress`, `sendBufferSize`, `trafficClass`).

**Fuente.** actor/src/main/scala/org/apache/pekko/io/Tcp.scala:674-679 (TcpSO), 702-933 (TcpMessage completo, incluida la conversión implícita privada `fromJava` -> `Util.immutableSeq`) ; actor/src/main/scala/org/apache/pekko/io/Udp.scala:259-365 (UdpMessage desde 257, UdpSO desde 354) ; actor/src/main/scala/org/apache/pekko/io/UdpConnected.scala:196-277 (UdpConnectedMessage) ; actor/src/main/scala/org/apache/pekko/io/Inet.scala:216-259 (SoJavaFactories) ; ejemplos Java: docs/src/test/java/jdocs/io/IODocTest.java:48-97, JavaReadBackPressure.java:45, UdpDocTest.java, UdpConnectedDocTest.java, JavaUdpMulticast.java.

**Precondiciones.** `options` es un `java.lang.Iterable<SocketOption>` (no nulo salvo `Nil` implícito); `Path`/`String` válidos para `writePath/writeFile`.

**Comportamiento.** Fachada pura: cada método construye el case class/object correspondiente de Scala con los defaults de la forma Scala. Conversiones propias: (a) `connect(remote, local, options, timeout, pullMode)`: `local` null -> `None` (`Option(localAddress)`), `timeout` null -> `None`, `options` Iterable -> `immutable.Seq` vía `Util.immutableSeq` (copia); la variante `java.time.Duration` convierte con `asScala` primero; (b) `connect(remote)` = `Connect(remote, None, Nil, None, pullMode=false)`; (c) `bind(handler, endpoint, backlog)` = `Bind(.., Nil)` y el `pullMode` de la forma larga se respeta; (d) `register(handler)` = `Register(handler)` (keepOpenOnPeerClosed=false, useResumeWriting=true por defecto de la case class); (e) `write(data)` = `Write(data)` (ack = NoAck); (f) `writeFile(String)` = `WritePath(Paths.get(String), ..)`; (g) UdpMessage.bind/simpleSender copian las opciones a `IndexedSeq` (`asScala.to(IndexedSeq)`); (h) `UdpConnectedMessage.connect(handler, remote, local, options)` envuelve `local` SIN `Option(..)` -> `Some(localAddress)`: pasar `null` produce `Some(null)` (diferencia con `TcpMessage.connect`, que usa `Option(..)`), y las otras formas usan `None`; (i) `UdpConnectedMessage.send(data, ack: AnyRef)` frente a `Udp.send(.., ack: Event)`: el tipo del ack difiere (`AnyRef` vs `Event`); (j) `TcpSO.keepAlive(on)` etc. = `KeepAlive(on)` etc.

**Errores.** Los de los constructores Scala (`require` de AC-IO-002.1: p. ej. `Bind` con backlog negativo, `Connect` con parámetros inválidos, `Write` vacío si la case class lo valida; `ResumeAccepting(batchSize > 0)`), no añadidos por la fachada. `null` donde la fachada no lo tolera -> `NullPointerException` (p. ej. `options` null en `UdpMessage.bind`).

**Orden / concurrencia.** n/a (constructores puros).

**Dependencias.** AC-IO-002.1, AC-IO-002.2..002.6, AC-IO-005, AC-IO-006.1/.2, AC-IO-007, AC-UTIL-018.2.

**Equivalente Rust propuesto.** Construir directamente los enums/structs de mensajes (`Tcp::Connect { .. }`, builders con `Default`). Sin API Java. Se preserva: valores por defecto de cada comando; `Option` para local/timeout. Diferencia a declarar: el caso `Some(null)` de `UdpConnectedMessage.connect` no puede existir en Rust.

**Aserciones.**
- A1. Los comandos creados con `TcpMessage.connect(...)`, `bind(...)`, `register(...)`, `resumeAccepting(1)` se envían a `IO.Tcp` y funcionan en los ejemplos de documentación — docs/src/test/java/jdocs/io/IODocTest.java:48,56,64,97 y JavaReadBackPressure.java:45 (el módulo `docs`; son ejemplos/compilación, sin aserciones sobre la igualdad de los comandos).
- A2. Ningún test de `actor-tests` usa `TcpMessage`, `UdpMessage` ni `UdpConnectedMessage` (búsqueda en el árbol de tests: sólo docs/src/test/java) — por tanto la igualdad `TcpMessage.x(...) == Tcp.X(...)` es *no afirmada por spec*. Caracterización propuesta: para cada fábrica, comparar con el case class construido a mano con los mismos argumentos (incluidos los casos null -> None).
- A3. `UdpConnectedMessage.connect(handler, remote, null, options)` -> `Connect(.., Some(null), ..)` — *no afirmada por spec*; derivada de UdpConnected.scala:208. Solo se documenta; no se porta.
- A4. `TcpSO.tcpNoDelay(true)` = `Tcp.SO.TcpNoDelay(true)` — *no afirmada por spec*; derivada de Tcp.scala:672-677. Trivial.

**Validación prevista.** Por fachada, sin pruebas dedicadas si Q-JAPI-1 excluye; el comportamiento de los comandos lo validan AC-IO-002.x/006/007. Con fachada: A2 como test de igualdad por fábrica. Sin oráculo.

**Decisiones.** Q-JAPI-1. Efecto: sin fachada, la fila se cierra por equivalencia con la construcción directa de los mensajes.

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-JAPI-011 — Clases base Java del núcleo: AbstractScheduler, JSerializer, AbstractActorRef/AbstractActorCell/AbstractPromiseActorRef/AbstractCircuitBreaker (offsets Unsafe) y annotation/* (@InternalApi, @DoNotInherit, @ApiMayChange, @InternalStableApi)

**Contrato observable.** Archivos Java/Scala de `pekko-actor` que sirven a Java y no son actores. (1) `AbstractScheduler` (público): clase abstracta que extiende `AbstractSchedulerBase` (= `Scheduler`, Scheduler.scala:503-504) para implementar un `Scheduler` en Java; exige `schedule(FiniteDuration, FiniteDuration, Runnable, ExecutionContext)` (`@Deprecated`), `scheduleOnce(FiniteDuration, Runnable, ExecutionContext)` y `maxFrequency()`; ofrece las dos sobrecargas con `java.time.Duration` ya implementadas. (2) `JSerializer` (público, Serializer.scala:292-303): base para escribir serializadores en Java; `fromBinary(bytes, manifest: Option[Class])` es final y delega en el abstracto `fromBinaryJava(bytes, manifest.orNull)` (manifest puede ser null). (3) `AbstractActorRef`, `AbstractActorCell`, `AbstractPromiseActorRef`, `AbstractCircuitBreaker`: clases `final`/package-private de utilería que obtienen con `Unsafe.objectFieldOffset` los desplazamientos de campos volátiles (`_cellDoNotCallMeDirectly`, `_lookupDoNotCallMeDirectly`, `_stateDoNotCallMeDirectly`, `_watchedByDoNotCallMeDirectly`, `_currentStateDoNotCallMeDirectly`, `_currentResetTimeoutDoNotCallMeDirectly`, y los de `ActorCell`: mailbox, children, nextName, functionRefs) para CAS en Scala; sin API ni comportamiento propio. (4) `annotation/ApiMayChange`, `DoNotInherit`, `InternalApi`, `InternalStableApi`: anotaciones `@Documented @Retention(CLASS)` (retención CLASS "to be accessible by MiMa") con `@Target` TYPE (DoNotInherit) o METHOD/CONSTRUCTOR/FIELD/TYPE/PACKAGE (las otras tres).

**Fuente.** actor/src/main/java/org/apache/pekko/actor/AbstractScheduler.java:14-57 (Javadoc contractual 14-31, clase 32, schedule abstracto 43 y sobrecarga Duration 55, scheduleOnce abstracto 72 y sobrecarga Duration 79, maxFrequency 89) ; actor/src/main/scala/org/apache/pekko/actor/Scheduler.scala:426-428 (AbstractSchedulerBase) ; actor/src/main/scala-2/org/apache/pekko/util/JavaDurationConverters.scala (asFiniteDuration) ; actor/src/main/scala/org/apache/pekko/serialization/Serializer.scala:298-315 (JSerializer) ; actor/src/main/java/org/apache/pekko/actor/AbstractActorRef.java ; actor/src/main/java/org/apache/pekko/actor/dungeon/AbstractActorCell.java ; actor/src/main/java/org/apache/pekko/pattern/AbstractPromiseActorRef.java ; actor/src/main/java/org/apache/pekko/pattern/AbstractCircuitBreaker.java ; actor/src/main/java/org/apache/pekko/annotation/ApiMayChange.java:36-45, DoNotInherit.java:33-36, InternalApi.java:31-40, InternalStableApi.java:33-42.

**Precondiciones.** `AbstractScheduler`: la subclase implementa los tres métodos abstractos. `JSerializer`: ctor con un único `ExtendedActorSystem` (preferido) o vacío, como todo serializador (AC-SER-001.2).

**Comportamiento.** (1) `AbstractScheduler.schedule(Duration, Duration, Runnable, EC)` = `schedule(asFiniteDuration(initialDelay), asFiniteDuration(interval), runnable, executor)` y `scheduleOnce(Duration, Runnable, EC)` = `scheduleOnce(asFiniteDuration(delay), runnable, executor)` (AbstractScheduler.java:55-60 y 79-82): fachada de conversión sobre los abstractos que implementa el usuario. Requisitos contractuales de implementación del Javadoc (AbstractScheduler.java:14-31): si el scheduler es `Closeable` DEBE ejecutar todas las tareas pendientes en `close()` para cerrar correctamente los dispatchers (una tarea ejecutada en `close()` puede correr antes de su plazo); DEBE lanzar `IllegalStateException` si no puede planificar una tarea; una vez planificada, la tarea DEBE ejecutarse; se instancia por reflexión al arrancar el `ActorSystem` con argumentos `(Config de system.settings.config, LoggingAdapter, ThreadFactory)` (AC-SCH-010). Los demás métodos de `Scheduler` (`scheduleWithFixedDelay`, `scheduleAtFixedRate`, mensajes a actor...) son `final` en `Scheduler` y se implementan sobre `scheduleOnce`/`schedule` (AC-SCH). (2) `JSerializer`: sólo reenvía `fromBinary` y exige implementar `identifier`, `includeManifest`, `toBinary`, `fromBinaryJava`. (3) Offsets Unsafe: inicialización estática; si el campo no existe lanza `ExceptionInInitializerError(t)`; detalle de plataforma JVM cubierto por AC-UTIL-020. (4) Anotaciones: sin efecto en ejecución; sólo documentación y verificación de compatibilidad binaria en el build.

**Errores.** `AbstractScheduler`: `NullPointerException` con Duration null (conversión); los de `Scheduler`. Offsets: `ExceptionInInitializerError` al cargar la clase si el nombre del campo cambia. Anotaciones: n/a.

**Orden / concurrencia.** Los offsets sirven a accesos atómicos (CAS/volatile) de ActorCell, RepointableActorRef, PromiseActorRef y CircuitBreaker (AC-CELL, AC-REF, AC-ASK, AC-CB); sin orden propio. El scheduler es llamado desde múltiples hilos (responsabilidad de la implementación).

**Dependencias.** AC-SCH (Scheduler, implementación pluggable AC-SCH-010), AC-SER-002 (Serializer), AC-UTIL-020 (Unsafe y offsets), AC-CELL, AC-ASK, AC-CB, AC-UTIL-018.2.

**Equivalente Rust propuesto.** (1) `trait Scheduler` implementable por el usuario (hace falta para `scheduler.implementation`, AC-SCH-010): un solo trait, sin clase "Abstract" Java ni sobrecarga Duration. (2) `JSerializer`: nada (el trait `Serializer` de Rust ya es el SPI único; `manifest.orNull` -> `Option<&TypeId>`). (3) Offsets Unsafe: no existen (atómicos de Rust: `AtomicPtr`, `ArcSwap`). (4) Anotaciones: `#[doc(hidden)]`, `pub(crate)`, `#[non_exhaustive]` y marcado de estabilidad en docs; `@DoNotInherit` -> traits selladas (`sealed` pattern). Se preserva el efecto observable de cada una: que un `Scheduler` de usuario se pueda conectar, y que las APIs internas no sean extensibles por el usuario.

**Aserciones.**
- A1. Ningún test de `actor-tests`, `testkit` o `actor-tests` Java ejercita `AbstractScheduler`, `JSerializer`, `AbstractActorRef`, `AbstractActorCell`, `AbstractPromiseActorRef`, `AbstractCircuitBreaker` ni las anotaciones directamente (búsqueda de sus nombres en `actor-tests/src/test`, `testkit/src` y `docs/src/test`: sólo aparecen definiciones en `actor/src/main`) — sin aserciones de Pekko. Los offsets Unsafe se ejercitan de forma indirecta por los specs de ActorCell/PromiseActorRef/CircuitBreaker (AC-CELL, AC-ASK, AC-CB).
- A2. `AbstractScheduler.scheduleOnce(Duration, ..)` y `schedule(Duration, Duration, ..)` (y los requisitos del Javadoc: ejecutar pendientes en close, IllegalStateException) convierten por `asFiniteDuration` y delegan en las abstractas — *no afirmada por spec*; derivada de AbstractScheduler.java. Caracterización propuesta: un scheduler de prueba que registra los `FiniteDuration` recibidos y comprueba la conversión.
- A3. `JSerializer.fromBinary(bytes, Some(c))` llama `fromBinaryJava(bytes, c)` y con `None` pasa `null` — *no afirmada por spec*; derivada de Serializer.scala:295-298. En Rust no aplica.
- A4. Las anotaciones tienen retención `CLASS` y sólo metadatos — *no afirmada por spec* (declaración en el fuente; sin comportamiento en tiempo de ejecución).

**Validación prevista.** (1) `AbstractScheduler`: ninguna en Rust; la conexión de un scheduler de usuario se prueba en AC-SCH-010. (2-4): sin pruebas (internos/JVM). Justificación: sin comportamiento observable por la API pública.

**Decisiones.** Q-JAPI-1 (para AbstractScheduler y JSerializer como fachada). Efecto: AbstractScheduler y JSerializer se cierran por equivalencia con el trait `Scheduler`/`Serializer` único; offsets y anotaciones son detalle interno sin contrato independiente (frontera: AC-UTIL-020).

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-JAPI-012 — ReceiveBuilder (receiveBuilder()): construcción del Receive por casos con orden de alta, isInstance, igualdad, predicados y matchAny

**Contrato observable.** `ReceiveBuilder.create()` (o `AbstractActor.receiveBuilder()`) acumula casos en orden de alta y `build()` devuelve un `Receive` cuya función parcial prueba los casos en ese orden y ejecuta el primero que aplica. Casos: `match(Class<P>, UnitApply<P>)`, `match(Class, TypedPredicate, UnitApply)`, `match(Class, BooleanSupplier, UnitApply)`, `matchUnchecked(...)` (3 formas, sin chequeo genérico), `matchEquals(obj, ...)` (3 formas) y `matchAny(...)` (2 formas). `type.isInstance(o)` incluye subtipos; `matchEquals` usa `object.equals(o)`.

**Fuente.** actor/src/main/java/org/apache/pekko/japi/pf/ReceiveBuilder.java:50-298 (addStatement 52-55, build 63-69, create 76, match/matchUnchecked 87-203, matchEquals 210-268, matchAny 278-297) ; actor/src/main/scala/org/apache/pekko/japi/pf/CaseStatements.scala:24-37 (UnitCaseStatement, CaseStatement.empty) ; actor/src/main/scala/org/apache/pekko/actor/AbstractActor.scala:44-57.

**Precondiciones.** Sin tipos nulos: `type`, `object`, `apply`, `predicate` y `externalPredicate` deben ser no nulos; nada se valida al construir.

**Comportamiento.** `addStatement` encadena con `orElse` (primero en añadirse, primero en probarse). `build()` devuelve `new Receive(statements.orElse(empty))` o `new Receive(empty)` si no hay casos; la PF `empty` no está definida para ningún mensaje (CaseStatement.empty = `PartialFunction.empty`). El comentario del código (`FIXME why no new Receive(statements)?`) reconoce que el `orElse(empty)` es redundante. Los predicados con `BooleanSupplier` se evalúan DESPUÉS de comprobar el tipo y en cada `isDefinedAt`/`apply` (la función parcial se consulta primero con `isDefinedAt` y luego con `apply` en el runtime). Para `matchUnchecked` con tipo y aplicación incompatibles, el fallo (`ClassCastException`) ocurre al aplicar, no al construir.

**Errores.** Sin coincidencia no hay excepción: `isDefinedAt` es falso y el actor trata el mensaje como no manejado. `matchEquals(null, ..)` lanza `NullPointerException` al evaluar `object.equals(o)` (en cada mensaje, no al registrar). `type == null` -> `NullPointerException` al evaluar `type.isInstance`. `matchUnchecked` con tipos cruzados -> `ClassCastException` al aplicar (afirmado, A11). Una excepción del cuerpo (`UnitApply.apply throws Exception`) sale por el `receive` y la gestiona la supervisión.

**Orden / concurrencia.** Orden de casos = orden de llamada. El builder es mutable y no seguro entre hilos; es habitual construirlo en `createReceive` (un solo hilo). DIFERENCIA DOCUMENTAL: el Javadoc de `build()` dice "After this call the builder will be reset" pero `ReceiveBuilder.build()` NO resetea `statements` (a diferencia de `AbstractPFBuilder.build()`, AbstractPFBuilder.java:39-46, que sí pone `statements = null`): llamar dos veces a `build()` devuelve dos `Receive` con los mismos casos y se pueden seguir añadiendo casos tras el primer `build()`.

**Dependencias.** AC-JAPI-001, AC-JAPI-004.3 (FI.UnitApply/TypedPredicate), AC-JAPI-004.2 (UnitPFBuilder, familia análoga), AC-CELL-001.

**Equivalente Rust propuesto.** Un `match` de Rust sobre un enum de mensajes ya es el equivalente idiomático para actores tipados; para actores clásicos con `Box<dyn Any>` (Q-X-3) se ofrece `ReceiveBuilder::new().match_type::<T>(|t| ..).match_type_if::<T>(pred, |t| ..).match_equals(v, |v| ..).match_any(|m| ..).build()`. Se conserva: orden de alta, primer caso gana, predicado tras el tipo, ausencia de excepción por no coincidencia. No se conserva: subtipos (Q-X-6/Q-JAPI-2: TypeId exacto), `matchUnchecked` (sin genéricos reificados), `BooleanSupplier` (cualquier closure sirve) ni el comportamiento "build no resetea" (decidir; recomendación: `build(self)` consume el builder).

**Aserciones.**
- A1. Sin casos: `isDefinedAt("hello")` y `isDefinedAt(42)` son falsos — ReceiveBuilderTest.java:89-94.
- A2. `match(Msg1.class, ..)`: Msg1 se procesa ("match Msg1"); Msg2, String e Integer no están definidos — ReceiveBuilderTest.java:96-106.
- A3. `match(Msg.class, ..)` (interfaz) acepta subclases Msg1 y Msg2 (isInstance) — ReceiveBuilderTest.java:108-121.
- A4. Con casos Msg1, Msg2, Msg en ese orden, cada mensaje va al handler más específico por ORDEN de alta (Msg1 -> "match Msg1", Msg2 -> "match Msg2") — ReceiveBuilderTest.java:135-150. (La sobrecarga se resuelve en compilación Java, no en el builder.)
- A5. Un handler general (`anotherHandleMsg(Msg)`) recibe Msg2 ("match Msg") y el específico Msg1 — ReceiveBuilderTest.java:160-174.
- A6. `match(type, predicate, apply)`: Msg1 con `true` coincide, Msg2 con `value.equals("foo")` coincide, Msg2("bar") no, String/Integer no — ReceiveBuilderTest.java:176-195.
- A7. `match(type, BooleanSupplier, apply)`: Msg1 coincide cuando el supplier da `true`; Msg2 no (tipo) — ReceiveBuilderTest.java:201-211.
- A8. `matchEquals(msg2, ..)`, `matchEquals("foo", ..)`, `matchEquals(17, ..)` usan `equals`: `Msg2("foo")` (otra instancia igual), "foo" y 17 coinciden; `Msg2("bar")`, "hello" y 42 no — ReceiveBuilderTest.java:213-237.
- A9. `matchAny` tras `match(Msg1)`: Msg1 va al primero, Msg2/String/Integer a `matchAny` (orden: el específico antes que el comodín) — ReceiveBuilderTest.java:239-256.
- A10. `matchUnchecked(List.class, (List<String> l) -> ..)` acepta una `List` — ReceiveBuilderTest.java:258-272.
- A11. `matchUnchecked(String.class, (Integer i) -> ..)` con "foo": `isDefinedAt` es true y `apply` lanza `ClassCastException` — ReceiveBuilderTest.java:274-287.
- A12. `build()` no resetea el builder y llamarlo dos veces comparte los casos; `matchEquals(null)` lanza `NullPointerException` al evaluar; `matchAny(BooleanSupplier, ..)` con supplier falso no coincide — *no afirmada por spec*; derivada de ReceiveBuilder.java:63-69,215-218,290-297. Verificada contra el jar publicado 1.7.1 (`ORACLE=jars tools/oracle/run.sh` con un script jshell local no versionado, 2026-10-31; es el cross-check de jars, no el build desde fuente): construir `match(String)`, `build()`, añadir `match(Integer)`, `build()` otra vez: la segunda PF acepta String e Integer (los casos previos se conservan), mientras que `PFBuilder` equivalente tras `build()` queda vacío (la segunda PF sólo acepta Integer); `matchEquals(null, ..)` -> `NullPointerException`; `matchAny(() -> false, ..)` no está definido. Caracterización propuesta con tests propios de Rust (marcados como caracterización, no test original de Pekko).

**Validación prevista.** Portar A1-A11 como tests del `ReceiveBuilder` de Rust adaptando `isInstance` a TypeId exacto (A3 se reescribe con un `trait`/`enum` registrado según Q-JAPI-2 o se marca como divergencia declarada) y A12 como caracterización. Sin oráculo: son semánticas de Java deterministas ya afirmadas por el spec. Para decidir "fachada o comportamiento propio": este es el ÚNICO componente de JAPI con semántica diferencial propia no derivada de otra implementación (no delega en un equivalente Scala; el equivalente Scala es `case` de pattern matching, que no es una clase).

**Decisiones.** Q-JAPI-2 (isInstance vs TypeId exacto), Q-JAPI-1 (fachada). Efecto: si no se ofrece builder (Q-JAPI-2 c), A1-A11 se cierran como "no aplicable, patrón nativo de Rust"; si se ofrece, A3/A9 definen la divergencia de subtipos. No bloquea otras filas.

**Estado.** analysis=extracted; impl=pending; validation=pending.

### AC-JAPI-013 — JavaPartialFunction: función parcial pura para Java (noMatch, isCheck, isDefinedAt/apply/applyOrElse)

**Contrato observable.** `abstract class JavaPartialFunction[A, B] extends AbstractPartialFunction[A, B]`: el usuario implementa `apply(x: A, isCheck: Boolean): B` (puede lanzar `Exception`) y señala "no definido" lanzando `JavaPartialFunction.noMatch()` (una `NoMatchException` preasignada, `NoStackTrace`). `isDefinedAt(x)` llama `apply(x, true)` y devuelve `true` salvo que se lance `NoMatch`; `apply(x)` llama `apply(x, false)` y convierte `NoMatch` en `scala.MatchError(x)`; `applyOrElse(x, default)` llama `apply(x, false)` y, ante `NoMatch`, devuelve `default(x)`.

**Fuente.** actor/src/main/scala/org/apache/pekko/japi/JavaAPI.scala:39-99 (JavaPartialFunction, NoMatch, noMatch, documentación del patrón isCheck 126-160) ; consumidor: testkit/src/main/scala/org/apache/pekko/testkit/javadsl/TestKit.scala (usa `JavaPartialFunction` en `expectMsgPF`/`receiveWhile`; verificado por búsqueda: es el único uso fuera de la definición).

**Precondiciones.** El cuerpo de `apply(x, isCheck)` debe ser puro respecto a `isCheck == true` (se llama potencialmente más de una vez por aplicación: una con `true` y otra con `false`).

**Comportamiento.** (1) `isDefinedAt` captura SÓLO `NoMatch` (cualquier otra excepción del cuerpo se propaga). (2) `apply` y `applyOrElse` capturan SÓLO `NoMatch`. (3) `noMatch()` devuelve siempre el mismo objeto `NoMatch` (case object) lanzable. (4) El uso típico `if (pf.isDefinedAt(x)) pf.apply(x)` ejecuta el cuerpo dos veces (con `isCheck` true y false), lo que permite evitar efectos secundarios en la comprobación. (5) `applyOrElse` ejecuta el cuerpo UNA sola vez (con `isCheck=false`), a diferencia de `isDefinedAt`+`apply`.

**Errores.** `apply(x)` sin coincidencia: `MatchError(x)`. Otras excepciones del usuario: propagadas tal cual (incluida `Exception` comprobada).

**Orden / concurrencia.** Sin estado; sujeto a lo que haga el cuerpo.

**Dependencias.** AC-JAPI-005.1 (misma unidad JavaAPI.scala), testkit javadsl (fuera de este módulo; frontera: pekko-testkit).

**Equivalente Rust propuesto.** `Fn(&A, bool) -> Result<B, NoMatch>` o, idiomáticamente, `Fn(&A) -> Option<B>`; las dos fases `isCheck` se sustituyen por el propio `Option`. Se preserva: distinguir "definido" de "no definido" y que `apply` sin coincidencia sea un error (`MatchError` como `Err`). Se pierde: la doble llamada (innecesaria con `Option`).

**Aserciones.**
- A1. Sin tests de Pekko sobre `JavaPartialFunction` en `actor-tests` (búsqueda exhaustiva: sólo aparece en JavaAPI.scala y testkit/javadsl/TestKit.scala) — *no afirmada por spec en este módulo*. La usan `TestKit.expectMsgPF`/`receiveWhile` del testkit Java, cuyos tests están en el módulo del testkit.
- A2. `isDefinedAt` = no lanza `NoMatch` con `isCheck=true`; `apply` convierte `NoMatch` en `MatchError`; `applyOrElse` usa `default` — *no afirmada por spec*; derivada de JavaAPI.scala:168-177. Medido con el jar (ver método en AC-JAPI-004.2): `isDefinedAt("a")` llama al cuerpo con `isCheck=true`; `apply("a")` con `false`; `applyOrElse` una sola vez con `false`; `apply(1)` sin coincidencia -> `scala.MatchError: 1 (of class java.lang.Integer)`. Caracterización propuesta para Rust: las mismas observaciones con un contador de llamadas.

**Validación prevista.** Ninguna si se excluye la fachada (el `Option` nativo cumple). Con fachada: A2 como test propio. Sin oráculo.

**Decisiones.** Q-JAPI-1. Efecto: sin fachada se cierra por equivalencia con `Option`. Fila nueva (omisión del inventario original: `JavaPartialFunction` no figuraba en ninguna fila).

**Estado.** analysis=extracted; impl=pending; validation=pending.

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

- AC-SER-013 — JavaSerializer habilitado y serialización Java de ActorRef: `serialization/Serializer.scala:306-362`, `actor/ActorRef.scala:434-468`, `SerializeSpec.scala:402-511` (el inventario solo cubría la variante deshabilitada, AC-SER-004).
- AC-SER-014 — `SerializationExtension` y su registro como library-extension: `serialization/SerializationExtension.scala:24-29`, `actor/ActorSystem.scala:1198-1223`, `reference.conf:78`, `SerializeSpec.scala:565-587` (SerializerDeadlockSpec).
- AC-IO-012 — `io/UnsynchronizedByteArrayInputStream.java` + `UnsynchronizedByteArrayInputStreamSpec.scala`.
- AC-IO-013 — `io/ByteBufferCleaner.java` + `ByteBufferCleanerSpec.scala`.
- AC-JAPI-010 — fábricas Java de IO (`TcpMessage` en `Tcp.scala:702-`, `UdpMessage` `Udp.scala:257-352`, `UdpConnectedMessage` `UdpConnected.scala:194-275`, `TcpSO`/`UdpSO`, `SoJavaFactories` `Inet.scala:187-219`).
- AC-JAPI-011 — `actor/AbstractScheduler.java`, `AbstractActorRef.java`, `pattern/AbstractPromiseActorRef.java`, `pattern/AbstractCircuitBreaker.java`, `annotation/*.java`.
- AC-JAPI-012 (cierre 2026-10-31) — `japi/pf/ReceiveBuilder.java` (297 líneas) y `ReceiveBuilderTest.java`: tenía sólo una mención en AC-JAPI-001 y es el único componente JAPI con semántica diferencial propia.
- AC-JAPI-013 (cierre 2026-10-31) — `JavaPartialFunction` (`japi/JavaAPI.scala:120-178`): no figuraba en ninguna fila.

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

Ninguna tras el cierre documental de 2026-10-31. Quedan pendientes de verificación con el oráculo (no bloquean `analysis=extracted`; constan como "no afirmada por spec" en cada bloque): AC-SER-007.1 A3-A4 (rama con `Information`; el factory Java de `withTransportInformation` es curried y no se pudo invocar desde jshell) y AC-IO-007 A6 (Q-IO-5). Ya medidos con el jar 1.7.1 el 2026-10-31: AC-JAPI-001 A8, AC-JAPI-012 A12, AC-JAPI-004.2 A2, AC-JAPI-013 A2, AC-JAPI-009.3 A7-A8, AC-SER-007.1 A1/A2/A5.

## Cobertura

Archivos del árbol de Pekko de esta área y las filas que los cubren.

**Serialización** (`actor/src/main/scala/org/apache/pekko/serialization/`): `AsyncSerializer.scala` -> AC-SER-006; `Decompression.scala` -> AC-SER-010; `NestedDeserialization.scala` -> AC-SER-009; `PrimitiveSerializers.scala` -> AC-SER-003.2/.3/.4; `Serialization.scala` -> AC-SER-001.1/.2/.3, AC-SER-004.2, AC-SER-007.1/.2; `SerializationExtension.scala` -> AC-SER-014; `SerializationSetup.scala` -> AC-SER-005; `Serializer.scala` -> AC-SER-002, AC-SER-003.1, AC-SER-004.1, AC-SER-013; `WireConfig.scala` -> AC-SER-011. Fuera de `serialization/`: `actor/dungeon/Dispatch.scala:34-39,172-224` -> AC-SER-008.1; `actor/dungeon/Children.scala:280-309` -> AC-SER-008.2; `actor/ActorRef.scala:434-468` -> AC-SER-013; `actor/ActorSystem.scala:1198-1223` -> AC-SER-014; `actor/Actor.scala:45` (`NoSerializationVerificationNeeded`) -> AC-SER-008.1/.2. `reference.conf`: `:78` AC-SER-014; `:129-140` AC-SER-008; `:761-797` AC-SER-004; `:780-785` AC-SER-009; `:799-848` AC-SER-003.5/AC-SER-001.2; `:852-863` AC-SER-010; `:865-880` AC-SER-012. Tests: `SerializeSpec`, `SerializationSetupSpec`, `DisabledJavaSerializerWarningSpec`, `PrimitivesSerializationSpec`, `AsyncSerializeSpec`, `DecompressionSpec`, `WireConfigSpec`, `NoVerification.scala` (clase auxiliar, sin comportamiento propio).

**IO** (`actor/src/main/scala/org/apache/pekko/io/`): `IO.scala` -> AC-IO-001; `Tcp.scala` -> AC-IO-002.1, AC-IO-003, AC-IO-005 (SO), AC-JAPI-010 (TcpMessage, TcpSO); `TcpManager.scala` -> AC-IO-004.1; `TcpListener.scala`, `TcpIncomingConnection.scala` -> AC-IO-002.2; `TcpOutgoingConnection.scala` -> AC-IO-002.3; `TcpConnection.scala` -> AC-IO-002.4/.5/.6/.7; `SelectionHandler.scala` -> AC-IO-003 (settings), AC-IO-004.1/.2/.3/.4; `Inet.scala` -> AC-IO-005; `Udp.scala` -> AC-IO-006.1/.2/.3; `UdpManager.scala` -> AC-IO-004.1; `UdpListener.scala` -> AC-IO-006.1; `UdpSender.scala`, `WithUdpSend.scala` -> AC-IO-006.2; `UdpConnected.scala`, `UdpConnectedManager.scala`, `UdpConnection.scala` -> AC-IO-007, AC-IO-004.1; `Dns.scala`, `DnsProvider.scala` -> AC-IO-008.1; `InetAddressDnsProvider.scala`, `SimpleDnsManager.scala` -> AC-IO-008.5; `InetAddressDnsResolver.scala` -> AC-IO-008.3; `SimpleDnsCache.scala` -> AC-IO-008.4; `DirectByteBufferPool.scala` -> AC-IO-010; Java: `UnsynchronizedByteArrayInputStream.java` -> AC-IO-012, `ByteBufferCleaner.java` -> AC-IO-013. `io/dns/`: `DnsProtocol.scala`, `DnsResourceRecords.scala`, `CachePolicy.scala`, `RecordType.scala`, `RecordClass.scala` -> AC-IO-008.2; `DnsSettings.scala` -> AC-IO-009.1; `IdGenerator.scala` -> AC-IO-009.7; `internal/AsyncDnsResolver.scala` -> AC-IO-009.2; `internal/DnsClient.scala` -> AC-IO-009.3; `internal/TcpDnsClient.scala` -> AC-IO-009.4; `internal/DnsMessage.scala`, `DomainName.scala`, `Question.scala`, `RecordTypeSerializer.scala`, `RecordClassSerializer.scala`, `package.scala` -> AC-IO-009.5; `internal/ResolvConfParser.scala` -> AC-IO-009.6; `internal/AsyncDnsManager.scala`, `AsyncDnsProvider.scala` -> AC-IO-009.8 / AC-IO-008.5. `reference.conf`: `:345-364` y `:930-938` AC-IO-011; `:940-1026` AC-IO-003; `:1028-1082` AC-IO-006.3; `:1084-1138` AC-IO-007; `:1140-1233` AC-IO-008.x / AC-IO-009.x. Tests: `TcpConnectionSpec`, `TcpIntegrationSpec`, `TcpIntegrationSpecSupport`, `TcpListenerSpec`, `CapacityLimitSpec`, `UdpIntegrationSpec`, `UdpConnectedIntegrationSpec`, `InetAddressDnsResolverSpec`, `SimpleDnsCacheSpec`, `UnsynchronizedByteArrayInputStreamSpec`, `ByteBufferCleanerSpec`, `dns/DnsSettingsSpec`, `dns/IdGeneratorSpec`, `dns/NameserverAddressParserSpec`, `dns/internal/*Spec` (AsyncDnsResolverSpec, DnsClientSpec, TcpDnsClientSpec, AsyncDnsManagerSpec, DomainNameSpec, MessageSpec, ResolvConfParserSpec). Sin leer: `dns/AsyncDnsResolverIntegrationSpec.scala`, `dns/DockerBindDnsService.scala` (requieren Docker).

**JAPI**: `actor/AbstractActor.scala` -> AC-JAPI-001/002; `actor/AbstractFSM.scala` y `japi/pf/FSM*Builder.java` -> AC-JAPI-003; `japi/pf/{Match,PFBuilder,AbstractMatch,AbstractPFBuilder}.java`, `pf/CaseStatements.scala` -> AC-JAPI-004.1; `UnitMatch.java`, `UnitPFBuilder.java` -> AC-JAPI-004.2; `FI.java` -> AC-JAPI-004.3; `DeciderBuilder.java` -> AC-JAPI-004.4; `japi/function/Function.scala`, `japi/JavaAPI.scala` -> AC-JAPI-005.1; `boilerplate/*.template` -> AC-JAPI-005.2; `japi/Throwables.scala` -> AC-JAPI-005.3; `japi/JAPI.java` -> AC-JAPI-005.4; `actor/AbstractProps.scala` -> AC-JAPI-006; `event/japi/EventBusJavaAPI.scala` -> AC-JAPI-007; `Scheduler.scala`, `ActorSelection.scala`, `ActorSystem.scala` sobrecargas -> AC-JAPI-008; `actor/TypedActor.scala` -> AC-JAPI-009.1/.2/.3; `actor/AbstractScheduler.java`, `AbstractActorRef.java`, `pattern/Abstract*.java`, `annotation/*.java` -> AC-JAPI-011. `dispatch/Abstract*.java`, `dispatch/affinity/` y `util/Unsafe.java`, `util/JavaVersion.scala`: pertenecen a MBX/DSP/UTIL y no son cubiertos por este grupo (sin comportamiento propio del módulo de IO/serialización/API Java; implementación interna de colas atómicas). Los archivos `annotation/ApiMayChange.java`, `DoNotInherit.java`, `InternalApi.java`, `InternalStableApi.java` son anotaciones sin comportamiento en tiempo de ejecución (solo metadatos de compilación). `actor/src/main/mima-filters` y `scala-2.13`/`scala-3` (compatibilidad de compilación) no definen comportamiento observable.

## Contradicciones y hallazgos del cierre documental (2026-10-31)

11. **AC-JAPI-006 A4** decía que los textos de validación de `Props.create` eran "derivada del código; no afirmada por spec": `ActorCreationTest.java:186-341` afirma tres de ellos con `assertEquals` exacto (closing-over, Creator erasado, clase abstracta). Corregido en el bloque.
12. **`ReceiveBuilder.build()` no resetea el builder** aunque su Javadoc (ReceiveBuilder.java:57-62) dice "After this call the builder will be reset"; `AbstractPFBuilder.build()` (usado por PFBuilder, UnitPFBuilder y los FSM*Builder) sí lo hace (AbstractPFBuilder.java:39-46). Contratado en AC-JAPI-012. El `FIXME` del propio código reconoce además que `statements.orElse(empty)` es redundante.
13. **`DeciderBuilder` no es `final`** (AC-JAPI-004.4 decía "clase final"; DeciderBuilder.java:20 es `public class` con constructor privado). Corregido.
14. **`UdpConnectedMessage.connect` usa `Some(localAddress)`** y `TcpMessage.connect` usa `Option(localAddress)`: con `null`, el primero produce `Some(null)` (AC-JAPI-010 A3).
15. **AC-JAPI-007 duplica AC-EVT-022** (ambas cubren `event/japi/EventBusJavaAPI.scala`); los únicos tests son ejemplos del módulo `docs` (`docs/src/test/java/jdocs/event/EventBusDocTest.java`), no de `actor-tests`. Se mantienen las dos filas con contrato idéntico por referencia; recomendación al coordinador: dejar una sola.
16. **Los tests de las fábricas Java de IO (AC-JAPI-010) no existen en `actor-tests`**; sólo hay ejemplos en `docs/src/test/java/jdocs/io/`. La igualdad fábrica = case class queda *no afirmada por spec*.
17. **AC-JAPI-001 `getReceiveTimeout()`**: con el valor por defecto `Duration.Undefined` la conversión `asJava` (`toNanos`) lanza `IllegalArgumentException: toNanos not allowed on infinite Durations` (medido con el jar 1.7.1).
18. **AC-JAPI-009.3 (hipótesis refutada por medición)**: se sospechó que `typedActorOf` descartaba el dispatcher de `TypedProps` porque sólo toma `props.actorProps().deploy` (TypedActor.scala:91,102). Medido con el jar 1.7.1: SÍ se aplica (el dispatcher de un `Props` vive en su `deploy`). No hay diferencia.
19. **Q-IO-5 (doConnect ignora la dirección resuelta) — CONFIRMADO con el jar 1.7.1**: `UdpConnection.doConnect(@unused address)` conecta con `channel.connect(remoteAddress)` (UdpConnection.scala:88), la dirección sin resolver, aunque `reportConnectFailure` le pase la resuelta. Medido: `UdpConnected.Connect` a `InetSocketAddress.createUnresolved("localhost", 40999)` -> `CommandFailed`; a una dirección ya resuelta -> `Connected`. Resuelve la incertidumbre de Q-IO-5; sólo queda la decisión del Owner de corregir o replicar.

## Revisión de exhaustividad (cierre documental 2026-10-31)

Clasificación del árbol de fuentes del grupo (`serialization/`, `io/`, `japi/`, clases base Java de `actor/src/main/java`, `AbstractActor*.scala`, `AbstractFSM.scala`, `TypedActor.scala`, `event/japi/`, y los tests Java de `actor-tests`).

Cubierto por contratos concretos:
- Serialización: todos los archivos de `serialization/` (ver "Cobertura"); `JSerializer` pasó a AC-JAPI-011 con referencia a AC-SER-002.
- IO: TCP, UDP, UDP conectado, DNS (AC-IO-001..013); costura `ChannelRegistry` y bucle de selección (AC-IO-004.4).
- JAPI: `AbstractActor` y familia (AC-JAPI-001/002), `ReceiveBuilder` (AC-JAPI-012), `AbstractFSM` y builders FSM (AC-JAPI-003), `japi.pf` (AC-JAPI-004.1-.4), `japi.function`/`JavaAPI.scala` (AC-JAPI-005.1-.4, AC-JAPI-013), `Props` Java (AC-JAPI-006), `TypedActor`/`TypedProps`/`TypedActorExtension` (AC-JAPI-009.1-.3), fábricas Java de IO (AC-JAPI-010).

Duplicado o fachada equivalente (con evidencia de que delega en la misma implementación):
- `AbstractActor.getContext()` es un cast de `context` y `ActorCell` implementa `AbstractActor.ActorContext` (ActorCell.scala:424-464): getters Java = alias de la celda (AC-JAPI-001). `UntypedAbstractActor`, `AbstractLoggingActor`, `AbstractActorWith*Stash`, `AbstractActorWithTimers` componen traits Scala (AC-JAPI-002).
- `AbstractFSM`: cada método reenvía a `FSM` (AbstractFSM.scala:86-549); los builders producen `PartialFunction` para esos métodos (AC-JAPI-003).
- `EventBusJavaAPI`: cada clase contiene un `bus` Scala anónimo y reenvía 1:1 (EventBusJavaAPI.scala:57-230) (AC-JAPI-007; duplicado de AC-EVT-022).
- Sobrecargas `java.time.Duration`/`Optional`/`CompletionStage`: conversión + delegación (AC-JAPI-008). `TcpMessage`/`UdpMessage`/`UdpConnectedMessage`/`TcpSO`/`UdpSO`/`SoJavaFactories`: constructores de case classes (AC-JAPI-010). `japi.function.*`, `Tuple3..22`, `Function3..22` (AC-JAPI-005.1/.2). `JAPI.java` (AC-JAPI-005.4). `FI.java` duplica `japi.function` (AC-JAPI-004.3).
- Comportamiento diferencial propio (no es sólo fachada): `ReceiveBuilder` (orden de alta, `isInstance`, `matchEquals(null)`, `build` sin reset, supplier externo) AC-JAPI-012; `PFBuilder`/`Match`/`UnitMatch` (isInstance, MatchError) AC-JAPI-004.1/.2; builders FSM (orden evento/dato/predicado, `null` comodín) AC-JAPI-003; `Props.create` (validaciones y mensajes) AC-JAPI-006; `JavaPartialFunction` (`isCheck`, `noMatch`) AC-JAPI-013; `japi.Option` (`none` singleton, `option(null)`) AC-JAPI-005.1; conversiones de `Duration` (desbordo a 292 años, `Undefined`) AC-JAPI-001/008; `UdpConnectedMessage.connect` con `Some(null)` AC-JAPI-010; `AbstractActor.getReceiveTimeout()` sin timeout AC-JAPI-001.

Detalle interno sin contrato independiente (justificado):
- `AbstractActorRef.java`, `AbstractActorCell.java`, `AbstractPromiseActorRef.java`, `AbstractCircuitBreaker.java`: offsets `Unsafe` de campos privados, sin API (AC-JAPI-011; ver AC-UTIL-020). `annotation/*.java`: metadatos con retención CLASS (AC-JAPI-011). `ChannelRegistryImpl`, `Task` y el `wakeUp` atómico: mecanismo del reactor NIO (AC-IO-004.4). `AbstractMatch`/`AbstractPFBuilder`/`CaseStatement`: clases auxiliares de AC-JAPI-004.1.

Fuera de estos módulos con frontera explícita:
- `CoordinatedShutdown` y sus sobrecargas Java (`addTask(Supplier<CompletionStage>)`, `run(reason, Optional)`, etc., CoordinatedShutdown.scala:577-801): AC-CS-* (otro grupo). `Scheduler` en sí (calendario, validaciones): AC-SCH-001..004; aquí sólo sus sobrecargas. `testkit/javadsl/TestKit.scala` (usa `JavaPartialFunction`): módulo pekko-testkit, fuera de pekko-actor. Tests de ejemplo en `docs/src/test/java/jdocs/{event,io}`: módulo `docs`; se citan como evidencia, no como tests de paridad.
- `dispatch/Abstract*.java`, `dispatch/affinity/`, `util/Unsafe.java`: MBX/DSP/UTIL.

Pendiente de análisis (NO fuera de alcance; asignar a los grupos indicados, no revisado en este grupo):
- Tests Java de `actor-tests` sin cita en el inventario: `CircuitBreakerTest.java` y `AbstractCircuitBreaker` (grupo CB), `StatusReplyTest.java` (ASK/StatusReply), `ActorWithMDC.java` (EVT, MDC), `setup/ActorSystemSetupTest.java` (SYS/CFG, Setup). Sin leer en este grupo: `LoggingAdapterTest.java`, `PatternsTest.java`, `CompletionStagesTests.java`, `JavaFutureTests.java`, `AddressTest.java`, `JavaExtension.java`, `ByteStringTest.java`, `ConstantFunJavaTest.java`, `JavaConverterScala3InlineTest.java`: están citados por filas de otros grupos (EVT, ASK, DSP, PATH, EXT, UTIL); el coordinador debe confirmar que cada uno se lee allí.
- `JavaAPI.java:46-63` (`mustCompile`: `Kill.getInstance()`, `PoisonPill.getInstance()`, `ReceiveTimeout.getInstance()`, `LocalScope.getInstance()`, `NoScopeGiven.getInstance()`, `Logging.loggerInitialized()`, `GetRoutees.getInstance()`, `NoRouter.getInstance()`, `FromConfig.getInstance()`, `ActorPaths.fromString`): fachadas Java de singletons (`getInstance`) y de `ActorPaths`; son compilación, pero los objetos están contratados en sus filas (AC-CELL/AC-RT/AC-PATH). Pendiente: confirmar que `ActorPaths` (clase estática Java) tiene fila en PATH; no verificado.
- `Props` Java API completa (más allá de `create`) y `ActorSystem.create` estáticos (ActorSystem.scala:205-211): fachadas pendientes de contrastar con AC-REF-007/AC-SYS.
