import type { VmAny } from './any.js';
import { isVmConst, type VmConst } from './const.js';
import { isVmFunction, type VmFunction } from './function.js';
import { isVmModule, type VmModule } from './module.js';

/** Mirascript 虚拟机内的不可变值 */
export type VmImmutable = VmConst | VmFunction | VmModule;

/**
 * 检查是否为 Mirascript 不可变值
 */
export function isVmImmutable(value: VmAny): value is VmImmutable;
/**
 * 检查是否为 Mirascript 不可变值
 */
export function isVmImmutable(value: unknown, checkDeep: boolean): value is VmImmutable;
/**
 * 检查是否为 Mirascript 不可变值
 */
export function isVmImmutable(value: unknown, checkDeep = false): value is VmImmutable {
    return isVmModule(value) || isVmFunction(value) || isVmConst(value, checkDeep);
}
