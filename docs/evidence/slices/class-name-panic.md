# `Actor::class_name` que lanza y `Props` cuyo `Drop` lanza (AC-CELL-027, huecos de Q-CELL-9)

Base: `origin/feat/pekko-rust-pilot-migration` @ c9882e9b.

## Defectos

1. `install_actor` llamaba a `Actor::class_name` (método público y sobrescribible) con el lock de estado de la celda tomado y sin
   `catch_unwind`. Un panic envenenaba el lock, tanto al crear como al reiniciar. En Pekko la clase es `getClass`
   (`ActorCell.scala:752`), que no ejecuta código del usuario.
2. `new_local` soltaba los `Props` sin protección cuando fallaba el mailbox: si eran los últimos y su `Drop` lanzaba, el nombre
   del hijo quedaba reservado.

## RED (antes del código de producción)

`cargo test -p rakka-actor --test classic_class_name_panic --test classic_mailbox_spi_failure --no-fail-fast`

```text
test a_restart_into_an_instance_whose_class_name_panics_keeps_the_actor_and_its_child ... FAILED
test an_actor_whose_class_name_panics_works_and_is_reported_by_its_type ... FAILED
  classic_class_name_panic.rs:19: left: Err(Timeout)  right: Ok("restarted")
  cell.rs:1699: called `Result::unwrap()` on an `Err` value: PoisonError { .. }
  cell.rs:518:  called `Result::unwrap()` on an `Err` value: PoisonError { .. }
  classic_class_name_panic.rs:19: left: Err(Timeout)  right: Ok("m")
test result: FAILED. 0 passed; 2 failed

test props_whose_drop_panics_do_not_keep_the_name_of_a_failed_creation ... FAILED
  classic_mailbox_spi_failure.rs:174: drop
  common/mod.rs:18: timed out waiting for the parent to report
test result: FAILED. 7 passed; 1 failed
```

## GREEN

Mismo comando: `2 passed; 0 failed` y `8 passed; 0 failed`.

## Segunda ronda (revisión de #460)

3. Un despliegue con mailbox reemplazaba el `MailboxType` del usuario soltándolo sin protección, con el nombre ya reservado.

RED: `test a_mailbox_type_replaced_by_a_deployment_may_panic_on_drop ... FAILED`
(`classic_mailbox_spi_failure.rs:176: drop`; `common/mod.rs:18: timed out waiting for the child to receive`; `8 passed; 1 failed`).
GREEN: `2 passed` y `9 passed`, `0 failed`. La aserción añadida sobre la etiqueta tras el restart pasó a la primera (conducta ya
implementada): no es un RED. Su mutante, `replaced-mailbox-type-is-dropped-unguarded`, está registrado y no ejecutado (Q-X-23).

## Alcance

Son integraciones (sistema, celda, mailbox y supervisión reales; sin dobles de lo que se valida). El plazo acotado es de 10 s.
Los `AssertUnwindSafe` justificados son los de la frontera de creación y `with_actor`; no tienen test propio.

Mutantes registrados y no ejecutados (Q-X-23): `class-name-is-asked-unguarded`, `failed-creation-drops-props-unguarded` y el de la segunda ronda.
Lo que sigue pendiente de Q-CELL-9 está en las notas de AC-CELL-027 (`docs/inventory/actor-classic.tsv`).
