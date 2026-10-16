import math
from functools import reduce as _reduce
from ..._helpers import _get_numbers

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
    return math.hypot(*numbers)


sum = _build(lambda *args: _reduce(lambda a, b: a + b, args, -0.0))
product = _build(lambda *args: _reduce(lambda a, b: a * b, args, 1.0))
