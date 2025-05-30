import { VmError } from './error.js';
import { VmExtern } from './extern.js';
import { VmModule } from './module.js';
import {
    isVmArray,
    isVmRecord,
    type TypeName,
    type VmAny,
    type VmArray,
    type VmConst,
    type VmImmutable,
    type VmRecord,
    type VmValue,
} from './types.js';
import { VmWrapper } from './wrapper.js';

/**
 * 将只读属性转换为可写属性
 */
type Mutable<T> = { -readonly [K in keyof T]: T[K] };
const { hasOwn, keys } = Object;
const { isNaN } = Number;
const { abs, min } = Math;
const isSame = (a: VmValue, b: VmValue): boolean => {
    // Check all primitive types, and fast path for reference equality
    if (a === b) return true;
    // Check for NaN
    if (typeof a == 'number' && typeof b == 'number') return a === b || (isNaN(a) && isNaN(b));
    // Any primitives arrive here are not equal
    if (typeof a != 'object' || typeof b != 'object') return false;
    // Handle nil values
    if (a == null || b == null) return false;
    // Handle wrapper values
    if (a instanceof VmWrapper) return a.same(b);
    if (b instanceof VmWrapper) return b.same(a);
    // Handle array values
    if (isVmArray(a) && isVmArray(b)) {
        // Compare array items
        if (a.length !== b.length) return false;
        for (let i = 0; i < a.length; i++) {
            if (!isSame(a[i] ?? null, b[i] ?? null)) return false;
        }
        return true;
    }
    // Handle record values
    if (!isVmArray(a) && !isVmArray(b)) {
        // Compare record fields
        const aKeys = keys(a);
        const bKeys = keys(b);
        if (aKeys.length !== bKeys.length) return false;
        for (const key of aKeys) {
            if (!hasOwn(b, key) || !isSame(a[key] ?? null, b[key] ?? null)) return false;
        }
        return true;
    }
    return false;
};

export const $Mul = (a: VmAny, b: VmAny): number => $ToNumber(a) * $ToNumber(b);
export const $Add = (a: VmAny, b: VmAny): number => $ToNumber(a) + $ToNumber(b);
export const $Sub = (a: VmAny, b: VmAny): number => $ToNumber(a) - $ToNumber(b);
export const $Div = (a: VmAny, b: VmAny): number => $ToNumber(a) / $ToNumber(b);
export const $Pow = (a: VmAny, b: VmAny): number => $ToNumber(a) ** $ToNumber(b);
export const $Gt = (a: VmAny, b: VmAny): boolean => $ToNumber(a) > $ToNumber(b);
export const $Gte = (a: VmAny, b: VmAny): boolean => $ToNumber(a) >= $ToNumber(b);
export const $Lt = (a: VmAny, b: VmAny): boolean => $ToNumber(a) < $ToNumber(b);
export const $Lte = (a: VmAny, b: VmAny): boolean => $ToNumber(a) <= $ToNumber(b);
export const $Eq = (a: VmAny, b: VmAny): boolean => {
    $Init(a);
    $Init(b);
    if (typeof a == 'number' && typeof b == 'number') return a === b;
    return isSame(a, b);
};
export const $Neq = (a: VmAny, b: VmAny): boolean => !$Eq(a, b);
export const $Aeq = (a: VmAny, b: VmAny): boolean => {
    const an = $ToNumber(a);
    const bn = $ToNumber(b);
    const EPS = 1e-15;
    if (isNaN(an) || isNaN(bn)) return false;
    const absoluteDifference = abs(an - bn);
    if (absoluteDifference < EPS) return true;
    const base = min(abs(an), abs(bn));
    return absoluteDifference < base * EPS;
};
export const $Naeq = (a: VmAny, b: VmAny): boolean => !$Aeq(a, b);
export const $Same = (a: VmAny, b: VmAny): boolean => {
    $Init(a);
    $Init(b);
    return isSame(a, b);
};
export const $Nsame = (a: VmAny, b: VmAny): boolean => !$Same(a, b);
export const $In = (value: VmAny, iterable: VmAny): boolean => {
    $Init(value);
    $Init(iterable);
    if (iterable == null) return false;
    if (isVmArray(iterable)) return iterable.includes(value as VmConst);
    if (iterable instanceof VmExtern) return iterable.has($ToString(value));
    if (typeof iterable == 'object') return hasOwn(iterable, $ToString(value));
    return false;
};
export const $Concat = (...args: string[]): string => {
    return args.map($ToString).join('');
};
export const $Pos = (a: VmAny): number => $ToNumber(a);
export const $Neg = (a: VmAny): number => -$ToNumber(a);
export const $Not = (a: VmAny): boolean => !$ToBool(a);
export const $Init: (value: VmAny) => asserts value is VmValue = (value) => {
    if (value === undefined) throw new VmError(`Uninitialized value`);
};
export const $CallDyn = (func: (...args: unknown[]) => unknown, ...args: unknown[]): unknown => {
    if (func instanceof VmExtern) {
        func = func.value as unknown as (...args: unknown[]) => unknown;
    }
    if (typeof func != 'function') {
        throw new TypeError(`Expected function, got ${$Type(func)}`);
    }
    return func(...args) ?? null;
};
export const $Type = (value: VmAny): TypeName => {
    if (value === undefined) return 'nil';
    if (value === null) return 'nil';
    if (value instanceof VmExtern) return 'extern';
    if (value instanceof VmModule) return 'module';
    if (Array.isArray(value)) return 'array';
    if (typeof value == 'object') return 'record';
    return typeof value as TypeName;
};
export const $ToBool = (value: VmAny): boolean => {
    $Init(value);
    return value != null && value !== false;
};

