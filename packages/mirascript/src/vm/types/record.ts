import { keys } from '../../helpers/utils.js';
import { isVmArray, VM_ARRAY_MAX_LENGTH } from './array.js';
import { isVmWrapper, type VmAny, type VmConst } from './index.js';

/**
 * Mirascript 记录
 * 仅拥有且可枚举的字符串键视作存在
 * 字段值 `undefined` 和 `null` 均视作 `nil`
 */
// eslint-disable-next-line @typescript-eslint/consistent-indexed-object-style
export type VmRecord = {
    readonly [key: string]: VmConst | undefined;
};

/**
 * 检查值是否为 Mirascript 记录
 */
export function isVmRecord(value: VmAny): value is VmRecord {
    if (value == null || typeof value !== 'object') return false;
    if (isVmWrapper(value)) return false;
    if (isVmArray(value)) return false;
    value satisfies VmRecord;
    return true;
}

/** 检查是否为仅包含从 0 开始的连续数字键的 MiraScript 记录 */
export function isVmArrayLikeRecordByEntires(entries: ReadonlyArray<readonly [string, unknown]>): boolean {
    const { length } = entries;
    if (length === 0) return true;
    if (length > VM_ARRAY_MAX_LENGTH) return false;
    const firstKey = entries[0]![0];
    if (firstKey !== '0') return false;
    const lastKey = entries[length - 1]![0];
    if (lastKey !== String(length - 1)) return false;
    return true;
}

/** 检查是否为仅包含从 0 开始的连续数字键的 MiraScript 记录 */
export function isVmArrayLikeRecordByKeys(keys: readonly string[]): boolean {
    const { length } = keys;
    if (length === 0) return true;
    if (length > VM_ARRAY_MAX_LENGTH) return false;
    const firstKey = keys[0]!;
    if (firstKey !== '0') return false;
    const lastKey = keys[length - 1]!;
    if (lastKey !== String(length - 1)) return false;
    return true;
}

/** 检查是否为仅包含从 0 开始的连续数字键的 MiraScript 记录 */
export function isVmArrayLikeRecord(value: VmRecord): boolean {
    return isVmArrayLikeRecordByKeys(keys(value));
}
