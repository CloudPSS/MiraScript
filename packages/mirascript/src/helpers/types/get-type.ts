import type { VmValue, VmTypeName, VmAny } from '../../vm/index.js';
import { isVmWrapper, isVmArray } from './basic.js';

/** 获取 MiraScript 类型 */
export function getVmType(value: VmValue): VmTypeName;
/** 获取 MiraScript 类型 */
export function getVmType(value: VmAny): VmTypeName | 'uninitialized';
/** 获取 MiraScript 类型 */
export function getVmType(value: VmAny): VmTypeName | 'uninitialized' {
    if (value === undefined) return 'uninitialized';
    if (value === null) return 'nil';
    if (isVmWrapper(value)) return value.type;
    if (isVmArray(value)) return 'array';
    if (typeof value == 'object') return 'record';
    return typeof value as VmTypeName;
}
