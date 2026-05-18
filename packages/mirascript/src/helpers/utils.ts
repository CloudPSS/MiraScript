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

/**
 * Determines whether an object has an enumerable property with the specified name.
 */
export const hasOwnEnumerable = Function.call.bind(
    // eslint-disable-next-line @typescript-eslint/unbound-method
    Object.prototype.propertyIsEnumerable,
) as (o: object, v: PropertyKey) => boolean;
