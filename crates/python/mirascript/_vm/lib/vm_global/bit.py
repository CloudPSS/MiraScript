from __future__ import annotations
import math

from ...types import VmAny
from .._helpers import _expect_number

__all__ = [
    "b_and",
    "b_or",
    "b_not",
    "b_xor",
    "shl",
    "sar",
    "shr",
]


def _to_int32(x: VmAny, name="x") -> int:
    x = _expect_number(name, x)
    if not math.isfinite(x):
        return 0

    x = int(x)
    return x


def b_and(x: VmAny, y: VmAny) -> float:
    x = _to_int32(x, "x")
    y = _to_int32(y, "y")

    result = x & y

    if result & 0x80000000:
        result -= 0x100000000

    return float(result)


def b_or(x: VmAny, y: VmAny) -> float:
    x = _to_int32(x, "x")
    y = _to_int32(y, "y")
    result = (x | y) & 0xFFFFFFFF
    if result & 0x80000000:
        result -= 0x100000000
    return float(result)


def b_not(x: VmAny) -> float:
    x = _to_int32(x, "x")
    result = (~x) & 0xFFFFFFFF
    if result & 0x80000000:
        result -= 0x100000000

    return float(result)


def b_xor(x: VmAny, y: VmAny) -> float:
    x = _to_int32(x, "x")
    y = _to_int32(y, "y")
    result = (x ^ y) & 0xFFFFFFFF

    if result & 0x80000000:
        result -= 0x100000000

    return float(result)


def shl(x: VmAny, y: VmAny) -> float:
    x = _to_int32(x, "x")
    y = _to_int32(y, "y")

    y &= 31  # 只保留低5位（0~31）
    x &= 0xFFFFFFFF  # 保持32位范围
    return ((x << y) | (x >> (32 - y))) & 0xFFFFFFFF


def sar(x: VmAny, y: VmAny) -> float:
    x = _to_int32(x, "x")
    y = _to_int32(y, "y")
    result = x >> y

    return float(result)


def shr(x: VmAny, y: VmAny) -> float:
    x = _to_int32(x, "x")
    y = _to_int32(y, "y")
    if x >= 0:
        return float(x >> y)
    else:
        return float((x + 0x100000000) >> y)
