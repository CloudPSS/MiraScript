import math
from random import random as _random

from ..._helpers import _expect_number


def atan2(y, x):
    return math.atan2(_expect_number("y", y), _expect_number("x", x))


def pow(x, y):
    return math.pow(_expect_number("x", x), _expect_number("y", y))


def random():
    return _random()
