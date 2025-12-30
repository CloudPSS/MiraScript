import { VmError } from '../../helpers/error.js';
import type { VmAny, VmValue } from '../types/index.js';

/** 断言值已初始化 */
export function $AssertInit(value: VmAny): asserts value is VmValue {
    if (value === undefined) throw new VmError(`Uninitialized value`, null);
}
