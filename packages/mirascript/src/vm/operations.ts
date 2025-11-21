import { VmError } from '../helpers/error.js';
import { hasOwnEnumerable, isNaN, isSafeInteger, keys, create } from '../helpers/utils.js';
import { toNumber, toString, toBoolean, toFormat } from '../helpers/convert/index.js';
import { display } from '../helpers/serialize.js';
import { isVmPrimitive, isVmArray, isVmRecord, isVmFunction, isVmExtern, isVmWrapper } from '../helpers/types.js';
import type { TypeName, VmAny, VmImmutable, VmRecord, VmValue, VmArray, VmConst } from './types/index.js';

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
    if (isVmWrapper(a)) return a.same(b);
    if (isVmWrapper(b)) return b.same(a);
    // Handle array values
    if (isVmArray(a) && isVmArray(b)) {
        const len = a.length;
        if (len !== b.length) {
            return false;
        }
        // Compare array items
        for (let i = 0; i < len; i++) {
            if (!isSame(a[i] ?? null, b[i] ?? null)) {
                return false;
            }
        }
        return true;
    }
    // Handle record values
    if (!isVmArray(a) && !isVmArray(b)) {
        // Compare record fields
        const aKeys = keys(a);
        const bKeys = keys(b);
        if (aKeys.length !== bKeys.length) {
            return false;
        }
        for (const key of aKeys) {
            if (!hasOwnEnumerable(b, key)) {
                return false;
            }
            /* c8 ignore next 2 */
            const av = a[key] ?? null;
            const bv = b[key] ?? null;
            if (!isSame(av, bv)) {
                return false;
            }
        }
        return true;
    }
    return false;
};

