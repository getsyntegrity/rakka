#!/usr/bin/env python3
"""The detection ladder of a replay mutant: known killers, then the families its file is affine to, then the full replay.

A divergence of ONE applicable recorded case, demonstrated against the clean control, is a real detection, so a known killer or a family
replay may prove DETECTED. The converse is not true: only the complete applicable corpus can show that a mutant survives. The ladder
therefore never answers "undetected": when it finds nothing it hands over to the full replay. A timeout or a harness error of a step is
never a detection and never falls through silently (the caller reports it).

The steps are injected (`kill_known`, `kill_family`, `learn`), so the policy is tested without a replay; check_divergence wires the real ones.

Derived-from: none (project tooling, Apache-2.0).
"""
import collections

KNOWN_KILLER, FAMILY, FULL_REPLAY, TIMED_OUT, HARNESS_ERROR = "known-killer", "family", "full-replay", "timeout", "harness-error"
MAX_FAMILIES = 3          # families tried before the full replay: bounds the cost of a wrong guess
CANDIDATE_LIMIT = 8       # divergent cases of a family that are confirmed one by one (same limit as the fast-kill candidates)

LadderResult = collections.namedtuple("LadderResult", "path killers detail")


class StepFailure(Exception):
    """A step could not give an answer: kind is "timeout" or "harness"."""

    def __init__(self, kind, message=""):
        super().__init__(message or kind)
        self.kind = kind


def run_ladder(known, families, kill_known, kill_family, learn):
    """-> LadderResult. `known`: killer cases recorded earlier; `families`: families in affinity order; `kill_known(case)` is True when that
    case is demonstrated; `kill_family(family)` returns the confirmed divergent cases (possibly none) or raises StepFailure."""
    for case in known:
        if kill_known(case):
            return LadderResult(KNOWN_KILLER, [case], f"known killer {case}")
    for family in families[:MAX_FAMILIES]:
        try:
            confirmed = kill_family(family)
        except StepFailure as failure:
            path = TIMED_OUT if failure.kind == "timeout" else HARNESS_ERROR
            return LadderResult(path, [], f"family {family}: {failure}")
        if confirmed:
            learn(confirmed)
            return LadderResult(FAMILY, list(confirmed), f"family {family}")
    return LadderResult(FULL_REPLAY, [], "no known killer or family confirmed a divergence")


def confirmed_killers(divergent, diverges_only_in_mutant, confirms_applicable, limit=CANDIDATE_LIMIT):
    """The divergent cases that are real kills: the clean control does not diverge on them (`diverges_only_in_mutant`) and the real tool
    classifies them as applicable (`confirms_applicable`). Order kept, at most `limit` are looked at."""
    return [case for case in divergent[:limit] if diverges_only_in_mutant(case) and confirms_applicable(case)]


def affinity(name, file, killers_by_mutant, file_by_mutant, family_of_case):
    """The families of the cases that killed the OTHER mutants of the same file, most frequent first, ties by name: deterministic. A case
    that coverage.tsv does not list is ignored; a file with no recorded killers has no affinity (the caller then runs the full replay)."""
    counts = collections.Counter()
    for other, cases in killers_by_mutant.items():
        if other == name or file_by_mutant.get(other) != file:
            continue
        for family in {family_of_case(case) for case in cases} - {None}:
            counts[family] += 1
    return [family for family, _ in sorted(counts.items(), key=lambda item: (-item[1], item[0]))]


_NOT_EVIDENCE = frozenset({"oracle", "src", "rs", "mod", "the", "of", "a", "is", "not", "to", "in", "and", "as", "with", "for", "by", "on", "it"})


def _tokens(text):
    return {token for token in text.lower().replace("_", "-").replace("/", "-").replace(".", "-").split("-") if token} - _NOT_EVIDENCE


def token_affinity(name, file, families):
    """The families whose name shares a whole word with the stem of the mutated file or with the mutant's own name (`deployer.rs` -> oracle-deployer;
    `deploy` is not `deployer`), the most shared words first, then the fewest extra words, then the name: deterministic. No shared word, no affinity."""
    evidence = _tokens(file.rsplit("/", 1)[-1]) | _tokens(name)
    scored = []
    for family in families:
        words = _tokens(family)
        shared = len(evidence & words)
        if shared:
            scored.append((-shared, len(words), family))
    return [family for _, _, family in sorted(scored)]


def combined(first, second):
    """`first` then the families of `second` that it does not have."""
    return list(first) + [family for family in second if family not in first]


def merged_killers(existing, new):
    """The killers file after learning `new`: it never shrinks and keeps its order."""
    return list(existing) + [case for case in dict.fromkeys(new) if case not in existing]


def learn_into(path, cases):
    try:
        with open(path, encoding="utf-8") as stored:
            existing = [line.strip() for line in stored if line.strip()]
    except OSError:
        existing = []
    with open(path, "w", encoding="utf-8") as stored:
        stored.write("".join(f"{case}\n" for case in merged_killers(existing, cases)))
