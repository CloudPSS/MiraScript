import math
from random import random as _random

from mirascript.vm.lib._helpers import expect_number


def atan2(y, x):
    return math.atan2(expect_number("y", y), expect_number("x", x))


def pow(x, y):
    return math.pow(expect_number("x", x), expect_number("y", y))


def random():
    return _random()