/** 将值转为字符串 */
function innerToString(value: VmAny): string {
    if (value == null) return 'nil';
    if (value instanceof VmModule || value instanceof VmExtern) return value.toString();
    if (Array.isArray(value)) return `[${value.map(innerToString).join(', ')}]`;
    if (typeof value == 'object') return JSON.stringify(value);
    if (typeof value == 'function') return `<function ${value.name}>`;
    return String(value);
}
export const $ToString = (value: VmAny): string => {
    $Init(value);
    if (value === null) return '';
    if (Array.isArray(value)) return value.map(innerToString).join(', ');
    return innerToString(value);
};
export const $ToNumber = Number;
export const $NonNil = (value: unknown): asserts value is NonNullable<unknown> => {
    if (value === null) throw new Error('Expected non-nil value');
};
export const $Get = (obj: VmAny, key: VmAny): VmValue => {
    $Init(obj);
    const pk = $ToString(key);
    if (obj == null || typeof obj != 'object') return null;
    if (obj instanceof VmWrapper) return obj.get(pk) ?? null;
    if (!hasOwn(obj, pk)) return null;
    return (obj as Record<string, VmImmutable>)[pk] ?? null;
};
export const $ArrayRange = (start: VmAny, end: VmAny): VmArray => {
    const arr = [];
    const s = $ToNumber(start);
    const e = $ToNumber(end);
    for (let i = s; i <= e; i++) {
        arr.push(i);
    }
    return arr;
};
export const $ArrayRangeExclusive = (start: VmAny, end: VmAny): VmArray => {
    const arr = [];
    const s = $ToNumber(start);
    const e = $ToNumber(end);
    for (let i = s; i < e; i++) {
        arr.push(i);
    }
    return arr;
};
export const $ArraySpread = (array: VmAny): Iterable<VmValue> => {
    $Init(array);
    if (!isVmArray(array)) throw new VmError(`Expected array, got ${$Type(array)}`);
    return array;
};
export const $ArrayFreeze = (array: Mutable<VmArray>): void => {
    // Noop
};
export const $RecordSpread = (record: VmAny): VmRecord | null => {
    $Init(record);
    if (!isVmRecord(record)) return null;
    return record;
};
export const $RecordFreeze = (record: Mutable<VmRecord>, optional: readonly string[]): void => {
    for (const field of optional) {
        if (record[field] == null) {
            delete record[field];
        }
    }
};
export const $Iterable = (value: VmAny): Iterable<VmValue> => {
    $Init(value);
    if (value instanceof VmWrapper) return value.keys();
    if (isVmArray(value)) return value;
    if (value != null && typeof value == 'object') return keys(value);
    if (typeof value == 'string') return value;
    throw new VmError(`Value is not iterable`);
};

let cp = Number.NaN;
let cpTimeout = 100; // Default timeout in milliseconds
export const $Cp = (): void => {
    if (!cp) {
        cp = Date.now();
    } else if (Date.now() - cp > cpTimeout) {
        throw new VmError('Execution timeout');
    }
};
export const $ClearCp = (): void => {
    cp = Number.NaN;
};
export const $ConfigCp = (timeout?: number): void => {
    cpTimeout = timeout ?? 100;
};
