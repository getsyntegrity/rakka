# U5c resume point (informational, not part of the approval hashes)

State: the **design** (`../RULEBOOK.md`) is approved (`981a6ea`); U5a (#463) and U5b (#465, squash `eeac94d`) are merged. The **execution package of U5c** is prepared on `develop` at `eeac94d` and waits for the Owner's approval of its hashes. **Execution is NOT authorized**: no agent has run and `pekko-actor/` is unchanged. #445 stays open.

Budget: maximum 560,000 tokens, evaluation stop 450,000, 45 minutes, one focused review with a hard limit of 100,000, no native review. Coordination by rule (90,000 reserve, +30,000 per extra round).

Hashes (sha256) to approve:
- `../RULEBOOK.md` 5991232cdd05e49f7ee6a2f2f7f1ae4016a9f878fbc76a6f3953e9aa7bc25209
- `settings.template.json` 5a6f72dfab80c223a5e003c4749ee5074cb16a10c83df70d4e54ef29e73bf53b
- `run-agent.sh` 85ce28a38d5cf569c8a8f64df53725eeac7b1fc751fef923a7ab8d01fd203bab
- `manifest.tsv` 137dc5d308e0847bdedf9d725ba3143d6f2d57fee2b5f27a1fef6b1c40860800
- `implementer-prompt.md` f1719e7b8fd5eb9f8f7a73ad39016115e1a6e01e62f3f08af6f4c9f13667894e
- `RUNBOOK.md` ddb4fafc278074dac1099992970e452f7119bd08c44a3386ae6f498d117fab12

To run, after approval: merge the package PR into `develop`, create `feat/actor-supervision-u5c`, follow RUNBOOK section 5 and run the command of section 8. Do not re-map or re-plan.

Not done and not claimed: the implementation, the module close (#461), `System.Shutdown`, parity.
