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
} from './types/index.js';
import { VmWrapper } from './types/wrapper.js';

const { hasOwn, keys } = Object;
const { isNaN } = Number;
const { abs, min } = Math;
const isSame = (a: VmValue, b: VmValue): boolean => {
    // Check for NaN
    if (typeof a == 'number' && typeof b == 'number') return a === b || (isNaN(a) && isNaN(b));
    // Check all primitive types, and fast path for reference equality
    if (a === b) return true;
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
export const $Not = (a: VmAny): boolean => !$ToBool(a);
export const $Init: (value: VmAny) => asserts value is VmValue = (value) => {
    if (value === undefined) throw new TypeError(`Uninitialized value`);
};
export const $CallDyn = (func: VmValue, args: readonly VmAny[]): VmValue => {
    for (const a of args) {
        $Init(a);
    }
    if (func instanceof VmExtern && func.callable()) {
        return func.call(args as readonly VmValue[]) ?? null;
    }
    if (isVmFunction(func)) {
        return func(...(args as readonly VmValue[])) ?? null;
    }
    throw new TypeError(`Expected callable, got ${$Type(func)}`);
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
    if (value instanceof VmWrapper) return value.toString();
    if (typeof value == 'function') {
        const name = getVmFunctionInfo(value)?.fullName;
        return name ? `<function ${name}>` : `<function>`;
    }
    if (isVmArray(value)) return `[${value.map(innerToString).join(', ')}]`;
    if (typeof value == 'object') {
        const entries = keys(value).map(
            (key) => `${key}: ${innerToString((value as Record<string, VmImmutable>)[key])}`,
        );
        return `(${entries.join(', ')})`;
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
    $Init(value);
    if (value === null) return '';
    if (isVmArray(value)) return value.map(innerToString).join(', ');
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
export const $Iterable = (value: VmAny): Iterable<VmValue> => {
    $Init(value);
    if (value instanceof VmWrapper) return value.keys();
    if (isVmArray(value)) return value;
    if (value != null && typeof value == 'object') return keys(value);
    if (typeof value == 'string') return value;
    throw new VmError(`Value is not iterable`, isVmFunction(value) ? [] : [value]);
};

export const $RecordSpread = (record: VmAny): VmRecord | null => {
    $Init(record);
    if (record == null || isVmRecord(record)) return record;
    throw new VmError(`Expected record or nil, got ${$Type(record)}`, null);
};

export const $ArraySpread = (array: VmAny): Iterable<VmValue> => {
    $Init(array);
    if (!isVmArray(array)) throw new VmError(`Expected array, got ${$Type(array)}`, []);
    return array;
};
