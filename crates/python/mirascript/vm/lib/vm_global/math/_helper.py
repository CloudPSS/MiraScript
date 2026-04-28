import math
import decimal
from typing import Callable
from ..._helpers import _expect_number
from mirascript.vm.types.const import Uninitialized


def _run(
    x,
    func: Callable[[float], float],
    nan=None,
    inf=None,
    neginf=None,
    poszero=None,
    negzero=None,
    except_inf=None,
):
    x = _expect_number("x", x)
    if math.isnan(x):
        if nan is not None:
            return nan
    elif math.isinf(x):
        if x < 0 and neginf is not None:
            return neginf
        elif x > 0 and inf is not None:
            return inf
    elif x == 0.0:
        if math.copysign(1.0, x) < 0:
            if negzero is not None:
                return negzero
        else:
            if poszero is not None:
                return poszero
    try:
        ret = func(x)
        if type(ret) is decimal.Decimal:
            ret = ret.quantize(decimal.Decimal("1"), rounding=decimal.ROUND_HALF_UP)
        return float(ret)
    except Exception:
        if except_inf == True:
            if x > 0:
                return math.inf
            else:
                return -math.inf
        return math.nan


def _build(
    func: Callable[[float], float],
    nan=None,
    inf=None,
    neginf=None,
    poszero=None,
    negzero=None,
    except_inf=None,
):
    def wrapper(x=Uninitialized):
        return _run(
            x,
            func,
            nan=nan,
            inf=inf,
            neginf=neginf,
            poszero=poszero,
            negzero=negzero,
            except_inf=except_inf,
        )

    return wrapper
