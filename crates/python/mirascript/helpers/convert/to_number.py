from typing import TypeVar, Union
import math
import re
from mirascript.vm.error import VmError
from mirascript.vm.types.const import Uninitialized


def is_decimal_number(num):
    """判断数字是否为小数（不是整数）"""

    if isinstance(num, (int, float)):
        return isinstance(num, float) and not num.is_integer()
    return False


def parseNumericLiteral(s):
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


def stringToNumber(s):
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
        num = parseNumericLiteral(s[1:])
        if num is not None:
            return -num
        return None
    elif s.startswith("+"):
        num = parseNumericLiteral(s[1:])
        if num is not None:
            return num
        return None
    else:
        return parseNumericLiteral(s)


T = TypeVar("T")


def toNumber(value, fallback: T = Uninitialized) -> Union[float, T]:
    if isinstance(value, bool):
        return float(1 if value else 0)

    if isinstance(value, (int, float)):
        return float(value)

    if isinstance(value, str):
        num = stringToNumber(value)
        if num is not None:
            return num

    if fallback is Uninitialized:
        raise VmError(f"Failed to convert value to number: {value}", math.nan)

    return fallback
