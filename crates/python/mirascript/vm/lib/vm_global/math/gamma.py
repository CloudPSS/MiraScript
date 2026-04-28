import math

from mirascript.vm.types.const import Uninitialized
from ..._helpers import _expect_number
from mirascript.vm.operations import isDecimalNumber


def _factorial(n: float):
    if n >= 171:
        return math.inf
    if n == 0 or n == 1:
        return 1.0
    r = 1
    for i in range(2, int(n) + 1):
        r *= i
    return float(r)


def _gamma(n: float):
    if math.isnan(n) or n == -math.inf:
        return math.nan
    if n >= 172:
        return math.inf
    if not isDecimalNumber(n):
        if n > 0:
            return _factorial(n - 1)
        elif n == 0:
            if math.copysign(1.0, n) < 0:
                return -math.inf
            else:
                return math.inf
        else:
            return math.nan
    try:
        return math.gamma(n)
    except OverflowError:
        return math.inf


def gamma(x=Uninitialized):
    n = _expect_number("x", x)
    return _gamma(n)


def factorial(x=Uninitialized):
    n = _expect_number("x", x)
    if math.isnan(n) or n < 0:
        return math.nan
    if n >= 171:
        return math.inf
    if not isDecimalNumber(n):
        return _factorial(n)
    return _gamma(n + 1)
