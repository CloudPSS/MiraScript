"""The complex standard library. Only two-field records cross the VM boundary."""

from __future__ import annotations

import math as _math

from .....types import Uninitialized, VmAny
from ...._helpers import _expect_number
from . import basic as b
from . import math as m
from .utils import binary, record, scalar, unary

I = record((0.0, 1.0))
# "from" is a Python keyword but remains the public MiraScript member name.
globals()["from"] = unary(lambda z: z)
real = scalar(lambda z: z[0])
imag = scalar(lambda z: z[1])
abs = scalar(lambda z: _math.hypot(*z))
arg = scalar(lambda z: _math.atan2(z[1], z[0]))
conj = unary(b.conj)
neg = unary(b.neg)
add = binary(b.add)
subtract = binary(b.subtract)
multiply = binary(b.multiply)
divide = binary(b.divide)
pow = binary(m.pow)
sqrt = unary(m.sqrt)
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
