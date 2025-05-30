import {
    isVmFunction,
    VmModule,
    type VmAny,
    type VmArray,
    type VmConst,
    type VmImmutable,
    type VmRecord,
    type VmValue,
} from './index.js';
import { VmWrapper } from './wrapper.js';
const { isArray } = Array;
const { getPrototypeOf, values } = Object;

const MAX_DEPTH = 100;
/**
 * 检查是否为 Mirascript 数组
 */
function isVmArray(value: readonly unknown[], depth: number): value is VmArray {
    // VmArray 应为普通数组
    // Array.prototype
    const proto1: unknown = getPrototypeOf(value);
    if (!isArray(proto1)) return false;
    // Object.prototype
    const proto2: unknown = getPrototypeOf(proto1);
    if (proto2 == null || isArray(proto2)) return false;
    // null
    const proto3: unknown = getPrototypeOf(proto2);
    if (proto3 != null) return false;
    if (!depth) return true;
    return value.every((item) => isVmConstInner(item, depth));
}
/**
 * 检查是否为 Mirascript 记录
 */
function isVmRecord(value: object, depth: number): value is VmRecord {
    // VmRecord 应为普通对象或空原型对象
    let isRecord;
    // Object.prototype
    const proto1: unknown = getPrototypeOf(value);
    if (proto1 == null) {
        isRecord = true;
    } else {
        // null
        const proto2: unknown = getPrototypeOf(proto1);
        if (proto2 != null) {
            isRecord = false;
        } else {
            isRecord = 'hasOwnProperty' in value;
        }
    }
    if (!isRecord) return false;
    if (!depth) return true;
    return values(value).every((value) => isVmConstInner(value, depth));
}
/**
 * 检查是否为 Mirascript 值语义值
 */
function isVmConstInner(value: unknown, depth: number): value is VmConst {
    if (depth++ >= MAX_DEPTH) return false;
    switch (typeof value) {
        case 'string':
        case 'number':
        case 'boolean':
        case 'undefined': // undefined 在复合类型内部被视为 nil
            return true;
        case 'object':
            if (value == null) return true;
            if (value instanceof VmWrapper) return false;
            if (isArray(value)) {
                return isVmArray(value, depth);
            } else {
                return isVmRecord(value, depth);
            }
        case 'function':
        case 'bigint':
        case 'symbol':
        default:
            return false; // Other types are not valid
    }
}
/**
 * 检查是否为 Mirascript 值语义值
 * @param value - 要检查的值
 * @param checkDeep - 是否深度检查数组和对象
 */
export function isVmConst(value: unknown, checkDeep = false): value is VmConst {
    switch (typeof value) {
        case 'string':
        case 'number':
        case 'boolean':
            return true;
        case 'object':
            if (value == null) return true;
            if (value instanceof VmWrapper) return false;
            if (!checkDeep) {
                if (isArray(value)) {
                    return isVmArray(value, 0);
                } else {
                    return isVmRecord(value, 0);
                }
            } else {
                return isVmConstInner(value, 1);
            }
        case 'undefined':
        case 'function':
        case 'bigint':
        case 'symbol':
        default:
            return false; // Other types are not valid
    }
}
/**
 * 检查是否为 Mirascript 不可变值
 * @param value - 要检查的值
 * @param checkDeep - 是否深度检查数组和对象
 */
export function isVmImmutable(value: unknown, checkDeep = false): value is VmImmutable {
    return value instanceof VmModule || isVmFunction(value) || isVmConst(value, checkDeep);
}
/**
 * 检查是否为 Mirascript 合法值
 * @param value - 要检查的值
 * @param checkDeep - 是否深度检查数组和对象
 */
export function isVmValue(value: unknown, checkDeep = false): value is VmValue {
    if (value === undefined) return false;
    return isVmAny(value, checkDeep);
}

/**
 * 检查是否为 Mirascript 值
 * @param value - 要检查的值
 * @param checkDeep - 是否深度检查数组和对象
 */
export function isVmAny(value: unknown, checkDeep = false): value is VmAny {
    switch (typeof value) {
        case 'string':
        case 'number':
        case 'boolean':
        case 'undefined':
            return true;
        case 'object':
            if (value == null) return true;
            if (value instanceof VmWrapper) return true;
            return isVmConst(value, checkDeep);
        case 'function':
            return isVmFunction(value);
        case 'bigint':
        case 'symbol':
        default:
            return false; // Other types are not valid
    }
}
