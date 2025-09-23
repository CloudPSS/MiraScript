export const { isArray } = Array;
export const { isFinite, isNaN, isInteger, isSafeInteger } = Number;
export const { hasOwn, keys, values, entries, create, getPrototypeOf, fromEntries, defineProperty } = Object;

/**
 * Determines whether an object has an enumerable property with the specified name.
 */
export const hasOwnEnumerable = Function.call.bind<
    object['propertyIsEnumerable'],
    [],
    [o: object, v: PropertyKey],
    boolean
>(
    // eslint-disable-next-line @typescript-eslint/unbound-method
    Object.prototype.propertyIsEnumerable,
);
