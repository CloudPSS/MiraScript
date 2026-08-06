import type { VmRecord } from '../../vm/types/index.js';
import { VM_ARRAY_MAX_LENGTH } from '../constants.js';
import { keys } from '../utils.js';

/** 检查是否为仅包含从 0 开始的连续数字键的 MiraScript 记录 */
export function isVmArrayLikeRecordByEntries(entries: ReadonlyArray<readonly [string, unknown]>): boolean {
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
