import type { VmAny, VmArray, VmConst, VmRecord } from '../../vm/types/index.js';
import { isArray, values } from '../utils.js';
import { isVmWrapper } from './basic.js';
import { isPlainArray, isPlainObject } from './proto.js';

const MAX_DEPTH = 16;
/** 检查是否为 Mirascript 数组 */
function isVmArrayDeep(value: readonly unknown[], depth: number): value is VmArray {
    if (!isPlainArray(value)) return false;
    if (depth <= 0) return true;
    const inner_depth = depth - 1;
    return value.every((item) => isVmConstInner(item, inner_depth));
}
/** 检查是否为 Mirascript 记录 */
function isVmRecordDeep(value: object, depth: number): value is VmRecord {
    if (!isPlainObject(value)) return false;
    if (depth <= 0) return true;
    const inner_depth = depth - 1;
    return values(value).every((value) => isVmConstInner(value, inner_depth));
}
/** 检查是否为 Mirascript 值语义值 */
function isVmConstInner(value: unknown, depth: number): value is VmConst {
    if (value == null) return true; // undefined 在复合类型内部被视为 nil
    if (typeof value == 'string' || typeof value == 'number' || typeof value == 'boolean') return true;
    if (typeof value == 'object') {
        if (isVmWrapper(value)) return false;
        if (isArray(value)) {
            return isVmArrayDeep(value, depth);
        } else {
            return isVmRecordDeep(value, depth);
        }
    }
    return false; // Other types are not valid
}

/** 检查是否为 Mirascript 值语义值 */
export function isVmConst(value: VmAny): value is VmConst;
/** 检查是否为 Mirascript 值语义值 */
export function isVmConst(value: unknown, checkDeep: boolean): value is VmConst;
/** 检查是否为 Mirascript 值语义值 */
export function isVmConst(value: unknown, checkDeep = false): value is VmConst {
    if (value === undefined) return false;
    return isVmConstInner(value, checkDeep ? MAX_DEPTH : 0);
}
