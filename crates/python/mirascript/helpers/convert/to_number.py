from typing_extensions import TypeVar, overload
import math
import re

from ...vm.error import VmError
from ...vm.types.types import Uninitialized


def _parse_number(s: str) -> "float | None":
    if s == "":
        return None
    ch = s[0]
    if ch < "0" or ch > "9":
        return None

    if re.match(r"^0[bB][01]+$", s):
        return float(int(s[2:], 2))
    elif re.match(r"^0[oO][0-7]+$", s):
        return float(int(s[2:], 8))
    elif re.match(r"^0[xX][0-9a-fA-F]+$", s):
        return float(int(s[2:], 16))
    elif re.match(r"^[0-9][0-9]*$", s):
        return float(int(s, 10))
    elif re.match(r"^[-+]?[0-9]*\.?[0-9]+([eE][-+]?[0-9]+)?$", s):
        return float(s)
    else:
        return None


def _string_to_number(s: str) -> "float | None":
    s = s.strip()
    if s == "":
        return None
    if s == "inf" or s == "+inf" or s == "Infinity" or s == "+Infinity":
        return math.inf
    if s == "-inf" or s == "-Infinity":
        return -math.inf
    if s == "nan" or s == "NaN":
        return math.nan
    if s.startswith("-"):
        num = _parse_number(s[1:])
        if num is not None:
            return -num
        return None
    elif s.startswith("+"):
        num = _parse_number(s[1:])
        if num is not None:
            return num
        return None
    else:
        return _parse_number(s)


T = TypeVar("T")


@overload
def toNumber(value) -> float: ...
@overload
def toNumber(value, fallback: T) -> "float | T": ...
def toNumber(value, fallback: T = Uninitialized) -> "float | T":
    if isinstance(value, bool):
        return 1.0 if value else 0.0

    if isinstance(value, (int, float)):
        return float(value)

    if isinstance(value, str):
        num = _string_to_number(value)
        if num is not None:
            return num

    if fallback is Uninitialized:
        raise VmError(f"Failed to convert value to number: {value}", math.nan)

    return fallback
