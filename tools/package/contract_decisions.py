"""Decisions that a contract block declares for its row; shared by generate.py and check.py so they cannot diverge.

Two declaration formats exist in docs/inventory/contracts/*.md, with or without a list bullet:
    - **Decisiones**: Q-X-4
    **Decisiones.** Q-X-4
A declaration belongs to the contract block opened by the nearest preceding heading; any other heading closes the block.
"""
import re

ALIASES = {"Q-CFG-1": "Q-X-1", "Q-CFG-2": "Q-X-2"}
DECISION_ID = re.compile(r"Q-(?:T-)?[A-Z]+-\d+")
CONTRACT_HEADING = re.compile(r"#+ (A[CT]-[A-Z]+-\d+(?:\.\d+)?)\b")
HEADING = re.compile(r"#+ ")
DECLARATION = re.compile(r"\s*(?:[-*]\s+)?\*\*Decisiones(?:\*\*:|\.\*\*)(.*)$")


def declared_decisions(markdown):
    """Row id -> normalized decision ids declared in that row's contract block (alias ids replaced by their canonical id)."""
    declared, current = {}, None
    for line in markdown.splitlines():
        heading = CONTRACT_HEADING.match(line)
        if heading:
            current = heading.group(1)
        elif HEADING.match(line):
            current = None
        elif current:
            declaration = DECLARATION.match(line)
            if declaration:
                ids = {ALIASES.get(q, q) for q in DECISION_ID.findall(declaration.group(1))}
                declared.setdefault(current, set()).update(ids)
    return declared
