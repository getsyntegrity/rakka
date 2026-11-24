# Selección de mailbox por Deploy y configuración (Slice 2, PR2a)

Base `c4c6cbb`, Pekko 2.0.0 `9fbd2283`. `AC-MBX-014` sigue `partial`.

Contrato (Scala): `ActorRefProvider.actorOf:692-701` aplica `props.withMailbox(d.mailbox)` si el `Deploy` hallado por `deployer.lookup(path)` tiene mailbox no vacío (Deploy/configuración pisa a Props; vacío deja a Props). `Children.makeChild:321-328` busca con `lookupDeploy = true` y el path del hijo; `Deployer.lookup` usa `elements.drop(1)`. El error nombra el id efectivo (`props2.mailbox`, `:724`).

Implementado: `Settings.deployer`, `Settings::from_config` (lee `pekko.actor.deployment`, ahora falible), `Props::deployed_over` y su uso en `attach_child`. Fuera: dispatcher del Deploy, routers, requirements.

Evidencia: `red-deploy-mailbox.txt` / `green-deploy-mailbox.txt`. Mutantes `deployment-mailbox-ignored`, `deployment-empty-mailbox-resets-the-props`, `props-mailbox-beats-the-deployment` DETECTED por el unitario esperado; el de path equivocado (`lookup_path(&self.path)`) se comprobó a mano: 4 integrales fallan.

Pendiente (PR2b, depende de #350): casos de oráculo contra Pekko 2.0.0 para Deploy y configuración (`MBX-CFG-04..`).
