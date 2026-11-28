"""Rows the Owner closed without own implementation: the marker their note carries and how to recognise them."""
import re

REMOVED = "ELIMINADO EN 2.0.0"
EQUIVALENCE = "EQUIVALENCIA DOCUMENTADA"
MARKER = re.compile(r"SIN IMPLEMENTACIÓN PROPIA: (%s|%s) \((Q-(?:T-)?[A-Z]+-\d+), aceptada\)" % (re.escape(REMOVED), EQUIVALENCE))


def marker(row):
    """(reason, decision id) written in the row's note, or None."""
    found = MARKER.search(row["notes"])
    return (found.group(1), found.group(2)) if found else None


def closed_without_implementation(row):
    return row["impl"] == "done" and marker(row) is not None
