/** Internal Cartesian pairs. Public values are records, never arrays. */
export type C = readonly [number, number];
const { abs, atan2, cos, sin, cosh, sinh, hypot, log, log1p, exp, expm1, sqrt, LN2 } = Math;
const inf = Infinity;
export const copySign = (x: number, y: number): number => (y < 0 || Object.is(y, -0) ? -abs(x) : abs(x));
// A vanishing Cartesian factor remains zero even when the radial factor overflows.
const radial = (r: number, x: number): number => (x === 0 && !Number.isNaN(r) ? copySign(0, r) * x : r * x);
/** Divide a sum without overflowing the sum or prematurely underflowing its terms. */
function sumOver(x: number, y: number, q: number): number {
    const sum = x + y;
    return !Number.isFinite(sum) && Number.isFinite(x) && Number.isFinite(y) ? x / q + y / q : sum / q;
}
export const add = ([a, b]: C, [c, d]: C): C => [a + c, b + d];
export const subtract = ([a, b]: C, [c, d]: C): C => [a - c, b - d];
export const neg = ([a, b]: C): C => [-a, -b];
export const conj = ([a, b]: C): C => [a, -b];
/** Multiply Cartesian components, rescaling overflowing finite products. */
export function multiply([a, b]: C, [c, d]: C): C {
    if (b === 0 && d === 0) return [a * c, radial(a, d) + radial(c, b)];
    if ([a, b, c, d].every(Number.isFinite) && [a * c, b * d, a * d, b * c].some((x) => !Number.isFinite(x))) {
        const s = Math.max(abs(a), abs(b)),
            t = Math.max(abs(c), abs(d));
        return [
            ((a / s) * (c / t) - (b / s) * (d / t)) * Math.min(s, t) * Math.max(s, t),
            ((a / s) * (d / t) + (b / s) * (c / t)) * Math.min(s, t) * Math.max(s, t),
        ];
    }
    return [a * c - b * d, a * d + b * c];
}
/** Scale the divisor before computing the Cartesian quotient. */
export function divide([a, b]: C, [c, d]: C): C {
    const s = Math.max(abs(c), abs(d));
    if (s === 0) return [copySign(inf, c) * a, copySign(inf, c) * b];
    if (s === inf && Number.isFinite(a) && Number.isFinite(b)) {
        c = copySign(abs(c) === inf ? 1 : 0, c);
        d = copySign(abs(d) === inf ? 1 : 0, d);
        return [0 * (a * c + b * d), 0 * (b * c - a * d)];
    }
    c /= s;
    d /= s;
    const q = c * c + d * d;
    if ((!Number.isFinite(a / s) || !Number.isFinite(b / s)) && Number.isFinite(a) && Number.isFinite(b)) {
        const u = Math.max(abs(a), abs(b));
        return [((((a / u) * c + (b / u) * d) / q) * u) / s, ((((b / u) * c - (a / u) * d) / q) * u) / s];
    }
    // Divide before adding to avoid overflow for large, balanced inputs.
    return [sumOver((a / s) * c, (b / s) * d, q), sumOver((b / s) * c, -(a / s) * d, q)];
}
export const polar = (r: number, theta: number): C => [radial(r, cos(theta)), radial(r, sin(theta))];
/** Compute log of the modulus without overflowing the modulus itself. */
export function logAbs([x, y]: C): number {
    const a = Math.max(abs(x), abs(y));
    const b = Math.min(abs(x), abs(y));
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
    if (x === inf) return [inf, Number.isNaN(y) ? NaN : copySign(0, y)];
    if (x === -inf) return [Number.isNaN(y) ? NaN : 0, copySign(inf, y)];
    const s = Math.max(abs(x), abs(y));
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
        if (w[1] !== 0 || Number.isNaN(w[0])) return [NaN, NaN];
        return polar(w[0] > 0 ? 0 : inf, w[0] * atan2(z[1], z[0]));
    }
    if (w[1] === 0) {
        // Preserve exact real integer powers, including their zero imaginary part.
        if (z[1] === 0 && Number.isInteger(w[0])) return [Math.pow(z[0], w[0]), copySign(0, z[1])];
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
    if (y === 0 && abs(x) <= 1) return [Math.asin(x), y];
    const s = Math.max(abs(x), abs(y));
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
    const delta = ax <= 1 ? hypot(y / sqrt(r + (1 + ax)), y / sqrt(t + (1 - ax))) / Math.SQRT2 : sqrt(a - 1);
    const gap =
        ax <= 1 ? hypot(sqrt(1 - ax), delta) : hypot(y / sqrt(r + (ax + 1)), y / sqrt(t + (ax - 1))) / Math.SQRT2;
    return [gap * sqrt(a + ax), copySign(log1p(delta * (delta + sqrt(a + 1))), y)];
}
/** Compute the principal inverse cosine without subtracting nearly equal angles. */
export function arcCosine([x, y]: C): C {
    if (y === 0 && abs(x) <= 1) return [Math.acos(x), -y];
    if (Math.max(abs(x), abs(y)) > 1e150) return [atan2(abs(y), x), -copySign(logAbs([x, y]) + LN2, y)];
    const [h, im] = inverseGeometry(x, y);
    return [atan2(h, x), -im];
}
/** Compute principal inverse tangent from angles and log moduli. */
export function arcTangent([x, y]: C): C {
    if (y === 0) return [Math.atan(x), y];
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
