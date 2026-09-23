"""Shared complex types, numerical helpers, and VM boundary adapters."""

from __future__ import annotations

import math
from typing_extensions import Callable, Tuple

from ......_helpers.types import is_vm_record
from .....types import Uninitialized, VmAny
from ...._helpers import _expect_number

C = Tuple[float, float]
INF = math.inf
NAN = math.nan


def math_call(f: Callable[[float], float], x: float) -> float:
    try:
        return f(x)
    except OverflowError:
        return math.copysign(INF, x) if f is math.sinh else INF
    except ValueError:
        if (f is math.log and x == 0) or (f is math.log1p and x == -1):
            return -INF
        return NAN


def div(a: float, b: float) -> float:
    if b == 0:
        return (
            NAN
            if a == 0 or math.isnan(a)
            else math.copysign(INF, a) * math.copysign(1, b)
        )
    return a / b


def maximum(a: float, b: float) -> float:
    return NAN if math.isnan(a) or math.isnan(b) else max(a, b)


def minimum(a: float, b: float) -> float:
    return NAN if math.isnan(a) or math.isnan(b) else min(a, b)


def radial(r: float, x: float) -> float:
    return math.copysign(0, r) * x if x == 0 and not math.isnan(r) else r * x


def parse(value: VmAny, name: str) -> C:
    if is_vm_record(value) and "0" in value and "1" in value:
        return (
            _expect_number(f"{name}.0", value["0"]),
            _expect_number(f"{name}.1", value["1"]),
        )
    return (_expect_number(name, value), 0.0)


def record(z: C) -> dict[str, float]:
    return {"0": z[0], "1": z[1]}


def unary(f: Callable[[C], C]):
    def call(z: VmAny = Uninitialized):
        return record(f(parse(z, "z")))

    return call


def binary(f: Callable[[C, C], C]):
    def call(a: VmAny = Uninitialized, b: VmAny = Uninitialized):
        return record(f(parse(a, "a"), parse(b, "b")))

    return call


def scalar(f: Callable[[C], float]):
    def call(z: VmAny = Uninitialized):
        return f(parse(z, "z"))

    return call
