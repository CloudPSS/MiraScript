import { isVmAny, type VmAny } from './any.js';
import type { VmExtern } from './extern.js';
import type { VmImmutable } from './immutable.js';

/** Mirascript 虚拟机内的合法值 */
export type VmValue = VmImmutable | VmExtern;

/**
 * 检查是否为 Mirascript 合法值
 */
export function isVmValue(value: VmAny): value is VmValue;
/**
 * 检查是否为 Mirascript 合法值
 */
export function isVmValue(value: unknown, checkDeep: boolean): value is VmValue;
/**
 * 检查是否为 Mirascript 合法值
 */
export function isVmValue(value: unknown, checkDeep = false): value is VmValue {
    if (value === undefined) return false;
    return isVmAny(value, checkDeep);
}
