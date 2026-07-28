/* eslint-disable no-loss-of-precision */

import { NotNumber, PositiveInfinity, isNaN } from '../../../../helpers/utils.js';
import { expectNumber, VmLib } from '../../helpers.js';
const { PI, exp, floor, cos, sin, pow, trunc } = Math;

const MEM = new DataView(new ArrayBuffer(8));
/**
 * Reinterpret the bits of a 64-bit float as a BigInt (unsigned 64-bit integer).
 * This mirrors Rust's `f64::to_bits()`.
 */
function f64ToBits(value: number): bigint {
    MEM.setFloat64(0, value, false);
    return MEM.getBigUint64(0, false);
}

/**
 * Reinterpret a BigInt (unsigned 64-bit integer) as a 64-bit float.
 * This mirrors Rust's `f64::from_bits()`.
 */
function f64FromBits(bits: bigint): number {
    MEM.setBigUint64(0, bits, false);
    return MEM.getFloat64(0, false);
}

// From https://crates.io/crates/libm

/*
"A Precision Approximation of the Gamma Function" - Cornelius Lanczos (1964)
"Lanczos Implementation of the Gamma Function" - Paul Godfrey (2001)
"An Analysis of the Lanczos Gamma Approximation" - Glendon Ralph Pugh (2004)

approximation method:

                        (x - 0.5)         S(x)
Gamma(x) = (x + g - 0.5)         *  ----------------
                                    exp(x + g - 0.5)

with
                 a1      a2      a3            aN
S(x) ~= [ a0 + ----- + ----- + ----- + ... + ----- ]
               x + 1   x + 2   x + 3         x + N

with a0, a1, a2, a3,.. aN constants which depend on g.
for x < 0 the following reflection formula is used:

Gamma(x)*Gamma(-x) = -pi/(x sin(pi x))

most ideas and constants are from boost and python
*/

/**
 * sin(pi * x) with x > 0, if sin(pi*x)==0 the sign is arbitrary.
 */
function sinpi(x: number): number {
    /* argument reduction: x = |x| mod 2 */
    /* spurious inexact when x is odd int */
    x = x * 0.5;
    x = 2 * (x - floor(x));

    /* reduce x into [-.25,.25] */
    let n = trunc(4 * x); // truncate to integer
    n = trunc((n + 1) / 2); // integer division
    x -= n * 0.5;

    x *= PI;

    switch (n) {
        case 1:
            return cos(x);
        case 2:
            return sin(-x);
        case 3:
            return -cos(x);
        default:
            // 0
            return sin(x);
    }
}

const N = 12;

// static const double g = 6.024680040776729583740234375;
const GMHALF = 5.524680040776729583740234375 as const;

const SNUM = [
    23_531_376_880.41075968857200767445163675473484680494, 42_919_803_642.649098768957899047001988850926355848959,
    35_711_959_237.355668049440185451547166705960488635843, 17_921_034_426.03720969991975575445893111267140326539,
    6_039_542_586.3520280050642916443072979210699388420708, 1_439_720_407.3117216736632230727949123939715485786772,
    248_874_557.86205415651146038641322942321632125127801, 31_426_415.58540019438061423162831820536287468498764,
    2_876_370.6289353724412254090516208496135991145378768, 186_056.26539522349504029498971604569928220784236328,
    8071.6720023658162106380029022722506138218516325024, 210.82427775157934587250973392071336271166969580291,
    2.5066282746310002701649081771338373386264310793408,
] as const;

const SDEN = [
    0, 39_916_800, 120_543_840, 150_917_976, 105_258_076, 45_995_730, 13_339_535, 2_637_558, 357_423, 32670, 1925, 66,
    1,
] as const;

/**
 * S(x) rational function for positive x.
 */
function s(x: number): number {
    let num = 0;
    let den = 0;

    /* to avoid overflow handle large x differently */
    if (x < 8) {
        for (let i = N; i >= 0; i--) {
            num = num * x + SNUM[i]!;
            den = den * x + SDEN[i]!;
        }
    } else {
        for (let i = 0; i <= N; i++) {
            num = num / x + SNUM[i]!;
            den = den / x + SDEN[i]!;
        }
    }

    return num / den;
}

/**
 * The [Gamma function](https://en.wikipedia.org/wiki/Gamma_function) (f64).
 */
function tgamma(x: number): number {
    const u = f64ToBits(x);
    const ix = Number((u >> 32n) & 0x7fff_ffffn);
    const sign = u >> 63n !== 0n;

    /* special cases */
    if (ix >= 0x7ff0_0000) {
        /* tgamma(nan)=nan, tgamma(inf)=inf, tgamma(-inf)=nan with invalid */
        return x + PositiveInfinity;
    }

    if (ix < (0x3ff - 54) << 20) {
        /* |x| < 2^-54: tgamma(x) ~ 1/x, +-0 raises div-by-zero */
        return 1 / x;
    }

    /* integer arguments */
    /* raise inexact when non-integer */
    if (x === floor(x)) {
        if (sign) {
            return NotNumber;
        }
        if (x <= FACT_MAX) {
            return fact(trunc(x) - 1);
        }
    }

    /* x >= 172: tgamma(x)=inf with overflow */
    /* x =< -184: tgamma(x)=+-0 with underflow */
    if (ix >= 0x4067_0000) {
        /* |x| >= 184 */
        if (sign) {
            if (floor(x) * 0.5 === floor(x * 0.5)) {
                return 0;
            } else {
                return -0;
            }
        }
        const x1p1023 = f64FromBits(0x7fe0_0000_0000_0000n); // 2^1023
        return x * x1p1023; // overflow to Infinity
    }

    const absx = sign ? -x : x;

    /* handle the error of x + g - 0.5 */
    const y = absx + GMHALF;
    let dy: number;
    if (absx > GMHALF) {
        dy = y - absx;
        dy -= GMHALF;
    } else {
        dy = y - GMHALF;
        dy -= absx;
    }

    let z = absx - 0.5;
    let r = s(absx) * exp(-y);

    if (x < 0) {
        /* reflection formula for negative x */
        /* sinpi(absx) is not 0, integers are already handled */
        r = -PI / (sinpi(absx) * absx * r);
        dy = -dy;
        z = -z;
    }

    r += (dy * (GMHALF + 0.5) * r) / y;
    const zHalf = pow(y, 0.5 * z);
    return r * zHalf * zHalf;
}

export const gamma = VmLib(
    (x) => {
        const n = expectNumber('x', x);
        return tgamma(n);
    },
    {
        summary: '返回 Gamma 函数的值',
        params: { x: { type: 'number', description: '要计算 Gamma 函数的数值' } },
        returns: { type: 'number' },
        examples: ['gamma(5) // 24'],
    },
);

const FACT_MAX = 171;
const FACT: Array<number | undefined> = [1, 1];
/** 计算整数的阶乘 */
function fact(n: number): number {
    const cached = FACT[n];
    if (cached != null) return cached;
    let r = 1n;
    for (let i = 2; i <= n; i++) {
        r *= BigInt(i);
        FACT[i] = Number(r);
    }
    return FACT[n]!;
}

export const factorial = VmLib(
    (x): number => {
        const n = expectNumber('x', x);
        if (isNaN(n) || n < 0) return NotNumber;
        return tgamma(n + 1);
    },
    {
        summary: '返回一个数的阶乘',
        params: { x: { type: 'number', description: '要计算阶乘的数值' } },
        returns: { type: 'number' },
        examples: ['factorial(5) // 120'],
    },
);
