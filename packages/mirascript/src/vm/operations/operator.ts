import { toFormat } from '../../helpers/convert/index.js';
import { isNaN } from '../../helpers/utils.js';
import type { VmAny } from '../types/index.js';
import { $AssertInit } from './common.js';
import { $ToBoolean, $ToNumber, $ToString } from './convert.js';
import { isSame, overloadNumberString } from './utils.js';

// String operations
export const $Concat = (...args: readonly string[]): string => {
    return args.map((a) => toFormat(a, null)).join('');
};

// Unary operations
export const $Pos = (a: VmAny): number => {
    return $ToNumber(a);
};
export const $Neg = (a: VmAny): number => {
    return -$ToNumber(a);
};
export const $Not = (a: VmAny): boolean => {
    return !$ToBoolean(a);
};

// Math operations
export const $Add = (a: VmAny, b: VmAny): number => {
    return $ToNumber(a) + $ToNumber(b);
};
export const $Sub = (a: VmAny, b: VmAny): number => {
    return $ToNumber(a) - $ToNumber(b);
};
export const $Mul = (a: VmAny, b: VmAny): number => {
    return $ToNumber(a) * $ToNumber(b);
};
export const $Div = (a: VmAny, b: VmAny): number => {
    return $ToNumber(a) / $ToNumber(b);
};
export const $Mod = (a: VmAny, b: VmAny): number => {
    return $ToNumber(a) % $ToNumber(b);
};
export const $Pow = (a: VmAny, b: VmAny): number => {
    return $ToNumber(a) ** $ToNumber(b);
};

// Logical operations without short-circuiting
export const $And = (a: VmAny, b: VmAny): boolean => {
    return $ToBoolean(a) && $ToBoolean(b);
};
export const $Or = (a: VmAny, b: VmAny): boolean => {
    return $ToBoolean(a) || $ToBoolean(b);
};

// Comparison operations
export const $Gt = (a: VmAny, b: VmAny): boolean => {
    if (overloadNumberString(a, b)) {
        return $ToNumber(a) > $ToNumber(b);
    } else {
        return $ToString(a) > $ToString(b);
    }
};
export const $Gte = (a: VmAny, b: VmAny): boolean => {
    if (overloadNumberString(a, b)) {
        return $ToNumber(a) >= $ToNumber(b);
    } else {
        return $ToString(a) >= $ToString(b);
    }
};
export const $Lt = (a: VmAny, b: VmAny): boolean => {
    if (overloadNumberString(a, b)) {
        return $ToNumber(a) < $ToNumber(b);
    } else {
        return $ToString(a) < $ToString(b);
    }
};
export const $Lte = (a: VmAny, b: VmAny): boolean => {
    if (overloadNumberString(a, b)) {
        return $ToNumber(a) <= $ToNumber(b);
    } else {
        return $ToString(a) <= $ToString(b);
    }
};

// Equality operations
export const $Eq = (a: VmAny, b: VmAny): boolean => {
    $AssertInit(a);
    $AssertInit(b);
    // Number comparison is a special case to handle NaN correctly
    if (typeof a == 'number' && typeof b == 'number') return a === b;
    return isSame(a, b);
};
export const $Neq = (a: VmAny, b: VmAny): boolean => {
    return !$Eq(a, b);
};

const { abs, min } = Math;
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
export const $Naeq = (a: VmAny, b: VmAny): boolean => {
    return !$Aeq(a, b);
};

export const $Same = (a: VmAny, b: VmAny): boolean => {
    $AssertInit(a);
    $AssertInit(b);
    return isSame(a, b);
};
export const $Nsame = (a: VmAny, b: VmAny): boolean => {
    return !$Same(a, b);
};
