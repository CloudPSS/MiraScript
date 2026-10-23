"""The complex standard library. Only two-field records cross the VM boundary."""

from __future__ import annotations

import cmath as _cmath
import math as _math
import operator as _operator

from .....types import Uninitialized, VmAny
from ...._helpers import _expect_number
from . import basic as b
from . import math as m
from .utils import binary, record, scalar, unary

I = record(1j)
# "from" is a Python keyword but remains the public MiraScript member name.
globals()["from"] = unary(lambda z: z)
real = scalar(lambda z: z.real)
imag = scalar(lambda z: z.imag)
abs = scalar(lambda z: _math.hypot(z.real, z.imag))
arg = scalar(_cmath.phase)
conj = unary(complex.conjugate)
neg = unary(_operator.neg)
add = binary(_operator.add)
subtract = binary(_operator.sub)
multiply = binary(b.multiply)
divide = binary(b.divide)
pow = binary(m.pow)
sqrt = unary(_cmath.sqrt)
cbrt = unary(m.cbrt)
exp = unary(m.exp)
expm1 = unary(m.expm1)
log = unary(m.log)
log1p = unary(m.log1p)
log2 = unary(lambda z: m.log_base(z, _math.log(2)))
log10 = unary(lambda z: m.log_base(z, _math.log(10)))
sin = unary(m.sin)
cos = unary(m.cos)
tan = unary(m.tan)
sinh = unary(m.sinh)
cosh = unary(m.cosh)
tanh = unary(m.tanh)
asin = unary(m.asin)
acos = unary(m.acos)
atan = unary(m.atan)
asinh = unary(m.asinh)
acosh = unary(m.acosh)
atanh = unary(m.atanh)


def polar(r: VmAny = Uninitialized, theta: VmAny = Uninitialized):
    return record(b.polar(_expect_number("r", r), _expect_number("theta", theta)))


__all__ = [
    "I",
    "from",  # pyright: ignore[reportUnsupportedDunderAll] -- exported through globals above
    "real",
    "imag",
    "abs",
    "arg",
    "conj",
    "neg",
    "add",
    "subtract",
    "multiply",
    "divide",
    "pow",
    "sqrt",
    "cbrt",
    "exp",
    "expm1",
    "log",
    "log1p",
    "log2",
    "log10",
    "sin",
    "cos",
    "tan",
    "sinh",
    "cosh",
    "tanh",
    "asin",
    "acos",
    "atan",
    "asinh",
    "acosh",
    "atanh",
    "polar",
]
