# Spec 21 — AC-SER (1/1)

**Estado:** pendiente de decisiones propuestas. **Filas:** 23. **Tareas:** 4 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-classic.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 03 `AC-CFG-1` (`03-ac-cfg-1.md`), por las filas AC-CFG-006, AC-CFG-010, AC-CFG-015, AC-CFG-016, AC-CFG-017.1, AC-CFG-017.2, AC-CFG-017.3, AC-CFG-017.4, AC-CFG-017.5, AC-CFG-017.6, AC-CFG-017.7.
- Spec 01 `AC-PATH-1` (`01-ac-path-1.md`), por las filas AC-PATH-010.
- Spec 04 `AC-REF-1` (`04-ac-ref-1.md`), por las filas AC-REF-007.1, AC-REF-007.2, AC-REF-023.
- Spec 13 `AC-SYS-1` (`13-ac-sys-1.md`), por las filas AC-SYS-018.
- Spec 23 `AC-IO-2` (`23-ac-io-2.md`), por las filas AC-IO-012. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 16 `AC-EXT-1` (`16-ac-ext-1.md`), por las filas AC-EXT-001.1, AC-EXT-001.2.

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-SER-2`, `Q-SER-3`, `Q-SER-4`, `Q-SER-5`, `Q-SER-6`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

casos rust-anything contra Pekko 2.0.0 (JDK 17) con `oracle/` y `tools/parity/` (decisión Q-X-12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: Resolución de serializador por clase: serializerFor/findSerializerFor, bindings… (+5 filas)

Filas: AC-SER-001.1, AC-SER-001.2, AC-SER-001.3, AC-SER-002, AC-SER-003.1, AC-SER-003.2. Contratos y aserciones: `docs/inventory/contracts/classic-io-ser.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: StringSerializer (id 20) y ByteStringSerializer (id 21) (+5 filas)

Filas: AC-SER-003.3, AC-SER-003.4, AC-SER-003.5, AC-SER-004.1, AC-SER-004.2, AC-SER-005. Contratos y aserciones: `docs/inventory/contracts/classic-io-ser.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 3: AsyncSerializer, AsyncSerializerWithStringManifest y AsyncSerializerWithStringManifestCS (+5 filas)

Filas: AC-SER-006, AC-SER-007.1, AC-SER-007.2, AC-SER-008.1, AC-SER-008.2, AC-SER-009. Contratos y aserciones: `docs/inventory/contracts/classic-io-ser.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 4: Límite de descompresión gzip (pekko.serialization.max-decompressed-size) y… (+4 filas)

