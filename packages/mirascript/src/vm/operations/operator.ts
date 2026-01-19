import { toFormat } from '../../helpers/convert/index.js';
import { isNaN } from '../../helpers/utils.js';
import type { VmAny } from '../types/index.js';
import { $AssertInit } from './common.js';
import { $ToBoolean, $ToNumber, $ToString } from './convert.js';
import { isSame, overloadNumberString } from './utils.js';

// String operations
/** 字符串连接 */
export function $Concat(...args: readonly string[]): string {
    return args.map((a) => toFormat(a, null)).join('');
}

// Unary operations
/** 正号 */
export function $Pos(a: VmAny): number {
    return $ToNumber(a);
}
/** 负号 */
export function $Neg(a: VmAny): number {
    return -$ToNumber(a);
}
/** 非 */
export function $Not(a: VmAny): boolean {
    return !$ToBoolean(a);
}

// Math operations
/** 加法 */
export function $Add(a: VmAny, b: VmAny): number {
    return $ToNumber(a) + $ToNumber(b);
}
/** 减法 */
export function $Sub(a: VmAny, b: VmAny): number {
    return $ToNumber(a) - $ToNumber(b);
}
/** 乘法 */
export function $Mul(a: VmAny, b: VmAny): number {
    return $ToNumber(a) * $ToNumber(b);
}
/** 除法 */
export function $Div(a: VmAny, b: VmAny): number {
    return $ToNumber(a) / $ToNumber(b);
}
/** 取模 */
export function $Mod(a: VmAny, b: VmAny): number {
    return $ToNumber(a) % $ToNumber(b);
}
/** 乘方 */
export function $Pow(a: VmAny, b: VmAny): number {
    return $ToNumber(a) ** $ToNumber(b);
}

// Logical operations without short-circuiting
/** 与 */
export function $And(a: VmAny, b: VmAny): boolean {
    return $ToBoolean(a) && $ToBoolean(b);
}
/** 或 */
export function $Or(a: VmAny, b: VmAny): boolean {
    return $ToBoolean(a) || $ToBoolean(b);
}

// Comparison operations
/** 大于 */
export function $Gt(a: VmAny, b: VmAny): boolean {
    if (overloadNumberString(a, b)) {
        return $ToNumber(a) > $ToNumber(b);
    } else {
        return $ToString(a) > $ToString(b);
    }
}
/** 大于等于 */
export function $Gte(a: VmAny, b: VmAny): boolean {
    if (overloadNumberString(a, b)) {
        return $ToNumber(a) >= $ToNumber(b);
    } else {
        return $ToString(a) >= $ToString(b);
    }
}
/** 小于 */
export function $Lt(a: VmAny, b: VmAny): boolean {
    if (overloadNumberString(a, b)) {
        return $ToNumber(a) < $ToNumber(b);
    } else {
        return $ToString(a) < $ToString(b);
    }
}
/** 小于等于 */
export function $Lte(a: VmAny, b: VmAny): boolean {
    if (overloadNumberString(a, b)) {
        return $ToNumber(a) <= $ToNumber(b);
    } else {
        return $ToString(a) <= $ToString(b);
    }
}

// Equality operations
/** 等于 */
export function $Eq(a: VmAny, b: VmAny): boolean {
    $AssertInit(a);
    $AssertInit(b);
    // Number comparison is a special case to handle NaN correctly
    if (typeof a == 'number' && typeof b == 'number') return a === b;
    return isSame(a, b);
}
/** 不等于 */
export function $Neq(a: VmAny, b: VmAny): boolean {
    return !$Eq(a, b);
}

const { abs, min } = Math;
/** 近似等于 */
export function $Aeq(a: VmAny, b: VmAny): boolean {
    if (overloadNumberString(a, b)) {
        const an = $ToNumber(a);
        const bn = $ToNumber(b);
        const EPS = 1e-15;
        if (isNaN(an) || isNaN(bn)) return false;
        // Since Inf - Inf is NaN, we must check for equality first
        if (an === bn) return true;
        const absoluteDifference = abs(an - bn);
        if (absoluteDifference < EPS) return true;
        const base = min(abs(an), abs(bn));
        return absoluteDifference < base * EPS;
    } else {
        // For strings, we use normalized case-insensitive comparison
        const as = $ToString(a);
        const bs = $ToString(b);
        if (as === bs) return true;
        const ai = as.toLowerCase();
        const bi = bs.toLowerCase();
        if (ai === bi) return true;
        const an = ai.normalize('NFC');
        const bn = bi.normalize('NFC');
        return an === bn;
    }
}
/** 不近似等于 */
export function $Naeq(a: VmAny, b: VmAny): boolean {
    return !$Aeq(a, b);
}

/** 全等于 */
export function $Same(a: VmAny, b: VmAny): boolean {
    $AssertInit(a);
    $AssertInit(b);
    return isSame(a, b);
}
/** 不全等于 */
export function $Nsame(a: VmAny, b: VmAny): boolean {
    return !$Same(a, b);
}
