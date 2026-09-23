import { isNaN, isInteger } from '../../../../helpers/utils.js';
import { multiply, polar } from './basic.js';
import { copySign, radial, inf, type C } from './utils.js';
const {
    abs,
    atan2,
    cos,
    sin,
    cosh,
    sinh,
    hypot,
    log,
    log1p,
    exp,
    expm1,
    sqrt,
    max,
    min,
    pow,
    asin,
    acos,
    atan,
    LN2,
    SQRT2,
} = Math;

/** Compute log of the modulus without overflowing the modulus itself. */
export function logAbs([x, y]: C): number {
    const a = max(abs(x), abs(y));
    const b = min(abs(x), abs(y));
    if (a === inf) return inf;
    if (a === 0) return -inf;
    return log(a) + 0.5 * log1p((b / a) ** 2);
}
export const logarithm = (z: C): C => [logAbs(z), atan2(z[1], z[0])];
export const logBase = (z: C, base: number): C => {
    const [re, im] = logarithm(z);
    return [re / base, im / base];
};
/** Compute the principal square root with scaled finite components. */
export function squareRoot([x, y]: C): C {
    if (abs(y) === inf) return [inf, y];
    if (x === inf) return [inf, isNaN(y) ? NaN : copySign(0, y)];
    if (x === -inf) return [isNaN(y) ? NaN : 0, copySign(inf, y)];
    const s = max(abs(x), abs(y));
    if (s === 0) return [0, y];
    const t = sqrt(s) * sqrt((hypot(x / s, y / s) + abs(x) / s) / 2);
    return x >= 0 ? [t, y / t / 2] : [abs(y) / t / 2, copySign(t, y)];
}
export const exponential = ([x, y]: C): C => polar(exp(x), y);
/** Retain small real and imaginary increments near zero. */
export function exponentialMinusOne([x, y]: C): C {
    return [expm1(x) * cos(y) - 2 * sin(y / 2) ** 2, radial(exp(x), sin(y))];
}
/** Use log1p near zero to avoid subtractive cancellation. */
export function logarithmOnePlus([x, y]: C): C {
    const re = abs(x) < 0.5 && abs(y) < 0.5 ? 0.5 * log1p(x * (2 + x) + y * y) : logAbs([1 + x, y]);
    return [re, atan2(y, 1 + x)];
}
/** Evaluate the principal power, handling zero bases and real integer powers. */
export function power(z: C, w: C): C {
    if (w[0] === 0 && w[1] === 0) return [1, 0];
    if (w[0] === 1 && w[1] === 0) return z;
    if (z[0] === 0 && z[1] === 0) {
        if (w[1] !== 0 || isNaN(w[0])) return [NaN, NaN];
        return polar(w[0] > 0 ? 0 : inf, w[0] * atan2(z[1], z[0]));
    }
    if (w[1] === 0) {
        // Preserve exact real integer powers, including their zero imaginary part.
        if (z[1] === 0 && isInteger(w[0])) return [pow(z[0], w[0]), copySign(0, z[1])];
        return polar(exp(logAbs(z) * w[0]), atan2(z[1], z[0]) * w[0]);
    }
    return exponential(multiply(w, logarithm(z)));
}
export const cubeRoot = (z: C): C => polar(exp(logAbs(z) / 3), atan2(z[1], z[0]) / 3);
export const sine = ([x, y]: C): C => [radial(cosh(y), sin(x)), radial(sinh(y), cos(x))];
export const cosine = ([x, y]: C): C => [radial(cosh(y), cos(x)), -radial(sinh(y), sin(x))];
export const hyperbolicSine = ([x, y]: C): C => [radial(sinh(x), cos(y)), radial(cosh(x), sin(y))];
export const hyperbolicCosine = ([x, y]: C): C => [radial(cosh(x), cos(y)), radial(sinh(x), sin(y))];
/** Use a decaying exponential to avoid hyperbolic overflow. */
export function tangent([x, y]: C): C {
    const t = exp(-2 * abs(y));
    const denominator = (1 - t) ** 2 + 4 * t * cos(x) ** 2;
    return [(4 * t * sin(x) * cos(x)) / denominator, copySign(-expm1(-4 * abs(y)) / denominator, y)];
}
/** Rotate the stable tangent kernel into the hyperbolic plane. */
export function hyperbolicTangent([x, y]: C): C {
    const [a, b] = tangent([y, x]);
    return [b, a];
}
/** Geometric inverse avoids cancellation from log(z + sqrt(z*z + 1)). */
export function arcSine([x, y]: C): C {
    if (y === 0 && abs(x) <= 1) return [asin(x), y];
    const s = max(abs(x), abs(y));
    if (s > 1e150) {
        const angle = atan2(x, abs(y));
        return [angle, copySign(logAbs([x, y]) + LN2, y)];
    }
    const [h, im] = inverseGeometry(x, y);
    return [atan2(x, h), im];
}
/** Compute square roots of the small geometric differences before they underflow. */
function inverseGeometry(x: number, y: number): C {
    const ax = abs(x),
        r = hypot(ax + 1, y),
        t = hypot(ax - 1, y),
        a = r / 2 + t / 2;
    const delta = ax <= 1 ? hypot(y / sqrt(r + (1 + ax)), y / sqrt(t + (1 - ax))) / SQRT2 : sqrt(a - 1);
    const gap = ax <= 1 ? hypot(sqrt(1 - ax), delta) : hypot(y / sqrt(r + (ax + 1)), y / sqrt(t + (ax - 1))) / SQRT2;
    return [gap * sqrt(a + ax), copySign(log1p(delta * (delta + sqrt(a + 1))), y)];
}
/** Compute the principal inverse cosine without subtracting nearly equal angles. */
export function arcCosine([x, y]: C): C {
    if (y === 0 && abs(x) <= 1) return [acos(x), -y];
    if (max(abs(x), abs(y)) > 1e150) return [atan2(abs(y), x), -copySign(logAbs([x, y]) + LN2, y)];
    const [h, im] = inverseGeometry(x, y);
    return [atan2(h, x), -im];
}
/** Compute principal inverse tangent from angles and log moduli. */
export function arcTangent([x, y]: C): C {
    if (y === 0) return [atan(x), y];
    const re = (atan2(x, 1 - y) + atan2(x, 1 + y)) / 2;
    const h = hypot(x, y - 1);
    const im =
        h === 0
            ? inf
            : abs(y) < 0.5 * h
              ? 0.25 * log1p((4 * y) / h / h)
              : (logAbs([x, y + 1]) - logAbs([x, y - 1])) / 2;
    return [re, im];
}
/** Rotate the principal inverse sine while preserving cut-side zeros. */
export function areaHyperbolicSine([x, y]: C): C {
    const [a, b] = arcSine([-y, x]);
    return [b, -a];
}
/** Select the nonnegative-real branch of inverse hyperbolic cosine. */
export function areaHyperbolicCosine(z: C): C {
    const [a, b] = arcCosine(z);
    return [abs(b), copySign(a, z[1])];
}
/** Rotate the principal inverse tangent while preserving cut-side zeros. */
export function areaHyperbolicTangent([x, y]: C): C {
    const [a, b] = arcTangent([-y, x]);
    return [b, -a];
}
