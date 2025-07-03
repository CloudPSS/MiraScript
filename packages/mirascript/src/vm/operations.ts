import { VmError } from './error.js';
import {
    isVmArray,
    VmModule,
    VmExtern,
    isVmRecord,
    getVmFunctionInfo,
    isVmPrimitive,
    type TypeName,
    type VmAny,
    type VmImmutable,
    type VmRecord,
    type VmValue,
    isVmFunction,
    type VmArray,
    type VmConst,
    isVmExtern,
} from './types/index.js';
import { VmWrapper } from './types/wrapper.js';

const { hasOwn, keys, create } = Object;
const { isNaN, isSafeInteger } = Number;
const { abs, min, trunc, ceil } = Math;
const { slice, at } = Array.prototype;

const isSame = (a: VmValue, b: VmValue): boolean => {
    // Check for NaN
    if (typeof a == 'number' && typeof b == 'number') return a === b || (isNaN(a) && isNaN(b));
    // Check all primitive types, and fast path for reference equality
    if (a === b) return true;
    // Any primitives and functions arrive here are not equal
    if (a == null || typeof a != 'object' || b == null || typeof b != 'object') return false;
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

const overloadNumberString = (a: VmAny, b: VmAny): boolean => {
    const useNumber = typeof a == 'number' || typeof b == 'number' || !(typeof a == 'string' || typeof b == 'string');
    return useNumber;
};

// Math operations
export const $Add = (a: VmAny, b: VmAny): number => $ToNumber(a) + $ToNumber(b);
export const $Sub = (a: VmAny, b: VmAny): number => $ToNumber(a) - $ToNumber(b);
export const $Mul = (a: VmAny, b: VmAny): number => $ToNumber(a) * $ToNumber(b);
export const $Div = (a: VmAny, b: VmAny): number => $ToNumber(a) / $ToNumber(b);
export const $Mod = (a: VmAny, b: VmAny): number => $ToNumber(a) % $ToNumber(b);
export const $Pow = (a: VmAny, b: VmAny): number => $ToNumber(a) ** $ToNumber(b);
// Logical operations without short-circuiting
export const $And = (a: VmAny, b: VmAny): boolean => $ToBoolean(a) && $ToBoolean(b);
export const $Or = (a: VmAny, b: VmAny): boolean => $ToBoolean(a) || $ToBoolean(b);
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
export const $Eq = (a: VmAny, b: VmAny): boolean => {
    $AssertInit(a);
    $AssertInit(b);
    // Number comparison is a special case to handle NaN correctly
    if (typeof a == 'number' && typeof b == 'number') return a === b;
    return isSame(a, b);
};
export const $Neq = (a: VmAny, b: VmAny): boolean => !$Eq(a, b);
const stringComparer = new Intl.Collator('en', {
    usage: 'sort',
    numeric: false,
    sensitivity: 'base',
});
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
        // For strings, we use localeCompare for case-insensitive accent-insensitive comparison
        const as = $ToString(a);
        const bs = $ToString(b);
        if (as === bs) return true;
        return stringComparer.compare(as, bs) === 0;
    }
};
export const $Naeq = (a: VmAny, b: VmAny): boolean => !$Aeq(a, b);
export const $Same = (a: VmAny, b: VmAny): boolean => {
    $AssertInit(a);
    $AssertInit(b);
    return isSame(a, b);
};
export const $Nsame = (a: VmAny, b: VmAny): boolean => !$Same(a, b);
export const $In = (value: VmAny, iterable: VmAny): boolean => {
    $AssertInit(value);
    $AssertInit(iterable);
    if (iterable == null) return false;
    if (isVmArray(iterable)) {
        // JS %SameValueZero is same with `isSame` in this context
        if (isVmPrimitive(value)) return iterable.includes(value);
        return iterable.some((item) => isSame(item, value));
    }
    if (iterable instanceof VmExtern) return iterable.has($ToString(value));
    if (typeof iterable == 'object') return hasOwn(iterable, $ToString(value));
    return false;
};
export const $Concat = (...args: string[]): string => {
    return args.map($ToString).join('');
};
export const $Pos = (a: VmAny): number => $ToNumber(a);
export const $Neg = (a: VmAny): number => -$ToNumber(a);
export const $Not = (a: VmAny): boolean => !$ToBoolean(a);
export const $Length = (value: VmAny): number => {
    $AssertInit(value);
    if (isVmArray(value)) return value.length;
    if (isVmRecord(value)) return keys(value).length;
    if (value instanceof VmWrapper) {
        return value.keys().length;
    }
    return Number.NaN;
};
export const $Omit = (value: VmAny, omitted: ReadonlyArray<string | number>): VmRecord => {
    $AssertInit(value);
    if (value == null || !isVmRecord(value)) return {};
    const result: Record<string, VmConst> = {};
    const valueKeys = keys(value);
    const omittedSet = new Set(omitted.map($ToString));
    for (const key of valueKeys) {
        if (!omittedSet.has(key)) {
            result[key] = value[key] ?? null;
        }
    }
    return result;
};
export const $Pick = (value: VmAny, picked: ReadonlyArray<string | number>): VmRecord => {
    $AssertInit(value);
    if (value == null || !isVmRecord(value)) return {};
    const result: Record<string, VmConst> = {};
    for (const key of picked) {
        const k = $ToString(key);
        if (hasOwn(value, k)) {
            result[k] = value[k] ?? null;
        }
    }
    return result;
};

