import { VmError } from '../../helpers/error.js';
import { getVmType, isVmArray, isVmRecord } from '../../helpers/types.js';
import type { VmTypeName, VmAny, VmArray, VmRecord, VmValue } from '../types/index.js';
import { $AssertInit } from './common.js';

/** 获取值的类型名称 */
export const $Type = (value: VmAny): VmTypeName => {
    // 允许未初始化值通过
    if (value == null) return 'nil';
    return getVmType(value);
};
/** 判断值是否为布尔值 */
export const $IsBoolean: (value: VmAny) => value is boolean = (value) => {
    $AssertInit(value);
    return typeof value == 'boolean';
};
/** 判断值是否为数字 */
export const $IsNumber: (value: VmAny) => value is number = (value) => {
    $AssertInit(value);
    return typeof value == 'number';
};
/** 判断值是否为字符串 */
export const $IsString: (value: VmAny) => value is string = (value) => {
    $AssertInit(value);
    return typeof value == 'string';
};
/** 判断值是否为记录 */
export const $IsRecord: (value: VmAny) => value is VmRecord = (value) => {
    $AssertInit(value);
    return isVmRecord(value);
};
/** 判断值是否为数组 */
export const $IsArray: (value: VmAny) => value is VmArray = (value) => {
    $AssertInit(value);
    return isVmArray(value);
};
/** 断言值非 nil */
export const $AssertNonNil: <T extends VmValue>(value: T | undefined) => asserts value is NonNullable<T> = (value) => {
    $AssertInit(value);
    if (value !== null) return;
    throw new VmError(`Expected non-nil value`, null);
};
