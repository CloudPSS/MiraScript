import { toNumber, toString, toBoolean, toFormat } from '../../helpers/convert/index.js';
import type { VmAny } from '../types/index.js';
import { $AssertInit } from './common.js';

/** 转换为布尔值 */
export const $ToBoolean = (value: VmAny): boolean => {
    if (typeof value == 'boolean') return value;
    $AssertInit(value);
    return toBoolean(value, undefined);
};
/** 转换为字符串 */
export const $ToString = (value: VmAny): string => {
    if (typeof value == 'string') return value;
    $AssertInit(value);
    return toString(value, undefined);
};
/** 转换为数字 */
export const $ToNumber = (value: VmAny): number => {
    if (typeof value == 'number') return value;
    $AssertInit(value);
    return toNumber(value, undefined);
};
/** 格式化值 */
export const $Format = (value: VmAny, format: string | null): string => {
    $AssertInit(value);
    return toFormat(value, format);
};
