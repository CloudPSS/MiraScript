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
