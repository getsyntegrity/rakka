# U5b resume point (informational, not part of the approval hashes)

State: the **design** (`../RULEBOOK.md`, D1-D12, three deliveries) is approved (`981a6ea`, PR #462) and U5a is merged (`c87fb0e`, #463). The **execution package of U5b** (RUNBOOK, manifest, permissions, launcher, implementer prompt) is prepared on `develop` at `c87fb0e` and waits for the Owner's approval of its hashes. **Execution is NOT authorized**: no implementation agent has run and `pekko-actor/` is unchanged. U5c has no package and is not authorized.

Budget of this delivery: maximum 560,000 tokens, evaluation stop 450,000, 60 minutes, review limited to 100,000 tokens, one focused review, no native review. Coordination is charged by rule (90,000 reserve, +30,000 per extra round). The U5 aggregate is **not** authorized.

Hashes (sha256) to approve:
- `../RULEBOOK.md` 5991232cdd05e49f7ee6a2f2f7f1ae4016a9f878fbc76a6f3953e9aa7bc25209
- `settings.template.json` e70f26508063b3bdb9e947fcedfaedcd59605301d5c091567bd04bcdc72319fa
- `run-agent.sh` 8ba870f0bf76926a35dabcb9bdb558de8d0302591779c661b3b4e6ff5db5d19c
- `manifest.tsv` 4d9649a274b7f1ff7292fd7eab8177c7cd001d8f93f1d3a9bd918a6da5ee39ed
- `implementer-prompt.md` 8ff17f0f5e2d5c6e8c20221387b93cbf3ceba815c51ab38a936ee09e26f35dc1
- `RUNBOOK.md` 7eed715885fb6d8ea6657d556ff8b9018b43c09c17cd6ccad63bdbba3a7f2fca

To run, after approval: merge the package PR into `develop`, create `feat/actor-supervision-u5b` from it, follow RUNBOOK section 5 and run the command of section 9. Do not re-map or re-plan.

Not done and not claimed: the implementation, U5c, ADR 0014 amendment (a) (applied with U5b by the coordinator), the RULEBOOK 3.3.1 text for the accepted U5a deviation, module close, parity.
