Entry for `docs/actor-core-parity.md`, kept here so the shared file is not edited by this slice.

- Update (ACT-015, #161, ADR 0015 I-2): the unexported Ask reply endpoint and target validation exist in `pekko-actor`
  (Go tests only). 🟡 Ask identity, isolation, single terminal transition and non-registration: Go tests pass, no Pekko oracle
  was run and the Go judge has no Ask scenario yet, so no parity is claimed. 🔴 The exported `Ask`, deadline, cancellation,
  `ErrRecipientTerminated`, shutdown (AK17) and the Pekko cases S03, S04, S12, S15, S17 remain pending (#162 to #166 and later).
  Details and the open Owner decision on `Unwatch`: `migration/stage-1/ask-endpoint-port/SCOPE.md`.
