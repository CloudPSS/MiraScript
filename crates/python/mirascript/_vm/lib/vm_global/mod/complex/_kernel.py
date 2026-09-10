"""Cartesian kernels shared in semantics with the TypeScript and Rust VMs.

Python math raises on IEEE singularities. The small scalar adapters below keep
those numerical results separate from the VM's argument conversion errors.
"""

from __future__ import annotations

import math
from typing_extensions import Callable, Tuple

C = Tuple[float, float]
INF = math.inf
NAN = math.nan


def _math(f: Callable[[float], float], x: float) -> float:
    try:
        return f(x)
    except OverflowError:
        return math.copysign(INF, x) if f is math.sinh else INF
    except ValueError:
        if (f is math.log and x == 0) or (f is math.log1p and x == -1):
            return -INF
        return NAN


def _div(a: float, b: float) -> float:
    if b == 0:
        return (
            NAN
            if a == 0 or math.isnan(a)
            else math.copysign(INF, a) * math.copysign(1, b)
        )
    return a / b


def _max(a: float, b: float) -> float:
    return NAN if math.isnan(a) or math.isnan(b) else max(a, b)


def _min(a: float, b: float) -> float:
    return NAN if math.isnan(a) or math.isnan(b) else min(a, b)


def _radial(r: float, x: float) -> float:
    return math.copysign(0, r) * x if x == 0 and not math.isnan(r) else r * x


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
        return (a * c, _radial(a, d) + _radial(c, b))
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
        _div(x, q) + _div(y, q)
        if not math.isfinite(total) and math.isfinite(x) and math.isfinite(y)
        else _div(total, q)
    )


def divide(z: C, w: C) -> C:
    a, b = z
    c, d = w
    s = _max(abs(c), abs(d))
    if s == 0:
        return (math.copysign(INF, c) * a, math.copysign(INF, c) * b)
    if s == INF and math.isfinite(a) and math.isfinite(b):
        c = math.copysign(1 if abs(c) == INF else 0, c)
        d = math.copysign(1 if abs(d) == INF else 0, d)
        return (0 * (a * c + b * d), 0 * (b * c - a * d))
    c, d = _div(c, s), _div(d, s)
    q = c * c + d * d
    if (
        (not math.isfinite(_div(a, s)) or not math.isfinite(_div(b, s)))
        and math.isfinite(a)
        and math.isfinite(b)
    ):
        u = max(abs(a), abs(b))
        return (
            _div(_div(_div(a, u) * c + _div(b, u) * d, q) * u, s),
            _div(_div(_div(b, u) * c - _div(a, u) * d, q) * u, s),
        )
    return (
        _sum_over(_div(a, s) * c, _div(b, s) * d, q),
        _sum_over(_div(b, s) * c, -_div(a, s) * d, q),
    )


def polar(r: float, theta: float) -> C:
    return (_radial(r, _math(math.cos, theta)), _radial(r, _math(math.sin, theta)))


def _log_abs(z: C) -> float:
    x, y = z
    a, b = _max(abs(x), abs(y)), _min(abs(x), abs(y))
    if a == INF:
        return INF
    if a == 0:
        return -INF
    return _math(math.log, a) + 0.5 * _math(math.log1p, _div(b, a) * _div(b, a))


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
    s = _max(abs(x), abs(y))
    if s == 0:
        return (0.0, y)
    t = _math(math.sqrt, s) * _math(
        math.sqrt, (math.hypot(x / s, y / s) + abs(x) / s) / 2
    )
    return (t, _div(y, t) / 2) if x >= 0 else (_div(abs(y), t) / 2, math.copysign(t, y))


def exp(z: C) -> C:
    return polar(_math(math.exp, z[0]), z[1])


def expm1(z: C) -> C:
    x, y = z
    half_sin = _math(math.sin, y / 2)
    return (
        _math(math.expm1, x) * _math(math.cos, y) - 2 * half_sin * half_sin,
        _radial(_math(math.exp, x), _math(math.sin, y)),
    )


def log1p(z: C) -> C:
    x, y = z
    re = (
        0.5 * _math(math.log1p, x * (2 + x) + y * y)
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
        return polar(_math(math.exp, _log_abs(z) * w[0]), math.atan2(z[1], z[0]) * w[0])
    return exp(multiply(w, log(z)))


def cbrt(z: C) -> C:
    return polar(_math(math.exp, _log_abs(z) / 3), math.atan2(z[1], z[0]) / 3)


def sin(z: C) -> C:
    x, y = z
    return (
        _radial(_math(math.cosh, y), _math(math.sin, x)),
        _radial(_math(math.sinh, y), _math(math.cos, x)),
    )


def cos(z: C) -> C:
    x, y = z
    return (
        _radial(_math(math.cosh, y), _math(math.cos, x)),
        -_radial(_math(math.sinh, y), _math(math.sin, x)),
    )


def sinh(z: C) -> C:
    x, y = z
    return (
        _radial(_math(math.sinh, x), _math(math.cos, y)),
        _radial(_math(math.cosh, x), _math(math.sin, y)),
    )


def cosh(z: C) -> C:
    x, y = z
    return (
        _radial(_math(math.cosh, x), _math(math.cos, y)),
        _radial(_math(math.sinh, x), _math(math.sin, y)),
    )


def tan(z: C) -> C:
    x, y = z
    t = _math(math.exp, -2 * abs(y))
    cx, sx = _math(math.cos, x), _math(math.sin, x)
    denominator = (1 - t) * (1 - t) + 4 * t * cx * cx
    return (
        _div(4 * t * sx * cx, denominator),
        math.copysign(_div(-_math(math.expm1, -4 * abs(y)), denominator), y),
    )


def tanh(z: C) -> C:
    a, b = tan((z[1], z[0]))
    return (b, a)


def asin(z: C) -> C:
    x, y = z
    if y == 0 and abs(x) <= 1:
        return (math.asin(x), y)
    s = _max(abs(x), abs(y))
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
            _div(y, _math(math.sqrt, r + (1 + ax))),
            _div(y, _math(math.sqrt, t + (1 - ax))),
        )
        / math.sqrt(2)
        if ax <= 1
        else _math(math.sqrt, a - 1)
    )
    gap = (
        math.hypot(_math(math.sqrt, 1 - ax), delta)
        if ax <= 1
        else math.hypot(
            _div(y, _math(math.sqrt, r + (ax + 1))),
            _div(y, _math(math.sqrt, t + (ax - 1))),
        )
        / math.sqrt(2)
    )
    return (
        gap * _math(math.sqrt, a + ax),
        math.copysign(_math(math.log1p, delta * (delta + _math(math.sqrt, a + 1))), y),
    )


def acos(z: C) -> C:
    x, y = z
    if y == 0 and abs(x) <= 1:
        return (math.acos(x), -y)
    if _max(abs(x), abs(y)) > 1e150:
        return (math.atan2(abs(y), x), -math.copysign(_log_abs(z) + math.log(2), y))
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
        im = 0.25 * _math(math.log1p, _div(_div(4 * y, h), h))
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
