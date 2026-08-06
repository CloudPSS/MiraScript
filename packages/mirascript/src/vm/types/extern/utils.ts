import { getOwnPropertyNames, getPrototypeOf, hasOwn, isArray } from '../../../helpers/utils.js';

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

const pFunction = Function.prototype;
const pArray = Array.prototype;
const pObject = Object.prototype;
/**
 * Can property be accessed on the extern object?
 */
export function canAccessProperty(value: object, key: string, read: boolean): boolean {
    // __proto__ and other “private” properties are not accessible
    if (key.startsWith('_')) return false;
    // Function-specific properties are not accessible
    if (typeof value == 'function' && (key === 'prototype' || key === 'arguments' || key === 'caller')) return false;
    if (hasOwn(value, key)) return true;
    if (!read) return true;
    if (!(key in value)) return false;
    if (key === 'constructor') return false; // constructor is not accessible
    // property is not readable if it is the same as the prototype's property
    const prop = (value as Record<string, unknown>)[key];
    if (key in pFunction && prop === pFunction[key as keyof (() => void)]) return false;
    if (key in pArray && prop === pArray[key as keyof unknown[]]) return false;
    if (key in pObject && prop === pObject[key as keyof object]) return false;
    return true;
}
