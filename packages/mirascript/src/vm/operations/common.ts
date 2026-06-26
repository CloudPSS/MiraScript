import { VmError } from '../../helpers/error.js';
import type { VmValue } from '../types/index.js';

/** 断言值已初始化 */
export const $AssertInit: <T extends VmValue>(value: T | undefined) => asserts value is T = (value) => {
    if (value === undefined) throw new VmError(`Uninitialized value`, null);
};
