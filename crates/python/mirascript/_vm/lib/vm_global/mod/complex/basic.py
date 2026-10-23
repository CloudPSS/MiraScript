"""Native complex arithmetic with MiraScript overflow/zero compatibility."""

from __future__ import annotations

import cmath
import math
import sys

from .utils import INF, div, math_call, maximum, radial

_MIN_COMPONENT = math.sqrt(sys.float_info.min)
_MAX_COMPONENT = math.sqrt(sys.float_info.max) / 2


def multiply(z: complex, w: complex) -> complex:
    a, b, c, d = z.real, z.imag, w.real, w.imag
    if b == 0 and d == 0:
        return complex(a * c, radial(a, d) + radial(c, b))
    if (
        cmath.isfinite(z)
        and cmath.isfinite(w)
        and any(not math.isfinite(x) for x in (a * c, b * d, a * d, b * c))
    ):
        s, t = max(abs(a), abs(b)), max(abs(c), abs(d))
        return complex(
            ((a / s * (c / t) - b / s * (d / t)) * min(s, t)) * max(s, t),
            ((a / s * (d / t) + b / s * (c / t)) * min(s, t)) * max(s, t),
        )
    if cmath.isfinite(z) and cmath.isfinite(w):
        return z * w
    # Native multiplication recovers infinities from some NaN intermediates.
    return complex(a * c - b * d, a * d + b * c)


def _sum_over(x: float, y: float, q: float) -> float:
    total = x + y
    return (
        div(x, q) + div(y, q)
        if not math.isfinite(total) and math.isfinite(x) and math.isfinite(y)
        else div(total, q)
    )


def divide(z: complex, w: complex) -> complex:
    a, b, c, d = z.real, z.imag, w.real, w.imag
    s = maximum(abs(c), abs(d))
    if s == 0:
        return complex(math.copysign(INF, c) * a, math.copysign(INF, c) * b)
    # CPython division can lose cancellation, overflow, or underflow at extremes.
    # Use it for ordinary finite operands and retain scaled division otherwise.
    if all(x == 0 or _MIN_COMPONENT <= abs(x) <= _MAX_COMPONENT for x in (a, b, c, d)):
        result = z / w
        # Cancellation zeros use the shared Cartesian formula's sign rules.
        if result.real != 0 and result.imag != 0:
            return result
    if s == INF and math.isfinite(a) and math.isfinite(b):
        c = math.copysign(1 if abs(c) == INF else 0, c)
        d = math.copysign(1 if abs(d) == INF else 0, d)
        return complex(0 * (a * c + b * d), 0 * (b * c - a * d))
    c, d = div(c, s), div(d, s)
    q = c * c + d * d
    if (
        (not math.isfinite(div(a, s)) or not math.isfinite(div(b, s)))
        and math.isfinite(a)
        and math.isfinite(b)
    ):
        u = max(abs(a), abs(b))
        return complex(
            div(div(div(a, u) * c + div(b, u) * d, q) * u, s),
            div(div(div(b, u) * c - div(a, u) * d, q) * u, s),
        )
    return complex(
        _sum_over(div(a, s) * c, div(b, s) * d, q),
        _sum_over(div(b, s) * c, -div(a, s) * d, q),
    )


def polar(r: float, theta: float) -> complex:
    if math.isfinite(theta) and not math.isnan(r):
        return cmath.rect(r, theta)
    return complex(
        radial(r, math_call(math.cos, theta)),
        radial(r, math_call(math.sin, theta)),
    )
