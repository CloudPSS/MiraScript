import { isArray } from '../../helpers/utils.js';
import type { VmAny, VmConst } from './index.js';

/**
 * Mirascript 数组
 * 数组中的 `undefined`、`null` 及 <empty slot> 均视作 `nil`
 */
export type VmArray = ReadonlyArray<VmConst | undefined>;

export const VM_ARRAY_MAX_LENGTH = 2 ** 31 - 1;

/**
 * 检查值是否为 Mirascript 数组
 */
export function isVmArray(value: VmAny): value is VmArray {
    if (!isArray(value)) return false;
    value as VmArray satisfies VmArray;
    return true;
}
