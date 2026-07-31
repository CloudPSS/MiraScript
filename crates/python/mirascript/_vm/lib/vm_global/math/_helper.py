from __future__ import annotations
import math
from typing_extensions import Callable, TypeAlias

from ....types import Uninitialized, VmAny
from ..._helpers import _expect_number

F: TypeAlias = "Callable[[float], float | int]"


def _run(
    x: VmAny,
    func: F,
    nan: float | None = None,
    posinf: float | None = None,
    neginf: float | None = None,
    poszero: float | None = None,
    negzero: float | None = None,
):
    x = _expect_number("x", x)
    if math.isnan(x):
        if nan is not None:
            return nan
    elif math.isinf(x):
        if x < 0 and neginf is not None:
            return neginf
        elif x > 0 and posinf is not None:
            return posinf
    elif x == 0.0:
        if math.copysign(1.0, x) < 0:
            if negzero is not None:
                return negzero
        else:
            if poszero is not None:
                return poszero
    try:
        ret = func(x)
        return float(ret)
    except Exception:
        return math.nan


def _build(
    func: F,
    nan: float | None = None,
    posinf: float | None = None,
    neginf: float | None = None,
    poszero: float | None = None,
    negzero: float | None = None,
):
    def wrapper(x: VmAny = Uninitialized):
        return _run(
            x,
            func,
            nan=nan,
            posinf=posinf,
            neginf=neginf,
            poszero=poszero,
            negzero=negzero,
        )

    return wrapper