Filas: AC-SER-010, AC-SER-011, AC-SER-012, AC-SER-013, AC-SER-014. Contratos y aserciones: `docs/inventory/contracts/classic-io-ser.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AC-SER-001.1 | Resolución de serializador por clase: serializerFor/findSerializerFor,… | `SerializerRegistry::serializer_for(TypeKey) -> Result<Arc<dyn Serializer>,… | actor-tests/src/test/scala/org/apache/pekko/serialization/SerializeSpec.scala:206-258 | Q-SER-1, Q-X-2, Q-X-6 |
| AC-SER-001.2 | Carga de serializers y bindings desde config y SerializationSetup: alias ->… | `SerializerRegistryBuilder::from_config(&Config, &SerializationSetup, &FactoryRegistry)` | actor-tests/src/test/scala/org/apache/pekko/serialization/SerializeSpec.scala:298-319 ;… | Q-SER-1, Q-X-2 |
| AC-SER-001.3 | Serialization.serialize/deserialize: por id+manifest, por clase y por… | `Serialization::{serialize, deserialize, deserialize_by_type, deserialize_buffer}` | actor-tests/src/test/scala/org/apache/pekko/serialization/SerializationSetupSpec.scala:10… | Q-SER-1, Q-SER-3 |
| AC-SER-002 | SPI Serializer (identifier, toBinary, fromBinary, includeManifest),… | `trait Serializer { fn identifier(&self)->i32; fn to_binary(&self,&dyn… | actor-tests/src/test/scala/org/apache/pekko/serialization/SerializeSpec.scala:260-290 ;… | Q-SER-1, Q-X-3 |
| AC-SER-003.1 | NullSerializer (id 0) y ByteArraySerializer (id 4, binding "[B") | `NullSerializer`, `BytesSerializer` | actor-tests/src/test/scala/org/apache/pekko/serialization/SerializeSpec.scala:260-290,401… | - |
| AC-SER-003.2 | LongSerializer (id 18) e IntSerializer (id 19): formato little-endian de ancho… | `I64Serializer`, `I32Serializer` | actor-tests/src/test/scala/org/apache/pekko/serialization/PrimitivesSerializationSpec.sca… | Q-SER-1 |
| AC-SER-003.3 | StringSerializer (id 20) y ByteStringSerializer (id 21) | `StringSerializer`, `ByteStringSerializer` | actor-tests/src/test/scala/org/apache/pekko/serialization/PrimitivesSerializationSpec.sca… | Q-SER-1, Q-X-3 |
| AC-SER-003.4 | BooleanSerializer (id 35): 1 byte, 0=false 1=true, rechaza otros valores | `BoolSerializer` | actor-tests/src/test/scala/org/apache/pekko/serialization/PrimitivesSerializationSpec.sca… | - |
| AC-SER-003.5 | Tabla por defecto de reference.conf: serializers, serialization-bindings y… | `default_serialization_table() -> SerializationSettings` | actor-tests/src/test/scala/org/apache/pekko/serialization/SerializeSpec.scala:389-426 | Q-SER-1, Q-SER-2 |
| AC-SER-004.1 | DisabledJavaSerializer: reemplaza JavaSerializer con… | `DisabledJavaSerializer` | actor-tests/src/test/scala/org/apache/pekko/serialization/DisabledJavaSerializerWarningSp… | Q-SER-2 |
| AC-SER-004.2 | allow-java-serialization, warn-about-java-serializer-usage,… | `JavaSerializerWarnings` | actor-tests/src/test/scala/org/apache/pekko/serialization/SerializeSpec.scala:475-485,542… | Q-SER-2, Q-X-7 |
| AC-SER-005 | SerializationSetup/SerializerDetails (registro programático de serializadores… | `SerializationSetup::new(\|sys\| vec![SerializerDetails::new(alias, serializer, use_for)])` | actor-tests/src/test/scala/org/apache/pekko/serialization/SerializationSetupSpec.scala:11… | Q-X-2 |
| AC-SER-006 | AsyncSerializer, AsyncSerializerWithStringManifest y… | `trait AsyncSerializer { async fn to_binary_async; async fn from_binary_async }` | actor-tests/src/test/scala/org/apache/pekko/serialization/AsyncSerializeSpec.scala:114-140 | Q-SER-5, Q-X-4 |
| AC-SER-007.1 | Serialization.serializedActorPath(actorRef): formato con la dirección de… | `serialized_actor_path(&ActorRef) -> String` | actor-tests/src/test/scala/org/apache/pekko/actor/ActorPathSpec.scala:162-185 (solo… | Q-SER-3 |
| AC-SER-007.2 | Información de transporte (Serialization.Information,… | `TransportInformation { address, system }` + `with_transport_information(system, f)` | derivada del código (ver bloque de contrato) | Q-SER-3 |
| AC-SER-008.1 | pekko.actor.serialize-messages: verifica que cada mensaje sobreviva a… | `MessageSerializationCheck` | actor-tests/src/test/scala/org/apache/pekko/actor/dungeon/DispatchSpec.scala:21-43 ;… | Q-SER-4 |
| AC-SER-008.2 | pekko.actor.serialize-creators: verifica argumentos de Props al crear un actor | `CreatorSerializationCheck` | actor-tests/src/test/scala/org/apache/pekko/serialization/SerializeSpec.scala:341-362 | Q-SER-4 |
| AC-SER-009 | Límite de profundidad de deserialización anidada… | `NestedDeserializationGuard::at_next_level(max, \|\| ..)` | derivada del código (ver bloque de contrato) | - |
| AC-SER-010 | Límite de descompresión gzip (pekko.serialization.max-decompressed-size) y… | `gunzip_limited(bytes, max: Option<u64>)` | actor-tests/src/test/scala/org/apache/pekko/serialization/DecompressionSpec.scala:41-102 | Q-SER-6, Q-X-1 |
| AC-SER-011 | WireConfig.parseString: HOCON de origen remoto sin resolver include | `WireConfigParser::parse_str` (modo sin includes del parser HOCON propio) | actor-tests/src/test/scala/org/apache/pekko/serialization/WireConfigSpec.scala:44-85 | Q-X-1 |
| AC-SER-012 | pekko.serialization.protobuf.allowed-classes / whitelist-class (solo… | n/a: claves aceptadas y no usadas | derivada del código (ver bloque de contrato) | Q-SER-6 |
| AC-SER-013 | JavaSerializer habilitado: toBinary/fromBinary con ObjectStream y… | n/a (ver Q-SER-2) | actor-tests/src/test/scala/org/apache/pekko/serialization/SerializeSpec.scala:428-540 ;… | Q-SER-2 |
| AC-SER-014 | SerializationExtension: registro como library-extension, acceso y prohibición… | `SerializationExtension` (ExtensionId) | actor-tests/src/test/scala/org/apache/pekko/serialization/SerializeSpec.scala:594-616 | Q-SER-6, Q-X-4 |

## Gates y criterio de cierre

```bash
cargo fmt --all --check
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo llvm-cov --workspace --lcov --output-path target/lcov.info   # cobertura de líneas nuevas o modificadas >= 90 %; reportar comando, resultado y líneas no cubiertas con motivo
```

La spec se cierra únicamente cuando: (1) todos los gates pasan; (2) cada aserción `A#` de cada fila tiene un test que se observó fallar y luego pasar; (3) la cobertura de líneas nuevas o modificadas es >= 90 % y está documentada; (4) las columnas `impl` y `validation` del inventario reflejan lo demostrado (`partial` si queda una decisión abierta o una diferencia); (5) las diferencias observables respecto de Pekko están anotadas con su `Q-`. Tests verdes no equivalen a paridad.

## Siguiente

Spec 22 `AC-IO-1` (`22-ac-io-1.md`).
