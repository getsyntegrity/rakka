# Entries for shared files (for the coordinator; this PR does not edit them)

## migration/cost-log.tsv

No row. The kit was not executed for TEN-005 (no port). A row would invent a step, tokens and wall-clock that were not
measured.

## Indexes / milestone / global matrices

- ADR index (if any): `0016 | Tenant identity and isolation model | proposed | Stage 1 | docs/adr/0016-tenant-identity-and-isolation-model.md | #393`.
- Milestone / matrices: TEN-005 nature = own extension (design); no Pekko source row; Pekko-parity column "not applicable, Pekko has no tenancy".
- Closure checklist of #393: ADR with layer matrix (done in PR); links from TEN-001 #205, TEN-003 #207, TEN-006 #394, PER-001 #211 and PER-007 #217 (pending); review and CI on the final SHA (see PR).
- Cross-ADR notes: ADR 0017 (#350) may cite ADR 0016 only as a pending proposal. ADR 0016 cites 0017 only as pending and adds nothing to ADR 0015.
