"""Basic Cartesian complex operations."""

from __future__ import annotations

import math

from .utils import C, INF, div, math_call, maximum, radial


def add(z: C, w: C) -> C:
    return (z[0] + w[0], z[1] + w[1])


def subtract(z: C, w: C) -> C:
    return (z[0] - w[0], z[1] - w[1])


def neg(z: C) -> C:
    return (-z[0], -z[1])


def conj(z: C) -> C:
    return (z[0], -z[1])


def multiply(z: C, w: C) -> C:
    a, b = z
    c, d = w
    if b == 0 and d == 0:
        return (a * c, radial(a, d) + radial(c, b))
    if all(math.isfinite(x) for x in (a, b, c, d)) and any(
        not math.isfinite(x) for x in (a * c, b * d, a * d, b * c)
    ):
        s, t = max(abs(a), abs(b)), max(abs(c), abs(d))
        return (
            ((a / s * (c / t) - b / s * (d / t)) * min(s, t)) * max(s, t),
            ((a / s * (d / t) + b / s * (c / t)) * min(s, t)) * max(s, t),
        )
    return (a * c - b * d, a * d + b * c)


def _sum_over(x: float, y: float, q: float) -> float:
    total = x + y
    return (
        div(x, q) + div(y, q)
        if not math.isfinite(total) and math.isfinite(x) and math.isfinite(y)
        else div(total, q)
    )


def divide(z: C, w: C) -> C:
    a, b = z
    c, d = w
    s = maximum(abs(c), abs(d))
    if s == 0:
        return (math.copysign(INF, c) * a, math.copysign(INF, c) * b)
    if s == INF and math.isfinite(a) and math.isfinite(b):
        c = math.copysign(1 if abs(c) == INF else 0, c)
        d = math.copysign(1 if abs(d) == INF else 0, d)
        return (0 * (a * c + b * d), 0 * (b * c - a * d))
    c, d = div(c, s), div(d, s)
    q = c * c + d * d
    if (
        (not math.isfinite(div(a, s)) or not math.isfinite(div(b, s)))
        and math.isfinite(a)
        and math.isfinite(b)
    ):
        u = max(abs(a), abs(b))
        return (
            div(div(div(a, u) * c + div(b, u) * d, q) * u, s),
            div(div(div(b, u) * c - div(a, u) * d, q) * u, s),
        )
    return (
        _sum_over(div(a, s) * c, div(b, s) * d, q),
        _sum_over(div(b, s) * c, -div(a, s) * d, q),
    )


def polar(r: float, theta: float) -> C:
    return (
        radial(r, math_call(math.cos, theta)),
        radial(r, math_call(math.sin, theta)),
    )
