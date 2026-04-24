import math
from .._helpers import _expect_number, _required
from mirascript.vm.types.const import Uninitialized
import decimal


def _build(
    func, nan=None, inf=None, neginf=None, poszero=None, negzero=None, except_inf=None
):
    def wrapper(x=Uninitialized):
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
            if poszero is not None:
                return poszero
        try:
            ret = func(x)
            if type(ret) is decimal.Decimal:
                ret = ret.quantize(decimal.Decimal("1"), rounding=decimal.ROUND_HALF_UP)
            return float(ret)
        except Exception as e:
            if except_inf == True:
                if x > 0:
                    return math.inf
                else:
                    return -math.inf
            return math.nan

    return wrapper


abs_ = _build(abs, math.nan, math.inf, math.inf, 0.0, 0.0)


def trunc(x=Uninitialized):
    ret = _build(math.trunc, math.nan, math.inf, -math.inf)(x)
    return math.copysign(ret, _expect_number("x", x))


floor = _build(math.floor, math.nan, math.inf, -math.inf, +0.0, -0.0)


def ceil(x=Uninitialized):
    ret = _build(math.ceil, math.nan, math.inf, -math.inf)(x)

    if ret == 0.0:
        return math.copysign(0.0, _expect_number("x", x))
    return ret


round_ = _build(lambda x: round(x, 0))


sign = _build(lambda v: 1 if v > 0 else -1 if v < 0 else 0, math.nan, 1, -1, +0.0, -0.0)

acos = _build(math.acos, math.nan, math.nan, math.nan)
acosh = _build(math.acosh, math.nan)
asin = _build(math.asin, math.nan, math.nan, math.nan)
asinh = _build(math.asinh, math.nan, math.inf, -math.inf)
atan = _build(math.atan, math.nan)


# atanh = build(math.atanh,math.nan,math.nan,math.nan)
def atanh(x=Uninitialized):
    x = _expect_number("x", x)
    ret = _build(math.atanh, math.nan)(x)
    if math.isnan(ret):
        if x == 1.0:
            return math.inf
        elif x == -1.0:
            return -math.inf
    if ret == 0.0:
        return math.copysign(0.0, x)
    return ret


cos = _build(math.cos, math.nan)
cosh = _build(math.cosh, math.nan)
sin = _build(math.sin, math.nan)
sinh = _build(math.sinh, math.nan)
tan = _build(math.tan, math.nan)
tanh = _build(math.tanh, math.nan)
exp = _build(math.exp, math.nan)
expm1 = _build(math.expm1, math.nan)
log = _build(math.log, math.nan, math.inf, math.nan, -math.inf, -math.inf)
log10 = _build(math.log10, math.nan, math.inf, math.nan, -math.inf, -math.inf)


# log1p = build(math.log1p,math.nan,math.inf,math.nan,0,-0)
def log1p(x=Uninitialized):
    x = _expect_number("x", x)
    ret = _build(math.log1p, math.nan, math.inf, math.nan)(x)
    if math.isnan(ret):
        if x == -1.0:
            return -math.inf
    if ret == 0.0:
        return math.copysign(0.0, x)
    return ret


log2 = _build(math.log2, math.nan, math.inf, math.nan, -math.inf, -math.inf)
sqrt = _build(math.sqrt, math.nan)
cbrt = _build(lambda x: math.copysign(abs(x) ** (1 / 3), x), math.nan)
