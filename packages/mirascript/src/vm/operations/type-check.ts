import { VmError } from '../../helpers/error.js';
import { isVmArray, isVmRecord, isVmWrapper } from '../../helpers/types.js';
import type { TypeName, VmAny, VmArray, VmRecord, VmValue } from '../types/index.js';
import { $AssertInit } from './common.js';

export const $Type = (value: VmAny): TypeName => {
    // 允许未初始化值通过
    if (value == null) return 'nil';
    if (isVmWrapper(value)) return value.type;
    if (isVmArray(value)) return 'array';
    if (typeof value == 'object') return 'record';
    return typeof value as TypeName;
};
export const $IsBoolean = (value: VmAny): value is boolean => {
    $AssertInit(value);
    return typeof value == 'boolean';
};
export const $IsNumber = (value: VmAny): value is number => {
    $AssertInit(value);
    return typeof value == 'number';
};
export const $IsString = (value: VmAny): value is string => {
    $AssertInit(value);
    return typeof value == 'string';
};
export const $IsRecord = (value: VmAny): value is VmRecord => {
    $AssertInit(value);
    return isVmRecord(value);
};
export const $IsArray = (value: VmAny): value is VmArray => {
    $AssertInit(value);
    return isVmArray(value);
};
export const $AssertNonNil = (value: VmAny): asserts value is NonNullable<VmValue> => {
    $AssertInit(value);
    if (value !== null) return;
    throw new VmError(`Expected non-nil value`, null);
};
