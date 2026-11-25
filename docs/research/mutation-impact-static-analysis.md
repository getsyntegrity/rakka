# Static impact analysis for mutation selection (research, closed)

Status: completed research increment, 2026-11-25. **No selector is implemented and no gate changed.** Nothing here is proven safe: the models were compared by counting, not validated against a full run (that is what a shadow-mode run would do).

## Question
For each of the 13 open PRs of the mailbox-compat stack (#362-#374) and the current inventory of 333 mutants (248 replay, 78 unit, 7 tool; 59 mutated source files), how many mutants would a selection by impact still have to run?

## Method (`tools/research/impact_static_analysis.py`, read-only, 7 s, no mutant executed)
- Changed files per PR from `gh pr view --json files` (three-dot, relative to the PR's own base).
- Dependency graph of `crates/rakka-actor/src` (98 files): `use crate::`/`super::`/`self::`/`mod` edges (391), CamelCase type-name token edges (501; 78 names are defined in several files and add edges to all), `macro_rules!` invocation edges and `include_str!` data edges.
- Table-only detection for `tools/parity/check_divergence.py`: AST equality of everything except the mutant tables.
- Models: **A** mutants in directly modified files; **B** A plus transitive dependencies (forward, reverse, both; also the `use crate::` graph alone); **C** conservative semantic impact with full-suite fallback (oracle, Cargo, harness logic, mutation infrastructure, `cfg` -> all 333; examples -> all replay mutants; corpus -> all replay mutants; table-only harness edits -> only the new or changed mutants; documentation and evidence outputs -> none); **C\*** C plus the unproven lemma that adding recorded cases cannot un-detect a mutant.
- New or edited mutants are always selected. Cost estimate: 112.3 s per mutant (the only measured value: the 12-mutant cold benchmark); the savings are **estimated, not measured**.

## Result (selected mutants of 333 per PR)
| PR | A | B (use graph) | B | C | note |
|---|---|---|---|---|---|
| #362 | 22 | 325 | 325 | 326 | source |
| #363 | 85 | 325 | 325 | 325 | source |
| #364, #365 | 3, 2 | 3, 2 | 3, 2 | 333 | oracle + corpus: A ignores them (unsafe) |
| #366 | 102 | 325 | 325 | 333 | oracle |
| #369 | 56 | 325 | 325 | 326 | source |
| #367 | 42 | 325 | 325 | 333 | oracle + harness logic |
| #370 | 16 | 325 | 325 | 325 | source |
| #371 | 32 | 325 | 325 | 326 | source |
| #368 | 0 | 0 | 0 | 0 | documentation only |
| #372 | 2 | 2 | 2 | 2 | documentation, table-only harness edit |
| #373, #374 | 0 | 0 | 0 | 333 | mutation infrastructure |

Over the 13 PRs: A selects 362 of 4,329 (91.6 % fewer) but ignores every non-source change; B selects 320-325 of 333 for any PR that touches crate source; C selects 3,628 of 4,329 (16.2 % fewer), all of the saving coming from the two documentation PRs.

## Findings and limits
1. The static graph collapses: `cell`, `props`, `mailboxes` and `system` are hubs, so one changed source file reaches 320-325 mutated files in `use crate::` terms alone.
2. A is unsafe: it cannot express oracle, corpus, harness or infrastructure changes (up to 15 changed files per PR outside any mutated file).
3. Ambiguity that a static model cannot resolve: 61 files with trait impls, 51 with glob imports, 29 with `as` renames, 23 with `dyn`, 78 type names defined in several files. The governing invariant (uncertainty widens the scope, never narrows it) therefore leaves little to exclude.
4. The graph was built on the tip of the stack and applied to every PR; per-PR historical graphs were not rebuilt.
5. The mutant inventory has no mutant of `dispatch/dispatcher.rs`.

Conclusion: do not build a general static selector. Reduction has to come from dynamic information (which scenarios exercise a mutated region), see the dynamic experiment.
