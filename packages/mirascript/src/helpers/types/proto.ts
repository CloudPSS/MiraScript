import { getPrototypeOf, hasOwn, isArray } from '../utils.js';

/** 检查是否为普通数组 */
export function isPlainArray(value: readonly unknown[]): boolean {
    // value 的原型应当为 Array.prototype
    const proto: unknown = getPrototypeOf(value);
    // 使用 isArray 检查原型是否为 Array.prototype，避免直接比较原型对象，防止跨环境问题
    return isArray(proto) && hasOwn(proto, 'push');
}

/** 检查是否为普通对象或空原型对象 */
export function isPlainObject(value: object): boolean {
    // value 的原型应当为 Object.prototype 或 null
    const proto1: unknown = getPrototypeOf(value);
    if (proto1 == null) return true;
    // Object.prototype 的原型应当为 null
    const proto2: unknown = getPrototypeOf(proto1);
    if (proto2 != null) return false;
    // 检查是否具有 Object.prototype 的方法，避免直接比较原型对象，防止跨环境问题
    return hasOwn(proto1, 'hasOwnProperty');
}
