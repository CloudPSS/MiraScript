import { VmError } from '../../helpers/error.js';
import { getVmType, isVmArray, isVmRecord } from '../../helpers/types.js';
import type { VmTypeName, VmAny, VmArray, VmRecord, VmValue } from '../types/index.js';
import { $AssertInit } from './common.js';

/** 获取值的类型名称 */
export function $Type(value: VmAny): VmTypeName {
    // 允许未初始化值通过
    if (value == null) return 'nil';
    return getVmType(value);
}
/** 判断值是否为布尔值 */
export function $IsBoolean(value: VmAny): value is boolean {
    $AssertInit(value);
    return typeof value == 'boolean';
}
/** 判断值是否为数字 */
export function $IsNumber(value: VmAny): value is number {
    $AssertInit(value);
    return typeof value == 'number';
}
/** 判断值是否为字符串 */
export function $IsString(value: VmAny): value is string {
    $AssertInit(value);
    return typeof value == 'string';
}
/** 判断值是否为记录 */
export function $IsRecord(value: VmAny): value is VmRecord {
    $AssertInit(value);
    return isVmRecord(value);
}
/** 判断值是否为数组 */
export function $IsArray(value: VmAny): value is VmArray {
    $AssertInit(value);
    return isVmArray(value);
}
/** 断言值非 nil */
export function $AssertNonNil<T extends VmValue>(value: T | undefined): asserts value is NonNullable<T> {
    $AssertInit(value);
    if (value !== null) return;
    throw new VmError(`Expected non-nil value`, null);
}
