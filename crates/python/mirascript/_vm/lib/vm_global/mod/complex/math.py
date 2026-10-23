"""Complex standard-library functions with MiraScript special-value semantics."""

from __future__ import annotations

import cmath
import math

from typing_extensions import Callable

from .basic import multiply, polar
from .utils import INF, NAN, math_call, radial


def log(z: complex) -> complex:
    if cmath.isnan(z):
        return complex(NAN, NAN)
    if z == 0:
        return complex(-INF, cmath.phase(z))
    return cmath.log(z)


def log_base(z: complex, base: float) -> complex:
    w = log(z)
    return complex(w.real / base, w.imag / base)


def exp(z: complex) -> complex:
    if cmath.isnan(z) or not math.isfinite(z.imag):
        return complex(NAN, NAN)
    try:
        return cmath.exp(z)
    except OverflowError:
        return polar(INF, z.imag)


def expm1(z: complex) -> complex:
    # cmath has no expm1; subtracting one from exp loses tiny real components.
    x, y = z.real, z.imag
    half_sin = math_call(math.sin, y / 2)
    return complex(
        math_call(math.expm1, x) * math_call(math.cos, y) - 2 * half_sin * half_sin,
        radial(math_call(math.exp, x), math_call(math.sin, y)),
    )


def log1p(z: complex) -> complex:
    # Keep log1p's near-zero accuracy and the sign of an imaginary zero.
    x, y = z.real, z.imag
    if abs(x) < 0.5 and abs(y) < 0.5:
        return complex(0.5 * math.log1p(x * (2 + x) + y * y), math.atan2(y, 1 + x))
    return log(complex(1 + x, y))


def pow(z: complex, w: complex) -> complex:
    if w == 0:
        return complex(1.0, 0.0)
    if w == 1:
        return z
    if z == 0:
        if w.imag != 0 or math.isnan(w.real):
            return complex(NAN, NAN)
        return polar(0.0 if w.real > 0 else INF, w.real * cmath.phase(z))
    if w.imag == 0 and z.imag == 0 and math.isfinite(w.real) and w.real.is_integer():
        try:
            re = math.pow(z.real, w.real)
        except OverflowError:
            re = -INF if z.real < 0 and w.real % 2 != 0 else INF
        except ValueError:
            re = NAN
        return complex(re, math.copysign(0, z.imag))
    if cmath.isfinite(z) and cmath.isfinite(w):
        try:
            result = z**w
            if cmath.isfinite(result) and result.real != 0 and result.imag != 0:
                return result
        except (OverflowError, ZeroDivisionError):
            pass
    if w.imag == 0:
        return polar(math_call(math.exp, log(z).real * w.real), cmath.phase(z) * w.real)
    return exp(multiply(w, log(z)))


def cbrt(z: complex) -> complex:
    return polar(math_call(math.exp, log(z).real / 3), cmath.phase(z) / 3)


def _trig(f: Callable[[complex], complex], z: complex) -> complex:
    if cmath.isnan(z) or not math.isfinite(z.real):
        return complex(NAN, NAN)
    try:
        return f(z)
    except OverflowError:
        # Preserve zero components when the hyperbolic factor overflows.
        x, y = z.real, z.imag
        if f is cmath.sin:
            return complex(
                radial(math_call(math.cosh, y), math.sin(x)),
                radial(math_call(math.sinh, y), math.cos(x)),
            )
        return complex(
            radial(math_call(math.cosh, y), math.cos(x)),
            -radial(math_call(math.sinh, y), math.sin(x)),
        )


def sin(z: complex) -> complex:
    return _trig(cmath.sin, z)


def cos(z: complex) -> complex:
    return _trig(cmath.cos, z)


def tan(z: complex) -> complex:
    if cmath.isnan(z) or not math.isfinite(z.real):
        return complex(NAN, NAN)
    return cmath.tan(z)


def sinh(z: complex) -> complex:
    if cmath.isnan(z) or not math.isfinite(z.imag):
        return complex(NAN, NAN)
    try:
        return cmath.sinh(z)
    except OverflowError:
        w = sin(complex(-z.imag, z.real))
        return complex(w.imag, -w.real)


def cosh(z: complex) -> complex:
    if cmath.isnan(z) or not math.isfinite(z.imag):
        return complex(NAN, NAN)
    try:
        return cmath.cosh(z)
    except OverflowError:
        return cos(complex(-z.imag, z.real))


def tanh(z: complex) -> complex:
    if cmath.isnan(z) or not math.isfinite(z.imag):
        return complex(NAN, NAN)
    return cmath.tanh(z)


def asin(z: complex) -> complex:
    if cmath.isnan(z):
        return complex(NAN, math.copysign(INF, z.imag) if cmath.isinf(z) else NAN)
    return cmath.asin(z)


def acos(z: complex) -> complex:
    if cmath.isnan(z):
        return complex(NAN, -math.copysign(INF, z.imag) if cmath.isinf(z) else NAN)
    return cmath.acos(z)


def atan(z: complex) -> complex:
    x, y = z.real, z.imag
    if y != 0 and not cmath.isfinite(z):
        if math.isinf(y) or cmath.isnan(z) or not math.isfinite(4 * y):
            return complex((math.atan2(x, 1 - y) + math.atan2(x, 1 + y)) / 2, NAN)
    if x == 0 and abs(y) == 1:
        return complex(x, math.copysign(INF, y))
    h = math.hypot(x, y - 1)
    # Retain the shared runtime's rounded log1p singularity near -i.
    if h != 0 and abs(y) < 0.5 * h and (4 * y / h) / h == -1:
        return complex(cmath.atan(z).real, -INF)
    return cmath.atan(z)


def asinh(z: complex) -> complex:
    if cmath.isnan(z) and not cmath.isinf(z):
        return complex(NAN, NAN)
    return cmath.asinh(z)


def acosh(z: complex) -> complex:
    if cmath.isnan(z) and not cmath.isinf(z):
        return complex(NAN, NAN)
    return cmath.acosh(z)


def atanh(z: complex) -> complex:
    if not cmath.isfinite(z) or (z.imag == 0 and abs(z.real) >= 0.5):
        w = atan(complex(-z.imag, z.real))
        return complex(w.imag, -w.real)
    return cmath.atanh(z)
