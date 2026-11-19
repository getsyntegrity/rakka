"""NA:Q-CFG-30: a replay divergence that is only the JDK 17 bug in `Double.toString` (decision Q-CFG-30).

JDK 17 sometimes prints a digit string that is not the shortest one that identifies the double, or not the closest of
the shortest ones (fixed in JDK 19, whose Javadoc defines the result: shortest length, then closest to the exact value,
then the even last digit on an exact tie; a single digit is widened to two). rakka follows the JDK 19+ definition.

A case is "not applicable" only when its signature matches; no case ids are listed anywhere. All arithmetic is on
Python integers and `Fraction`s (never `float`): the exact value of a double is mantissa * 2**exponent.

  a) both outputs parse to the same double, bit for bit;
  b) the rakka digits are the JDK 19+ result (shortest, then closest, then even on a tie);
  c) the oracle digits are not;
  d) same layout (decimal or E notation, same decimal exponent);
  e) the rest of the output is identical byte for byte.
"""
import re
from fractions import Fraction

# a Java double literal as `Double.toString` writes it (it always has a point); word characters and points around it
# mean it is part of something else (an identifier, a version)
TOKEN = re.compile(r"(?<![\w.])-?\d+\.\d+(?:E-?\d+)?(?![\w.])")
MANTISSA_BITS = 53
MIN_EXPONENT = -1074  # exponent of the last bit of a subnormal
MAX_EXPONENT = 971    # exponent of the last bit of the largest double


def _round_half_even(q):
    whole, rest = divmod(q.numerator, q.denominator)
    twice = 2 * rest
    if twice > q.denominator or (twice == q.denominator and whole % 2 == 1):
        whole += 1
    return whole


def nearest_double(q):
    """Positive exact `Fraction` -> (mantissa, exponent) of the nearest double (ties to even), or None above the range."""
    exponent = max(q.numerator.bit_length() - q.denominator.bit_length() - MANTISSA_BITS, MIN_EXPONENT)
    # an estimate can be off by one in either direction: settle on the exponent that gives 53 bits
    while exponent > MIN_EXPONENT and q / Fraction(2) ** exponent < 2 ** (MANTISSA_BITS - 1):
        exponent -= 1
    while q / Fraction(2) ** exponent >= 2 ** MANTISSA_BITS:
        exponent += 1
    mantissa = _round_half_even(q / Fraction(2) ** exponent)
    if mantissa == 2 ** MANTISSA_BITS:
        mantissa //= 2
        exponent += 1
    if exponent > MAX_EXPONENT:
        return None
    return mantissa, exponent


class Literal:
    """A decimal token of `Double.toString`: sign, significant digits without trailing zeros, decimal exponent of the
    first digit, and whether it is written in E notation."""

    def __init__(self, text):
        self.text = text
        self.negative = text.startswith("-")
        body = text.lstrip("-")
        self.scientific = "E" in body
        mantissa, _, power = body.partition("E")
        whole, _, fraction = mantissa.partition(".")
        digits = whole + fraction
        first = len(digits) - len(digits.lstrip("0"))
        self.digits = digits[first:].rstrip("0") or "0"
        self.exponent = len(whole) - 1 - first + (int(power) if power else 0)

    def value(self):
        return Fraction(int(self.digits)) * Fraction(10) ** (self.exponent - len(self.digits) + 1)

    def double(self):
        """(sign, mantissa, exponent) of the double the literal rounds to; zero is (sign, 0, 0); None above the range."""
        if int(self.digits) == 0:
            return (self.negative, 0, 0)
        near = nearest_double(self.value())
        return None if near is None else (self.negative, *near)


def _decimal_exponent(x):
    """floor(log10(x)) for a positive exact `Fraction`."""
    estimate = len(str(x.numerator)) - len(str(x.denominator))
    while Fraction(10) ** estimate > x:
        estimate -= 1
    while Fraction(10) ** (estimate + 1) <= x:
        estimate += 1
    return estimate


def shortest_digits(mantissa, exponent):
    """JDK 19+ digits (string without trailing zeros, decimal exponent of the first one) of mantissa * 2**exponent."""
    x = Fraction(mantissa) * Fraction(2) ** exponent
    top = _decimal_exponent(x)

    def candidates(length):
        unit = Fraction(10) ** (top - length + 1)
        low = x // unit
        found = []
        for k in (low - 1, low, low + 1, low + 2):
            if k > 0 and nearest_double(k * unit) == (mantissa, exponent):
                found.append((abs(x - k * unit), k, unit))
        return found

    shortest = next(n for n in range(1, 40) if candidates(n))
    pool = candidates(max(shortest, 2))  # one digit is shown as two (`4.9E-324`, `1.0E-7`)
    nearest = min(distance for distance, _k, _u in pool)
    # an exact tie leaves two candidates; the even last digit wins
    _distance, k, _unit = min((c for c in pool if c[0] == nearest), key=lambda c: c[1] % 2)
    literal = str(k)
    return literal.rstrip("0") or "0", top + len(literal) - max(shortest, 2)


def _is_jdk19_result(literal):
    near = literal.double()
    if near is None or near[1] == 0:
        return False
    return (literal.digits, literal.exponent) == shortest_digits(near[1], near[2])


def _split(text):
    """-> (pieces between the numbers, the numbers)."""
    return TOKEN.split(text), TOKEN.findall(text)


def is_q_cfg_30(oracle_output, rakka_output):
    """True when `rakka_output` differs from `oracle_output` only by the signature described in the module docs."""
    try:
        oracle_text, rakka_text = oracle_output.decode("utf-8"), rakka_output.decode("utf-8")
    except UnicodeDecodeError:
        return False
    oracle_pieces, oracle_numbers = _split(oracle_text)
    rakka_pieces, rakka_numbers = _split(rakka_text)
    if oracle_pieces != rakka_pieces or len(oracle_numbers) != len(rakka_numbers):  # (e) the rest of the output
        return False
    different = [(o, r) for o, r in zip(oracle_numbers, rakka_numbers) if o != r]
    if len(different) != 1:
        return False
    oracle, rakka = Literal(different[0][0]), Literal(different[0][1])
    same_double = oracle.double() is not None and oracle.double() == rakka.double()  # (a)
    same_layout = oracle.scientific == rakka.scientific and oracle.exponent == rakka.exponent  # (d)
    return same_double and _is_jdk19_result(rakka) and not _is_jdk19_result(oracle) and same_layout  # (b) (c)
