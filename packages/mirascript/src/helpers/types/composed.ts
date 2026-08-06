import type { VmAny, VmImmutable, VmValue } from '../../vm/types/index.js';
import { isVmFunction, isVmModule, isVmWrapper } from './basic.js';
import { isVmConst } from './const.js';

/** 检查是否为 Mirascript 不可变值 */
export function isVmImmutable(value: VmAny): value is VmImmutable;
/** 检查是否为 Mirascript 不可变值 */
export function isVmImmutable(value: unknown, checkDeep: boolean): value is VmImmutable;
/** 检查是否为 Mirascript 不可变值 */
export function isVmImmutable(value: unknown, checkDeep = false): value is VmImmutable {
    return isVmModule(value) || isVmFunction(value) || isVmConst(value, checkDeep);
}

/** 检查是否为 Mirascript 合法值 */
export function isVmValue(value: VmAny): value is VmValue;
/** 检查是否为 Mirascript 合法值 */
export function isVmValue(value: unknown, checkDeep: boolean): value is VmValue;
/** 检查是否为 Mirascript 合法值 */
export function isVmValue(value: unknown, checkDeep = false): value is VmValue {
    return isVmWrapper(value) || isVmFunction(value) || isVmConst(value, checkDeep);
}

/** 检查是否为 Mirascript 值 */
export function isVmAny(value: unknown, checkDeep: boolean): value is VmAny {
    if (value == null) return true;
    return isVmValue(value, checkDeep);
}