const overloadNumberString = (a: VmAny, b: VmAny): boolean => {
    if (typeof a == 'number' || typeof b == 'number') return true;
    if (typeof a == 'string' || typeof b == 'string') return false;
    return true;
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
export const $In = (value: VmAny, iterable: VmAny): boolean => {
    $AssertInit(value);
    $AssertInit(iterable);
    if (iterable == null) return false;
    if (typeof iterable != 'object') return false;
    if (isVmArray(iterable)) {
        if (value == null) {
            // array may have empty slots
            for (const item of iterable) if (item == null) return true;
            return false;
        }
        // JS %SameValueZero is same with `isSame` in this context
        if (isVmPrimitive(value)) return iterable.includes(value);
        // value is not null here, so it's ok to skip empty slots, since `isSame(null, something)` is always false
        return iterable.some((item = null) => isSame(item, value satisfies NonNullable<VmValue>));
    }
    // iterable is a record or an extern here, value should be a string
    const key = toString(value, undefined);
    if (isVmWrapper(iterable)) return iterable.has(key);
    return hasOwnEnumerable(iterable satisfies VmRecord, key);
};
export const $Concat = (...args: readonly string[]): string => {
    return args.map((a) => toFormat(a, null)).join('');
};
export const $Pos = (a: VmAny): number => {
    return $ToNumber(a);
};
export const $Neg = (a: VmAny): number => {
    return -$ToNumber(a);
};
export const $Not = (a: VmAny): boolean => {
    return !$ToBoolean(a);
};
export const $Length = (value: VmAny): number => {
    $AssertInit(value);
    if (isVmArray(value)) return value.length;
    if (isVmRecord(value)) return keys(value).length;
    if (isVmWrapper(value)) {
        return value.keys().length;
    }
    return Number.NaN;
};
export const $Omit = (value: VmAny, omitted: ReadonlyArray<number | string>): VmRecord => {
    $AssertInit(value);
    if (value == null || !isVmRecord(value)) return {};
    const result: Record<string, VmConst> = {};
    const valueKeys = keys(value);
    const omittedSet = new Set(omitted.map($ToString));
    for (const key of valueKeys) {
        if (!omittedSet.has(key)) {
            /* c8 ignore next */
            result[key] = value[key] ?? null;
        }
    }
    return result;
};
export const $Pick = (value: VmAny, picked: ReadonlyArray<number | string>): VmRecord => {
    $AssertInit(value);
    if (value == null || !isVmRecord(value)) return {};
    const result: Record<string, VmConst> = {};
    for (const key of picked) {
        const k = $ToString(key);
        if (hasOwnEnumerable(value, k)) {
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
    if (value === undefined) throw new VmError(`Uninitialized value`, null);
};
export const $Call = (func: VmValue, args: readonly VmAny[]): VmValue => {
    for (const a of args) {
        $AssertInit(a);
    }
    if (isVmExtern(func)) {
        return func.call(args as readonly VmValue[]) ?? null;
    }
    if (isVmFunction(func)) {
        return func(...(args as readonly VmValue[])) ?? null;
    }
    throw new VmError(`Value is not callable: ${display(func)}`, null);
};
export const $Type = (value: VmAny): TypeName => {
    // 允许未初始化值通过
    if (value == null) return 'nil';
    if (isVmWrapper(value)) return value.type;
    if (isVmArray(value)) return 'array';
    if (typeof value == 'object') return 'record';
    return typeof value as TypeName;
};
export const $ToBoolean = (value: VmAny): boolean => {
    if (typeof value == 'boolean') return value;
    $AssertInit(value);
    return toBoolean(value, undefined);
};
export const $ToString = (value: VmAny): string => {
    if (typeof value == 'string') return value;
    $AssertInit(value);
    return toString(value, undefined);
};
export const $ToNumber = (value: VmAny): number => {
    if (typeof value == 'number') return value;
    $AssertInit(value);
    return toNumber(value, undefined);
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
    if (value !== null) return;
    throw new VmError('Expected non-nil value', null);
};
export const $Has = (obj: VmAny, key: VmAny): boolean => {
    $AssertInit(obj);
    const pk = $ToString(key);
    if (obj == null || typeof obj != 'object') return false;
    if (isVmWrapper(obj)) return obj.has(pk);
    return hasOwnEnumerable(obj, pk);
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
    if (isVmWrapper(obj)) return obj.get(pk) ?? null;
    if (!hasOwnEnumerable(obj, pk)) return null;
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
export const $Iterable = (value: VmAny): Iterable<VmValue | undefined> => {
    $AssertInit(value);
    if (isVmWrapper(value)) return value.keys();
    if (isVmArray(value)) return value;
    if (value != null && typeof value == 'object') return keys(value);
    throw new VmError(`Value is not iterable`, isVmFunction(value) ? [] : [value]);
};

export const $RecordSpread = (record: VmAny): VmRecord | null => {
    $AssertInit(record);
    if (record == null || isVmRecord(record)) return record;
    if (isVmArray(record)) {
        const result: Record<string, VmConst> = {};
        const len = record.length;
        for (let i = 0; i < len; i++) {
            const item = record[i];
            result[i] = item ?? null;
        }
        return result;
    }
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
    throw new VmError(`Expected record, array, extern or nil, got ${$Type(record)}`, null);
};

export const $ArraySpread = (array: VmAny): Iterable<VmConst | undefined> => {
    $AssertInit(array);
    if (array == null) return [];
    if (isVmArray(array)) return array;
    if (isVmExtern(array) && typeof (array.value as Iterable<unknown>)[Symbol.iterator] == 'function') {
        const result: VmConst[] = [];
        for (const item of array.value as Iterable<unknown>) {
            // 当前只有 Primitive 不会进行二次包装
            if (isVmPrimitive(item)) {
                result.push(item);
            } else {
                result.push(null);
            }
        }
        return result;
    }
    throw new VmError(`Expected array, iterable extern or nil, got ${$Type(array)}`, []);
};

export const $Format = (value: VmAny, format: VmAny): string => {
    $AssertInit(value);
    const f = format == null ? '' : $ToString(format);
    return toFormat(value, f);
};
