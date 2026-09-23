"""Transcendental Cartesian complex operations."""

from __future__ import annotations

import math

from .basic import multiply, polar
from .utils import C, INF, NAN, div, math_call, maximum, minimum, radial


def _log_abs(z: C) -> float:
    x, y = z
    a, b = maximum(abs(x), abs(y)), minimum(abs(x), abs(y))
    if a == INF:
        return INF
    if a == 0:
        return -INF
    return math_call(math.log, a) + 0.5 * math_call(math.log1p, div(b, a) * div(b, a))


def log(z: C) -> C:
    return (_log_abs(z), math.atan2(z[1], z[0]))


def log_base(z: C, base: float) -> C:
    a, b = log(z)
    return (a / base, b / base)


def sqrt(z: C) -> C:
    x, y = z
    if abs(y) == INF:
        return (INF, y)
    if x == INF:
        return (INF, NAN if math.isnan(y) else math.copysign(0, y))
    if x == -INF:
        return (NAN if math.isnan(y) else 0.0, math.copysign(INF, y))
    s = maximum(abs(x), abs(y))
    if s == 0:
        return (0.0, y)
    t = math_call(math.sqrt, s) * math_call(
        math.sqrt, (math.hypot(x / s, y / s) + abs(x) / s) / 2
    )
    return (t, div(y, t) / 2) if x >= 0 else (div(abs(y), t) / 2, math.copysign(t, y))


def exp(z: C) -> C:
    return polar(math_call(math.exp, z[0]), z[1])


def expm1(z: C) -> C:
    x, y = z
    half_sin = math_call(math.sin, y / 2)
    return (
        math_call(math.expm1, x) * math_call(math.cos, y) - 2 * half_sin * half_sin,
        radial(math_call(math.exp, x), math_call(math.sin, y)),
    )


def log1p(z: C) -> C:
    x, y = z
    re = (
        0.5 * math_call(math.log1p, x * (2 + x) + y * y)
        if abs(x) < 0.5 and abs(y) < 0.5
        else _log_abs((1 + x, y))
    )
    return (re, math.atan2(y, 1 + x))


def pow(z: C, w: C) -> C:
    if w[0] == 0 and w[1] == 0:
        return (1.0, 0.0)
    if w[0] == 1 and w[1] == 0:
        return z
    if z[0] == 0 and z[1] == 0:
        if w[1] != 0 or math.isnan(w[0]):
            return (NAN, NAN)
        return polar(0.0 if w[0] > 0 else INF, w[0] * math.atan2(z[1], z[0]))
    if w[1] == 0:
        if z[1] == 0 and math.isfinite(w[0]) and w[0].is_integer():
            try:
                re = math.pow(z[0], w[0])
            except OverflowError:
                re = -INF if z[0] < 0 and w[0] % 2 != 0 else INF
            except ValueError:
                re = NAN
            return (re, math.copysign(0, z[1]))
        return polar(
            math_call(math.exp, _log_abs(z) * w[0]),
            math.atan2(z[1], z[0]) * w[0],
        )
    return exp(multiply(w, log(z)))


def cbrt(z: C) -> C:
    return polar(math_call(math.exp, _log_abs(z) / 3), math.atan2(z[1], z[0]) / 3)


def sin(z: C) -> C:
    x, y = z
    return (
        radial(math_call(math.cosh, y), math_call(math.sin, x)),
        radial(math_call(math.sinh, y), math_call(math.cos, x)),
    )


def cos(z: C) -> C:
    x, y = z
    return (
        radial(math_call(math.cosh, y), math_call(math.cos, x)),
        -radial(math_call(math.sinh, y), math_call(math.sin, x)),
    )


def sinh(z: C) -> C:
    x, y = z
    return (
        radial(math_call(math.sinh, x), math_call(math.cos, y)),
        radial(math_call(math.cosh, x), math_call(math.sin, y)),
    )


def cosh(z: C) -> C:
    x, y = z
    return (
        radial(math_call(math.cosh, x), math_call(math.cos, y)),
        radial(math_call(math.sinh, x), math_call(math.sin, y)),
    )


def tan(z: C) -> C:
    x, y = z
    t = math_call(math.exp, -2 * abs(y))
    cx, sx = math_call(math.cos, x), math_call(math.sin, x)
    denominator = (1 - t) * (1 - t) + 4 * t * cx * cx
    return (
        div(4 * t * sx * cx, denominator),
        math.copysign(div(-math_call(math.expm1, -4 * abs(y)), denominator), y),
    )


def tanh(z: C) -> C:
    a, b = tan((z[1], z[0]))
    return (b, a)


def asin(z: C) -> C:
    x, y = z
    if y == 0 and abs(x) <= 1:
        return (math.asin(x), y)
    s = maximum(abs(x), abs(y))
    if s > 1e150:
        return (math.atan2(x, abs(y)), math.copysign(_log_abs(z) + math.log(2), y))
    h, im = _inverse_geometry(x, y)
    return (math.atan2(x, h), im)


def _inverse_geometry(x: float, y: float) -> C:
    ax = abs(x)
    r, t = math.hypot(ax + 1, y), math.hypot(ax - 1, y)
    a = r / 2 + t / 2
    delta = (
        math.hypot(
            div(y, math_call(math.sqrt, r + (1 + ax))),
            div(y, math_call(math.sqrt, t + (1 - ax))),
        )
        / math.sqrt(2)
        if ax <= 1
        else math_call(math.sqrt, a - 1)
    )
    gap = (
        math.hypot(math_call(math.sqrt, 1 - ax), delta)
        if ax <= 1
        else math.hypot(
            div(y, math_call(math.sqrt, r + (ax + 1))),
            div(y, math_call(math.sqrt, t + (ax - 1))),
        )
        / math.sqrt(2)
    )
    return (
        gap * math_call(math.sqrt, a + ax),
        math.copysign(
            math_call(math.log1p, delta * (delta + math_call(math.sqrt, a + 1))),
            y,
        ),
    )


def acos(z: C) -> C:
    x, y = z
    if y == 0 and abs(x) <= 1:
        return (math.acos(x), -y)
    if maximum(abs(x), abs(y)) > 1e150:
        return (
            math.atan2(abs(y), x),
            -math.copysign(_log_abs(z) + math.log(2), y),
        )
    h, im = _inverse_geometry(x, y)
    return (math.atan2(h, x), -im)


def atan(z: C) -> C:
    x, y = z
    if y == 0:
        return (math.atan(x), y)
    re = (math.atan2(x, 1 - y) + math.atan2(x, 1 + y)) / 2
    h = math.hypot(x, y - 1)
    if h == 0:
        im = INF
    elif abs(y) < 0.5 * h:
        im = 0.25 * math_call(math.log1p, div(div(4 * y, h), h))
    else:
        im = (_log_abs((x, y + 1)) - _log_abs((x, y - 1))) / 2
    return (re, im)


def asinh(z: C) -> C:
    a, b = asin((-z[1], z[0]))
    return (b, -a)


def acosh(z: C) -> C:
    a, b = acos(z)
    return (abs(b), math.copysign(a, z[1]))


def atanh(z: C) -> C:
    a, b = atan((-z[1], z[0]))
    return (b, -a)
