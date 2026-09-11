import { isFinite, isNaN } from '../../../../helpers/utils.js';
import { copySign, inf, type C } from './utils.js';
const { abs, cos, sin, max, min } = Math;

// A vanishing Cartesian factor remains zero even when the radial factor overflows.
const radial = (r: number, x: number): number => (x === 0 && !isNaN(r) ? copySign(0, r) * x : r * x);

export const add = ([a, b]: C, [c, d]: C): C => [a + c, b + d];
export const subtract = ([a, b]: C, [c, d]: C): C => [a - c, b - d];
export const neg = ([a, b]: C): C => [-a, -b];
export const conj = ([a, b]: C): C => [a, -b];
/** Multiply Cartesian components, rescaling overflowing finite products. */
export function multiply([a, b]: C, [c, d]: C): C {
    if (b === 0 && d === 0) return [a * c, radial(a, d) + radial(c, b)];
    if ([a, b, c, d].every(isFinite) && [a * c, b * d, a * d, b * c].some((x) => !isFinite(x))) {
        const s = max(abs(a), abs(b)),
            t = max(abs(c), abs(d));
        return [
            ((a / s) * (c / t) - (b / s) * (d / t)) * min(s, t) * max(s, t),
            ((a / s) * (d / t) + (b / s) * (c / t)) * min(s, t) * max(s, t),
        ];
    }
    return [a * c - b * d, a * d + b * c];
} /** Divide a sum without overflowing the sum or prematurely underflowing its terms. */
function sumOver(x: number, y: number, q: number): number {
    const sum = x + y;
    return !isFinite(sum) && isFinite(x) && isFinite(y) ? x / q + y / q : sum / q;
} /** Scale the divisor before computing the Cartesian quotient. */
export function divide([a, b]: C, [c, d]: C): C {
    const s = max(abs(c), abs(d));
    if (s === 0) return [copySign(inf, c) * a, copySign(inf, c) * b];
    if (s === inf && isFinite(a) && isFinite(b)) {
        c = copySign(abs(c) === inf ? 1 : 0, c);
        d = copySign(abs(d) === inf ? 1 : 0, d);
        return [0 * (a * c + b * d), 0 * (b * c - a * d)];
    }
    c /= s;
    d /= s;
    const q = c * c + d * d;
    if ((!isFinite(a / s) || !isFinite(b / s)) && isFinite(a) && isFinite(b)) {
        const u = max(abs(a), abs(b));
        return [((((a / u) * c + (b / u) * d) / q) * u) / s, ((((b / u) * c - (a / u) * d) / q) * u) / s];
    }
    // Divide before adding to avoid overflow for large, balanced inputs.
    return [sumOver((a / s) * c, (b / s) * d, q), sumOver((b / s) * c, -(a / s) * d, q)];
}
export const polar = (r: number, theta: number): C => [radial(r, cos(theta)), radial(r, sin(theta))];
