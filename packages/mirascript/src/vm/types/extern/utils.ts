import { getOwnPropertyNames, getPrototypeOf, isArray } from '../../../helpers/utils.js';

/**
 * Should this extern be treated as array-like?
 *
 * By default, this method returns true if the wrapped value is an Array or a TypedArray.
 */
export function isArrayLike(value: object): value is ArrayLike<unknown> {
    return isArray(value) || (ArrayBuffer.isView(value) && 'length' in value);
}

/**
 * Get extern object keys, including inherited ones.
 */
export function getKeys(value: object, includeNonEnumerable: boolean): string[] {
    if (!includeNonEnumerable) {
        const keys: string[] = [];
        for (const key in value) {
            keys.push(key);
        }
        return keys;
    }
    const keys = new Set<string>();
    let e: unknown = value;
    while (e != null && (typeof e == 'object' || typeof e == 'function')) {
        for (const key of getOwnPropertyNames(e)) {
            keys.add(key);
        }
        e = getPrototypeOf(e);
    }
    return Array.from(keys);
}

// eslint-disable-next-line @typescript-eslint/unbound-method
const ObjectToString = Object.prototype.toString;
// eslint-disable-next-line @typescript-eslint/unbound-method
const FunctionToString = Function.prototype.toString;
const ArrayToString = Array.prototype.toString;
// eslint-disable-next-line @typescript-eslint/unbound-method
const TypedArrayToString = Uint8Array.prototype.toString;

/** Check toString method of the extern object */
export function hasCustomToString(value: object): boolean {
    // eslint-disable-next-line @typescript-eslint/unbound-method
    const { toString } = value;
    if (typeof toString != 'function') return false;
    if (
        toString === ObjectToString ||
        toString === FunctionToString ||
        toString === ArrayToString ||
        toString === TypedArrayToString
    )
        return false;
    return true;
}
