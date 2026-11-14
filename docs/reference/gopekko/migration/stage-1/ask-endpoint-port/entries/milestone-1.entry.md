## Ask increment I-2 (ACT-015, #161): scenario registration

Entry for `migration/stage-1/milestone-1.md`, kept here so the shared file is not edited by this slice.

ADR 0015 scenario ids are proposed ids. This slice implements the ones that need only the unexported endpoint, as in-package tests
(`pekko-actor/actor/ask_endpoint_internal_test.go`). They are **not** judge scenarios: the judge drives exported behavior, and the
first Ask judge scenario arrives with the exported `Ask` (#162). `judge/go-scope.json` is unchanged on purpose.

| Id | Condition | State |
|---|---|---|
| AK20 | invalid target (nil, typed nil, non-native, unregistered cell, ID coincidence): `ErrInvalidHandle` | Go test passes |
| AK21 | native ref of another System: accepted and owned by its own System (no origin identity, D6) | Go test passes |
| AK23 | endpoint vs Watch, WatchWith, Unwatch, Stop, Context.Stop, Done (D7/D8) | Go test passes; `Unwatch` pinned to the code, ADR text differs (Owner decision pending) |
| (D8) | unique `ask-<n>` identity, no `byID`/name entry, concurrent requests do not cross, first reply wins | Go test passes |
| AK01, AK02, AK03, AK05, AK08, AK12, AK13, AK15, AK22 | need the exported `Ask` | pending #162 to #165 |
| AK04, AK06, AK07, AK09, AK14, AK16, AK18, AK19a/b | later increments | pending |
| AK17 | needs `System.Shutdown` | blocked (D7) |
