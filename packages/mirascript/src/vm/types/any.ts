import { isVmConst } from './const.js';
import { isVmFunction } from './function.js';
import { isVmWrapper } from './wrapper.js';
import type { VmValue } from './value.js';

/** Mirascript 虚拟机内的值（包括未初始化变量） */
export type VmAny = VmValue | VmUninitialized;

/** Mirascript 虚拟机内的未初始化变量 */
export type VmUninitialized = undefined;

/**
 * 检查是否为 Mirascript 值
 */
export function isVmAny(value: unknown, checkDeep: boolean): value is VmAny {
    switch (typeof value) {
        case 'string':
        case 'number':
        case 'boolean':
        case 'undefined':
            return true;
        case 'object':
            if (value == null) return true;
            if (isVmWrapper(value)) return true;
            return isVmConst(value, checkDeep);
        case 'function':
            return isVmFunction(value);
        case 'bigint':
        case 'symbol':
        default:
            return false; // Other types are not valid
    }
}
