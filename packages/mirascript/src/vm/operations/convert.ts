import { toNumber, toString, toBoolean, toFormat } from '../../helpers/convert/index.js';
import type { VmAny } from '../types/index.js';
import { $AssertInit } from './common.js';

export const $ToBoolean = (value: VmAny): boolean => {
    if (typeof value == 'boolean') return value;
    $AssertInit(value);
    return toBoolean(value, undefined);
};
export const $ToString = (value: VmAny): string => {
    if (typeof value == 'string') return value;
    $AssertInit(value);
    return toString(value, undefined);
};
export const $ToNumber = (value: VmAny): number => {
    if (typeof value == 'number') return value;
    $AssertInit(value);
    return toNumber(value, undefined);
};
export const $Format = (value: VmAny, format: VmAny): string => {
    $AssertInit(value);
    const f = format == null ? '' : $ToString(format);
    return toFormat(value, f);
};
