# P10 corrección D2, D3, D4 del render conciso (revisión independiente de #290)

Filas: AC-CFG-013. **Sigue `partial`.** Corrige solo D2, D3 y D4; D1, D5 y D6 siguen abiertas (abajo).

## RED (observado)
`tools/parity/fuzz_config_render.py --seed 1 --count 3000` (oráculo `config-render-lines` = `Deploy.toString` de Pekko 2.0.0 contra el gemelo Rust): **1939 de 2549 entradas divergían** antes de cambiar el código (texto original de escalares, fusión de listas y objetos, citas en `${...}`).

## GREEN
- D2: una racha de dos o más valores simples y espacios se imprime como un solo string con los textos originales (`1.50`, `1e2`, `true`, `null`); un valor simple solo entre sustituciones conserva su tipo (`${x}7${y}`, el `007` se imprime como número `7`).
- D3: los espacios junto a una lista o un objeto se descartan; listas contiguas se concatenan y objetos contiguos se fusionan (gana el de la derecha).
- D4: los elementos de una ruta `${...}` se imprimen sin comillas salvo que estén vacíos o tengan un carácter que no sea letra, dígito, `-` o `_` (unidades UTF-16: una letra astral se cita). Solo cambia `Subst::display`; `join_path` y los mensajes de error no se tocan.
- Corpus: `TS-36..TS-58` (23 casos nuevos, `dp-0080..dp-0102`) grabados con el oráculo; los 102 casos coinciden. Replay completo en la descripción del PR.
- Fuzz diferencial con semillas conservadas (1 a 5, 3000 a 6000 entradas, 25 000 en total): **0 diferencias**. Las entradas se regeneran con la misma semilla; no hubo entradas divergentes que conservar tras el arreglo (la lista del RED se obtiene ejecutando el fuzzer en el commit anterior).
- Casos del revisor (`/tmp/adv`, 91 de una línea): coinciden salvo tres de D5 (`a=[1] x`, `a=1 [2] ${x}`, `a=${"a\"b"}`) y cuatro que son artefactos de su sonda (duplica barras invertidas al escapar).

## Mutantes nuevos (8)
Los 8 DETECTED (casos que divergen entre paréntesis), control CLEAN 2944/2944: concatenation-drops-the-text-of-null (1), concatenation-number-loses-trailing-zeros (2), concatenation-keeps-white-space-next-to-containers (7), concatenation-does-not-fuse-adjacent-lists (2), concatenation-fuses-objects-in-the-wrong-order (1), concatenation-quotes-a-lone-plain-value (3), substitution-path-quotes-digit-elements (4), substitution-path-leaves-astral-elements-bare (1). Replay completo sin mutar: 2944/2944, 1 NA. Cobertura del crate 98,39 % (`config/mod.rs` 96,1 %, `config/model.rs` 97,9 %). fmt, clippy, deny, `cargo test --workspace` y `check.py` OK. El caso `TS-58` (letra astral) existe porque sin él `substitution-path-leaves-astral-elements-bare` era equivalente (un emoji no es letra).

## Divergencias abiertas
- **D1** desempate de claves de igual valor numérico (`"7"`, `"007"`): depende del orden de `HashMap` de Java (PR-D).
- **D5** parser: `${"a\"b"}`, `a=[1] x` y `a=1 [2] ${x}` no dan el mismo resultado.
- **D6** surrogate suelto en un valor (Q-X-17).
- Letras y dígitos de la regla D4 se aproximan con `char::is_alphabetic` / `is_numeric` frente a `Character.isLetterOrDigit`: puede diferir en categorías raras (`No`, `Nl`); el fuzz usa `é` y un carácter astral, no toda la tabla.
No se declara paridad completa de `render_concise` mientras estén abiertas.

## Veredicto independiente y hallazgos abiertos (HEAD 483ee6c: APPROVE para D2, D3 y D4)

El revisor repitió el corpus (102/102 idénticos), los 91 casos del revisor anterior, 391 entradas propias y unas 13 000 generadas, y tres semillas nuevas del fuzzer (101, 202, 303: 0 diferencias). Reejecutó 3 de los 8 mutantes (2 DETECTED con control CLEAN; el tercero lo detuvo por tiempo). No verificó cobertura, gates ni check.py.

Abiertas, sin corregir y no aceptadas:
- **D4b** (en alcance, baja): la regla de letra o dígito de `${...}` usa `is_alphabetic`/`is_numeric` y deja sin comillas elementos que Java cita: `²`, `Ⅷ`, `①`, `½`, `aͅ`, `कि`, `ः` (Nl, No y Other_Alphabetic). Arreglo: comprobar la categoría general L* o Nd; requiere una tabla de categorías (decisión de dependencia pendiente).
- **D5 ampliada** (parser): escalar mezclado con lista u objeto (`a=[1] x`, `a={b=1} x y`: Pekko descarta el escalar), lista con objeto (`a=${1.2}[]{}` da WrongType en Pekko), `a{b=1} {c=2}` y `a {} ${x}` (Pekko da error de parseo), tabulador literal dentro de comillas (error en Pekko), U+FEFF dentro de texto sin comillas y en claves.
- **D7** (apilado de capas): una clave cuyo valor posterior es una lista con una sustitución sin resolver conserva la capa anterior en Pekko y Rust la descarta (`x=${?x}, x=[${?x}]`, `x={k=1}, x=[${?x}]`). Es lógica de fusión anterior a este PR.
- El fuzzer es estrecho (sin mezclas escalar/contenedor, sin BOM, sin `+=`, sin capas); el harness es de una línea por entrada, así que no prueba saltos de línea reales dentro de listas u objetos.