const sliceCore = (value: VmArray, start: number, end: number, exclusive: boolean): VmArray => {
    const { length } = value;

    if (isNaN(start)) start = 0;
    else if (start < 0) start = length + start;
    if (isNaN(end)) end = exclusive ? length : length - 1;
    else if (end < 0) end = length + end;

    start = ceil(start);
    if (exclusive || !isSafeInteger(end)) {
        end = ceil(end);
    } else {
        end = end + 1;
    }
    return slice.call(value, start, end) as VmArray;
};
export const $Slice = (value: VmAny, start: VmAny, end: VmAny): VmArray => {
    $AssertInit(value);
    if (!isVmArray(value)) throw new VmError(`Expected array, got ${$Type(value)}`, []);
    const s = start != null ? $ToNumber(start) : 0;
    const e = end != null ? $ToNumber(end) : value.length - 1;
    return sliceCore(value, s, e, false);
};
export const $SliceExclusive = (value: VmAny, start: VmAny, end: VmAny): VmArray => {
    $AssertInit(value);
    if (!isVmArray(value)) throw new VmError(`Expected array, got ${$Type(value)}`, []);
    const s = start != null ? $ToNumber(start) : 0;
    const e = end != null ? $ToNumber(end) : value.length;
    return sliceCore(value, s, e, true);
};
export const $AssertInit: (value: VmAny) => asserts value is VmValue = (value) => {
    if (value === undefined) throw new TypeError(`Uninitialized value`);
};
export const $Call = (func: VmValue, args: readonly VmAny[]): VmValue => {
    for (const a of args) {
        $AssertInit(a);
    }
    if (func instanceof VmExtern && func.callable) {
        return func.call(args as readonly VmValue[]) ?? null;
    }
    if (isVmFunction(func)) {
        return func(...(args as readonly VmValue[])) ?? null;
    }
    throw new VmError(`Expected callable, got ${$Type(func)}`, null);
};
export const $Type = (value: VmAny): TypeName => {
    if (value === undefined) return 'nil';
    if (value === null) return 'nil';
    if (value instanceof VmExtern) return 'extern';
    if (value instanceof VmModule) return 'module';
    if (isVmArray(value)) return 'array';
    if (typeof value == 'object') return 'record';
    return typeof value as TypeName;
};
export const $ToBoolean = (value: VmAny): boolean => {
    $AssertInit(value);
    return value != null && value !== false;
};

