import math
from ..._helpers import _expect_number
from mirascript.vm.types.const import Uninitialized
from ._helper import _build


abs_ = _build(abs, math.nan, math.inf, math.inf, 0.0, 0.0)


sign = _build(lambda v: 1 if v > 0 else -1 if v < 0 else v)

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
