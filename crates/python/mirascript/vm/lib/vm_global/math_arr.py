from mirascript.vm.operations import ToNumber_
from .._helpers import get_numbers
from functools import reduce as _reduce
import math
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
hypot = build(math.hypot)
sum_ = build(lambda *args: _reduce(lambda a, b: a + b, args, -0.0))
product = build(lambda *args: _reduce(lambda a, b: a * b, args, 1.0))

# def min_(*arr):
    
#     numbers = get_numbers(arr)
#     return min(numbers)

# def hypot(*arr):
#     numbers = get_numbers(arr)
#     return math.hypot(*numbers)

# def sum_(*arr):

#     numbers = get_numbers(arr)
#     return sum(numbers)

# def product(*arr):
#     numbers = get_numbers(arr)
#     return _reduce(lambda a,b:a*b, numbers, 1)