import math
import sys
from functools import reduce as _reduce
from ..._helpers import _get_numbers
from .unary import abs

__all__ = [
    "max",
    "min",
    "hypot",
    "sum",
    "product",
]


def _is_positive_zero(num):
    return num == 0.0 and math.copysign(1, num) > 0


def _is_negative_zero(num):
    return num == 0.0 and math.copysign(1, num) < 0


def _build(func):
    def wrapper(*args):
        numbers = _get_numbers(args)
        return func(*numbers)

    return wrapper


@_build
def max(*args):
    high = -math.inf
    for num in args:
        if math.isnan(num):
            return math.nan
        if _is_positive_zero(num) and _is_negative_zero(high):
            high = num
        elif num > high:
            high = num
    return high


@_build
def min(*args):
    low = math.inf
    for num in args:
        if math.isnan(num):
            return math.nan
        if _is_negative_zero(num) and _is_positive_zero(low):
            low = num
        elif num < low:
            low = num
    return low


def hypot(*args):
    numbers = _get_numbers(args)
    if len(numbers) == 0:
        return 0.0
    # math.hypot accepts an arbitrary number of args starting Python 3.8.
    # For older Pythons, provide a compatible fallback.
    if sys.version_info >= (3, 8):
        return math.hypot(*numbers)

    if len(numbers) == 1:
        return abs(numbers[0])
    if len(numbers) == 2:
        return math.hypot(numbers[0], numbers[1])

    # Manual computation for >2 args on older Pythons
    s = 0.0
    for n in numbers:
        if math.isinf(n):
            return math.inf
        s += n * n
    return math.sqrt(s)


sum = _build(lambda *args: _reduce(lambda a, b: a + b, args, -0.0))
product = _build(lambda *args: _reduce(lambda a, b: a * b, args, 1.0))
