import type { VmConst, VmRecord } from '../vm/index.js';

export const { isArray } = Array;
export const {
    isFinite,
    isNaN,
    isInteger,
    isSafeInteger,
    NaN: NotNumber,
    POSITIVE_INFINITY: PositiveInfinity,
    NEGATIVE_INFINITY: NegativeInfinity,
} = Number;
export const { hasOwn, keys, values, entries, create, fromEntries, defineProperty, getOwnPropertyNames, freeze } =
    Object;
export const { apply, getPrototypeOf } = Reflect;

// Polyfill for https://github.com/tc39/proposal-object-keys-length
/** 获取对象的键数量 */
export const keysLength =
    'keysLength' in Object && typeof Object.keysLength == 'function'
        ? (Object.keysLength as (o: object) => number)
        : (o: object): number => keys(o).length;

/**
 * Determines whether an object has an enumerable property with the specified name.
 */
export const hasOwnEnumerable = Function.call.bind(
    // eslint-disable-next-line @typescript-eslint/unbound-method
    Object.prototype.propertyIsEnumerable,
) as (o: object, v: PropertyKey) => boolean;

const SPECIAL_KEYS = new Set(getOwnPropertyNames(Object.prototype));
/**
 * Set property on an vm record.
 */
export const setRecord = (obj: Record<string, VmConst | undefined>, key: string, value: VmConst | undefined): void => {
    if (!SPECIAL_KEYS.has(key)) {
        obj[key] = value ?? null;
    } else {
        Object.defineProperty(obj, key, {
            value: value ?? null,
            configurable: true,
            writable: true,
            enumerable: true,
        });
    }
};

/**
 * Get property from an vm record.
 */
export const getRecord = (obj: VmRecord, key: string): VmConst | undefined => {
    if (!hasOwnEnumerable(obj, key)) return undefined;
    return obj[key] ?? null;
};
