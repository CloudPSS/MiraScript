import { toFormat } from '../../helpers/convert/index.js';
import { isNaN } from '../../helpers/utils.js';
import type { VmAny } from '../types/index.js';
import { $AssertInit } from './common.js';
import { $ToBoolean, $ToNumber, $ToString } from './convert.js';
import { isSame, overloadNumberString } from './utils.js';

// String operations
/** 字符串连接 */
export function $Concat(...args: readonly VmAny[]): string {
    return args.map((a) => toFormat(a, null)).join('');
}

// Unary operations
/** 正号 */
export const $Pos = $ToNumber;
/** 负号 */
export const $Neg = (a: VmAny): number => -$ToNumber(a);
/** 非 */
export const $Not = (a: VmAny): boolean => !$ToBoolean(a);

// Math operations
const add = (a: VmAny, b: VmAny): number => $ToNumber(a) + $ToNumber(b);
/** 加法 */
export const $Add = (a: VmAny, b: VmAny): number => {
    if (typeof a == 'number' && typeof b == 'number') return a + b;
    return add(a, b);
};
const sub = (a: VmAny, b: VmAny): number => $ToNumber(a) - $ToNumber(b);
/** 减法 */
export const $Sub = (a: VmAny, b: VmAny): number => {
    if (typeof a == 'number' && typeof b == 'number') return a - b;
    return sub(a, b);
};
const mul = (a: VmAny, b: VmAny): number => $ToNumber(a) * $ToNumber(b);
/** 乘法 */
export const $Mul = (a: VmAny, b: VmAny): number => {
    if (typeof a == 'number' && typeof b == 'number') return a * b;
    return mul(a, b);
};
const div = (a: VmAny, b: VmAny): number => $ToNumber(a) / $ToNumber(b);
/** 除法 */
export const $Div = (a: VmAny, b: VmAny): number => {
    if (typeof a == 'number' && typeof b == 'number') return a / b;
    return div(a, b);
};
const mod = (a: VmAny, b: VmAny): number => $ToNumber(a) % $ToNumber(b);
/** 取模 */
export const $Mod = (a: VmAny, b: VmAny): number => {
    if (typeof a == 'number' && typeof b == 'number') return a % b;
    return mod(a, b);
};
const pow = (a: VmAny, b: VmAny): number => $ToNumber(a) ** $ToNumber(b);
/** 乘方 */
export const $Pow = (a: VmAny, b: VmAny): number => {
    if (typeof a == 'number' && typeof b == 'number') return a ** b;
    return pow(a, b);
};

// Logical operations without short-circuiting
const and = (a: VmAny, b: VmAny): boolean => $ToBoolean(a) && $ToBoolean(b);
/** 与 */
export const $And = (a: VmAny, b: VmAny): boolean => {
    if (typeof a == 'boolean' && typeof b == 'boolean') return a && b;
    return and(a, b);
};
const or = (a: VmAny, b: VmAny): boolean => $ToBoolean(a) || $ToBoolean(b);
/** 或 */
export const $Or = (a: VmAny, b: VmAny): boolean => {
    if (typeof a == 'boolean' && typeof b == 'boolean') return a || b;
    return or(a, b);
};

// Comparison operations
/** 大于 */
export const $Gt = (a: VmAny, b: VmAny): boolean => {
    if (overloadNumberString(a, b)) {
        return $ToNumber(a) > $ToNumber(b);
    } else {
        return $ToString(a) > $ToString(b);
    }
};
/** 大于等于 */
export const $Gte = (a: VmAny, b: VmAny): boolean => {
    if (overloadNumberString(a, b)) {
        return $ToNumber(a) >= $ToNumber(b);
    } else {
        return $ToString(a) >= $ToString(b);
    }
};
/** 小于 */
export const $Lt = (a: VmAny, b: VmAny): boolean => {
    if (overloadNumberString(a, b)) {
        return $ToNumber(a) < $ToNumber(b);
    } else {
        return $ToString(a) < $ToString(b);
    }
};
/** 小于等于 */
export const $Lte = (a: VmAny, b: VmAny): boolean => {
    if (overloadNumberString(a, b)) {
        return $ToNumber(a) <= $ToNumber(b);
    } else {
        return $ToString(a) <= $ToString(b);
    }
};

// Equality operations
/** 等于 */
export const $Eq = (a: VmAny, b: VmAny): boolean => {
    $AssertInit(a);
    $AssertInit(b);
    // Number comparison is a special case to handle NaN correctly
    if (typeof a == 'number' && typeof b == 'number') return a === b;
    return isSame(a, b);
};
/** 不等于 */
export const $Neq = (a: VmAny, b: VmAny): boolean => {
    return !$Eq(a, b);
};

const { abs, min } = Math;
/** 近似等于 */
export const $Aeq = (a: VmAny, b: VmAny): boolean => {
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
};
/** 不近似等于 */
export const $Naeq = (a: VmAny, b: VmAny): boolean => {
    return !$Aeq(a, b);
};

/** 全等于 */
export const $Same = (a: VmAny, b: VmAny): boolean => {
    $AssertInit(a);
    $AssertInit(b);
    return isSame(a, b);
};
/** 不全等于 */
export const $Nsame = (a: VmAny, b: VmAny): boolean => {
    return !$Same(a, b);
};
