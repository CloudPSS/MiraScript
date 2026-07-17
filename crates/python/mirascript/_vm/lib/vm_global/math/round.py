import math
import builtins
from ..._helpers import _expect_number, _expect_integer_range
from ....types.types import Uninitialized
from ._helper import _run

__all__ = [
    "trunc",
    "floor",
    "ceil",
    "round",
]


def _build(func):
    def wrapper(x=Uninitialized, n=None):
        n = _expect_integer_range("n", n, 0, 15) if n is not None else 0
        if n == 0:
            ret = _run(x, func, math.nan, math.inf, -math.inf)
        else:
            factor = 10**n
            ret = _run(
                x, lambda v: func(v * factor) / factor, math.nan, math.inf, -math.inf
            )
        if ret == 0.0:
            return math.copysign(0.0, _expect_number("x", x))
        return ret

    return wrapper


trunc = _build(math.trunc)
floor = _build(math.floor)
ceil = _build(math.ceil)
round = _build(lambda x: builtins.round(x, 0))
