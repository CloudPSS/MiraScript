import { getPrototypeOf, isArray, values } from '../../helpers/utils.js';
import type { VmAny, VmArray, VmConst, VmImmutable, VmRecord, VmValue } from './index.js';
import { isVmWrapper } from './wrapper.js';
import { isVmModule } from './module.js';
import { isVmFunction } from './function.js';

const MAX_DEPTH = 32;
/**
 * 检查是否为 Mirascript 数组
 */
function isVmArrayDeep(value: readonly unknown[], depth: number): value is VmArray {
    // VmArray 应为普通数组
    // Array.prototype
    const proto1: unknown = getPrototypeOf(value);
    if (!isArray(proto1)) return false;
    if (!depth) return true;
    return value.every((item) => isVmConstInner(item, depth));
}
/**
 * 检查是否为 Mirascript 记录
 */
function isVmRecordDeep(value: object, depth: number): value is VmRecord {
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
            if (isVmWrapper(value)) return false;
            if (isArray(value)) {
                return isVmArrayDeep(value, depth);
            } else {
                return isVmRecordDeep(value, depth);
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
 */
export function isVmConst(value: VmAny): value is VmConst;
/**
 * 检查是否为 Mirascript 值语义值
 */
export function isVmConst(value: unknown, checkDeep: boolean): value is VmConst;
/**
 * 检查是否为 Mirascript 值语义值
 */
export function isVmConst(value: unknown, checkDeep = false): value is VmConst {
    switch (typeof value) {
        case 'string':
        case 'number':
        case 'boolean':
            return true;
        case 'object':
            if (value == null) return true;
            if (isVmWrapper(value)) return false;
            if (!checkDeep) {
                if (isArray(value)) {
                    return isVmArrayDeep(value, 0);
                } else {
                    return isVmRecordDeep(value, 0);
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
 */
export function isVmImmutable(value: VmAny): value is VmImmutable;
/**
 * 检查是否为 Mirascript 不可变值
 */
export function isVmImmutable(value: unknown, checkDeep: boolean): value is VmImmutable;
/**
 * 检查是否为 Mirascript 不可变值
 */
export function isVmImmutable(value: unknown, checkDeep = false): value is VmImmutable {
    return isVmModule(value) || isVmFunction(value) || isVmConst(value, checkDeep);
}
/**
 * 检查是否为 Mirascript 合法值
 */
export function isVmValue(value: VmAny): value is VmValue;
/**
 * 检查是否为 Mirascript 合法值
 */
export function isVmValue(value: unknown, checkDeep: boolean): value is VmValue;
/**
 * 检查是否为 Mirascript 合法值
 */
export function isVmValue(value: unknown, checkDeep = false): value is VmValue {
    if (value === undefined) return false;
    return isVmAny(value, checkDeep);
}

/**
 * 检查是否为 Mirascript 值
 */
export function isVmAny(value: unknown, checkDeep: boolean): value is VmAny {
    switch (typeof value) {
        case 'string':
        case 'number':
        case 'boolean':
        case 'undefined':
            return true;
        case 'object':
            if (value == null) return true;
            if (isVmWrapper(value)) return true;
            return isVmConst(value, checkDeep);
        case 'function':
            return isVmFunction(value);
        case 'bigint':
        case 'symbol':
        default:
            return false; // Other types are not valid
    }
}