/** 将值转为字符串 */
function innerToString(value: VmAny, useBraces: boolean): string {
    if (value == null) return 'nil';
    if (value instanceof VmWrapper) return value.toString();
    if (typeof value == 'function') {
        const name = getVmFunctionInfo(value)?.fullName;
        return name ? `<function ${name}>` : `<function>`;
    }
    if (isVmArray(value)) {
        const results = Array.from({ length: value.length }, (_, i) => innerToString(value[i], true)).join(', ');
        if (!useBraces) return results;
        return `[${results}]`;
    }
    if (typeof value == 'object') {
        const entries = keys(value)
            .map((key) => `${key}: ${innerToString(value[key], true)}`)
            .join(', ');
        if (!useBraces) return entries;
        return `(${entries})`;
    }
    if (typeof value == 'number') {
        if (isNaN(value)) return 'nan';
        if (value === Infinity) return 'inf';
        if (value === -Infinity) return '-inf';
        return String(value);
    }
    return String(value);
}
export const $ToString = (value: VmAny): string => {
    $AssertInit(value);
    if (value === null) return '';
    return innerToString(value, false);
};
export const $ToNumber = (value: VmAny): number => {
    $AssertInit(value);
    if (value == null) return 0;
    if (typeof value == 'number') return value;
    if (typeof value == 'string') return Number(value);
    if (typeof value == 'boolean') return value ? 1 : 0;
    return Number.NaN;
};
export const $IsBoolean = (value: VmAny): value is boolean => {
    $AssertInit(value);
    return typeof value == 'boolean';
};
export const $IsNumber = (value: VmAny): value is number => {
    $AssertInit(value);
    return typeof value == 'number';
};
export const $IsString = (value: VmAny): value is string => {
    $AssertInit(value);
    return typeof value == 'string';
};
export const $IsRecord = (value: VmAny): value is VmRecord => {
    $AssertInit(value);
    return isVmRecord(value);
};
export const $IsArray = (value: VmAny): value is VmArray => {
    $AssertInit(value);
    return isVmArray(value);
};
export const $AssertNonNil = (value: VmAny): asserts value is NonNullable<VmValue> => {
    $AssertInit(value);
    if (value === null) throw new Error('Expected non-nil value');
};
export const $Has = (obj: VmAny, key: VmAny): boolean => {
    $AssertInit(obj);
    const pk = $ToString(key);
    if (obj == null || typeof obj != 'object') return false;
    if (obj instanceof VmWrapper) return obj.has(pk);
    return hasOwn(obj, pk);
};
export const $Get = (obj: VmAny, key: VmAny): VmValue => {
    $AssertInit(obj);
    if (isVmArray(obj)) {
        const index = $ToNumber(key);
        if (isNaN(index)) return null;
        return (at.call(obj, trunc(index)) as VmConst | undefined) ?? null;
    }
    const pk = $ToString(key);
    if (obj == null || typeof obj != 'object') return null;
    if (obj instanceof VmWrapper) return obj.get(pk) ?? null;
    if (!hasOwn(obj, pk)) return null;
    return (obj as Record<string, VmImmutable>)[pk] ?? null;
};
export const $Set = (obj: VmAny, key: VmAny, value: VmAny): void => {
    $AssertInit(obj);
    $AssertInit(value);
    const pk = $ToString(key);
    if (obj == null) return;
    if (!isVmExtern(obj)) throw new VmError(`Expected extern, got ${$Type(obj)}`, undefined);
    obj.set(pk, value);
};
export const $Iterable = (value: VmAny): Iterable<VmValue> => {
    $AssertInit(value);
    if (value instanceof VmWrapper) return value.keys();
    if (isVmArray(value)) return value;
    if (value != null && typeof value == 'object') return keys(value);
    if (typeof value == 'string') return value;
    throw new VmError(`Value is not iterable`, isVmFunction(value) ? [] : [value]);
};

export const $RecordSpread = (record: VmAny): VmRecord | null => {
    $AssertInit(record);
    if (record == null || isVmRecord(record)) return record;
    if (isVmExtern(record)) {
        const result: Record<string, VmConst> = create(null);
        for (const key of record.keys()) {
            const value = record.get(key) ?? null;
            // 当前只有 Primitive 不会进行二次包装
            if (isVmPrimitive(value)) {
                result[key] = value;
            }
        }
        return result;
    }
    throw new VmError(`Expected record, extern or nil, got ${$Type(record)}`, null);
};

export const $ArraySpread = (array: VmAny): Iterable<VmConst> => {
    $AssertInit(array);
    if (array == null) return [];
    if (isVmArray(array)) return array;
    if (isVmExtern(array) && 'length' in array.value && typeof array.value.length == 'number') {
        const result = Array.from(array.value as ArrayLike<VmAny>, (item) => {
            // 当前只有 Primitive 不会进行二次包装
            if (isVmPrimitive(item)) return item;
            return null;
        });
        return result;
    }
    throw new VmError(`Expected array, iterable extern or nil, got ${$Type(array)}`, []);
};
