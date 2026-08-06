import type { VmAny, VmArray, VmConst, VmRecord } from '../../vm/types/index.js';
import { getPrototypeOf, isArray, values } from '../utils.js';
import { isVmWrapper } from './basic.js';

const MAX_DEPTH = 16;
/**
 * 检查是否为 Mirascript 数组
 */
function isVmArrayDeep(value: readonly unknown[], depth: number): value is VmArray {
    // VmArray 应为普通数组
    // Array.prototype
    const proto1: unknown = getPrototypeOf(value);
    if (!isArray(proto1)) return false;
    if (depth <= 0) return true;
    const inner_depth = depth - 1;
    return value.every((item) => isVmConstInner(item, inner_depth));
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
    if (depth <= 0) return true;
    const inner_depth = depth - 1;
    return values(value).every((value) => isVmConstInner(value, inner_depth));
}
/**
 * 检查是否为 Mirascript 值语义值
 */
function isVmConstInner(value: unknown, depth: number): value is VmConst {
    switch (typeof value) {
        case 'object':
            if (value == null) return true;
            if (isVmWrapper(value)) return false;
            if (isArray(value)) {
                return isVmArrayDeep(value, depth);
            } else {
                return isVmRecordDeep(value, depth);
            }
        case 'string':
        case 'number':
        case 'boolean':
        case 'undefined': // undefined 在复合类型内部被视为 nil
            return true;
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
    if (value === undefined) return false;
    return isVmConstInner(value, checkDeep ? MAX_DEPTH : 0);
}
