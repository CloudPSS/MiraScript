"""The complex standard library. Only two-field records cross the VM boundary."""

from __future__ import annotations

import math
from typing_extensions import Callable

from ......_helpers.types import is_vm_record
from .....types import Uninitialized, VmAny
from ...._helpers import _expect_number
from . import _kernel as k


def _parse(value: VmAny, name: str) -> k.C:
    if is_vm_record(value) and "0" in value and "1" in value:
        return (
            _expect_number(f"{name}.0", value["0"]),
            _expect_number(f"{name}.1", value["1"]),
        )
    return (_expect_number(name, value), 0.0)


def _record(z: k.C) -> dict[str, float]:
    return {"0": z[0], "1": z[1]}


def _unary(f: Callable[[k.C], k.C]):
    def call(z: VmAny = Uninitialized):
        return _record(f(_parse(z, "z")))

    return call


def _binary(f: Callable[[k.C, k.C], k.C]):
    def call(a: VmAny = Uninitialized, b: VmAny = Uninitialized):
        return _record(f(_parse(a, "a"), _parse(b, "b")))

    return call


def _scalar(f: Callable[[k.C], float]):
    def call(z: VmAny = Uninitialized):
        return f(_parse(z, "z"))

    return call


I = _record((0.0, 1.0))
# "from" is a Python keyword but remains the public MiraScript member name.
globals()["from"] = _unary(lambda z: z)
real = _scalar(lambda z: z[0])
imag = _scalar(lambda z: z[1])
abs = _scalar(lambda z: math.hypot(*z))
arg = _scalar(lambda z: math.atan2(z[1], z[0]))
conj = _unary(k.conj)
neg = _unary(k.neg)
add = _binary(k.add)
subtract = _binary(k.subtract)
multiply = _binary(k.multiply)
divide = _binary(k.divide)
pow = _binary(k.pow)
sqrt = _unary(k.sqrt)
cbrt = _unary(k.cbrt)
exp = _unary(k.exp)
expm1 = _unary(k.expm1)
log = _unary(k.log)
log1p = _unary(k.log1p)
log2 = _unary(lambda z: k.log_base(z, math.log(2)))
log10 = _unary(lambda z: k.log_base(z, math.log(10)))
sin = _unary(k.sin)
cos = _unary(k.cos)
tan = _unary(k.tan)
sinh = _unary(k.sinh)
cosh = _unary(k.cosh)
tanh = _unary(k.tanh)
asin = _unary(k.asin)
acos = _unary(k.acos)
atan = _unary(k.atan)
asinh = _unary(k.asinh)
acosh = _unary(k.acosh)
atanh = _unary(k.atanh)


def polar(r: VmAny = Uninitialized, theta: VmAny = Uninitialized):
    return _record(k.polar(_expect_number("r", r), _expect_number("theta", theta)))


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
