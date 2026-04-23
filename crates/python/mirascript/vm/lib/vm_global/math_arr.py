from .._helpers import get_numbers
from functools import reduce as _reduce
import math
import sys
from ...types.checker import is_positive_zero, is_negative_zero


def build(func):
    def wrapper(*args):
        numbers = get_numbers(args)
        return func(*numbers)

    return wrapper


def _max(*args):
    high = -math.inf
    for num in args:
        if math.isnan(num):
            return math.nan
        if is_positive_zero(num) and is_negative_zero(high):
            high = num
        elif num > high:
            high = num
    return high


max_ = build(_max)


def _min(*args):
    low = math.inf
    for num in args:
        if math.isnan(num):
            return math.nan
        if is_negative_zero(num) and is_positive_zero(low):
            low = num
        elif num < low:
            low = num
    return low


min_ = build(_min)


def hypot(*args):
    numbers = get_numbers(args)
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
        if math.isnan(n):
            return math.nan
        s += n * n
    return math.sqrt(s)


sum_ = build(lambda *args: _reduce(lambda a, b: a + b, args, -0.0))
product = build(lambda *args: _reduce(lambda a, b: a * b, args, 1.0))
